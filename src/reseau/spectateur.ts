/**
 * Joueur mort pour de bon, à plusieurs : il reste dans la partie en
 * spectateur. La caméra suit un allié encore debout (ou à terre, s'il n'y a
 * plus que ça) ; les touches gauche et droite passent d'un allié à l'autre.
 */
import type { Entrees } from '../controles/entrees';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import type { EtatJoueur } from '../etat';
import type { CameraJeu } from '../moteur/camera';
import type { Combattant } from './combattant';

export class Spectateur {
  /** Allié suivi (sa place dans la liste), −1 : aucun. */
  private suivi = -1;
  private gaucheAvant = false;
  private droiteAvant = false;

  /** Une image : choisit l'allié suivi par la caméra ; renvoie son pseudo, ou null hors du mode spectateur. */
  maj(moi: EtatJoueur, allies: readonly Combattant[], entrees: Entrees, camera: CameraJeu): string | null {
    if (!moi.mort || moi.tempsMort < REGLES_MULTIJOUEUR.reanimation.aTerre) {
      this.suivi = -1;
      camera.cible = null;
      return null;
    }
    const gauche = entrees.action('gauche');
    const droite = entrees.action('droite');
    const pas = (droite && !this.droiteAvant ? 1 : 0) - (gauche && !this.gaucheAvant ? 1 : 0);
    this.gaucheAvant = gauche;
    this.droiteAvant = droite;
    // L'allié suivi est parti ou tombé : un autre, debout de préférence
    const actuel = allies[this.suivi];
    if (pas !== 0 || !actuel || !suivable(actuel)) this.suivi = this.prochain(allies, pas < 0 ? -1 : 1);
    const c = allies[this.suivi];
    camera.cible = c ? c.etatJoueur : null;
    return c ? c.info.pseudo : null;
  }

  /** Allié suivant dans le sens `sens` : debout d'abord, sinon à terre. */
  private prochain(allies: readonly Combattant[], sens: number): number {
    const n = allies.length;
    for (const debout of [true, false]) {
      for (let p = 1; p <= n; p++) {
        const i = (((this.suivi < 0 ? 0 : this.suivi) + sens * p) % n + n) % n;
        const c = allies[i];
        if (suivable(c) && (!debout || !c.etatJoueur.mort)) return i;
      }
    }
    return -1;
  }
}

function suivable(c: Combattant): boolean {
  return !c.parti && (!c.etatJoueur.mort || c.etatJoueur.tempsMort < REGLES_MULTIJOUEUR.reanimation.aTerre);
}
