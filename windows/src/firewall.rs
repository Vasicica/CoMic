//! De ce nu soseste nimic: intrebam Windows despre propriul lui Firewall.
//!
//! Ascultam pe portul bun, telefonul trimite, si totusi liniste. Pana acum
//! programul tacea si el, iar tu ramaneai sa ghicesti intre adresa gresita,
//! Firewall, ruter si dispozitivul de iesire.
//!
//! Doua cauze adevarate, amandoua traite:
//!
//! 1. **Nicio regula.** Windows a intrebat la prima pornire daca CoMic poate
//!    primi legaturi, si raspunsul a fost nu. Nu mai intreaba a doua oara.
//! 2. **Regula pe profilul gresit.** Asta e cea perfida, si cea pe care planul
//!    initial nu o prevedea. Regula exista si zice "permite", dar e scrisa
//!    pentru retelele *private*. Cand pornesti partajarea prin USB, Windows
//!    vede o retea **noua** si o pune implicit pe *public*. Nu te intreaba
//!    nimic, fiindca are deja o regula. Pur si simplu arunca tot pe acel drum.
//!    Simptomul: merge prin aer, tace prin fir.
//!
//! **De ce PowerShell si nu COM.** Raspunsul il cerem in JSON (`ConvertTo-Json`),
//! deci citim nume de campuri in engleza, nu text tradus. Masina pe care s-a
//! scris asta raspunde in rusa; o citire de text s-ar fi rupt acolo. Interogam
//! numai, nu schimbam nimic, si nu cerem drepturi de administrator.

use std::io;

/// O retea la care calculatorul e legat acum.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Network {
    /// Numele pe care il vezi in Setari, de exemplu "Retea 3".
    pub name: String,
    /// Placa de retea: "Ethernet 3", "Wi-Fi".
    pub interface: String,
    /// "Private", "Public" sau "DomainAuthenticated".
    pub category: String,
}

/// O regula de Firewall care priveste programul nostru.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Rule {
    pub name: String,
    /// Profilele pe care se aplica: "Private", "Public, Private", "Any".
    pub profiles: String,
}

/// Ce am aflat, in termeni pe care ii poate folosi interfata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnosis {
    /// Nicio regula care sa ne lase sa primim legaturi.
    NoRule,
    /// Exista reguli, dar nu acopera retelele astea.
    Uncovered(Vec<Network>),
    /// Firewall-ul pare in regula; cauza e in alta parte.
    LooksFine,
}

/// Sfatul, in bucati, ca interfata sa-l poata spune in limba ta.
///
/// Trimitem bucati si nu o propozitie gata facuta pentru ca fereastra vorbeste
/// zece limbi, iar motorul nu stie pe care o foloseste acum. Numele retelelor
/// si al placilor nu se traduc: asa le vezi si in Setarile Windows.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    /// "noRule" sau "blocked". Interfata alege propozitia dupa asta.
    pub kind: &'static str,
    /// Numele retelelor, asa cum le arata Windows: "Retea 3".
    pub networks: Vec<String>,
    /// Placile de retea, pentru comanda de reparare: "Ethernet 3".
    pub interfaces: Vec<String>,
}

impl Diagnosis {
    /// Ce are de spus interfetei. `None` cand nu e nimic de spus.
    pub fn advice(&self) -> Option<Advice> {
        match self {
            Diagnosis::LooksFine => None,
            Diagnosis::NoRule => Some(Advice {
                kind: "noRule",
                networks: Vec::new(),
                interfaces: Vec::new(),
            }),
            Diagnosis::Uncovered(networks) => Some(Advice {
                kind: "blocked",
                networks: networks.iter().map(|n| n.name.clone()).collect(),
                interfaces: networks.iter().map(|n| n.interface.clone()).collect(),
            }),
        }
    }
}

/// Potriveste retelele active cu profilele pe care le acopera regulile.
///
/// Toata socoteala e aici, despartita de retea, ca sa poata fi incercata cu
/// situatii pe care nu le putem crea la comanda pe o masina adevarata.
pub fn diagnose(networks: &[Network], rules: &[Rule]) -> Diagnosis {
    if rules.is_empty() {
        return Diagnosis::NoRule;
    }

    let descoperite: Vec<Network> = networks
        .iter()
        .filter(|network| !covered(&network.category, rules))
        .cloned()
        .collect();

    if descoperite.is_empty() {
        Diagnosis::LooksFine
    } else {
        Diagnosis::Uncovered(descoperite)
    }
}

/// Acopera vreo regula profilul retelei asteia?
///
/// "Any" acopera tot. Altfel se potrivesc pe nume, fara sa conteze litera mare
/// sau mica, cautand in lista despartita prin virgule pe care o da Windows.
/// `DomainAuthenticated` din partea retelei se numeste `Domain` in reguli:
/// acelasi lucru, doua nume, si nepotrivirea lor ar da un raspuns fals.
fn covered(category: &str, rules: &[Rule]) -> bool {
    let category = category.trim();
    let cautat = if category.eq_ignore_ascii_case("DomainAuthenticated") {
        "domain".to_string()
    } else {
        category.to_ascii_lowercase()
    };

    rules.iter().any(|rule| {
        rule.profiles.split(',').any(|profile| {
            let profile = profile.trim();
            profile.eq_ignore_ascii_case("Any") || profile.eq_ignore_ascii_case(&cautat)
        })
    })
}

/// Ce intreaba programul nostru Windows-ul.
///
/// Doua interogari, amandoua numai de citire. A doua porneste de la filtrele
/// de aplicatie, nu de la reguli: regulile sunt cu miile pe un Windows obisnuit
/// si le-am intreba pe fiecare in parte, in timp ce filtrele le putem cere pe
/// toate deodata si pastra doar ce ne priveste.
const QUERY: &str = r#"
# Fara randul asta, PowerShell scrie la iesire in codul de pagina al consolei,
# nu in UTF-8, si orice nume de retea cu litere nelatine ne ajunge stricat.
# Masina pe care s-a scris asta are retele numite cu chirilice, si prima
# incercare a intors patru semne de intrebare in loc de "Retea".
[Console]::OutputEncoding = [Text.Encoding]::UTF8

$retele = Get-NetConnectionProfile | ForEach-Object {
  [pscustomobject]@{
    name      = [string]$_.Name
    interface = [string]$_.InterfaceAlias
    category  = [string]$_.NetworkCategory
  }
}
$reguli = Get-NetFirewallApplicationFilter |
  Where-Object { $_.Program -like '*comic*' } |
  Get-NetFirewallRule |
  Where-Object { $_.Direction -eq 'Inbound' -and $_.Enabled -eq 'True' -and $_.Action -eq 'Allow' } |
  ForEach-Object {
    [pscustomobject]@{
      name     = [string]$_.DisplayName
      profiles = [string]$_.Profile
    }
  }
[pscustomobject]@{
  networks = @($retele)
  rules    = @($reguli)
} | ConvertTo-Json -Depth 4 -Compress
"#;

#[derive(Debug, serde::Deserialize)]
struct Answer {
    #[serde(default)]
    networks: Vec<Network>,
    #[serde(default)]
    rules: Vec<Rule>,
}

/// Intreaba Windows si intoarce ce a raspuns.
#[cfg(windows)]
pub fn ask_windows() -> io::Result<(Vec<Network>, Vec<Rule>)> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    /// Fara asta, o fereastra neagra ar clipi peste interfata la fiecare
    /// intrebare. CREATE_NO_WINDOW, din documentatia Windows.
    const FARA_FEREASTRA: u32 = 0x0800_0000;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", QUERY])
        .creation_flags(FARA_FEREASTRA)
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let answer: Answer = serde_json::from_str(text.trim())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok((answer.networks, answer.rules))
}

/// Pe alte sisteme nu exista Firewall-ul Windows, deci nu avem ce intreba.
/// Exista ca sa se poata compila si rula testele si pe Linux, unde s-a lucrat
/// o parte din proiect.
#[cfg(not(windows))]
pub fn ask_windows() -> io::Result<(Vec<Network>, Vec<Rule>)> {
    Ok((Vec::new(), Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retea(name: &str, interface: &str, category: &str) -> Network {
        Network {
            name: name.to_string(),
            interface: interface.to_string(),
            category: category.to_string(),
        }
    }

    fn regula(profiles: &str) -> Rule {
        Rule {
            name: "CoMic".to_string(),
            profiles: profiles.to_string(),
        }
    }

    #[test]
    fn says_there_is_no_rule_when_windows_knows_none() {
        let verdict = diagnose(&[retea("Retea", "Ethernet", "Private")], &[]);

        assert_eq!(verdict, Diagnosis::NoRule);
    }

    #[test]
    fn finds_the_network_a_rule_does_not_cover() {
        // Exact situatia din 19 noiembrie 2026: partajarea prin USB a adus o
        // retea noua, Windows a pus-o pe public, iar regula era doar privata.
        let retele = [
            retea("Retea", "Ethernet", "Private"),
            retea("Retea 3", "Ethernet 3", "Public"),
        ];

        let verdict = diagnose(&retele, &[regula("Private")]);

        assert_eq!(verdict, Diagnosis::Uncovered(vec![retele[1].clone()]));
    }

    #[test]
    fn is_content_when_every_network_is_covered() {
        let retele = [
            retea("Retea", "Ethernet", "Private"),
            retea("Retea 3", "Ethernet 3", "Public"),
        ];

        let verdict = diagnose(&retele, &[regula("Private"), regula("Public")]);

        assert_eq!(verdict, Diagnosis::LooksFine);
    }

    #[test]
    fn treats_any_as_covering_everything() {
        // Asa e scrisa regula lui WO Mic pe masina de proba, si de-aceea lui
        // ii mergea prin cablu cand noua nu.
        let retele = [retea("Retea 3", "Ethernet 3", "Public")];

        assert_eq!(diagnose(&retele, &[regula("Any")]), Diagnosis::LooksFine);
    }

    #[test]
    fn reads_the_comma_separated_list_windows_returns() {
        let retele = [retea("Retea 3", "Ethernet 3", "Public")];

        let verdict = diagnose(&retele, &[regula("Domain, Private, Public")]);

        assert_eq!(verdict, Diagnosis::LooksFine);
    }

    #[test]
    fn knows_domain_and_domainauthenticated_are_the_same_thing() {
        // Reteaua se numeste DomainAuthenticated, regula se numeste Domain.
        // Daca nu le impacam, am striga "neacoperit" pe un calculator de birou
        // unde totul e in regula.
        let retele = [retea("Birou", "Ethernet", "DomainAuthenticated")];

        assert_eq!(diagnose(&retele, &[regula("Domain")]), Diagnosis::LooksFine);
    }

    #[test]
    fn names_every_uncovered_network_not_just_the_first() {
        let retele = [
            retea("Cablu", "Ethernet 3", "Public"),
            retea("Cafenea", "Wi-Fi", "Public"),
        ];

        let verdict = diagnose(&retele, &[regula("Private")]);

        assert_eq!(verdict, Diagnosis::Uncovered(retele.to_vec()));
    }

    #[test]
    fn reads_the_answer_windows_sends_back() {
        // Forma exacta pe care o scoate interogarea din QUERY.
        let json = r#"{"networks":[{"name":"Retea 3","interface":"Ethernet 3",
            "category":"Public"}],"rules":[{"name":"CoMic","profiles":"Private"}]}"#;

        let answer: Answer = serde_json::from_str(json).expect("raspuns valid");

        assert_eq!(answer.networks[0].interface, "Ethernet 3");
        assert_eq!(answer.rules[0].profiles, "Private");
    }

    #[test]
    fn has_nothing_to_say_when_the_firewall_is_fine() {
        assert_eq!(Diagnosis::LooksFine.advice(), None);
    }

    #[test]
    fn hands_the_interface_the_names_it_needs_to_speak() {
        let verdict = Diagnosis::Uncovered(vec![retea("Retea 3", "Ethernet 3", "Public")]);

        let advice = verdict.advice().expect("are ceva de spus");

        assert_eq!(advice.kind, "blocked");
        // Numele pentru propozitie, placa pentru comanda de reparare.
        assert_eq!(advice.networks, vec!["Retea 3"]);
        assert_eq!(advice.interfaces, vec!["Ethernet 3"]);
    }

    #[test]
    fn survives_a_windows_with_no_rules_and_no_networks() {
        // ConvertTo-Json scoate liste goale; nu avem voie sa crapam pe ele.
        let answer: Answer =
            serde_json::from_str(r#"{"networks":[],"rules":[]}"#).expect("raspuns valid");

        assert_eq!(diagnose(&answer.networks, &answer.rules), Diagnosis::NoRule);
    }
}
