//! Fereastra CoMic: leaga interfata de motorul audio.
//!
//! Interfata nu atinge niciodata direct fluxurile audio. Ii trimite motorului
//! comenzi si ii citeste starea, pentru ca fluxurile cpal nu pot trai decat pe
//! firul care le-a creat (vezi comic_core::engine).

// Fara consola neagra in spatele ferestrei, la varianta de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use comic_core::effects::{VoicePreset, VoiceSettings, PRESETS};
use comic_core::engine::{Engine, StartRequest, Status};
use comic_core::{audio, monitor};
use tauri::State;

#[tauri::command]
fn output_devices() -> Vec<String> {
    audio::output_device_names()
}

#[tauri::command]
fn input_devices() -> Vec<String> {
    monitor::input_device_names()
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

fn main() {
    tauri::Builder::default()
        .manage(Engine::spawn())
        .invoke_handler(tauri::generate_handler![
            output_devices,
            input_devices,
            voices,
            start,
            stop,
            set_voice,
            set_gain,
            status
        ])
        .run(tauri::generate_context!())
        .expect("fereastra CoMic nu a putut porni");
}
