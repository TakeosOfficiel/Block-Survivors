/**
 * Application de bureau (Electron) : une fenêtre qui charge le jeu construit
 * (dist/index.html), ou le serveur de développement Vite quand
 * BLOCK_SURVIVORS_URL est défini (npm run app:dev, rechargement à chaud).
 *
 * Mise à jour façon Discord (application installée) : au lancement, une
 * petite fenêtre (lanceur.html) cherche une nouvelle version dans les releases
 * du dépôt public. Le plus souvent, seule la page du jeu a changé : elle est
 * téléchargée (quelques Mo) et le jeu s'ouvre aussitôt dessus (pageJeu.cjs).
 * Si Electron lui-même a changé, l'installateur complet est téléchargé,
 * installé sans rien demander, puis le jeu repart à jour. Sans nouvelle
 * version (ou hors ligne), le jeu s'ouvre aussitôt. Pendant la partie, une
 * version sortie entre-temps se télécharge en arrière-plan et sert au prochain
 * lancement (la page la signale discrètement).
 */
const { app, BrowserWindow, Menu, ipcMain, shell } = require('electron');
const path = require('node:path');
const { autoUpdater } = require('electron-updater');
const pageJeu = require('./pageJeu.cjs');

const VERIFICATION_MISES_A_JOUR = 30 * 60 * 1000;
/** Sans réponse au bout de ce délai, le jeu s'ouvre sans attendre la recherche. */
const DELAI_RECHERCHE = 8000;
/** Téléchargement plus long que ça : le lanceur propose de jouer sans l'attendre. */
const DELAI_PASSER = 15000;
/** Version téléchargée et prête à installer (envoyée aussi aux fenêtres ouvertes après). */
let versionPrete = null;
/** La version prête n'est qu'une nouvelle page du jeu : il suffit de la recharger. */
let pagePrete = false;
let recherche = false;
/** Fenêtre de lancement (null une fois le jeu ouvert). */
let lanceur = null;
let jeuLance = false;
let telechargement = false;

// Les sons du jeu sont permis dès le lancement, sans attendre un premier clic
app.commandLine.appendSwitch('autoplay-policy', 'no-user-gesture-required');

function creerFenetre() {
  const fenetre = new BrowserWindow({
    width: 1280,
    height: 820,
    minWidth: 800,
    minHeight: 520,
    useContentSize: true,
    show: false,  // affichée une fois prête : pas de flash blanc au lancement
    title: 'Block Survivors',
    icon: path.join(__dirname, 'icone.png'),
    backgroundColor: '#2b2b2b',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      sandbox: true,
    },
  });

  fenetre.once('ready-to-show', () => {
    fenetre.maximize();
    fenetre.show();
    // Le lanceur se ferme une fois le jeu affiché (jamais avant : l'application quitterait)
    lanceur?.close();
    lanceur = null;
  });
  fenetre.webContents.on('did-finish-load', () => {
    if (versionPrete) fenetre.webContents.send('mise-a-jour-prete', versionPrete);
  });

  const urlDev = process.env.BLOCK_SURVIVORS_URL;
  if (urlDev) fenetre.loadURL(urlDev);
  else fenetre.loadFile(pageJeu.cheminPage());

  // F11 : plein écran
  fenetre.webContents.on('before-input-event', (evenement, entree) => {
    if (entree.type === 'keyDown' && entree.key === 'F11') {
      fenetre.setFullScreen(!fenetre.isFullScreen());
      evenement.preventDefault();
    }
  });
  // Les liens externes s'ouvrent dans le navigateur, jamais dans le jeu
  fenetre.webContents.setWindowOpenHandler(({ url }) => {
    void shell.openExternal(url);
    return { action: 'deny' };
  });
  return fenetre;
}

/** Petite fenêtre sans cadre du lancement : logo, état de la mise à jour, progression. */
function creerLanceur() {
  lanceur = new BrowserWindow({
    width: 320,
    height: 360,
    frame: false,
    resizable: false,
    maximizable: false,
    fullscreenable: false,
    show: false,
    center: true,
    title: 'Block Survivors',
    icon: path.join(__dirname, 'icone.png'),
    backgroundColor: '#1f1e22',
    webPreferences: {
      preload: path.join(__dirname, 'lanceur-preload.cjs'),
      contextIsolation: true,
      sandbox: true,
    },
  });
  lanceur.once('ready-to-show', () => lanceur?.show());
  lanceur.on('closed', () => { lanceur = null; });
  lanceur.loadFile(path.join(__dirname, 'lanceur.html'));
}

/** Texte et progression (0 à 1, négative : inconnue) affichés par le lanceur. */
function etatLanceur(texte, progression = -1) {
  lanceur?.webContents.send('lanceur-etat', texte, progression);
}

/** Ouvre le jeu (une seule fois) ; le lanceur se ferme quand il s'affiche. */
function lancerJeu() {
  if (jeuLance) return;
  jeuLance = true;
  etatLanceur('Lancement…');
  creerFenetre();
}

/** Seulement l'application installée : pas en développement ni lancée depuis les sources. */
function misesAJourActives() {
  return app.isPackaged && !process.env.BLOCK_SURVIVORS_URL;
}

function brancherMisesAJour() {
  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.on('checking-for-update', () => etatLanceur('Recherche de mises à jour…'));
  autoUpdater.on('update-available', (info) => {
    if (jeuLance) return;
    telechargement = true;
    etatLanceur(`Téléchargement de la version ${info.version}…`, 0);
    setTimeout(() => lanceur?.webContents.send('lanceur-passable'), DELAI_PASSER);
  });
  autoUpdater.on('download-progress', (p) => {
    if (!jeuLance) etatLanceur(`Téléchargement de la mise à jour… ${Math.floor(p.percent)} %`, p.percent / 100);
  });
  autoUpdater.on('update-not-available', () => lancerJeu());
  autoUpdater.on('update-downloaded', (info) => {
    versionPrete = info.version;
    if (!jeuLance) {
      // Au lancement : installée tout de suite, sans rien demander, puis le jeu repart à jour
      etatLanceur('Installation de la mise à jour…', 1);
      setTimeout(() => autoUpdater.quitAndInstall(true, true), 500);
      return;
    }
    // En pleine partie : rien n'interrompt le jeu, elle s'installera à la fermeture
    pagePrete = false;
    annoncerVersionPrete();
  });
  // Hors ligne, release incomplète… : on joue quand même
  autoUpdater.on('error', () => lancerJeu());
}

/** Prévient les fenêtres du jeu qu'une version est prête. */
function annoncerVersionPrete() {
  for (const fenetre of BrowserWindow.getAllWindows()) fenetre.webContents.send('mise-a-jour-prete', versionPrete);
}

/**
 * Cherche une nouvelle version : la page du jeu seule si elle suffit, sinon
 * l'installateur complet (electron-updater). Au lancement, le jeu s'ouvre
 * ensuite sur la version la plus récente ; en pleine partie, rien ne
 * l'interrompt.
 */
async function chercherMiseAJour() {
  if (recherche) return;
  recherche = true;
  if (!jeuLance) etatLanceur('Recherche de mises à jour…');
  try {
    let publiee;
    try {
      publiee = await pageJeu.chercher(DELAI_RECHERCHE);
    } catch {
      publiee = { legere: false };  // jeu.json introuvable : electron-updater décide
    }
    if (!publiee) {
      lancerJeu();
      return;
    }
    if (!publiee.legere) {
      await autoUpdater.checkForUpdates().catch(() => lancerJeu());
      return;
    }
    if (pagePrete && versionPrete === publiee.version) return;
    if (!jeuLance) {
      telechargement = true;
      etatLanceur(`Téléchargement de la version ${publiee.version}…`, 0);
      setTimeout(() => lanceur?.webContents.send('lanceur-passable'), DELAI_PASSER);
    }
    try {
      await pageJeu.telecharger(publiee, (part) => {
        if (!jeuLance) etatLanceur(`Téléchargement de la mise à jour… ${Math.max(0, Math.floor(part * 100))} %`, part);
      });
    } catch {
      lancerJeu();
      return;
    }
    if (!jeuLance) {
      lancerJeu();  // s'ouvre directement sur la nouvelle page
      return;
    }
    versionPrete = publiee.version;
    pagePrete = true;
    annoncerVersionPrete();
  } finally {
    recherche = false;
  }
}

Menu.setApplicationMenu(null);
ipcMain.on('quitter', () => app.quit());
// Redémarre sur la nouvelle version, sans demander (installateur silencieux)
ipcMain.on('installer-mise-a-jour', () => {
  if (!versionPrete) return;
  if (!pagePrete) {
    autoUpdater.quitAndInstall(true, true);
    return;
  }
  // Nouvelle page du jeu : rechargée sur place, sans réinstaller
  versionPrete = null;
  pagePrete = false;
  for (const fenetre of BrowserWindow.getAllWindows()) fenetre.loadFile(pageJeu.cheminPage());
});
// Le téléchargement traîne : le joueur lance le jeu, la mise à jour finira en arrière-plan
ipcMain.on('lanceur-passer', () => lancerJeu());
app.whenReady().then(() => {
  if (!misesAJourActives()) {
    creerFenetre();
    return;
  }
  brancherMisesAJour();
  creerLanceur();
  void chercherMiseAJour();
  setTimeout(() => { if (!telechargement) lancerJeu(); }, DELAI_RECHERCHE);
  setInterval(() => void chercherMiseAJour(), VERIFICATION_MISES_A_JOUR);
});
app.on('window-all-closed', () => app.quit());
