//! CoMic — receptorul de Windows.
//!
//! Primeste audio de la telefon prin UDP si il reda in microfonul virtual,
//! ca orice aplicatie (CS2, Discord, Chrome) sa il vada ca microfon normal.

use comic_core::{audio, effects, jitter, monitor, net};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use jitter::JitterBuffer;

struct Options {
    port: u16,
    device: Option<String>,
    gain: f32,
    voice: Option<String>,
    /// Cand e dat, sunetul vine de la acest microfon in loc de retea.
    input: Option<String>,
    buffer_frames: usize,
    voice_settings: effects::VoiceSettings,
    list_devices: bool,
    list_inputs: bool,
    list_voices: bool,
}

const USAGE: &str = "\
CoMic receiver

Folosire:
  comic-receiver [OPTIUNI]

Optiuni:
  --port <PORT>      portul UDP pentru audio (implicit 45678)
  --device <NUME>    fragment din numele dispozitivului de redare
                     (implicit: se cauta \"CABLE Input\", apoi cel implicit)
  --gain <FACTOR>    amplificare, 1.0 = neschimbat (implicit 1.0)
  --voice <NUME>     voce din catalog (vezi --list-voices)
  --input <NUME>     asculta de proba: sunetul vine de la acest microfon
                     in loc de retea, si iese in dispozitivul ales cu --device
  --buffer <1-10>    marimea rezervei, in cadre de 10 ms (implicit 3)
                     mai mic = latenta mai mica, mai mare = rezista la retea slaba
  --mix <0-100>      cat de expus e efectul, in procente (implicit 100)
                     0 = numai vocea ta, 50 = jumatate-jumatate, 100 = numai efectul
  --low-cut <HZ>     taie sub frecventa data (bubuit, vant) — incearca 100-300
  --high-cut <HZ>    taie peste frecventa data (sasait, ascutis) — incearca 3000-8000
  --pitch <SEMITON>  inlocuieste inaltimea vocii alese, de la -12 la +12
  --drive <0-10>     inlocuieste distorsiunea vocii alese
  --list-devices     afiseaza dispozitivele de redare si iese
  --list-inputs      afiseaza microfoanele si iese
  --list-voices      afiseaza vocile disponibile si iese
  --help             acest mesaj
";

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        port: net::DEFAULT_AUDIO_PORT,
        device: None,
        gain: 1.0,
        voice: None,
        input: None,
        buffer_frames: jitter::DEFAULT_PREFILL_FRAMES,
        voice_settings: effects::VoiceSettings::default(),
        list_devices: false,
        list_inputs: false,
        list_voices: false,
    };
    let mut arguments = std::env::args().skip(1);

    while let Some(argument) = arguments.next() {
        let mut value = || arguments.next().ok_or(format!("{argument} cere o valoare"));
        match argument.as_str() {
            "--port" => options.port = value()?.parse().map_err(|_| "port invalid".to_string())?,
            "--device" => options.device = Some(value()?),
            "--gain" => options.gain = value()?.parse().map_err(|_| "gain invalid".to_string())?,
            "--voice" => options.voice = Some(value()?),
            "--input" => options.input = Some(value()?),
            "--buffer" => {
                options.buffer_frames =
                    value()?.parse().map_err(|_| "buffer invalid".to_string())?
            }
            "--mix" => {
                let percent: f32 = value()?.parse().map_err(|_| "mix invalid".to_string())?;
                options.voice_settings.mix = Some(percent / 100.0);
            }
            "--low-cut" => {
                options.voice_settings.low_cut_hz = Some(
                    value()?
                        .parse()
                        .map_err(|_| "low-cut invalid".to_string())?,
                )
            }
            "--high-cut" => {
                options.voice_settings.high_cut_hz = Some(
                    value()?
                        .parse()
                        .map_err(|_| "high-cut invalid".to_string())?,
                )
            }
            "--pitch" => {
                options.voice_settings.semitones =
                    Some(value()?.parse().map_err(|_| "pitch invalid".to_string())?)
            }
            "--drive" => {
                options.voice_settings.drive =
                    Some(value()?.parse().map_err(|_| "drive invalid".to_string())?)
            }
            "--list-devices" => options.list_devices = true,
            "--list-inputs" => options.list_inputs = true,
            "--list-voices" => options.list_voices = true,
            "--help" | "-h" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("optiune necunoscuta: {other}")),
        }
    }

    Ok(options)
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    if options.list_devices {
        println!("Dispozitive de redare:");
        for name in audio::output_device_names() {
            println!("  {name}");
        }
        return;
    }

    if options.list_inputs {
        println!("Microfoane:");
        for name in monitor::input_device_names() {
            println!("  {name}");
        }
        return;
    }

    if options.list_voices {
        println!("Voci disponibile:");
        for preset in effects::PRESETS {
            let tier = if preset.free {
                "proba gratuita"
            } else {
                "versiune platita"
            };
            println!(
                "  {:<14} {:<14} {} ({tier})",
                preset.id, preset.name, preset.description
            );
        }
        return;
    }

    let voice = match options.voice.as_deref() {
        Some(id) => match effects::find_preset(id) {
            Some(preset) => {
                println!("[voce] {} - {}", preset.name, preset.description);
                Some(effects::VoiceChain::new(preset, options.voice_settings))
            }
            None => {
                eprintln!("Vocea \"{id}\" nu exista. Vezi --list-voices.");
                std::process::exit(2);
            }
        },
        None => None,
    };

    match options.input.clone() {
        Some(microphone) => run_monitor(&options, &microphone, voice),
        None => run_network(&options, voice),
    }
}

/// Ascultare de proba: sunetul vine de la un microfon deja instalat in Windows.
/// Pana exista aplicatia de telefon, asa poti auzi vocile cu vocea ta.
fn run_monitor(options: &Options, microphone: &str, voice: Option<effects::VoiceChain>) {
    let queue = Arc::new(Mutex::new(monitor::MicQueue::new()));

    let capture = match monitor::start_capture(Some(microphone), Arc::clone(&queue)) {
        Ok(capture) => capture,
        Err(error) => {
            eprintln!("[microfon] {error}");
            eprintln!("Sfat: vezi microfoanele disponibile cu --list-inputs.");
            std::process::exit(1);
        }
    };
    println!("[microfon] ascult de la \"{}\"", capture.device_name);

    let playback = start_playback(options, voice, move |mono| match queue.lock() {
        Ok(mut queue) => {
            queue.fill(mono);
        }
        Err(_) => mono.fill(0),
    });

    println!("[gata] vorbeste - te auzi in \"{}\".", playback.device_name);
    println!("Ctrl+C pentru oprire.");
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

/// Functionarea normala: sunetul vine de pe telefon, prin retea.
fn run_network(options: &Options, voice: Option<effects::VoiceChain>) {
    let buffer = Arc::new(Mutex::new(JitterBuffer::with_prefill(
        options.buffer_frames,
    )));

    let buffer_for_playback = Arc::clone(&buffer);
    let playback = start_playback(options, voice, move |mono| {
        match buffer_for_playback.lock() {
            Ok(mut buffer) => {
                buffer.fill(mono);
            }
            Err(_) => mono.fill(0),
        }
    });
    println!("[audio] redau in \"{}\"", playback.device_name);

    net::spawn_discovery(options.port);
    spawn_stats_reporter(Arc::clone(&buffer));

    if let Err(error) = net::receive_audio(options.port, buffer) {
        eprintln!("[retea] nu pot asculta pe portul {}: {error}", options.port);
        std::process::exit(1);
    }
}

fn start_playback(
    options: &Options,
    voice: Option<effects::VoiceChain>,
    fill_mono: impl FnMut(&mut [i16]) + Send + 'static,
) -> audio::Playback {
    match audio::start(options.device.as_deref(), options.gain, voice, fill_mono) {
        Ok(playback) => playback,
        Err(error) => {
            eprintln!("[audio] {error}");
            eprintln!("Sfat: instaleaza VB-Cable, apoi verifica cu --list-devices.");
            std::process::exit(1);
        }
    }
}

/// Afiseaza o linie de stare pe secunda, cat timp curge audio.
fn spawn_stats_reporter(buffer: Arc<Mutex<JitterBuffer>>) {
    std::thread::spawn(move || {
        let mut previous_received = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let Ok(buffer) = buffer.lock() else { continue };
            let stats = buffer.stats;
            if stats.frames_received == previous_received {
                continue; // liniste: nu umplem consola degeaba
            }
            previous_received = stats.frames_received;

            println!(
                "[stare] buffer {:>3} ms | cadre {} | intarziate {} | pierdute {} | taiate {} | underrun {}",
                buffer.buffered_ms(),
                stats.frames_received,
                stats.frames_dropped_late,
                stats.frames_missing,
                stats.frames_trimmed,
                stats.underruns
            );
        }
    });
}
