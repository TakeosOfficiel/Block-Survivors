/**
 * Fin de partie : écran de mort du jeu (voile rouge, « Vous êtes mort ! »)
 * ou de victoire (voile doré), score, bilan, Rejouer et Menu principal.
 */
import type { IssuePartie } from '../etat';
import type { Recompense } from '../progression/sauvegarde';
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
  private readonly recompense = element('fin-recompense');
  private readonly iconeEmeraude: string | null;
  private readonly hud = element('hud');
  private readonly actions: ActionsFin;

  constructor(actions: ActionsFin, iconeEmeraude: string | null) {
    this.actions = actions;
    this.iconeEmeraude = iconeEmeraude;
    element<HTMLButtonElement>('fin-rejouer').addEventListener('click', () => this.actions.rejouer());
    element<HTMLButtonElement>('fin-menu').addEventListener('click', () => this.actions.menuPrincipal());
    window.addEventListener('keydown', this.surTouche);
  }

  get affiche(): boolean {
    return !this.racine.classList.contains('cache');
  }

  afficher(issue: Exclude<IssuePartie, null>, score: number, temps: string, recompense: Recompense | null): void {
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
    this.afficherRecompense(recompense);
    this.actions.remplirBilan(this.bilan);
    // Comme l'écran de mort du jeu : plus de HUD derrière
    this.hud.classList.add('cache');
    this.racine.classList.remove('cache');
  }

  /** « +12 émeraudes » et ce que la partie a débloqué. */
  private afficherRecompense(recompense: Recompense | null): void {
    this.recompense.replaceChildren();
    if (!recompense) return;
    const gain = document.createElement('div');
    gain.className = 'fin-emeraudes';
    if (this.iconeEmeraude) {
      const image = document.createElement('img');
      image.src = this.iconeEmeraude;
      image.alt = '';
      gain.append(image);
    }
    gain.append(`+${recompense.emeraudes} émeraude${recompense.emeraudes > 1 ? 's' : ''}`);
    this.recompense.append(gain);
    for (const deblocage of recompense.deblocages) {
      const ligne = document.createElement('div');
      ligne.className = 'fin-deblocage';
      ligne.textContent = `Débloqué : ${deblocage.nom} !`;
      this.recompense.append(ligne);
    }
  }

  /** Entrée rejoue directement. */
  private readonly surTouche = (e: KeyboardEvent): void => {
    if (!this.affiche || e.repeat || e.code !== 'Enter') return;
    e.preventDefault();
    this.actions.rejouer();
  };
}
