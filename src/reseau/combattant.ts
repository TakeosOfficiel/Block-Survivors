/**
 * Hôte : un joueur distant et tout ce qui lui appartient (état, bonus, modèle,
 * arsenal, systèmes d'armes, talents, tirage des cartes), construit comme le
 * joueur local dans jeu.ts.
 *
 * Commutation (prototype) : les systèmes d'armes, les talents et le joueur
 * lisent `etat.joueur`, `etat.modificateurs`, `etat.talents` et
 * `etat.personnage`. Avant de mettre à jour un combattant, on fait pointer
 * ces champs sur les siens (`Commutateur.vers`), puis on remet ceux du joueur
 * local : aucun système de combat n'a besoin de savoir qu'il y a plusieurs
 * joueurs. Ce ne sont que des affectations de références (rien n'est alloué).
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { cumulerBonusArmes } from '../combat/armeEquipee';
import { Arsenal } from '../combat/arsenal';
import { Balayages } from '../combat/balayages';
import { Chutes } from '../combat/chutes';
import type { ContexteCombat } from '../combat/contexte';
import { Fissures } from '../combat/fissures';
import { Foudre } from '../combat/foudre';
import { Heurts } from '../combat/heurts';
import { Invocations } from '../combat/invocations';
import { Meteores } from '../combat/meteores';
import { Novas } from '../combat/novas';
import { OrbesArcaniques } from '../combat/orbesArcaniques';
import { Orbites } from '../combat/orbites';
import { Pieges } from '../combat/pieges';
import { Pluies } from '../combat/pluies';
import { ProjectilesJoueur } from '../combat/projectilesJoueur';
import { Ricochets } from '../combat/ricochets';
import { mecaniquesClasse } from '../combat/talents/classes';
import { MecaniquesObjets } from '../combat/talents/objets';
import { Talents } from '../combat/talents/talents';
import { Tornades } from '../combat/tornades';
import { Tourbillons } from '../combat/tourbillons';
import { CONFIG, type ActionDeplacement } from '../config';
import { PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { DefinitionPersonnage, Modificateurs } from '../donnees/types';
import { BarresVie } from '../effets/barresVie';
import { Etiquette } from '../effets/etiquette';
import type { Ombres } from '../effets/ombres';
import type { Particules } from '../effets/particules';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { Retours } from '../effets/retours';
import type { Ennemis } from '../entites/ennemis';
import { Joueur, type CommandesJoueur } from '../entites/joueur';
import type { Orbes } from '../entites/orbes';
import type { ProjectilesEnnemis } from '../entites/projectilesEnnemis';
import { creerEtatPartie, type EtatJoueur, type EtatPartie, type StatistiquesPartie, type TalentsPartie } from '../etat';
import type { CarteMonde } from '../monde/carte';
import { ChampDeFlux } from '../monde/navigation';
import { Ameliorations } from '../progression/ameliorations';
import { Equipement } from '../progression/equipement';
import { creerModificateurs, cumulerModificateurs } from '../progression/modificateurs';
import { Passifs } from '../progression/passifs';
import { BITS_ACTIONS, type CarteJouee, type InfoJoueur } from './protocole';
import { apparenceDistante, codeEtiquette, positionDepart, texteEtiquette } from './apparence';
import { IDS_ARMES } from '../donnees/armes';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { GERBES } from '../donnees/particules';

/** Ce qui change d'un joueur à l'autre dans l'état de la partie. */
export interface ProfilCombat {
  /** Rang dans l'équipe (0 : le joueur local). */
  rang: number;
  joueur: EtatJoueur;
  modificateurs: Modificateurs;
  talents: TalentsPartie;
  personnage: DefinitionPersonnage;
  idPersonnage: IdPersonnage;
  /** Talents branchés sur les coups portés aux ennemis. */
  mecaniques: Talents;
  /** Son bilan (dégâts par arme…) ; les chiffres de l'équipe (ennemis tués, XP, boss) sont communs. */
  stats: StatistiquesPartie;
}

/**
 * Bilan d'un joueur distant : ses propres dégâts, coups et soins, mais les
 * ennemis tués, l'XP, les émeraudes, les coffres et les boss restent ceux de
 * l'équipe (mêmes tableaux, valeurs lues et écrites dans `equipe`).
 */
function creerStatistiquesJoueur(equipe: StatistiquesPartie): StatistiquesPartie {
  const n = IDS_ARMES.length;
  const stats = {
    tuesParType: equipe.tuesParType,
    degatsParArme: new Float64Array(n),
    coupsParArme: new Int32Array(n),
    critiquesParArme: new Int32Array(n),
    plusGrosCoupParArme: new Float64Array(n),
    tuesParArme: new Int32Array(n),
    acquisitionArme: new Float64Array(n).fill(-1),
    degatsSubis: 0,
    pvSoignes: 0,
  } as StatistiquesPartie;
  for (const cle of ['xpTotale', 'emeraudesRamassees', 'coffresOuverts', 'bossVaincus'] as const) {
    Object.defineProperty(stats, cle, {
      get: () => equipe[cle],
      set: (v: number) => { equipe[cle] = v; },
      enumerable: true,
    });
  }
  return stats;
}

/** L'état de la partie, champs en lecture seule compris modifiables (commutation). */
type EtatCommutable = { -readonly [K in keyof EtatPartie]: EtatPartie[K] };

/** Fait pointer l'état de la partie sur un joueur (`vers`), puis le remet (`restaurer`). */
export class Commutateur {
  private readonly etat: EtatCommutable;
  private readonly ennemis: Ennemis;
  /** Joueur local (celui de jeu.ts). */
  readonly local: ProfilCombat;
  private actuel: ProfilCombat;
  /** Profils par rang dans l'équipe (0 : le joueur local). */
  private readonly profils: ProfilCombat[] = [];

  constructor(etat: EtatPartie, ennemis: Ennemis, mecaniquesLocales: Talents) {
    this.etat = etat as EtatCommutable;
    this.ennemis = ennemis;
    this.local = {
      rang: 0,
      joueur: etat.joueur, modificateurs: etat.modificateurs, talents: etat.talents,
      personnage: etat.personnage, idPersonnage: etat.idPersonnage, mecaniques: mecaniquesLocales, stats: etat.stats,
    };
    this.actuel = this.local;
    this.profils.push(this.local);
  }

  /** Rang du joueur qui agit en ce moment. */
  get actif(): number {
    return this.actuel.rang;
  }

  /** Bascule sur le joueur de rang `rang` ; renvoie le rang d'avant. */
  activer(rang: number): number {
    const avant = this.actuel.rang;
    this.vers(this.profils[rang] ?? this.local);
    return avant;
  }

  /** Inscrit un joueur de l'équipe (à son rang). */
  inscrire(profil: ProfilCombat): void {
    this.profils[profil.rang] = profil;
  }

  /** Bascule sur `profil` ; renvoie celui d'avant (à redonner à `vers` ensuite). */
  vers(profil: ProfilCombat): ProfilCombat {
    const avant = this.actuel;
    if (profil === avant) return avant;
    const e = this.etat;
    e.joueur = profil.joueur;
    e.modificateurs = profil.modificateurs;
    e.talents = profil.talents;
    e.personnage = profil.personnage;
    e.idPersonnage = profil.idPersonnage;
    e.stats = profil.stats;
    this.ennemis.definirTalents(profil.mecaniques);
    this.actuel = profil;
    return avant;
  }

  restaurer(): void {
    this.vers(this.local);
  }
}

/** Touches d'un joueur distant, d'après sa dernière entrée reçue. */
export class CommandesDistantes implements CommandesJoueur {
  bits = 0;
  action(nom: ActionDeplacement): boolean {
    return (this.bits & BITS_ACTIONS[nom]) !== 0;
  }
}

/** Ce que la partie partage avec les combattants. */
export interface PartageCombat {
  readonly etat: EtatPartie;
  readonly carte: CarteMonde;
  readonly ennemis: Ennemis;
  readonly projectilesEnnemis: ProjectilesEnnemis;
  readonly orbes: Orbes;
  readonly particules: Particules;
  readonly scene: THREE.Scene;
  readonly camera: THREE.Camera;
  readonly textures: BibliothequeTextures;
  readonly commutateur: Commutateur;
}

export class Combattant {
  readonly info: InfoJoueur;
  readonly profil: ProfilCombat;
  readonly commandes = new CommandesDistantes();
  readonly joueur: Joueur;
  readonly flux: ChampDeFlux;
  readonly etiquette: Etiquette;
  readonly arsenal: Arsenal;
  readonly passifs: Passifs;
  readonly ameliorations: Ameliorations;
  readonly equipement: Equipement;
  /** Hôte : cartes (niveaux, coffres) qu'il n'a pas encore choisies. */
  cartesEnAttente = 0;
  /** Choisit ses cartes : protégé, ignoré des ennemis, ses armes au repos. */
  enChoix = false;
  /** À terre : progression de sa réanimation (0 à 1). */
  reanimation = 0;
  /** Hôte : s passées à choisir ses cartes. */
  tempsChoix = 0;
  private etiquetteAffichee = -1;
  private minuterieBouclier = 0;
  /** Numéro de sa dernière entrée reçue, et quand (temps réel, s). */
  derniereEntree = 0;
  instantEntree = 0;
  /** Parti en cours de partie : il ne compte plus. */
  parti = false;

  private readonly partage: PartageCombat;
  private readonly balayages: Balayages;
  private readonly projectiles: ProjectilesJoueur;
  private readonly orbites: Orbites;
  private readonly foudre: Foudre;
  private readonly invocations: Invocations;
  private readonly fissures: Fissures;
  private readonly chutes: Chutes;
  private readonly tourbillons: Tourbillons;
  private readonly heurts: Heurts;
  private readonly tornades: Tornades;
  private readonly novas: Novas;
  private readonly meteores: Meteores;
  private readonly orbesArcaniques: OrbesArcaniques;
  private readonly ricochets: Ricochets;
  private readonly pluies: Pluies;
  private readonly pieges: Pieges;

  /** `rang` : place dans l'équipe (1 à 3 ; 0 est le joueur local). */
  constructor(info: InfoJoueur, rang: number, partage: PartageCombat, retours: Retours) {
    this.info = info;
    const bonusTalents = info.bonus;
    this.partage = partage;
    const { etat, carte, ennemis, scene, textures } = partage;
    const personnage = PERSONNAGES[info.personnage];

    // Son état, ses bonus permanents (sauvegarde multi du joueur) et ses talents, comme dans jeu.ts
    const joueur = creerEtatPartie(etat.arene, info.personnage).joueur;
    positionDepart(etat.arene, info.index, joueur);
    const modificateurs = creerModificateurs();
    const bonus = { ...personnage.bonus };
    cumulerModificateurs(bonus, bonusTalents.modif);
    // Armure de sa Collection (pièces et ensemble)
    for (const modif of info.armure.bonus) cumulerModificateurs(bonus, modif);
    this.passifs = new Passifs(bonus);
    this.passifs.calculer(modificateurs);
    joueur.pvMax += modificateurs.pvMax;
    joueur.pv = joueur.pvMax;
    const talents: TalentsPartie = { armes: cumulerBonusArmes(bonusTalents.bonusArmes), mecaniques: bonusTalents.mecaniques };
    const commutateur = partage.commutateur;
    // Les mécaniques sont créées une fois l'état commuté (leur constructeur le lit)
    const provisoire: ProfilCombat = {
      rang,
      joueur, modificateurs, talents, personnage, idPersonnage: info.personnage, mecaniques: commutateur.local.mecaniques,
      stats: creerStatistiquesJoueur(commutateur.local.stats),
    };
    const avant = commutateur.vers(provisoire);
    const mecaniques = new Talents(etat);
    this.profil = { ...provisoire, mecaniques };
    commutateur.inscrire(this.profil);
    commutateur.vers(this.profil);

    this.joueur = new Joueur(etat, carte, this.commandes, retours, scene, textures, apparenceDistante(info), info.armure.portees);
    this.joueur.talents = mecaniques;
    this.flux = new ChampDeFlux(carte);
    this.etiquette = new Etiquette(scene, info.pseudo);

    const combat: ContexteCombat = {
      etat,
      carte,
      ennemis,
      particules: partage.particules,
      retours,
      talents: mecaniques,
      declencherGesteJoueur: (angle) => this.joueur.declencherGeste(angle),
    };
    this.balayages = new Balayages(combat, textures, scene);
    this.projectiles = new ProjectilesJoueur(combat);
    this.orbites = new Orbites(combat);
    this.foudre = new Foudre(combat, scene);
    this.invocations = new Invocations(combat, scene, textures, new BarresVie(scene, partage.camera, CONFIG.armes.golems.capacite), this.projectiles);
    this.fissures = new Fissures(combat, textures, scene);
    this.chutes = new Chutes(combat, textures, scene);
    this.tourbillons = new Tourbillons(combat, textures, scene);
    this.heurts = new Heurts(combat, partage.projectilesEnnemis);
    this.tornades = new Tornades(combat, textures, scene);
    this.novas = new Novas(combat, textures, scene);
    this.meteores = new Meteores(combat, textures, scene);
    this.orbesArcaniques = new OrbesArcaniques(combat, textures, scene);
    this.ricochets = new Ricochets(combat);
    this.pluies = new Pluies(combat, textures, scene);
    this.pieges = new Pieges(combat, textures, scene);
    this.arsenal = new Arsenal(combat, {
      balayages: this.balayages,
      projectiles: this.projectiles,
      orbites: this.orbites,
      foudre: this.foudre,
      invocations: this.invocations,
      fissures: this.fissures,
      chutes: this.chutes,
      tourbillons: this.tourbillons,
      heurts: this.heurts,
      tornades: this.tornades,
      novas: this.novas,
      meteores: this.meteores,
      orbesArcaniques: this.orbesArcaniques,
      ricochets: this.ricochets,
      pluies: this.pluies,
      pieges: this.pieges,
    });
    for (const id of personnage.armesDepart) this.arsenal.ameliorer(id);
    for (const id of bonusTalents.armes) this.arsenal.ameliorer(id);
    mecaniques.brancher(combat, this.arsenal, { scene, textures, orbes: partage.orbes }, mecaniquesClasse(info.personnage));
    this.ameliorations = new Ameliorations(etat, this.arsenal, this.passifs);
    // Équipement des coffres : ses bonus passent par ses passifs, ses effets par ses talents (découvertes : chez lui)
    this.equipement = new Equipement(etat, this.passifs, this.arsenal, () => {});
    mecaniques.brancherObjets(new MecaniquesObjets(mecaniques, this.equipement));
    commutateur.vers(avant);
  }

  get etatJoueur(): EtatJoueur {
    return this.profil.joueur;
  }

  /** Marche (avant les ennemis). */
  majJoueur(dt: number): void {
    const avant = this.partage.commutateur.vers(this.profil);
    this.joueur.maj(dt);
    this.partage.commutateur.vers(avant);
    this.flux.maj(this.etatJoueur.x, this.etatJoueur.z);
  }

  /** Armes et talents (après les ennemis, qui ont reconstruit leur grille). */
  majArmes(dt: number): void {
    const avant = this.partage.commutateur.vers(this.profil);
    this.arsenal.maj(dt);
    this.profil.mecaniques.maj(dt);
    this.projectiles.maj(dt);
    this.invocations.maj(dt);
    this.foudre.maj(dt);
    this.fissures.maj(dt);
    this.chutes.maj(dt);
    this.tourbillons.maj(dt);
    this.tornades.maj(dt);
    this.novas.maj(dt);
    this.meteores.maj(dt);
    this.orbesArcaniques.maj(dt);
    this.ricochets.maj(dt);
    this.pluies.maj(dt);
    this.pieges.maj(dt);
    this.partage.commutateur.vers(avant);
  }

  /** Un coup reçu (ennemi, flèche, explosion), au nom de ce joueur. */
  subirDegats(degats: number, boss?: boolean): boolean {
    const avant = this.partage.commutateur.vers(this.profil);
    const porte = this.joueur.subirDegats(degats, boss);
    this.partage.commutateur.vers(avant);
    return porte;
  }

  /** Carte choisie par le joueur (montée de niveau ou coffre). */
  appliquerCarte(carte: CarteJouee): void {
    const avant = this.partage.commutateur.vers(this.profil);
    if (carte.type === 'arme' || carte.type === 'eveil' || carte.type === 'passif' || carte.type === 'soin') this.ameliorations.appliquer(carte);
    else this.equipement.appliquer(carte);
    this.partage.commutateur.vers(avant);
  }

  // --- Affichage ----------------------------------------------------------

  /** Modèle, étiquette (état du joueur), bouclier pendant le choix des cartes ; `effets` : ses armes aussi. */
  majVisuelOmbres(ombres: Ombres, dtReel: number, effets: boolean): void {
    const avant = this.partage.commutateur.vers(this.profil);
    this.joueur.majVisuel(ombres);
    const j = this.etatJoueur;
    const code = codeEtiquette(j, this.enChoix, this.reanimation);
    if (code !== this.etiquetteAffichee) {
      this.etiquetteAffichee = code;
      this.etiquette.ecrire(texteEtiquette(this.info.pseudo, code), j.mort ? '#ff8080' : this.enChoix ? '#ffd75e' : '#ffffff');
    }
    this.etiquette.placer(j.x, this.joueur.hauteur, j.z, !j.mort || j.tempsMort < REGLES_MULTIJOUEUR.reanimation.aTerre);
    if (this.enChoix && (this.minuterieBouclier -= dtReel) <= 0) {
      this.minuterieBouclier = REGLES_MULTIJOUEUR.choixCartes.periodeBouclier;
      this.partage.particules.emettre(j.x, 1, j.z, GERBES.arcane);
    }
    if (!effets) {
      this.partage.commutateur.vers(avant);
      return;
    }
    this.invocations.majVisuel(ombres);
    this.profil.mecaniques.majVisuel(ombres);
    this.chutes.majVisuel(ombres);
    this.meteores.majVisuel(ombres);
    this.partage.commutateur.vers(avant);
  }

  majVisuelProjectiles(rendu: RenduProjectiles): void {
    const avant = this.partage.commutateur.vers(this.profil);
    const temps = this.partage.etat.tempsJeu;
    this.projectiles.majVisuel(rendu, temps);
    this.ricochets.majVisuel(rendu);
    this.orbites.majVisuel(this.arsenal.armes, rendu, temps);
    this.partage.commutateur.vers(avant);
  }

  majVisuelEffets(dtJeu: number): void {
    const avant = this.partage.commutateur.vers(this.profil);
    this.balayages.majVisuel(dtJeu);
    this.foudre.majVisuel();
    this.fissures.majVisuel();
    this.tourbillons.majVisuel();
    this.tornades.majVisuel();
    this.novas.majVisuel();
    this.orbesArcaniques.majVisuel();
    this.pluies.majVisuel();
    this.pieges.majVisuel();
    this.partage.commutateur.vers(avant);
  }
}
