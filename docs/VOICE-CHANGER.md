# Voice changer — functia care aduce bani

## Ce s-a decis (actualizat 2026-11-18)

| Functie | Gratuit | Pro |
|---|---|---|
| Microfon, fara limita de timp | da | da |
| Toate metodele de conectare | da | da |
| Vocile Normal, Pirat, Mosneag, folosite de-adevaratelea (in Discord, CS2...) | da | da |
| Fine tuning: cat de tare e efectul, taierea basului si a inaltelor | da | da |
| Restul vocilor (azi 14, mai multe cu actualizarile) | nu | da |
| Advanced: inaltime, distorsiune, tremur, metal, liber, tinute minte per voce | nu | da |

- Vocea se schimba **de pe ambele dispozitive**, telefon si PC.
- Pro se cumpara **numai pe telefon**, prin Google Play Billing. Telefonul anunta PC-ul pe
  canalul de control (criptat). PC-ul nu are sistem de plata propriu.

  **Starea calatoreste deja** (2026-11-19). Telefonul trimite `pro` in salutul de pe
  canalul de control, si un mesaj `pro` cand se schimba in timpul sesiunii, ca o
  cumparatura facuta in timp ce transmiti sa deschida vocile pe loc. PC-ul tine minte
  ce i-a spus ultimul telefon, in `prefs.json`: altfel vocile s-ar incuia ori de cate
  ori telefonul nu e legat, inclusiv la ascultarea de proba cu microfonul propriu.
  Un telefon care nu trimite campul nu primeste Pro pe gratis: lipsa inseamna nu.

  **Cumparatura inca nu exista.** Google Play Billing cere cont de Play Console, doua
  produse create acolo si o compilare semnata urcata pe o pista de testare. Pana atunci,
  in locul ei sta un comutator in Settings, aratat **numai in compilarea de depanare**
  (`BuildConfig.DEBUG`), ca drumul sa poata fi incercat acum. Cand vine Billing, el scrie
  in aceeasi cheie si comutatorul dispare singur.
- Diferenta fata de WO Mic ramane argumentul principal: **microfonul e gratuit si nelimitat**.

## Pretul (propunere, de confirmat)

Doua produse in Google Play, ca omul sa aleaga:

| Produs | Tip in Play | Pret |
|---|---|---|
| CoMic Pro lunar | abonament, se reinnoieste lunar | $1.99 / luna |
| CoMic Pro pe viata | produs unic (one-time) | $9.99 o data |

De ce ambele: abonamentul aduce venit constant, dar multi oameni refuza din principiu
abonamentele la unelte (exact motivul pentru care s-a plecat de la WO Mic). Varianta pe
viata ii prinde pe ei. Google ia o parte din fiecare plata (15% in Moldova si "restul lumii";
in SUA, UE si UK, din 2026, mai mult la produsele unice). Preturile locale le propune Google
automat, pe tari.

Reguli Google pentru abonamente, de respectat in ecranul de plata:
- pretul exact si perioada ("$1.99 pe luna, se reinnoieste automat") vizibile inainte de plata;
- buton clar de inchidere ("Nu acum"): aplicatia merge si fara Pro;
- link spre anulare (Centrul de abonamente Google Play) in Setari › Cont;
- daca vreodata se da perioada de proba: cat dureaza, cat costa dupa, cum anulezi.

## Vocile si cum se fac

Procesarea sta pe calculator, in Rust (`windows/src/effects`). O voce noua = un preset in
`voices.rs`, nu cod nou. Motorul are acum: inaltime, filtre (trece-jos, trece-sus, trece-banda),
modulatie inelara (metal), tremolo, soapta (zgomot care urmareste vocea), distorsiune, ecou.

Voci adaugate pe 2026-11-18: Pestera, Sub apa, Fantoma, Soapta, Monstru, Urias, Telefon.
Pozele lor au venit pe 2026-11-19, deci toate vocile-personaj au acum poza. Raman cu iconita
doar Normal si Megafon, si asa trebuie sa fie: niciuna nu e un personaj.

### De ce unele voci nu suna "a personaj"

Motorul de inaltime muta si formantii odata cu tonul (vezi `pitch.rs`). De aceea o voce de
barbat ridicata suna a veverita, nu a femeie sau a fata din anime. Pentru voci de femeie,
copil sau anime trebuie separat **timbrul** (formantii) de inaltime: motor spectral, o
lucrare separata. Pana atunci, raman vocile pe care efectele simple le fac credibil.

Pentru poze: intai asculti vocea gata, apoi faci poza care i se potriveste. Acelasi stil
pentru toate (bust 3D, fundal negru, personaj centrat), ca grila sa arate unitar.

Formatul cerut: **640x480 JPEG**, nu patrat. Cartela din fereastra are `aspect-ratio: 4/3`
(`.fx-art` in `windows/app/ui/index.html`), iar pe telefon pozele stau in
`android/app/src/main/res/drawable-xhdpi/voice_<id>.jpg` — acelasi fisier, acelasi format.
O poza primita in alt raport se taie din mijloc, dar daca personajul e subtire si inconjurat
de negru, taietura din mijloc il lasa prea mic la marimea unei cartele: atunci se taie mai
strans in jurul lui. Asa s-a facut cu Soapta.

## Folosire abuziva

Interdictia de a imita persoane reale, de a insela si de a hartui e scrisa in
`docs/legal/terms.en.md` si `terms.ro.md` (sectiunea 2). Linkul spre ele sta in Ajutor
(PC) si in Setari › Cont (telefon). Se promoveaza catre "jucatori", nu catre copii.
