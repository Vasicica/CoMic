//! Protocolul CoMic v1. Vezi docs/PROTOCOL.md.

pub const MAGIC: u16 = 0x434D; // "CM"
pub const VERSION: u8 = 1;
pub const HEADER_LEN: usize = 12;

/// Rata de esantionare si dimensiunea cadrului sunt fixe in v1.
pub const SAMPLE_RATE: u32 = 48_000;
pub const FRAME_MS: u32 = 10;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE / 1000 * FRAME_MS) as usize; // 480

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    PcmS16Le,
    Opus,
}

impl Codec {
    fn from_wire(value: u8) -> Option<Self> {
        match value {
            0 => Some(Codec::PcmS16Le),
            1 => Some(Codec::Opus),
            _ => None,
        }
    }
}

/// Un cadru audio decodat din reprezentarea de pe fir.
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub sequence: u32,
    pub timestamp: u32,
    pub samples: Vec<i16>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u8),
    UnknownCodec(u8),
    /// Opus ajunge in Faza 1b; pana atunci trimitem PCM brut.
    CodecNotImplemented(Codec),
    OddPayloadLength,
}

/// Decodeaza un pachet UDP intr-un cadru. Nu muteaza intrarea.
pub fn parse_packet(packet: &[u8]) -> Result<Frame, ParseError> {
    if packet.len() < HEADER_LEN {
        return Err(ParseError::TooShort);
    }

    let magic = u16::from_be_bytes([packet[0], packet[1]]);
    if magic != MAGIC {
        return Err(ParseError::BadMagic);
    }

    let version = packet[2];
    if version != VERSION {
        return Err(ParseError::UnsupportedVersion(version));
    }

    let codec = Codec::from_wire(packet[3]).ok_or(ParseError::UnknownCodec(packet[3]))?;
    if codec != Codec::PcmS16Le {
        return Err(ParseError::CodecNotImplemented(codec));
    }

    let sequence = u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]);
    let timestamp = u32::from_be_bytes([packet[8], packet[9], packet[10], packet[11]]);

    let payload = &packet[HEADER_LEN..];
    if !payload.len().is_multiple_of(2) {
        return Err(ParseError::OddPayloadLength);
    }

    let samples = payload
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();

    Ok(Frame {
        sequence,
        timestamp,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm_packet(sequence: u32, samples: &[i16]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&MAGIC.to_be_bytes());
        packet.push(VERSION);
        packet.push(0); // PCM
        packet.extend_from_slice(&sequence.to_be_bytes());
        packet.extend_from_slice(&0u32.to_be_bytes());
        for sample in samples {
            packet.extend_from_slice(&sample.to_le_bytes());
        }
        packet
    }

    #[test]
    fn parses_a_well_formed_pcm_packet() {
        // Arrange
        let packet = pcm_packet(7, &[-32_768, 0, 32_767]);

        // Act
        let frame = parse_packet(&packet).expect("pachetul trebuie acceptat");

        // Assert
        assert_eq!(frame.sequence, 7);
        assert_eq!(frame.samples, vec![-32_768, 0, 32_767]);
    }

    #[test]
    fn rejects_a_packet_whose_magic_does_not_match() {
        let mut packet = pcm_packet(1, &[0]);
        packet[0] = 0xFF;

        assert_eq!(parse_packet(&packet), Err(ParseError::BadMagic));
    }

    #[test]
    fn rejects_a_packet_shorter_than_the_header() {
        assert_eq!(parse_packet(&[0x43, 0x4D]), Err(ParseError::TooShort));
    }

    #[test]
    fn reports_opus_as_not_yet_implemented() {
        let mut packet = pcm_packet(1, &[0]);
        packet[3] = 1; // Opus

        assert_eq!(
            parse_packet(&packet),
            Err(ParseError::CodecNotImplemented(Codec::Opus))
        );
    }
}
