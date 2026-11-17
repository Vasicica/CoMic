# Voice changer — functia care aduce bani

Ideea utilizatorului: doua voci (pirat, mosneag), gratuite doar ca **test** —
te auzi pe tine cum ai suna. Ca sa le folosesti cu adevarat in Zoom, CS2 sau
Discord, platesti.

Rationamentul comercial este corect: microfonul este o unealta, iar uneltele
se cumpara greu. Vocile sunt distractie, iar distractia se cumpara usor —
mai ales de adolescenti care se glumesc in jocuri.

## Cum se imparte gratuit / platit

| Functie | Gratuit | Platit |
|---|---|---|
| Microfon, fara limita de timp | da | — |
| Toate metodele de conectare | da | — |
| Ascultare de proba a vocilor (te auzi in casti) | da | — |
| Vocile trimise mai departe in microfonul virtual | nu | da |
| Voci suplimentare peste cele doua | nu | da |

Diferenta fata de WO Mic este chiar aici: **la noi microfonul este gratuit si
nelimitat**. Se plateste doar distractia. Asta e si argumentul de marketing,
si raspunsul la ce l-a enervat pe utilizator la WO Mic.

## Cum se face tehnic

Procesarea sta pe calculator, in Rust, nu pe telefon. Trei motive: calculatorul
are putere de calcul, aplicatia de telefon ramane simpla, iar vocile noi se
livreaza printr-o actualizare de program, nu prin magazin.

Lantul devine:

    retea -> buffer de jitter -> [efecte] -> microfon virtual
                                    |
                                    +------> casti (ascultare de proba)

Nucleul: deplasare de inaltime (pitch shift) si de formanti, independent una de
alta. Formantii sunt cei care fac diferenta dintre "aceeasi voce accelerata" si
"alta persoana" — fara ei iese sunet de desene animate.

**Pirat**: inaltime cu 4-5 semitonuri in jos, formanti in jos, putina distorsiune
si o urma de reverberatie de incapere.
**Mosneag**: inaltime putin in jos, formanti in sus, tremolo lent si putin
zgomot de respiratie — batranetea se aude ca instabilitate, nu ca inaltime.

Cost in latenta: 20-30 ms peste lantul existent. Acceptabil in jocuri si
meeting-uri. Se poate opri complet cand nu e folosit.

## De discutat inainte de Faza 3

- **Abonament sau plata unica.** 2.50 EUR/luna este plauzibil, dar o unealta se
  vinde mai bine la plata unica — iar utilizatorul insusi a plecat de la WO Mic
  din cauza abonamentului. Varianta mixta: aplicatia gratuita pentru totdeauna,
  pachetele de voci la plata unica mica.
- **Public tinta.** Daca produsul este prezentat ca fiind pentru copii, intra sub
  politica Google Play pentru familii si sub reguli europene stricte de abonament
  (anulare simpla, drept de retragere 14 zile). Se promoveaza catre "jucatori",
  nu catre copii.
- **Folosire abuziva.** Un voice changer poate fi folosit pentru a imita pe
  cineva. Conditiile de utilizare trebuie sa interzica explicit impersonarea.
