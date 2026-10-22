/**
 * Fin de partie : écran de mort du jeu (voile rouge, « Vous êtes mort ! »)
 * ou de victoire (voile doré), score, maîtrise de la classe gagnée avec l'XP
 * de la partie, bilan, Rejouer et Menu principal.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { PERSONNAGES } from '../donnees/personnages';
import type { IssuePartie } from '../etat';
import type { Recompense } from '../progression/sauvegarde';
import { element } from './dom';
import { bandeauMaitrise, partNiveau } from './maitrise';

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
  private readonly textures: BibliothequeTextures;
  private readonly hud = element('hud');
  private readonly actions: ActionsFin;

  constructor(actions: ActionsFin, textures: BibliothequeTextures) {
    this.actions = actions;
    this.textures = textures;
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

  /** Maîtrise de la classe (la barre se remplit de l'XP de la partie) et ce que la partie a débloqué. */
  private afficherRecompense(recompense: Recompense | null): void {
    this.recompense.replaceChildren();
    if (!recompense) return;
    const { avant, apres } = recompense;
    const bandeau = bandeauMaitrise(this.textures, PERSONNAGES[recompense.personnage].nom, apres, null);
    // La barre part d'où elle en était (du début si un niveau est passé) et se remplit
    const rempli = bandeau.querySelector<HTMLElement>('.maitrise-rempli');
    if (rempli) {
      const final = rempli.style.width;
      rempli.style.width = apres.niveau > avant.niveau ? '0%' : `${partNiveau(avant) * 100}%`;
      requestAnimationFrame(() => requestAnimationFrame(() => { rempli.style.width = final; }));
    }
    const gain = document.createElement('div');
    gain.className = 'fin-maitrise';
    const niveaux = apres.niveau - avant.niveau;
    gain.textContent = niveaux > 0
      ? `+${new Intl.NumberFormat('fr-FR').format(recompense.xp)} XP · niveau ${avant.niveau} → ${apres.niveau} : +${niveaux} point${niveaux > 1 ? 's' : ''} de talent !`
      : `+${new Intl.NumberFormat('fr-FR').format(recompense.xp)} XP de maîtrise`;
    gain.classList.toggle('niveau', niveaux > 0);
    this.recompense.append(bandeau, gain);
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
