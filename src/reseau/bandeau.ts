/**
 * Ce que le joueur local doit savoir en multijoueur, dans un bandeau en haut
 * de l'écran : à terre (et sa réanimation), temps restant pour choisir ses
 * cartes, connexion perdue. Le texte n'est reconstruit (et le DOM touché) que
 * quand ce qu'il affiche change.
 */
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import type { EtatJoueur } from '../etat';

/** Ce qu'affiche le bandeau, codé en un nombre (situation × 1000 + valeur). */
const RIEN = 0;
const MORT = 1;
const A_TERRE = 2;
const REANIMATION = 3;
const CHOIX = 4;
const CONNEXION_PERDUE = 5;

export class SuiviLocal {
  private readonly element: HTMLDivElement;
  private affiche = -1;
  /** Instant (ms, horloge réelle) où l'écran des cartes s'est ouvert, −1 s'il est fermé. */
  private debutChoix = -1;
  private connexionPerdue = false;

  constructor() {
    this.element = document.createElement('div');
    this.element.className = 'reseau-alerte cache';
    document.body.append(this.element);
  }

  /** Allié suivi en spectateur, tel qu'affiché. */
  private spectateurAffiche: string | null = null;

  signalerConnexionPerdue(): void {
    this.connexionPerdue = true;
  }

  /**
   * Une image : met le bandeau à jour ; renvoie vrai quand le temps de choix
   * des cartes est écoulé (une carte doit être prise d'office).
   */
  /**
   * `spectateur` : pseudo de l'allié que suit la caméra, une fois mort pour de
   * bon (null : aucun).
   */
  maj(cartesOuvertes: boolean, j: EtatJoueur, reanimation: number, spectateur: string | null = null): boolean {
    const delai = REGLES_MULTIJOUEUR.choixCartes.delai;
    // Horloge réelle : le délai reste juste même quand les images ralentissent
    const maintenant = performance.now();
    if (!cartesOuvertes) this.debutChoix = -1;
    else if (this.debutChoix < 0) this.debutChoix = maintenant;
    const tempsChoix = cartesOuvertes ? (maintenant - this.debutChoix) / 1000 : 0;
    const r = REGLES_MULTIJOUEUR.reanimation;
    let code = RIEN;
    if (this.connexionPerdue) code = CONNEXION_PERDUE * 1000;
    else if (j.mort && j.tempsMort >= r.aTerre) code = MORT * 1000 + (spectateur === null ? 0 : 1);
    else if (j.mort && reanimation > 0) code = REANIMATION * 1000 + Math.floor(reanimation * 10) * 10;
    else if (j.mort) code = A_TERRE * 1000 + Math.ceil(r.aTerre - j.tempsMort);
    else if (cartesOuvertes) code = CHOIX * 1000 + Math.max(0, Math.ceil(delai - tempsChoix));
    if (code !== this.affiche || spectateur !== this.spectateurAffiche) {
      this.affiche = code;
      this.spectateurAffiche = spectateur;
      this.element.textContent = texte(code, spectateur ?? '');
      this.element.classList.toggle('cache', code === RIEN);
    }
    if (tempsChoix < delai) return false;
    this.debutChoix = -1;
    return true;
  }
}

function texte(code: number, spectateur: string): string {
  const valeur = code % 1000;
  switch (Math.floor(code / 1000)) {
    case CONNEXION_PERDUE: return 'Connexion avec l\'hôte perdue';
    case MORT: return valeur === 0
      ? 'Vous êtes mort : vos alliés continuent sans vous'
      : `Spectateur : vous suivez ${spectateur} (← → pour changer)`;
    case REANIMATION: return `Réanimation : ${valeur} %`;
    case A_TERRE: return `À terre ! Un allié doit rester près de vous pour vous relever (${valeur} s)`;
    case CHOIX: return `Choisissez une carte : la partie continue autour de vous (${valeur} s)`;
    default: return '';
  }
}
