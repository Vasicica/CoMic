package md.comic.phone.stream

import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.NetworkInterface
import java.net.SocketTimeoutException
import java.util.Collections
import md.comic.phone.protocol.Discovery

/**
 * Intreaba reteaua unde este calculatorul, ca sa nu scrii adresa de mana.
 *
 * Intreaba pe **fiecare** retea la care e legat telefonul, nu doar pe cea
 * implicita. Asta conteaza cand telefonul e si pe Wi-Fi, si legat prin cablu
 * cu partajare USB: un singur strigat catre `255.255.255.255` pleaca numai pe
 * interfata implicita, asa ca descoperirea gasea calculatorul la adresa lui de
 * Wi-Fi chiar cand aveai cablul in mana. Alegeai acel calculator si mergeai
 * prin aer, crezand ca mergi prin fir.
 *
 * Acelasi calculator poate raspunde acum de doua ori, o data pe fiecare drum.
 * Asa si trebuie: sunt doua cai diferite catre el, cu latente diferite, si
 * alegerea e a ta.
 *
 * Nu arunca daca reteaua tace: o lista goala e un raspuns valid, iar adresa
 * scrisa de mana ramane oricand varianta sigura.
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
                for (target in broadcastTargets()) {
                    // O retea care refuza broadcastul nu trebuie sa le opreasca
                    // pe celelalte: fiecare strigat isi tine greselile pentru el.
                    runCatching {
                        socket.send(
                            DatagramPacket(question, question.size, target, Discovery.PORT),
                        )
                    }
                }

                collectReplies(socket, found)
            }
        } catch (error: IOException) {
            // Retea fara broadcast, Wi-Fi inchis, ruter care filtreaza: toate
            // inseamna acelasi lucru pentru utilizator, adica scrie adresa de mana.
            return found.values.toList()
        }

        return found.values.toList()
    }

    /**
     * Adresa de broadcast a fiecarei retele la care suntem legati.
     *
     * `255.255.255.255` ramane la coada, ca ultima incercare: unele retele nu
     * dau o adresa de broadcast proprie, si atunci ea e tot ce avem.
     */
    private fun broadcastTargets(): List<InetAddress> = runCatching {
        Collections.list(NetworkInterface.getNetworkInterfaces())
            .filter { it.isUp && !it.isLoopback }
            .flatMap { iface -> iface.interfaceAddresses.mapNotNull { it.broadcast } }
    }.getOrDefault(emptyList())
        .let { gasite -> withFallback(gasite) }

    /**
     * Scoate adresele repetate si pune la coada strigatul catre toata lumea.
     *
     * Separata ca sa poata fi incercata fara retea: aici se ascunde greseala
     * de a trimite de doua ori pe acelasi drum, sau de a uita rezerva.
     */
    internal fun withFallback(found: List<InetAddress>): List<InetAddress> {
        val targets = LinkedHashSet(found)
        runCatching { targets.add(InetAddress.getByName("255.255.255.255")) }
        return targets.toList()
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
