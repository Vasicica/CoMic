//! Buffer de jitter: reordoneaza pachetele UDP si absoarbe neregularitatile retelei.
//!
//! Reteaua livreaza cadrele inegal si uneori in alta ordine. Placa de sunet, in
//! schimb, cere exact cate esantioane ii trebuie, la fix. Buffer-ul sta intre ele.
//!
//! Doua reguli il guverneaza, si amandoua sunt despre latenta:
//!
//! - Un cadru pierdut costa exact un cadru de liniste. Nu asteptam dupa el, pentru
//!   ca asteptarea se vede in latenta si se aude ca intrerupere.
//! - Daca rezerva creste peste maxim (ceas mai rapid pe telefon decat pe placa de
//!   sunet), taiem din cea mai veche. Altfel latenta urca incet si nu mai coboara.

use std::collections::BTreeMap;

use crate::proto::{Frame, FRAME_SAMPLES, SAMPLE_RATE};

/// Rezerva acumulata inainte de a porni redarea: 3 x 10 ms.
/// Utilizatorul o poate schimba intre 1 si 10, ca in WO Mic: mai mica inseamna
/// latenta mai mica, mai mare inseamna rezistenta la o retea instabila.
pub const DEFAULT_PREFILL_FRAMES: usize = 3;
pub const MIN_PREFILL_FRAMES: usize = 1;
pub const MAX_PREFILL_FRAMES: usize = 10;
/// Peste atata rezerva, taiem din cea veche ca sa nu creasca latenta.
const MAX_BUFFER_FRAMES: usize = 10;
/// Cate cadre mai noi tolerem in asteptarea unuia intarziat, inainte sa-l declaram
/// pierdut. 2 cadre = 20 ms de toleranta la reordonare.
const REORDER_WINDOW: usize = 2;

const MAX_BUFFER_SAMPLES: usize = MAX_BUFFER_FRAMES * FRAME_SAMPLES;

pub struct JitterBuffer {
    /// Cadre primite dar neconsumate, cheia este numarul de secventa.
    pending: BTreeMap<u32, Vec<i16>>,
    /// Esantioane gata de redare, in ordine.
    ready: Vec<i16>,
    /// Urmatorul numar de secventa asteptat; `None` inainte de primul cadru.
    next_sequence: Option<u32>,
    playing: bool,
    /// Cat acumulam inainte de a porni redarea.
    prefill_samples: usize,
    pub stats: Stats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub frames_received: u64,
    pub frames_dropped_late: u64,
    pub frames_missing: u64,
    pub frames_trimmed: u64,
    pub underruns: u64,
}

impl JitterBuffer {
    /// `frames` este marimea rezervei, in cadre de 10 ms. Valorile din afara
    /// intervalului acceptat sunt aduse la margine, nu respinse: utilizatorul
    /// care scrie 20 vrea "cat mai mult", nu o eroare.
    pub fn with_prefill(frames: usize) -> Self {
        let frames = frames.clamp(MIN_PREFILL_FRAMES, MAX_PREFILL_FRAMES);

        Self {
            pending: BTreeMap::new(),
            ready: Vec::new(),
            next_sequence: None,
            playing: false,
            prefill_samples: frames * FRAME_SAMPLES,
            stats: Stats::default(),
        }
    }

    pub fn push(&mut self, frame: Frame) {
        self.stats.frames_received += 1;

        let expected = *self.next_sequence.get_or_insert(frame.sequence);
        if frame.sequence < expected {
            // A ajuns prea tarziu: locul lui din flux a trecut deja.
            self.stats.frames_dropped_late += 1;
            return;
        }

        self.pending.insert(frame.sequence, frame.samples);
    }

    /// Muta in `ready` urmatorul cadru din flux. Returneaza `false` daca nu se
    /// poate inca - adica asteptam legitim un cadru care mai are timp sa ajunga.
    fn advance_one_frame(&mut self) -> bool {
        let Some(expected) = self.next_sequence else {
            return false;
        };

        if let Some(samples) = self.pending.remove(&expected) {
            self.ready.extend_from_slice(&samples);
            self.next_sequence = Some(expected.wrapping_add(1));
            return true;
        }

        // Cadrul asteptat nu e aici. Daca in spatele lui s-au strans destule cadre
        // mai noi, inseamna ca s-a pierdut: il inlocuim cu liniste si mergem mai
        // departe. (In Faza 1b, Opus va reconstrui cadrul in loc de liniste.)
        if self.pending.len() > REORDER_WINDOW {
            self.ready.resize(self.ready.len() + FRAME_SAMPLES, 0);
            self.next_sequence = Some(expected.wrapping_add(1));
            self.stats.frames_missing += 1;
            return true;
        }

        false
    }

    fn drain_available(&mut self) {
        while self.advance_one_frame() {}
    }

    /// Taie din rezerva cea mai veche cand s-a adunat prea multa, ca sa nu urce
    /// latenta. Se intampla cand ceasul telefonului merge mai repede decat al
    /// placii de sunet.
    fn trim_excess(&mut self) {
        if self.ready.len() <= MAX_BUFFER_SAMPLES {
            return;
        }

        let excess = self.ready.len() - MAX_BUFFER_SAMPLES;
        self.ready.drain(..excess);
        self.stats.frames_trimmed += (excess / FRAME_SAMPLES) as u64;
    }

    /// Umple `output` cu esantioane mono. Returneaza `false` la underrun
    /// (am livrat liniste pentru ca reteaua a ramas in urma).
    pub fn fill(&mut self, output: &mut [i16]) -> bool {
        self.drain_available();
        self.trim_excess();

        if !self.playing {
            if self.ready.len() < self.prefill_samples.max(output.len()) {
                output.fill(0);
                return false;
            }
            self.playing = true;
        }

        if self.ready.len() < output.len() {
            // Livram tot ce avem, restul liniste, si reluam acumularea rezervei.
            let available = self.ready.len();
            output[..available].copy_from_slice(&self.ready);
            output[available..].fill(0);
            self.ready.clear();
            self.playing = false;
            self.stats.underruns += 1;
            return false;
        }

        let tail = self.ready.split_off(output.len());
        output.copy_from_slice(&self.ready);
        self.ready = tail;
        true
    }

    pub fn buffered_ms(&self) -> u32 {
        (self.ready.len() as u32) / (SAMPLE_RATE / 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valoarea esantioanelor identifica cadrul si nu se confunda cu linistea.
    fn frame(sequence: u32) -> Frame {
        Frame {
            sequence,
            timestamp: sequence * FRAME_SAMPLES as u32,
            samples: vec![(sequence as i16 + 1) * 100; FRAME_SAMPLES],
        }
    }

    fn push_frames(buffer: &mut JitterBuffer, sequences: impl IntoIterator<Item = u32>) {
        for sequence in sequences {
            buffer.push(frame(sequence));
        }
    }

    #[test]
    fn outputs_silence_until_the_prefill_is_reached() {
        // Arrange
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, 0..DEFAULT_PREFILL_FRAMES as u32 - 1);
        let mut output = [99i16; FRAME_SAMPLES];

        // Act
        let filled = buffer.fill(&mut output);

        // Assert
        assert!(!filled);
        assert!(output.iter().all(|&sample| sample == 0));
    }

    #[test]
    fn a_larger_buffer_setting_waits_for_more_audio_before_playing() {
        // Arrange: rezerva maxima, alimentata cu rezerva implicita
        let mut buffer = JitterBuffer::with_prefill(MAX_PREFILL_FRAMES);
        push_frames(&mut buffer, 0..DEFAULT_PREFILL_FRAMES as u32);
        let mut output = [0i16; FRAME_SAMPLES];

        // Act
        let filled = buffer.fill(&mut output);

        // Assert
        assert!(!filled, "cu rezerva mare, atata sunet nu este de ajuns");
    }

    #[test]
    fn clamps_a_buffer_setting_outside_the_accepted_range() {
        // Arrange & Act
        let too_large = JitterBuffer::with_prefill(999);
        let too_small = JitterBuffer::with_prefill(0);

        // Assert
        assert_eq!(
            too_large.prefill_samples,
            MAX_PREFILL_FRAMES * FRAME_SAMPLES
        );
        assert_eq!(
            too_small.prefill_samples,
            MIN_PREFILL_FRAMES * FRAME_SAMPLES
        );
    }

    #[test]
    fn reorders_packets_that_arrive_out_of_sequence() {
        // Arrange
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, [0, 2, 1, 3]);
        let mut output = [0i16; FRAME_SAMPLES * 4];

        // Act
        let filled = buffer.fill(&mut output);

        // Assert
        assert!(filled);
        assert_eq!(output[0], 100);
        assert_eq!(output[FRAME_SAMPLES], 200);
        assert_eq!(output[FRAME_SAMPLES * 2], 300);
        assert_eq!(output[FRAME_SAMPLES * 3], 400);
        assert_eq!(buffer.stats.frames_missing, 0, "nimic nu s-a pierdut");
    }

    #[test]
    fn waits_briefly_for_a_frame_that_is_merely_late() {
        // Arrange: cadrul 1 lipseste, dar in spatele lui sunt doar doua cadre.
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, [0, 2, 3]);

        // Act
        buffer.drain_available();

        // Assert
        assert_eq!(buffer.stats.frames_missing, 0, "inca il asteptam pe 1");
        assert_eq!(
            buffer.buffered_ms(),
            10,
            "doar cadrul 0 a trecut mai departe"
        );
    }

    #[test]
    fn replaces_a_lost_frame_with_exactly_one_frame_of_silence() {
        // Arrange: cadrul 1 se pierde, iar in spatele lui se aduna destule cadre.
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, [0, 2, 3, 4]);
        let mut output = [0i16; FRAME_SAMPLES * 4];

        // Act
        let filled = buffer.fill(&mut output);

        // Assert
        assert!(filled);
        assert_eq!(buffer.stats.frames_missing, 1);
        assert_eq!(output[0], 100, "cadrul 0");
        assert_eq!(output[FRAME_SAMPLES], 0, "golul lasat de cadrul 1");
        assert_eq!(output[FRAME_SAMPLES * 2], 300, "cadrul 2 vine imediat dupa");
        assert_eq!(output[FRAME_SAMPLES * 3], 400);
    }

    #[test]
    fn discards_a_packet_that_arrives_after_its_slot_has_played() {
        // Arrange
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, 0..DEFAULT_PREFILL_FRAMES as u32);
        let mut output = [0i16; FRAME_SAMPLES * DEFAULT_PREFILL_FRAMES];
        buffer.fill(&mut output);

        // Act
        buffer.push(frame(0));

        // Assert
        assert_eq!(buffer.stats.frames_dropped_late, 1);
    }

    #[test]
    fn counts_an_underrun_when_the_network_falls_behind() {
        // Arrange
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, 0..DEFAULT_PREFILL_FRAMES as u32);
        let mut output = [0i16; FRAME_SAMPLES * DEFAULT_PREFILL_FRAMES];
        buffer.fill(&mut output);
        let mut starved = [7i16; FRAME_SAMPLES];

        // Act
        let filled = buffer.fill(&mut starved);

        // Assert
        assert!(!filled);
        assert_eq!(buffer.stats.underruns, 1);
        assert!(starved.iter().all(|&sample| sample == 0));
    }

    #[test]
    fn trims_the_oldest_audio_when_the_buffer_grows_past_the_limit() {
        // Arrange: de doua ori mai multe cadre decat limita.
        let mut buffer = JitterBuffer::with_prefill(DEFAULT_PREFILL_FRAMES);
        push_frames(&mut buffer, 0..MAX_BUFFER_FRAMES as u32 * 2);
        let mut output = [0i16; FRAME_SAMPLES];

        // Act
        buffer.fill(&mut output);

        // Assert
        assert!(buffer.stats.frames_trimmed > 0, "s-a taiat din rezerva");
        assert!(
            buffer.buffered_ms() <= (MAX_BUFFER_FRAMES * 10) as u32,
            "latenta ramane sub limita"
        );
        assert_ne!(output[0], 100, "cadrul cel mai vechi a fost sacrificat");
    }
}
