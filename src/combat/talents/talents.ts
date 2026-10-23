/**
 * Mécaniques propres des talents : le socle reçoit les événements de la
 * partie (arme déclenchée, coup porté, ennemi tué, choc contre un mur, coup
 * reçu par le joueur) et les transmet aux mécaniques de la classe jouée.
 * Les systèmes d'armes lisent aussi directement le rang d'un talent qui les
 * modifie (`rang`), par exemple les Répliques de l'Onde de choc.
 *
 * Les dégâts causés par un talent sont des « réactions » : ils ne relancent
 * pas les talents qui comptent les coups (pas de boucle).
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../../assets/textures';
import { CONFIG } from '../../config';
import { ARMES, IDS_ARMES, type IdArme } from '../../donnees/armes';
import { ETATS } from '../../donnees/etats';
import { GERBES, type IdGerbe } from '../../donnees/particules';
import { PLAFONDS } from '../../donnees/plafonds';
import type { DefinitionAllie, EtatEveil, EveilArme, FamilleArme } from '../../donnees/types';
import type { Ombres } from '../../effets/ombres';
import type { Ennemi } from '../../entites/ennemi';
import type { CrochetsTalents, NatureCoup } from '../../entites/ennemis';
import type { CrochetsJoueur } from '../../entites/types';
import type { EtatPartie } from '../../etat';
import type { ArmeEquipee } from '../armeEquipee';
import type { Arsenal, SystemesCombat } from '../arsenal';
import type { ContexteCombat } from '../contexte';
import type { Allie } from '../invocations';
import { ajouterBouclier } from '../defense';
import type { MecaniquesObjets } from './objets';

/** Famille de chaque arme, par index dans IDS_ARMES. */
const FAMILLES: readonly (FamilleArme | undefined)[] = IDS_ARMES.map((id) => ARMES[id].famille);

export function indexArme(id: IdArme): number {
  return IDS_ARMES.indexOf(id);
}

/**
 * Mécaniques d'une classe : chaque crochet est neutre par défaut ; une classe
 * ne redéfinit que ceux dont ses talents ont besoin.
 */
export class MecaniquesClasse {
  protected readonly t: Talents;

  constructor(talents: Talents) {
    this.t = talents;
  }

  maj(_dt: number): void {}
  /** Juste avant que l'arme se déclenche : peut renforcer ses statistiques pour ce coup. */
  avantDeclenchement(_arme: ArmeEquipee): void {}
  /** Juste après (`declenchee` : elle avait une cible) : remet ses statistiques. */
  apresDeclenchement(_arme: ArmeEquipee, _declenchee: boolean): void {}
  /** Multiplicateur du délai avant le prochain déclenchement de l'arme. */
  delai(_arme: ArmeEquipee): number {
    return 1;
  }
  multiplicateur(_e: Ennemi, _arme: number, _nature: NatureCoup): number {
    return 1;
  }
  bonusCritique(_e: Ennemi, _arme: number, _nature: NatureCoup): number {
    return 0;
  }
  facteurCritique(_e: Ennemi, _arme: number, _nature: NatureCoup, base: number): number {
    return base;
  }
  /** Coup porté par une arme ou un allié (pas une réaction de talent). */
  apresCoup(_e: Ennemi, _degats: number, _arme: number, _critique: boolean, _nature: NatureCoup): void {}
  surMort(_e: Ennemi, _arme: number, _nature: NatureCoup): void {}
  surImpactMur(_e: Ennemi): void {}
  surSortiePiege(_e: Ennemi): void {}
  /** Cumuls de poison au maximum (Venin concentré ou virulent). */
  cumulsPoison(): number {
    return ETATS.poison.cumulsMax;
  }
  vitesse(): number {
    return 1;
  }
  degatsRecus(degats: number, _boss: boolean): number {
    return degats;
  }
  armure(): number {
    return 0;
  }
  /** `evites` : dégâts retirés par l'armure et absorbés par le bouclier. */
  apresCoupRecu(_subis: number, _evites: number): void {}
  /** À 0 PV, avant le totem : vrai si un talent sauve le joueur (il remet lui-même ses PV). */
  eviterMort(): boolean {
    return false;
  }

  // Alliés invoqués (Invocateur)
  /** Places en plus dans le quota de ce type d'allié. */
  quotaAllie(_def: DefinitionAllie): number {
    return 0;
  }
  surCreationAllie(_a: Allie): void {}
  /** Cible à viser d'abord (ennemi marqué), ou null pour la plus proche. */
  ciblePrioritaire(_a: Allie): Ennemi | null {
    return null;
  }
  multiplicateurAllie(_a: Allie, _e: Ennemi): number {
    return 1;
  }
  /** Multiplicateur du délai entre deux coups de l'allié. */
  cadenceAllie(_a: Allie): number {
    return 1;
  }
  vitesseAllie(_a: Allie): number {
    return 1;
  }
  /** Coup au corps à corps d'un allié (morsure, charge). */
  surMorsure(_a: Allie, _e: Ennemi): void {}
  /** L'allié tombe à 0 PV : vrai s'il revient (il remet lui-même ses PV). */
  surMortAllie(_a: Allie): boolean {
    return false;
  }
  /** Dessin des créatures des talents (ravageur, crocs). */
  majVisuel(_ombres: Ombres): void {}
}

/** Ce que les talents qui ont leur propre rendu (ravageur, crocs) ou qui ramassent l'XP utilisent. */
export interface RessourcesTalents {
  readonly scene: THREE.Scene;
  readonly textures: BibliothequeTextures;
  readonly orbes: { aimanter(x: number, z: number, rayon: number): void };
}

export class Talents implements CrochetsTalents, CrochetsJoueur {
  readonly etat: EtatPartie;
  private readonly rangs: Readonly<Record<string, number>>;
  private classe: MecaniquesClasse;
  /** Effets propres de l'équipement, après ceux de la classe (voir objets.ts). */
  private objets: MecaniquesObjets | null = null;
  private contexteCombat: ContexteCombat | null = null;
  private ressourcesPartie: RessourcesTalents | null = null;
  private arsenalPartie: Arsenal | null = null;
  /** Dégâts causés par un talent en cours : ils ne relancent pas les talents. */
  private enReaction = false;
  /** État infligé par l'éveil de chaque arme (index dans IDS_ARMES), ou null. */
  private readonly etatsEveils: (EtatEveil | null)[] = IDS_ARMES.map(() => null);
  // Mouvements du joueur, pour les talents qui en dépendent
  /** s depuis le dernier pas, et s de marche continue. */
  tempsImmobile = 0;
  tempsMouvement = 0;
  /** Blocs parcourus depuis le début de la partie. */
  parcouru = 0;
  private dernierX = 0;
  private dernierZ = 0;
  /** Soins des talents déjà rendus dans la seconde (plafond de régénération). */
  private soinsSeconde = 0;
  private minuterieSoins = 1;
  // Ordres des talents aux systèmes d'armes, lus au déclenchement
  /** La prochaine Onde de choc part dans 6 directions (Séisme). */
  ondeCirculaire = false;
  /** Dégâts de la flèche spectrale géante que tire ce coup de l'Arc (Puissance ; 0 : aucune), et sa portée. */
  flecheGeante = 0;
  porteeFlecheGeante = 0;
  /** Cette volée part aussi vers l'arrière (Vent arrière). */
  voleeArriere = false;
  /** Part des dégâts du déclenchement en cours (Écho arcanique : 0,6), captée au lancer par les systèmes. */
  facteurDeclenchement = 1;
  /** La Nova en cours de lancer gèle (Zéro absolu). */
  novaGel = false;

  constructor(etat: EtatPartie) {
    this.etat = etat;
    this.rangs = etat.talents.mecaniques;
    this.classe = new MecaniquesClasse(this);
    this.dernierX = etat.joueur.x;
    this.dernierZ = etat.joueur.z;
  }

  /** Branche le combat et les mécaniques de la classe (une fois les systèmes créés). */
  get systemes(): SystemesCombat {
    return this.arsenal.systemes;
  }

  get ressources(): RessourcesTalents {
    if (!this.ressourcesPartie) throw new Error('Talents non branchés');
    return this.ressourcesPartie;
  }

  brancher(contexte: ContexteCombat, arsenal: Arsenal, ressources: RessourcesTalents, classe: (t: Talents) => MecaniquesClasse): void {
    this.contexteCombat = contexte;
    this.arsenalPartie = arsenal;
    this.ressourcesPartie = ressources;
    this.classe = classe(this);
  }

  brancherObjets(objets: MecaniquesObjets): void {
    this.objets = objets;
  }

  get contexte(): ContexteCombat {
    if (!this.contexteCombat) throw new Error('Talents non branchés');
    return this.contexteCombat;
  }

  get arsenal(): Arsenal {
    if (!this.arsenalPartie) throw new Error('Talents non branchés');
    return this.arsenalPartie;
  }

  /** Rang du talent à mécanique dans l'arbre actif (0 : absent ou inactif). */
  rang(id: string): number {
    return this.rangs[id] ?? 0;
  }

  /** Valeur selon le rang : `valeurs[rang − 1]`, 0 sans le talent. */
  valeur(id: string, valeurs: readonly number[]): number {
    const r = this.rang(id);
    return r > 0 ? valeurs[Math.min(r, valeurs.length) - 1] : 0;
  }

  famille(arme: number): FamilleArme | undefined {
    return arme >= 0 ? FAMILLES[arme] : undefined;
  }

  // --- Outils des mécaniques ------------------------------------------------

  /** Dégâts de zone causés par un talent (réaction : ne relance pas les talents). */
  blesserZone(x: number, z: number, rayon: number, degats: number, arme: number, recul = 0, tampon = TAMPON): number {
    const ennemis = this.contexte.ennemis;
    const n = ennemis.chercher(x, z, rayon, tampon);
    const avant = this.enReaction;
    this.enReaction = true;
    for (let k = 0; k < n; k++) ennemis.blesser(ennemis.obtenir(tampon[k]), degats, x, z, recul, arme);
    this.enReaction = avant;
    return n;
  }

  /** Un coup causé par un talent (réaction). */
  blesser(e: Ennemi, degats: number, sourceX: number, sourceZ: number, recul: number, arme: number, nature: NatureCoup = 'arme'): void {
    const avant = this.enReaction;
    this.enReaction = true;
    this.contexte.ennemis.blesser(e, degats, sourceX, sourceZ, recul, arme, nature);
    this.enReaction = avant;
  }

  get reaction(): boolean {
    return this.enReaction;
  }

  /** Cercle de particules au sol (anneau, onde). */
  cercle(x: number, z: number, rayon: number, gerbe: IdGerbe = 'anneau', points = 24): void {
    const particules = this.contexte.particules;
    for (let k = 0; k < points; k++) {
      const a = (k / points) * Math.PI * 2;
      particules.emettre(x + Math.sin(a) * rayon, 0.3, z + Math.cos(a) * rayon, GERBES[gerbe]);
    }
  }

  /** Soigne le joueur (stats comprises). */
  soigner(pv: number): void {
    const j = this.etat.joueur;
    if (j.mort || pv <= 0) return;
    const avant = j.pv;
    j.pv = Math.min(j.pvMax, j.pv + pv);
    this.etat.stats.pvSoignes += j.pv - avant;
  }

  /** Soin limité par le plafond de régénération (par seconde, régénération comprise). */
  soignerPlafonne(pv: number): void {
    const j = this.etat.joueur;
    const mods = this.etat.modificateurs;
    const regen = mods.regeneration + mods.regenerationPart * j.pvMax;
    const reste = Math.max(0, PLAFONDS.regeneration * j.pvMax - regen - this.soinsSeconde);
    const soin = Math.min(pv, reste);
    this.soinsSeconde += soin;
    this.soigner(soin);
  }

  bouclier(pv: number): void {
    ajouterBouclier(this.etat.joueur, pv);
  }

  // --- Événements (arsenal, ennemis, joueur) --------------------------------

  maj(dt: number): void {
    const j = this.etat.joueur;
    const pas = Math.hypot(j.x - this.dernierX, j.z - this.dernierZ);
    this.dernierX = j.x;
    this.dernierZ = j.z;
    this.parcouru += pas;
    if (pas > CONFIG.talents.seuilMouvement * dt) {
      this.tempsMouvement += dt;
      this.tempsImmobile = 0;
    } else {
      this.tempsImmobile += dt;
      this.tempsMouvement = 0;
    }
    if ((this.minuterieSoins -= dt) <= 0) {
      this.minuterieSoins += 1;
      this.soinsSeconde = 0;
    }
    if (!j.mort && this.etat.issue === null) {
      this.classe.maj(dt);
      this.objets?.maj(dt);
    }
  }

  avantDeclenchement(arme: ArmeEquipee): void {
    this.classe.avantDeclenchement(arme);
  }

  apresDeclenchement(arme: ArmeEquipee, declenchee: boolean): void {
    this.classe.apresDeclenchement(arme, declenchee);
  }

  /** Multiplicateur du délai de l'arme (au mieux −50 %). */
  delai(arme: ArmeEquipee): number {
    return Math.max(PLAFONDS.delaiMin, this.classe.delai(arme));
  }

  multiplicateur(e: Ennemi, arme: number, nature: NatureCoup): number {
    const m = this.classe.multiplicateur(e, arme, nature);
    return this.objets ? m * this.objets.multiplicateur(e, arme, nature) : m;
  }

  bonusCritique(e: Ennemi, arme: number, nature: NatureCoup): number {
    return this.classe.bonusCritique(e, arme, nature);
  }

  facteurCritique(e: Ennemi, arme: number, nature: NatureCoup, base: number): number {
    return this.classe.facteurCritique(e, arme, nature, base);
  }

  apresCoup(e: Ennemi, degats: number, arme: number, critique: boolean, nature: NatureCoup): void {
    if (this.enReaction) return;
    // État de l'éveil de l'arme (Aura de feu, Haches givrées…)
    const etat = arme >= 0 && (nature === 'arme' || nature === 'allie') && e.mort < 0 ? this.etatsEveils[arme] : null;
    if (etat) {
      const ennemis = this.contexte.ennemis;
      switch (etat.type) {
        case 'brulure': ennemis.bruler(e, 1, etat.valeur, etat.duree, arme); break;
        case 'poison': ennemis.empoisonner(e, 1, etat.valeur, etat.duree, arme); break;
        case 'ralenti': ennemis.ralentir(e, etat.valeur, etat.duree); break;
        case 'etourdi': ennemis.etourdir(e, etat.valeur); break;
      }
    }
    this.classe.apresCoup(e, degats, arme, critique, nature);
    this.objets?.apresCoup(e, degats, arme, critique, nature);
  }

  /** L'arme d'index `arme` vient d'être éveillée. */
  definirEveil(arme: number, eveil: EveilArme): void {
    this.etatsEveils[arme] = eveil.etat ?? null;
  }

  surMort(e: Ennemi, arme: number, nature: NatureCoup): void {
    this.classe.surMort(e, arme, nature);
    this.objets?.surMort(e);
  }

  surImpactMur(e: Ennemi): void {
    this.classe.surImpactMur(e);
  }

  surSortiePiege(e: Ennemi): void {
    this.classe.surSortiePiege(e);
  }

  cumulsPoison(): number {
    return this.classe.cumulsPoison();
  }

  vitesse(): number {
    return this.classe.vitesse();
  }

  degatsRecus(degats: number, boss: boolean): number {
    return this.classe.degatsRecus(degats, boss);
  }

  armure(): number {
    return this.classe.armure();
  }

  apresCoupRecu(subis: number, evites: number): void {
    this.classe.apresCoupRecu(subis, evites);
    this.objets?.apresCoupRecu(subis);
  }

  eviterMort(): boolean {
    return this.classe.eviterMort();
  }

  quotaAllie(def: DefinitionAllie): number {
    return this.classe.quotaAllie(def);
  }

  surCreationAllie(a: Allie): void {
    this.classe.surCreationAllie(a);
  }

  ciblePrioritaire(a: Allie): Ennemi | null {
    return this.classe.ciblePrioritaire(a);
  }

  multiplicateurAllie(a: Allie, e: Ennemi): number {
    return this.classe.multiplicateurAllie(a, e);
  }

  cadenceAllie(a: Allie): number {
    return this.classe.cadenceAllie(a);
  }

  vitesseAllie(a: Allie): number {
    return this.classe.vitesseAllie(a);
  }

  surMorsure(a: Allie, e: Ennemi): void {
    this.classe.surMorsure(a, e);
  }

  surMortAllie(a: Allie): boolean {
    return this.classe.surMortAllie(a);
  }

  majVisuel(ombres: Ombres): void {
    this.classe.majVisuel(ombres);
  }
}

const TAMPON = new Int32Array(256);
