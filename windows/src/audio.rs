//! Redarea in microfonul virtual prin WASAPI (via cpal).
//!
//! Pana in Faza 2 microfonul virtual este VB-Cable: scriem in difuzorul
//! "CABLE Input", iar aplicatiile (CS2, Discord, Chrome) aud "CABLE Output"
//! ca microfon. Cand apare driverul propriu, se schimba doar numele cautat aici.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};

use crate::effects::VoiceChain;
use crate::proto::SAMPLE_RATE;

/// Numele clasic al capatului de scriere, cand Windows nu l-a redenumit.
const CLASSIC_HINT: &str = "cable input";

/// Producatorul driverului. Partea asta a numelui vine din driver, nu din
/// capat, deci ramane si cand Windows pune peste capat numele generic in
/// limba sistemului ("Динамики", "Speakers", "Lautsprecher").
const VENDOR_HINT: &str = "vb-audio";

/// Capatul cu saisprezece canale, pe care nu vrem sa nimerim: are perechea
/// lui de ascultare, pe care nu asculta nimeni din intamplare.
const MULTI_CHANNEL_HINT: &str = "16 ch";

/// Ce se intampla cu sunetul inainte sa ajunga la placa, tinut in spatele unui
/// lacat ca sa poata fi schimbat *in timp ce* sunetul curge. Fara asta, orice
/// schimbare de voce ar cere oprirea si repornirea fluxului, adica o pauza
/// auzita si o reconectare inutila.
pub struct Processing {
    pub voice: Option<VoiceChain>,
    pub gain: f32,
}

impl Processing {
    pub fn new(voice: Option<VoiceChain>, gain: f32) -> Self {
        Self { voice, gain }
    }
}

pub struct Playback {
    _stream: cpal::Stream,
    pub device_name: String,
    /// Microfonul virtual nu exista deloc pe acest calculator, asa ca sunetul
    /// a plecat in ce era la indemana: difuzoarele.
    ///
    /// E cea mai inselatoare pana din tot programul. Omul isi aude vocea din
    /// boxe, deci crede ca merge; dar Discord, CS2 si restul nu aud nimic,
    /// fiindca nu exista microfonul in care sa asculte. Fara campul asta,
    /// programul tace exact cand ar trebui sa vorbeasca.
    ///
    /// Fals si cand omul a ales el difuzoarele din lista: atunci alegerea e a
    /// lui, iar numele dispozitivului se vede oricum in bara de stare.
    pub virtual_mic_missing: bool,
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

/// Numele acesta este capatul de scriere al unui cablu virtual?
///
/// Nu cautam un singur nume, fiindca numele nu tine. Pe un Windows in rusa,
/// capatul de scriere al VB-Cable se cheama "Динамики (2- VB-Audio Virtual
/// Cable)": cuvintele "CABLE Input" nu apar nicaieri, desi driverul e acelasi.
/// Windows lasa capatul sa fie redenumit, si atunci ii pune numele generic
/// in limba sistemului. Ce ramane in nume e producatorul driverului, fiindca
/// acela vine din driver, nu din capat.
pub fn is_virtual_mic(name: &str) -> bool {
    let name = name.to_lowercase();
    name.contains(CLASSIC_HINT) || name.contains(VENDOR_HINT)
}

/// Dintre capetele virtuale, acesta e cel pe care asculta aplicatiile?
///
/// VB-Cable scoate si un capat cu saisprezece canale, care isi are perechea
/// lui de ascultare. Daca am scrie acolo, sunetul ar pleca intr-un capat pe
/// care nu asculta nimeni: tot tacere, dar cu alt motiv.
fn is_plain_virtual_mic(name: &str) -> bool {
    is_virtual_mic(name) && !name.to_lowercase().contains(MULTI_CHANNEL_HINT)
}

/// Numele capatului virtual pe care l-am alege, daca exista vreunul.
///
/// Un singur loc hotaraste care e, ca fereastra sa arate in lista exact ce va
/// folosi programul. Cat timp raspunsul statea in doua locuri, unul in Rust si
/// unul in interfata, ele se puteau desparti fara sa bage nimeni de seama.
pub fn virtual_mic_name() -> Option<String> {
    pick_virtual_mic(&output_device_names())
}

/// Alegerea propriu-zisa, despartita de placa de sunet ca sa poata fi probata.
fn pick_virtual_mic(names: &[String]) -> Option<String> {
    names
        .iter()
        .find(|name| is_plain_virtual_mic(name))
        .or_else(|| names.iter().find(|name| is_virtual_mic(name)))
        .cloned()
}

/// Exista microfonul virtual pe acest calculator?
///
/// Fereastra intreaba inainte sa apesi Conecteaza, ca sa poata spune din timp
/// ca lipseste, nu dupa ce omul a vorbit degeaba un minut intreg.
pub fn has_virtual_mic() -> bool {
    virtual_mic_name().is_some()
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

    if let Some(wanted) = virtual_mic_name() {
        if let Some(device) = devices.into_iter().find(|device| matches(device, &wanted)) {
            return Ok(device);
        }
        // Numele a disparut intre cele doua enumerari: driverul a fost scos
        // chiar acum. Nu ne oprim cu o eroare; cadem pe cel implicit, iar
        // fereastra spune ca sunetul nu ajunge unde trebuie.
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
    processing: Arc<Mutex<Processing>>,
    mut fill_mono: impl FnMut(&mut [i16]) + Send + 'static,
) -> Result<Playback, AudioError> {
    let device = pick_device(requested_device)?;
    let device_name = device.name().unwrap_or_else(|_| "necunoscut".to_string());

    // Daca am nimerit chiar microfonul virtual, el exista si nu mai cautam.
    // Cautam doar cand am ajuns in alta parte, ca sa deosebim "VB-Cable nu e
    // instalat" de "omul a ales el difuzoarele". Primul cere un mesaj; al
    // doilea nu, fiindca e alegerea lui.
    let virtual_mic_missing = !is_virtual_mic(&device_name) && !has_virtual_mic();

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

                // Daca lacatul e ocupat o clipa (interfata schimba vocea chiar
                // acum), lasam sunetul sa treaca neprelucrat: o clipa de voce
                // naturala e mai buna decat o intrerupere.
                let gain = match processing.lock() {
                    Ok(mut processing) => {
                        if let Some(voice) = processing.voice.as_mut() {
                            voice.process(&mut mono);
                        }
                        processing.gain
                    }
                    Err(_) => 1.0,
                };

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
        virtual_mic_missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_classic_name_whatever_the_letter_case() {
        assert!(is_virtual_mic("CABLE Input (VB-Audio Virtual Cable)"));
        assert!(is_virtual_mic("cable input (vb-audio virtual cable)"));
    }

    /// Numele adevarat de pe calculatorul pe care s-a scris asta: un Windows
    /// in rusa, unde capatul a fost redenumit in "Динамики" (difuzoare).
    /// Cuvintele "CABLE Input" nu apar nicaieri, desi driverul e acelasi.
    /// Cat timp cautam numai dupa ele, programul credea ca VB-Cable nu e
    /// instalat si trimitea sunetul in boxe, fara sa spuna nimic.
    #[test]
    fn recognises_the_writing_end_after_windows_renamed_it() {
        assert!(is_virtual_mic("Динамики (2- VB-Audio Virtual Cable)"));
        assert!(is_virtual_mic("Speakers (2- VB-Audio Virtual Cable)"));
        assert!(is_virtual_mic("Lautsprecher (2- VB-Audio Virtual Cable)"));
    }

    #[test]
    fn does_not_mistake_real_speakers_for_the_virtual_mic() {
        assert!(!is_virtual_mic("Динамики (High Definition Audio Device)"));
        assert!(!is_virtual_mic("Speakers (Realtek(R) Audio)"));
        assert!(!is_virtual_mic("Headphones (2- USB Audio Device)"));
    }

    /// Capatul cu saisprezece canale e tot un cablu virtual, deci lipsa lui nu
    /// e o pana; dar nu e cel pe care asculta aplicatiile, deci nu il alegem
    /// cat timp exista celalalt.
    #[test]
    fn knows_the_sixteen_channel_end_is_not_the_one_apps_listen_on() {
        let saisprezece = "CABLE In 16 Ch (2- VB-Audio Virtual Cable)";
        assert!(is_virtual_mic(saisprezece));
        assert!(!is_plain_virtual_mic(saisprezece));
        assert!(is_plain_virtual_mic("Динамики (2- VB-Audio Virtual Cable)"));
    }

    fn nume(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|s| s.to_string()).collect()
    }

    /// Lista adevarata de pe calculatorul pe care s-a scris asta, in ordinea
    /// in care o da Windows: capatul cu saisprezece canale sta inaintea celui
    /// bun. O cautare "primul care seamana" ar nimeri gresit.
    #[test]
    fn prefers_the_plain_end_even_when_the_other_comes_first() {
        let gasit = pick_virtual_mic(&nume(&[
            "CABLE In 16 Ch (2- VB-Audio Virtual Cable)",
            "Динамики (2- VB-Audio Virtual Cable)",
            "Динамики (High Definition Audio Device)",
        ]));
        assert_eq!(gasit.as_deref(), Some("Динамики (2- VB-Audio Virtual Cable)"));
    }

    /// Daca exista numai capatul cu saisprezece canale, il luam pe el: tot e
    /// mai aproape de adevar decat difuzoarele, si nu spunem ca lipseste ceva
    /// ce este instalat.
    #[test]
    fn falls_back_to_the_sixteen_channel_end_when_it_is_all_there_is() {
        let gasit = pick_virtual_mic(&nume(&[
            "Динамики (High Definition Audio Device)",
            "CABLE In 16 Ch (2- VB-Audio Virtual Cable)",
        ]));
        assert_eq!(
            gasit.as_deref(),
            Some("CABLE In 16 Ch (2- VB-Audio Virtual Cable)")
        );
    }

    #[test]
    fn finds_nothing_when_no_virtual_cable_is_installed() {
        let gasit = pick_virtual_mic(&nume(&[
            "Динамики (High Definition Audio Device)",
            "Headphones (2- USB Audio Device)",
        ]));
        assert_eq!(gasit, None);
    }
}
