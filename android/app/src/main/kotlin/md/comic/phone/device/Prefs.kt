package md.comic.phone.device

import android.content.Context
import android.content.SharedPreferences

/**
 * Unde isi tine aplicatia cele cateva lucruri de care are nevoie la pornire.
 *
 * Numele stau aici si nu in fiecare ecran: portul se vede atat in Connect cat
 * si in Settings, iar doua ecrane care scriu in chei scrise de mana ajung,
 * mai curand sau mai tarziu, sa scrie in chei diferite.
 */
object Prefs {
    private const val FILE = "comic"

    const val ADDRESS = "address"
    const val PORT = "port"
    const val LANGUAGE = "lang"

    /**
     * A cumparat omul Pro?
     *
     * Aici e singurul loc din tot proiectul care stie raspunsul: cumparatura se
     * face pe telefon, prin Google Play, iar calculatorul afla de la noi, pe
     * canalul de control. El nu are plata proprie si nu verifica nimic singur.
     *
     * Deocamdata nimic nu scrie aici in afara comutatorului din Settings, care
     * exista numai in compilarea de depanare. Cand vine Google Play Billing, el
     * ia locul comutatorului; restul drumului e deja scris si incercat.
     */
    const val PRO = "pro"

    fun of(context: Context): SharedPreferences =
        context.getSharedPreferences(FILE, Context.MODE_PRIVATE)
}
