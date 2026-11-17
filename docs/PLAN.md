# CoMic — microfon de telefon pentru Windows

Inlocuitor propriu pentru WO Mic: Android ca microfon, Windows ca receptor.
Fara limite de timp, fara abonament, fara opriri cu ecranul stins.

## Decizii luate (2026-11-17)

| Decizie | Alegere | Motiv |
|---|---|---|
| Platforme | Android + Windows | iOS costa 99 $/an si restricteaza microfonul in fundal |
| Limbaj Windows | Rust | un singur .exe, latenta mica, acces direct la driver in Faza 2 |
| Limbaj Android | Kotlin nativ | control real pe AudioRecord/AAudio, imposibil in Flutter/RN |
| Microfon virtual (v1) | VB-Cable | cost zero; driver propriu doar la monetizare |
| Codec | Opus, cadre de 10 ms | calitate mai buna la banda mai mica decat WO Mic |
| Transport | Wi-Fi cu IP scris manual, USB, Bluetooth | utilizatorul alege metoda, ca in WO Mic — vezi [CONEXIUNI.md](CONEXIUNI.md) |
| Wi-Fi Direct | exclus | suport inconsecvent, nu aduce nimic peste Wi-Fi sau USB |
| Voice changer | doua voci, proba gratuita, folosire platita | vezi [VOICE-CHANGER.md](VOICE-CHANGER.md) |
| Obiectiv v1 | instrument personal | arhitectura pregatita pentru licentiere, fara cod de plati acum |

## Arhitectura

    [Android]                          [Windows]
    AudioRecord 48k mono
      -> cadre 10 ms
      -> encoder Opus          UDP      -> jitter buffer adaptiv 20-60 ms
      -> criptare AES-GCM   =========>  -> decriptare + decoder Opus (cu PLC)
      -> socket UDP                     -> WASAPI render in VB-Cable
    Foreground Service                  -> orice app vede "CABLE Input" ca microfon
    (tip microphone, WakeLock,
     WifiLock, reconnect)

Piesa care face produsul sa existe este a treia: driverul audio virtual.
Fara el, sunetul ajunge in calculator dar nicio aplicatie nu-l vede ca microfon.
In v1 rolul lui e jucat de VB-Cable.

## Faze

### Faza 0 — dovada conceptului
Scop: sa auzim vocea si sa masuram latenta reala in casa.
Varianta fara lant de unelte: pagina web pe telefon (getUserMedia + AudioWorklet)
-> WebSocket -> receptor Python pe Windows -> redare.
Dovedeste traseul de retea si redarea inainte sa instalam ceva.
Rezultat asteptat: voce inteligibila, latenta masurata.

### Faza 1 — produs utilizabil personal
- Android: Kotlin, Opus, Foreground Service tip `microphone`, WakeLock + WifiLock,
  scutire de la optimizarea bateriei, auto-reconnect la schimbare de retea
- Windows: Rust, jitter buffer adaptiv, descoperire automata (mDNS/broadcast),
  GUI cu VU meter, selectie device de iesire, gain, hotkey de mute, autostart
- Protocol: header 12 bytes + payload Opus, pairing prin cod QR / 6 cifre,
  criptare AES-GCM (altfel oricine de pe acelasi Wi-Fi asculta microfonul)
In acest punct WO Mic se dezinstaleaza.

### Faza 2 — driver audio virtual propriu
Bazat pe SysVAD / Virtual Audio Driver. Necesita certificat EV code-signing
(300-600 EUR/an) + Microsoft Partner Center (~19 $) pentru attestation signing.
Fara semnare, driverul nu se instaleaza pe Windows 10/11.
Se face doar cand exista decizia de a vinde — se plateste din primele vanzari.

### Faza 3 — monetizare
Chei de licenta semnate Ed25519 (verificabile offline), trial, Google Play (25 $),
instalator semnat, Lemon Squeezy sau Paddle ca merchant of record (ei platesc TVA).

### Faza 4 — avantaj competitiv
Mod invers (boxele telefonului ca difuzor PC), multi-telefon, noise suppression
(RNNoise / WebRTC APM), mai tarziu camera ca webcam.

## Latenta asteptata

| Transport | Latenta |
|---|---|
| USB tethering | 25-40 ms |
| Wi-Fi 5 GHz | 50-80 ms |
| Wi-Fi 2.4 GHz aglomerat | 80-150 ms |

Suficient pentru apeluri, streaming, dictare, voice-over.
Insuficient pentru monitorizare live in casti in timp ce canti — se spune cinstit
in documentatie, nu se ascunde.

## Limite de sistem, nu bug-uri

- Android lasa un singur app sa tina microfonul. La un apel telefonic pierdem
  microfonul. Se gestioneaza elegant (pauza + reluare automata), nu se ascunde.
- Windows nu permite microfoane virtuale fara driver in kernel. Nu exista
  scurtatura curata; de aceea VB-Cable in v1 si driver propriu in Faza 2.

## Legal

Reproducem functionalitatea WO Mic — legal. Nu copiem cod si nu folosim numele lor.
Numele propriu trebuie verificat sa fie liber pe Google Play inainte de Faza 3.

## Costuri

| Element | Cost | Faza |
|---|---|---|
| Dezvoltare Faza 0-1 | 0 EUR | acum |
| Google Play Developer | 25 $ o data | 3 |
| Certificat EV code-signing | 300-600 EUR/an | 2-3 |
| Microsoft Partner Center | ~19 $ o data | 2 |
| Procesator de plati | ~5% + comision | 3 |
| Domeniu | ~15 EUR/an | 3 |
