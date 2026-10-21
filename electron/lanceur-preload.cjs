/** Pont de la fenêtre de lancement : elle affiche l'état de la mise à jour et peut la passer. */
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('lanceur', {
  /** `rappel(texte, progression)` : progression entre 0 et 1, négative si inconnue. */
  surEtat: (rappel) => ipcRenderer.on('lanceur-etat', (_evenement, texte, progression) => rappel(String(texte), Number(progression))),
  /** Le téléchargement traîne : le joueur peut lancer le jeu sans l'attendre. */
  surPassable: (rappel) => ipcRenderer.on('lanceur-passable', () => rappel()),
  passer: () => ipcRenderer.send('lanceur-passer'),
});
