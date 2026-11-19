//! Preferintele care trebuie sa supravietuiasca inchiderii ferestrei.
//!
//! Un fisier JSON in folderul de configurare al aplicatiei. Nu registrul si nu
//! o baza de date: trei valori logice nu merita mai mult, si un fisier text se
//! poate citi cu ochiul atunci cand ceva nu se poarta cum trebuie.
//!
//! Regula fisierului: **daca nu se poate citi, programul nu se opreste.** Un
//! JSON stricat, un folder fara drept de scriere sau un fisier inexistent dau
//! valorile implicite. O fereastra care refuza sa porneasca din cauza unei
//! preferinte ar fi mult mai rea decat o preferinta uitata.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Ce ne amintim intre doua porniri.
///
/// `serde(default)` nu e un detaliu: face ca un fisier scris de o versiune mai
/// veche, cu mai putine campuri, sa se citeasca in continuare. Campul nou ia
/// valoarea implicita in loc sa dea toata citirea peste cap.
///
/// Si valoarea implicita e aleasa, nu intamplatoare: toate trei oprite. Un
/// program care se scrie singur in pornirea automata a Windows-ului, sau care
/// deschide ascultarea fara sa fie rugat, e un program obraznic. Fiecare din
/// cele trei se aprinde numai cu mana.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    /// Scrie CoMic in lista de pornire a Windows-ului.
    pub launch_at_startup: bool,
    /// Butonul X ascunde fereastra in bara de sistem in loc sa inchida programul.
    pub minimize_to_tray: bool,
    /// Porneste ascultarea imediat ce fereastra se deschide.
    pub listen_on_launch: bool,
    /// Ce a spus ultimul telefon despre Pro.
    ///
    /// Cumparatura se face pe telefon, prin Google Play; calculatorul nu are
    /// plata proprie. Dar daca n-ar tine minte, vocile s-ar incuia de fiecare
    /// data cand telefonul nu e legat, inclusiv la ascultarea de proba cu
    /// microfonul calculatorului. Deci: telefonul spune, calculatorul retine.
    pub pro: bool,
}

/// Preferintele incarcate in memorie, plus locul de pe disc unde se scriu.
pub struct PrefsStore {
    path: PathBuf,
    current: Mutex<Prefs>,
}

impl PrefsStore {
    /// Citeste fisierul de la calea data. Orice problema da valorile implicite.
    pub fn load(path: PathBuf) -> Self {
        let current = read(&path).unwrap_or_default();
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    /// Unde sta fisierul. Folosita in mesajele de eroare, ca sa fie limpede ce
    /// cale a dat gres.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get(&self) -> Prefs {
        *self.guard()
    }

    /// Inlocuieste preferintele cu altele si le scrie pe disc.
    ///
    /// Memoria se schimba chiar daca scrierea cade: altfel comutatorul ar
    /// reveni la vechea valoare in aceeasi sesiune in care l-ai apasat. Eroarea
    /// pleaca mai departe, ca sa poata fi spusa.
    pub fn set(&self, next: Prefs) -> io::Result<()> {
        *self.guard() = next;
        write(&self.path, &next)
    }

    /// Un `Mutex` otravit inseamna ca un alt fir a crapat tinandu-l. Datele de
    /// aici sunt trei valori logice copiabile, deci nu pot fi pe jumatate
    /// scrise: luam ce e inauntru si mergem inainte, in loc sa crapam si noi.
    fn guard(&self) -> std::sync::MutexGuard<'_, Prefs> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn read(path: &Path) -> Option<Prefs> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write(path: &Path, prefs: &Prefs) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(prefs)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Fiecare test isi ia propriul folder: testele ruleaza in acelasi proces,
    /// pe fire diferite, si un nume comun i-ar face sa se calce.
    fn cale_de_test(nume: &str) -> PathBuf {
        static CONTOR: AtomicU32 = AtomicU32::new(0);
        let unic = CONTOR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!(
                "comic-prefs-{}-{}-{nume}",
                std::process::id(),
                unic
            ))
            .join("prefs.json")
    }

    #[test]
    fn starts_with_everything_off_when_there_is_no_file() {
        let store = PrefsStore::load(cale_de_test("lipsa"));

        assert_eq!(store.get(), Prefs::default());
        assert!(!store.get().launch_at_startup);
        assert!(!store.get().minimize_to_tray);
        assert!(!store.get().listen_on_launch);
    }

    #[test]
    fn remembers_what_was_written_after_a_fresh_load() {
        let path = cale_de_test("dus-intors");
        let dorit = Prefs {
            launch_at_startup: true,
            minimize_to_tray: false,
            listen_on_launch: true,
            pro: true,
        };

        PrefsStore::load(path.clone()).set(dorit).unwrap();
        let reincarcat = PrefsStore::load(path);

        assert_eq!(reincarcat.get(), dorit);
    }

    #[test]
    fn remembers_what_the_phone_said_about_pro() {
        let path = cale_de_test("pro");

        PrefsStore::load(path.clone())
            .set(Prefs {
                pro: true,
                ..Prefs::default()
            })
            .unwrap();

        assert!(PrefsStore::load(path).get().pro);
    }

    #[test]
    fn an_older_file_does_not_hand_out_pro() {
        // Fisier scris inainte sa existe campul. Lipsa lui inseamna nu.
        let path = cale_de_test("fara-pro");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"minimizeToTray":true}"#).unwrap();

        let store = PrefsStore::load(path);

        assert!(store.get().minimize_to_tray);
        assert!(!store.get().pro, "nu dam Pro pe gratis unui fisier vechi");
    }

    #[test]
    fn fills_in_the_missing_fields_of_an_older_file() {
        // Asa arata fisierul scris de o versiune care nu cunostea decat un
        // singur comutator. Nu are voie sa strice citirea.
        let path = cale_de_test("vechi");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"launchAtStartup":true}"#).unwrap();

        let store = PrefsStore::load(path);

        assert!(store.get().launch_at_startup);
        assert!(!store.get().minimize_to_tray);
        assert!(!store.get().listen_on_launch);
    }

    #[test]
    fn falls_back_to_the_defaults_when_the_file_is_corrupt() {
        let path = cale_de_test("stricat");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{ asta nu e JSON").unwrap();

        assert_eq!(PrefsStore::load(path).get(), Prefs::default());
    }

    #[test]
    fn creates_the_folder_when_it_does_not_exist_yet() {
        // La prima pornire, folderul de configurare al aplicatiei nu exista.
        let path = cale_de_test("folder-nou");
        assert!(!path.parent().unwrap().exists());

        PrefsStore::load(path.clone())
            .set(Prefs {
                minimize_to_tray: true,
                ..Prefs::default()
            })
            .unwrap();

        assert!(path.exists());
    }
}
