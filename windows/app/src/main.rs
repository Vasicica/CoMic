//! Fereastra CoMic: leaga interfata de motorul audio.
//!
//! Interfata nu atinge niciodata direct fluxurile audio. Ii trimite motorului
//! comenzi si ii citeste starea, pentru ca fluxurile cpal nu pot trai decat pe
//! firul care le-a creat (vezi comic_core::engine).

// Fara consola neagra in spatele ferestrei, la varianta de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod prefs;

use std::path::PathBuf;

use comic_core::effects::{VoicePreset, VoiceSettings, PRESETS};
use comic_core::engine::{Engine, StartRequest, Status};
use comic_core::{audio, monitor};
use prefs::{Prefs, PrefsStore};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, State, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

/// Spune daca pictograma din bara de sistem a putut fi creata.
///
/// Fara ea, "ascunde in bara" ar fi o capcana cu usa intr-un singur sens:
/// fereastra s-ar ascunde si n-ar mai exista niciun loc de unde sa o aduci
/// inapoi. De-aceea preferinta se poate aprinde numai cand exista pictograma.
struct TrayPresent(bool);

#[tauri::command]
fn output_devices() -> Vec<String> {
    audio::output_device_names()
}

#[tauri::command]
fn input_devices() -> Vec<String> {
    monitor::input_device_names()
}

/// Adresa de unde se ia microfonul virtual.
///
/// Sta aici, nu in interfata, ca sa fie un singur loc de schimbat cand CoMic
/// isi va avea propriul driver si legatura asta nu va mai trebui nimanui.
const VB_CABLE_URL: &str = "https://vb-audio.com/Cable/";

/// Ce stie programul despre microfonul virtual.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct VirtualMic {
    /// Numele capatului ales, cand exista unul.
    ///
    /// Fereastra il pune in lista de dispozitive in loc sa-l caute singura.
    /// Cat timp il cauta singura, raspunsul statea in doua locuri si se puteau
    /// desparti: aici scria "CABLE Input", iar in interfata orice nume cu
    /// "cable" in el. Pe un Windows unde capatul se cheama "Динамики", cele
    /// doua dadeau raspunsuri diferite.
    name: Option<String>,
    /// De unde se ia. Fereastra o primeste ca sa o poata arata scrisa daca
    /// butonul nu reuseste sa deschida browserul. Altfel adresa ar sta in
    /// doua locuri, aici si in interfata, si s-ar desparti la prima schimbare.
    installer_url: String,
}

/// Exista microfonul virtual pe calculatorul asta, si care e?
///
/// Fereastra intreaba la pornire, nu dupa ce apesi Conecteaza. Fara el,
/// sunetul pleaca in difuzoare: te auzi, deci crezi ca merge, dar in Discord
/// nu ajunge nimic. E mai bine sa afli inainte sa vorbesti degeaba.
#[tauri::command]
fn virtual_mic() -> VirtualMic {
    VirtualMic {
        name: audio::virtual_mic_name(),
        installer_url: VB_CABLE_URL.to_string(),
    }
}

/// Deschide pagina de unde se ia VB-Cable.
///
/// Intoarce ce s-a intamplat, nu ce s-a cerut: daca browserul nu s-a deschis,
/// fereastra arata adresa scrisa, ca omul sa o poata copia de mana.
#[tauri::command]
fn open_vb_cable(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(VB_CABLE_URL, None::<&str>)
        .map_err(|error| error.to_string())
}

/// Catalogul de voci vine din Rust, ca interfata sa nu tina o copie proprie
/// care ar ramane in urma la prima voce adaugata.
#[tauri::command]
fn voices() -> Vec<VoicePreset> {
    PRESETS.to_vec()
}

#[tauri::command]
fn start(engine: State<Engine>, request: StartRequest) {
    engine.start(request);
}

#[tauri::command]
fn stop(engine: State<Engine>) {
    engine.stop();
}

#[tauri::command]
fn set_voice(engine: State<Engine>, voice: String, settings: VoiceSettings) {
    engine.set_voice(voice, settings);
}

#[tauri::command]
fn set_gain(engine: State<Engine>, gain: f32) {
    engine.set_gain(gain);
}

#[tauri::command]
fn status(engine: State<Engine>) -> Status {
    engine.status()
}

#[tauri::command]
fn read_prefs(store: State<PrefsStore>) -> Prefs {
    store.get()
}

/// Ce s-a intamplat de fapt, nu ce s-a cerut.
///
/// Distinctia e tot rostul acestei lucrari. Pornirea automata se scrie in
/// registrul Windows si poate fi refuzata; bara de sistem poate lipsi. Daca am
/// intoarce cererea, comutatorul ar ramane aprins peste un lucru care nu s-a
/// intamplat -- exact minciuna pe care o reparam.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefsResult {
    prefs: Prefs,
    error: Option<String>,
}

#[tauri::command]
fn write_prefs(app: AppHandle, store: State<PrefsStore>, prefs: Prefs) -> PrefsResult {
    let inainte = store.get();
    let mut obtinut = prefs;
    let mut eroare: Option<String> = None;

    // ---- pornirea odata cu Windows ----
    // Atingem registrul numai cand s-a cerut o schimbare: scrierea la fiecare
    // apasare ar rescrie degeaba calea executabilului.
    if prefs.launch_at_startup != inainte.launch_at_startup {
        let autolaunch = app.autolaunch();
        let scris = if prefs.launch_at_startup {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(problema) = scris {
            // Intrebam sistemul cum sta de fapt. Daca nici el nu raspunde,
            // ramane valoarea de dinainte: nu inventam o reusita.
            obtinut.launch_at_startup =
                autolaunch.is_enabled().unwrap_or(inainte.launch_at_startup);
            eroare = Some(format!(
                "Windows startup entry could not be changed: {problema}"
            ));
        }
    }

    // ---- ascunderea in bara de sistem ----
    if prefs.minimize_to_tray && !app.state::<TrayPresent>().0 {
        obtinut.minimize_to_tray = false;
        eroare = Some("No system tray icon, so the window cannot hide there.".into());
    }

    if let Err(problema) = store.set(obtinut) {
        eroare = Some(format!(
            "Settings could not be saved to {}: {problema}",
            store.path().display()
        ));
    }

    PrefsResult {
        prefs: obtinut,
        error: eroare,
    }
}

/// Aduce fereastra in fata, din orice stare: ascunsa, minimizata sau in spate.
///
/// Toate trei sunt necesare. `show` singur nu ridica o fereastra minimizata si
/// nici n-o aduce peste celelalte.
fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open CoMic", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let mut tray = TrayIconBuilder::new()
        .tooltip("CoMic")
        .menu(&menu)
        // Meniul numai pe clic dreapta. Daca ar aparea si pe stanga, clicul
        // stanga n-ar mai putea deschide fereastra -- si asta e lucrul pe care
        // il incearca oricine intai.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            // Ieșirea trebuie sa fie ieșire. `exit` nu trece prin cererea de
            // inchidere a ferestrei, deci nu se loveste de ascunderea in bara.
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Pe ridicarea butonului, nu pe apasare: sistemul trimite ambele,
            // si pe apasare fereastra ar clipi.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        });

    // Pictograma vine din `bundle.icon` din tauri.conf.json. Daca lipseste,
    // facem pictograma fara imagine: un loc gol in bara e mai bun decat o
    // fereastra care crapa la pornire.
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.build(app)?;
    Ok(())
}

/// Unde stau preferintele: `%APPDATA%\<identificator>\prefs.json`.
///
/// Acelasi loc pe care l-ar alege si `app.path().app_config_dir()`, dar
/// calculat fara handle-ul aplicatiei. Motivul e ordinea din Tauri: `setup`
/// creeaza fereastra **inainte** sa cheme inchiderea noastra de pornire, deci
/// interfata ar putea cere preferintele inainte sa fie inregistrate si comanda
/// ar crapa. Calculand calea devreme, le inregistram pe constructor si cursa
/// dispare.
///
/// Fara `%APPDATA%` cadem pe folderul temporar: preferintele nu vor trece o
/// repornire, dar programul merge.
fn prefs_path(identifier: &str) -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(identifier)
        .join("prefs.json")
}

fn main() {
    // Contextul ne da identificatorul scris in tauri.conf.json, ca sa nu-l
    // copiem a doua oara in cod, de unde s-ar putea desincroniza.
    let context = tauri::generate_context!();
    let store = PrefsStore::load(prefs_path(&context.config().identifier));

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(Engine::spawn())
        .manage(store)
        .setup(|app| {
            // Sistemul e adevarul, nu fisierul nostru: daca intrarea din
            // registru a fost scoasa pe la spatele nostru, comutatorul trebuie
            // sa arate stins cand deschizi fereastra.
            let store = app.state::<PrefsStore>();
            let salvat = store.get();
            if let Ok(chiar_pornit) = app.autolaunch().is_enabled() {
                if chiar_pornit != salvat.launch_at_startup {
                    let _ = store.set(Prefs {
                        launch_at_startup: chiar_pornit,
                        ..salvat
                    });
                }
            }

            let tray = build_tray(app.handle());
            if let Err(problema) = &tray {
                eprintln!("CoMic: bara de sistem nu a putut fi pornita: {problema}");
                // Fara pictograma, preferinta ar fi o capcana. O stingem, ca
                // fereastra sa spuna adevarul la urmatoarea deschidere.
                let acum = store.get();
                if acum.minimize_to_tray {
                    let _ = store.set(Prefs {
                        minimize_to_tray: false,
                        ..acum
                    });
                }
            }
            app.manage(TrayPresent(tray.is_ok()));

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let ascunde = window.state::<PrefsStore>().get().minimize_to_tray
                    && window.state::<TrayPresent>().0;
                if ascunde {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            output_devices,
            input_devices,
            voices,
            start,
            stop,
            set_voice,
            set_gain,
            status,
            read_prefs,
            write_prefs,
            virtual_mic,
            open_vb_cable
        ])
        .run(context)
        .expect("fereastra CoMic nu a putut porni");
}
