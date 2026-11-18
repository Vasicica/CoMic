// Traducerile CoMic: sursa unica e shared/i18n/<limba>.json.
// Scriptul asta le copiaza acolo unde le citesc aplicatiile:
//
//   windows/app/ui/i18n.js                       fereastra de PC (Tauri)
//   android/app/src/main/res/values*/strings.xml aplicatia de telefon
//
// Rulare, din radacina proiectului:  node tools/i18n_build.mjs
// Dupa orice schimbare intr-un .json, rulezi din nou. Nu edita fisierele
// generate de mana: se suprascriu.

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const srcDir = join(root, 'shared', 'i18n');
const SOURCE = 'en';

const langs = readdirSync(srcDir)
  .filter((f) => f.endsWith('.json'))
  .map((f) => f.replace('.json', ''))
  .sort((a, b) => (a === SOURCE ? -1 : b === SOURCE ? 1 : a.localeCompare(b)));

const load = (lang) => JSON.parse(readFileSync(join(srcDir, lang + '.json'), 'utf8'));
const en = load(SOURCE);

// ---- verificari: o traducere incompleta nu ajunge in aplicatie ----
const placeholders = (s) => (s.match(/\{\w+\}/g) || []).sort().join(',');
const problems = [];
const all = {};
for (const lang of langs) {
  const d = load(lang);
  for (const key of Object.keys(en)) {
    if (!(key in d)) problems.push(`${lang}: lipseste "${key}"`);
    else if (placeholders(d[key]) !== placeholders(en[key])) problems.push(`${lang}: "${key}" are alte {..} decat engleza`);
    else if (/[\u2014\u2013]|--/.test(d[key])) problems.push(`${lang}: "${key}" are liniuta lunga`);
  }
  for (const key of Object.keys(d)) if (!(key in en)) problems.push(`${lang}: cheie in plus "${key}"`);
  all[lang] = d;
}
if (problems.length) {
  console.error(problems.join('\n'));
  process.exit(1);
}

// ---- PC ----
const pcOut = join(root, 'windows', 'app', 'ui', 'i18n.js');
mkdirSync(dirname(pcOut), { recursive: true });
writeFileSync(
  pcOut,
  '// GENERAT de tools/i18n_build.mjs din shared/i18n/*.json. Nu edita de mana.\n' +
    'window.COMIC_I18N = ' + JSON.stringify(all, null, 1) + ';\n',
  'utf8'
);
console.log('scris', pcOut);

// ---- Android ----
// Numele de resurse nu pot avea puncte: "ph.ready.wifi" -> "ph_ready_wifi".
// {pc} devine %1$s, ca sa mearga getString(R.string.x, valoare).
// Cateva nume vechi raman, ca sa nu se strice codul care le foloseste deja.
const ALIASES = {
  notification_title: 'notif.title',
  notification_text: 'notif.text',
  stop: 'notif.stop',
  channel_name: 'notif.channel',
};
const ANDROID_DIR = { en: 'values', pt: 'values-pt', uk: 'values-uk' };
const escapeXml = (s) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
   .replace(/'/g, "\\'").replace(/"/g, '\\"').replace(/^@/, '\\@').replace(/^\?/, '\\?');
const hasArgs = (s) => /\{\w+\}/.test(s);
const toAndroid = (s) => {
  // Fara argumente, textul nu trece prin String.format, deci % ramane %.
  if (!hasArgs(s)) return escapeXml(s);
  let n = 0;
  const seen = {};
  return escapeXml(s.replace(/%/g, '%%')).replace(/\{(\w+)\}/g, (_, name) => {
    if (!seen[name]) seen[name] = ++n;
    return `%${seen[name]}$s`;
  });
};
for (const lang of langs) {
  const d = all[lang];
  const dir = join(root, 'android', 'app', 'src', 'main', 'res', ANDROID_DIR[lang] ?? `values-${lang}`);
  mkdirSync(dir, { recursive: true });
  const lines = ['<?xml version="1.0" encoding="utf-8"?>',
    '<!-- GENERAT de tools/i18n_build.mjs din shared/i18n/' + lang + '.json. Nu edita de mana. -->',
    '<resources>'];
  if (lang === SOURCE) lines.push('    <string name="app_name" translatable="false">CoMic</string>');
  for (const [alias, key] of Object.entries(ALIASES)) {
    lines.push(`    <string name="${alias}">${toAndroid(d[key])}</string>`);
  }
  for (const key of Object.keys(en)) {
    if (key.startsWith('_')) continue;
    // Un % de procent intr-un text fara argumente: aapt trebuie sa stie ca nu e format.
    const value = toAndroid(d[key]);
    const formatted = !hasArgs(d[key]) && d[key].includes('%') ? ' formatted="false"' : '';
    lines.push(`    <string name="${key.replace(/[.\-]/g, '_')}"${formatted}>${value}</string>`);
  }
  lines.push('</resources>', '');
  writeFileSync(join(dir, 'strings.xml'), lines.join('\n'), 'utf8');
  console.log('scris', join(dir, 'strings.xml'));
}
