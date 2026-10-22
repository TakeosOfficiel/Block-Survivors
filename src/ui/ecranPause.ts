/**
 * Pause : Échap l'ouvre et la ferme ; la partie se met aussi en pause quand
 * la fenêtre perd le focus. Un bandeau en travers de l'écran (résumé de la
 * partie, Options, Reprendre, Recommencer, Menu principal) ; « Statistiques »
 * affiche à sa place le bilan complet de la partie en cours.
 */
import { element } from './dom';
import type { PanneauOptions } from './panneauOptions';

export interface ActionsPause {
  /** Faux quand un autre écran (cartes, fin) a déjà figé la partie. */
  peutPauser(): boolean;
  recommencer(): void;
  menuPrincipal(): void;
  remplirResume(conteneur: HTMLElement): void;
  remplirBilan(conteneur: HTMLElement): void;
}

export class EcranPause {
  private readonly racine = element('pause');
  private readonly pastilles = element('pause-pastilles');
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
    element<HTMLButtonElement>('pause-statistiques').addEventListener('click', () => this.statistiques(true));
    element<HTMLButtonElement>('pause-retour').addEventListener('click', () => this.statistiques(false));
    window.addEventListener('keydown', this.surTouche);
  }

  definir(active: boolean): void {
    if (active === this.active) return;
    if (active && !this.actions.peutPauser()) return;
    this.active = active;
    if (!active) this.options.fermer();
    if (active) this.actions.remplirResume(this.pastilles);
    this.statistiques(false);
    this.racine.classList.toggle('cache', !active);
  }

  /** Bilan complet à la place du bandeau, ou retour au bandeau. */
  private statistiques(visibles: boolean): void {
    if (visibles) this.actions.remplirBilan(this.bilan);
    this.racine.classList.toggle('statistiques', visibles);
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Escape' || e.repeat) return;
    e.preventDefault();
    // Échap quitte d'abord les statistiques
    if (this.active && this.racine.classList.contains('statistiques')) {
      this.statistiques(false);
      return;
    }
    this.definir(!this.active);
  };
}
