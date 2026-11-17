package md.comic.phone.ui

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.math.PI
import kotlin.math.sin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import md.comic.phone.protocol.Discovery
import md.comic.phone.protocol.Packet
import md.comic.phone.stream.DiscoveryClient
import md.comic.phone.stream.StreamService

// Aceleasi culori ca fereastra de Windows, ca sa se vada ca sunt un singur
// program. Vezi windows/app/ui/index.html.
private val Bg = Color(0xFF0A0A0A)
private val Panel = Color(0xFF101010)
private val Border = Color(0xFF1C1C1C)
private val TextMain = Color(0xFFEDEDED)
private val Muted = Color(0xFF8F8F8F)
private val Accent = Color(0xFF818CF8)
private val Ok = Color(0xFF4ADE80)
private val Danger = Color(0xFFE5484D)

@Composable
fun ComicTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(
            background = Bg,
            surface = Panel,
            primary = Accent,
            onPrimary = Bg,
            onBackground = TextMain,
            onSurface = TextMain,
            error = Danger,
        ),
        content = content,
    )
}

private const val PREFS = "comic"
private const val KEY_ADDRESS = "address"
private const val KEY_PORT = "port"

@Composable
fun StreamScreen() {
    val context = LocalContext.current
    val state by StreamService.state.collectAsStateWithLifecycle()
    val scope = rememberCoroutineScope()
    val prefs = remember { context.getSharedPreferences(PREFS, Context.MODE_PRIVATE) }

    var address by remember { mutableStateOf(prefs.getString(KEY_ADDRESS, "").orEmpty()) }
    var port by remember { mutableStateOf(prefs.getInt(KEY_PORT, Packet.AUDIO_PORT).toString()) }
    var found by remember { mutableStateOf(emptyList<Discovery.Computer>()) }
    var searching by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf<String?>(null) }

    fun begin() {
        val portNumber = port.toIntOrNull()
        if (address.isBlank()) {
            notice = "Scrie adresa calculatorului sau apasa Caută."
            return
        }
        if (portNumber == null || portNumber < 1 || portNumber > 65_535) {
            notice = "Portul trebuie sa fie un numar intre 1 si 65535."
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
            notice = "Fara acces la microfon nu se poate trimite nimic."
        }
    }

    fun start() {
        val missing = missingPermissions(context)
        if (missing.isEmpty()) begin() else askPermissions.launch(missing.toTypedArray())
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Bg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
    ) {
        Text("CoMic", color = TextMain, fontSize = 30.sp, fontWeight = FontWeight.SemiBold)
        Text("Microfonul telefonului, trimis la calculator.", color = Muted, fontSize = 14.sp)

        Spacer(Modifier.height(24.dp))

        LevelMeter(level = if (state.running) state.level else 0f)

        Spacer(Modifier.height(20.dp))

        StatusLine(running = state.running, target = state.target, frames = state.framesSent)

        Spacer(Modifier.height(20.dp))

        OutlinedTextField(
            value = address,
            onValueChange = { address = it },
            label = { Text("Adresa calculatorului") },
            placeholder = { Text("192.168.1.42") },
            singleLine = true,
            enabled = !state.running,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(12.dp))

        Row(verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(
                value = port,
                onValueChange = { typed -> port = typed.filter { it.isDigit() } },
                label = { Text("Port") },
                singleLine = true,
                enabled = !state.running,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                modifier = Modifier.width(140.dp),
            )
            Spacer(Modifier.width(12.dp))
            OutlinedButton(
                onClick = {
                    searching = true
                    scope.launch {
                        val computers = withContext(Dispatchers.IO) { DiscoveryClient.search() }
                        found = computers
                        searching = false
                        notice = if (computers.isEmpty()) {
                            "Nu am gasit niciun calculator. Scrie adresa de mana."
                        } else {
                            null
                        }
                    }
                },
                enabled = !state.running && !searching,
            ) {
                Text(if (searching) "Caut…" else "Caută calculatorul")
            }
        }

        if (found.isNotEmpty()) {
            Spacer(Modifier.height(12.dp))
            found.forEach { computer ->
                ComputerRow(computer) {
                    address = computer.address
                    port = computer.audioPort.toString()
                    found = emptyList()
                }
            }
        }

        notice?.let {
            Spacer(Modifier.height(12.dp))
            Text(it, color = Danger, fontSize = 13.sp)
        }

        state.error?.let {
            Spacer(Modifier.height(12.dp))
            Text(it, color = Danger, fontSize = 13.sp)
        }

        Spacer(Modifier.height(24.dp))

        Button(
            onClick = { if (state.running) StreamService.stop(context) else start() },
            colors = ButtonDefaults.buttonColors(
                containerColor = if (state.running) Danger else Accent,
                contentColor = Bg,
            ),
            shape = RoundedCornerShape(14.dp),
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp),
        ) {
            Text(
                if (state.running) "Opreste" else "Porneste",
                fontSize = 17.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }

        Spacer(Modifier.height(16.dp))

        Text(
            "Pe calculator deschide CoMic si apasa Connect. Telefonul trimite " +
                "catre adresa de mai sus, pe portul ales.",
            color = Muted,
            fontSize = 12.sp,
        )
    }
}

@Composable
private fun StatusLine(running: Boolean, target: String, frames: Long) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(10.dp)
                .background(if (running) Ok else Border, RoundedCornerShape(5.dp)),
        )
        Spacer(Modifier.width(8.dp))
        Column {
            Text(
                if (running) "Trimit catre $target" else "Oprit",
                color = TextMain,
                fontSize = 15.sp,
            )
            if (running) {
                // Un cadru = 10 ms, deci secundele se citesc direct din numar.
                Text("${frames / 100} s de sunet trimis", color = Muted, fontSize = 12.sp)
            }
        }
    }
}

@Composable
private fun ComputerRow(computer: Discovery.Computer, onPick: () -> Unit) {
    Card(
        colors = CardDefaults.cardColors(containerColor = Panel),
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 8.dp),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(modifier = Modifier.fillMaxWidth(0.7f)) {
                Text(computer.name, color = TextMain, fontSize = 15.sp)
                Text("${computer.address}:${computer.audioPort}", color = Muted, fontSize = 12.sp)
            }
            Spacer(Modifier.width(8.dp))
            OutlinedButton(onClick = onPick) { Text("Alege") }
        }
    }
}

/**
 * Indicatorul de nivel. Barele din mijloc sar mai sus decat cele de la capete,
 * ca sa arate ca un VU-metru si nu ca un dreptunghi care creste.
 */
@Composable
private fun LevelMeter(level: Float) {
    val bars = 28
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(64.dp)
            .background(Panel, RoundedCornerShape(12.dp))
            .padding(horizontal = 10.dp, vertical = 10.dp),
        verticalAlignment = Alignment.Bottom,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        for (index in 0 until bars) {
            val shape = 0.45f + 0.55f * sin(PI * (index + 0.5) / bars).toFloat()
            val height = (6f + 38f * level * shape).coerceIn(6f, 44f)
            Box(
                modifier = Modifier
                    .width(5.dp)
                    .height(height.dp)
                    .background(
                        if (level > 0.88f) Danger else Accent,
                        RoundedCornerShape(3.dp),
                    ),
            )
        }
    }
}

private fun missingPermissions(context: Context): List<String> {
    val needed = mutableListOf<String>()
    if (!granted(context, Manifest.permission.RECORD_AUDIO)) {
        needed.add(Manifest.permission.RECORD_AUDIO)
    }
    // Din Android 13, notificarea serviciului se cere separat. Fara ea serviciul
    // porneste, dar utilizatorul nu vede nimic si nu-l poate opri din bara de sus.
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
        !granted(context, Manifest.permission.POST_NOTIFICATIONS)
    ) {
        needed.add(Manifest.permission.POST_NOTIFICATIONS)
    }
    return needed
}

private fun granted(context: Context, permission: String): Boolean =
    ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
