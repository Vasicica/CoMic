package md.comic.phone.protocol

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test

/**
 * Octetii asteptati nu sunt ghiciti: au fost generati cu acelasi calcul pe care
 * il face `tools/test_tone.py`, trimitatorul de referinta pe care receptorul
 * din Windows il accepta deja. Daca un test de aici cade, pachetul nostru nu
 * mai seamana cu ce stie sa citeasca `windows/src/proto.rs`.
 */
class PacketTest {
    private fun hex(bytes: ByteArray): String =
        bytes.joinToString(" ") { String.format("%02X", it) }

    @Test
    fun `cadrul arata exact cum il asteapta receptorul`() {
        // Arrange
        val samples = shortArrayOf(-32_768, 0, 32_767)

        // Act
        val packet = Packet.build(sequence = 7, samples = samples)

        // Assert: antet big-endian, esantioane little-endian
        assertEquals(
            "43 4D 01 00 00 00 00 07 00 00 0D 20 00 80 00 00 FF 7F",
            hex(packet),
        )
    }

    @Test
    fun `un cadru plin are 972 de octeti`() {
        val packet = Packet.build(sequence = 0, samples = ShortArray(Packet.FRAME_SAMPLES))

        assertEquals(Packet.HEADER_LEN + Packet.FRAME_BYTES, packet.size)
        assertEquals(972, packet.size)
    }

    @Test
    fun `marcajul de timp creste cu un cadru intreg la fiecare pachet`() {
        // Cadrul 1 are marcajul 480 = 0x000001E0.
        val packet = Packet.build(sequence = 1, samples = shortArrayOf(1))

        assertEquals("00 00 01 E0", hex(packet.copyOfRange(8, 12)))
    }

    @Test
    fun `numarul de secventa se scrie pe patru octeti big-endian`() {
        val packet = Packet.build(sequence = 0x01020304, samples = ShortArray(0))

        assertEquals("01 02 03 04", hex(packet.copyOfRange(4, 8)))
    }

    @Test
    fun `trimite numai partea valida din tampon`() {
        // Microfonul a dat doar doua esantioane, desi tamponul e de patru.
        val samples = shortArrayOf(1, 2, 999, 999)

        val packet = Packet.build(sequence = 0, samples = samples, count = 2)

        assertEquals(Packet.HEADER_LEN + 4, packet.size)
        assertEquals("01 00 02 00", hex(packet.copyOfRange(12, 16)))
    }

    @Test
    fun `refuza un count mai mare decat tamponul`() {
        assertThrows(IllegalArgumentException::class.java) {
            Packet.build(sequence = 0, samples = ShortArray(10), count = 11)
        }
    }

    @Test
    fun `secventa mare nu arunca, se scrie asa cum e`() {
        // La 100 de cadre pe secunda, marcajul se roteste dupa vreo 12 zile.
        // Protocolul cere rotire, nu eroare: receptorul citeste un u32.
        val packet = Packet.build(sequence = Int.MAX_VALUE, samples = shortArrayOf(0))

        assertEquals(Packet.HEADER_LEN + 2, packet.size)
        assertEquals("7F FF FF FF", hex(packet.copyOfRange(4, 8)))
    }
}

class DiscoveryTest {
    @Test
    fun `citeste raspunsul calculatorului`() {
        val found = Discovery.parseReply("COMIC-HERE-1 VASILE-PC 45678", "192.168.1.5")

        assertEquals(
            Discovery.Computer(name = "VASILE-PC", address = "192.168.1.5", audioPort = 45_678),
            found,
        )
    }

    @Test
    fun `pastreaza numele cu spatii intreg`() {
        val found = Discovery.parseReply("COMIC-HERE-1 PC din camera 45678", "10.0.0.2")

        assertEquals("PC din camera", found?.name)
        assertEquals(45_678, found?.audioPort)
    }

    @Test
    fun `nu ia in seama pachetele altor programe`() {
        assertNull(Discovery.parseReply("SSDP-DISCOVER * HTTP/1.1", "10.0.0.3"))
        assertNull(Discovery.parseReply("", "10.0.0.3"))
        assertNull(Discovery.parseReply("COMIC-HERE-1", "10.0.0.3"))
    }

    @Test
    fun `refuza un port care nu e numar sau e in afara intervalului`() {
        assertNull(Discovery.parseReply("COMIC-HERE-1 PC saizecicinci", "10.0.0.4"))
        assertNull(Discovery.parseReply("COMIC-HERE-1 PC 0", "10.0.0.4"))
        assertNull(Discovery.parseReply("COMIC-HERE-1 PC 70000", "10.0.0.4"))
    }
}
