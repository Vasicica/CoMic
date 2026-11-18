package md.comic.phone.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

/**
 * Culorile si masurile interfetei, intr-un singur loc.
 *
 * Vin din `phone-design/README.md` si sunt aceleasi cu ale ferestrei de
 * Windows (`windows/app/ui/index.html`), ca sa se vada ca telefonul si
 * calculatorul sunt un singur program, nu doua care seamana.
 *
 * Pana acum stateau ca valori private in StreamScreen.kt. Acolo nu mai aveau
 * loc: trei ecrane nu pot imprumuta culori dintr-unul singur.
 */
object CoMic {
    /** Fundalul ecranului. */
    val Bg = Color(0xFF0A0A0A)

    /** Fundalul barei de file, mai inchis decat ecranul, ca sa se desprinda. */
    val BgDeep = Color(0xFF080808)

    /** Cartonasele si panourile. */
    val Surface = Color(0xFF101010)

    /** Optiunea aprinsa dintr-un selector segmentat. */
    val SegmentOn = Color(0xFF1E1E1E)

    val Border = Color(0xFF1C1C1C)

    /** Linia care desparte randurile din acelasi cartonas. */
    val Separator = Color(0xFF191919)

    val TextMain = Color(0xFFF5F5F5)

    /** Textul lung din cartonasele de avertisment; mai moale decat titlul. */
    val Body = Color(0xFFD4D4D4)

    val Muted = Color(0xFF8F8F8F)

    /** Cenusiul celor care nu se pot apasa inca. */
    val Faint = Color(0xFF5A5A5A)

    val Accent = Color(0xFF818CF8)
    val Ok = Color(0xFF4ADE80)
    val Danger = Color(0xFFE5484D)

    /** Galbenul: tot ce tine de CoMic Pro, plus avertismentele. */
    val Pro = Color(0xFFFBBF24)
}

/** Razele colturilor, numite ca sa nu fie imprastiate cifre prin ecrane. */
object Radius {
    val Small = 8.dp
    val Card = 12.dp
    val Large = 14.dp
    val Pill = 18.dp
}

/** Spatiul lateral al continutului, din `phone-design/README.md`. */
val ScreenPadding = 20.dp

@Composable
fun CoMicTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        // Numai intuneric. Aplicatia asta se foloseste in timpul jocurilor si
        // al apelurilor, adesea seara; o varianta deschisa ar fi o a doua
        // interfata de intretinut fara ca cineva sa o fi cerut.
        colorScheme = darkColorScheme(
            background = CoMic.Bg,
            surface = CoMic.Surface,
            primary = CoMic.Accent,
            onPrimary = CoMic.Bg,
            onBackground = CoMic.TextMain,
            onSurface = CoMic.TextMain,
            error = CoMic.Danger,
        ),
        content = content,
    )
}
