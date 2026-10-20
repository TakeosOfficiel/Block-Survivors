/**
 * Boucle de jeu : deltaTime borné, simulation en sous-pas fixes, puis affichage.
 * Elle ne connaît aucun système ; le jeu lui fournit ses callbacks.
 */
import { CONFIG } from '../config';

export interface CallbacksBoucle {
  /** Un pas de simulation, en temps de jeu (s). */
  simuler(dt: number): void;
  /** Une image : temps de jeu simulé, temps réel borné, temps réel mesuré (s). */
  afficher(dtJeu: number, dtReel: number, dtBrut: number): void;
  /** Multiplicateur du temps de jeu. */
  vitesseTemps(): number;
}

export class Boucle {
  private readonly callbacks: CallbacksBoucle;
  private tempsPrecedent = -1;

  constructor(callbacks: CallbacksBoucle) {
    this.callbacks = callbacks;
  }

  demarrer(): void {
    requestAnimationFrame(this.image);
  }

  private readonly image = (tempsMs: number): void => {
    requestAnimationFrame(this.image);

    if (this.tempsPrecedent < 0) this.tempsPrecedent = tempsMs;
    const dtBrut = Math.max(0, (tempsMs - this.tempsPrecedent) / 1000);
    this.tempsPrecedent = tempsMs;
    const dtReel = Math.min(dtBrut, CONFIG.temps.dtMax);

    // Sous-pas bornés : la simulation reste stable même avec le temps accéléré
    const dtJeuDemande = dtReel * this.callbacks.vitesseTemps();
    const nbPas = Math.min(Math.ceil(dtJeuDemande / CONFIG.temps.pasSimMax), CONFIG.temps.sousPasMax);
    const pas = nbPas > 0 ? Math.min(dtJeuDemande / nbPas, CONFIG.temps.pasSimMax) : 0;
    for (let i = 0; i < nbPas; i++) this.callbacks.simuler(pas);

    this.callbacks.afficher(pas * nbPas, dtReel, dtBrut);
  };
}
