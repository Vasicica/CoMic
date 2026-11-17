// Porneste compilarea APK-ului pe serverele EAS.
//
// De ce exista ocolisul asta. eas-cli vede folderul android/ si incearca sa
// citeasca applicationId din android/app/build.gradle.kts cu expresia
//
//     /applicationId ['"](.*)['"]/
//
// adica spatiu, apoi ghilimea. Asa se scrie in Groovy. In Kotlin DSL semnul
// egal e obligatoriu, "applicationId = ...", deci expresia nu potriveste
// nimic si compilarea se opreste inainte sa inceapa, cu mesajul
//
//     Could not read applicationId from Android project.
//
// Variabila EAS_DANGEROUS_OVERRIDE_ANDROID_APPLICATION_ID sare peste citirea
// aceea; e prima verificare din getApplicationIdAsync, in
// eas-cli/build/project/android/applicationId.js. Numele ei e speriat pentru
// ca valoarea pusa acolo poate sa nu fie cea din codul nativ -- de-asta o
// citim chiar din Gradle, in loc s-o scriem de mana. Asa nu exista doua
// adevaruri care sa se desparta in timp.

import { spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const gradleFile = join(root, 'android', 'app', 'build.gradle.kts');

let gradle;
try {
  gradle = readFileSync(gradleFile, 'utf8');
} catch (error) {
  console.error(`Nu pot citi ${gradleFile}`);
  console.error(error.message);
  process.exit(1);
}

const found = gradle.match(/applicationId\s*=\s*["'](.+?)["']/);
if (!found) {
  console.error(`Nu gasesc applicationId in ${gradleFile}.`);
  console.error('Fara el, EAS nu stie cum se numeste aplicatia.');
  process.exit(1);
}

const applicationId = found[1];
console.log(`applicationId = ${applicationId}  (citit din build.gradle.kts)`);

// Argumentele in plus, daca dai vreunul (de pilda --clear-cache), ajung in
// comanda. Comanda trece printr-un shell, deci orice semn pe care shell-ul il
// interpreteaza ar fi o gaura; de-asta primim numai argumente simple si
// refuzam restul, in loc sa le punem in ghilimele si sa speram.
const ARGUMENT_SIMPLU = /^[-A-Za-z0-9=._:/]+$/;
const extra = process.argv.slice(2);
const stricat = extra.find((argument) => !ARGUMENT_SIMPLU.test(argument));

if (stricat) {
  console.error(`Argument pe care nu-l trimit mai departe: ${stricat}`);
  console.error('Primesc numai litere, cifre si - = . _ : /');
  console.error('Pentru orice altceva, cheama eas direct, cu');
  console.error('  set EAS_DANGEROUS_OVERRIDE_ANDROID_APPLICATION_ID=' + applicationId);
  process.exit(1);
}

// O singura comanda ca text, nu comanda plus lista de argumente: asa Node nu
// mai are ce sa concateneze nescapat, si avertismentul DEP0190 dispare.
// shell: true e necesar pe Windows, unde "eas" e un .cmd, iar Node refuza sa
// porneasca fisiere .cmd fara shell.
const comanda = ['eas build --platform android --profile apk', ...extra].join(' ');

const child = spawn(comanda, {
  stdio: 'inherit',
  shell: true,
  env: {
    ...process.env,
    EAS_DANGEROUS_OVERRIDE_ANDROID_APPLICATION_ID: applicationId,
  },
});

child.on('error', (error) => {
  console.error(`Nu pot porni eas: ${error.message}`);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  process.exit(signal ? 1 : (code ?? 1));
});
