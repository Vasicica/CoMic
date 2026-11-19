//! Canalul de control: telefonul si calculatorul stiu unul de altul.
//!
//! Pana acum telefonul striga in gol. Trimitea UDP catre o adresa si nu afla
//! niciodata daca asculta cineva; calculatorul vedea pachete sosind si nu stia
//! nici al cui e telefonul, nici cand a plecat. De aici veneau toate orele
//! pierdute cu "am apasat si nu se aude": nu aveai de unde sti daca e reteaua,
//! portul sau dispozitivul de iesire.
//!
//! Canalul asta repara exact atat. TCP, pe **acelasi numar de port** ca audio,
//! ca sa fie un singur numar de tinut minte si o singura poveste la Firewall.
//!
//! **De ce JSON si nu un format propriu.** Pe canalul asta va circula mai
//! tarziu starea Pro, apoi perechea si cheia de criptare. Cu JSON si `serde`,
//! un camp nou nu strica telefoanele vechi: lipsa lui da valoarea implicita.
//! Un format propriu pe pozitii ar cere o versiune noua la fiecare adaugare.
//! Pretul, cativa octeti pe mesaj, nu conteaza: mesajele sunt rare, iar
//! sunetul circula pe celalalt drum.
//!
//! **Cine masoara dus-intorsul.** Telefonul face PING, calculatorul raspunde
//! PONG. Asa telefonul afla singur daca mai asculta cineva, care e intrebarea
//! lui cea mai importanta, iar calculatorul afla ca telefonul a plecat din
//! chiar tacerea lui. Numarul masurat de telefon se intoarce in PING-ul
//! urmator, ca sa-l putem arata si noi.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Versiunea canalului. Creste doar cand un telefon vechi nu mai poate vorbi
/// cu un calculator nou, nu la fiecare camp adaugat.
pub const VERSION: u8 = 1;

/// Cat asteptam intre doua incercari de a primi o legatura noua.
const ACCEPT_POLL: Duration = Duration::from_millis(200);

/// Cat asteptam date pe o legatura deschisa inainte sa verificam daca ni s-a
/// cerut sa ne oprim.
const READ_POLL: Duration = Duration::from_millis(200);

/// Dupa atata liniste consideram ca telefonul a plecat fara sa-si ia ramas bun
/// (a iesit din raza, s-a terminat bateria, a cazut Wi-Fi-ul). Telefonul
/// trimite PING la doua secunde, deci cinci batai pierdute inseamna ceva real.
const SILENCE_LIMIT: Duration = Duration::from_secs(10);

/// Cea mai lunga linie pe care o acceptam. Un mesaj de control are zeci de
/// octeti; orice peste atat inseamna ca nu vorbim cu CoMic.
const MAX_LINE: usize = 8 * 1024;

/// Cate linii neintelese ingaduim pe o legatura inainte sa o inchidem.
///
/// Cateva sunt iertate: un telefon mai nou ne-ar putea trimite un mesaj pe
/// care nu-l cunoastem inca, si ar fi prostesc sa rupem legatura pentru atat.
/// Dar fara nicio limita, un program strain care nimereste portul ar primi
/// erori la nesfarsit si ar tine legatura ocupata degeaba.
const MAX_CONFUSION: u32 = 3;

/// Ce ne trimite telefonul.
///
/// Toate campurile au valoare implicita: un telefon care trimite un mesaj
/// incomplet primeste zerouri, nu o legatura inchisa. La marginea sistemului,
/// ingaduinta e mai buna decat rigoarea.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
pub enum FromPhone {
    /// Primul mesaj de pe orice legatura.
    Hello {
        #[serde(default)]
        v: u8,
        #[serde(default)]
        name: String,
        #[serde(default)]
        rate: u32,
        #[serde(default)]
        codec: String,
    },
    /// Bataia de inima. `rtt` e ce a masurat telefonul la dus-intorsul anterior.
    Ping {
        #[serde(default)]
        seq: u32,
        #[serde(default)]
        rtt: Option<u32>,
    },
    Mute {
        #[serde(default)]
        on: bool,
    },
    /// Plecare anuntata. Fara ea tot am afla, dar abia dupa [`SILENCE_LIMIT`].
    Bye,
}

/// Ce ii raspundem telefonului.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum ToPhone {
    #[serde(rename = "hello-ack")]
    HelloAck {
        v: u8,
        name: String,
        port: u16,
        rate: u32,
        codec: String,
    },
    #[serde(rename = "pong")]
    Pong { seq: u32 },
    /// Motivul e in engleza: ajunge pe ecranul telefonului doar la nevoie, iar
    /// traducerea lui ar cere trimiterea unei chei, nu a unei propozitii.
    #[serde(rename = "error")]
    Error { reason: String },
}

/// Ce stie calculatorul despre telefonul legat acum.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub connected: bool,
    /// Numele telefonului, asa cum si l-a spus singur.
    pub name: String,
    pub address: String,
    /// Dus-intorsul masurat de telefon, in milisecunde.
    pub rtt_ms: Option<u32>,
    pub muted: bool,
}

/// Desface o linie primita de la telefon.
///
/// Intoarce `Err` cu motivul, pentru ca un mesaj pe care nu-l intelegem nu e
/// acelasi lucru cu o legatura rupta: raspundem si mergem mai departe.
pub fn parse_line(line: &str) -> Result<FromPhone, String> {
    serde_json::from_str(line.trim()).map_err(|error| error.to_string())
}

/// Imbraca un raspuns intr-o linie gata de trimis.
pub fn encode(message: &ToPhone) -> String {
    // Serializarea unei structuri fara campuri imposibile nu poate esua; daca
    // totusi ar esua, o linie goala e mai buna decat o panica in miezul
    // programului.
    serde_json::to_string(message).unwrap_or_default() + "\n"
}

/// Verifica salutul telefonului si pregateste raspunsul.
///
/// Separata de retea tocmai ca sa poata fi incercata fara niciun socket.
pub fn answer_hello(
    hello: &FromPhone,
    pc_name: &str,
    audio_port: u16,
) -> Result<(String, ToPhone), String> {
    let FromPhone::Hello { v, name, .. } = hello else {
        return Err("expected hello".into());
    };

    if *v != VERSION {
        return Err(format!(
            "unsupported protocol version {v}, this PC speaks {VERSION}"
        ));
    }

    let name = name.trim();
    let phone = if name.is_empty() {
        "Phone".to_string()
    } else {
        // Un nume lung ar strica asezarea ferestrei, iar unul cu randuri noi ar
        // putea strecura text pe unde nu trebuie.
        name.chars()
            .filter(|letter| !letter.is_control())
            .take(40)
            .collect()
    };

    Ok((
        phone,
        ToPhone::HelloAck {
            v: VERSION,
            name: pc_name.to_string(),
            port: audio_port,
            rate: crate::proto::SAMPLE_RATE,
            codec: "pcm".to_string(),
        },
    ))
}

/// Asculta legaturi de control pana cand `stop` devine adevarat.
///
/// Un singur telefon pe rand: asta e si realitatea produsului, si cea mai
/// simpla poveste de spus in interfata.
pub fn serve_control(port: u16, peer: Arc<Mutex<Peer>>, stop: Arc<AtomicBool>) -> io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    listener.set_nonblocking(true)?;
    println!("[control] ascult pe TCP {port}");

    let pc_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".to_string());

    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, address)) => {
                serve_one(stream, address, port, &pc_name, &peer, &stop);
                forget_peer(&peer);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL);
            }
            Err(error) => {
                eprintln!("[control] nu am putut primi legatura: {error}");
                thread::sleep(ACCEPT_POLL);
            }
        }
    }

    println!("[control] am oprit ascultarea pe TCP {port}");
    Ok(())
}

/// Duce o legatura de la salut pana la despartire.
fn serve_one(
    mut stream: TcpStream,
    address: SocketAddr,
    audio_port: u16,
    pc_name: &str,
    peer: &Arc<Mutex<Peer>>,
    stop: &Arc<AtomicBool>,
) {
    if stream.set_read_timeout(Some(READ_POLL)).is_err() {
        return;
    }
    let _ = stream.set_nodelay(true);

    let mut buffer = Vec::new();
    let mut greeted = false;
    let mut confusion = 0;
    let mut last_word = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        if last_word.elapsed() > SILENCE_LIMIT {
            eprintln!("[control] {address} a tacut prea mult, inchid");
            return;
        }

        let line = match poll_line(&mut stream, &mut buffer) {
            Incoming::Line(line) => line,
            Incoming::Idle => continue,
            Incoming::Closed => return,
        };
        if line.is_empty() {
            continue;
        }
        last_word = Instant::now();

        let message = match parse_line(&line) {
            Ok(message) => message,
            Err(reason) => {
                let _ = stream.write_all(encode(&ToPhone::Error { reason }).as_bytes());
                confusion += 1;
                if confusion >= MAX_CONFUSION {
                    eprintln!("[control] {address} nu vorbeste CoMic, inchid");
                    return;
                }
                continue;
            }
        };

        let reply = Reply {
            stream: &mut stream,
            greeted: &mut greeted,
            address,
            audio_port,
            pc_name,
            peer,
        };
        if !handle(&message, reply) {
            return;
        }
    }
}

/// Tot ce are nevoie un raspuns, strans la un loc ca sa nu avem o functie cu
/// sapte parametri.
struct Reply<'a> {
    stream: &'a mut TcpStream,
    greeted: &'a mut bool,
    address: SocketAddr,
    audio_port: u16,
    pc_name: &'a str,
    peer: &'a Arc<Mutex<Peer>>,
}

/// Raspunde la un mesaj. `false` inseamna "inchide legatura".
fn handle(message: &FromPhone, reply: Reply<'_>) -> bool {
    let Reply {
        stream,
        greeted,
        address,
        audio_port,
        pc_name,
        peer,
    } = reply;

    match message {
        FromPhone::Hello { .. } => match answer_hello(message, pc_name, audio_port) {
            Ok((phone, answer)) => {
                if stream.write_all(encode(&answer).as_bytes()).is_err() {
                    return false;
                }
                println!("[control] {phone} ({address}) s-a legat");
                update(peer, |peer| {
                    *peer = Peer {
                        connected: true,
                        name: phone.clone(),
                        address: address.ip().to_string(),
                        rtt_ms: None,
                        muted: false,
                    };
                });
                *greeted = true;
                true
            }
            Err(reason) => {
                eprintln!("[control] {address} respins: {reason}");
                let _ = stream.write_all(encode(&ToPhone::Error { reason }).as_bytes());
                false
            }
        },
        // Orice altceva inainte de salut inseamna un program strain pe portul
        // nostru. Nu ne certam cu el, doar inchidem.
        _ if !*greeted => false,
        FromPhone::Ping { seq, rtt } => {
            update(peer, |peer| peer.rtt_ms = *rtt);
            stream
                .write_all(encode(&ToPhone::Pong { seq: *seq }).as_bytes())
                .is_ok()
        }
        FromPhone::Mute { on } => {
            update(peer, |peer| peer.muted = *on);
            true
        }
        FromPhone::Bye => {
            println!("[control] {address} si-a luat ramas bun");
            false
        }
    }
}

fn update(peer: &Arc<Mutex<Peer>>, change: impl FnOnce(&mut Peer)) {
    if let Ok(mut peer) = peer.lock() {
        change(&mut peer);
    }
}

fn forget_peer(peer: &Arc<Mutex<Peer>>) {
    update(peer, |peer| *peer = Peer::default());
}

enum Incoming {
    Line(String),
    /// Nimic nou inca. Nu e o eroare: asa aflam ca putem verifica `stop`.
    Idle,
    Closed,
}

/// Scoate urmatoarea linie intreaga din flux.
///
/// Citim de mana, nu cu `BufRead::read_line`, fiindca termenul de asteptare ar
/// taia o linie sosita pe jumatate si am pierde-o. Asa, ce a sosit ramane in
/// tampon pana se completeaza.
fn poll_line(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Incoming {
    loop {
        if let Some(at) = buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=at).collect();
            return Incoming::Line(String::from_utf8_lossy(&line).trim().to_string());
        }
        if buffer.len() > MAX_LINE {
            return Incoming::Closed;
        }

        let mut chunk = [0u8; 512];
        match stream.read(&mut chunk) {
            Ok(0) => return Incoming::Closed,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                return Incoming::Idle
            }
            Err(_) => return Incoming::Closed,
        }
    }
}

/// Porneste canalul pe un fir separat. Daca portul e ocupat spunem si mergem
/// mai departe: sunetul trebuie sa curga si fara canal de control.
pub fn spawn_control(port: u16, peer: Arc<Mutex<Peer>>, stop: Arc<AtomicBool>) {
    thread::spawn(move || {
        if let Err(error) = serve_control(port, peer, stop) {
            eprintln!("[control] canalul nu a pornit pe TCP {port}: {error}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(version: u8, name: &str) -> FromPhone {
        FromPhone::Hello {
            v: version,
            name: name.to_string(),
            rate: 48_000,
            codec: "pcm".to_string(),
        }
    }

    #[test]
    fn reads_a_hello_from_the_phone() {
        let line = r#"{"t":"hello","v":1,"name":"Pixel 7","rate":48000,"codec":"pcm"}"#;

        assert_eq!(parse_line(line), Ok(hello(1, "Pixel 7")));
    }

    #[test]
    fn fills_in_the_fields_a_phone_left_out() {
        // Un telefon mai vechi, sau unul care trimite minimul.
        let parsed = parse_line(r#"{"t":"ping"}"#).expect("trebuie acceptat");

        assert_eq!(parsed, FromPhone::Ping { seq: 0, rtt: None });
    }

    #[test]
    fn refuses_a_line_that_is_not_our_protocol() {
        assert!(parse_line("GET / HTTP/1.1").is_err());
        assert!(parse_line(r#"{"t":"colinde"}"#).is_err());
    }

    #[test]
    fn greets_a_phone_that_speaks_our_version() {
        let (phone, reply) =
            answer_hello(&hello(VERSION, "Pixel 7"), "DESKTOP-VASIL", 45_678).expect("salut bun");

        assert_eq!(phone, "Pixel 7");
        assert_eq!(
            reply,
            ToPhone::HelloAck {
                v: VERSION,
                name: "DESKTOP-VASIL".to_string(),
                port: 45_678,
                rate: 48_000,
                codec: "pcm".to_string(),
            }
        );
    }

    #[test]
    fn turns_away_a_phone_from_another_version() {
        let refusal = answer_hello(&hello(9, "Pixel 7"), "PC", 45_678);

        assert!(refusal.is_err(), "o versiune straina nu are ce cauta");
        assert!(refusal.unwrap_err().contains('9'));
    }

    #[test]
    fn gives_a_nameless_phone_a_name() {
        let (phone, _) = answer_hello(&hello(VERSION, "   "), "PC", 45_678).expect("salut bun");

        assert_eq!(phone, "Phone");
    }

    #[test]
    fn trims_a_name_that_would_break_the_window() {
        let lung = "x".repeat(200);
        let (phone, _) = answer_hello(&hello(VERSION, &lung), "PC", 45_678).expect("salut bun");

        assert_eq!(phone.chars().count(), 40);
    }

    #[test]
    fn strips_control_characters_from_a_phone_name() {
        let (phone, _) =
            answer_hello(&hello(VERSION, "Pi\nxel\t7"), "PC", 45_678).expect("salut bun");

        assert_eq!(phone, "Pixel7");
    }

    #[test]
    fn every_answer_is_one_line() {
        let line = encode(&ToPhone::Pong { seq: 3 });

        assert!(line.ends_with('\n'), "mesajele se despart prin rand nou");
        assert_eq!(line.matches('\n').count(), 1);
        assert_eq!(line.trim(), r#"{"t":"pong","seq":3}"#);
    }

    #[test]
    fn a_fresh_peer_is_nobody() {
        let peer = Peer::default();

        assert!(!peer.connected);
        assert!(peer.name.is_empty());
        assert_eq!(peer.rtt_ms, None);
    }
}
