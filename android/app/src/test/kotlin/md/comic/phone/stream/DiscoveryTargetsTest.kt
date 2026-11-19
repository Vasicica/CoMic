package md.comic.phone.stream

import java.net.InetAddress
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Alegerea adreselor catre care pleaca intrebarea descoperirii.
 *
 * Trimiterea propriu-zisa cere o retea adevarata si nu se poate incerca aici,
 * dar partea unde se ascund greselile se poate: sa nu strigam de doua ori pe
 * acelasi drum, si sa nu uitam rezerva.
 */
class DiscoveryTargetsTest {

    private val catreToti = InetAddress.getByName("255.255.255.255")

    private fun adrese(vararg text: String) = text.map { InetAddress.getByName(it) }

    @Test
    fun `asks every network the phone is attached to`() {
        // Asa arata telefonul legat si prin Wi-Fi, si prin cablu cu partajare USB.
        val wifiSiCablu = adrese("192.168.1.255", "10.161.32.255")

        val tinte = DiscoveryClient.withFallback(wifiSiCablu)

        assertTrue("fiecare retea trebuie intrebata", tinte.containsAll(wifiSiCablu))
    }

    @Test
    fun `keeps the shout to everyone as a last resort`() {
        val tinte = DiscoveryClient.withFallback(adrese("192.168.1.255"))

        assertEquals("rezerva sta la coada, nu in fata", catreToti, tinte.last())
    }

    @Test
    fun `still asks when no network offers a broadcast address`() {
        // Unele retele nu dau adresa de broadcast. Atunci rezerva e tot ce avem,
        // si descoperirea trebuie sa mearga mai departe, nu sa taca.
        val tinte = DiscoveryClient.withFallback(emptyList())

        assertEquals(listOf(catreToti), tinte)
    }

    @Test
    fun `never shouts twice down the same road`() {
        // Doua interfete pot da aceeasi adresa de broadcast, si ea poate fi
        // chiar rezerva. Un pachet trimis de doua ori ar aduce raspunsuri
        // duble si ar incetini cautarea degeaba.
        val cuRepetitii = adrese("192.168.1.255", "192.168.1.255", "255.255.255.255")

        val tinte = DiscoveryClient.withFallback(cuRepetitii)

        assertEquals("nicio adresa de doua ori", tinte.size, tinte.toSet().size)
        assertEquals(2, tinte.size)
    }
}
