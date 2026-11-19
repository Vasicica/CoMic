package md.comic.phone.device

import android.content.Context
import android.content.Intent
import android.os.PowerManager
import android.provider.Settings

/**
 * Paginile din Setarile Android pe care le deschidem, si ce putem citi din ele.
 *
 * Nu cerem nicio scutire direct: pentru bateria neoptimizata exista si o cale
 * scurta, dar cere o permisiune pe care Google o da doar unor feluri de
 * aplicatii. Deschidem lista si lasam omul sa aleaga.
 */
object SystemPages {
    /**
     * Deschide prima pagina din [actions] care exista pe telefonul asta.
     *
     * Mai multe incercari, nu una: paginile cele mai folositoare sunt adesea
     * cele pe care Android nu le promite. Daca niciuna nu merge, deschidem
     * Setarile, ca omul sa nu ramana cu un buton care nu face nimic.
     */
    fun open(context: Context, vararg actions: String) {
        val incercari = actions.toList() + Settings.ACTION_SETTINGS
        for (action in incercari) {
            val intent = Intent(action).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            if (runCatching { context.startActivity(intent) }.isSuccess) return
        }
    }

    /**
     * Pagina "Punct de acces si modem", unde se aprinde partajarea prin USB.
     *
     * `TETHER_SETTINGS` duce fix acolo, dar e o actiune pe care Android nu o
     * promite: nu are constanta in SDK si pe unele telefoane lipseste.
     * De-aceea e scrisa de mana, cu setarile wireless ca rezerva. Pana la
     * schimbarea asta butonul deschidea direct rezerva, si te lasa cu doua
     * niveluri de cautat exact cand aveai nevoie de un singur comutator.
     */
    fun openTethering(context: Context) =
        open(context, "android.settings.TETHER_SETTINGS", Settings.ACTION_WIRELESS_SETTINGS)

    fun openBattery(context: Context) =
        open(context, Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)

    /**
     * Daca sistemul lasa aplicatia in pace cand ecranul e stins.
     *
     * Cand nu o lasa, Android poate taia microfonul dupa un timp, iar sunetul
     * se opreste fara ca nimeni sa fi apasat nimic.
     */
    fun batteryUnrestricted(context: Context): Boolean =
        context.getSystemService(PowerManager::class.java)
            ?.isIgnoringBatteryOptimizations(context.packageName) ?: false
}
