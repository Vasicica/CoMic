package md.comic.phone.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import md.comic.phone.ui.components.AppBar
import md.comic.phone.ui.components.NoticeCard
import md.comic.phone.ui.components.Tab
import md.comic.phone.ui.components.TabBar
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.ScreenPadding

/**
 * Aplicatia intreaga: bara de sus, continutul filei alese, bara de file.
 *
 * Doar Connect e construit. Effects si Settings urmeaza; pana atunci filele
 * exista si spun pe sleau ca sunt goale, in loc sa arate comenzi care n-ar
 * face nimic.
 */
@Composable
fun ComicApp() {
    var tab by remember { mutableStateOf(Tab.Connect) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(CoMic.Bg)
            // Fundalul merge pana la marginea ecranului, dar continutul nu: de
            // la Android 15, o aplicatie care tinteste SDK 35 se intinde
            // obligatoriu sub bara de stare, deci spatiul acela trebuie cerut
            // singur, altfel titlul sta peste ceas.
            .safeDrawingPadding(),
    ) {
        AppBar(onGetPro = { tab = Tab.Effects })

        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = ScreenPadding),
        ) {
            when (tab) {
                Tab.Connect -> ConnectScreen()
                Tab.Effects -> Placeholder(
                    title = "Voice effects",
                    explanation = "Not built yet. Picking a voice from the phone needs the " +
                        "TCP control channel — right now the phone only sends audio one " +
                        "way and has no way to tell the PC anything. Until then, change " +
                        "the voice in CoMic on the PC.",
                )

                Tab.Settings -> Placeholder(
                    title = "Settings",
                    explanation = "Not built yet. The port lives on the Connect tab for now. " +
                        "Microphone choice, gain, noise suppression and the background " +
                        "options come next.",
                )
            }
            Spacer(Modifier.height(24.dp))
        }

        TabBar(selected = tab, onSelect = { tab = it })
    }
}

/**
 * O fila care inca nu exista.
 *
 * Spune de ce lipseste, nu doar ca lipseste: "coming soon" nu ajuta pe nimeni,
 * iar motivul e de obicei chiar lucrul care se construieste in continuare.
 */
@Composable
private fun Placeholder(title: String, explanation: String) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Spacer(Modifier.height(40.dp))
        Text(title, color = CoMic.TextMain, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
        Spacer(Modifier.height(6.dp))
        Text(
            "Under construction",
            color = CoMic.Muted,
            fontSize = 13.sp,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(22.dp))
        NoticeCard(explanation)
    }
}
