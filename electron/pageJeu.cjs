/**
 * Mise à jour légère : tout le jeu tient dans une seule page (dist/index.html,
 * quelques Mo), le reste de l'installateur (Electron, plus de 300 Mo) ne change
 * presque jamais. Chaque release du dépôt public porte la page (jeu-X.Y.Z.html)
 * et jeu.json : sa version, son empreinte SHA-512 et l'empreinte du socle qu'elle
 * attend (scripts/socle.mjs). Si ce socle est celui de l'application installée
 * (electron/socle.json, écrit à la construction), seule la page est téléchargée,
 * vérifiée puis rangée dans le dossier de l'utilisateur ; le jeu l'ouvre à la
 * place de celle de l'installation. Sinon, l'installateur complet reste
 * nécessaire (electron-updater). Les sauvegardes ne bougent pas : toutes les
 * pages file:// partagent le même stockage.
 */
const { app, net } = require('electron');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const DEPOT = 'TakeosOfficiel/Block-Survivors-version';
const URL_PUBLIEE = `https://github.com/${DEPOT}/releases/latest/download/jeu.json`;
const urlPage = (version) => `https://github.com/${DEPOT}/releases/download/v${version}/jeu-${version}.html`;
/** Page livrée avec l'installation. */
const PAGE_INSTALLEE = path.join(__dirname, '..', 'dist', 'index.html');
const FORMAT_VERSION = /^\d+\.\d+\.\d+$/;

/** Empreinte du socle de cette installation, ou null (lancé depuis les sources). */
const SOCLE = (() => {
  try {
    const lu = JSON.parse(fs.readFileSync(path.join(__dirname, 'socle.json'), 'utf8'));
    return typeof lu.socle === 'string' ? lu.socle : null;
  } catch {
    return null;
  }
})();

const dossier = () => path.join(app.getPath('userData'), 'jeu');

/** « 0.10.0 » est plus récente que « 0.9.2 ». */
function plusRecente(a, b) {
  const pa = a.split('.').map(Number);
  const pb = b.split('.').map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}

/** Page téléchargée utilisable ({ version, chemin }) : même socle et plus récente que l'installation. */
function pageTelechargee() {
  try {
    const lu = JSON.parse(fs.readFileSync(path.join(dossier(), 'jeu.json'), 'utf8'));
    if (!SOCLE || lu.socle !== SOCLE || !FORMAT_VERSION.test(lu.version) || !plusRecente(lu.version, app.getVersion())) return null;
    const chemin = path.join(dossier(), `jeu-${lu.version}.html`);
    return fs.existsSync(chemin) ? { version: lu.version, chemin } : null;
  } catch {
    return null;
  }
}

/** La page du jeu à ouvrir : la plus récente des deux. */
function cheminPage() {
  return pageTelechargee()?.chemin ?? PAGE_INSTALLEE;
}

/**
 * Nouvelle version publiée ? Renvoie null si le jeu est à jour, sinon
 * `{ version, legere, sha512 }` : `legere` quand la page seule suffit.
 * Rejette si la recherche échoue (hors ligne, release sans jeu.json…).
 */
async function chercher(delaiMs) {
  const reponse = await net.fetch(URL_PUBLIEE, { cache: 'no-store', signal: AbortSignal.timeout(delaiMs) });
  if (!reponse.ok) throw new Error(`jeu.json : ${reponse.status}`);
  const publiee = await reponse.json();
  if (!FORMAT_VERSION.test(publiee.version) || typeof publiee.sha512 !== 'string') throw new Error('jeu.json illisible');
  const actuelle = pageTelechargee()?.version ?? app.getVersion();
  if (!plusRecente(publiee.version, actuelle)) return null;
  return { version: publiee.version, legere: SOCLE !== null && publiee.socle === SOCLE, sha512: publiee.sha512 };
}

/** Télécharge la page de cette version, la vérifie et la range ; `progression(0 à 1, négative : inconnue)`. */
async function telecharger(publiee, progression) {
  const reponse = await net.fetch(urlPage(publiee.version), { cache: 'no-store' });
  if (!reponse.ok || !reponse.body) throw new Error(`page : ${reponse.status}`);
  const total = Number(reponse.headers.get('content-length')) || 0;
  const morceaux = [];
  let recu = 0;
  const lecteur = reponse.body.getReader();
  for (;;) {
    const { done, value } = await lecteur.read();
    if (done) break;
    morceaux.push(value);
    recu += value.length;
    progression(total > 0 ? recu / total : -1);
  }
  const contenu = Buffer.concat(morceaux);
  if (crypto.createHash('sha512').update(contenu).digest('base64') !== publiee.sha512) throw new Error('page abîmée');

  const ici = dossier();
  fs.mkdirSync(ici, { recursive: true });
  const fichier = `jeu-${publiee.version}.html`;
  fs.writeFileSync(path.join(ici, fichier), contenu);
  // jeu.json en dernier, et d'un coup : jamais de page annoncée à moitié écrite
  const description = path.join(ici, 'jeu.json');
  fs.writeFileSync(`${description}.tmp`, JSON.stringify({ version: publiee.version, socle: SOCLE }));
  fs.renameSync(`${description}.tmp`, description);
  for (const nom of fs.readdirSync(ici)) {
    if (nom.endsWith('.html') && nom !== fichier) fs.rmSync(path.join(ici, nom), { force: true });
  }
}

module.exports = { cheminPage, chercher, telecharger };
