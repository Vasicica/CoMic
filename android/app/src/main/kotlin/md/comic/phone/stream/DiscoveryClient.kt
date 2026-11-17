package md.comic.phone.stream

import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.SocketTimeoutException
import md.comic.phone.protocol.Discovery

/**
 * Intreaba reteaua unde este calculatorul, ca sa nu scrii adresa de mana.
 *
 * Trimite un broadcast si asculta raspunsurile pana expira timpul. Nu arunca
 * daca reteaua tace: o lista goala e un raspuns valid, iar adresa scrisa de
 * mana ramane oricand varianta sigura.
 *
 * Blocheaza pana la [timeoutMs] milisecunde, deci nu o chema din firul
 * interfetei.
 */
object DiscoveryClient {
    private const val BUFFER_BYTES = 128

    fun search(timeoutMs: Int = 1_200): List<Discovery.Computer> {
        val found = LinkedHashMap<String, Discovery.Computer>()

        try {
            DatagramSocket().use { socket ->
                socket.broadcast = true
                socket.soTimeout = timeoutMs

                val question = Discovery.QUERY.toByteArray(Charsets.US_ASCII)
                socket.send(
                    DatagramPacket(
                        question,
                        question.size,
                        InetAddress.getByName("255.255.255.255"),
                        Discovery.PORT,
                    ),
                )

                collectReplies(socket, found)
            }
        } catch (error: IOException) {
            // Retea fara broadcast, Wi-Fi inchis, ruter care filtreaza: toate
            // inseamna acelasi lucru pentru utilizator — scrie adresa de mana.
            return found.values.toList()
        }

        return found.values.toList()
    }

    private fun collectReplies(
        socket: DatagramSocket,
        found: MutableMap<String, Discovery.Computer>,
    ) {
        val buffer = ByteArray(BUFFER_BYTES)
        while (true) {
            val reply = DatagramPacket(buffer, buffer.size)
            try {
                socket.receive(reply)
            } catch (timeout: SocketTimeoutException) {
                return // am ascultat destul
            }

            val text = String(reply.data, reply.offset, reply.length, Charsets.US_ASCII)
            val address = reply.address?.hostAddress ?: continue
            val computer = Discovery.parseReply(text, address) ?: continue
            // Cheia e adresa: doua raspunsuri de la acelasi calculator nu
            // trebuie sa apara ca doua calculatoare in lista.
            found[address] = computer
        }
    }
}
