# CoMic

**Use your Android phone as a wireless microphone for Windows.**
A free, open-source alternative to WO Mic, with no time limits and no ads, plus a built-in voice changer.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/Vasicica/CoMic/actions/workflows/ci.yml/badge.svg)](https://github.com/Vasicica/CoMic/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Vasicica/CoMic?include_prereleases)](https://github.com/Vasicica/CoMic/releases)

[Română](README.ro.md)

> **Status: early (v0.1).** It works daily on the author's machines, but expect rough edges.
> Bug reports are very welcome.

## Why

Most "phone as mic" apps cut you off after a few minutes, show ads, or stop when the
screen turns off. CoMic does none of that. The microphone is free and unlimited, forever.

## Features

- **Phone mic to PC** over Wi-Fi or USB tethering. Works with the screen off.
- **Automatic discovery** on the local network, or type the phone's address by hand.
- **Any app can use it** (Discord, OBS, Zoom, games) through [VB-Cable](https://vb-audio.com/Cable/).
  If VB-Cable is missing, CoMic tells you and links to it instead of failing silently.
- **Voice changer**, processed on the PC in Rust: Normal, Pirate, Old man and more,
  14 voices today, more coming with updates.
- **Firewall diagnostics:** when no sound arrives, CoMic asks Windows why and tells you.
- **10 languages:** English, Română, Русский, Українська, Polski, Türkçe, Deutsch,
  Français, Español, Português.
- Small, native apps: Kotlin + Jetpack Compose on Android, Rust + Tauri on Windows.

## How it works

```
[Android]                                   [Windows]
microphone 48 kHz mono                      jitter buffer
 -> 10 ms frames                            -> voice effects (Rust)
 -> UDP  ================================>  -> WASAPI output into VB-Cable
 Foreground service, keeps going             -> apps see "CABLE Output" as a microphone
 with the screen off
         TCP control channel (same port): hello, ping, mute
```

Default port: **45678** (UDP audio + TCP control). Details in [docs/PROTOCOL.md](docs/PROTOCOL.md).

Expected latency: about 25 to 40 ms over USB tethering, 50 to 80 ms on 5 GHz Wi-Fi.
Good for calls, streaming and gaming. Not low enough for live singing monitoring.

## Install

1. Download the latest **APK** (phone) and **CoMic.exe** (PC) from
   [Releases](https://github.com/Vasicica/CoMic/releases).
2. Install [VB-Cable](https://vb-audio.com/Cable/) on the PC (free).
3. Start CoMic on the PC, then on the phone. Pick the PC from the list or type its address.
4. In Discord / OBS / your game, choose **CABLE Output** as the microphone.

No sound? See [Troubleshooting](#troubleshooting).

## Build from source

**Windows app** (Rust workspace in `windows/`):

```bash
cd windows
cargo test               # audio core, no window needed
cd app && cargo run      # the desktop window (Tauri 2)
```

You need stable Rust and the [Tauri prerequisites for Windows](https://v2.tauri.app/start/prerequisites/).

**Android app** (`android/`): JDK 17 and Android SDK platform 36. Full instructions in
[android/README.md](android/README.md).

```bash
cd android
./gradlew test
./gradlew assembleDebug
```

**Translations** live in `shared/i18n/*.json`. After editing, run `node tools/i18n_build.mjs`.

## Troubleshooting

```bash
cd windows
cargo run -- --list-devices      # where the sound is actually going
cargo run -- --check-firewall    # is Windows Firewall blocking port 45678?
```

The most common cause: a new network (for example USB tethering) is classified as
**Public** in Windows while the firewall rule only allows **Private** networks.

## Roadmap

- [x] Wi-Fi and USB tethering connection
- [x] Voice changer with 14 voices
- [x] 10 languages
- [ ] Opus compression (today audio is sent as raw PCM)
- [ ] Pairing code + encryption (AES-GCM). **Until then, use CoMic only on networks you trust.**
- [ ] USB via adb, Bluetooth
- [ ] Noise suppression
- [ ] Own virtual microphone driver (no VB-Cable needed)
- [ ] iOS (maybe)

## Contributing

Translations are the easiest way in: one JSON file, no build tools needed.
See [CONTRIBUTING.md](CONTRIBUTING.md) and the issues labeled
[good first issue](https://github.com/Vasicica/CoMic/labels/good%20first%20issue).

## Pro version

The code here is free and complete. The Google Play build will offer an optional **Pro**
upgrade (extra voices and advanced voice tuning) to support development.
The microphone itself is, and will stay, free and unlimited.

## License

[MIT](LICENSE) © 2026 Ciolac Vasile.
Not affiliated with WO Mic or VB-Audio.
Please don't use the voice changer to impersonate real people or deceive anyone
(see [docs/legal/terms.en.md](docs/legal/terms.en.md)).
