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

## Compilare in cloud, cu EAS (fara Android Studio)

Daca ai `eas-cli` si un cont Expo, APK-ul se face pe serverele lor si nu-ti mai
trebuie nimic instalat local:

    eas login
    eas init          # o singura data: scrie projectId in app.json
    npm run apk       # sau: eas build --platform android --profile apk

Configuratia e in `.eas/build/android-apk.yml`, iar profilul in `eas.json`.

**Nu e calea obisnuita a EAS.** Exemplul oficial trece prin `eas/prebuild`,
care genereaza folderul `android/` dintr-un proiect React Native — adica ar
rescrie exact munca de aici. Noi folosim EAS doar ca masina de compilare: ia
codul din git, cheama Gradle, intoarce APK-ul. Primul pas al configuratiei
tipareste ce unelte are imaginea, ca o eroare sa se vada la vedere.

EAS compileaza din ce e **comis in git**, nu din ce e pe disc: ce n-ai comis nu
ajunge in APK.

## Compilare locala

Codul se compileaza si local, dar Android nu merge fara uneltele lui. Ai nevoie
de **una** din cele doua variante:

**Simpla — Android Studio** (vreo 1,5 GB plus 5-8 GB de SDK): se descarca de la
developer.android.com/studio, deschizi folderul `android/` si apesi Sync.
Studio aduce singur JDK-ul, SDK-ul si Gradle.

**Slaba — numai linia de comanda** (vreo 2-3 GB): un JDK 17 si
`commandlinetools` din aceeasi pagina, apoi
`sdkmanager "platforms;android-35" "build-tools;35.0.0" "platform-tools"`.

Fisierele `gradlew`, `gradlew.bat` si `gradle/wrapper/gradle-wrapper.jar` nu
sunt aici (primele doua sunt scripturi, al treilea e binar). Android Studio le
reface la prima sincronizare; de la linia de comanda, `gradle wrapper` face
acelasi lucru odata ce ai Gradle.

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
