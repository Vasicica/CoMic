//! Nucleul CoMic: captare, retea, efecte si redare.
//!
//! Aceleasi module servesc si interfata grafica (`comic-app`), si linia de
//! comanda (`comic-receiver`). Logica audio nu stie nimic despre interfata.

pub mod audio;
pub mod control;
pub mod effects;
pub mod engine;
pub mod jitter;
pub mod monitor;
pub mod net;
pub mod proto;
