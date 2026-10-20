/**
 * Pause : Échap l'ouvre et la ferme ; la partie se met aussi en pause quand
 * la fenêtre perd le focus. Sans effet pendant le choix des cartes.
 */
import { element } from './dom';

export class EcranPause {
  private readonly racine = element('pause');
  private readonly peutPauser: () => boolean;
  /** Vrai tant que la pause est affichée. */
  active = false;

  /** `peutPauser` : faux quand un autre écran (cartes) a déjà figé la partie. */
  constructor(peutPauser: () => boolean) {
    this.peutPauser = peutPauser;
    element<HTMLButtonElement>('pause-reprendre').addEventListener('click', () => this.definir(false));
    window.addEventListener('keydown', this.surTouche);
    window.addEventListener('blur', () => this.definir(true));
  }

  definir(active: boolean): void {
    if (active && !this.peutPauser()) return;
    this.active = active;
    this.racine.classList.toggle('cache', !active);
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Escape' || e.repeat) return;
    e.preventDefault();
    this.definir(!this.active);
  };
}
