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
    /** Deschide o pagina din Setari; daca lipseste pe telefonul asta, Setarile. */
    fun open(context: Context, action: String) {
        val intent = Intent(action).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        val inlocuitor = Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        runCatching { context.startActivity(intent) }
            .onFailure { runCatching { context.startActivity(inlocuitor) } }
    }

    fun openTethering(context: Context) = open(context, Settings.ACTION_WIRELESS_SETTINGS)

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
