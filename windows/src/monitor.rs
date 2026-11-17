//! Captarea de la un microfon real, pentru ascultarea de proba.
//!
//! Serveste la doua lucruri:
//!
//! 1. Te auzi pe tine cu vocea schimbata, in casti, inainte sa o folosesti
//!    intr-un joc sau o sedinta. Fara asta, alegi vocea pe orbite.
//! 2. Pana exista aplicatia de Android, orice microfon deja instalat in Windows
//!    poate fi sursa — inclusiv WO Mic. Asa poti incerca vocile azi.
//!
//! Dupa ce aplicatia de telefon va fi gata, sursa devine reteaua, iar modulul
//! asta ramane pentru ascultarea de proba.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat};

use crate::audio::AudioError;
use crate::proto::SAMPLE_RATE;

/// Cat sunet captat tinem in asteptare. Peste atat, aruncam ce e vechi:
/// la ascultarea de proba latenta mica este mai importanta decat integritatea.
const MAX_QUEUE_MS: usize = 60;
const MAX_QUEUE_SAMPLES: usize = SAMPLE_RATE as usize / 1000 * MAX_QUEUE_MS;

/// Coada dintre firul de captare si cel de redare.
pub struct MicQueue {
    samples: VecDeque<i16>,
}

impl MicQueue {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(MAX_QUEUE_SAMPLES),
        }
    }

    pub fn push(&mut self, sample: i16) {
        if self.samples.len() >= MAX_QUEUE_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Umple `output`; ce lipseste ramane liniste.
    pub fn fill(&mut self, output: &mut [i16]) -> bool {
        let available = self.samples.len().min(output.len());
        for slot in output[..available].iter_mut() {
            *slot = self.samples.pop_front().unwrap_or(0);
        }
        output[available..].fill(0);
        available == output.len()
    }
}

/// Reesantionare liniara catre 48 kHz, cand microfonul lucreaza la alta rata.
struct Resampler {
    /// Cate esantioane de intrare consumam pentru unul de iesire.
    step: f32,
    position: f32,
}

impl Resampler {
    fn new(input_rate: u32) -> Self {
        Self {
            step: input_rate as f32 / SAMPLE_RATE as f32,
            position: 0.0,
        }
    }

    fn process(&mut self, input: &[f32], mut emit: impl FnMut(f32)) {
        while self.position < input.len() as f32 {
            let index = self.position as usize;
            let fraction = self.position - index as f32;
            let current = input[index];
            let next = *input.get(index + 1).unwrap_or(&current);

            emit(current * (1.0 - fraction) + next * fraction);
            self.position += self.step;
        }
        self.position -= input.len() as f32;
    }
}

/// Media canalelor, apoi reesantionare, apoi in coada.
fn deliver(mono_block: &[f32], resampler: &mut Resampler, queue: &Mutex<MicQueue>) {
    let Ok(mut queue) = queue.lock() else { return };
    resampler.process(mono_block, |sample| {
        queue.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    });
}

pub struct Capture {
    _stream: cpal::Stream,
    pub device_name: String,
}

pub fn input_device_names() -> Vec<String> {
    cpal::default_host()
        .input_devices()
        .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
        .unwrap_or_default()
}

fn pick_device(requested: Option<&str>) -> Result<Device, AudioError> {
    let host = cpal::default_host();

    if let Some(needle) = requested {
        let needle = needle.to_lowercase();
        return host
            .input_devices()
            .map_err(|error| AudioError::Cpal(error.to_string()))?
            .find(|device| {
                device
                    .name()
                    .map(|name| name.to_lowercase().contains(&needle))
                    .unwrap_or(false)
            })
            .ok_or(AudioError::NoInputDevice);
    }

    host.default_input_device().ok_or(AudioError::NoInputDevice)
}

/// Porneste captarea. Esantioanele ajung in coada, in mono, la 48 kHz.
pub fn start_capture(
    requested_device: Option<&str>,
    queue: Arc<Mutex<MicQueue>>,
) -> Result<Capture, AudioError> {
    let device = pick_device(requested_device)?;
    let device_name = device.name().unwrap_or_else(|_| "necunoscut".to_string());

    let supported = device
        .default_input_config()
        .map_err(|error| AudioError::Cpal(error.to_string()))?;
    let channels = supported.channels() as usize;
    let mut resampler = Resampler::new(supported.sample_rate().0);
    let config = supported.config();

    let error_handler = |error| eprintln!("[microfon] eroare in flux: {error}");
    // Tampon reutilizat intre apeluri, ca sa nu alocam in bucla audio.
    let mut mono_block: Vec<f32> = Vec::new();

    let stream = match supported.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |input: &[f32], _: &cpal::InputCallbackInfo| {
                mono_block.clear();
                mono_block.extend(
                    input
                        .chunks(channels)
                        .map(|frame| frame.iter().sum::<f32>() / channels as f32),
                );
                deliver(&mono_block, &mut resampler, &queue);
            },
            error_handler,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |input: &[i16], _: &cpal::InputCallbackInfo| {
                mono_block.clear();
                mono_block.extend(input.chunks(channels).map(|frame| {
                    frame.iter().map(|&sample| sample as f32).sum::<f32>()
                        / (channels as f32 * i16::MAX as f32)
                }));
                deliver(&mono_block, &mut resampler, &queue);
            },
            error_handler,
            None,
        ),
        other => return Err(AudioError::UnsupportedFormat(other)),
    }
    .map_err(|error| AudioError::Cpal(error.to_string()))?;

    stream
        .play()
        .map_err(|error| AudioError::Cpal(error.to_string()))?;

    Ok(Capture {
        _stream: stream,
        device_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_what_it_has_and_pads_the_rest_with_silence() {
        // Arrange
        let mut queue = MicQueue::new();
        for sample in [100, 200, 300] {
            queue.push(sample);
        }
        let mut output = [-1i16; 5];

        // Act
        let complete = queue.fill(&mut output);

        // Assert
        assert!(!complete);
        assert_eq!(output, [100, 200, 300, 0, 0]);
    }

    #[test]
    fn drops_the_oldest_audio_instead_of_growing_without_limit() {
        // Arrange
        let mut queue = MicQueue::new();

        // Act
        for sample in 0..MAX_QUEUE_SAMPLES as i32 + 10 {
            queue.push(sample as i16);
        }

        // Assert
        assert_eq!(queue.samples.len(), MAX_QUEUE_SAMPLES);
        assert_eq!(queue.samples[0], 10, "primele esantioane au fost aruncate");
    }

    #[test]
    fn passes_audio_through_unchanged_when_the_rate_already_matches() {
        // Arrange
        let mut resampler = Resampler::new(SAMPLE_RATE);
        let input = [0.0, 0.25, 0.5, 0.75];
        let mut output = Vec::new();

        // Act
        resampler.process(&input, |sample| output.push(sample));

        // Assert
        assert_eq!(output, input.to_vec());
    }

    #[test]
    fn doubles_the_sample_count_when_the_microphone_runs_at_half_the_rate() {
        // Arrange: 24 kHz catre 48 kHz
        let mut resampler = Resampler::new(SAMPLE_RATE / 2);
        let input = [0.0, 1.0, 0.0, 1.0];
        let mut output = Vec::new();

        // Act
        resampler.process(&input, |sample| output.push(sample));

        // Assert
        assert_eq!(
            output.len(),
            8,
            "o rata la jumatate produce de doua ori mai multe"
        );
    }

    #[test]
    fn keeps_its_place_between_consecutive_blocks() {
        // Arrange
        let mut resampler = Resampler::new(SAMPLE_RATE / 2);
        let mut total = 0;

        // Act: doua blocuri consecutive, ca intr-un flux real
        for _ in 0..2 {
            resampler.process(&[0.0, 0.5, 1.0], |_| total += 1);
        }

        // Assert: 3 esantioane la 24 kHz = 6 la 48 kHz, de doua ori
        assert_eq!(total, 12, "pozitia nu trebuie resetata intre blocuri");
    }
}
