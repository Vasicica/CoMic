package md.comic.phone.stream

import android.os.Build
import java.io.BufferedReader
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException
import org.json.JSONException
import org.json.JSONObject

/**
 * Canalul de control: telefonul si calculatorul stiu unul de altul.
 *
 * Pana la el, telefonul striga in gol. Trimitea UDP catre o adresa si nu afla
 * niciodata daca asculta cineva: o adresa gresita, un Firewall inchis sau un
 * CoMic nepornit aratau toate la fel, adica tacere. Canalul asta raspunde la
 * singura intrebare care conteaza pe telefon: **ma aude cineva?**
 *
 * TCP pe acelasi numar de port pe care audio circula prin UDP. Mesajele sunt
 * JSON, cate unul pe linie. Oglinda lui `windows/src/control.rs`; formatul e
 * scris in `docs/PROTOCOL.md`.
 *
 * Noi facem PING, calculatorul raspunde PONG: asa masuram chiar noi
 * dus-intorsul, si tot asa afla si el ca am plecat, din tacerea noastra.
 */
class ControlClient(
    private val address: InetAddress,
    private val port: Int,
    private val onState: (LinkState) -> Unit,
) {

    @Volatile
    private var muted = false

    @Volatile
    private var mutedSent = true

    /**
     * Tine legatura pana cand [stop] devine adevarat. Blocheaza firul curent,
     * deci nu o chema din firul interfetei.
     */
    fun run(stop: () -> Boolean) {
        onState(LinkState.Connecting)

        val link = try {
            Socket().also {
                it.connect(InetSocketAddress(address, port), CONNECT_TIMEOUT_MS)
                it.soTimeout = READ_TIMEOUT_MS
                it.tcpNoDelay = true
            }
        } catch (error: IOException) {
            // Cel mai folositor mesaj din toata aplicatia: nu "ceva nu merge",
            // ci "la adresa aia nu raspunde nimeni".
            onState(LinkState.Unreachable(error.message ?: "no answer"))
            return
        }

        try {
            link.use { converse(it, stop) }
        } catch (error: IOException) {
            onState(LinkState.Unreachable(error.message ?: "connection lost"))
        }
    }

    /** Anunta calculatorul ca microfonul e taiat (sau dezlegat). */
    fun setMuted(on: Boolean) {
        if (muted != on) {
            muted = on
            mutedSent = false
        }
    }

    private fun converse(link: Socket, stop: () -> Boolean) {
        val out = link.getOutputStream()
        val reader = link.getInputStream().bufferedReader()

        out.write(
            line(
                JSONObject()
                    .put("t", "hello")
                    .put("v", VERSION)
                    .put("name", phoneName())
                    .put("rate", 48_000)
                    .put("codec", "pcm"),
            ),
        )
        out.flush()

        var pc = ""
        var audioPort = port
        var rtt: Int? = null
        var seq = 0
        var pingAt = 0L
        var nextPing = 0L

        while (!stop()) {
            // Mutul pleaca imediat ce se schimba, nu la urmatoarea bataie.
            if (!mutedSent) {
                out.write(line(JSONObject().put("t", "mute").put("on", muted)))
                out.flush()
                mutedSent = true
            }

            val now = System.currentTimeMillis()
            if (pc.isNotEmpty() && now >= nextPing) {
                seq += 1
                pingAt = now
                nextPing = now + PING_INTERVAL_MS
                val ping = JSONObject().put("t", "ping").put("seq", seq)
                rtt?.let { ping.put("rtt", it) }
                out.write(line(ping))
                out.flush()
            }

            val message = nextMessage(reader) ?: continue
            when (message.optString("t")) {
                "hello-ack" -> {
                    pc = message.optString("name", "PC")
                    audioPort = message.optInt("port", port)
                    nextPing = 0L // prima bataie, fara intarziere
                    onState(LinkState.Linked(pc, audioPort, null))
                }
                "pong" -> {
                    rtt = (System.currentTimeMillis() - pingAt).toInt()
                    onState(LinkState.Linked(pc, audioPort, rtt))
                }
                "error" -> {
                    onState(LinkState.Refused(message.optString("reason", "refused")))
                    return
                }
            }
        }

        sayGoodbye(link)
    }

    /**
     * Urmatorul mesaj, sau `null` daca n-a venit nimic in rastimpul asta.
     *
     * Tacerea nu e o eroare: asa aflam ca putem verifica daca ni s-a cerut sa
     * ne oprim, si tot asa trimitem bataia urmatoare.
     */
    private fun nextMessage(reader: BufferedReader): JSONObject? = try {
        val line = reader.readLine() ?: throw IOException("connection closed")
        if (line.isBlank()) null else JSONObject(line)
    } catch (_: SocketTimeoutException) {
        null
    } catch (_: JSONException) {
        // O linie pe care n-o intelegem nu e motiv sa rupem legatura: poate fi
        // un mesaj dintr-o versiune mai noua de calculator.
        null
    }

    /**
     * Ramas bun, cu greselile inghitite.
     *
     * Daca a cazut reteaua, calculatorul tot afla ca am plecat: dupa zece
     * secunde de tacere. Mesajul asta doar il scuteste de asteptare.
     */
    private fun sayGoodbye(link: Socket) {
        try {
            val out = link.getOutputStream()
            out.write(line(JSONObject().put("t", "bye")))
            out.flush()
        } catch (_: IOException) {
        }
        onState(LinkState.Gone)
    }

    private fun line(message: JSONObject): ByteArray = (message.toString() + "\n").toByteArray()

    /**
     * Cum ne prezentam. `Build.MODEL` e numele comercial ("Pixel 7"), il stie
     * utilizatorul si nu cere nicio permisiune.
     */
    private fun phoneName(): String = Build.MODEL?.trim().orEmpty().ifEmpty { "Phone" }

    companion object {
        const val VERSION = 1
        private const val CONNECT_TIMEOUT_MS = 3_000
        private const val READ_TIMEOUT_MS = 500
        private const val PING_INTERVAL_MS = 2_000L
    }
}

/** Ce stie telefonul despre calculatorul de la celalalt capat. */
sealed interface LinkState {
    /** Incercam sa ne legam. */
    data object Connecting : LinkState

    /** Ne-am legat. [rttMs] lipseste pana la primul dus-intors masurat. */
    data class Linked(val pcName: String, val audioPort: Int, val rttMs: Int?) : LinkState

    /** Calculatorul ne-a refuzat si a spus de ce (de obicei alta versiune). */
    data class Refused(val reason: String) : LinkState

    /**
     * Nu raspunde nimeni la adresa aia. Adresa gresita, Firewall inchis, sau
     * CoMic nepornit pe calculator.
     */
    data class Unreachable(val reason: String) : LinkState

    /** Ne-am luat ramas bun. */
    data object Gone : LinkState
}
