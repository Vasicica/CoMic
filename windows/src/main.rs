//! CoMic — receptorul de Windows.
//!
//! Primeste audio de la telefon prin UDP si il reda in microfonul virtual,
//! ca orice aplicatie (CS2, Discord, Chrome) sa il vada ca microfon normal.

use std::thread;
use std::time::Duration;

use comic_core::effects;
use comic_core::engine::{Engine, Source, StartRequest};
use comic_core::firewall::{self, Diagnosis};
use comic_core::{audio, jitter, monitor, net};

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
    check_firewall: bool,
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
  --low-cut <HZ>     taie sub frecventa data (bubuit, vant), incearca 100-300
  --high-cut <HZ>    taie peste frecventa data (sasait, ascutis), incearca 3000-8000
  --pitch <SEMITON>  inlocuieste inaltimea vocii alese, de la -12 la +12
  --drive <0-10>     inlocuieste distorsiunea vocii alese
  --wobble <0-100>   tremurul vocii, in procente (0 = fara)
  --metal <0-200>    sunetul metalic, in Hz (0 = fara)
  --list-devices     afiseaza dispozitivele de redare si iese
  --list-inputs      afiseaza microfoanele si iese
  --list-voices      afiseaza vocile disponibile si iese
  --check-firewall   intreaba Windows daca Firewall-ul lasa telefonul sa ajunga
                     la noi pe fiecare retea, si iese
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
        check_firewall: false,
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
            "--wobble" => {
                let percent: f32 = value()?.parse().map_err(|_| "wobble invalid".to_string())?;
                options.voice_settings.wobble = Some(percent / 100.0);
            }
            "--metal" => {
                options.voice_settings.metal_hz =
                    Some(value()?.parse().map_err(|_| "metal invalid".to_string())?)
            }
            "--list-devices" => options.list_devices = true,
            "--list-inputs" => options.list_inputs = true,
            "--list-voices" => options.list_voices = true,
            "--check-firewall" => options.check_firewall = true,
            "--help" | "-h" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("optiune necunoscuta: {other}")),
        }
    }

    Ok(options)
}

/// Spune ce a raspuns Windows despre propriul lui Firewall.
///
/// Mesajul e scris ca pentru cineva care tocmai a apasat Porneste si nu aude
/// nimic: ce retea, ce profil, si ce comanda rezolva. Nu "verifica Firewall-ul",
/// care nu i-a ajutat niciodata pe nimeni.
fn report_firewall() {
    let (networks, rules) = match firewall::ask_windows() {
        Ok(raspuns) => raspuns,
        Err(error) => {
            eprintln!("Nu am putut intreba Windows: {error}");
            return;
        }
    };

    println!("Retele active:");
    for network in &networks {
        println!(
            "  {} ({}): {}",
            network.name, network.interface, network.category
        );
    }

    println!("\nReguli de Firewall pentru CoMic:");
    if rules.is_empty() {
        println!("  (niciuna)");
    }
    for rule in &rules {
        println!("  {} -> {}", rule.name, rule.profiles);
    }

    println!();
    match firewall::diagnose(&networks, &rules) {
        Diagnosis::LooksFine => {
            println!("Firewall-ul lasa telefonul sa ajunga la noi pe toate retelele active.");
            println!("Daca tot nu se aude, cauza e in alta parte: adresa gresita pe telefon,");
            println!("alt port, sau dispozitivul de iesire.");
        }
        Diagnosis::NoRule => {
            println!("Nu exista nicio regula care sa ne lase sa primim legaturi.");
            println!("Windows a intrebat la prima pornire si raspunsul a fost nu.");
            println!("Nu mai intreaba a doua oara; regula se adauga de mana.");
        }
        Diagnosis::Uncovered(descoperite) => {
            println!("Firewall-ul blocheaza telefonul pe retelele astea:");
            for network in &descoperite {
                println!(
                    "  \"{}\" ({}) e {}, iar regulile noastre nu acopera profilul asta.",
                    network.name, network.interface, network.category
                );
            }
            println!("\nAsa se repara, din PowerShell pornit ca administrator:");
            for network in &descoperite {
                println!(
                    "  Set-NetConnectionProfile -InterfaceAlias \"{}\" -NetworkCategory Private",
                    network.interface
                );
            }
            println!("\nMarcheaza reteaua ca de incredere. Pentru un cablu catre propriul");
            println!("telefon e potrivit; pentru o retea publica, gandeste-te de doua ori.");
        }
    }
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    if options.check_firewall {
        report_firewall();
        return;
    }

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
            let tier = if preset.free { "gratuita" } else { "Pro" };
            println!(
                "  {:<14} {:<14} {} ({tier})",
                preset.id, preset.name, preset.description
            );
        }
        return;
    }

    // Validam vocea aici, ca utilizatorul sa afle imediat daca a gresit numele,
    // nu dupa ce motorul porneste si tace.
    if let Some(id) = options.voice.as_deref() {
        match effects::find_preset(id) {
            Some(preset) => println!("[voce] {} - {}", preset.name, preset.description),
            None => {
                eprintln!("Vocea \"{id}\" nu exista. Vezi --list-voices.");
                std::process::exit(2);
            }
        }
    }

    let source = if options.input.is_some() {
        Source::Microphone
    } else {
        Source::Network
    };

    let engine = Engine::spawn();
    engine.start(StartRequest {
        source,
        output_device: options.device.clone(),
        input_device: options.input.clone(),
        port: options.port,
        buffer_frames: options.buffer_frames,
        gain: options.gain,
        voice: options
            .voice
            .clone()
            .unwrap_or_else(|| "normal".to_string()),
        settings: options.voice_settings,
    });

    report_until_interrupted(&engine, source);
}

/// Afiseaza starea o data pe secunda, pana la Ctrl+C.
fn report_until_interrupted(engine: &Engine, source: Source) -> ! {
    let mut announced = false;
    let mut previous_received = 0;

    loop {
        thread::sleep(Duration::from_secs(1));
        let status = engine.status();

        if let Some(error) = status.error.as_deref() {
            eprintln!("[eroare] {error}");
            eprintln!("Sfat: vezi dispozitivele cu --list-devices si --list-inputs.");
            std::process::exit(1);
        }

        if status.running && !announced {
            announced = true;
            match source {
                Source::Microphone => println!(
                    "[gata] vorbeste - de la \"{}\" te auzi in \"{}\". Ctrl+C pentru oprire.",
                    status.input_device, status.output_device
                ),
                Source::Network => println!(
                    "[gata] redau in \"{}\". Ctrl+C pentru oprire.",
                    status.output_device
                ),
            }
        }

        // La microfon nu exista statistici de retea, deci aratam doar nivelul.
        if source == Source::Microphone {
            continue;
        }

        if status.frames_received == previous_received {
            continue; // liniste: nu umplem consola degeaba
        }
        previous_received = status.frames_received;

        println!(
            "[stare] buffer {:>3} ms | cadre {} | pierdute {} | underrun {} | nivel {:.2}",
            status.buffered_ms,
            status.frames_received,
            status.frames_missing,
            status.underruns,
            status.level
        );
    }
}
