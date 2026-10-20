/**
 * Directeur des boss : à intervalles réguliers (CALENDRIER_BOSS), annonce
 * l'arrivée du prochain boss quelques secondes avant, puis le fait apparaître
 * hors du champ de la caméra.
 */
import { CALENDRIER_BOSS } from '../donnees/boss';
import { ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { PROGRESSION } from '../donnees/progression';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import type { Terrain } from '../monde/navigation';
import { longueur } from '../utils/maths';
import type { DetecteurVisibilite } from './directeur';
import type { Ennemis } from './ennemis';

/** Ce qui affiche l'alerte à l'écran. */
export interface AfficheurAlerte {
  alerter(texte: string, duree: number): void;
}

export class DirecteurBoss {
  private readonly etat: EtatPartie;
  private readonly terrain: Terrain;
  private readonly ennemis: Ennemis;
  private readonly vue: DetecteurVisibilite;
  private readonly afficheur: AfficheurAlerte;
  private readonly retours: Retours;
  private prochain = 0;               // index dans l'ordre des boss
  private moment = CALENDRIER_BOSS.premierPassage;
  private annonce = false;

  constructor(
    etat: EtatPartie,
    terrain: Terrain,
    ennemis: Ennemis,
    vue: DetecteurVisibilite,
    afficheur: AfficheurAlerte,
    retours: Retours,
  ) {
    this.retours = retours;
    this.etat = etat;
    this.terrain = terrain;
    this.ennemis = ennemis;
    this.vue = vue;
    this.afficheur = afficheur;
  }

  /** Nom du prochain boss (debug). */
  get prochainNom(): string {
    return ENNEMIS[this.idProchain()].nom;
  }

  maj(): void {
    // Aucun boss au-delà de la fin de partie (la victoire arrive avant)
    if (this.etat.joueur.mort || this.etat.issue !== null || this.moment >= PROGRESSION.dureePartie) return;
    const t = this.etat.tempsJeu;
    if (!this.annonce && t >= this.moment - CALENDRIER_BOSS.avertissement) {
      this.annonce = true;
      this.afficheur.alerter(`Un boss approche : ${this.prochainNom} !`, CALENDRIER_BOSS.avertissement);
    }
    if (t >= this.moment) {
      this.faireApparaitre();
      this.moment += CALENDRIER_BOSS.intervalle;
      this.annonce = false;
    }
  }

  /** Fait apparaître le prochain boss tout de suite (debug), sans décaler le calendrier. */
  forcer(): void {
    this.afficheur.alerter(`${this.prochainNom} est là !`, 2);
    this.faireApparaitre();
  }

  private idProchain(): IdEnnemi {
    const ordre = CALENDRIER_BOSS.ordre;
    return ordre[this.prochain % ordre.length];
  }

  /** Apparition hors champ sur une case libre, sinon au coin opposé au joueur. */
  private faireApparaitre(): void {
    const id = this.idProchain();
    this.prochain++;
    const j = this.etat.joueur;
    const limite = this.etat.arene.taille / 2 - ENNEMIS[id].rayon - 1;
    let x = j.x > 0 ? -limite : limite;
    let z = j.z > 0 ? -limite : limite;
    for (let essai = 0; essai < 16; essai++) {
      const cx = (Math.random() * 2 - 1) * limite;
      const cz = (Math.random() * 2 - 1) * limite;
      if (this.terrain.carte.bloqueMarche(cx, cz)) continue;
      if (longueur(cx - j.x, cz - j.z) > 12 && !this.vue.estVisible(cx, cz, 3)) {
        x = cx;
        z = cz;
        break;
      }
    }
    this.ennemis.apparaitre(id, x, z);
    this.retours.son('boss');
    this.retours.secousse(0.6);
  }
}
