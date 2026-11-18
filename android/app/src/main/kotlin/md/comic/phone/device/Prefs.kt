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

    fun of(context: Context): SharedPreferences =
        context.getSharedPreferences(FILE, Context.MODE_PRIVATE)
}
