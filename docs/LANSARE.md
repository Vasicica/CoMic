# Lansarea pe Google Play: ce trebuie facut si unde sunt capcanele

Verificat pe 2026-11-18. Regulile Google se schimba des: inainte de fiecare pas, deschide linkul.
Nu e sfat juridic sau fiscal; la bani si taxe intreaba un contabil din Moldova.

## 1. Contul de dezvoltator

- [ ] Cont personal Google Play Console: 25 $ o data. Verificare de identitate cu act cu poza.
- [ ] **Moldova e tara acceptata pentru vanzare** (cont de comerciant, plati in USD).
      https://support.google.com/googleplay/android-developer/answer/9306917
- [ ] **Test inchis obligatoriu: minim 12 testeri, 14 zile la rand**, inainte de publicare.
      Cine iese si revine nu se pune. Strange de acum 15-20 de oameni (rezerva).
      https://support.google.com/googleplay/android-developer/answer/14151465
- [ ] Banii de la Google sunt venit: intreaba un contabil cum il declari in Moldova
      (persoana fizica, intreprinzator, IT Park etc.).

## 2. Cerinte tehnice Android

- [ ] **targetSdk = 36** (Android 16). Obligatoriu pentru aplicatii noi din 31 august 2026.
      https://developer.android.com/google/play/requirements/target-sdk
- [ ] **Declaratia pentru serviciul in prim-plan de tip microfon** (App content in Play Console):
      descrierea functiei, ce se intampla daca sistemul il opreste, **un video** in care
      utilizatorul apasa Start si porneste microfonul, cazul de utilizare
      „Background Audio Access”. Toate se verifica de om.
      https://support.google.com/googleplay/android-developer/answer/13392821
- [ ] Din Android 14 serviciul de microfon **nu poate porni din fundal**: doar cand aplicatia
      e pe ecran (apasarea pe Start). Reconectarea automata trebuie sa tina cont de asta.
- [ ] Bluetooth (cand se face): permisiunea `BLUETOOTH_CONNECT` (Android 12+), cu explicatie.

## 3. Formularele din Play Console

- [ ] **Politica de confidentialitate la un link public** (ex. GitHub Pages sau un site mic).
      Textul e gata: `docs/legal/privacy-policy.en.md` si `.ro.md`. Completeaza [NUME], [EMAIL], [DATA].
- [ ] **Data safety.** Propunere, de verificat cu ghidul Google: Audio, „Voice or sound recordings”,
      prelucrate efemer, criptate in transfer, nepartajate, necesare functiei. Plata: o face Google.
      Fara reclame, fara analitica, fara cont.
- [ ] **Public tinta: 13+**, nu copii. Altfel intri sub politica „Families” (reguli mult mai stricte).
      Promovare catre jucatori, nu catre copii.
- [ ] Chestionarul de clasificare a continutului.
- [ ] Descrierea din magazin in toate limbile (titlu, descriere scurta, descriere lunga, capturi).
      Fara „cel mai bun”, „nr. 1”, fara nume de alte aplicatii (WO Mic) in titlu sau descriere.

## 4. Abonament si plata

- [ ] Doua produse: abonament lunar ($1.99) si produs unic „pe viata” ($9.99). Vezi `VOICE-CHANGER.md`.
- [ ] Ecranul de plata: pret exact si perioada, „se reinnoieste automat”, buton „Nu acum”.
- [ ] In Setari › Cont: link spre anularea abonamentului in Google Play.
- [ ] Verificarea cumparaturii pe server sau cel putin cu biblioteca Play Billing, nu doar un
      „true” salvat local (altfel se sparge in 5 minute).
- [ ] Testare cu „License testers” din Play Console, fara bani reali.
- [ ] Taxa Google la produsul unic, din 2026: in SUA/UE/UK mai mare decat 15%; in Moldova si
      „restul lumii” 15%. Pe 100 de vanzari de $1.99 raman realist ~$140-170.

## 5. Legi

- [ ] **Moldova: Legea 195/2024** privind protectia datelor, aplicabila din **23 august 2026**
      (aliniata la GDPR). Politica de confidentialitate o acopera.
- [ ] **UE: GDPR** se aplica pentru utilizatorii din UE. Drepturile lor sunt in politica.
- [ ] **Consumatori UE:** dreptul de retragere si rambursarile le gestioneaza Google Play.
      Abonamentul trebuie sa se poata anula usor (Google Play o face).
- [ ] **TVA:** la vanzarile prin Google Play, Google colecteaza si plateste TVA-ul in tarile
      unde e obligat; nu factura tu clientilor.
- [ ] **Imitarea altor persoane** e interzisa in `docs/legal/terms.*.md`, sectiunea 2.
- [ ] Numele „CoMic”: cauta-l pe Google Play si in registrele de marci (AGEPI, EUIPO)
      inainte sa platesti grafica si reclama.

## 6. Windows

- [ ] **VB-Cable:** se poate livra impreuna cu CoMic, „asa cum este”, cu mentiunea ca vine
      de la vb-cable.com si e donationware. Daca utilizatorii nu pot vedea ca e VB-Cable,
      trebuie licente platite. VB-Cable A+B si C+D **nu** au voie sa fie redistribuite.
      https://vb-audio.com/Services/licensing.htm
- [ ] **adb.exe** (pentru USB debugging): binarele Google vin cu licenta SDK Android; citeste-o
      inainte sa-l pui in installer. Alternative: utilizatorul instaleaza Platform Tools singur,
      sau compilezi adb din sursa AOSP (Apache 2.0).
- [ ] **Firewall-ul Windows** intreaba la prima pornire daca CoMic poate primi conexiuni pe
      UDP 45678. Daca omul apasa „Cancel”, nu merge nimic si nu stie de ce. Programul trebuie sa
      detecteze ca nu vine nimic si sa explice ce sa faca.
- [ ] **SmartScreen:** un .exe nesemnat arata „Windows protected your PC”. Multi se sperie.
      Semnarea codului costa; pana atunci, explica pe pagina de descarcare cum treci de mesaj.

## 7. Ca sa nu apara bug-uri mai tarziu

- [ ] **Versiunea protocolului** in prima discutie telefon-PC. Un telefon actualizat si un PC
      vechi trebuie sa-si spuna clar „actualizeaza CoMic pe PC”, nu sa taca.
- [ ] **Microfonul ocupat** (apel, alta aplicatie): pauza si mesaj, nu tacere.
- [ ] **Telefoane Xiaomi, Huawei, Samsung** opresc aplicatiile in fundal mai agresiv. Testeaza pe
      cat mai multe; pune in Ajutor pasii pentru fiecare.
- [ ] **PC-ul intra in sleep** in timp ce merge: la trezire, programul trebuie sa reporneasca ascultarea.
- [ ] **VB-Cable lipsa:** programul il detecteaza si arata un buton spre instalare, nu o eroare tehnica.
- [ ] **Doua PC-uri in aceeasi retea:** lista arata ambele, cu numele lor.
- [ ] **Raport de erori:** daca adaugi vreodata trimiterea automata a erorilor, se schimba
      politica de confidentialitate si formularul Data safety. Fa-l cu acord explicit.

## 8. Idei de functii (in ordinea in care as face)

1. **Cod QR pentru conectare:** PC-ul arata un QR, telefonul il scaneaza. Cel mai rapid mod de a
   lega telefonul, fara IP scris de mana.
2. **Tasta rapida pe PC** pentru mut si pentru schimbarea vocii in timpul jocului.
3. **Vocile mele (Pro):** salvezi reglajele cu un nume si le ai pe telefon si pe PC.
4. **Soundboard (Pro):** butoane cu sunete scurte trimise in microfon. Foarte cerut de jucatori.
5. **Reducerea zgomotului** reala (RNNoise), deja in PLAN la Faza 4.
6. **Mod invers:** boxele telefonului ca difuzor pentru PC (in PLAN).
7. **Voci de femeie / copil / anime** dupa motorul spectral (formanti separati de inaltime).
8. **iOS** mai tarziu: costa 99 $/an si Apple limiteaza microfonul in fundal (vezi PLAN).
