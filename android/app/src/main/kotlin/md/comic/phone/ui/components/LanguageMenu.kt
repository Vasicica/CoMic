package md.comic.phone.ui.components

import androidx.compose.foundation.background
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.sp
import md.comic.phone.R
import md.comic.phone.device.AppLanguage
import md.comic.phone.ui.theme.CoMic

/**
 * Lista limbilor: limba telefonului, apoi cele zece traduse.
 *
 * Sta aici fiindca se deschide din doua locuri, cum cere
 * `phone-design/README.md`: butonul cu glob din bara de sus si randul Language
 * din Settings. Cine o deschide isi tine singur starea de deschis sau inchis.
 */
@Composable
fun LanguageMenu(
    expanded: Boolean,
    language: String?,
    onPick: (String?) -> Unit,
    onDismiss: () -> Unit,
) {
    DropdownMenu(
        expanded = expanded,
        onDismissRequest = onDismiss,
        modifier = Modifier.background(CoMic.Surface),
    ) {
        DropdownMenuItem(
            text = {
                Text(
                    stringResource(R.string.lang_system),
                    color = if (language == null) CoMic.Accent else CoMic.Body,
                    fontSize = 14.sp,
                )
            },
            onClick = { onPick(null) },
        )
        AppLanguage.CODES.forEach { code ->
            DropdownMenuItem(
                text = {
                    Text(
                        AppLanguage.name(code),
                        color = if (code == language) CoMic.Accent else CoMic.Body,
                        fontSize = 14.sp,
                    )
                },
                onClick = { onPick(code) },
            )
        }
    }
}
