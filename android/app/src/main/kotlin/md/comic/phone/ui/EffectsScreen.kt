package md.comic.phone.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import md.comic.phone.R
import md.comic.phone.effects.VOICES
import md.comic.phone.effects.Voice
import md.comic.phone.ui.components.CoMicCard
import md.comic.phone.ui.components.NoticeCard
import md.comic.phone.ui.components.NotYetTag
import md.comic.phone.ui.components.SectionLabel
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.Radius

/**
 * Ecranul Effects: ce voci exista si care intra in planul gratuit.
 *
 * Designul cere un selector de voci. Aici e o lista, nu un selector, si e o
 * abatere facuta cu buna stiinta: ca sa alegi vocea de pe telefon, telefonul
 * trebuie sa poata spune ceva calculatorului, iar deocamdata doar trimite sunet
 * intr-o singura directie. Un card care s-ar aprinde la atingere si n-ar
 * schimba nimic in casti ar fi mai rau decat o lista cinstita.
 *
 * Ce arata totusi adevarat: ce voci are programul, cum arata fiecare si cat
 * costa. Cand apare canalul din `docs/CONEXIUNI.md`, lista devine selector si
 * cursoarele se aprind.
 */
@Composable
fun EffectsScreen() {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            stringResource(R.string.fx_title),
            color = CoMic.TextMain,
            fontSize = 22.sp,
            fontWeight = FontWeight.SemiBold,
        )
        Spacer(Modifier.height(14.dp))

        NoticeCard(stringResource(R.string.ph_soonEffects))
        Spacer(Modifier.height(22.dp))

        SectionLabel(stringResource(R.string.ph_voiceList))
        Spacer(Modifier.height(10.dp))

        // Doua pe rand, cu mana, nu cu LazyVerticalGrid: ecranul intreg se
        // deruleaza deja, iar o grila care se deruleaza singura s-ar bate cap
        // in cap cu el.
        VOICES.chunked(2).forEach { pereche ->
            Row(modifier = Modifier.fillMaxWidth()) {
                pereche.forEachIndexed { index, voce ->
                    if (index > 0) Spacer(Modifier.width(10.dp))
                    VoiceCard(voce, modifier = Modifier.weight(1f))
                }
                // Ultimul rand are o singura voce: locul celei lipsa sta gol,
                // ca sa nu se lateasca cardul singur pe tot randul.
                if (pereche.size == 1) {
                    Spacer(Modifier.width(10.dp))
                    Spacer(Modifier.weight(1f))
                }
            }
            Spacer(Modifier.height(10.dp))
        }

        Spacer(Modifier.height(12.dp))
        TunePanel()
    }
}

/** Un card de voce: poza sau pictograma, numele, ce face, si planul. */
@Composable
private fun VoiceCard(voice: Voice, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier
            .background(CoMic.Surface, RoundedCornerShape(Radius.Card))
            .padding(10.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(4f / 3f)
                .clip(RoundedCornerShape(Radius.Small))
                .background(CoMic.SegmentOn),
            contentAlignment = Alignment.Center,
        ) {
            val photo = voice.photo
            if (photo != null) {
                Image(
                    painter = painterResource(photo),
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(4f / 3f),
                )
            } else {
                // Vocile adaugate ultima data n-au poza inca. Pictograma spune
                // ce fel de voce e, fara sa pretinda ca e un portret.
                Icon(
                    painter = painterResource(voice.icon),
                    contentDescription = null,
                    tint = CoMic.Muted,
                    modifier = Modifier.size(28.dp),
                )
            }
        }

        Spacer(Modifier.height(8.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                stringResource(voice.nameRes),
                color = CoMic.TextMain,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
            )
            PlanTag(free = voice.free)
        }
        Spacer(Modifier.height(3.dp))
        Text(
            stringResource(voice.descRes),
            color = CoMic.Muted,
            fontSize = 11.sp,
            lineHeight = 15.sp,
        )
    }
}

/** FREE verde sau PRO galben: planul vocii, nu starea butonului. */
@Composable
private fun PlanTag(free: Boolean) {
    val color = if (free) CoMic.Ok else CoMic.Pro
    Text(
        stringResource(if (free) R.string.tag_free else R.string.tag_pro),
        color = color,
        fontSize = 8.sp,
        fontWeight = FontWeight.Bold,
        modifier = Modifier
            .background(color.copy(alpha = 0.14f), RoundedCornerShape(4.dp))
            .padding(horizontal = 5.dp, vertical = 2.dp),
    )
}

/**
 * Cursoarele de reglaj, stinse.
 *
 * Stau aici ca sa se vada ce se va putea regla, dar nu se pot muta: valorile
 * stau pe calculator, iar telefonul nu are inca drum pana la ele.
 */
@Composable
private fun TunePanel() {
    CoMicCard {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                stringResource(R.string.ph_fineTune),
                color = CoMic.TextMain,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
            )
            NotYetTag()
        }
        Spacer(Modifier.height(4.dp))
        Text(
            stringResource(R.string.ph_tuneOnPc),
            color = CoMic.Muted,
            fontSize = 12.sp,
            lineHeight = 17.sp,
        )
        Spacer(Modifier.height(12.dp))

        Column(modifier = Modifier.alpha(0.45f)) {
            listOf(
                R.string.adv_pitch,
                R.string.adv_tone,
                R.string.adv_drive,
                R.string.adv_wobble,
                R.string.adv_metal,
            ).forEach { eticheta ->
                Text(stringResource(eticheta), color = CoMic.Muted, fontSize = 12.sp)
                Slider(value = 0.5f, onValueChange = {}, enabled = false)
                Spacer(Modifier.height(2.dp))
            }
        }
    }
}
