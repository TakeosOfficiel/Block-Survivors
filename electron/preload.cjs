/** Pont entre la page du jeu et Electron : seulement ce dont l'interface a besoin. */
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('bureau', {
  quitter: () => ipcRenderer.send('quitter'),
  /** Mise à jour automatique : `rappel(version)` quand une nouvelle version est téléchargée. */
  surMiseAJourPrete: (rappel) => ipcRenderer.on('mise-a-jour-prete', (_evenement, version) => rappel(String(version))),
  installerMiseAJour: () => ipcRenderer.send('installer-mise-a-jour'),
});
