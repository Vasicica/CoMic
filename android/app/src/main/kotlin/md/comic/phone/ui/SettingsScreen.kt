package md.comic.phone.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import md.comic.phone.BuildConfig
import md.comic.phone.R
import md.comic.phone.device.AppLanguage
import md.comic.phone.device.Prefs
import md.comic.phone.device.SystemPages
import md.comic.phone.protocol.Packet
import md.comic.phone.ui.components.LanguageMenu
import md.comic.phone.ui.components.NotYetTag
import md.comic.phone.ui.components.SectionLabel
import md.comic.phone.ui.components.SegmentOption
import md.comic.phone.ui.components.Segmented
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.Radius

/**
 * Ecranul Settings, dupa grupele din `phone-design/README.md`.
 *
 * Trei lucruri de aici chiar lucreaza: portul, limba si starea bateriei.
 * Restul se vad, dar nu raspund, si fiecare poarta eticheta care spune ca nu e
 * scris inca. Asa stii dintr-o privire ce poti regla azi, fara sa incerci.
 *
 * Doua randuri nu au comutator desi designul cere unul: trimiterea cu ecranul
 * stins si butonul din notificare merg deja, si nu se pot opri. Un comutator
 * aprins pe care nu-l poti stinge e o minciuna mai mica, dar tot o minciuna,
 * deci scrie doar cum stau lucrurile.
 */
@Composable
fun SettingsScreen(language: String?, onLanguage: (String?) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            stringResource(R.string.set_title),
            color = CoMic.TextMain,
            fontSize = 22.sp,
            fontWeight = FontWeight.SemiBold,
        )
        Spacer(Modifier.height(20.dp))

        AudioGroup()
        ConnectionGroup()
        BackgroundGroup()
        GeneralGroup(language = language, onLanguage = onLanguage)
        AccountGroup()
    }
}

/** Sunetul: microfonul, calitatea, volumul. Nimic de aici nu e scris inca. */
@Composable
private fun AudioGroup() {
    Group(R.string.set_audio) {
        SettingRow(stringResource(R.string.set_mic), notYet = true) {}
        DeadSegments(
            listOf(R.string.set_micBottom, R.string.set_micTop, R.string.set_micHeadset),
            implicit = 0,
        )
        Separator()
        SettingRow(stringResource(R.string.set_quality), notYet = true) {}
        DeadSegments(
            listOf(R.string.set_qLow, R.string.set_qStd, R.string.set_qHigh),
            implicit = 1,
        )
        Separator()
        SettingRow(stringResource(R.string.set_gain), notYet = true) {}
        Slider(
            value = 1f,
            onValueChange = {},
            valueRange = 0.5f..2f,
            enabled = false,
            modifier = Modifier.alpha(0.45f),
        )
        Separator()
        SettingRow(
            stringResource(R.string.set_noise),
            hint = stringResource(R.string.set_noiseHint),
            notYet = true,
        ) { DeadSwitch(on = true) }
        Separator()
        SettingRow(
            stringResource(R.string.set_agc),
            hint = stringResource(R.string.set_agcHint),
            notYet = true,
        ) { DeadSwitch(on = false) }
    }
}

/** Conexiunea. Portul e singurul rand care face ceva. */
@Composable
private fun ConnectionGroup() {
    val context = LocalContext.current
    val prefs = remember { Prefs.of(context) }
    var port by remember { mutableStateOf(prefs.getInt(Prefs.PORT, Packet.AUDIO_PORT).toString()) }

    Group(R.string.set_connection) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(stringResource(R.string.set_port), color = CoMic.TextMain, fontSize = 14.sp)
            OutlinedTextField(
                value = port,
                onValueChange = { typed ->
                    val curat = typed.filter { it.isDigit() }.take(5)
                    port = curat
                    // Se scrie pe loc, nu la vreun buton Save: acelasi numar il
                    // citeste si ecranul Connect la pornire.
                    curat.toIntOrNull()
                        ?.takeIf { it in 1..65_535 }
                        ?.let { prefs.edit().putInt(Prefs.PORT, it).apply() }
                },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                modifier = Modifier.width(120.dp),
            )
        }
        Separator()
        SettingRow(
            stringResource(R.string.set_reconnect),
            hint = stringResource(R.string.set_reconnectHint),
            notYet = true,
        ) { DeadSwitch(on = true) }
        Separator()
        SettingRow(
            stringResource(R.string.set_pairing),
            hint = stringResource(R.string.set_pairingHint),
            notYet = true,
        ) { DeadSwitch(on = true) }
        Separator()
        SettingRow(
            stringResource(R.string.set_calls),
            hint = stringResource(R.string.set_callsHint),
            notYet = true,
        ) { DeadSwitch(on = true) }
    }
}

/**
 * Fundalul. Primele doua randuri spun cum stau lucrurile, fara comutator:
 * amandoua merg deja si nu se pot opri. Bateria insa chiar se poate verifica.
 */
@Composable
private fun BackgroundGroup() {
    val context = LocalContext.current
    val scutit = SystemPages.batteryUnrestricted(context)

    Group(R.string.set_background) {
        SettingRow(
            stringResource(R.string.set_screenOff),
            hint = stringResource(R.string.set_screenOffHint),
        ) { StateText(stringResource(R.string.on), ok = true) }
        Separator()
        SettingRow(
            stringResource(R.string.set_notif),
            hint = stringResource(R.string.set_notifHint),
        ) { StateText(stringResource(R.string.on), ok = true) }
        Separator()
        SettingRow(
            stringResource(R.string.set_battery),
            hint = stringResource(
                if (scutit) R.string.set_batteryOk else R.string.set_batteryBad,
            ),
        ) {
            Text(
                stringResource(R.string.set_check),
                color = CoMic.Accent,
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier
                    .clickable { SystemPages.openBattery(context) }
                    .padding(horizontal = 8.dp, vertical = 6.dp),
            )
        }
    }
}

/** Limba. Aceeasi lista ca butonul cu glob din bara de sus. */
@Composable
private fun GeneralGroup(language: String?, onLanguage: (String?) -> Unit) {
    var open by remember { mutableStateOf(false) }

    Group(R.string.set_general) {
        SettingRow(stringResource(R.string.lang_label)) {
            Box {
                Text(
                    if (language == null) {
                        stringResource(R.string.lang_system)
                    } else {
                        AppLanguage.name(language)
                    },
                    color = CoMic.Accent,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier
                        .clickable { open = true }
                        .padding(horizontal = 8.dp, vertical = 6.dp),
                )
                LanguageMenu(
                    expanded = open,
                    language = language,
                    onPick = { code ->
                        open = false
                        onLanguage(code)
                    },
                    onDismiss = { open = false },
                )
            }
        }
    }
}

/** Contul. Plata vine prin Google Play si inca nu e scrisa. */
@Composable
private fun AccountGroup() {
    Group(R.string.set_account) {
        SettingRow(
            stringResource(R.string.set_pro),
            hint = stringResource(R.string.set_planFree),
            notYet = true,
        ) {}
        Separator()
        SettingRow(stringResource(R.string.set_help), notYet = true) {}
        Separator()
        SettingRow(stringResource(R.string.set_about), notYet = true) {}
        ProTestSwitch()
    }
}

/**
 * Comutatorul care tine locul cumparaturii, pana exista Google Play Billing.
 *
 * **Numai in compilarea de depanare.** `BuildConfig.DEBUG` e fals in orice
 * compilare de lansare, deci randul asta nu exista in aplicatia pe care o ia
 * cineva din magazin. Un buton "deblocheaza Pro" intr-un produs vandut ar fi,
 * evident, o gluma proasta.
 *
 * Rostul lui: drumul de la telefon la calculator e scris si trebuie incercat
 * acum, nu peste luni, cand se deschide contul de Play Console. Cand vine
 * Billing, el scrie in aceeasi cheie si randul asta dispare singur.
 */
@Composable
private fun ProTestSwitch() {
    if (!BuildConfig.DEBUG) return

    val context = LocalContext.current
    var pro by remember { mutableStateOf(Prefs.of(context).getBoolean(Prefs.PRO, false)) }

    Separator()
    SettingRow(
        stringResource(R.string.set_proDebug),
        hint = stringResource(R.string.set_proDebugHint),
    ) {
        Switch(
            checked = pro,
            onCheckedChange = { pornit ->
                pro = pornit
                Prefs.of(context).edit().putBoolean(Prefs.PRO, pornit).apply()
            },
        )
    }
}

// ---------- piesele din care sunt facute grupele ----------

/** Un grup: titlu mic deasupra, randurile intr-un card. */
@Composable
private fun Group(labelRes: Int, content: @Composable ColumnScope.() -> Unit) {
    SectionLabel(stringResource(labelRes))
    Spacer(Modifier.height(10.dp))
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Card))
            .padding(horizontal = 14.dp, vertical = 4.dp),
        content = content,
    )
    Spacer(Modifier.height(22.dp))
}

/**
 * Un rand: numele la stanga, lamurirea dedesubt, comanda la dreapta.
 *
 * `notYet` pune eticheta care spune ca lucrul nu e scris. Nu stinge comanda:
 * aceea trebuie stinsa de cine o pune, ca sa nu existe doua feluri de a spune
 * acelasi lucru.
 */
@Composable
private fun SettingRow(
    label: String,
    hint: String? = null,
    notYet: Boolean = false,
    right: @Composable () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 13.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(label, color = CoMic.TextMain, fontSize = 14.sp)
                if (notYet) {
                    Spacer(Modifier.width(8.dp))
                    NotYetTag()
                }
            }
            if (hint != null) {
                Spacer(Modifier.height(3.dp))
                Text(hint, color = CoMic.Muted, fontSize = 11.sp, lineHeight = 15.sp)
            }
        }
        Spacer(Modifier.width(10.dp))
        right()
    }
}

/** Linia subtire dintre doua randuri. */
@Composable
private fun Separator() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(1.dp)
            .background(CoMic.Separator),
    )
}

/** Un comutator care arata cum va fi, dar nu se poate muta. */
@Composable
private fun DeadSwitch(on: Boolean) {
    Switch(
        checked = on,
        onCheckedChange = {},
        enabled = false,
        modifier = Modifier.alpha(0.45f),
    )
}

/** Un selector cu toate opriile stinse: se vede ce se va putea alege. */
@Composable
private fun DeadSegments(labels: List<Int>, implicit: Int) {
    Segmented(
        options = labels.map { SegmentOption(stringResource(it), enabled = false) },
        // Nu se poate muta; arata doar de unde va porni cand va merge.
        selectedIndex = implicit,
        onSelect = {},
    )
    Spacer(Modifier.height(10.dp))
}

/** Starea unui lucru care merge deja: verde cand e bine, galben cand nu. */
@Composable
private fun StateText(text: String, ok: Boolean) {
    Text(
        text,
        color = if (ok) CoMic.Ok else CoMic.Pro,
        fontSize = 13.sp,
        fontWeight = FontWeight.Medium,
    )
}
