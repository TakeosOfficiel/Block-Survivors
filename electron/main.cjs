/**
 * Application de bureau (Electron) : une fenêtre qui charge le jeu construit
 * (dist/index.html), ou le serveur de développement Vite quand
 * BLOCK_SURVIVORS_URL est défini (npm run app:dev, rechargement à chaud).
 */
const { app, BrowserWindow, Menu, ipcMain, shell } = require('electron');
const path = require('node:path');

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

Menu.setApplicationMenu(null);
ipcMain.on('quitter', () => app.quit());
app.whenReady().then(creerFenetre);
app.on('window-all-closed', () => app.quit());
