# CoMic — design pentru aplicația de telefon (Android)

Interfața aplicației Android, în același stil ca aplicația de PC din `../design/`.
Acest folder este **sursa de adevăr pentru UI-ul de telefon**: când construiești ecranele în `android/` (Kotlin, `md.comic.phone.ui`), copiază de aici culorile, dimensiunile, textele și stările.

## Ce e aici

| Fișier | Ce este |
|---|---|
| `phone-ui.html` | Prototip complet, interactiv (HTML + CSS + JS, fără build). Deschide-l în browser; ecranul are 390 × 844. |
| `assets/avatars/*.jpg` | Avatarele vocilor, 480×480, numite după `id`-ul din `windows/src/effects/voices.rs`. |
| `README.md` | Specificația de mai jos. |

## Decizii de produs reflectate în design

Decis 2026-11-18:
- **Gratuit:** microfonul, fără limită de timp, **toate** metodele de conectare (Wi-Fi, USB, Bluetooth) și vocile **Normal, Pirate (`pirat`), Grandpa (`mosneag`)** — folosite de-adevăratelea, nu doar ca probă.
- **Pro** (Google Play Billing), două variante: **$1.99 pe lună** (abonament) sau **$9.99 o singură dată** (pe viață). Ecranul de plată cu ambele variante e pe canvas (Phone, efecte gratuit). Detalii și regulile Google pentru abonamente: `docs/VOICE-CHANGER.md`.
- O singură cumpărătură deblochează **și PC-ul**: telefonul îi spune receptorului pe canalul de control că e Pro.
- Vocea se schimbă **de pe ambele dispozitive**, PC și telefon; o schimbare pe unul apare imediat pe celălalt.
- Procesarea vocii rămâne pe PC (vezi `docs/VOICE-CHANGER.md`); telefonul trimite `id`-ul vocii și valorile cursoarelor pe canalul de control.
- Fluxul de conectare urmează `docs/CONEXIUNI.md`: pe telefon apeși **Start**, telefonul afișează mare IP-ul și ascultă pe 45678, PC-ul se conectează. Wi-Fi Direct nu există în UI (exclus).

## Structură

Ecran 390 × 844, de sus în jos:
1. **App bar** 64 px — logo 28 px (accent) + „CoMic”; dreapta: badge `PRO` (galben) dacă e Pro, altfel butonul „Get Pro” (duce la Effects).
2. **Conținut** — padding 20 px lateral, derulare verticală.
3. **Tab bar** 78 px, fundal `#080808` — `Connect`, `Effects`, `Settings`. Tab activ: text `#F5F5F5` + pastilă 56×30 accent 18%.

## Ecranul Connect

- **Selector metodă** (segmented, 3): Wi-Fi · USB · Bluetooth.
- **Butonul mare** (160 px, în două inele 212 / 186 px) — are 4 stări:

| Stare | Etichetă | Aspect | La apăsare |
|---|---|---|---|
| `idle` | Start | gri închis, iconiță accent, umbră accent | → `waiting` (pornește Foreground Service, ascultă pe 45678) |
| `waiting` | Stop | inele accent 6% / 12%, puls lent | → `idle` |
| `live` | Live | verde `#4ADE80` 14%, inele verzi | mute / unmute |
| `live` + mut | Muted | gri, iconiță mic tăiat | unmute |

- Sub buton: titlu 18 px + explicație 13 px, diferite pe metodă și stare (textele exacte sunt în prototip).
- **Detalii când nu e conectat**, după metodă:
  - **Wi-Fi:** card cu IP-ul telefonului mare (JetBrains Mono 30 px, accent când așteaptă) + `:45678` + numele rețelei + **cod de împerechere** din 6 cifre; listă „PCs on this network” (descoperire prin broadcast 45679) cu buton Connect; notă că unele routere blochează descoperirea → scrii IP-ul de mână pe PC.
  - **USB:** stare cablu (detectat / nu); alegere între **USB debugging** (adb forward) și **USB tethering**, ca radio-carduri; buton care deschide setarea Android potrivită.
  - **Bluetooth:** listă PC-uri împerecheate (în rază / nu); avertisment galben: împerechere întâi din Windows, latență 100–200 ms; buton „Open Bluetooth settings”.
- **Când e conectat (`live`):** VU-metru (32 bare, galben peste 88%), 3 statistici (Latency, Via, Quality), rând „Voice on PC” (avatar + nume; „Change” la Pro, „Unlock voices” gratuit → Effects), buton „Disconnect”.

## Ecranul Effects

- Titlu „Voice effects” + subtitlu după stare.
- **Gratuit:** card „CoMic Pro” (chenar galben 28%): titlu „Every voice, tuned your way”, 3 beneficii cu bifă verde (All 9 voices · Fine-tune pitch, tone, grit and more · One purchase — phone and PC), două carduri de plan (Monthly $1.99, Lifetime $9.99, cel ales cu chenar galben), buton galben „Continue”, „Cancel anytime in Google Play.” + „Restore purchase”.
- **Neconectat:** bandă „Connect to a PC to hear the voice live” + buton Connect.
- **Card Fine-tune** pentru vocea aleasă: Pitch, Tone (formant), Grit, Wobble, Metal + buton Reset.
  - Pro: cursoarele **rămân unde le lași** și se țin minte **per voce** (`tune[voiceId]`). Se suprascriu doar la „Reset”. La mișcare se trimite valoarea la PC imediat, fără să re-desenezi ecranul (altfel degetul pierde cursorul).
  - Gratuit: aceleași cursoare, `disabled`, opacitate 45%, etichetă `PRO`.
  - Intervale și legătura cu receptorul: vezi tabelul Advanced din `../design/README.md` (`--pitch`, `--formant`, `--drive`, `--wobble`, `--metal`).
- **Grilă 2 coloane** cu 10 carduri (imagine 4:3 + nume + descriere). Selectat: chenar accent + glow + bifă. Pirate și Grandpa au eticheta verde `FREE` cât timp e gratuit. Blocat: opacitate 55%, imagine 60% grayscale, lacăt galben, nu se poate alege.
- Tab-ul Effects **nu** mai are eticheta PRO (o parte din voci sunt gratuite).

| Card | `id` | Avatar |
|---|---|---|
| Normal | `normal` (fără efect) | iconiță microfon |
| Pirate | `pirat` | pirat.jpg |
| Grandpa | `mosneag` | mosneag.jpg |
| Robot | `robot` | robot.jpg |
| Radio | `radio` | radio.jpg |
| Megaphone | `megafon` | iconiță megafon |
| Dinosaur | `dinozaur` | dinozaur.jpg |
| Demon | `demon` | demon.jpg |
| Squirrel | `veverita` | veverita.jpg |
| Alien | `extraterestru` | extraterestru.jpg |

## Ecranul Settings

Grupuri (titlu mic uppercase, card `#101010`, rânduri 13 px padding, separator `#191919`):

| Grup | Setare | Tip | Implicit | Legătură tehnică |
|---|---|---|---|---|
| Audio | Microphone | segmented: Bottom / Top / Headset | Bottom | sursa `AudioRecord` |
| Audio | Quality | segmented: Data saver / Standard / High | Standard | bitrate Opus 24 / 48 / 96 kbps, 48 kHz |
| Audio | Mic gain | slider 0.5–2.0 | 1.0 | |
| Audio | Noise suppression | switch | on | `NoiseSuppressor` / RNNoise (Faza 4) |
| Audio | Auto gain | switch | off | `AutomaticGainControl` |
| Connection | Port | câmp numeric | 45678 | |
| Connection | Auto-reconnect | switch | on | la schimbare de rețea |
| Connection | Require pairing code | switch | on | împerechere 6 cifre + AES-GCM |
| Connection | Pause during calls | switch | on | Android dă microfonul apelului |
| Background | Stream with screen off | switch | on | Foreground Service tip `microphone`, WakeLock, WifiLock |
| Background | Controls in notification | switch | on | Mute / Stop din notificare |
| Background | Battery optimization | stare + „Check” | — | scutire de la optimizarea bateriei |
| General | Language | select | System default | en, ro, ru, uk, fr, es, pt, de, tr, pl |
| Account | CoMic Pro | stare + Upgrade / Manage | — | „Free · mic, all connections, 2 voices” / „Active · all voices + fine-tuning” |
| Account | Help, About | linkuri | — | |

## Tokens

Aceleași ca pe PC (`../design/README.md`), plus:
```
seg-on   #1E1E1E   opțiunea activă din segmented
body     #D4D4D4   text pe cardurile de avertisment / Pro
pro      #FBBF24   tot ce ține de Pro (badge, lacăt, buton de cumpărare)
raze     8 / 12 / 14 / 18 px; colțurile ecranului în prototip 36 px (doar ramă de prezentare)
```
Font Inter, cifrele / IP-urile în JetBrains Mono. Ținte de atingere ≥ 44 px pe butoanele principale.

## Note pentru implementare (Jetpack Compose)

- Tema: dark only, `background = #0A0A0A`, `surface = #101010`, `primary = #818CF8`. Pune culorile într-un singur `CoMicColors`.
- Stările de conexiune se modelează ca `sealed interface ConnState { Idle; Waiting; Live(muted) }`, iar ecranul se desenează din ea. Butonul mare și textele depind doar de stare + metodă.
- Ce e demo în prototip: „DESKTOP-VASIL”, „VASIL-LAPTOP”, IP-urile, codul de împerechere, latențele și VU-metrul animat aleator. În aplicație vin din `StreamService`, `DiscoveryClient` și nivelul RMS al `AudioRecord`.
- Textele UI sunt în engleză; pune-le în `strings.xml` ca să poată fi traduse.
- Designul vizual e și pe canvas-ul Claude „CoMic — Desktop App”, rândul „Phone app”.

## Actualizare 2026-11-18

- **Texte și traduceri:** toate textele sunt în `shared/i18n/<limbă>.json` (10 limbi). `node tools/i18n_build.mjs` le pune în `android/app/src/main/res/values*/strings.xml`. Ecranele Kotlin trebuie să folosească `stringResource(R.string.ph_ready_title)` etc. în loc de texte scrise direct în cod.
- **Limba se alege discret:** butonul mic cu glob și codul limbii (EN, RO...) în bara de sus, lângă Get Pro. Lista e și în Setări › General.
- **Fără liniuțe lungi (—) în texte:** sună a text generat. Folosește punct, virgulă sau „și”.
