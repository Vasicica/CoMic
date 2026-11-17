package md.comic.phone.protocol

/**
 * Protocolul CoMic v1, partea de telefon. Oglinda lui `windows/src/proto.rs`.
 *
 * Amanuntul care se greseste cel mai usor: antetul este big-endian, iar
 * esantioanele PCM sunt little-endian. Asa le citeste receptorul si asa le
 * trimite si `tools/test_tone.py`, trimitatorul de referinta.
 *
 * Un cadru de 10 ms la 48 kHz mono = 480 esantioane = 960 de octeti de PCM,
 * plus 12 octeti de antet = 972 de octeti pe fir.
 */
object Packet {
    /** Magic "CM", scris pe doi octeti big-endian. */
    const val MAGIC_FIRST: Byte = 0x43 // 'C'
    const val MAGIC_SECOND: Byte = 0x4D // 'M'
    const val VERSION: Byte = 1
    const val CODEC_PCM_S16LE: Byte = 0

    const val HEADER_LEN = 12
    const val SAMPLE_RATE = 48_000
    const val FRAME_MS = 10
    const val FRAME_SAMPLES = SAMPLE_RATE / 1000 * FRAME_MS // 480
    const val FRAME_BYTES = FRAME_SAMPLES * 2 // 960

    /** Portul pe care asculta receptorul din Windows, daca nu se schimba. */
    const val AUDIO_PORT = 45_678

    /**
     * Imbraca un cadru de esantioane in pachetul CoMic.
     *
     * @param sequence numarul cadrului, de la 0, creste cu 1 la fiecare 10 ms
     * @param samples tamponul citit din microfon
     * @param count cate esantioane din tampon sunt valide
     */
    fun build(sequence: Int, samples: ShortArray, count: Int = samples.size): ByteArray {
        require(count >= 0 && count <= samples.size) {
            "count = $count nu incape in tamponul de ${samples.size}"
        }

        val packet = ByteArray(HEADER_LEN + count * 2)
        packet[0] = MAGIC_FIRST
        packet[1] = MAGIC_SECOND
        packet[2] = VERSION
        packet[3] = CODEC_PCM_S16LE
        writeBigEndian(packet, 4, sequence)
        // Marcajul de timp se numara in esantioane. Inmultirea se roteste la 32
        // de biti exact cum cere protocolul (`u32` in Rust): Int-ul din Kotlin
        // are tot 32 de biti, iar semnul nu conteaza pe fir.
        writeBigEndian(packet, 8, sequence * FRAME_SAMPLES)

        var at = HEADER_LEN
        for (index in 0 until count) {
            val sample = samples[index].toInt()
            packet[at] = (sample and 0xFF).toByte()
            packet[at + 1] = ((sample shr 8) and 0xFF).toByte()
            at += 2
        }
        return packet
    }

    private fun writeBigEndian(target: ByteArray, offset: Int, value: Int) {
        target[offset] = ((value ushr 24) and 0xFF).toByte()
        target[offset + 1] = ((value ushr 16) and 0xFF).toByte()
        target[offset + 2] = ((value ushr 8) and 0xFF).toByte()
        target[offset + 3] = (value and 0xFF).toByte()
    }
}

/**
 * Gasirea calculatorului fara sa scrii adresa de mana.
 *
 * Telefonul striga [QUERY] prin broadcast pe [PORT]; calculatorul care ruleaza
 * CoMic raspunde cu numele lui si cu portul pe care asteapta audio. Vezi
 * `serve_discovery` in `windows/src/net.rs`.
 *
 * Descoperirea e un confort, nu o cerinta: unele retele blocheaza broadcastul,
 * si atunci adresa scrisa de mana ramane singura cale.
 */
object Discovery {
    const val PORT = 45_679
    const val QUERY = "COMIC-DISCOVER-1"
    private const val REPLY_PREFIX = "COMIC-HERE-1"

    /** Un calculator care a raspuns. */
    data class Computer(
        val name: String,
        val address: String,
        val audioPort: Int,
    )

    /**
     * Desface un raspuns de forma `COMIC-HERE-1 NUMELE-PC 45678`.
     *
     * Intoarce `null` pentru orice altceva: pe portul de descoperire pot ajunge
     * si pachete de la alte programe, si nu vrem sa aratam gunoi in lista.
     */
    fun parseReply(reply: String, address: String): Computer? {
        val parts = reply.trim().split(' ').filter { it.isNotEmpty() }
        if (parts.size < 3 || parts[0] != REPLY_PREFIX) return null

        val port = parts.last().toIntOrNull() ?: return null
        if (port < 1 || port > 65_535) return null

        // Numele poate avea spatii, desi COMPUTERNAME nu are; luam tot ce e
        // intre prefix si port, ca sa nu pierdem jumatate de nume.
        val name = parts.subList(1, parts.size - 1).joinToString(" ")
        if (name.isEmpty()) return null

        return Computer(name = name, address = address, audioPort = port)
    }
}
