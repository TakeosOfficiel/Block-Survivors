/**
 * Pause : Échap l'ouvre et la ferme ; la partie se met aussi en pause quand
 * la fenêtre perd le focus. Reprendre, options, recommencer, menu principal,
 * et le bilan de la partie en cours.
 */
import { element } from './dom';
import type { PanneauOptions } from './panneauOptions';

export interface ActionsPause {
  /** Faux quand un autre écran (cartes, fin) a déjà figé la partie. */
  peutPauser(): boolean;
  recommencer(): void;
  menuPrincipal(): void;
  remplirBilan(conteneur: HTMLElement): void;
}

export class EcranPause {
  private readonly racine = element('pause');
  private readonly bilan = element('pause-bilan');
  private readonly actions: ActionsPause;
  private readonly options: PanneauOptions;
  /** Vrai tant que la pause est affichée. */
  active = false;

  constructor(actions: ActionsPause, options: PanneauOptions) {
    this.actions = actions;
    this.options = options;
    element<HTMLButtonElement>('pause-reprendre').addEventListener('click', () => this.definir(false));
    element<HTMLButtonElement>('pause-options').addEventListener('click', () => {
      this.racine.classList.add('cache');
      this.options.ouvrir(() => this.racine.classList.toggle('cache', !this.active));
    });
    element<HTMLButtonElement>('pause-recommencer').addEventListener('click', () => this.actions.recommencer());
    element<HTMLButtonElement>('pause-menu').addEventListener('click', () => this.actions.menuPrincipal());
    window.addEventListener('keydown', this.surTouche);
  }

  definir(active: boolean): void {
    if (active === this.active) return;
    if (active && !this.actions.peutPauser()) return;
    this.active = active;
    if (!active) this.options.fermer();
    if (active) this.actions.remplirBilan(this.bilan);
    this.racine.classList.toggle('cache', !active);
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Escape' || e.repeat) return;
    e.preventDefault();
    this.definir(!this.active);
  };
}
