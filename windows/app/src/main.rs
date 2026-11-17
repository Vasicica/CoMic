//! Fereastra CoMic.
//!
//! Interfata vine din `ui/`, generata dupa specificatia din `design/README.md`.
//! Deocamdata fereastra doar afiseaza interfata; legarea ei de motorul audio
//! urmeaza imediat dupa.

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("fereastra CoMic nu a putut porni");
}
