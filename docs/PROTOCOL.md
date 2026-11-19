# Protocol CoMic v1 (schita)

Header de 12 bytes, big-endian, urmat de payload.

    offset  dim  camp
    0       2    magic = 0x434D ("CM")
    2       1    versiune = 1
    3       1    codec: 0 = PCM s16le, 1 = Opus
    4       4    numar de secventa (u32, creste cu 1 pe cadru)
    8       4    timestamp in esantioane (u32)
    12      -    payload (cadru Opus sau PCM de 10 ms)

Cadru de 10 ms la 48 kHz mono = 480 esantioane = 960 bytes PCM.

## Canal de control (TCP)

**Scris si pornit.** Vezi `windows/src/control.rs`.

Telefonul se leaga prin TCP pe **acelasi numar de port** pe care audio circula
prin UDP (implicit 45678): un singur numar de tinut minte, si o singura
poveste cand trebuie deschis in Firewall. Calculatorul e serverul, telefonul
se conecteaza, fiindca tot el stie adresa.

Mesajele sunt JSON, cate unul pe linie, incheiate cu `\n`. JSON si nu un
format propriu pe pozitii pentru ca pe canalul asta va circula mai tarziu
starea Pro, apoi perechea si cheia: cu JSON, un camp nou nu strica telefoanele
vechi, fiindca lipsa lui da valoarea implicita.

    telefon -> PC   {"t":"hello","v":1,"name":"Pixel 7","rate":48000,"codec":"pcm"}
    PC -> telefon   {"t":"hello-ack","v":1,"name":"DESKTOP-VASIL","port":45678,"rate":48000,"codec":"pcm"}
    telefon -> PC   {"t":"ping","seq":12,"rtt":23}
    PC -> telefon   {"t":"pong","seq":12}
    telefon -> PC   {"t":"mute","on":true}
    telefon -> PC   {"t":"bye"}
    PC -> telefon   {"t":"error","reason":"..."}

**Cine masoara dus-intorsul.** Telefonul face PING, calculatorul raspunde PONG.
Asa telefonul afla singur daca mai asculta cineva, care e intrebarea lui cea
mai importanta, iar calculatorul afla ca telefonul a plecat din chiar tacerea
lui. Numarul masurat de telefon se intoarce in PING-ul urmator, ca sa-l poata
arata si fereastra de PC.

**Reguli de purtare.**

- `hello` trebuie sa fie primul mesaj. Orice altceva inainte inchide legatura
  in tacere: inseamna un program strain nimerit pe portul nostru.
- O versiune straina primeste `error` si legatura se inchide.
- Trei linii neintelese pe aceeasi legatura o inchid. Cateva sunt iertate (un
  telefon mai nou ne-ar putea trimite un mesaj necunoscut), dar fara limita un
  program strain ar primi erori la nesfarsit.
- Zece secunde de tacere inseamna telefon plecat. El trimite PING la doua
  secunde, deci cinci batai pierdute inseamna ceva real.
- Un singur telefon pe rand.
- Numele telefonului se taie la 40 de caractere si i se scot caracterele de
  control, inainte sa ajunga in fereastra.

**Ce nu face inca.** Nu poarta starea Pro (Faza 3), nu face perechea si nu
transporta chei (vezi Securitate). Sunetul curge si fara el: un telefon care nu
stie de canal trimite UDP ca inainte, iar fereastra arata rezerva fara sa stie
al cui e telefonul.

## Securitate
Cheie derivata din codul de pairing (QR sau 6 cifre) prin HKDF.
Payload criptat AES-256-GCM; nonce = numarul de secventa.
Numerele de secventa se verifica pentru a respinge pachete reluate.
