/** Pont entre la page du jeu et Electron : seulement ce dont l'interface a besoin. */
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('bureau', {
  quitter: () => ipcRenderer.send('quitter'),
});
