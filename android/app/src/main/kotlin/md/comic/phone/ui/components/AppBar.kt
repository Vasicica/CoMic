package md.comic.phone.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import md.comic.phone.R
import md.comic.phone.device.AppLanguage
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.Radius
import md.comic.phone.ui.theme.ScreenPadding

/**
 * Bara de sus: sigla si numele la stanga, limba si Pro la dreapta.
 *
 * Butonul cu glob sta langa Get Pro, cum cere `phone-design/README.md`. Acelasi
 * lista se va vedea si in Setari, General.
 */
@Composable
fun AppBar(language: String?, onLanguage: (String?) -> Unit, onGetPro: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(64.dp)
            .padding(horizontal = ScreenPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(28.dp)
                    .background(CoMic.Accent, RoundedCornerShape(Radius.Small)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(R.drawable.ic_mic),
                    contentDescription = null,
                    tint = CoMic.Bg,
                    modifier = Modifier.size(16.dp),
                )
            }
            Spacer(Modifier.width(10.dp))
            Text(
                "CoMic",
                color = CoMic.TextMain,
                fontSize = 17.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            LanguageButton(language = language, onPick = onLanguage)
            Spacer(Modifier.width(8.dp))
            ProPill(onClick = onGetPro)
        }
    }
}

/**
 * Globul si codul limbii. Apasat, deschide lista celor zece plus limba
 * telefonului.
 *
 * Numele fiecarei limbi e scris in limba ei, luat din Android: traducerile
 * noastre isi stiu doar numele propriu, iar lista are nevoie de toate deodata.
 */
@Composable
private fun LanguageButton(language: String?, onPick: (String?) -> Unit) {
    var open by remember { mutableStateOf(false) }
    val context = LocalContext.current

    Box {
        Row(
            modifier = Modifier
                .border(1.dp, CoMic.Border, RoundedCornerShape(Radius.Pill))
                .clickable { open = true }
                .padding(horizontal = 10.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                painter = painterResource(R.drawable.ic_globe),
                contentDescription = stringResource(R.string.lang_label),
                tint = CoMic.Muted,
                modifier = Modifier.size(13.dp),
            )
            Spacer(Modifier.width(5.dp))
            Text(
                // Fara alegere, pe buton sta limba in care chiar se vede
                // programul acum, nu un semn de intrebare.
                AppLanguage.short(language ?: context.resources.configuration.locales[0].language),
                color = CoMic.Muted,
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium,
            )
        }

        LanguageMenu(
            expanded = open,
            language = language,
            onPick = { code ->
                open = false
                onPick(code)
            },
            onDismiss = { open = false },
        )
    }
}

/** Butonul catre Pro: coroana si textul, in pastila conturata. */
@Composable
private fun ProPill(onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .border(1.dp, CoMic.Pro.copy(alpha = 0.35f), RoundedCornerShape(Radius.Pill))
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_crown),
            contentDescription = null,
            tint = CoMic.Pro,
            modifier = Modifier.size(13.dp),
        )
        Spacer(Modifier.width(6.dp))
        Text(
            stringResource(R.string.pro_get),
            color = CoMic.Pro,
            fontSize = 13.sp,
            fontWeight = FontWeight.Medium,
        )
    }
}

