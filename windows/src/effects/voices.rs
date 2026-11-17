//! Catalogul de voci. O voce este un set de setari, nu cod nou.
//!
//! Asta este decizia care face vandabile pachetele de voci: ca sa adaugi o voce
//! scrii zece randuri aici, nu o zi de programare. Motorul ramane acelasi.

/// Setarile unei voci. Campurile `None` inseamna "efectul nu se aplica".
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoicePreset {
    /// Numele scurt folosit in linia de comanda.
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Vocile gratuite se pot asculta de proba; folosirea lor in microfonul
    /// virtual ramane pentru versiunea platita (vezi docs/VOICE-CHANGER.md).
    pub free: bool,
    /// Deplasarea inaltimii, in semitonuri. 0 = neschimbata.
    pub semitones: f32,
    /// Filtru trece-banda: (frecventa centrala in Hz, factorul Q).
    pub bandpass: Option<(f32, f32)>,
    /// Modulatie inelara, in Hz. Da caracterul metalic.
    pub ring_hz: Option<f32>,
    /// Tremolo: (viteza in Hz, adancime 0..1).
    pub tremolo: Option<(f32, f32)>,
    /// Distorsiune. 0 = fara.
    pub drive: f32,
    /// Corectie de volum, pentru ca efectele schimba taria perceputa.
    pub gain: f32,
}

const DEFAULT: VoicePreset = VoicePreset {
    id: "",
    name: "",
    description: "",
    free: false,
    semitones: 0.0,
    bandpass: None,
    ring_hz: None,
    tremolo: None,
    drive: 0.0,
    gain: 1.0,
};

pub const PRESETS: &[VoicePreset] = &[
    VoicePreset {
        id: "normal",
        name: "Normal",
        description: "vocea ta, neschimbata — baza pentru reglaje proprii",
        free: true,
        ..DEFAULT
    },
    VoicePreset {
        id: "pirat",
        name: "Pirat",
        description: "voce coborata, aspra, cu putina distorsiune",
        free: true,
        semitones: -4.0,
        drive: 2.5,
        gain: 0.9,
        ..DEFAULT
    },
    VoicePreset {
        id: "mosneag",
        name: "Mosneag",
        description: "voce putin coborata, cu tremur — batranetea se aude ca instabilitate",
        free: true,
        semitones: -2.0,
        tremolo: Some((5.5, 0.35)),
        drive: 1.2,
        ..DEFAULT
    },
    VoicePreset {
        id: "robot",
        name: "Robot",
        description: "modulatie inelara, sunetul metalic clasic",
        ring_hz: Some(55.0),
        drive: 1.5,
        ..DEFAULT
    },
    VoicePreset {
        id: "radio",
        name: "Statie radio",
        description: "banda ingusta si compresie, ca intr-un walkie-talkie",
        bandpass: Some((1_000.0, 0.4)),
        drive: 3.0,
        gain: 1.4,
        ..DEFAULT
    },
    VoicePreset {
        id: "megafon",
        name: "Megafon",
        description: "distorsiune puternica pe banda ingusta",
        bandpass: Some((1_400.0, 0.6)),
        drive: 6.0,
        gain: 1.2,
        ..DEFAULT
    },
    VoicePreset {
        id: "dinozaur",
        name: "Dinozaur",
        description: "mult in jos, cu marait",
        semitones: -9.0,
        tremolo: Some((18.0, 0.25)),
        drive: 3.5,
        gain: 1.1,
        ..DEFAULT
    },
    VoicePreset {
        id: "demon",
        name: "Demon",
        description: "coborata adanc si distorsionata",
        semitones: -7.0,
        drive: 5.0,
        gain: 1.0,
        ..DEFAULT
    },
    VoicePreset {
        id: "veverita",
        name: "Veverita",
        description: "inaltime mult in sus",
        semitones: 7.0,
        ..DEFAULT
    },
    VoicePreset {
        id: "anime",
        name: "Fata din anime",
        description: "ridicata si luminoasa, tanara — nu piuitul veveritei",
        // Doua semitonuri sub veverita: atat desparte "fata tanara" de
        // "veverita". Filtrul e larg (Q mic), deci nu ingusteaza banda ca la
        // statia radio — doar subtiaza pieptul vocii si scoate in fata
        // frecventele unde se aude limpezimea. Fara distorsiune: caracterul
        // trebuie sa ramana curat.
        semitones: 5.0,
        bandpass: Some((1_600.0, 0.3)),
        gain: 1.15,
        ..DEFAULT
    },
    VoicePreset {
        id: "extraterestru",
        name: "Extraterestru",
        description: "modulatie inelara rapida peste o voce usor ridicata",
        semitones: 2.0,
        ring_hz: Some(140.0),
        ..DEFAULT
    },
];

pub fn find_preset(id: &str) -> Option<&'static VoicePreset> {
    PRESETS
        .iter()
        .find(|preset| preset.id.eq_ignore_ascii_case(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_preset_regardless_of_letter_case() {
        assert_eq!(find_preset("RoBoT").map(|preset| preset.id), Some("robot"));
    }

    #[test]
    fn returns_nothing_for_an_unknown_voice() {
        assert!(find_preset("vocea-lui-bunicu").is_none());
    }

    #[test]
    fn every_preset_has_a_unique_id() {
        for (index, preset) in PRESETS.iter().enumerate() {
            let duplicate = PRESETS[index + 1..]
                .iter()
                .find(|other| other.id == preset.id);
            assert!(duplicate.is_none(), "id duplicat: {}", preset.id);
        }
    }

    #[test]
    fn only_the_starter_voices_are_free() {
        let free: Vec<&str> = PRESETS.iter().filter(|p| p.free).map(|p| p.id).collect();
        assert_eq!(free, vec!["normal", "pirat", "mosneag"]);
    }

    #[test]
    fn no_preset_is_silent_or_deafening() {
        for preset in PRESETS {
            assert!(
                (0.5..=2.0).contains(&preset.gain),
                "{} are un volum nerezonabil: {}",
                preset.id,
                preset.gain
            );
            assert!(
                (-12.0..=12.0).contains(&preset.semitones),
                "{} depaseste o octava: {}",
                preset.id,
                preset.semitones
            );
        }
    }
}
