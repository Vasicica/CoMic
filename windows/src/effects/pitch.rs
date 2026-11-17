//! Deplasarea inaltimii vocii, prin linie de intarziere cu doua capete.
//!
//! Metoda clasica: citim din acelasi tampon cu doua capete care se misca mai
//! repede sau mai incet decat scrierea, si le amestecam cu o fereastra care
//! insumeaza exact 1. Rezultatul nu are intreruperi la trecerea de la un cap la
//! altul, iar costul este mic — important, pentru ca asta ruleaza in bucla audio.
//!
//! Limita de care trebuie stiut: metoda muta si formantii odata cu inaltimea.
//! Vocea coborata suna a voce incetinita, nu a alta persoana. Separarea
//! formantilor cere un motor spectral si vine intr-o versiune ulterioara;
//! caracterul vocilor il dau pana atunci celelalte efecte din lant.

use std::f32::consts::PI;

/// Lungimea ferestrei de amestec. Prea scurta suna metalic, prea lunga intarzie.
/// 1024 esantioane la 48 kHz = ~21 ms, compromisul obisnuit pentru voce.
const WINDOW_SAMPLES: usize = 1024;
/// Tamponul tine de doua ori fereastra, ca ambele capete sa aiba de unde citi.
const BUFFER_SAMPLES: usize = WINDOW_SAMPLES * 2;

pub struct PitchShifter {
    buffer: Vec<f32>,
    write_index: usize,
    /// Pozitia in fereastra, in [0, 1).
    phase: f32,
    /// 2.0 = o octava mai sus, 0.5 = o octava mai jos.
    ratio: f32,
}

impl PitchShifter {
    pub fn new(ratio: f32) -> Self {
        Self {
            buffer: vec![0.0; BUFFER_SAMPLES],
            write_index: 0,
            phase: 0.0,
            ratio: ratio.clamp(0.25, 4.0),
        }
    }

    /// Citeste din tampon la o distanta fractionara in urma capului de scriere.
    fn read_delayed(&self, delay: f32) -> f32 {
        let position = self.write_index as f32 - delay;
        let wrapped = position.rem_euclid(BUFFER_SAMPLES as f32);

        let lower = wrapped.floor() as usize;
        let upper = (lower + 1) % BUFFER_SAMPLES;
        let fraction = wrapped - lower as f32;

        self.buffer[lower] * (1.0 - fraction) + self.buffer[upper] * fraction
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.buffer[self.write_index] = input;
        self.write_index = (self.write_index + 1) % BUFFER_SAMPLES;

        // Semnul conteaza: ca sa urce inaltimea, capul de citire trebuie sa se
        // apropie de cel de scriere, adica intarzierea sa scada in timp.
        self.phase = (self.phase + (1.0 - self.ratio) / WINDOW_SAMPLES as f32).rem_euclid(1.0);
        let opposite_phase = (self.phase + 0.5).rem_euclid(1.0);

        // Ferestrele celor doua capete insumeaza exact 1, deci amplitudinea
        // ramane constanta oricat de des se schimba capul dominant.
        let weight = 0.5 * (1.0 - (2.0 * PI * self.phase).cos());

        let first = self.read_delayed(self.phase * WINDOW_SAMPLES as f32);
        let second = self.read_delayed(opposite_phase * WINDOW_SAMPLES as f32);

        first * weight + second * (1.0 - weight)
    }
}

/// Converteste semitonuri in raportul de inaltime cerut de `PitchShifter`.
pub fn semitones_to_ratio(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numara trecerile prin zero — masura simpla si suficienta a inaltimii.
    fn zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|pair| (pair[0] < 0.0) != (pair[1] < 0.0))
            .count()
    }

    fn sine(frequency: f32, length: usize) -> Vec<f32> {
        (0..length)
            .map(|index| (2.0 * PI * frequency * index as f32 / 48_000.0).sin())
            .collect()
    }

    fn process_all(shifter: &mut PitchShifter, input: &[f32]) -> Vec<f32> {
        input
            .iter()
            .map(|&sample| shifter.process_sample(sample))
            .collect()
    }

    #[test]
    fn an_octave_up_roughly_doubles_the_frequency() {
        // Arrange
        let input = sine(220.0, 24_000);
        let mut shifter = PitchShifter::new(2.0);

        // Act: a doua jumatate, dupa ce tamponul s-a umplut
        let output = process_all(&mut shifter, &input);
        let settled = &output[12_000..];

        // Assert
        let ratio = zero_crossings(settled) as f32 / zero_crossings(&input[12_000..]) as f32;
        assert!(
            (1.7..=2.3).contains(&ratio),
            "asteptam aproape dublu, am primit {ratio}"
        );
    }

    #[test]
    fn an_octave_down_roughly_halves_the_frequency() {
        let input = sine(440.0, 24_000);
        let mut shifter = PitchShifter::new(0.5);

        let output = process_all(&mut shifter, &input);
        let settled = &output[12_000..];

        let ratio = zero_crossings(settled) as f32 / zero_crossings(&input[12_000..]) as f32;
        assert!(
            (0.35..=0.65).contains(&ratio),
            "asteptam aproape jumatate, am primit {ratio}"
        );
    }

    #[test]
    fn keeps_the_amplitude_steady_while_the_window_crossfades() {
        // Arrange
        let input = sine(300.0, 24_000);
        let mut shifter = PitchShifter::new(0.75);

        // Act
        let output = process_all(&mut shifter, &input);

        // Assert: fara goluri sau varfuri la trecerea dintre capete
        let settled = &output[12_000..];
        let peak = settled.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));
        assert!(peak > 0.5, "semnalul nu trebuie sa se stinga, varf {peak}");
        assert!(
            peak <= 1.05,
            "semnalul nu trebuie sa sara peste intrare, varf {peak}"
        );
    }

    #[test]
    fn a_ratio_of_one_leaves_the_signal_essentially_untouched() {
        let input = sine(440.0, 24_000);
        let mut shifter = PitchShifter::new(1.0);

        let output = process_all(&mut shifter, &input);

        let crossings_in = zero_crossings(&input[12_000..]) as i32;
        let crossings_out = zero_crossings(&output[12_000..]) as i32;
        // O trecere diferenta vine din marginea ferestrei, nu din inaltime.
        assert!(
            (crossings_in - crossings_out).abs() <= 2,
            "inaltimea trebuie sa ramana aceeasi: {crossings_in} vs {crossings_out}"
        );
    }

    #[test]
    fn converts_semitones_to_the_expected_ratios() {
        assert!((semitones_to_ratio(12.0) - 2.0).abs() < 0.001);
        assert!((semitones_to_ratio(-12.0) - 0.5).abs() < 0.001);
        assert!((semitones_to_ratio(0.0) - 1.0).abs() < 0.001);
    }
}
