# CoMic pentru Android

Trimite microfonul telefonului catre receptorul CoMic din Windows, prin UDP.

## Ce face

1. Citeste microfonul la 48 kHz, mono, 16 biti.
2. Taie sunetul in cadre de 10 ms (480 esantioane).
3. Pune in fata antetul CoMic v1 (12 octeti) si trimite pachetul prin UDP.
4. Gaseste calculatorul singur, prin broadcast, sau accepta adresa scrisa de mana.

Captura ruleaza intr-un serviciu in prim-plan, deci merge si cu ecranul stins.
Notificarea permanenta nu e un capriciu: din Android 9, o aplicatie trimisa in
fundal pierde microfonul in cateva secunde fara ea.

## Compilare locala

Asta e calea care merge si e cea rapida: o schimbare ajunge pe telefon in
vreo douazeci de secunde.

Ai nevoie de trei lucruri: **JDK 17**, **uneltele Android de linie de
comanda** si **Gradle**. Pune-le unde vrei; `local.properties` (care nu se
comite, fiind al masinii tale) arata unde e SDK-ul:

    sdk.dir=C:/Android/sdk

Pachetele din SDK: `platform-tools` (acolo e `adb`), `platforms/android-36` si
`build-tools/36.0.0`. Trebuie **36**, nu 35: aplicatia tinteste Android 16
(vezi `app/build.gradle.kts`), iar `compileSdk` nu poate fi mai mic decat
`targetSdk`. Se iau asa:

    android sdk install "platforms;android-36" "build-tools;36.0.0"

Vechiul `sdkmanager` e invechit si inlocuit de programul `android`. Daca totusi
il chemi, tine minte ca invelisul lui `.bat` rupe numele pachetului la
punct-si-virgula: primesti „Package not found" si, cu toate astea, iesire
reusita, deci nici macar nu afli ca nu s-a instalat nimic.

Wrapperul (`gradlew`, `gradlew.bat`, `gradle/wrapper/gradle-wrapper.jar`) e in
depozit, deci cine cloneaza nu are nevoie de un Gradle instalat: wrapperul si-l
aduce singur, in versiunea scrisa in `gradle-wrapper.properties`.

## Compilare in cloud, cu EAS

**Nu merge deocamdata.** E pregatita, dar se opreste; vezi mai jos de ce.
Pentru lucru de zi cu zi foloseste compilarea locala, care e si mult mai
rapida: EAS a stat 69 de minute la coada inainte sa inceapa.

    eas login
    eas init          # o singura data: scrie projectId in app.json
    npm run apk       # porneste compilarea

Configuratia e in `.eas/build/android-apk.yml`, iar profilul in `eas.json`.
EAS compileaza din ce e **comis in git**, nu din ce e pe disc.

**Ce mai lipseste.** Primul pas al configuratiei tipareste ce unelte are
imaginea EAS, si de-acolo s-a aflat: imaginea are **Java 11**, iar proiectul
cere Java 17. Se alege alta imagine din `eas.json`, prin campul `image` al
profilului. Wrapperul, cealalta piedica, a fost intre timp adus in depozit.

**Foloseste `npm run apk`, nu `eas build` direct.** Comanda directa se opreste
cu `Could not read applicationId from Android project`: eas-cli vede folderul
`android/` si cauta identificatorul aplicatiei cu expresia
`/applicationId ['"](.*)['"]/` — spatiu, apoi ghilimea, adica sintaxa Groovy.
Noi scriem Kotlin DSL, unde semnul egal e obligatoriu, deci expresia nu
potriveste nimic. `tools/eas_apk.mjs` citeste identificatorul din Gradle si il
da mai departe prin variabila de mediu pe care eas-cli o verifica inaintea
citirii.

**Nu e calea obisnuita a EAS.** Exemplul oficial trece prin `eas/prebuild`,
care genereaza folderul `android/` dintr-un proiect React Native — adica ar
rescrie exact munca de aici. Noi folosim EAS doar ca masina de compilare.

## Cum se compileaza si se instaleaza

    ./gradlew assembleDebug     # face APK-ul
    ./gradlew installDebug      # il pune pe telefonul legat prin USB
    ./gradlew test              # testele de protocol, fara telefon

APK-ul iese in `app/build/outputs/apk/debug/`.

## Cum se foloseste

Pe calculator: deschizi CoMic, mergi la Options, alegi dispozitivul de ieșire
(`CABLE Input` ca sa te auda celelalte programe, difuzoarele ca sa auzi tu),
apoi apesi **Quick Connect**.

Pe telefon: apesi **Caută calculatorul**. Daca reteaua lasa broadcastul sa
treaca, calculatorul apare in lista cu numele lui. Daca nu apare, scrii adresa
de mana — unele rutere filtreaza broadcastul, si atunci asta e singura cale.
Apoi **Porneste**.

## Testele

`app/src/test/` verifica partea care trebuie sa fie exacta: octetii pachetului.
Valorile asteptate nu sunt ghicite — au fost generate cu acelasi calcul pe care
il face `tools/test_tone.py`, trimitatorul pe care receptorul din Windows il
accepta deja. Daca un test de acolo cade, pachetul nostru nu mai seamana cu ce
stie sa citeasca `windows/src/proto.rs`.

Testele ruleaza pe JVM, deci nu cer nici telefon, nici emulator.

## Ce nu e facut inca

- **Opus.** Acum se trimite PCM brut: 768 kbit/s, bine pe Wi-Fi 5 GHz si prin
  USB, prea mult pe 2,4 GHz aglomerat. Protocolul are deja loc pentru codec = 1.
- **Canal de control (TCP).** Fara el nu exista HELLO, PING sau BYE: telefonul
  trimite in gol si nu stie daca mai asculta cineva. Vezi `docs/PROTOCOL.md`.
- **Criptare.** `docs/PROTOCOL.md` cere pereche prin cod QR si AES-256-GCM.
  Pana atunci sunetul circula in clar prin retea — bine in casa, nu intr-o
  retea publica.
- **USB si Bluetooth.** `docs/CONEXIUNI.md` le descrie; aici e doar Wi-Fi.
- **Pictograma pentru bara de stare.** Notificarea foloseste deocamdata
  microfonul din resursele sistemului.

## Unde e ce

    app/src/main/kotlin/md/comic/phone/
      MainActivity.kt           fereastra, nimic altceva
      protocol/Protocol.kt      antetul pachetului si descoperirea (Kotlin curat)
      stream/Streamer.kt        microfon -> cadre -> UDP
      stream/StreamService.kt   serviciul in prim-plan si starea publicata
      stream/DiscoveryClient.kt broadcastul care gaseste calculatorul
      ui/StreamScreen.kt        interfata
