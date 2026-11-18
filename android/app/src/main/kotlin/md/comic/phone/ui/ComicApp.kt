package md.comic.phone.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import md.comic.phone.device.AppLanguage
import md.comic.phone.ui.components.AppBar
import md.comic.phone.ui.components.Tab
import md.comic.phone.ui.components.TabBar
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.ScreenPadding

/**
 * Aplicatia intreaga, cu limba aleasa pusa peste tot.
 *
 * Limba se schimba aici, nu in fiecare ecran: dam mai departe un Context cu
 * alta configuratie, si `stringResource` de oriunde mai jos citeste din el.
 * Asa butonul cu glob schimba tot ecranul pe loc, fara repornirea aplicatiei.
 */
@Composable
fun ComicApp() {
    val phone = LocalContext.current
    var language by remember { mutableStateOf(AppLanguage.saved(phone)) }
    val chosen = remember(phone, language) { AppLanguage.wrap(phone, language) }

    CompositionLocalProvider(
        LocalContext provides chosen,
        // Si configuratia, nu doar contextul: `stringResource` se uita la ea ca
        // sa stie cand trebuie sa deseneze din nou.
        LocalConfiguration provides chosen.resources.configuration,
    ) {
        Shell(
            language = language,
            onLanguage = { code ->
                // Scris pe loc: la pornirea urmatoare o gaseste tot asa, iar
                // notificarea serviciului o citeste din acelasi loc.
                AppLanguage.save(phone, code)
                language = code
            },
        )
    }
}

/** Bara de sus, continutul filei alese, bara de file. */
@Composable
private fun Shell(language: String?, onLanguage: (String?) -> Unit) {
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
        AppBar(
            language = language,
            onLanguage = onLanguage,
            onGetPro = { tab = Tab.Effects },
        )

        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = ScreenPadding),
        ) {
            when (tab) {
                Tab.Connect -> ConnectScreen()
                Tab.Effects -> EffectsScreen()
                Tab.Settings -> SettingsScreen(language = language, onLanguage = onLanguage)
            }
            Spacer(Modifier.height(24.dp))
        }

        TabBar(selected = tab, onSelect = { tab = it })
    }
}
