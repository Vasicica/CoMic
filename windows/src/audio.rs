//! Redarea in microfonul virtual prin WASAPI (via cpal).
//!
//! Pana in Faza 2 microfonul virtual este VB-Cable: scriem in difuzorul
//! "CABLE Input", iar aplicatiile (CS2, Discord, Chrome) aud "CABLE Output"
//! ca microfon. Cand apare driverul propriu, se schimba doar numele cautat aici.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};

use crate::effects::VoiceChain;
use crate::proto::SAMPLE_RATE;

/// Fragmentul de nume dupa care recunoastem microfonul virtual.
const VIRTUAL_DEVICE_HINT: &str = "CABLE Input";

pub struct Playback {
    _stream: cpal::Stream,
    pub device_name: String,
}

#[derive(Debug)]
pub enum AudioError {
    NoOutputDevice,
    NoInputDevice,
    UnsupportedFormat(SampleFormat),
    Cpal(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::NoOutputDevice => write!(f, "nu am gasit niciun dispozitiv de redare"),
            AudioError::NoInputDevice => write!(f, "nu am gasit niciun microfon"),
            AudioError::UnsupportedFormat(format) => {
                write!(f, "formatul {format:?} nu este suportat")
            }
            AudioError::Cpal(message) => write!(f, "eroare audio: {message}"),
        }
    }
}

/// Numele tuturor dispozitivelor de redare, pentru `--list-devices`.
pub fn output_device_names() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices()
        .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
        .unwrap_or_default()
}

/// Alege dispozitivul: cel cerut explicit, altfel VB-Cable, altfel cel implicit.
fn pick_device(requested: Option<&str>) -> Result<Device, AudioError> {
    let host = cpal::default_host();
    let devices: Vec<Device> = host
        .output_devices()
        .map_err(|error| AudioError::Cpal(error.to_string()))?
        .collect();

    let matches = |device: &Device, needle: &str| {
        device
            .name()
            .map(|name| name.to_lowercase().contains(&needle.to_lowercase()))
            .unwrap_or(false)
    };

    if let Some(needle) = requested {
        return devices
            .into_iter()
            .find(|device| matches(device, needle))
            .ok_or(AudioError::NoOutputDevice);
    }

    let virtual_device = devices
        .into_iter()
        .find(|device| matches(device, VIRTUAL_DEVICE_HINT));
    if let Some(device) = virtual_device {
        return Ok(device);
    }

    host.default_output_device()
        .ok_or(AudioError::NoOutputDevice)
}

/// Porneste redarea continua.
///
/// `fill_mono` este sursa sunetului si nu stie nimic despre redare: poate fi
/// buffer-ul de jitter (sunet de pe telefon) sau coada microfonului (ascultare
/// de proba). Asa, acelasi lant de redare si de efecte serveste ambele cazuri.
pub fn start(
    requested_device: Option<&str>,
    gain: f32,
    mut voice: Option<VoiceChain>,
    mut fill_mono: impl FnMut(&mut [i16]) + Send + 'static,
) -> Result<Playback, AudioError> {
    let device = pick_device(requested_device)?;
    let device_name = device.name().unwrap_or_else(|_| "necunoscut".to_string());

    let default_config = device
        .default_output_config()
        .map_err(|error| AudioError::Cpal(error.to_string()))?;
    let sample_format = default_config.sample_format();
    let channels = default_config.channels();

    let config = StreamConfig {
        channels,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    if sample_format != SampleFormat::F32 {
        return Err(AudioError::UnsupportedFormat(sample_format));
    }

    // Tampon reutilizat intre apeluri, ca sa nu alocam in callback-ul audio.
    let mut mono = vec![0i16; 0];
    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let frames = output.len() / channels as usize;
                if mono.len() != frames {
                    mono.resize(frames, 0);
                }

                fill_mono(&mut mono);

                if let Some(voice) = voice.as_mut() {
                    voice.process(&mut mono);
                }

                for (frame_index, frame) in output.chunks_mut(channels as usize).enumerate() {
                    let sample = (mono[frame_index] as f32 / i16::MAX as f32) * gain;
                    frame.fill(sample.clamp(-1.0, 1.0));
                }
            },
            |error| eprintln!("[audio] eroare in flux: {error}"),
            None,
        )
        .map_err(|error| AudioError::Cpal(error.to_string()))?;

    stream
        .play()
        .map_err(|error| AudioError::Cpal(error.to_string()))?;

    Ok(Playback {
        _stream: stream,
        device_name,
    })
}
