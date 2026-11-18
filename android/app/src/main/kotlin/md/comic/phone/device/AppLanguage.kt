package md.comic.phone.device

import android.content.Context
import android.content.res.Configuration
import java.util.Locale

/**
 * Limba aleasa de mana, pastrata intre porniri.
 *
 * Android alege singur limba din setarile telefonului, si de obicei bine. Dar
 * designul cere butonul cu glob, si are dreptate: cineva cu telefonul in
 * engleza poate vrea programul in romana. Alegerea sta in acelasi fisier de
 * preferinte ca adresa calculatorului.
 *
 * Nu folosim `AppCompatDelegate.setApplicationLocales`: ar cere intreg
 * `androidx.appcompat` numai pentru atat, iar sub Android 13 are nevoie pe
 * deasupra de un serviciu declarat in manifest. Un Context cu alta
 * configuratie face aceeasi treaba cu ce avem deja.
 */
object AppLanguage {
    /** Limbile pentru care exista traduceri in `shared/i18n`. */
    val CODES = listOf("en", "ro", "ru", "uk", "fr", "es", "pt", "de", "tr", "pl")

    /** Codul ales, sau `null` cat timp urmam telefonul. */
    fun saved(context: Context): String? =
        Prefs.of(context).getString(Prefs.LANGUAGE, null)

    fun save(context: Context, code: String?) {
        val edit = Prefs.of(context).edit()
        if (code == null) edit.remove(Prefs.LANGUAGE) else edit.putString(Prefs.LANGUAGE, code)
        edit.apply()
    }

    /** Acelasi context, dar in limba aleasa. Neschimbat cand nu s-a ales nimic. */
    fun localized(context: Context): Context = wrap(context, saved(context))

    fun wrap(context: Context, code: String?): Context {
        if (code == null) return context
        val config = Configuration(context.resources.configuration)
        config.setLocale(Locale.forLanguageTag(code))
        return context.createConfigurationContext(config)
    }

    /** Ce scrie pe buton: RO, EN, UK. */
    fun short(code: String): String = code.uppercase(Locale.ROOT)

    /**
     * Numele limbii in limba ei.
     *
     * Vine din Android, nu din traducerile noastre: acolo fiecare fisier isi
     * stie doar numele lui, iar meniul are nevoie de toate zece deodata.
     */
    fun name(code: String): String {
        val locale = Locale.forLanguageTag(code)
        val written = locale.getDisplayLanguage(locale)
        // Unele limbi se scriu cu litera mica. In lista stau unele sub altele,
        // deci incep toate cu majuscula.
        return written.replaceFirstChar { it.titlecase(locale) }
    }
}
