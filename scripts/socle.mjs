/**
 * Empreinte du socle de l'application de bureau : tout ce que contient
 * l'installateur à part la page du jeu (dist/index.html). Fichiers d'Electron
 * (electron/), versions d'Electron et d'electron-updater, réglages de
 * l'installateur. Tant qu'elle ne change pas, une nouvelle version n'a besoin
 * que de la nouvelle page du jeu : le jeu installé la télécharge seule (quelques
 * Mo) au lieu de réinstaller toute l'application (voir electron/main.cjs).
 *
 *   node scripts/socle.mjs            affiche l'empreinte
 *   node scripts/socle.mjs --ecrire   l'écrit dans electron/socle.json (avant electron-builder)
 *   node scripts/socle.mjs --page release
 *       copie la page construite en release/jeu-X.Y.Z.html et écrit
 *       release/jeu.json (version, empreinte SHA-512 de la page, socle attendu)
 */
import { createHash } from 'node:crypto';
import { copyFileSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const RACINE = join(import.meta.dirname, '..');
const DOSSIER = join(RACINE, 'electron');
const FICHIER = 'socle.json';

const hachage = createHash('sha256');
for (const nom of readdirSync(DOSSIER).filter((n) => n !== FICHIER).sort()) {
  // Fins de ligne normalisées : la même empreinte sous Windows et ailleurs
  const contenu = readFileSync(join(DOSSIER, nom));
  hachage.update(`${nom}\0`).update(nom.endsWith('.png') ? contenu : contenu.toString('utf8').replace(/\r\n/g, '\n')).update('\0');
}
const paquet = JSON.parse(readFileSync(join(RACINE, 'package.json'), 'utf8'));
const verrou = JSON.parse(readFileSync(join(RACINE, 'package-lock.json'), 'utf8'));
const versionInstallee = (nom) => verrou.packages?.[`node_modules/${nom}`]?.version ?? '';
hachage.update(JSON.stringify({
  main: paquet.main,
  build: paquet.build,
  electron: versionInstallee('electron'),
  electronUpdater: versionInstallee('electron-updater'),
  electronBuilder: versionInstallee('electron-builder'),
}));
const socle = hachage.digest('hex').slice(0, 16);

const indexPage = process.argv.indexOf('--page');
if (indexPage >= 0) {
  const sortie = resolve(RACINE, process.argv[indexPage + 1] ?? 'release');
  const page = join(RACINE, 'dist', 'index.html');
  const sha512 = createHash('sha512').update(readFileSync(page)).digest('base64');
  copyFileSync(page, join(sortie, `jeu-${paquet.version}.html`));
  writeFileSync(join(sortie, 'jeu.json'), `${JSON.stringify({ version: paquet.version, socle, sha512 }, null, 2)}\n`);
  console.log(`jeu-${paquet.version}.html, jeu.json → socle ${socle}`);
} else if (process.argv.includes('--ecrire')) {
  writeFileSync(join(DOSSIER, FICHIER), `${JSON.stringify({ socle }, null, 2)}\n`);
  console.log(`electron/${FICHIER} → ${socle}`);
} else {
  console.log(socle);
}
