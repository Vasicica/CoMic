//! Motorul audio, pe un fir propriu care primeste comenzi.
//!
//! De ce un fir separat si nu apeluri directe din interfata: pe Windows,
//! fluxurile audio ale cpal nu pot fi mutate intre fire. Trebuie create,
//! tinute si distruse in acelasi loc. Asa ca un singur fir le detine, iar
//! interfata ii trimite comenzi si citeste o stare publicata periodic.
//!
//! Avantajul colateral e ca interfata nu se blocheaza niciodata: pornirea unui
//! dispozitiv audio poate dura sute de milisecunde, iar fereastra ramane vie.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::audio::{self, Processing};
use crate::control::{self, Peer};
use crate::effects::{self, VoiceSettings};
use crate::firewall::{self, Advice};
use crate::jitter::JitterBuffer;
use crate::monitor::{self, MicQueue};
use crate::net;

/// Cat de des publicam starea cand nu vine nicio comanda.
const STATUS_INTERVAL: Duration = Duration::from_millis(80);

/// Dupa atata liniste intrebam Windows de ce nu soseste nimic.
///
/// Destul cat sa nu sarim la concluzii cat timp omul inca umbla pe telefon,
/// si destul de putin cat sa nu apuci sa te intrebi daca programul e stricat.
const SILENCE_BEFORE_ADVICE: Duration = Duration::from_secs(8);

/// De unde vine sunetul.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// De pe telefon, prin retea.
    Network,
    /// De la un microfon deja instalat in Windows (ascultare de proba).
    Microphone,
}

/// Tot ce trebuie ca sa pornim. Vine din interfata, intr-o singura bucata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    pub source: Source,
    pub output_device: Option<String>,
    pub input_device: Option<String>,
    pub port: u16,
    pub buffer_frames: usize,
    pub gain: f32,
    pub voice: String,
    pub settings: VoiceSettings,
}

/// Starea publicata catre interfata. Se citeste de zeci de ori pe secunda,
/// deci e mica si usor de copiat.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
    pub source: Option<Source>,
    pub output_device: String,
    pub input_device: String,
    pub voice: String,
    /// Nivelul sunetului, 0.0 - 1.0, pentru VU-metru.
    pub level: f32,
    pub buffered_ms: u32,
    pub frames_received: u64,
    pub frames_missing: u64,
    pub underruns: u64,
    /// Telefonul legat pe canalul de control. Pana la canal, fereastra ghicea
    /// din pachetele UDP ca "se intampla ceva"; acum stie al cui e telefonul,
    /// cat e dus-intorsul si cand a plecat.
    pub phone: Peer,
    /// De ce nu soseste nimic, cand am asteptat destul ca sa fie ceva de spus.
    /// Nu e o eroare: noi lucram cum trebuie, altceva ne sta in cale.
    pub advice: Option<Advice>,
    /// Sunetul iese pe difuzoare fiindca microfonul virtual nu e instalat.
    /// Totul merge, si tocmai de aceea e primejdios: omul se aude pe el si
    /// crede ca si ceilalti il aud. Vezi `audio::Playback`.
    pub virtual_mic_missing: bool,
    /// Ultima eroare, in cuvinte pe care le poate citi utilizatorul.
    pub error: Option<String>,
}

enum Command {
    Start(Box<StartRequest>),
    Stop,
    SetVoice {
        voice: String,
        settings: Box<VoiceSettings>,
    },
    SetGain(f32),
    Shutdown,
}

/// Ce tine firul cat timp sunetul curge.
struct Running {
    playback: audio::Playback,
    capture: Option<monitor::Capture>,
    jitter: Option<Arc<Mutex<JitterBuffer>>>,
    processing: Arc<Mutex<Processing>>,
    level: Arc<Mutex<f32>>,
    /// Starea telefonului, scrisa de firul canalului de control si citita aici.
    /// Lipseste pe ascultarea de proba: acolo nu exista telefon.
    peer: Option<Arc<Mutex<Peer>>>,
    /// Cand am pornit, ca sa stim de cat timp tace reteaua.
    started: Instant,
    /// De ce tace. Se cauta o singura data, pe firul ei, fiindca intrebarea
    /// catre Windows dureaza aproape o secunda si motorul nu are voie sa stea.
    advice: Arc<Mutex<Option<Advice>>>,
    advice_asked: AtomicBool,
    stop_network: Arc<AtomicBool>,
    network_thread: Option<JoinHandle<()>>,
}

impl Running {
    /// Opreste tot, in ordinea inversa pornirii.
    fn shut_down(mut self) {
        self.stop_network.store(true, Ordering::Relaxed);
        if let Some(handle) = self.network_thread.take() {
            let _ = handle.join();
        }
        drop(self.capture.take());
    }
}

pub struct Engine {
    commands: Sender<Command>,
    status: Arc<Mutex<Status>>,
    worker: Option<JoinHandle<()>>,
}

impl Engine {
    /// Porneste firul motorului. Nu porneste inca niciun sunet.
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::channel();
        let status = Arc::new(Mutex::new(Status::default()));
        let status_for_worker = Arc::clone(&status);

        let worker = thread::spawn(move || {
            let mut running: Option<Running> = None;

            loop {
                match receiver.recv_timeout(STATUS_INTERVAL) {
                    Ok(Command::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                        if let Some(active) = running.take() {
                            active.shut_down();
                        }
                        return;
                    }
                    Ok(command) => {
                        running = apply(command, running, &status_for_worker);
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                }

                publish(&running, &status_for_worker);
            }
        });

        Self {
            commands: sender,
            status,
            worker: Some(worker),
        }
    }

    pub fn status(&self) -> Status {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_default()
    }

    pub fn start(&self, request: StartRequest) {
        let _ = self.commands.send(Command::Start(Box::new(request)));
    }

    pub fn stop(&self) {
        let _ = self.commands.send(Command::Stop);
    }

    pub fn set_voice(&self, voice: String, settings: VoiceSettings) {
        let _ = self.commands.send(Command::SetVoice {
            voice,
            settings: Box::new(settings),
        });
    }

    pub fn set_gain(&self, gain: f32) {
        let _ = self.commands.send(Command::SetGain(gain));
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Executa o comanda si returneaza noua stare a motorului.
fn apply(
    command: Command,
    running: Option<Running>,
    status: &Arc<Mutex<Status>>,
) -> Option<Running> {
    match command {
        Command::Start(request) => {
            if let Some(active) = running {
                active.shut_down();
            }
            match start_running(&request) {
                Ok(active) => {
                    if let Ok(mut status) = status.lock() {
                        status.error = None;
                        status.source = Some(request.source);
                        status.voice = request.voice.clone();
                        status.output_device = active.playback.device_name.clone();
                        status.virtual_mic_missing = active.playback.virtual_mic_missing;
                        status.input_device = active
                            .capture
                            .as_ref()
                            .map(|capture| capture.device_name.clone())
                            .unwrap_or_default();
                    }
                    Some(active)
                }
                Err(message) => {
                    if let Ok(mut status) = status.lock() {
                        status.error = Some(message);
                        status.source = None;
                    }
                    None
                }
            }
        }
        Command::Stop => {
            if let Some(active) = running {
                active.shut_down();
            }
            if let Ok(mut status) = status.lock() {
                status.source = None;
                status.error = None;
            }
            None
        }
        Command::SetVoice { voice, settings } => {
            if let Some(active) = running.as_ref() {
                if let Ok(mut processing) = active.processing.lock() {
                    processing.voice = build_voice(&voice, *settings);
                }
            }
            if let Ok(mut status) = status.lock() {
                status.voice = voice;
            }
            running
        }
        Command::SetGain(gain) => {
            if let Some(active) = running.as_ref() {
                if let Ok(mut processing) = active.processing.lock() {
                    processing.gain = gain;
                }
            }
            running
        }
        Command::Shutdown => None,
    }
}

/// Construieste lantul de efecte. O voce necunoscuta nu este o eroare fatala:
/// inseamna doar voce naturala, ca utilizatorul sa nu ramana fara sunet.
fn build_voice(voice: &str, settings: VoiceSettings) -> Option<effects::VoiceChain> {
    effects::find_preset(voice).map(|preset| effects::VoiceChain::new(preset, settings))
}

fn start_running(request: &StartRequest) -> Result<Running, String> {
    let processing = Arc::new(Mutex::new(Processing::new(
        build_voice(&request.voice, request.settings),
        request.gain,
    )));
    let level = Arc::new(Mutex::new(0.0_f32));
    let stop_network = Arc::new(AtomicBool::new(false));

    let (playback, capture, jitter, peer, network_thread) = match request.source {
        Source::Microphone => {
            let queue = Arc::new(Mutex::new(MicQueue::new()));
            let capture =
                monitor::start_capture(request.input_device.as_deref(), Arc::clone(&queue))
                    .map_err(|error| error.to_string())?;

            let queue_for_fill = Arc::clone(&queue);
            let level_for_fill = Arc::clone(&level);
            let playback = audio::start(
                request.output_device.as_deref(),
                Arc::clone(&processing),
                move |mono| {
                    match queue_for_fill.lock() {
                        Ok(mut queue) => {
                            queue.fill(mono);
                        }
                        Err(_) => mono.fill(0),
                    }
                    publish_level(&level_for_fill, mono);
                },
            )
            .map_err(|error| error.to_string())?;

            (playback, Some(capture), None, None, None)
        }
        Source::Network => {
            let buffer = Arc::new(Mutex::new(JitterBuffer::with_prefill(
                request.buffer_frames,
            )));

            let buffer_for_fill = Arc::clone(&buffer);
            let level_for_fill = Arc::clone(&level);
            let playback = audio::start(
                request.output_device.as_deref(),
                Arc::clone(&processing),
                move |mono| {
                    match buffer_for_fill.lock() {
                        Ok(mut buffer) => {
                            buffer.fill(mono);
                        }
                        Err(_) => mono.fill(0),
                    }
                    publish_level(&level_for_fill, mono);
                },
            )
            .map_err(|error| error.to_string())?;

            let port = request.port;
            let buffer_for_net = Arc::clone(&buffer);
            let stop_for_net = Arc::clone(&stop_network);
            net::spawn_discovery(port, Arc::clone(&stop_network));

            // Canalul de control asculta pe TCP, pe acelasi numar de port pe
            // care audio asculta pe UDP. Un singur numar de tinut minte, si o
            // singura poveste cand trebuie deschis in Firewall.
            let peer = Arc::new(Mutex::new(Peer::default()));
            control::spawn_control(port, Arc::clone(&peer), Arc::clone(&stop_network));

            let thread = thread::spawn(move || {
                if let Err(error) = net::receive_audio(port, buffer_for_net, stop_for_net) {
                    eprintln!("[motor] ascultarea pe {port} s-a oprit: {error}");
                }
            });

            (playback, None, Some(buffer), Some(peer), Some(thread))
        }
    };

    Ok(Running {
        playback,
        capture,
        jitter,
        processing,
        level,
        peer,
        started: Instant::now(),
        advice: Arc::new(Mutex::new(None)),
        advice_asked: AtomicBool::new(false),
        stop_network,
        network_thread,
    })
}

/// Nivelul folosit de VU-metru: valoarea eficace (RMS) a cadrului curent.
/// RMS, nu varful, pentru ca asa se misca la fel ca taria perceputa de ureche.
fn publish_level(level: &Arc<Mutex<f32>>, samples: &[i16]) {
    if samples.is_empty() {
        return;
    }

    let sum: f64 = samples
        .iter()
        .map(|sample| {
            let normalised = *sample as f64 / i16::MAX as f64;
            normalised * normalised
        })
        .sum();
    let rms = (sum / samples.len() as f64).sqrt() as f32;

    if let Ok(mut level) = level.lock() {
        // Urca instant si coboara lin: altfel VU-metrul palpaie neplacut.
        *level = if rms > *level {
            rms
        } else {
            *level * 0.8 + rms * 0.2
        };
    }
}

fn publish(running: &Option<Running>, status: &Arc<Mutex<Status>>) {
    let Ok(mut status) = status.lock() else {
        return;
    };

    let Some(active) = running else {
        status.running = false;
        status.level = 0.0;
        status.buffered_ms = 0;
        status.phone = Peer::default();
        status.advice = None;
        // Campul vorbeste despre sunetul care curge acum. Cand nu curge nimic,
        // nu e nimic de spus aici; ca VB-Cable lipseste de tot afla fereastra
        // separat, inca de la pornire.
        status.virtual_mic_missing = false;
        return;
    };

    status.running = true;
    status.level = active.level.lock().map(|level| *level).unwrap_or(0.0);
    status.phone = match active.peer.as_ref() {
        Some(peer) => peer.lock().map(|peer| peer.clone()).unwrap_or_default(),
        // Ascultarea de proba merge de la microfonul calculatorului: nu exista
        // niciun telefon de aratat, si nu inventam unul.
        None => Peer::default(),
    };

    if let Some(jitter) = active.jitter.as_ref() {
        if let Ok(jitter) = jitter.lock() {
            status.buffered_ms = jitter.buffered_ms();
            status.frames_received = jitter.stats.frames_received;
            status.frames_missing = jitter.stats.frames_missing;
            status.underruns = jitter.stats.underruns;
        }
    }

    ask_why_nothing_arrives(active, &status.phone, status.frames_received);
    status.advice = active.advice.lock().ok().and_then(|advice| advice.clone());
}

/// Cand ascultam reteaua si nu soseste nimic, aflam de ce in loc sa tacem.
///
/// Conditia e inadins severa: niciun cadru de sunet **si** niciun telefon pe
/// canalul de control. Daca macar unul din ele a ajuns, drumul e deschis si
/// Firewall-ul nu e de vina, deci n-avem ce sfat sa dam.
fn ask_why_nothing_arrives(active: &Running, phone: &Peer, frames: u64) {
    // `jitter` exista numai cand ascultam reteaua. La ascultarea de proba, de
    // la microfonul calculatorului, nu e nicio retea de invinuit.
    let pe_retea = active.jitter.is_some();
    let liniste = frames == 0 && !phone.connected;

    if !pe_retea || !liniste || active.started.elapsed() < SILENCE_BEFORE_ADVICE {
        return;
    }
    // O singura intrebare pe sesiune: raspunsul nu se schimba de la sine, iar
    // `publish` trece pe aici de douasprezece ori pe secunda.
    if active.advice_asked.swap(true, Ordering::Relaxed) {
        return;
    }

    let locul = Arc::clone(&active.advice);
    thread::spawn(move || {
        let Ok((networks, rules)) = firewall::ask_windows() else {
            return;
        };
        if let Ok(mut locul) = locul.lock() {
            *locul = firewall::diagnose(&networks, &rules).advice();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_silent_frame_reports_no_level() {
        // Arrange
        let level = Arc::new(Mutex::new(0.0));

        // Act
        publish_level(&level, &[0; 480]);

        // Assert
        assert_eq!(*level.lock().unwrap(), 0.0);
    }

    #[test]
    fn a_full_scale_frame_reports_a_level_near_one() {
        // Arrange
        let level = Arc::new(Mutex::new(0.0));

        // Act
        publish_level(&level, &[i16::MAX; 480]);

        // Assert
        let measured = *level.lock().unwrap();
        assert!(
            measured > 0.95,
            "asteptam aproape 1.0, am primit {measured}"
        );
    }

    #[test]
    fn the_level_rises_at_once_but_falls_gradually() {
        // Arrange
        let level = Arc::new(Mutex::new(0.0));
        publish_level(&level, &[i16::MAX; 480]);
        let loud = *level.lock().unwrap();

        // Act: liniste imediat dupa
        publish_level(&level, &[0; 480]);
        let after_silence = *level.lock().unwrap();

        // Assert
        assert!(after_silence < loud, "nivelul trebuie sa coboare");
        assert!(
            after_silence > 0.0,
            "dar nu dintr-o data, altfel VU-metrul palpaie"
        );
    }

    #[test]
    fn an_unknown_voice_falls_back_to_the_natural_one() {
        assert!(build_voice("vocea-lui-bunicu", VoiceSettings::default()).is_none());
        assert!(build_voice("pirat", VoiceSettings::default()).is_some());
    }
}
