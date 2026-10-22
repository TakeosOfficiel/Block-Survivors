/**
 * État d'une partie : source de vérité de la simulation.
 * Aucun objet three.js ni élément DOM ici.
 */
import type { BonusArmeCumule } from './combat/armeEquipee';
import { IDS_ARMES, type IdArme } from './donnees/armes';
import { IDS_ENNEMIS } from './donnees/ennemis';
import type { IdPersonnage } from './donnees/personnages';
import { PERSONNAGES } from './donnees/personnages';
import { PROGRESSION } from './donnees/progression';
import type { DefinitionArene, DefinitionPersonnage, Modificateurs } from './donnees/types';
import { creerModificateurs } from './progression/modificateurs';

export interface EtatJoueur {
  x: number;              // position au sol (blocs)
  z: number;
  vx: number;             // vitesse (blocs/s)
  vz: number;
  angle: number;          // orientation (rad), 0 = face vers +Z
  pv: number;
  pvMax: number;
  invulnerabilite: number;  // s restantes sans pouvoir être blessé (après un coup)
  protection: number;     // s d'invincibilité après une résurrection (totem)
  bouclier: number;       // PV d'absorption (cœurs dorés), pris avant les PV
  viesUtilisees: number;  // totems déjà consommés
  mort: boolean;
  tempsMort: number;      // s écoulées depuis la mort
  niveau: number;
  xp: number;             // XP accumulée dans le niveau en cours
  xpNiveau: number;       // XP nécessaire pour passer au niveau suivant
}

/** Bilan chiffré de la partie (écrans de pause et de fin). */
export interface StatistiquesPartie {
  /** Ennemis tués, par index dans IDS_ENNEMIS. */
  readonly tuesParType: Int32Array;
  /** Par index dans IDS_ARMES : dégâts infligés, coups portés, coups critiques, plus gros coup, ennemis achevés. */
  readonly degatsParArme: Float64Array;
  readonly coupsParArme: Int32Array;
  readonly critiquesParArme: Int32Array;
  readonly plusGrosCoupParArme: Float64Array;
  readonly tuesParArme: Int32Array;
  /** Temps de jeu où chaque arme a été obtenue (−1 = jamais), pour ses dégâts par seconde. */
  readonly acquisitionArme: Float64Array;
  degatsSubis: number;
  pvSoignes: number;
  xpTotale: number;
  emeraudesRamassees: number;
  coffresOuverts: number;
  bossVaincus: number;
}

/** Fin de partie : null tant qu'elle continue. */
export type IssuePartie = 'victoire' | 'defaite' | null;

export interface EtatPartie {
  readonly arene: DefinitionArene;
  readonly idPersonnage: IdPersonnage;
  readonly personnage: DefinitionPersonnage;
  tempsJeu: number;       // secondes de jeu écoulées (suivent la vitesse du temps)
  ennemisTues: number;
  /** Montées de niveau dont les cartes n'ont pas encore été choisies. */
  niveauxEnAttente: number;
  /** Relances des cartes encore disponibles. */
  relancesRestantes: number;
  /** Émeraudes ramassées dans la partie (elles ouvrent les coffres). */
  emeraudes: number;
  /** Cartes de coffre pas encore choisies, et titre de leur écran. */
  readonly choixCoffre: { restants: number; titre: string };
  issue: IssuePartie;
  readonly stats: StatistiquesPartie;
  /** Bonus cumulés des passifs. */
  readonly modificateurs: Modificateurs;
  /** Talents de l'arbre actif : bonus d'armes précises et mécaniques propres (avec leur rang). */
  talents: TalentsPartie;
  readonly joueur: EtatJoueur;
}

export interface TalentsPartie {
  readonly armes: Partial<Record<IdArme, BonusArmeCumule>>;
  readonly mecaniques: Readonly<Record<string, number>>;
}

export function creerEtatPartie(arene: DefinitionArene, idPersonnage: IdPersonnage): EtatPartie {
  const personnage = PERSONNAGES[idPersonnage];
  return {
    arene,
    idPersonnage,
    personnage,
    tempsJeu: 0,
    ennemisTues: 0,
    niveauxEnAttente: 0,
    relancesRestantes: PROGRESSION.relances,
    emeraudes: 0,
    choixCoffre: { restants: 0, titre: '' },
    issue: null,
    stats: {
      tuesParType: new Int32Array(IDS_ENNEMIS.length),
      degatsParArme: new Float64Array(IDS_ARMES.length),
      coupsParArme: new Int32Array(IDS_ARMES.length),
      critiquesParArme: new Int32Array(IDS_ARMES.length),
      plusGrosCoupParArme: new Float64Array(IDS_ARMES.length),
      tuesParArme: new Int32Array(IDS_ARMES.length),
      acquisitionArme: new Float64Array(IDS_ARMES.length).fill(-1),
      degatsSubis: 0,
      pvSoignes: 0,
      xpTotale: 0,
      bossVaincus: 0,
      emeraudesRamassees: 0,
      coffresOuverts: 0,
    },
    modificateurs: creerModificateurs(),
    talents: { armes: {}, mecaniques: {} },
    joueur: {
      x: arene.depart?.x ?? 0, z: arene.depart?.z ?? 0, vx: 0, vz: 0, angle: 0,
      pv: personnage.pv,
      pvMax: personnage.pv,
      invulnerabilite: 0,
      protection: 0,
      bouclier: 0,
      viesUtilisees: 0,
      mort: false,
      tempsMort: 0,
      niveau: 1,
      xp: 0,
      xpNiveau: PROGRESSION.xpPremierNiveau,
    },
  };
}
