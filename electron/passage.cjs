/**
 * Passage à la version Rust (docs/PLAN-RUST.md, section 5, « Passage depuis Electron »).
 * Coupé tant que BASCULE vaut false : on l'allume dans la toute dernière version Electron,
 * une fois la version Rust publiée en stable. Les joueurs n'ont alors rien à faire :
 *
 * 1. le manifeste signé de la version Rust (canal « win-stable » ou « linux-stable ») est lu
 *    et vérifié (ed25519, avec les clés publiques de CLES, et date d'expiration) ;
 * 2. chaque profil (localStorage du jeu) est écrit en `.bssave` dans le dossier
 *    `depuis-electron/` des sauvegardes de la version Rust, avec `actif.txt` : elle les
 *    importe à son premier lancement ;
 * 3. l'installeur Rust est téléchargé et vérifié (taille et SHA-256 signés) ;
 * 4. Windows : l'appli Electron est désinstallée sans effacer ses données (avant
 *    l'installation, sinon sa désinstallation retirerait les nouveaux raccourcis du même
 *    nom), puis l'installeur Rust est lancé. Linux : l'AppImage Rust remplace l'AppImage
 *    Electron au même endroit, puis est lancée.
 *
 * Au moindre échec, rien n'est fait : le jeu Electron s'ouvre comme d'habitude, et le
 * passage sera retenté au lancement suivant.
 */
const { app, BrowserWindow } = require('electron');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawn } = require('node:child_process');

/** À passer à true dans la dernière version Electron (avec CLES rempli). */
const BASCULE = false;
/**
 * Clés publiques racines des manifestes (base64, 32 octets) : celles de la variable
 * BS_CLES_MAJ. La clé de la CI n'y est pas : elle est autorisée par la délégation jointe.
 */
const CLES = [];
const DEPOT = 'https://github.com/TakeosOfficiel/Block-Survivors-version/releases/download/rust-{canal}/{nom}';
const DOSSIER_PROFILS = 'depuis-electron';

/** Mêmes constantes que src/progression/transfert.ts (fichiers .bssave). */
const ENTETE = 'BSSAVE1:';
const SECRET = 'Block Survivors · sauvegarde · 7c1e5a93d2f84b60';
const DONNEES_PROFIL = ['profil', 'progression', 'progression.multi', 'collection', 'choix', 'amis'];

function canal() {
  if (process.platform === 'win32') return 'win-stable';
  if (process.platform === 'linux' && process.env.APPIMAGE) return 'linux-stable';
  return null;
}

/** Préfixe DER (SPKI) d'une clé publique ed25519 brute. */
const SPKI_ED25519 = Buffer.from('302a300506032b6570032100', 'hex');

/** Signature ed25519 (base64) de `texte` par l'une des `cles` (base64). */
function signee(texte, signatureB64, cles) {
  const signature = Buffer.from(signatureB64 ?? '', 'base64');
  const donnees = Buffer.from(texte, 'utf8');
  return cles.some((cle) => {
    const brute = Buffer.from(cle, 'base64');
    if (brute.length !== 32) return false;
    const publique = crypto.createPublicKey({ key: Buffer.concat([SPKI_ED25519, brute]), format: 'der', type: 'spki' });
    return crypto.verify(null, donnees, publique, signature);
  });
}

/**
 * Manifeste vérifié (objet), ou une exception. Signé par une racine, ou par une clé de
 * la délégation jointe (signée par une racine, pas expirée), comme bs-maj.
 */
function verifierManifeste(texteEnveloppe, cles, maintenant = Date.now() / 1000) {
  const enveloppe = JSON.parse(texteEnveloppe);
  if (!signee(enveloppe.manifeste, enveloppe.signature, cles)) {
    const d = enveloppe.delegation;
    if (!d || !signee(d.texte, d.signature, cles)) throw new Error('signature du manifeste invalide');
    const delegation = JSON.parse(d.texte);
    if (!(delegation.expire > maintenant)) throw new Error('délégation expirée');
    if (!signee(enveloppe.manifeste, enveloppe.signature, delegation.cles ?? [])) throw new Error('signature du manifeste invalide');
  }
  const manifeste = JSON.parse(enveloppe.manifeste);
  if (!(manifeste.expire > maintenant)) throw new Error('manifeste expiré');
  return manifeste;
}

/** Contenu d'un fichier .bssave (AES-256-GCM, comme la WebCrypto du jeu). */
function chiffrer(texte) {
  const cle = crypto.createHash('sha256').update(SECRET, 'utf8').digest();
  const iv = crypto.randomBytes(12);
  const chiffreur = crypto.createCipheriv('aes-256-gcm', cle, iv);
  const corps = Buffer.concat([chiffreur.update(texte, 'utf8'), chiffreur.final(), chiffreur.getAuthTag()]);
  return ENTETE + Buffer.concat([iv, corps]).toString('base64');
}

/**
 * Fichiers .bssave des profils, à partir du localStorage du jeu (clé → valeur) :
 * `{ fichiers: [{nom, contenu}], actif }`.
 */
function fichiersProfils(stockage, version, date = new Date().toISOString()) {
  const fichiers = [];
  let actif = null;
  let registre = null;
  try {
    registre = JSON.parse(stockage['block-survivors.profils'] ?? 'null');
  } catch { /* registre illisible : comme sans profils */ }
  if (registre && Array.isArray(registre.ids) && registre.ids.length > 0) {
    for (const id of registre.ids) {
      const donnees = {};
      for (const nom of DONNEES_PROFIL) {
        const valeur = stockage[`block-survivors.p.${id}.${nom}`];
        if (typeof valeur === 'string') donnees[nom] = valeur;
      }
      if (Object.keys(donnees).length === 0) continue;
      const nom = `profil-${String(id).padStart(2, '0')}.bssave`;
      fichiers.push({ nom, contenu: chiffrer(JSON.stringify({ format: 'block-survivors-profil', version, date, donnees })) });
      if (id === registre.actif) actif = nom;
    }
  } else {
    // Sauvegarde d'avant les profils (jeu jamais relancé depuis) : toutes ses clés
    const donnees = {};
    for (const [cle, valeur] of Object.entries(stockage)) {
      if (cle.startsWith('block-survivors.')) donnees[cle] = valeur;
    }
    if (Object.keys(donnees).length > 0) {
      fichiers.push({ nom: 'profil-01.bssave', contenu: chiffrer(JSON.stringify({ format: 'block-survivors-sauvegarde', donnees })) });
      actif = 'profil-01.bssave';
    }
  }
  return { fichiers, actif };
}

/** Dossier des sauvegardes de la version Rust (bs-sauvegarde, Stockage::du_systeme). */
function dossierRust() {
  if (process.platform === 'win32') return path.join(app.getPath('appData'), 'block-survivors');
  const xdg = process.env.XDG_DATA_HOME;
  return path.join(xdg && xdg.length > 0 ? xdg : path.join(os.homedir(), '.local', 'share'), 'block-survivors');
}

/** Tout le localStorage du jeu, lu par une page vide (même origine file:// que le jeu). */
async function lireStockageDuJeu() {
  const fenetre = new BrowserWindow({ show: false, webPreferences: { sandbox: true, contextIsolation: true } });
  try {
    await fenetre.loadFile(path.join(__dirname, 'passage.html'));
    const texte = await fenetre.webContents.executeJavaScript('JSON.stringify(Object.fromEntries(Object.entries(localStorage)))');
    return JSON.parse(texte);
  } finally {
    fenetre.destroy();
  }
}

async function telecharger(url) {
  const reponse = await fetch(url, { redirect: 'follow' });
  if (!reponse.ok) throw new Error(`${url} : HTTP ${reponse.status}`);
  return Buffer.from(await reponse.arrayBuffer());
}

/** Lance la suite hors d'Electron (qui doit ensuite quitter). */
function lancerInstallation(installeur) {
  if (process.platform === 'win32') {
    const dossierAppli = path.dirname(process.execPath);
    const desinstalleur = path.join(dossierAppli, `Uninstall ${app.getName()}.exe`);
    const script = path.join(os.tmpdir(), 'block-survivors-passage.cmd');
    // Attendre la fermeture d'Electron, le désinstaller sur place (« _?= » : sans copie, on
    // attend la fin ; ses données restent), retirer son dossier, puis installer la version Rust
    fs.writeFileSync(script, [
      '@echo off',
      'ping -n 4 127.0.0.1 >nul',
      fs.existsSync(desinstalleur) ? `"${desinstalleur}" /S _?=${dossierAppli}` : 'rem pas de désinstalleur',
      `rmdir /s /q "${dossierAppli}"`,
      `start "" "${installeur}"`,
      '',
    ].join('\r\n'));
    spawn('cmd.exe', ['/d', '/c', script], { detached: true, stdio: 'ignore', windowsHide: true }).unref();
  } else {
    // L'AppImage Rust prend la place de l'AppImage Electron (raccourcis gardés)
    const cible = process.env.APPIMAGE;
    fs.copyFileSync(installeur, `${cible}.nouveau`);
    fs.chmodSync(`${cible}.nouveau`, 0o755);
    fs.renameSync(`${cible}.nouveau`, cible);
    spawn(cible, [], { detached: true, stdio: 'ignore' }).unref();
  }
}

/**
 * Fait le passage si tout est prêt ; `etat(texte, progression)` informe le lanceur.
 * Renvoie true si Electron doit quitter (la version Rust prend la suite).
 */
async function passerARust(etat) {
  const c = canal();
  if (!BASCULE || !c || CLES.length === 0) return false;
  try {
    etat('Passage à la nouvelle version…');
    const adresse = (nom) => DEPOT.replace('{canal}', c).replace('{nom}', nom);
    const manifeste = verifierManifeste((await telecharger(adresse('version.json'))).toString('utf8'), CLES);
    const attendu = (manifeste.installeurs ?? []).find((i) => (c.startsWith('win') ? i.fichier.endsWith('-Setup.exe') : i.fichier.endsWith('.AppImage')));
    if (!attendu) throw new Error('pas d\'installeur dans le manifeste');

    const { fichiers, actif } = fichiersProfils(await lireStockageDuJeu(), app.getVersion());
    const dossier = path.join(dossierRust(), DOSSIER_PROFILS);
    fs.mkdirSync(dossier, { recursive: true });
    for (const f of fichiers) fs.writeFileSync(path.join(dossier, f.nom), f.contenu);
    if (actif) fs.writeFileSync(path.join(dossier, 'actif.txt'), actif);

    etat(`Téléchargement de la version ${manifeste.version}…`);
    const octets = await telecharger(adresse(attendu.fichier));
    const empreinte = crypto.createHash('sha256').update(octets).digest('hex');
    if (octets.length !== attendu.taille || empreinte !== attendu.sha256.toLowerCase()) throw new Error('installeur corrompu');
    const installeur = path.join(os.tmpdir(), attendu.fichier);
    fs.writeFileSync(installeur, octets, { mode: 0o755 });

    etat('Installation…', 1);
    lancerInstallation(installeur);
    return true;
  } catch (erreur) {
    console.error('passage à la version Rust impossible :', erreur);
    return false;
  }
}

module.exports = { passerARust, verifierManifeste, fichiersProfils, chiffrer };
