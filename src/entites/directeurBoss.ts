/**
 * Directeur des boss (ou des mini-boss) : à intervalles réguliers (son
 * calendrier), annonce (si demandé) l'arrivée du prochain quelques secondes avant, puis le
 * fait apparaître hors du champ de la caméra. Les mini-boss ont les PV
 * multipliés comme les ennemis ordinaires à ce moment de la partie.
 */
import { APPARITIONS } from '../donnees/apparitions';
import { ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { PROGRESSION } from '../donnees/progression';
import type { IdSon } from '../donnees/sons';
import type { DefinitionArene, DefinitionCalendrierBoss } from '../donnees/types';
import type { Retours } from '../effets/retours';
import { minutesSansFin, type EtatPartie } from '../etat';
import type { Terrain } from '../monde/navigation';
import { longueur } from '../utils/maths';
import type { DetecteurVisibilite } from './directeur';
import type { Ennemis } from './ennemis';

/** Ce qui affiche l'alerte à l'écran. */
export interface AfficheurAlerte {
  alerter(texte: string, duree: number): void;
}

/** Ce qui distingue les boss des mini-boss. */
export interface ReglagesDirecteur {
  readonly calendrier: DefinitionCalendrierBoss;
  /** Ordre d'arrivée dans l'arène (puis en boucle). */
  readonly liste: (arene: DefinitionArene) => readonly IdEnnemi[];
  /** Texte de l'alerte avant l'apparition (null : arrive sans prévenir). */
  readonly annonce: ((nom: string) => string) | null;
  /** Distance minimale au joueur (blocs) ; son et secousse à l'apparition (null / 0 : aucun). */
  readonly distance: number;
  readonly son: IdSon | null;
  readonly secousse: number;
}

export class DirecteurBoss {
  private readonly etat: EtatPartie;
  private readonly reglages: ReglagesDirecteur;
  private readonly terrain: Terrain;
  private readonly ennemis: Ennemis;
  private readonly vue: DetecteurVisibilite;
  private readonly afficheur: AfficheurAlerte;
  private readonly retours: Retours;
  private prochain = 0;               // index dans l'ordre des boss
  private moment: number;
  private annonce = false;

  constructor(
    reglages: ReglagesDirecteur,
    etat: EtatPartie,
    terrain: Terrain,
    ennemis: Ennemis,
    vue: DetecteurVisibilite,
    afficheur: AfficheurAlerte,
    retours: Retours,
  ) {
    this.reglages = reglages;
    this.moment = reglages.calendrier.premierPassage;
    this.retours = retours;
    this.etat = etat;
    this.terrain = terrain;
    this.ennemis = ennemis;
    this.vue = vue;
    this.afficheur = afficheur;
  }

  /** Secondes avant le prochain boss, ou −1 s'il n'en viendra plus avant la fin de la partie. */
  get tempsAvantProchain(): number {
    if (this.termine()) return -1;
    return Math.max(0, this.moment - this.etat.tempsJeu);
  }

  /** Nom du prochain boss (HUD, alerte). */
  get prochainNom(): string {
    return ENNEMIS[this.idProchain()].nom;
  }

  maj(): void {
    // Aucun boss au-delà de la fin de partie (la victoire arrive avant), sauf en mode sans fin
    const c = this.reglages.calendrier;
    if (this.etat.joueur.mort || this.etat.issue !== null || this.termine()) return;
    if (this.reglages.liste(this.etat.arene).length === 0) return;
    const t = this.etat.tempsJeu;
    if (!this.annonce && t >= this.moment - c.avertissement) {
      this.annonce = true;
      if (this.reglages.annonce) this.afficheur.alerter(this.reglages.annonce(this.prochainNom), c.avertissement);
    }
    if (t >= this.moment) {
      this.faireApparaitre();
      this.moment += c.intervalle;
      this.annonce = false;
    }
  }

  /** Mode sans fin : le calendrier reprend, le prochain venant au plus tôt un intervalle après maintenant. */
  reprendre(): void {
    this.moment = Math.max(this.moment, this.etat.tempsJeu + this.reglages.calendrier.intervalle);
    this.annonce = false;
  }

  private termine(): boolean {
    return !this.etat.sansFin && this.moment >= PROGRESSION.dureePartie;
  }

  /** Fait apparaître le prochain boss tout de suite (debug), sans décaler le calendrier. */
  forcer(): void {
    this.afficheur.alerter(`${this.prochainNom} est là !`, 2);
    this.faireApparaitre();
  }

  private idProchain(): IdEnnemi {
    const ordre = this.reglages.liste(this.etat.arene);
    return ordre[this.prochain % ordre.length];
  }

  /** Apparition hors champ sur une case libre, sinon au coin opposé au joueur. */
  private faireApparaitre(): void {
    const id = this.idProchain();
    this.prochain++;
    const j = this.etat.joueur;
    const limiteX = this.etat.arene.largeur / 2 - ENNEMIS[id].rayon - 1;
    const limiteZ = this.etat.arene.profondeur / 2 - ENNEMIS[id].rayon - 1;
    let x = j.x > 0 ? -limiteX : limiteX;
    let z = j.z > 0 ? -limiteZ : limiteZ;
    for (let essai = 0; essai < 16; essai++) {
      const cx = (Math.random() * 2 - 1) * limiteX;
      const cz = (Math.random() * 2 - 1) * limiteZ;
      if (this.terrain.carte.bloqueMarche(cx, cz) || !this.terrain.flux.atteignable(cx, cz)) continue;
      if (longueur(cx - j.x, cz - j.z) > this.reglages.distance && !this.vue.estVisible(cx, cz, 3)) {
        x = cx;
        z = cz;
        break;
      }
    }
    const a = APPARITIONS;
    const montee = Math.min(this.etat.tempsJeu / a.dureeMontee, 1);
    const selonTemps = this.reglages.calendrier.pvSelonTemps ? a.multiplicateurPv.debut + (a.multiplicateurPv.fin - a.multiplicateurPv.debut) * montee : 1;
    const pv = selonTemps * (1 + minutesSansFin(this.etat) * PROGRESSION.sansFin.pvParMinute);
    this.ennemis.apparaitre(id, x, z, pv);
    if (this.reglages.son) this.retours.son(this.reglages.son);
    if (this.reglages.secousse > 0) this.retours.secousse(this.reglages.secousse);
  }
}
