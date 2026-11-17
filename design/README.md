# CoMic — design pentru aplicația de Windows

Interfața nouă a receptorului de Windows (înlocuiește stilul vechi tip WO Mic).
Acest folder este **sursa de adevăr pentru UI**: când implementezi interfața în `windows/`, copiază de aici culorile, dimensiunile, textele și stările.

## Ce e aici

| Fișier | Ce este |
|---|---|
| `comic-ui.html` | Prototip complet, interactiv, de sine stătător (HTML + CSS + JS, fără build). Deschide-l în browser. |
| `assets/avatars/*.jpg` | Avatarele vocilor, 480×480, numite după `id`-ul din `windows/src/effects/voices.rs`. |
| `README.md` | Specificația de mai jos. |

## Fereastra

- Dimensiune: **1040 × 720**, colțuri 14 px, chenar 1 px `#1F1F1F`, fără bară de titlu Windows (frameless, bară proprie).
- De sus în jos:
  1. **Title bar** 44 px — logo (pătrat 24 px, culoarea accent, iconiță microfon) + „CoMic” + versiune; dreapta: Minimize, Close (Close devine roșu la hover). Zona e „drag” pentru mutarea ferestrei.
  2. **Menu bar** 52 px — tab-urile `Connection`, `Options`, `Effects`, `Help` (iconiță 16 px + text 13 px/500). Activ: fundal `#171717`, text `#F5F5F5`. Hover: fundal `#151515`. În dreapta: chip „Voice: <vocea activă>”.
  3. **Zona dinamică** — padding 28/32 px, își schimbă conținutul după tab (fade + 4 px slide, 0.25 s).
  4. **Status bar** 36 px, fundal `#080808` — stânga: punct de stare + „Connected/Disconnected” + dispozitiv + latență; dreapta: iconiță protocol + „Wi-Fi / USB / Bluetooth / Manual IP” + detaliu (ex. `UDP 45678`).

## Tokens

```
bg            #0A0A0A   fundalul ferestrei (mat)
bar           #080808   status bar
surface       #101010   carduri / panouri
surface-hover #141414
surface-2     #171717   tab activ, pill-uri
input         #0A0A0A
border        #1C1C1C   chenar carduri
border-soft   #161616   separatoare între bare
border-strong #262626   inputuri, butoane ghost
text          #EDEDED   text-strong #F5F5F5
muted         #8F8F8F   text secundar
faint         #7A7A7A   etichete mici (trece 4.5:1 pe #0A0A0A)
dim           #A3A3A3   iconițe inactive
accent        #818CF8   (indigo) butoane principale, selecție, toggle-uri
accent-ink    #0A0A0A   text pe accent
ok            #4ADE80   conectat
warn          #FBBF24   tag PRO, vârfuri în VU-metru
danger        #E5484D   hover pe Close
```

Raze: 6 / 8 / 12 / 14 px. Font: **Inter** (400/500/600), cifre și IP-uri în **JetBrains Mono**. Titluri de view 20 px/600, letter-spacing −0.02em. Tranziții 0.2 s ease; respectă `prefers-reduced-motion`.

## View-uri

### Connection (implicit)
- Titlu „Connect your phone” + subtitlu.
- **Status strip**: iconiță microfon, „Streaming from <telefon>” / „Waiting for a phone”, VU-metru (40 bare, verde, vârfurile >88% galbene), Latency + Sample rate.
- **4 carduri** pe un rând: Wi-Fi, USB, Bluetooth, IP address (input IP + port).
  - Stări: *neconectat* — pill gri („Available”, „No device”, „Not paired”, „Manual”), buton accent „Quick Connect”; *conectat* — chenar verde 45%, inel verde 3 px, pill verde „Connected”, buton ghost „Disconnect”.
  - O singură conexiune activă odată; status bar-ul se actualizează.

### Effects (voice changer)
- Grilă **5 × 2**, carduri cu imagine 4:3 + nume + descriere scurtă.
- Selectat: chenar accent + glow (`0 0 0 1px accent, 0 10px 36px -10px accent/70%`) + bifă rotundă în colțul dreapta-sus. Hover: ridicare 3 px, zoom 5% pe imagine.
- Tag în stânga-sus: `FREE` (gri) sau `PRO` cu lacăt (galben). Corespunde câmpului `free` din Rust.
- Buton „Test with my voice” = modul de probă `--input` din receptor.

| Card | `id` în voices.rs | Avatar | Tag |
|---|---|---|---|
| Normal | `normal` | iconiță microfon | FREE |
| Pirate | `pirat` | pirat.jpg | FREE |
| Grandpa | `mosneag` | mosneag.jpg | FREE |
| Robot | `robot` | robot.jpg | PRO |
| Radio | `radio` | radio.jpg | PRO |
| Megaphone | `megafon` | iconiță megafon (nu există imagine încă) | PRO |
| Dinosaur | `dinozaur` | dinozaur.jpg | PRO |
| Demon | `demon` | demon.jpg | PRO |
| Squirrel | `veverita` | veverita.jpg | PRO |
| Alien | `extraterestru` | extraterestru.jpg | PRO |

Când adaugi o voce nouă în `PRESETS`, adaugă un card cu același `id` și un avatar `assets/avatars/<id>.jpg` (pătrat, fundal negru, personaj centrat).

#### Reglaje sub grila de voci

Două panouri pe un rând (`.two.tuning`), în același stil ca panourile din Options.

**Fine tuning** — reglajele utilizatorului, *nu* se schimbă când schimbi vocea:

| Control | Interval | Afișare | Opțiune în receptor |
|---|---|---|---|
| Effect amount | 0–100, pas 5 | `100%` | `--mix` |
| Low cut | 0–400 Hz, pas 10 | `Off` la 0, altfel `150 Hz` | `--low-cut` |
| High cut | 2000–20000 Hz, pas 500 | `Off` la maxim, altfel `7.0 kHz` | `--high-cut` |

Sub *Effect amount* stă un avertisment galben (`.hint`), vizibil **doar** când vocea aleasă deplasează înălțimea (`semitones != 0`) **și** mix-ul e sub 100: „This voice shifts pitch — below 100% you hear two voices at once. Use Pitch instead." Motivul e real: amestecul unei voci transpuse cu originalul sună a două persoane, nu a efect mai slab.

**Advanced** — pornesc de la valorile vocii și revin la ele la fiecare schimbare de voce:

| Control | Interval | Afișare | Opțiune în receptor |
|---|---|---|---|
| Pitch | −12…+12 semitonuri, pas 1 | `+7 st` | `--pitch` |
| Drive | 0–10, pas 0.5 | `2.5` | `--drive` |

Butonul **Reset to voice** din antetul panoului Advanced readuce mix-ul la 100% și scoate ambele tăieri; înălțimea și distorsiunea revin oricum la valorile vocii.

De aceea fiecare intrare din `VOICES` are acum `semitones` și `drive` — aceleași valori ca în `PRESETS` din Rust. Când schimbi un preset în Rust, schimbă-l și aici.

### Options
Două panouri:
- **Audio**: Output device (select; implicit „CABLE Input”) → `--device`; Gain 0.5–2.0 → `--gain`; Buffer 1–10 cadre de 10 ms → `--buffer`.
- **General**: toggle-uri Launch at startup, Auto-reconnect, Minimize to tray, Beta updates; UDP port → `--port` (implicit 45678).

### Help
Trei pași (telefon → VB-Cable → „CABLE Output” în aplicație) + listă de linkuri (Troubleshooting, Report a problem, About).

## Note pentru implementare

- Textele UI sunt în engleză, ca în meniu. Dacă vrei și română, păstrează-le într-un singur fișier de traduceri.
- Butoane reale (`<button>`), etichete pentru fiecare input, focus vizibil 2 px accent, ținte de click ≥ 34 px.
- Prototipul folosește date demo („Pixel 8”, latențe fixe, VU-metru animat aleator). În aplicație ele vin din receptor: nivelul RMS, statisticile din `JitterBuffer`, dispozitivul descoperit prin `net::spawn_discovery`.
- Dacă UI-ul se face cu un webview (ex. Tauri), `comic-ui.html` se poate folosi aproape direct; dacă se face nativ (ex. egui/iced), urmează tokens-urile și dimensiunile de mai sus.
- Designul vizual mai e și pe canvas-ul Claude „CoMic — Desktop App”.
