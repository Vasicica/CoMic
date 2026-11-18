package md.comic.phone.ui

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.math.PI
import kotlin.math.sin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import md.comic.phone.R
import md.comic.phone.device.PhoneNetwork
import md.comic.phone.protocol.Discovery
import md.comic.phone.protocol.Packet
import md.comic.phone.stream.DiscoveryClient
import md.comic.phone.stream.StreamService
import md.comic.phone.ui.components.CoMicCard
import md.comic.phone.ui.components.MonoText
import md.comic.phone.ui.components.NoticeCard
import md.comic.phone.ui.components.NotYetTag
import md.comic.phone.ui.components.SectionLabel
import md.comic.phone.ui.components.SegmentOption
import md.comic.phone.ui.components.Segmented
import md.comic.phone.ui.theme.CoMic
import md.comic.phone.ui.theme.Radius

/**
 * Ecranul Connect, dupa `phone-design/phone-ui.html`.
 *
 * O abatere de la design, facuta cu buna stiinta: designul arata mare IP-ul
 * **telefonului**, pentru ca acolo calculatorul se conecteaza la telefon.
 * Aplicatia merge invers -- telefonul trimite catre adresa calculatorului --
 * si asa ramane pana scriem canalul de control din `docs/CONEXIUNI.md`. Deci
 * cartonasul mare arata calculatorul ales, nu telefonul. Mai bine o abatere de
 * la macheta decat un ecran care cere ceva ce nu se intampla.
 */

private const val PREFS = "comic"
private const val KEY_ADDRESS = "address"
private const val KEY_PORT = "port"

private enum class Method { Wifi, Usb, Bluetooth }

@Composable
fun ConnectScreen() {
    val context = LocalContext.current
    val state by StreamService.state.collectAsStateWithLifecycle()
    val scope = rememberCoroutineScope()
    val prefs = remember { context.getSharedPreferences(PREFS, Context.MODE_PRIVATE) }

    var method by remember { mutableStateOf(Method.Wifi) }
    var address by remember { mutableStateOf(prefs.getString(KEY_ADDRESS, "").orEmpty()) }
    var port by remember { mutableStateOf(prefs.getInt(KEY_PORT, Packet.AUDIO_PORT).toString()) }
    var found by remember { mutableStateOf(emptyList<Discovery.Computer>()) }
    var searching by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf<String?>(null) }

    fun begin() {
        val portNumber = port.toIntOrNull()
        if (address.isBlank()) {
            notice = "Pick a PC from the list, or type its address."
            return
        }
        if (portNumber == null || portNumber !in 1..65_535) {
            notice = "The port must be a number between 1 and 65535."
            return
        }
        notice = null
        prefs.edit().putString(KEY_ADDRESS, address).putInt(KEY_PORT, portNumber).apply()
        StreamService.start(context, address.trim(), portNumber)
    }

    val askPermissions = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions(),
    ) { granted ->
        if (granted[Manifest.permission.RECORD_AUDIO] == true) {
            begin()
        } else {
            notice = "Without microphone access there is nothing to send."
        }
    }

    fun start() {
        val missing = missingPermissions(context)
        if (missing.isEmpty()) begin() else askPermissions.launch(missing.toTypedArray())
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Segmented(
            options = listOf(
                SegmentOption("Wi-Fi", R.drawable.ic_wifi),
                SegmentOption("USB", R.drawable.ic_usb),
                // Nicio linie de Bluetooth nu exista in proiect. Se vede, ca sa
                // stii ca e prevazut, dar nu se poate alege.
                SegmentOption("Bluetooth", R.drawable.ic_bt, enabled = false),
            ),
            selectedIndex = method.ordinal,
            onSelect = { method = Method.entries[it] },
        )

        Spacer(Modifier.height(28.dp))

        BigButton(
            live = state.running,
            level = state.level,
            onClick = { if (state.running) StreamService.stop(context) else start() },
        )

        Spacer(Modifier.height(20.dp))

        Headline(running = state.running, target = state.target, method = method)

        Spacer(Modifier.height(20.dp))

        if (state.running) {
            LivePanel(
                level = state.level,
                frames = state.framesSent,
                target = state.target,
                method = method,
                onDisconnect = { StreamService.stop(context) },
            )
        } else {
            when (method) {
                Method.Wifi -> WifiPanel(
                    address = address,
                    port = port,
                    found = found,
                    searching = searching,
                    onAddress = { address = it },
                    onPort = { port = it },
                    onSearch = {
                        searching = true
                        scope.launch {
                            val pcs = withContext(Dispatchers.IO) { DiscoveryClient.search() }
                            found = pcs
                            searching = false
                            notice = if (pcs.isEmpty()) {
                                "No PC answered. Type the address by hand."
                            } else {
                                null
                            }
                        }
                    },
                    onPick = { computer ->
                        address = computer.address
                        port = computer.audioPort.toString()
                        found = emptyList()
                    },
                )

                Method.Usb -> UsbPanel(
                    tetherAddress = PhoneNetwork.usbTetherAddress(),
                    cablePlugged = PhoneNetwork.isUsbCablePlugged(context),
                    onOpenSettings = { openSettings(context, Settings.ACTION_WIRELESS_SETTINGS) },
                )

                // Nu se poate ajunge aici: segmentul e stins. Ramane ca plasa de
                // siguranta, pentru ziua in care Bluetooth-ul chiar va exista.
                Method.Bluetooth -> NoticeCard("Bluetooth is not built yet.")
            }
        }

        notice?.let {
            Spacer(Modifier.height(14.dp))
            Text(it, color = CoMic.Danger, fontSize = 13.sp)
        }
        state.error?.let {
            Spacer(Modifier.height(14.dp))
            Text(it, color = CoMic.Danger, fontSize = 13.sp)
        }
    }
}

/**
 * Butonul mare, in doua inele. Doua stari, nu patru.
 *
 * Designul cere `idle`, `waiting`, `live` si `muted`. Avem doua: pornit si
 * oprit. Celelalte lipsesc fiindca nu le putem sti -- fara canal de control,
 * telefonul trimite in gol si nu afla niciodata daca il aude cineva, deci
 * n-avem cum sa deosebim "astept" de "sunt viu". Iar `muted` n-ar avea ce
 * opri: taierea sunetului nu e scrisa inca.
 */
@Composable
private fun BigButton(live: Boolean, level: Float, onClick: () -> Unit) {
    val ring = if (live) CoMic.Ok else CoMic.Accent
    Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
        Box(
            modifier = Modifier
                .size(212.dp)
                .background(ring.copy(alpha = if (live) 0.06f else 0.04f), CircleShape),
        )
        Box(
            modifier = Modifier
                .size(186.dp)
                .background(ring.copy(alpha = if (live) 0.12f else 0.07f), CircleShape),
        )
        Column(
            modifier = Modifier
                .size(160.dp)
                .background(
                    if (live) CoMic.Ok.copy(alpha = 0.14f) else CoMic.Surface,
                    CircleShape,
                )
                .clickable(onClick = onClick),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            Icon(
                painter = painterResource(R.drawable.ic_mic),
                contentDescription = null,
                tint = ring,
                modifier = Modifier
                    .size(34.dp)
                    // Pictograma se aprinde cu vocea: e singura miscare din ecran
                    // care arata ca microfonul chiar aude ceva.
                    .alpha(if (live) (0.65f + 0.35f * level).coerceIn(0f, 1f) else 1f),
            )
            Spacer(Modifier.height(8.dp))
            Text(
                if (live) "Stop" else "Start",
                color = ring,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

@Composable
private fun Headline(running: Boolean, target: String, method: Method) {
    val title = if (running) "Sending" else "Ready to stream"
    val subtitle = when {
        running -> "Microphone open, going to $target."
        method == Method.Usb -> "Plug the phone into your PC, then tap Start."
        else -> "Pick your PC below, then tap Start."
    }
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(title, color = CoMic.TextMain, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Spacer(Modifier.height(4.dp))
        Text(subtitle, color = CoMic.Muted, fontSize = 13.sp, textAlign = TextAlign.Center)
    }
}

// ---------- Wi-Fi ----------

@Composable
private fun WifiPanel(
    address: String,
    port: String,
    found: List<Discovery.Computer>,
    searching: Boolean,
    onAddress: (String) -> Unit,
    onPort: (String) -> Unit,
    onSearch: () -> Unit,
    onPick: (Discovery.Computer) -> Unit,
) {
    CoMicCard {
        Text("Sending to this PC", color = CoMic.Muted, fontSize = 12.sp)
        Spacer(Modifier.height(8.dp))
        Row(verticalAlignment = Alignment.Bottom) {
            MonoText(
                text = address.ifBlank { "not chosen" },
                color = if (address.isBlank()) CoMic.Faint else CoMic.Accent,
                size = if (address.isBlank()) 20.sp else 26.sp,
                weight = FontWeight.SemiBold,
            )
            if (address.isNotBlank()) {
                MonoText(":$port", CoMic.Muted, 14.sp)
            }
        }
        Spacer(Modifier.height(14.dp))
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(1.dp)
                .background(CoMic.Separator),
        )
        Spacer(Modifier.height(14.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Pairing code", color = CoMic.Muted, fontSize = 13.sp)
            // Imperecherea si criptarea stau in docs/PROTOCOL.md si nicaieri
            // altundeva. Un cod din sase cifre afisat aici n-ar apara nimic.
            NotYetTag()
        }
    }

    Spacer(Modifier.height(22.dp))
    SectionLabel("PCs on this network")
    Spacer(Modifier.height(10.dp))

    found.forEach { computer ->
        ComputerRow(computer) { onPick(computer) }
        Spacer(Modifier.height(8.dp))
    }

    PlainButton(
        label = if (searching) "Searching…" else "Search for PCs",
        enabled = !searching,
        onClick = onSearch,
    )

    Spacer(Modifier.height(10.dp))
    Text(
        "Not listed? Some routers block discovery — type the address by hand.",
        color = CoMic.Muted,
        fontSize = 12.sp,
        lineHeight = 17.sp,
    )

    Spacer(Modifier.height(14.dp))
    Row(verticalAlignment = Alignment.CenterVertically) {
        OutlinedTextField(
            value = address,
            onValueChange = onAddress,
            label = { Text("PC address") },
            placeholder = { Text("192.168.1.10") },
            singleLine = true,
            modifier = Modifier.weight(1f),
        )
        Spacer(Modifier.width(10.dp))
        OutlinedTextField(
            value = port,
            onValueChange = { typed -> onPort(typed.filter { it.isDigit() }) },
            label = { Text("Port") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.width(110.dp),
        )
    }
}

@Composable
private fun ComputerRow(computer: Discovery.Computer, onPick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Card))
            .padding(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_pc),
            contentDescription = null,
            tint = CoMic.Muted,
            modifier = Modifier.size(20.dp),
        )
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                computer.name,
                color = CoMic.TextMain,
                fontSize = 14.sp,
                fontWeight = FontWeight.Medium,
            )
            MonoText("${computer.address}:${computer.audioPort}", CoMic.Muted, 12.sp)
        }
        Spacer(Modifier.width(10.dp))
        Text(
            "Pick",
            color = CoMic.Bg,
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier
                .background(CoMic.Accent, RoundedCornerShape(Radius.Small))
                .clickable(onClick = onPick)
                .padding(horizontal = 14.dp, vertical = 8.dp),
        )
    }
}

// ---------- USB ----------

@Composable
private fun UsbPanel(tetherAddress: String?, cablePlugged: Boolean, onOpenSettings: () -> Unit) {
    val tethering = tetherAddress != null

    CoMicCard {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier
                        .size(7.dp)
                        .background(if (cablePlugged) CoMic.Ok else CoMic.Faint, CircleShape),
                )
                Spacer(Modifier.width(9.dp))
                Text(
                    if (cablePlugged) "Cable detected" else "Cable not detected",
                    color = CoMic.TextMain,
                    fontSize = 13.sp,
                )
            }
            Text("25–40 ms", color = CoMic.Muted, fontSize = 12.sp)
        }
    }

    Spacer(Modifier.height(14.dp))

    // Varianta care chiar merge, si merge fara nicio linie de cod noua:
    // receptorul asculta pe 0.0.0.0, deci si pe placa de pe cablu.
    MethodCard(
        title = "USB tethering",
        detail = if (tethering) {
            "On. The phone is $tetherAddress on the cable. Search for PCs above, " +
                "or type the PC's cable address."
        } else {
            "Off. Turn it on and the cable becomes a small network — everything " +
                "then works exactly like Wi-Fi, only faster."
        },
        active = tethering,
        enabled = true,
    )

    Spacer(Modifier.height(10.dp))

    MethodCard(
        title = "USB debugging",
        detail = "Needs the TCP control channel: adb only forwards TCP, and we send UDP.",
        active = false,
        enabled = false,
    )

    Spacer(Modifier.height(14.dp))
    PlainButton(label = "Open tethering settings", enabled = true, onClick = onOpenSettings)
}

@Composable
private fun MethodCard(title: String, detail: String, active: Boolean, enabled: Boolean) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Large))
            .border(
                1.dp,
                if (active) CoMic.Accent.copy(alpha = 0.55f) else CoMic.Border,
                RoundedCornerShape(Radius.Large),
            )
            .padding(16.dp)
            .alpha(if (enabled) 1f else 0.5f),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(title, color = CoMic.TextMain, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
            if (!enabled) NotYetTag()
        }
        Spacer(Modifier.height(6.dp))
        Text(detail, color = CoMic.Muted, fontSize = 12.sp, lineHeight = 17.sp)
    }
}

// ---------- cat timp trimite ----------

@Composable
private fun LivePanel(
    level: Float,
    frames: Long,
    target: String,
    method: Method,
    onDisconnect: () -> Unit,
) {
    VuMeter(level)
    Spacer(Modifier.height(16.dp))

    CoMicCard {
        // "Latency" lipseste dintre statistici, desi designul o cere: ar cere
        // masurarea drumului dus-intors, iar intors nu exista inca.
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Stat("Sent", "${frames / 100} s")
            Stat("Via", if (method == Method.Usb) "USB" else "Wi-Fi")
            Stat("To", target.substringBefore(':'))
        }
    }

    Spacer(Modifier.height(14.dp))
    NoticeCard(
        "CoMic cannot confirm the PC is hearing this yet — the phone only sends. " +
            "The TCP control channel will close that loop.",
    )

    Spacer(Modifier.height(14.dp))
    PlainButton(label = "Disconnect", enabled = true, onClick = onDisconnect)
}

@Composable
private fun Stat(label: String, value: String) {
    Column(horizontalAlignment = Alignment.Start) {
        Text(label.uppercase(), color = CoMic.Muted, fontSize = 10.sp, letterSpacing = 0.7.sp)
        Spacer(Modifier.height(3.dp))
        MonoText(value, CoMic.TextMain, 15.sp, FontWeight.Medium)
    }
}

/** Indicatorul de nivel: barele din mijloc sar mai sus decat cele de la capete. */
@Composable
private fun VuMeter(level: Float) {
    val bars = 32
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(62.dp)
            .background(CoMic.Surface, RoundedCornerShape(Radius.Card))
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.Bottom,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        for (index in 0 until bars) {
            val shape = 0.45f + 0.55f * sin(PI * (index + 0.5) / bars).toFloat()
            val height = (4f + 38f * level * shape).coerceIn(4f, 42f)
            Box(
                modifier = Modifier
                    .width(4.dp)
                    .height(height.dp)
                    .background(
                        if (level > 0.88f) CoMic.Pro else CoMic.Accent,
                        RoundedCornerShape(2.dp),
                    ),
            )
        }
    }
}

@Composable
private fun PlainButton(label: String, enabled: Boolean, onClick: () -> Unit) {
    Text(
        label,
        color = if (enabled) CoMic.TextMain else CoMic.Faint,
        fontSize = 14.sp,
        fontWeight = FontWeight.Medium,
        textAlign = TextAlign.Center,
        modifier = Modifier
            .fillMaxWidth()
            .background(CoMic.Surface, RoundedCornerShape(Radius.Large))
            .border(1.dp, CoMic.Border, RoundedCornerShape(Radius.Large))
            .then(if (enabled) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(vertical = 14.dp),
    )
}

/** Deschide o pagina din Setarile Android; daca lipseste, deschide Setarile. */
private fun openSettings(context: Context, action: String) {
    val intent = Intent(action).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    val fallback = Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    runCatching { context.startActivity(intent) }
        .onFailure { runCatching { context.startActivity(fallback) } }
}

private fun missingPermissions(context: Context): List<String> {
    val needed = mutableListOf<String>()
    if (!granted(context, Manifest.permission.RECORD_AUDIO)) {
        needed.add(Manifest.permission.RECORD_AUDIO)
    }
    // Din Android 13, notificarea serviciului se cere separat. Fara ea serviciul
    // porneste, dar nu se vede nimic si nu-l poti opri din bara de sus.
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
        !granted(context, Manifest.permission.POST_NOTIFICATIONS)
    ) {
        needed.add(Manifest.permission.POST_NOTIFICATIONS)
    }
    return needed
}

private fun granted(context: Context, permission: String): Boolean =
    ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
