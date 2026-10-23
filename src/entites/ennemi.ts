/** Données d'un ennemi actif (objet du pool, réinitialisé à chaque apparition). */
import { IDS_ARMES } from '../donnees/armes';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import type { DefinitionEnnemi } from '../donnees/types';
import type { Allie } from '../combat/invocations';
import { creerEtatAnimation, type EtatAnimation } from '../modeles/animation';

/** Brûlure ou poison : cumuls, dégâts par cumul et par seconde, temps restant. */
export interface DegatsPeriodiques {
  cumuls: number;
  degatsParSeconde: number;
  temps: number;                // s restantes (remises à neuf à chaque application)
  tic: number;                  // s avant les prochains dégâts
  arme: number;                 // index dans IDS_ARMES (−1 hors armes), crédité des dégâts
  source: number;               // multijoueur : joueur de l'équipe qui l'a infligé (0 en solo)
}

export interface Ennemi {
  id: number;                   // identifiant unique de l'apparition (les index du pool changent)
  indexType: number;            // index dans IDS_ENNEMIS
  definition: DefinitionEnnemi;
  x: number;
  z: number;
  /** Recul (blocs/s), amorti avec le temps. */
  vx: number;
  vz: number;
  /** Direction voulue par le comportement (longueur 0 ou 1). */
  dirX: number;
  dirZ: number;
  angle: number;
  vitesseSol: number;           // pour l'animation
  pv: number;
  pvMax: number;
  minuterie: number;            // s avant la prochaine attaque
  blessure: number;             // s de surcouche rouge restantes
  meche: number;                // s de mèche écoulées (creeper)
  mort: number;                 // −1 = vivant, sinon s depuis la mort
  aExploser: boolean;
  /** Ligne de vue directe vers le joueur (sinon : suit le champ de flux). */
  voitJoueur: boolean;
  /** Tireur : aucun bloc n'arrête son tir vers le joueur (sinon il se déplace au lieu de tirer). */
  ligneDeTir: boolean;
  minuterieVue: number;         // s avant de revérifier la ligne de vue
  /** Allié invoqué qui l'a attiré (corps à corps), et son identifiant pour le reconnaître. */
  cibleAllie: Allie | null;
  cibleAllieId: number;
  /** Par arme (index dans IDS_ARMES) : temps de jeu avant lequel elle ne peut plus le toucher. */
  immunites: Float32Array;
  // États (etats.ts)
  brulure: DegatsPeriodiques;
  poison: DegatsPeriodiques;
  saignement: DegatsPeriodiques;
  ralenti: number;              // part de vitesse perdue (0 à 1)
  ralentiTemps: number;         // s restantes
  enracine: number;             // s restantes sans pouvoir marcher
  etourdi: number;              // s restantes sans marcher ni attaquer
  marque: number;               // dégâts en plus subis des alliés (0,15 = +15 %)
  marqueTemps: number;
  /** s avant les prochaines particules d'état. */
  minuterieEtats: number;
  // Talents (voir combat/talents) : marque de l'Archère, cumuls contre un boss, piège, morsures
  marqueChasseur: number;       // s restantes
  cumulsTalent: number;
  tempsCumuls: number;          // s restantes
  /** Temps de jeu jusqu'auquel l'ennemi compte comme pris dans un piège. */
  piege: number;
  dansPiege: boolean;
  /** Deux derniers loups qui l'ont mordu (id d'invocation) et quand (temps de jeu). */
  morsureA: number;
  morsureB: number;
  tempsMorsureA: number;
  tempsMorsureB: number;
  encercle: number;             // s restantes de vulnérabilité (Encerclement)
  animation: EtatAnimation;
}

function creerDegatsPeriodiques(): DegatsPeriodiques {
  return { cumuls: 0, degatsParSeconde: 0, temps: 0, tic: 0, arme: -1, source: 0 };
}

/** Efface tous les états (apparition). */
export function effacerEtats(e: Ennemi): void {
  e.brulure.cumuls = 0;
  e.poison.cumuls = 0;
  e.saignement.cumuls = 0;
  e.marqueChasseur = 0;
  e.cumulsTalent = 0;
  e.tempsCumuls = 0;
  e.piege = 0;
  e.dansPiege = false;
  e.morsureA = 0;
  e.morsureB = 0;
  e.tempsMorsureA = -1;
  e.tempsMorsureB = -1;
  e.encercle = 0;
  e.ralenti = 0;
  e.ralentiTemps = 0;
  e.enracine = 0;
  e.etourdi = 0;
  e.marque = 0;
  e.marqueTemps = 0;
  e.minuterieEtats = 0;
}

export function creerEnnemi(): Ennemi {
  return {
    id: 0,
    indexType: 0,
    definition: ENNEMIS[IDS_ENNEMIS[0]],
    x: 0, z: 0, vx: 0, vz: 0, dirX: 0, dirZ: 0,
    angle: 0, vitesseSol: 0,
    pv: 1, pvMax: 1,
    minuterie: 0, blessure: 0, meche: 0, mort: -1,
    aExploser: false,
    voitJoueur: true,
    ligneDeTir: true,
    minuterieVue: 0,
    cibleAllie: null,
    cibleAllieId: 0,
    immunites: new Float32Array(IDS_ARMES.length),
    brulure: creerDegatsPeriodiques(),
    poison: creerDegatsPeriodiques(),
    saignement: creerDegatsPeriodiques(),
    marqueChasseur: 0, cumulsTalent: 0, tempsCumuls: 0, piege: 0, dansPiege: false,
    morsureA: 0, morsureB: 0, tempsMorsureA: -1, tempsMorsureB: -1, encercle: 0,
    ralenti: 0, ralentiTemps: 0, enracine: 0, etourdi: 0, marque: 0, marqueTemps: 0, minuterieEtats: 0,
    animation: creerEtatAnimation(),
  };
}
