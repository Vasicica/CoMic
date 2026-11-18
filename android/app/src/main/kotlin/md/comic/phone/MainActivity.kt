package md.comic.phone

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import md.comic.phone.ui.ComicApp
import md.comic.phone.ui.theme.CoMicTheme

/**
 * Singura fereastra a aplicatiei. Nu tine stare proprie: trimiterea traieste in
 * [md.comic.phone.stream.StreamService], ca sa mearga si cu ecranul stins, iar
 * fereastra doar o porneste, o opreste si arata ce face.
 */
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            CoMicTheme {
                ComicApp()
            }
        }
    }
}
