package md.comic.phone.device

import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import java.net.Inet4Address
import java.net.NetworkInterface
import java.net.SocketException

/**
 * Ce stie telefonul despre propriile lui legaturi.
 *
 * Totul de aici se citeste fara nicio permisiune noua. Numele retelei Wi-Fi
 * lipseste dinadins: din Android 8.1 incolo el cere permisiunea de localizare,
 * iar asta e un pret prea mare pentru o eticheta. Designul il arata; noi
 * preferam sa nu cerem locatia utilizatorului ca sa scriem "Home_5G".
 */
object PhoneNetwork {

    /** O adresa a telefonului, impreuna cu placa de retea pe care sta. */
    data class LocalAddress(val interfaceName: String, val address: String) {
        /** Placa prin care telefonul imparte legatura pe cablu. */
        val isUsb: Boolean
            get() = USB_INTERFACES.any { interfaceName.startsWith(it) }
    }

    /**
     * Numele placilor prin care Android face partajarea prin USB.
     *
     * `rndis0` e cel obisnuit; `usb0` apare pe unele telefoane, iar `ncm0` pe
     * cele noi, care folosesc NCM in loc de RNDIS.
     */
    private val USB_INTERFACES = listOf("rndis", "usb", "ncm")

    /**
     * Toate adresele IPv4 ale telefonului, fara bucla locala.
     *
     * IPv6 lipseste pentru ca protocolul nostru tinteste o adresa scrisa de om;
     * o adresa IPv6 n-ar incapea pe ecran si n-ar scrie-o nimeni de mana.
     */
    fun localAddresses(): List<LocalAddress> = try {
        NetworkInterface.getNetworkInterfaces()
            ?.toList()
            .orEmpty()
            .filter { it.isUp && !it.isLoopback }
            .flatMap { card ->
                card.inetAddresses
                    .toList()
                    .filterIsInstance<Inet4Address>()
                    .mapNotNull { it.hostAddress }
                    .map { LocalAddress(card.name, it) }
            }
    } catch (error: SocketException) {
        // Fara retea nu e o eroare: e o lista goala, si asa o spunem.
        emptyList()
    }

    /**
     * Adresa telefonului pe cablu, daca partajarea prin USB e pornita.
     *
     * Asta nu e o ghicire, ci o dovada: placa `rndis0` exista numai cat timp
     * partajarea merge. Calculatorul primeste o adresa in aceeasi retea, de
     * obicei `192.168.42.x`, si de-acolo totul merge exact ca pe Wi-Fi --
     * fiindca receptorul asculta pe `0.0.0.0`, adica pe toate placile.
     */
    fun usbTetherAddress(): String? = localAddresses().firstOrNull { it.isUsb }?.address

    /**
     * Este cablul bagat?
     *
     * Citim starea bateriei, fiindca Android nu raspunde direct la intrebarea
     * despre cablul de date. Un cablu bagat intr-o priza de perete da acelasi
     * raspuns ca unul bagat in calculator -- de aceea textul de pe ecran spune
     * "cable detected", nu "PC detected".
     */
    fun isUsbCablePlugged(context: Context): Boolean {
        val status: Intent? = context.registerReceiver(
            null,
            IntentFilter(Intent.ACTION_BATTERY_CHANGED),
        )
        val plugged = status?.getIntExtra(BatteryManager.EXTRA_PLUGGED, -1) ?: -1
        return plugged == BatteryManager.BATTERY_PLUGGED_USB
    }
}
