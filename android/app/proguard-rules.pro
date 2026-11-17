# Nu avem nevoie de reguli proprii: aplicatia nu foloseste reflectie si nu
# serializeaza clase cu nume care trebuie pastrate. Fisierul exista pentru ca
# build.gradle.kts il refera la varianta release.
#
# Daca se adauga vreodata serializare (de exemplu kotlinx.serialization pentru
# canalul de control), aici vin regulile de pastrare a numelor.
