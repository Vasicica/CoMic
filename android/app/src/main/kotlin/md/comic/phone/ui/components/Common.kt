package md.comic.phone.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import md.comic.phone.R
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.Radius
import md.comic.phone.ui.theme.ScreenPadding

/**
 * Piesele din care sunt facute toate cele trei ecrane.
 *
 * Masurile vin din `phone-design/README.md`. Aici sta numai ce se foloseste in
 * mai mult de un ecran; ce apartine unui singur ecran ramane in fisierul lui.
 */

/** Bara de sus: sigla, numele, si in dreapta butonul catre Pro. */
@Composable
fun AppBar(onGetPro: () -> Unit) {
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

        Row(
            modifier = Modifier
                .border(1.dp, CoMic.Pro.copy(alpha = 0.35f), RoundedCornerShape(Radius.Pill))
                .clickable(onClick = onGetPro)
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
            Text("Get Pro", color = CoMic.Pro, fontSize = 13.sp, fontWeight = FontWeight.Medium)
        }
    }
}

/** Filele aplicatiei. `Effects` poarta eticheta PRO cat timp vocile sunt blocate. */
enum class Tab(val label: String, val icon: Int, val proBadge: Boolean) {
    Connect("Connect", R.drawable.ic_mic, false),
    Effects("Effects", R.drawable.ic_sparkle, true),
    Settings("Settings", R.drawable.ic_sliders, false),
}

@Composable
fun TabBar(selected: Tab, onSelect: (Tab) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(78.dp)
            .background(CoMic.BgDeep),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Tab.entries.forEach { tab ->
            val active = tab == selected
            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .clickable { onSelect(tab) },
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center,
            ) {
                Box(contentAlignment = Alignment.Center) {
                    // Pastila din spatele pictogramei marcheaza fila aprinsa.
                    Box(
                        modifier = Modifier
                            .size(width = 56.dp, height = 30.dp)
                            .background(
                                if (active) CoMic.Accent.copy(alpha = 0.18f) else Color.Transparent,
                                RoundedCornerShape(Radius.Pill),
                            ),
                    )
                    Icon(
                        painter = painterResource(tab.icon),
                        contentDescription = null,
                        tint = if (active) CoMic.TextMain else CoMic.Muted,
                        modifier = Modifier.size(19.dp),
                    )
                    if (tab.proBadge) {
                        Text(
                            "PRO",
                            color = CoMic.Pro,
                            fontSize = 7.sp,
                            fontWeight = FontWeight.Bold,
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .background(
                                    CoMic.Pro.copy(alpha = 0.16f),
                                    RoundedCornerShape(4.dp),
                                )
                                .padding(horizontal = 3.dp, vertical = 1.dp),
                        )
                    }
                }
                Spacer(Modifier.height(5.dp))
                Text(
                    tab.label,
                    color = if (active) CoMic.TextMain else CoMic.Muted,
                    fontSize = 11.sp,
                    fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                )
            }
        }
    }
}

/** O optiune dintr-un selector segmentat. `enabled = false` = exista, dar nu inca. */
data class SegmentOption(val label: String, val icon: Int, val enabled: Boolean = true)

/**
 * Selectorul de metoda: Wi-Fi, USB, Bluetooth.
 *
 * O optiune stinsa nu se poate apasa si se vede ca atare. Asta e intelegerea:
 * ce nu merge inca are voie sa se vada, dar nu are voie sa raspunda ca si cum
 * ar merge.
 */
@Composable
fun Segmented(options: List<SegmentOption>, selectedIndex: Int, onSelect: (Int) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Card))
            .padding(4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        options.forEachIndexed { index, option ->
            val active = index == selectedIndex
            Row(
                modifier = Modifier
                    .weight(1f)
                    .height(38.dp)
                    .background(
                        if (active) CoMic.SegmentOn else Color.Transparent,
                        RoundedCornerShape(Radius.Small),
                    )
                    .then(if (option.enabled) Modifier.clickable { onSelect(index) } else Modifier)
                    .alpha(if (option.enabled) 1f else 0.4f),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Center,
            ) {
                Icon(
                    painter = painterResource(option.icon),
                    contentDescription = null,
                    tint = if (active) CoMic.TextMain else CoMic.Muted,
                    modifier = Modifier.size(15.dp),
                )
                Spacer(Modifier.width(7.dp))
                Text(
                    option.label,
                    color = if (active) CoMic.TextMain else CoMic.Muted,
                    fontSize = 13.sp,
                    fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                )
            }
        }
    }
}

/** Cartonasul obisnuit: fundal `surface`, colturi rotunjite. */
@Composable
fun CoMicCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Large))
            .padding(16.dp),
        content = content,
    )
}

/** Titlul mic, cu majuscule, dinaintea unei liste. */
@Composable
fun SectionLabel(text: String) {
    Text(
        text.uppercase(),
        color = CoMic.Muted,
        fontSize = 11.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = 0.8.sp,
    )
}

/**
 * Eticheta pusa peste orice nu are inca nimic in spate.
 *
 * Exista ca sa nu fie nevoie sa ghicesti. Un buton palid ar putea insemna
 * "indisponibil acum"; eticheta asta spune ca lucrul nu e scris inca.
 */
@Composable
fun NotYetTag(text: String = "Not yet") {
    Row(
        modifier = Modifier
            .background(CoMic.Pro.copy(alpha = 0.12f), RoundedCornerShape(Radius.Small))
            .padding(horizontal = 7.dp, vertical = 3.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_lock),
            contentDescription = null,
            tint = CoMic.Pro,
            modifier = Modifier.size(10.dp),
        )
        Spacer(Modifier.width(5.dp))
        Text(text, color = CoMic.Pro, fontSize = 10.sp, fontWeight = FontWeight.SemiBold)
    }
}

/** Cartonasul galben de avertisment, cu semnul de informare. */
@Composable
fun NoticeCard(text: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Pro.copy(alpha = 0.08f), RoundedCornerShape(Radius.Card))
            .border(1.dp, CoMic.Pro.copy(alpha = 0.28f), RoundedCornerShape(Radius.Card))
            .padding(14.dp),
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_info),
            contentDescription = null,
            tint = CoMic.Pro,
            modifier = Modifier.size(15.dp),
        )
        Spacer(Modifier.width(10.dp))
        Text(text, color = CoMic.Body, fontSize = 12.sp, lineHeight = 17.sp)
    }
}

/** Numerele si adresele se scriu cu spatiere fixa, ca sa nu joace pe ecran. */
@Composable
fun MonoText(
    text: String,
    color: Color,
    size: TextUnit,
    weight: FontWeight = FontWeight.Normal,
    modifier: Modifier = Modifier,
) {
    Text(
        text,
        color = color,
        fontSize = size,
        fontWeight = weight,
        fontFamily = FontFamily.Monospace,
        modifier = modifier,
    )
}
