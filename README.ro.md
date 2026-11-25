# CoMic

**Telefonul tău Android ca microfon wireless pentru Windows.**
Alternativă gratuită și open-source la WO Mic: fără limită de timp, fără reclame, cu schimbător de voce inclus.

[English](README.md)

> **Stadiu: început (v0.1).** Merge zilnic pe calculatoarele autorului, dar pot apărea probleme.
> Rapoartele de bug-uri sunt foarte binevenite.

## De ce

Majoritatea aplicațiilor „telefon ca microfon” te opresc după câteva minute, au reclame
sau se opresc când se stinge ecranul. CoMic nu face nimic din toate astea. Microfonul e
gratuit și nelimitat, pentru totdeauna.

## Ce face

- **Microfonul telefonului în PC**, prin Wi-Fi sau partajare USB. Merge și cu ecranul stins.
- **Găsește singur calculatorul** în rețea, sau scrii adresa de mână.
- **Îl poate folosi orice program** (Discord, OBS, Zoom, jocuri), prin [VB-Cable](https://vb-audio.com/Cable/).
  Dacă VB-Cable lipsește, CoMic îți spune și îți dă linkul.
- **Schimbător de voce**, procesat pe PC în Rust: Normal, Pirat, Moșneag și altele. 14 voci azi.
- **Diagnostic de Firewall:** când nu ajunge sunetul, CoMic întreabă Windows-ul de ce și îți spune.
- **10 limbi:** engleză, română, rusă, ucraineană, poloneză, turcă, germană, franceză, spaniolă, portugheză.
- Aplicații native și mici: Kotlin + Jetpack Compose pe Android, Rust + Tauri pe Windows.

## Instalare

1. Descarcă ultimul **APK** (telefon) și **CoMic.exe** (PC) din
   [Releases](https://github.com/Vasicica/CoMic/releases).
2. Instalează [VB-Cable](https://vb-audio.com/Cable/) pe PC (gratuit).
3. Pornește CoMic pe PC, apoi pe telefon. Alegi PC-ul din listă sau scrii adresa.
4. În Discord / OBS / joc, alege **CABLE Output** ca microfon.

Portul implicit: **45678** (UDP pentru sunet, TCP pentru control).

## Compilare din sursă

Vezi secțiunea [Build from source](README.md#build-from-source) din README-ul în engleză
și [android/README.md](android/README.md).

## Ce urmează

- [ ] Compresie Opus (azi sunetul merge ca PCM brut)
- [ ] Cod de împerechere și criptare. **Până atunci, folosește CoMic doar în rețele în care ai încredere.**
- [ ] USB prin adb, Bluetooth
- [ ] Eliminarea zgomotului
- [ ] Driver propriu de microfon virtual (fără VB-Cable)

## Contribuții

Cel mai simplu mod de a ajuta: o traducere. Un singur fișier JSON, fără unelte de compilare.
Vezi [CONTRIBUTING.md](CONTRIBUTING.md).

## Licență

[MIT](LICENSE) © 2026 Ciolac Vasile. Nu are legătură cu WO Mic sau VB-Audio.
