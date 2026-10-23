/**
 * Fin de partie : « Vous êtes mort ! » (voile rouge) ou « Victoire ! » (voile
 * doré), le personnage et l'arène, quatre tuiles (temps, ennemis, niveau,
 * émeraudes), la maîtrise de la classe gagnée avec l'XP de la partie, puis le
 * bilan complet dans un cadre doré. Menu principal, Rejouer, et après une
 * victoire « Continuer sans fin » (la partie reprend jusqu'à la mort).
 */
import type { BibliothequeTextures } from '../assets/textures';
import { ARMURES } from '../donnees/armures';
import { PERSONNAGES } from '../donnees/personnages';
import type { IdMonde, PieceArmure } from '../donnees/types';
import type { EtatPartie, IssuePartie } from '../etat';
import type { Recompense } from '../progression/sauvegarde';
import { formaterTemps } from '../utils/format';
import { remplirTuiles } from './bilan';
import { element } from './dom';
import { iconePiece } from './iconesObjets';
import { bandeauMaitrise, partNiveau } from './maitrise';

export interface ActionsFin {
  rejouer(): void;
  continuer(): void;
  menuPrincipal(): void;
  remplirBilan(conteneur: HTMLElement): void;
}

const NOMBRE = new Intl.NumberFormat('fr-FR');

export class EcranFin {
  private readonly racine = element('fin');
  private readonly titre = element('fin-titre');
  private readonly sousTitre = element('fin-sous-titre');
  private readonly tuiles = element('fin-tuiles');
  private readonly bilan = element('fin-bilan');
  private readonly recompense = element('fin-recompense');
  private readonly boutonContinuer = element<HTMLButtonElement>('fin-continuer');
  private readonly textures: BibliothequeTextures;
  private readonly hud = element('hud');
  private readonly actions: ActionsFin;

  constructor(actions: ActionsFin, textures: BibliothequeTextures) {
    this.actions = actions;
    this.textures = textures;
    element<HTMLButtonElement>('fin-rejouer').addEventListener('click', () => this.actions.rejouer());
    element<HTMLButtonElement>('fin-menu').addEventListener('click', () => this.actions.menuPrincipal());
    this.boutonContinuer.addEventListener('click', () => this.actions.continuer());
    window.addEventListener('keydown', this.surTouche);
  }

  get affiche(): boolean {
    return !this.racine.classList.contains('cache');
  }

  /** `piece` : pièce d'armure de la Collection gagnée avec la victoire. */
  afficher(issue: Exclude<IssuePartie, null>, etat: EtatPartie, recompense: Recompense | null, piece: PieceArmure | null): void {
    const victoire = issue === 'victoire';
    this.racine.classList.toggle('victoire', victoire);
    this.titre.textContent = victoire ? 'Victoire !' : 'Vous êtes mort !';
    const lieu = `${etat.personnage.nom} · ${etat.arene.nom}`;
    this.sousTitre.textContent = etat.sansFin
      ? `Mode sans fin · ${lieu}${recompense?.recordSansFin ? ` · nouveau record : ${formaterTemps(etat.tempsJeu)} !` : ''}`
      : lieu;
    this.sousTitre.classList.toggle('record', recompense?.recordSansFin === true);
    remplirTuiles(this.tuiles, etat, this.textures);
    this.afficherRecompense(recompense);
    if (piece) this.afficherPiece(etat.arene.monde, piece);
    this.actions.remplirBilan(this.bilan);
    this.bilan.scrollTop = 0;
    // Une partie gagnée peut continuer (une seule fois : ensuite, seule la mort l'arrête)
    this.boutonContinuer.classList.toggle('cache', !victoire || etat.sansFin);
    // Comme l'écran de mort du jeu : plus de HUD derrière
    this.hud.classList.add('cache');
    this.racine.classList.remove('cache');
  }

  /** Mode sans fin : la partie reprend, le HUD revient. */
  masquer(): void {
    this.racine.classList.add('cache');
    this.hud.classList.remove('cache');
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
      ? `+${NOMBRE.format(recompense.xp)} XP · niveau ${avant.niveau} → ${apres.niveau} : +${niveaux} point${niveaux > 1 ? 's' : ''} de talent !`
      : `+${NOMBRE.format(recompense.xp)} XP de maîtrise`;
    gain.classList.toggle('niveau', niveaux > 0);
    this.recompense.append(bandeau, gain);
    for (const deblocage of recompense.deblocages) {
      const ligne = document.createElement('div');
      ligne.className = 'fin-deblocage';
      ligne.textContent = `Débloqué : ${deblocage.nom} !`;
      this.recompense.append(ligne);
    }
  }

  /** Pièce d'armure gagnée : son icône, son nom et son monde, mis en avant sous la maîtrise. */
  private afficherPiece(monde: IdMonde, piece: PieceArmure): void {
    const ligne = document.createElement('div');
    ligne.className = 'fin-piece';
    const image = document.createElement('img');
    image.src = this.textures.url(iconePiece(monde, piece)) ?? '';
    image.alt = '';
    const texte = document.createElement('span');
    texte.textContent = `Nouvelle pièce pour la Collection : ${ARMURES[monde].pieces[piece].nom} !`;
    ligne.append(image, texte);
    this.recompense.append(ligne);
  }

  /** Entrée rejoue directement. */
  private readonly surTouche = (e: KeyboardEvent): void => {
    if (!this.affiche || e.repeat || e.code !== 'Enter') return;
    e.preventDefault();
    this.actions.rejouer();
  };
}
