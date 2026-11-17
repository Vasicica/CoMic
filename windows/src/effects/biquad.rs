//! Filtre biquad de ordinul doi, dupa formulele standard RBJ.
//!
//! Trei forme ne ajung pentru voce:
//! - `high_pass` taie de jos (bubuit, vant, zgomot de masa)
//! - `low_pass` taie de sus (sasait, ascutis obositor)
//! - `band_pass` lasa doar mijlocul (sunetul de statie radio)

use std::f32::consts::PI;

use crate::proto::SAMPLE_RATE;

/// Factorul Q folosit pentru taierile de la capete. 0.707 este Butterworth:
/// cel mai drept raspuns posibil, fara cocoasa la frecventa de taiere.
pub const GENTLE_Q: f32 = 0.707;

pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

/// Marimile comune tuturor celor trei forme.
struct Terms {
    cos_omega: f32,
    alpha: f32,
    a0: f32,
}

fn terms(frequency_hz: f32, q: f32) -> Terms {
    // Peste jumatate din rata de esantionare filtrul nu mai are sens, asa ca
    // marginim frecventa inainte sa calculam coeficientii.
    let frequency = frequency_hz.clamp(20.0, SAMPLE_RATE as f32 * 0.45);
    let omega = 2.0 * PI * frequency / SAMPLE_RATE as f32;
    let alpha = omega.sin() / (2.0 * q.max(0.1));

    Terms {
        cos_omega: omega.cos(),
        alpha,
        a0: 1.0 + alpha,
    }
}

impl Biquad {
    fn from_numerator(b0: f32, b1: f32, b2: f32, terms: &Terms) -> Self {
        Self {
            b0: b0 / terms.a0,
            b1: b1 / terms.a0,
            b2: b2 / terms.a0,
            a1: -2.0 * terms.cos_omega / terms.a0,
            a2: (1.0 - terms.alpha) / terms.a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn low_pass(frequency_hz: f32, q: f32) -> Self {
        let terms = terms(frequency_hz, q);
        let shared = (1.0 - terms.cos_omega) / 2.0;
        Self::from_numerator(shared, 1.0 - terms.cos_omega, shared, &terms)
    }

    pub fn high_pass(frequency_hz: f32, q: f32) -> Self {
        let terms = terms(frequency_hz, q);
        let shared = (1.0 + terms.cos_omega) / 2.0;
        Self::from_numerator(shared, -(1.0 + terms.cos_omega), shared, &terms)
    }

    pub fn band_pass(frequency_hz: f32, q: f32) -> Self {
        let terms = terms(frequency_hz, q);
        Self::from_numerator(terms.alpha, 0.0, -terms.alpha, &terms)
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Amplitudinea de varf a unei sinusoide trecute prin filtru, dupa ce
    /// raspunsul s-a stabilizat.
    fn response_at(filter: &mut Biquad, frequency: f32) -> f32 {
        let length = 9_600;
        let mut peak = 0.0_f32;

        for index in 0..length {
            let phase = 2.0 * PI * frequency * index as f32 / SAMPLE_RATE as f32;
            let output = filter.process_sample(phase.sin());
            if index > length / 2 {
                peak = peak.max(output.abs());
            }
        }

        peak
    }

    #[test]
    fn a_high_pass_removes_the_low_end_and_keeps_the_voice() {
        // Arrange & Act
        let low = response_at(&mut Biquad::high_pass(300.0, GENTLE_Q), 60.0);
        let voice = response_at(&mut Biquad::high_pass(300.0, GENTLE_Q), 1_500.0);

        // Assert
        assert!(low < 0.1, "60 Hz trebuie taiat, am primit {low}");
        assert!(voice > 0.9, "1500 Hz trebuie pastrat, am primit {voice}");
    }

    #[test]
    fn a_low_pass_removes_the_high_end_and_keeps_the_voice() {
        let high = response_at(&mut Biquad::low_pass(3_000.0, GENTLE_Q), 12_000.0);
        let voice = response_at(&mut Biquad::low_pass(3_000.0, GENTLE_Q), 500.0);

        assert!(high < 0.1, "12 kHz trebuie taiat, am primit {high}");
        assert!(voice > 0.9, "500 Hz trebuie pastrat, am primit {voice}");
    }

    #[test]
    fn a_band_pass_keeps_its_centre_and_cuts_both_sides() {
        let centre = response_at(&mut Biquad::band_pass(1_000.0, 1.0), 1_000.0);
        let below = response_at(&mut Biquad::band_pass(1_000.0, 1.0), 100.0);
        let above = response_at(&mut Biquad::band_pass(1_000.0, 1.0), 10_000.0);

        assert!(centre > 0.9, "centrul trece, am primit {centre}");
        assert!(below < 0.2, "sub banda se taie, am primit {below}");
        assert!(above < 0.2, "peste banda se taie, am primit {above}");
    }

    #[test]
    fn stays_stable_when_asked_for_an_impossible_frequency() {
        // Arrange: peste jumatatea ratei de esantionare filtrul ar exploda
        let mut filter = Biquad::low_pass(100_000.0, GENTLE_Q);

        // Act
        let output = response_at(&mut filter, 1_000.0);

        // Assert
        assert!(output.is_finite(), "coeficientii trebuie sa ramana valizi");
        assert!(
            output <= 1.5,
            "fara amplificare necontrolata, am primit {output}"
        );
    }
}
