package md.comic.phone.effects

import md.comic.phone.R

/**
 * O voce, asa cum o arata telefonul.
 *
 * `photo` lipseste la vocile adaugate ultima data: inca n-au poza desenata, si
 * pana atunci stau cu pictograma, nu cu o poza de imprumut care ar insemna
 * altceva.
 */
data class Voice(
    val id: String,
    val nameRes: Int,
    val descRes: Int,
    val free: Boolean,
    val icon: Int,
    val photo: Int? = null,
)

/**
 * Catalogul, in aceeasi ordine ca `windows/src/effects/voices.rs`.
 *
 * Stapanul listei e receptorul: el decide ce voci exista si care sunt gratuite.
 * Aici doar le aratam. Daca adaugi una acolo, adaug-o si aici, altfel telefonul
 * va arata o lista mai scurta decat ce poate face programul.
 *
 * Numele si descrierile vin din traduceri, deci se schimba cu limba.
 */
val VOICES = listOf(
    Voice("normal", R.string.voice_normal_name, R.string.voice_normal_desc, true, R.drawable.ic_mic),
    Voice(
        "pirat", R.string.voice_pirat_name, R.string.voice_pirat_desc, true,
        R.drawable.ic_sparkle, R.drawable.voice_pirat,
    ),
    Voice(
        "mosneag", R.string.voice_mosneag_name, R.string.voice_mosneag_desc, true,
        R.drawable.ic_sparkle, R.drawable.voice_mosneag,
    ),
    Voice(
        "robot", R.string.voice_robot_name, R.string.voice_robot_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_robot,
    ),
    Voice(
        "radio", R.string.voice_radio_name, R.string.voice_radio_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_radio,
    ),
    Voice(
        "megafon", R.string.voice_megafon_name, R.string.voice_megafon_desc, false,
        R.drawable.ic_mega,
    ),
    Voice(
        "dinozaur", R.string.voice_dinozaur_name, R.string.voice_dinozaur_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_dinozaur,
    ),
    Voice(
        "demon", R.string.voice_demon_name, R.string.voice_demon_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_demon,
    ),
    Voice(
        "veverita", R.string.voice_veverita_name, R.string.voice_veverita_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_veverita,
    ),
    Voice(
        "extraterestru", R.string.voice_extraterestru_name,
        R.string.voice_extraterestru_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_extraterestru,
    ),
    Voice(
        "pestera", R.string.voice_pestera_name, R.string.voice_pestera_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_pestera,
    ),
    Voice(
        "subapa", R.string.voice_subapa_name, R.string.voice_subapa_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_subapa,
    ),
    Voice(
        "fantoma", R.string.voice_fantoma_name, R.string.voice_fantoma_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_fantoma,
    ),
    Voice(
        "soapta", R.string.voice_soapta_name, R.string.voice_soapta_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_soapta,
    ),
    Voice(
        "monstru", R.string.voice_monstru_name, R.string.voice_monstru_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_monstru,
    ),
    Voice(
        "urias", R.string.voice_urias_name, R.string.voice_urias_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_urias,
    ),
    Voice(
        "telefon", R.string.voice_telefon_name, R.string.voice_telefon_desc, false,
        R.drawable.ic_sparkle, R.drawable.voice_telefon,
    ),
)
