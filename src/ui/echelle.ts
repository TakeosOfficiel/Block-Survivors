/**
 * Échelle de l'interface, comme le réglage « Taille de l'interface » du jeu :
 * menus, écrans et HUD sont conçus pour une fenêtre de référence et agrandis
 * ou réduits pour tenir dans la fenêtre réelle, sans jamais défiler.
 * Au-delà de 1, l'échelle avance par quarts pour garder des pixels réguliers.
 */
const LARGEUR_REFERENCE = 1280;
const HAUTEUR_REFERENCE = 820;
const ECHELLE_MIN = 0.5;

export function suivreEchelleInterface(): void {
  const appliquer = () => {
    const brute = Math.min(window.innerWidth / LARGEUR_REFERENCE, window.innerHeight / HAUTEUR_REFERENCE);
    const echelle = brute >= 1 ? Math.floor(brute * 4) / 4 : Math.max(ECHELLE_MIN, brute);
    document.documentElement.style.setProperty('--echelle', String(echelle));
  };
  window.addEventListener('resize', appliquer);
  appliquer();
}
