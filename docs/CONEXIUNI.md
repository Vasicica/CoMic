# Modalitati de conectare telefon - calculator

Cerinta: utilizatorul alege metoda, exact ca in WO Mic. Programul nu impune una.

## 1. Wi-Fi cu adresa IP scrisa de mana (principal)

Modelul pe care il cunoaste utilizatorul din WO Mic:

1. Pe telefon apesi **Start**. Telefonul afiseaza mare adresa lui, de exemplu
   `192.168.1.3`, si asculta pe portul 45678.
2. Pe calculator apesi **Connect**, alegi metoda **Wi-Fi** si scrii adresa.
3. Enter. Esti conectat.

Tehnic: calculatorul deschide o conexiune TCP de control catre telefon si ii
spune pe ce port UDP asteapta audio. Telefonul incepe sa trimita UDP catre
calculator. Controlul pe TCP (sigur, ordonat), audio pe UDP (rapid).

**Descoperire automata**, ca bonus peste WO Mic: calculatorul trimite un
broadcast pe portul 45679 si telefoanele din retea raspund cu numele si adresa.
Nu inlocuieste scrierea manuala a IP-ului — unele retele blocheaza broadcast-ul,
si atunci metoda manuala este singura care functioneaza.

Latenta: 50-80 ms pe 5 GHz, 80-150 ms pe 2.4 GHz aglomerat.

## 2. Cablu USB (cea mai buna latenta)

Doua variante, amandoua utile:

**2a. Prin depanare USB** (ru: "отладка по USB") — metoda din WO Mic.
Utilizatorul activeaza Developer Options -> USB debugging. Calculatorul ruleaza
`adb forward tcp:45678 tcp:45678` si de acolo vorbeste cu telefonul prin
`127.0.0.1`. Necesita `adb.exe` livrat cu programul nostru (~5 MB, parte din
Android SDK Platform-Tools).

**2b. Prin partajare USB** (USB tethering) — fara depanare, fara adb.
Telefonul porneste partajarea prin USB, capata o adresa IP pe interfata USB
(de obicei `192.168.42.x`), iar restul merge exact ca la Wi-Fi. Mai simplu
pentru utilizatorul obisnuit, dar schimba temporar rutarea datelor pe telefon.

Latenta: 25-40 ms. Cel mai stabil mod, fara influenta retelei.

## 3. Bluetooth (pentru laptopuri)

Prin **RFCOMM / Serial Port Profile**, nu prin profilul de casti.
Profilul de casti (HSP/SCO) limiteaza sunetul la 8-16 kHz si suna prost —
exact capcana in care cad multe aplicatii. RFCOMM ne lasa sa trimitem Opus
la 24-32 kbps, deci calitate buna la banda mica.

Limitari de spus cinstit:
- Latenta 100-200 ms, cea mai mare dintre toate metodele.
- Majoritatea calculatoarelor de birou nu au Bluetooth; are sens mai ales la laptop.
- Imperecherea se face intai din setarile Windows, apoi din program.

Prioritate: dupa Wi-Fi si USB.

## 4. Wi-Fi Direct — exclus

Decizie luata: nu il implementam. Suport inconsecvent intre producatori de
telefoane, configurare greoaie, si nu aduce nimic peste Wi-Fi normal sau USB.

## Arhitectura care face posibile toate acestea

Un singur strat de transport cu mai multe implementari, nu patru programe lipite:

    trait Transport {
        fn connect(&mut self, target: &Target) -> Result<Session>;
        fn recv_frame(&mut self) -> Result<Frame>;
    }

    WifiTransport      -> UDP + control TCP
    UsbAdbTransport    -> adb forward, apoi acelasi cod ca Wi-Fi pe 127.0.0.1
    BluetoothTransport -> RFCOMM, cadre delimitate pe flux serial

Buffer-ul de jitter, decodorul si redarea raman neschimbate, indiferent de
metoda. Doar sursa pachetelor difera. Asta inseamna ca adaugarea Bluetooth-ului
mai tarziu nu atinge codul audio deja testat.
