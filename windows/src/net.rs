//! Receptia audio pe UDP plus raspuns la descoperirea din retea.

use std::io;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::jitter::JitterBuffer;
use crate::proto::{self, ParseError};

pub const DEFAULT_AUDIO_PORT: u16 = 45_678;
pub const DISCOVERY_PORT: u16 = 45_679;

/// Cel mai mare pachet pe care il acceptam: un cadru PCM de 10 ms plus header.
const MAX_PACKET: usize = proto::HEADER_LEN + proto::FRAME_SAMPLES * 2 + 64;

const DISCOVERY_QUERY: &[u8] = b"COMIC-DISCOVER-1";

/// Cat asteptam un pachet inainte sa verificam daca ni s-a cerut sa ne oprim.
/// Fara acest termen, `recv_from` ar astepta la nesfarsit si firul nu s-ar mai
/// putea opri niciodata din interfata.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Bucla de receptie audio. Blocheaza firul curent pana cand `stop` devine true.
pub fn receive_audio(
    port: u16,
    buffer: Arc<Mutex<JitterBuffer>>,
    stop: Arc<AtomicBool>,
) -> io::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", port))?;
    socket.set_read_timeout(Some(POLL_INTERVAL))?;
    println!("[retea] ascult audio pe UDP {port}");

    let mut packet = [0u8; MAX_PACKET];
    while !stop.load(Ordering::Relaxed) {
        let (length, sender) = match socket.recv_from(&mut packet) {
            Ok(result) => result,
            // Termenul expirat inseamna doar liniste pe retea, nu o eroare.
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                continue
            }
            Err(error) => {
                eprintln!("[retea] recv a esuat: {error}");
                continue;
            }
        };

        match proto::parse_packet(&packet[..length]) {
            Ok(frame) => {
                if let Ok(mut buffer) = buffer.lock() {
                    buffer.push(frame);
                }
            }
            // Pachetele stricate sau de la alte programe se ignora in liniste;
            // doar codecul neimplementat merita un mesaj, fiind o nepotrivire
            // reala de versiune intre telefon si calculator.
            Err(ParseError::CodecNotImplemented(codec)) => {
                eprintln!("[retea] {sender} trimite {codec:?}, inca nesuportat");
            }
            Err(_) => {}
        }
    }

    println!("[retea] am oprit ascultarea pe UDP {port}");
    Ok(())
}

/// Raspunde telefoanelor care intreaba prin broadcast unde este calculatorul.
pub fn serve_discovery(audio_port: u16, stop: Arc<AtomicBool>) -> io::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(POLL_INTERVAL))?;
    println!("[retea] raspund la descoperire pe UDP {DISCOVERY_PORT}");

    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".to_string());
    let reply = format!("COMIC-HERE-1 {hostname} {audio_port}");

    let mut packet = [0u8; 64];
    while !stop.load(Ordering::Relaxed) {
        let Ok((length, sender)) = socket.recv_from(&mut packet) else {
            continue;
        };
        if &packet[..length] == DISCOVERY_QUERY {
            let _ = socket.send_to(reply.as_bytes(), sender);
        }
    }

    Ok(())
}

/// Porneste serviciul de descoperire pe un fir separat, fara a opri programul
/// daca portul este deja ocupat — descoperirea este un confort, nu o cerinta.
pub fn spawn_discovery(audio_port: u16, stop: Arc<AtomicBool>) {
    thread::spawn(move || {
        if let Err(error) = serve_discovery(audio_port, stop) {
            eprintln!("[retea] descoperirea nu a pornit: {error}");
        }
    });
}
