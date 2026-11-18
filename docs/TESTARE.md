# Plan de testare inainte de Google Play

Ce ai: telefonul Android, calculatorul de birou (Windows) si notebook-ul (Windows).
Asta ajunge pentru toate metodele. Fiecare test se bifeaza pe ambele calculatoare,
pentru ca driverele de Wi-Fi, USB si Bluetooth difera de la un PC la altul.

Pentru fiecare test noteaza: a mers / n-a mers, latenta afisata, si ce s-a auzit.

## 0. Pregatire (o data pe fiecare calculator)

- [ ] VB-Cable instalat, Windows repornit
- [ ] `comic-receiver.exe` porneste fara erori
- [ ] `--list-devices` arata „CABLE Input”
- [ ] In Windows › Sunet › Inregistrare apare „CABLE Output”
- [ ] Unealta de verificare: Discord (Setari › Voce › „Let's Check”) sau Sound Recorder din Windows

## 1. Wi-Fi (metoda principala)

- [ ] Telefon si PC pe acelasi router, **5 GHz** — conectare scriind IP-ul de mana
- [ ] Acelasi lucru pe **2.4 GHz** — noteaza diferenta de latenta
- [ ] Descoperirea automata: PC-ul apare in lista „PCs on this network” pe telefon
- [ ] Router care blocheaza descoperirea (ex. retea „Guest”): IP-ul scris de mana tot merge
- [ ] **Hotspot de pe telefon**, notebook-ul conectat la el — merge fara router deloc
- [ ] Codul de imperechere: un cod gresit e refuzat

## 2. USB

- [ ] **USB debugging**: Developer options › USB debugging pornit, cablu, conectare
- [ ] **USB tethering**: fara developer options, doar partajare prin USB
- [ ] Cablul scos in timpul transmisiei: aplicatia arata deconectat, nu se blocheaza
- [ ] Cablul pus inapoi: se reconecteaza (daca Auto-reconnect e pornit)
- [ ] Cablu de doar incarcare (fara date): mesaj clar, nu tacere

## 3. Bluetooth (pe notebook)

- [ ] Imperechere telefon ↔ notebook din setarile Windows
- [ ] Conectare din CoMic, se aude vocea
- [ ] Noteaza latenta (asteptat 100–200 ms) si calitatea
- [ ] Notebook-ul se departeaza 5–10 m: ce se intampla cand semnalul slabeste

## 4. Wi-Fi Direct

Exclus in `CONEXIUNI.md`: suport inconsecvent intre telefoane si nu aduce nimic
peste hotspot (testul 1) sau USB. Hotspot-ul de pe telefon acopera exact situatia
„nu am router”. Daca totusi vrei sa-l incerci, e o decizie noua de luat inainte de cod.

## 5. Situatii reale (pe metoda care merge cel mai bine)

- [ ] **1 ora** de transmisie continua, fara intreruperi
- [ ] **Ecranul stins** si telefonul in buzunar 10 minute
- [ ] **Apel telefonic** in timpul transmisiei: pauza, apoi reluare automata
- [ ] **Schimbare de retea** (iesi din raza Wi-Fi si revii)
- [ ] Bateria: cat consuma o ora (procente)
- [ ] Discord, CS2, Zoom, Chrome (ex. Google Meet): toate vad „CABLE Output”

## 6. Voci

- [ ] Pirate si Grandpa merg in versiunea gratuita, in Discord
- [ ] Vocile Pro sunt blocate fara Pro, pe telefon **si** pe PC
- [ ] Schimbi vocea pe telefon → se schimba pe PC in < 1 secunda
- [ ] Schimbi vocea pe PC → telefonul arata vocea noua
- [ ] Cursoarele Pro **raman unde le-ai lasat**: dupa schimbarea vocii si intoarcere,
      dupa schimbarea tab-ului, dupa reconectare
- [ ] „Reset” readuce doar vocea curenta la preset
- [ ] Cumparatura de test (Google Play are „licence testers”): Pro se deblocheaza
      pe telefon si apoi pe PC, fara restart

## 7. Limbi

- [ ] Fiecare limba: nimic taiat, nimic iesit din buton (germana si rusa au cuvinte lungi)
- [ ] Limba „System default” urmeaza limba telefonului / Windows-ului

## 8. Google Play — testul inchis obligatoriu

Conturile personale noi de dezvoltator trebuie sa ruleze un **test inchis cu cel putin
12 testeri, timp de 14 zile la rand**, inainte sa poata publica. Testerii care ies si
revin nu se pun. Planifica asta: prieteni, familie, colegi — 12 conturi Google care
instaleaza aplicatia din linkul de test si o tin instalata doua saptamani.

## 9. Ce s-a adaugat pe 2026-11-18 (de verificat acasa)

Inainte: din `windows/`, `cargo test -p comic-receiver` (trebuie 51 de teste trecute),
apoi `node tools/i18n_build.mjs` din radacina proiectului, apoi construiesti aplicatia.

- [ ] Vocile noi, fiecare cu vocea ta (butonul „Test with my voice”):
      Pestera (ecou), Sub apa, Fantoma, Soapta, Monstru, Urias, Telefon. Noteaza care suna bine
      si care trebuie reglata (valorile stau in `windows/src/effects/voices.rs`).
- [ ] Cursoarele Advanced: muti Pitch, porneste transmisia de pe telefon, asteapta un minut.
      **Cursorul trebuie sa ramana unde l-ai lasat.** (Asta era bug-ul.)
- [ ] Schimbi vocea si te intorci: reglajele tale pentru vocea aceea sunt tot acolo.
- [ ] „Reset to voice” readuce doar vocea curenta la valorile ei.
- [ ] Wobble si Metal se aud pe orice voce, chiar si pe Normal.
- [ ] Limba: globul din coltul de sus, langa minimizare. Treci prin toate 10 si cauta
      texte taiate sau iesite din butoane (germana si rusa au cuvinte lungi).
- [ ] Limba ramane aleasa dupa ce inchizi si redeschizi programul.
- [ ] Fereastra: trage de colt de la ecran complet pana la cea mai mica marime.
      Nu trebuie sa apara niciodata bara de derulare pe orizontala.
      Pe ecranul Connection, toate 4 metodele trebuie sa se vada fara derulare.
- [ ] Varianta gratuita: in `windows/app/ui/index.html` pune `pro: false` in `state`,
      reconstruieste si verifica: vocile Pro sunt gri si nu se pot alege, Advanced e blocat
      cu mesajul despre telefon.
