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
    height: 800,
    minWidth: 960,
    minHeight: 600,
    title: 'Block Survivors',
    backgroundColor: '#404040',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      sandbox: true,
    },
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
