package md.comic.phone.stream

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import java.net.InetAddress
import java.net.UnknownHostException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import md.comic.phone.MainActivity
import md.comic.phone.R
import md.comic.phone.device.AppLanguage

/** Ce vede interfata despre trimitere. */
data class StreamState(
    val running: Boolean = false,
    val level: Float = 0f,
    val framesSent: Long = 0L,
    val target: String = "",
    /**
     * Ce se intampla pe canalul de control. `null` cat timp nu trimitem.
     *
     * Asta raspunde la intrebarea care pana acum nu avea raspuns pe telefon:
     * ma aude cineva? Sunetul pleaca si fara canal (un calculator mai vechi nu
     * stie de el), dar atunci se vede limpede ca pleaca in gol.
     */
    val link: LinkState? = null,
    val error: String? = null,
)

/**
 * Tine microfonul deschis si trimite catre calculator, si cand ecranul e stins.
 *
 * De ce serviciu in prim-plan: din Android 9 incolo, o aplicatie trimisa in
 * fundal pierde microfonul in cateva secunde. Notificarea permanenta e pretul
 * pe care sistemul il cere ca sa te lase sa inregistrezi mai departe — si e
 * cinstit sa se vada cand telefonul asculta.
 */
class StreamService : Service() {

    private val main = Handler(Looper.getMainLooper())

    private var worker: Thread? = null

    @Volatile
    private var stopping = false

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> stopStreaming()
            ACTION_START -> startStreaming(
                address = intent.getStringExtra(EXTRA_ADDRESS).orEmpty(),
                port = intent.getIntExtra(EXTRA_PORT, 0),
            )
            else -> stopSelf()
        }
        return START_NOT_STICKY
    }

    private fun startStreaming(address: String, port: Int) {
        if (worker != null) return // deja trimitem

        createChannel()
        ServiceCompat.startForeground(
            this,
            NOTIFICATION_ID,
            buildNotification(address),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
            } else {
                0
            },
        )

        stopping = false
        publish { StreamState(running = true, target = "$address:$port") }

        worker = Thread({ stream(address, port) }, "comic-stream").also { it.start() }
    }

    private fun stream(address: String, port: Int) {
        val resolved = try {
            InetAddress.getByName(address)
        } catch (error: UnknownHostException) {
            finish("Adresa $address nu poate fi gasita.")
            return
        }

        // Canalul de control merge pe firul lui, in paralel cu sunetul.
        //
        // Sunetul nu il asteapta. Un calculator mai vechi nu stie de canal, si
        // ar fi rau sa nu mai mearga deloc cu el; iar daca nu raspunde nimeni,
        // se vede din starea canalului, limpede, in loc sa ghicesti din tacere.
        val control = ControlClient(resolved, port) { link ->
            publish { it.copy(link = link) }
        }
        val controlThread = Thread({ control.run { stopping } }, "comic-control")
            .also { it.start() }

        // Nivelul ajunge la interfata de 20 de ori pe secunda, nu de 100: ochiul
        // nu vede diferenta, iar fereastra nu se redeseneaza degeaba.
        var lastLevelAt = 0L
        val streamer = Streamer { level ->
            val now = System.currentTimeMillis()
            if (now - lastLevelAt >= LEVEL_INTERVAL_MS) {
                lastLevelAt = now
                publish { it.copy(level = level) }
            }
        }

        val outcome = streamer.run(resolved, port) { stopping }

        // Canalul se opreste dupa sunet, ca sa apuce sa-si ia ramas bun. Fara
        // asta calculatorul ar afla ca am plecat abia dupa zece secunde de
        // tacere, si ar arata un telefon care nu mai e acolo.
        stopping = true
        controlThread.join(CONTROL_STOP_MS)
        publish { it.copy(link = null) }

        when (outcome) {
            is StreamOutcome.Finished -> finish(null, outcome.framesSent)
            is StreamOutcome.Failed -> finish(outcome.reason)
        }
    }

    /**
     * Chemata de pe firul de trimitere, cand s-a terminat singur.
     *
     * Trecem prin firul principal pentru ca metodele de ciclu de viata ale unui
     * serviciu se asteapta acolo. Si eliberam `worker`: altfel, dupa o eroare,
     * o a doua apasare pe Porneste ar crede ca mai trimitem si n-ar face nimic.
     */
    private fun finish(reason: String?, framesSent: Long = 0L) {
        publish { it.copy(running = false, level = 0f, framesSent = framesSent, error = reason) }
        main.post {
            worker = null
            stopForegroundCompat()
            stopSelf()
        }
    }

    private fun stopStreaming() {
        stopping = true
        worker?.join(STOP_TIMEOUT_MS)
        worker = null
        stopForegroundCompat()
        stopSelf()
    }

    private fun stopForegroundCompat() {
        ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
    }

    override fun onDestroy() {
        stopping = true
        worker?.join(STOP_TIMEOUT_MS)
        worker = null
        publish { it.copy(running = false, level = 0f) }
        super.onDestroy()
    }

    private fun buildNotification(address: String): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val stop = PendingIntent.getService(
            this,
            1,
            Intent(this, StreamService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )

        // Notificarea se scrie in afara interfetei, deci nu trece prin
        // CompositionLocalProvider. Fara asta ar ramane in limba telefonului
        // chiar daca programul e pus pe alta.
        val vorbe = AppLanguage.localized(this)

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(vorbe.getString(R.string.notification_title))
            .setContentText(vorbe.getString(R.string.notification_text, address))
            // Pictogramele din bara de stare trebuie sa fie siluete albe, nu
            // desene colorate: sistemul le taie la o singura culoare. Cea a
            // sistemului arata corect; a noastra ar ieşi un patrat plin.
            .setSmallIcon(android.R.drawable.ic_btn_speak_now)
            .setContentIntent(open)
            .addAction(0, vorbe.getString(R.string.stop), stop)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }

    private fun createChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            AppLanguage.localized(this).getString(R.string.channel_name),
            NotificationManager.IMPORTANCE_LOW,
        )
        channel.setShowBadge(false)
        getSystemService(NotificationManager::class.java)?.createNotificationChannel(channel)
    }

    companion object {
        private const val ACTION_START = "md.comic.phone.action.START"
        private const val ACTION_STOP = "md.comic.phone.action.STOP"
        private const val EXTRA_ADDRESS = "address"
        private const val EXTRA_PORT = "port"
        private const val CHANNEL_ID = "stream"
        private const val NOTIFICATION_ID = 1
        private const val LEVEL_INTERVAL_MS = 50L
        private const val STOP_TIMEOUT_MS = 500L

        /**
         * Cat asteptam canalul de control sa-si ia ramas bun.
         *
         * Mai mult decat oprirea sunetului, fiindca aici mai e un mesaj de
         * trimis pe retea. Dar nu mult: daca reteaua a cazut, calculatorul
         * tot afla din tacere, iar utilizatorul nu trebuie sa astepte.
         */
        private const val CONTROL_STOP_MS = 1_000L

        // Starea sta aici pentru ca serviciul si fereastra sunt doua lucruri
        // separate in Android: fereastra poate fi inchisa si redeschisa, iar
        // trimiterea merge mai departe. La marimea acestei aplicatii, un singur
        // loc pentru stare e mai limpede decat un strat de legatura.
        private val mutableState = MutableStateFlow(StreamState())
        val state: StateFlow<StreamState> = mutableState.asStateFlow()

        internal fun publish(block: (StreamState) -> StreamState) = mutableState.update(block)

        fun start(context: Context, address: String, port: Int) {
            val intent = Intent(context, StreamService::class.java)
                .setAction(ACTION_START)
                .putExtra(EXTRA_ADDRESS, address)
                .putExtra(EXTRA_PORT, port)
            ContextCompat.startForegroundService(context, intent)
        }

        fun stop(context: Context) {
            val intent = Intent(context, StreamService::class.java).setAction(ACTION_STOP)
            context.startService(intent)
        }
    }
}
