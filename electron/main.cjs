/**
 * Application de bureau (Electron) : une fenêtre qui charge le jeu construit
 * (dist/index.html), ou le serveur de développement Vite quand
 * BLOCK_SURVIVORS_URL est défini (npm run app:dev, rechargement à chaud).
 *
 * Mise à jour automatique (application installée) : au lancement puis toutes
 * les 30 minutes, electron-updater cherche une nouvelle version dans les
 * releases du dépôt public, la télécharge en arrière-plan et prévient la page ;
 * le joueur redémarre quand il veut (sinon elle s'installe en quittant le jeu).
 */
const { app, BrowserWindow, Menu, ipcMain, shell } = require('electron');
const path = require('node:path');
const { autoUpdater } = require('electron-updater');

const VERIFICATION_MISES_A_JOUR = 30 * 60 * 1000;
/** Version téléchargée et prête à installer (envoyée aussi aux fenêtres ouvertes après). */
let versionPrete = null;

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
  });
  fenetre.webContents.on('did-finish-load', () => {
    if (versionPrete) fenetre.webContents.send('mise-a-jour-prete', versionPrete);
  });

  const urlDev = process.env.BLOCK_SURVIVORS_URL;
  if (urlDev) fenetre.loadURL(urlDev);
  else fenetre.loadFile(path.join(__dirname, '..', 'dist', 'index.html'));

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
}

/** Cherche une mise à jour ; une erreur (hors ligne…) ne gêne jamais le jeu. */
function verifierMiseAJour() {
  autoUpdater.checkForUpdates().catch(() => {});
}

function brancherMisesAJour() {
  // Seulement l'application installée : pas en développement ni lancée depuis les sources
  if (!app.isPackaged || process.env.BLOCK_SURVIVORS_URL) return;
  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.on('update-downloaded', (info) => {
    versionPrete = info.version;
    for (const fenetre of BrowserWindow.getAllWindows()) fenetre.webContents.send('mise-a-jour-prete', versionPrete);
  });
  autoUpdater.on('error', () => {});
  verifierMiseAJour();
  setInterval(verifierMiseAJour, VERIFICATION_MISES_A_JOUR);
}

Menu.setApplicationMenu(null);
ipcMain.on('quitter', () => app.quit());
// Redémarre sur la nouvelle version, sans demander (installateur silencieux)
ipcMain.on('installer-mise-a-jour', () => {
  if (versionPrete) autoUpdater.quitAndInstall(true, true);
});
app.whenReady().then(() => {
  creerFenetre();
  brancherMisesAJour();
});
app.on('window-all-closed', () => app.quit());
