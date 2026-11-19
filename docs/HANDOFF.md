# Predare: ce s-a facut in sesiunea din aplicatia Claude (17-18 noiembrie 2026)

Sesiunea a lucrat pe acest folder de la distanta (fara terminal pe calculatorul asta).
Fisierele de mai jos sunt deja scrise in proiect. **Nimic nu a fost compilat sau rulat pe
Windows.** Codul Rust de efecte a fost testat pe Linux (51/51 teste). Aplicatia Tauri a fost
compilata pe Linux, nu pe Windows. Android nu a fost atins in cod Kotlin (fisierele Kotlin erau
prea adanc ca sa poata fi citite de la distanta).

## 1. Decizii de produs (vezi si docs/VOICE-CHANGER.md)

- **Gratuit:** microfon nelimitat, toate conexiunile, vocile Normal, Pirat (`pirat`), Mosneag (`mosneag`),
  panoul Fine tuning (mix, taieri).
- **Pro:** restul vocilor + panoul Advanced (pitch, drive, wobble, metal), cu valori tinute minte per voce.
- **Pret propus:** abonament $1.99/luna SAU $9.99 o data (pe viata). Doua produse in Google Play.
- Pro se cumpara **numai pe telefon** (Google Play Billing); telefonul anunta PC-ul pe canalul de control.
  PC-ul nu are plata proprie.
- Vocea se schimba de pe **ambele** dispozitive.
- **10 limbi:** en, ro, ru, uk, fr, es, pt, de, tr, pl. Alegerea limbii: buton mic cu glob in colt.
- Fara liniute lungi (em dash) in textele pentru utilizator: suna a text generat.
- Wi-Fi Direct ramane exclus (hotspot-ul de pe telefon acopera cazul fara router).

## 2. Ce s-a schimbat in cod

### Rust, `windows/src`
- `effects/voices.rs`: campuri noi in `VoicePreset`: `low_pass`, `high_pass`, `whisper`, `echo`.
  7 voci noi: `pestera`, `subapa`, `fantoma`, `soapta`, `monstru`, `urias`, `telefon` (toate Pro).
  Comentariul la `free`: vocile gratuite se folosesc fara limita (nu mai sunt doar „proba”).
  Volumele (`gain`) noilor voci au fost egalizate dupa masurare (RMS apropiat de Normal).
- `effects/mod.rs`: structuri `Echo` si `Whisper`, filtrele vocii, ordinea lantului actualizata.
  `VoiceSettings` are campuri noi `wobble: Option<f32>` (0..1) si `metal_hz: Option<f32>` (0 = oprit).
  Protectie: un esantion NaN/infinit devine 0. 5 teste noi (ecou, soapta, wobble, metal=0, setari extreme).
- `main.rs`: optiuni CLI `--wobble <0-100>` si `--metal <0-200>`; eticheta din `--list-voices`
  „gratuita” / „Pro”; scoase liniutele lungi din textul de ajutor.

### Fereastra de PC, `windows/app`
- `ui/index.html`:
  - **Bug reparat:** citirea `status` (de ~20 ori/s) chema `render()`, care chema `syncTuning()` si
    punea Pitch/Drive inapoi la valorile vocii. Acum `syncTuning` ruleaza doar cand se schimba vocea
    sau planul; valorile Advanced stau in `state.tune[voiceId]`.
  - Cursoare noi Wobble si Metal, trimise in `settings()` ca `wobble` si `metalHz`.
  - Traduceri: `t(key, vars)`, atribute `data-i18n` / `data-i18n-aria`, limba in `localStorage`
    (`comic.lang`), prima data din `navigator.language`. Numele/descrierile vocilor vin din traduceri.
  - Meniu de limba in bara de titlu (`#lang-btn`, `#lang-menu`).
  - Fereastra umple tot (100vw/100vh) si se adapteaza: sub 880 px cardurile de conexiune devin
    randuri, sub 620 px tab-urile raman doar cu iconite, sub 460 px dispar descrierile.
    Verificat in browser la 1280, 1040, 820, 560, 380 px: fara derulare orizontala.
  - Voci blocate cand nu e Pro (gri, nu se pot alege), panoul Advanced blocat cu mesaj.
  - **`state.pro = true` deocamdata**, ca sa se poata testa toate vocile. TODO: vine de la telefon.
  - Textele cardurilor nu mai inventeaza un telefon („Pixel 8”): spun ce face programul.
  - Iconite noi in sprite: `i-cave`, `i-water`, `i-ghost`, `i-whisper`, `i-monster`, `i-giant`, `i-phone`.
- `ui/i18n.js`: GENERAT (vezi mai jos). Se include cu `<script src="i18n.js">` inainte de scriptul principal.
- `tauri.conf.json`: `minWidth` 360, `minHeight` 480 (era 900×620).
- `src/main.rs`, `src/prefs.rs`: **neschimbate**.

### Traduceri
- `shared/i18n/<limba>.json`: sursa unica, ~250 de chei, aceleasi in toate limbile.
- `tools/i18n_build.mjs`: verifica cheile si `{placeholder}`-ele, apoi scrie
  `windows/app/ui/i18n.js` si `android/app/src/main/res/values*/strings.xml`.
  Rulare: `node tools/i18n_build.mjs` din radacina, dupa orice schimbare in .json.

### Android
- `res/values/strings.xml` e acum **engleza** (inainte era romana), plus `values-ro`, `values-ru` etc.
  Numele vechi `notification_title`, `notification_text`, `stop`, `channel_name` sunt pastrate.
  Restul cheilor: punctul devine `_` (ex. `ph.ready.title` -> `ph_ready_title`, `{pc}` -> `%1$s`).
- **Nefacut:** ecranele Kotlin inca au texte scrise direct in cod. Trebuie mutate pe `stringResource(...)`.

## 3. Documente noi sau schimbate
- `docs/legal/privacy-policy.en.md` / `.ro.md`, `docs/legal/terms.en.md` / `.ro.md`
  (sectiunea 2 din termeni interzice imitarea altor persoane). De completat: [NUME], [ADRESA], [EMAIL], [DATA].
- `docs/LANSARE.md`: lista pentru Google Play, legi, Windows, idei de functii, cu linkuri.
- `docs/TESTARE.md`: plan de testare; sectiunea 9 = ce e nou de verificat.
- `docs/VOICE-CHANGER.md`: rescris cu deciziile de mai sus.
- `design/` si `phone-design/`: prototipuri HTML + README actualizate (Pro, preturi, limba, fara liniute lungi).
  Designul vizual complet (PC + 5 ecrane de telefon, interactiv) e si pe canvas-ul Claude „CoMic — Desktop App”.

## 4. Pasii urmatori, in ordine

1. Din `windows/`: `cargo test -p comic-receiver` (trebuie 51 trecute), apoi construieste si porneste
   aplicatia Tauri pe Windows. Verifica punctele din `docs/TESTARE.md` sectiunea 9.
2. **Micsoreaza pozele vocilor** din `windows/app/ui/assets/avatars`: acum ~4,5 MB la 1472×704
   (robot.png 1,7 MB), afisate la ~180 px. La 480×480 (taiate patrat, centrat) ar fi ~30 KB fiecare.
   Programul ar scadea de la ~10 MB la ~6-7 MB, installer-ul la ~2-3 MB.
3. Android: muta textele Kotlin pe `stringResource`, adauga butonul de limba in bara de sus
   (vezi `phone-design/`), ecranele Connect / Effects / Settings dupa `phone-design/README.md`.
4. Pro: Google Play Billing cu doua produse (lunar + pe viata), mesaj pe canalul de control
   telefon -> PC cu starea Pro; pe PC `state.pro` vine din acel mesaj.
5. Mesaj de ajutor cand Firewall-ul Windows blocheaza UDP 45678 (vezi `docs/LANSARE.md` sectiunea 6).
6. Poze pentru vocile noi (acum au iconite). Acelasi stil ca restul. **Facut pe 2026-11-19**,
   vezi sectiunea 7.

## 5. Limitari cunoscute
- Cursorul „Tone (formant)” din designuri **nu exista in aplicatie**: `pitch.rs` muta formantii
  odata cu tonul. Separarea lor cere un motor spectral (lucrare separata). Tot de aici vine de ce
  vocile de femeie/anime nu suna credibil.
- Lista `VOICES` din `ui/index.html` e folosita doar cand pagina e deschisa in browser;
  in aplicatie catalogul vine din Rust (`voices`). Daca schimbi un preset, schimba in ambele.

## 6. Ce s-a facut pe Windows (18 noiembrie 2026, sesiune cu terminal)

Pasii 1, 2 si 3 din lista de mai sus sunt gata. Pasii 4, 5 si 6 s-au facut pe
19 noiembrie 2026, vezi sectiunea 7.

- **Pasul 1:** `cargo test -p comic-receiver` trece 51 din 51. Aplicatia Tauri se
  compileaza si porneste pe Windows, fara nimic in `stderr`. Traducerile merg,
  corectura cursoarelor e la locul ei, fereastra se adapteaza pana la 380 px.
  Singurul lucru gasit stricat: trei fisiere Rust neformatate, rezolvate cu
  `cargo fmt`.
- **Pasul 2:** pozele vocilor de PC taiate la 640x480 (4:3, aceeasi incadrare pe
  care o face CSS-ul), de la 4,3 MB la 272 KB. `robot.png` a devenit `robot.jpg`.
- **Pasul 3:** textele Kotlin sunt pe `stringResource`; butonul cu glob e in bara
  de sus si schimba limba pe loc, fara repornire (`device/AppLanguage.kt`, un
  Context cu alta configuratie, fara `androidx.appcompat`); ecranele Effects si
  Settings exista. `StreamScreen.kt`, vechiul ecran unic, a fost sters.

Ce lucreaza de-adevaratelea pe telefon azi: Connect (Wi-Fi si USB tethering),
limba, portul din Settings si starea bateriei. Restul randurilor se vad, dar
poarta eticheta care spune ca nu sunt scrise. Ecranul Effects e o lista de voci,
nu un selector, fiindca fara canalul de control telefonul nu poate cere PC-ului
sa schimbe vocea.

Chei noi de traducere: 32, in `ph.*` si `on`. Scoasa: `ph.soonSettings`.

## 7. Ce s-a facut pe 19 noiembrie 2026

- **Pasul 4, jumatatea care se putea face:** starea Pro calatoreste de la telefon la
  calculator, pe canalul de control (`pro` in salut, plus un mesaj `pro` cand se schimba
  in timpul transmisiei). Calculatorul o tine minte in `prefs.json`, altfel vocile s-ar
  incuia ori de cate ori telefonul nu e legat. Google Play Billing **nu** s-a facut: cere
  cont de Play Console, doua produse create acolo si o compilare semnata urcata pe o pista
  de testare, deci n-ar fi fost nici ce chema, nici ce proba. In locul lui sta un comutator
  aratat numai in compilarea de depanare.
- **Pasul 5:** cand nu soseste nimic pe retea, programul intreaba Windows-ul de ce si spune
  raspunsul (`windows/src/firewall.rs`). Capcana adevarata nu e ca lipseste regula, ci ca
  reteaua noua facuta de tethering e clasata Publica in timp ce regulile sunt Private, si
  Windows nu mai intreaba o data fiindca o regula exista deja.
- **Pasul 6:** cele sapte voci noi au primit poze. Sunt in
  `windows/app/ui/assets/avatars/<id>.jpg` si in
  `android/app/src/main/res/drawable-xhdpi/voice_<id>.jpg`, acelasi fisier in ambele locuri,
  640x480. Iconitele care tineau locul (`i-cave`, `i-water`, `i-ghost`, `i-whisper`,
  `i-monster`, `i-giant`, `i-phone` si perechile lor `ic_*.xml` de pe Android) au fost sterse:
  dupa ce o voce are poza, iconita ei nu se mai vede niciodata, iar codul care o tine ar
  spune ca se vede. Raman cu iconita doar Normal si Megafon, pentru ca nu sunt personaje.
