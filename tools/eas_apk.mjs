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

// shell: true e necesar pe Windows, unde "eas" e un .cmd si Node refuza sa
// porneasca fisiere .cmd fara shell.
const child = spawn(
  'eas',
  ['build', '--platform', 'android', '--profile', 'apk', ...process.argv.slice(2)],
  {
    stdio: 'inherit',
    shell: true,
    env: {
      ...process.env,
      EAS_DANGEROUS_OVERRIDE_ANDROID_APPLICATION_ID: applicationId,
    },
  },
);

child.on('error', (error) => {
  console.error(`Nu pot porni eas: ${error.message}`);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  process.exit(signal ? 1 : (code ?? 1));
});
