package md.comic.phone.stream

import android.Manifest
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import androidx.annotation.RequiresPermission
import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import kotlin.math.min
import kotlin.math.sqrt
import md.comic.phone.protocol.Packet

/** Cum s-a terminat o sesiune de trimitere. */
sealed interface StreamOutcome {
    /** Oprit de utilizator, dupa ce a trimis [framesSent] cadre. */
    data class Finished(val framesSent: Long) : StreamOutcome

    /** S-a oprit singur. [reason] e scris pentru ochii utilizatorului. */
    data class Failed(val reason: String) : StreamOutcome
}

/**
 * Citeste microfonul, taie sunetul in cadre de 10 ms si le trimite prin UDP.
 *
 * Bucla e scrisa ca sa blocheze: AudioRecord da inapoi exact cat sunet s-a
 * adunat, deci citirea insasi tine ritmul. Nu exista `sleep` si nici ceas
 * propriu — daca am dormi, am adauga intarziere peste cea a microfonului.
 *
 * Firul pe care rulezi [run] e al ei pana la oprire, deci nu o chema din firul
 * interfetei.
 */
class Streamer(private val onLevel: (Float) -> Unit) {

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    fun run(address: InetAddress, port: Int, stop: () -> Boolean): StreamOutcome {
        val record = openMicrophone() ?: return StreamOutcome.Failed(
            "Microfonul nu a putut fi deschis. Verifica permisiunea si daca il foloseste alta aplicatie.",
        )

        return try {
            DatagramSocket().use { socket ->
                record.startRecording()
                if (record.recordingState != AudioRecord.RECORDSTATE_RECORDING) {
                    StreamOutcome.Failed("Microfonul nu a pornit inregistrarea.")
                } else {
                    pump(record, socket, address, port, stop)
                }
            }
        } catch (error: IOException) {
            StreamOutcome.Failed(error.message ?: "Reteaua a refuzat trimiterea.")
        } catch (error: SecurityException) {
            StreamOutcome.Failed("Aplicatia nu are voie sa foloseasca microfonul.")
        } finally {
            // Oprirea si eliberarea merg impreuna: un AudioRecord nelasat liber
            // tine microfonul ocupat pentru celelalte aplicatii.
            runCatching { record.stop() }
            record.release()
        }
    }

    private fun pump(
        record: AudioRecord,
        socket: DatagramSocket,
        address: InetAddress,
        port: Int,
        stop: () -> Boolean,
    ): StreamOutcome {
        val frame = ShortArray(Packet.FRAME_SAMPLES)
        var sequence = 0
        var framesSent = 0L

        while (!stop()) {
            val filled = readFullFrame(record, frame, stop)
            if (filled < 0) {
                return StreamOutcome.Failed("Citirea din microfon a dat eroare ($filled).")
            }
            if (filled == 0) continue // oprire ceruta in timpul citirii

            val packet = Packet.build(sequence, frame, filled)
            socket.send(DatagramPacket(packet, packet.size, address, port))

            onLevel(levelOf(frame, filled))
            sequence++
            framesSent++
        }

        return StreamOutcome.Finished(framesSent)
    }

    /**
     * Umple un cadru intreg. AudioRecord poate da mai putin decat ceri, deci
     * citim pana se umple: un cadru scurt ar face receptorul sa creada ca s-a
     * pierdut sunet pe drum.
     */
    private fun readFullFrame(record: AudioRecord, frame: ShortArray, stop: () -> Boolean): Int {
        var filled = 0
        while (filled < frame.size) {
            if (stop()) return filled
            val read = record.read(frame, filled, frame.size - filled)
            if (read < 0) return read
            if (read == 0) return filled
            filled += read
        }
        return filled
    }

    /** Nivelul pentru indicatorul din interfata: media patratica, 0..1. */
    private fun levelOf(frame: ShortArray, count: Int): Float {
        if (count == 0) return 0f
        var sum = 0.0
        for (index in 0 until count) {
            val sample = frame[index].toDouble()
            sum += sample * sample
        }
        val rms = sqrt(sum / count) / Short.MAX_VALUE
        return min(1.0, rms).toFloat()
    }

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    private fun openMicrophone(): AudioRecord? {
        // Tamponul intern al sistemului: cat ceri tu, dar niciodata sub minimul
        // cerut de telefon. Patru cadre dau loc de respiro daca firul intarzie
        // o clipa, fara sa adauge intarziere cand totul merge bine.
        val minimum = AudioRecord.getMinBufferSize(
            Packet.SAMPLE_RATE,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_16BIT,
        )
        if (minimum <= 0) return null
        val bufferBytes = maxOf(minimum, Packet.FRAME_BYTES * 4)

        val record = try {
            AudioRecord(
                // VOICE_COMMUNICATION aduce anularea ecoului si reducerea
                // zgomotului din telefon. Conteaza aici: sunetul prelucrat iese
                // din boxele calculatorului si s-ar putea intoarce in microfon.
                MediaRecorder.AudioSource.VOICE_COMMUNICATION,
                Packet.SAMPLE_RATE,
                AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT,
                bufferBytes,
            )
        } catch (error: IllegalArgumentException) {
            return null
        }

        if (record.state != AudioRecord.STATE_INITIALIZED) {
            record.release()
            return null
        }
        return record
    }
}
