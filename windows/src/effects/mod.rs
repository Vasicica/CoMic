//! Lantul de efecte care transforma vocea.
//!
//! Ordinea conteaza si nu este intamplatoare:
//!   inaltime -> taiere jos -> taiere sus -> filtrele vocii -> banda
//!   -> modulatie inelara -> tremolo -> soapta -> distorsiune -> ecou
//!   -> volum -> amestec cu vocea originala
//!
//! Distorsiunea sta spre final pentru ca trebuie sa primeasca semnalul deja
//! filtrat; invers, ar aspri si ce urmeaza sa fie taiat oricum. Ecoul vine
//! dupa distorsiune, ca repetarile sa nu fie distorsionate a doua oara.
//! Amestecul cu
//! vocea originala vine la urma, ca sa poti doza cat de expus este efectul.

pub mod biquad;
pub mod pitch;
pub mod voices;

use std::f32::consts::PI;

use biquad::{Biquad, GENTLE_Q};
use pitch::{semitones_to_ratio, PitchShifter};
pub use voices::{find_preset, VoicePreset, PRESETS};

use serde::{Deserialize, Serialize};

use crate::proto::SAMPLE_RATE;

/// Reglajele utilizatorului, asezate peste setarile vocii alese.
/// `None` inseamna "lasa cum e in voce".
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSettings {
    /// Cat de expus este efectul: 0.0 = numai vocea ta, 1.0 = numai efectul.
    pub mix: Option<f32>,
    /// Taie sub aceasta frecventa (bubuit, vant, zgomot de masa).
    pub low_cut_hz: Option<f32>,
    /// Taie peste aceasta frecventa (sasait, ascutis).
    pub high_cut_hz: Option<f32>,
    /// Inlocuieste deplasarea de inaltime a vocii.
    pub semitones: Option<f32>,
    /// Inlocuieste distorsiunea vocii.
    pub drive: Option<f32>,
    /// Inlocuieste adancimea tremurului, 0..1. 0 = fara tremur.
    pub wobble: Option<f32>,
    /// Inlocuieste modulatia inelara, in Hz. 0 = fara sunet metalic.
    pub metal_hz: Option<f32>,
}

/// Viteza tremurului cand vocea nu are unul al ei, dar utilizatorul cere.
const DEFAULT_WOBBLE_RATE_HZ: f32 = 5.5;

/// Linie de intarziere cu reintoarcere: fiecare repetare e mai slaba.
struct Echo {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    wet: f32,
}

impl Echo {
    fn new(delay_ms: f32, feedback: f32, wet: f32) -> Self {
        let samples = ((delay_ms.clamp(1.0, 1_000.0) / 1000.0) * SAMPLE_RATE as f32) as usize;
        Self {
            buffer: vec![0.0; samples.max(1)],
            index: 0,
            // Peste 0.9 ecoul nu se mai stinge si suna a microfonie.
            feedback: feedback.clamp(0.0, 0.9),
            wet: wet.clamp(0.0, 1.0),
        }
    }

    fn process_sample(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        self.buffer[self.index] = input + delayed * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        input + delayed * self.wet
    }
}

/// Soapta: inlocuieste tonul vocii cu zgomot care urmareste cat de tare vorbesti.
/// Zgomotul trece printr-un filtru pe zona consoanelor, ca sa sune a respiratie.
struct Whisper {
    amount: f32,
    envelope: f32,
    seed: u32,
    filter: Biquad,
}

impl Whisper {
    fn new(amount: f32) -> Self {
        Self {
            amount: amount.clamp(0.0, 1.0),
            envelope: 0.0,
            seed: 0x9E37_79B9,
            filter: Biquad::band_pass(2_500.0, 0.6),
        }
    }

    /// Zgomot alb determinist (xorshift): fara dependinte, acelasi rezultat in teste.
    fn noise(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn process_sample(&mut self, input: f32) -> f32 {
        // Urca repede (~1 ms) si coboara lent (~30 ms), ca un VU-metru.
        let level = input.abs();
        let speed = if level > self.envelope { 0.02 } else { 0.0007 };
        self.envelope += (level - self.envelope) * speed;

        let noise = self.noise();
        let breath = self.filter.process_sample(noise) * self.envelope * 4.0;
        input * (1.0 - self.amount) + breath * self.amount
    }
}

/// Oscilator folosit de modulatia inelara si de tremolo.
struct Oscillator {
    phase: f32,
    step: f32,
}

impl Oscillator {
    fn new(frequency_hz: f32) -> Self {
        Self {
            phase: 0.0,
            step: 2.0 * PI * frequency_hz / SAMPLE_RATE as f32,
        }
    }

    fn next_value(&mut self) -> f32 {
        let value = self.phase.sin();
        self.phase = (self.phase + self.step) % (2.0 * PI);
        value
    }
}

pub struct VoiceChain {
    shifter: Option<PitchShifter>,
    low_cut: Option<Biquad>,
    high_cut: Option<Biquad>,
    preset_high_pass: Option<Biquad>,
    preset_low_pass: Option<Biquad>,
    bandpass: Option<Biquad>,
    ring: Option<Oscillator>,
    tremolo: Option<(Oscillator, f32)>,
    whisper: Option<Whisper>,
    drive: f32,
    echo: Option<Echo>,
    gain: f32,
    mix: f32,
}

impl VoiceChain {
    pub fn new(preset: &VoicePreset, settings: VoiceSettings) -> Self {
        let semitones = settings.semitones.unwrap_or(preset.semitones);
        let drive = settings.drive.unwrap_or(preset.drive).max(0.0);
        let ring_hz = settings.metal_hz.or(preset.ring_hz).unwrap_or(0.0);
        let (wobble_rate, preset_depth) = preset.tremolo.unwrap_or((DEFAULT_WOBBLE_RATE_HZ, 0.0));
        let wobble_depth = settings.wobble.unwrap_or(preset_depth).clamp(0.0, 1.0);

        Self {
            shifter: (semitones != 0.0).then(|| PitchShifter::new(semitones_to_ratio(semitones))),
            low_cut: settings
                .low_cut_hz
                .map(|frequency| Biquad::high_pass(frequency, GENTLE_Q)),
            high_cut: settings
                .high_cut_hz
                .map(|frequency| Biquad::low_pass(frequency, GENTLE_Q)),
            preset_high_pass: preset
                .high_pass
                .map(|frequency| Biquad::high_pass(frequency, GENTLE_Q)),
            preset_low_pass: preset
                .low_pass
                .map(|frequency| Biquad::low_pass(frequency, GENTLE_Q)),
            bandpass: preset
                .bandpass
                .map(|(centre, q)| Biquad::band_pass(centre, q)),
            ring: (ring_hz > 0.0).then(|| Oscillator::new(ring_hz)),
            tremolo: (wobble_depth > 0.0).then(|| (Oscillator::new(wobble_rate), wobble_depth)),
            whisper: (preset.whisper > 0.0).then(|| Whisper::new(preset.whisper)),
            drive,
            echo: preset
                .echo
                .map(|(delay_ms, feedback, wet)| Echo::new(delay_ms, feedback, wet)),
            gain: preset.gain,
            mix: settings.mix.unwrap_or(1.0).clamp(0.0, 1.0),
        }
    }

    /// Transforma esantioanele pe loc. Lucram in f32 si revenim la i16 la final,
    /// ca sa nu pierdem precizie intre etape.
    pub fn process(&mut self, samples: &mut [i16]) {
        for sample in samples.iter_mut() {
            let dry = *sample as f32 / i16::MAX as f32;
            let mut wet = dry;

            if let Some(shifter) = self.shifter.as_mut() {
                wet = shifter.process_sample(wet);
            }
            if let Some(low_cut) = self.low_cut.as_mut() {
                wet = low_cut.process_sample(wet);
            }
            if let Some(high_cut) = self.high_cut.as_mut() {
                wet = high_cut.process_sample(wet);
            }
            if let Some(filter) = self.preset_high_pass.as_mut() {
                wet = filter.process_sample(wet);
            }
            if let Some(filter) = self.preset_low_pass.as_mut() {
                wet = filter.process_sample(wet);
            }
            if let Some(bandpass) = self.bandpass.as_mut() {
                wet = bandpass.process_sample(wet);
            }
            if let Some(ring) = self.ring.as_mut() {
                wet *= ring.next_value();
            }
            if let Some((oscillator, depth)) = self.tremolo.as_mut() {
                let modulation = 1.0 - *depth * (1.0 - oscillator.next_value()) * 0.5;
                wet *= modulation;
            }
            if let Some(whisper) = self.whisper.as_mut() {
                wet = whisper.process_sample(wet);
            }
            if self.drive > 0.0 {
                // tanh satureaza lin: nu taie varfurile, le rotunjeste.
                wet = (wet * self.drive).tanh() / self.drive.tanh();
            }
            if let Some(echo) = self.echo.as_mut() {
                wet = echo.process_sample(wet);
            }
            wet *= self.gain;

            let mut mixed = wet * self.mix + dry * (1.0 - self.mix);
            if !mixed.is_finite() {
                // Un filtru instabil nu are voie sa ajunga in casti ca zgomot maxim.
                mixed = 0.0;
            }
            *sample = (mixed.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frequency: f32, length: usize) -> Vec<i16> {
        (0..length)
            .map(|index| {
                let phase = 2.0 * PI * frequency * index as f32 / SAMPLE_RATE as f32;
                (phase.sin() * 16_000.0) as i16
            })
            .collect()
    }

    /// Varful in i32: comparatiile dintre varfuri depasesc usor gama i16.
    fn peak(samples: &[i16]) -> i32 {
        samples
            .iter()
            .map(|sample| (*sample as i32).abs())
            .max()
            .unwrap_or(0)
    }

    fn chain_for(id: &str) -> VoiceChain {
        VoiceChain::new(
            find_preset(id).expect("vocea trebuie sa existe"),
            VoiceSettings::default(),
        )
    }

    #[test]
    fn the_radio_voice_stays_loud_without_clipping_into_distortion_artefacts() {
        // Arrange
        let mut samples = tone(1_000.0, 9_600);

        // Act
        chain_for("radio").process(&mut samples);

        // Assert
        let settled = &samples[4_800..];
        assert!(peak(settled) > 10_000, "vocea trebuie sa ramana puternica");
        assert!(
            peak(settled) <= i16::MAX as i32,
            "nu avem voie sa depasim gama esantionului"
        );
    }

    #[test]
    fn the_robot_voice_changes_the_signal_without_silencing_it() {
        // Arrange
        let original = tone(300.0, 9_600);
        let mut processed = original.clone();

        // Act
        chain_for("robot").process(&mut processed);

        // Assert
        assert_ne!(processed, original, "modulatia trebuie sa schimbe semnalul");
        assert!(peak(&processed) > 2_000, "semnalul nu trebuie sa dispara");
    }

    #[test]
    fn the_old_man_voice_makes_the_volume_wobble() {
        // Arrange
        let mut samples = tone(300.0, 48_000);

        // Act
        chain_for("mosneag").process(&mut samples);

        // Assert: comparam doua sferturi de secunda aflate in faze diferite
        let first = peak(&samples[12_000..16_000]);
        let second = peak(&samples[16_000..20_000]);
        let difference = (first - second).abs();
        assert!(
            difference > 300,
            "tremolo-ul trebuie sa se auda, diferenta {difference}"
        );
    }

    #[test]
    fn a_mix_of_zero_leaves_the_voice_exactly_as_it_was() {
        // Arrange
        let original = tone(400.0, 9_600);
        let mut processed = original.clone();
        let settings = VoiceSettings {
            mix: Some(0.0),
            ..VoiceSettings::default()
        };

        // Act
        VoiceChain::new(find_preset("demon").unwrap(), settings).process(&mut processed);

        // Assert: diferentele de rotunjire sunt de cel mult un pas de cuantizare
        let largest_difference = original
            .iter()
            .zip(&processed)
            .map(|(before, after)| (*before as i32 - *after as i32).abs())
            .max()
            .unwrap_or(0);
        assert!(
            largest_difference <= 1,
            "cu mix 0 vocea ramane neatinsa, diferenta {largest_difference}"
        );
    }

    #[test]
    fn a_half_mix_lands_between_the_original_and_the_full_effect() {
        // Arrange
        let input = tone(400.0, 9_600);
        let with_mix = |mix: f32| {
            let mut samples = input.clone();
            let settings = VoiceSettings {
                mix: Some(mix),
                ..VoiceSettings::default()
            };
            VoiceChain::new(find_preset("megafon").unwrap(), settings).process(&mut samples);
            samples
        };

        // Act
        let half = with_mix(0.5);
        let full = with_mix(1.0);

        // Assert
        assert_ne!(half, input, "la jumatate efectul se aude");
        assert_ne!(half, full, "dar nu la fel de tare ca la maxim");
    }

    #[test]
    fn a_low_cut_removes_the_rumble_the_preset_would_have_kept() {
        // Arrange
        let mut rumble = tone(60.0, 9_600);
        let settings = VoiceSettings {
            low_cut_hz: Some(300.0),
            ..VoiceSettings::default()
        };

        // Act
        VoiceChain::new(find_preset("normal").unwrap(), settings).process(&mut rumble);

        // Assert
        let remaining = peak(&rumble[4_800..]);
        assert!(
            remaining < 2_000,
            "bubuitul de 60 Hz trebuie sa dispara, a ramas {remaining}"
        );
    }

    #[test]
    fn a_high_cut_removes_the_hiss_the_preset_would_have_kept() {
        // Arrange
        let mut hiss = tone(12_000.0, 9_600);
        let settings = VoiceSettings {
            high_cut_hz: Some(3_000.0),
            ..VoiceSettings::default()
        };

        // Act
        VoiceChain::new(find_preset("normal").unwrap(), settings).process(&mut hiss);

        // Assert
        let remaining = peak(&hiss[4_800..]);
        assert!(
            remaining < 2_000,
            "sasaitul de 12 kHz trebuie sa dispara, a ramas {remaining}"
        );
    }

    #[test]
    fn overriding_the_pitch_changes_the_result() {
        // Arrange
        let input = tone(440.0, 24_000);
        let mut as_preset = input.clone();
        let mut lowered = input.clone();
        let settings = VoiceSettings {
            semitones: Some(-12.0),
            ..VoiceSettings::default()
        };

        // Act
        chain_for("normal").process(&mut as_preset);
        VoiceChain::new(find_preset("normal").unwrap(), settings).process(&mut lowered);

        // Assert
        assert_ne!(
            as_preset, lowered,
            "reglajul de inaltime trebuie sa conteze"
        );
    }

    #[test]
    fn no_voice_ever_overflows_the_sample_range() {
        // Arrange: semnal aproape de maxim, cazul in care distorsiunea ar putea sari
        let loud: Vec<i16> = tone(250.0, 9_600)
            .iter()
            .map(|sample| sample.saturating_mul(2))
            .collect();

        for preset in PRESETS {
            // Act
            let mut samples = loud.clone();
            VoiceChain::new(preset, VoiceSettings::default()).process(&mut samples);

            // Assert
            assert!(
                samples.iter().all(|&sample| sample != i16::MIN),
                "{} produce esantioane la limita",
                preset.id
            );
        }
    }

    #[test]
    fn the_cave_voice_repeats_a_click_after_the_echo_delay() {
        // Arrange: un singur clic, apoi liniste
        let mut samples = vec![0i16; 48_000];
        samples[0] = 16_000;

        // Act
        chain_for("pestera").process(&mut samples);

        // Assert: la 220 ms (10 560 esantioane) clicul se aude din nou
        let echo = peak(&samples[10_400..10_800]);
        let before = peak(&samples[2_000..10_000]);
        assert!(echo > 1_000, "ecoul trebuie sa se auda, varf {echo}");
        assert!(echo > before * 4, "intre clic si ecou e liniste");
    }

    #[test]
    fn the_whisper_voice_replaces_the_tone_but_follows_the_loudness() {
        // Arrange
        let mut speech = tone(200.0, 24_000);
        let mut silence = vec![0i16; 24_000];

        // Act
        chain_for("soapta").process(&mut speech);
        chain_for("soapta").process(&mut silence);

        // Assert
        assert!(peak(&speech[12_000..]) > 1_000, "soapta trebuie sa se auda");
        assert!(peak(&silence) < 50, "fara voce, soapta tace");
    }

    #[test]
    fn the_wobble_setting_adds_a_tremble_to_a_voice_that_had_none() {
        // Arrange
        let mut samples = tone(300.0, 48_000);
        let settings = VoiceSettings {
            wobble: Some(0.6),
            ..VoiceSettings::default()
        };

        // Act
        VoiceChain::new(find_preset("normal").unwrap(), settings).process(&mut samples);

        // Assert
        let first = peak(&samples[12_000..16_000]);
        let second = peak(&samples[16_000..20_000]);
        assert!(
            (first - second).abs() > 1_000,
            "tremurul trebuie sa se auda"
        );
    }

    #[test]
    fn a_metal_setting_of_zero_turns_the_robot_back_into_a_clean_voice() {
        // Arrange
        let input = tone(300.0, 9_600);
        let mut robot = input.clone();
        let mut no_metal = input.clone();
        let settings = VoiceSettings {
            metal_hz: Some(0.0),
            drive: Some(0.0),
            ..VoiceSettings::default()
        };

        // Act
        chain_for("robot").process(&mut robot);
        VoiceChain::new(find_preset("robot").unwrap(), settings).process(&mut no_metal);

        // Assert
        assert_ne!(robot, input, "robotul normal schimba vocea");
        assert_eq!(no_metal, input, "fara metal si distorsiune ramane vocea ta");
    }

    #[test]
    fn extreme_settings_never_produce_noise_bursts_or_crash() {
        // Arrange: tot la maxim, pe un semnal saturat
        let loud: Vec<i16> = (0..9_600)
            .map(|index| if index % 2 == 0 { i16::MAX } else { i16::MIN })
            .collect();
        let settings = VoiceSettings {
            mix: Some(1.0),
            low_cut_hz: Some(20_000.0),
            high_cut_hz: Some(20.0),
            semitones: Some(12.0),
            drive: Some(10.0),
            wobble: Some(1.0),
            metal_hz: Some(200.0),
        };

        for preset in PRESETS {
            // Act
            let mut samples = loud.clone();
            VoiceChain::new(preset, settings).process(&mut samples);

            // Assert
            assert!(
                samples.iter().all(|&sample| sample != i16::MIN),
                "{} iese din gama",
                preset.id
            );
        }
    }

    #[test]
    fn every_catalogued_voice_produces_audible_output() {
        let input = tone(220.0, 24_000);

        for preset in PRESETS {
            let mut samples = input.clone();
            VoiceChain::new(preset, VoiceSettings::default()).process(&mut samples);

            // Jumatatea a doua, dupa ce liniile de intarziere s-au umplut.
            assert!(
                peak(&samples[12_000..]) > 1_000,
                "{} nu produce sunet audibil",
                preset.id
            );
        }
    }
}
