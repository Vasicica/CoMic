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

## Canal de control (TCP sau UDP separat)
HELLO / HELLO-ACK (negociere versiune, rata, codec), PING / PONG (RTT si
detectie deconectare), BYE, MUTE.

## Securitate
Cheie derivata din codul de pairing (QR sau 6 cifre) prin HKDF.
Payload criptat AES-256-GCM; nonce = numarul de secventa.
Numerele de secventa se verifica pentru a respinge pachete reluate.
