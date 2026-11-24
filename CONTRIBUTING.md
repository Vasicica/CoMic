# Contributing to CoMic

Thanks for being here. This page is short on purpose.

## Add a translation (the easiest way in)

You need a text editor. No Android device, no Rust, no build.

Every language lives in **one file**: `shared/i18n/<code>.json`. Copy
`shared/i18n/en.json` to your language code and translate the values. Leave the
keys alone.

```bash
cp shared/i18n/en.json shared/i18n/it.json   # Italian, for example
```

Then run the generator and open a pull request with both the source file and
everything it changed:

```bash
node tools/i18n_build.mjs
```

It writes `windows/app/ui/i18n.js` and the Android `strings.xml` files. **Do not
edit those by hand** — they are generated and will be overwritten.

Four rules, all checked automatically. If you break one, the generator refuses
to run and names the key:

1. **Every key in `en.json` must exist in your file.** A missing key is an
   error, not a fallback. An app that silently shows English in random places is
   worse than one that fails loudly at build time.
2. **No extra keys.** If you add one, English does not have it, and every other
   language is now behind without anyone noticing.
3. **Placeholders must match English exactly.** `{port}`, `{phone}`, `{pc}` are
   filled in at runtime. Translate the words around them; never translate, drop
   or invent one. On Android `{pc}` becomes `%1$s`, so a mismatch is a crash,
   not a typo.
4. **No em dashes.** Use a colon or a full stop. They are banned project-wide so
   the ten languages read as one voice.

Two more things worth knowing:

- Keep it **short**. These strings sit in a phone-sized window. German and
  Russian run long; if a sentence doubles in length, rewrite it shorter rather
  than letting it clip.
- Write like a person talking, not like a manual. The English source is the
  tone to match.

Languages today: `en`, `ro`, `ru`, `uk`, `pl`, `tr`, `de`, `fr`, `es`, `pt`.

## Report a bug

Open an issue and say these three things:

- What you did, what you expected, what happened instead.
- Your Windows version, your Android version and phone model.
- How you connected: Wi-Fi, USB tethering, or a typed address.

If no sound arrives, run this first and paste the whole output. It answers the
two most common causes before anyone has to guess:

```bash
cargo run -- --list-devices
cargo run -- --check-firewall
```

The first says which device the sound would actually go to. The second asks
Windows whether its firewall is in the way.

## Send a pull request

- One change per pull request. A small one that lands beats a big one that
  stalls.
- Run the tests: `cd windows && cargo test`, and `cd android && ./gradlew test`.
- Match the style around you. Comments explain **why**, not what: the code
  already says what.
- If you changed behaviour, say in the pull request how you checked it on a real
  machine. "Tests pass" is not the same as "I heard it work".

## Building from source

**Phone app:** see `android/README.md`. It lists the exact JDK and SDK packages.

**Desktop app:** a Rust workspace in `windows/`. You need a stable Rust
toolchain and the usual Tauri prerequisites for Windows.

```bash
cd windows && cargo test          # core, no window needed
cd windows/app && cargo run       # the window itself
```

You also need [VB-Cable](https://vb-audio.com/Cable/) installed for sound to
reach other applications. Without it CoMic plays to your speakers and says so.

## Code of conduct

By taking part you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
