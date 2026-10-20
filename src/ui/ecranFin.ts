/**
 * Fin de partie : écran de mort du jeu (voile rouge, « Vous êtes mort ! »)
 * ou de victoire (voile doré), score, bilan, Rejouer et Menu principal.
 */
import type { IssuePartie } from '../etat';
import { element } from './dom';

export interface ActionsFin {
  rejouer(): void;
  menuPrincipal(): void;
  remplirBilan(conteneur: HTMLElement): void;
}

export class EcranFin {
  private readonly racine = element('fin');
  private readonly titre = element('fin-titre');
  private readonly sousTitre = element('fin-sous-titre');
  private readonly bilan = element('fin-bilan');
  private readonly hud = element('hud');
  private readonly actions: ActionsFin;

  constructor(actions: ActionsFin) {
    this.actions = actions;
    element<HTMLButtonElement>('fin-rejouer').addEventListener('click', () => this.actions.rejouer());
    element<HTMLButtonElement>('fin-menu').addEventListener('click', () => this.actions.menuPrincipal());
    window.addEventListener('keydown', this.surTouche);
  }

  get affiche(): boolean {
    return !this.racine.classList.contains('cache');
  }

  afficher(issue: Exclude<IssuePartie, null>, score: number, temps: string): void {
    const victoire = issue === 'victoire';
    this.racine.classList.toggle('victoire', victoire);
    this.titre.textContent = victoire ? 'Victoire !' : 'Vous êtes mort !';

    // « Score : 123 » comme sur l'écran de mort du jeu, valeur en jaune
    const valeur = document.createElement('span');
    valeur.className = 'valeur';
    valeur.textContent = new Intl.NumberFormat('fr-FR').format(score);
    this.sousTitre.replaceChildren(
      victoire ? `Vous avez survécu ${temps}. Score : ` : `Survécu ${temps}. Score : `,
      valeur,
    );
    this.actions.remplirBilan(this.bilan);
    // Comme l'écran de mort du jeu : plus de HUD derrière
    this.hud.classList.add('cache');
    this.racine.classList.remove('cache');
  }

  /** Entrée rejoue directement. */
  private readonly surTouche = (e: KeyboardEvent): void => {
    if (!this.affiche || e.repeat || e.code !== 'Enter') return;
    e.preventDefault();
    this.actions.rejouer();
  };
}
