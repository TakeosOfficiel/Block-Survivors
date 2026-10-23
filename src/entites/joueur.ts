/** Joueur : déplacement, collisions avec les bords, vie, modèle animé du personnage choisi. */
import { ArmureHeros } from '../modeles/armure';
import type { IdMonde, PieceArmure } from '../donnees/types';
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { degatsApresDefense } from '../combat/defense';
import { CONFIG, type ActionDeplacement } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import { PLAFONDS } from '../donnees/plafonds';
import type { Ombres } from '../effets/ombres';
import type { Retours } from '../effets/retours';
import type { ApparenceEntite } from '../donnees/types';
import type { EtatPartie } from '../etat';
import type { CarteMonde, Point } from '../monde/carte';
import { angleChute, avancerAnimation, calculerRotations, creerEtatAnimation, declencherAttaque } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { ecartAngle, facteurLissage, longueur } from '../utils/maths';
import type { CibleJoueur, CrochetsJoueur, SourceEntites } from './types';

const corrige: Point = { x: 0, z: 0 };

/** Ce qui commande la marche : le clavier, ou les entrées reçues d'un joueur distant (multijoueur). */
export interface CommandesJoueur {
  action(nom: ActionDeplacement): boolean;
}

export class Joueur implements SourceEntites, CibleJoueur {
  readonly actifs = 1;
  /** Ignore tous les dégâts (debug). */
  invincible = false;
  /** Lacet de la caméra (rad) qui oriente ZQSD ; 0 dans la vue de dessus. */
  lacetCommandes = 0;
  /** Talents branchés sur le joueur (après la création des systèmes de combat). */
  talents: CrochetsJoueur | null = null;

  private readonly etat: EtatPartie;
  private readonly carte: CarteMonde;
  private readonly entrees: CommandesJoueur;
  private readonly retours: Retours;
  private readonly modele: ModeleCompile;
  private readonly rendu: ModeleInstancie;
  /** Armure de la Collection portée (aucune pièce : rien n'est dessiné). */
  private readonly armure: ArmureHeros;
  private readonly pose: PoseModele;
  private readonly animation = creerEtatAnimation();

  constructor(
    etat: EtatPartie,
    carte: CarteMonde,
    entrees: CommandesJoueur,
    retours: Retours,
    scene: THREE.Scene,
    textures: BibliothequeTextures,
    apparence: ApparenceEntite,
    armure: Partial<Record<PieceArmure, IdMonde>> = {},
  ) {
    this.etat = etat;
    this.carte = carte;
    this.entrees = entrees;
    this.retours = retours;
    this.modele = compilerModele(MODELES[apparence.modele]);
    this.rendu = new ModeleInstancie(this.modele, apparence, textures, 1, scene);
    this.pose = creerPoseModele(this.modele);
    this.armure = new ArmureHeros(this.modele, armure, textures, scene);
  }

  /** Sommet de la tête (blocs), pour l'étiquette du pseudo. */
  get hauteur(): number {
    return this.modele.hauteur;
  }

  /** Un pas de simulation. */
  maj(dt: number): void {
    const j = this.etat.joueur;
    if (j.mort) {
      j.tempsMort += dt;
      return;
    }
    if (j.invulnerabilite > 0) j.invulnerabilite -= dt;
    if (j.protection > 0) j.protection -= dt;
    const mods = this.etat.modificateurs;
    if (mods.regeneration > 0 || mods.regenerationPart > 0) {
      const avant = j.pv;
      // PV fixes des passifs et part des PV max des talents, dans le même plafond
      const regeneration = Math.min(mods.regeneration + mods.regenerationPart * j.pvMax, PLAFONDS.regeneration * j.pvMax);
      j.pv = Math.min(j.pvMax, j.pv + regeneration * dt);
      this.etat.stats.pvSoignes += j.pv - avant;
    }

    // Direction voulue d'après le clavier
    let dx = (this.entrees.action('droite') ? 1 : 0) - (this.entrees.action('gauche') ? 1 : 0);
    let dz = (this.entrees.action('bas') ? 1 : 0) - (this.entrees.action('haut') ? 1 : 0);
    const norme = longueur(dx, dz);
    if (norme > 0) {
      dx /= norme;
      dz /= norme;
    }
    // Vue à la 3e personne : « haut » avance dans le sens où regarde la caméra
    if (this.lacetCommandes !== 0) {
      const cos = Math.cos(this.lacetCommandes);
      const sin = Math.sin(this.lacetCommandes);
      const x = dx * cos + dz * sin;
      dz = dz * cos - dx * sin;
      dx = x;
    }

    // Vitesse : approche exponentielle de la vitesse cible (réactif, sans à-coups)
    const k = facteurLissage(JOUEUR_BASE.reactivite, dt);
    const vitesse = this.etat.personnage.vitesse * mods.vitesse * (this.talents?.vitesse() ?? 1) * this.carte.vitesseSol(j.x, j.z);
    j.vx += (dx * vitesse - j.vx) * k;
    j.vz += (dz * vitesse - j.vz) * k;
    j.x += j.vx * dt;
    j.z += j.vz * dt;

    // Décor et bords : on glisse le long des obstacles, la vitesse bloquée est annulée
    this.carte.resoudreCollision(j.x, j.z, JOUEUR_BASE.rayon, corrige);
    if (corrige.x !== j.x) j.vx = 0;
    if (corrige.z !== j.z) j.vz = 0;
    j.x = corrige.x;
    j.z = corrige.z;

    // Sol brûlant (magma)
    const degatsSol = this.carte.degatsSol(j.x, j.z) * Math.max(0, 1 + this.etat.modificateurs.degatsSol);
    if (degatsSol > 0) this.subirDegats(degatsSol);

    // Orientation vers la direction de marche
    if (norme > 0) {
      j.angle += ecartAngle(j.angle, Math.atan2(dx, dz)) * facteurLissage(JOUEUR_BASE.vitesseRotation, dt);
    }
    avancerAnimation(this.animation, this.modele, dt, longueur(j.vx, j.vz));
  }

  /** Se tourne vers sa cible et joue le geste d'attaque (coup d'épée) ; la marche le réoriente ensuite. */
  declencherGeste(angle: number): void {
    this.etat.joueur.angle = angle;
    declencherAttaque(this.animation, this.modele);
  }

  subirDegats(degats: number, boss = false): boolean {
    const j = this.etat.joueur;
    if (j.mort || this.invincible || j.invulnerabilite > 0 || j.protection > 0 || this.etat.issue !== null) return false;
    const talents = this.talents;
    if (talents) degats = talents.degatsRecus(degats, boss);
    // Équipement : dégâts reçus réduits (au mieux −60 %)
    degats *= Math.max(0.4, 1 + this.etat.modificateurs.degatsRecus);
    // Le bouclier absorbe d'abord ; s'il prend tout, le coup ne coûte aucun PV
    let reste = degatsApresDefense(degats, this.etat.modificateurs.armure + (talents?.armure() ?? 0));
    const bloque = degats - reste;
    const absorbe = Math.min(j.bouclier, reste);
    j.bouclier -= absorbe;
    reste -= absorbe;
    const subis = Math.min(j.pv, reste);
    j.pv -= subis;
    j.invulnerabilite = JOUEUR_BASE.invulnerabilite;
    this.etat.stats.degatsSubis += subis;
    this.retours.son('blessure_joueur');
    this.retours.secousse(0.18 + Math.min(subis, 10) * 0.03);
    if (j.pv === 0 && talents?.eviterMort()) return true;
    if (j.pv > 0) talents?.apresCoupRecu(subis, bloque + absorbe);
    if (j.pv === 0 && this.etat.modificateurs.vies > j.viesUtilisees) {
      this.renaitre();
    } else if (j.pv === 0) {
      j.mort = true;
      j.vx = 0;
      j.vz = 0;
    }
    return true;
  }

  /** Totem d'immortalité : consommé à la mort, il rend la moitié des PV et protège quelques secondes. */
  private renaitre(): void {
    const j = this.etat.joueur;
    const t = CONFIG.joueur.totem;
    j.viesUtilisees++;
    j.pv = Math.max(1, Math.ceil(j.pvMax * t.pv));
    j.protection = t.protection;
    this.retours.son('totem');
    this.retours.secousse(0.6);
  }

  /** Recopie l'état dans la scène. */
  majVisuel(ombres: Ombres): void {
    const j = this.etat.joueur;
    const pose = this.pose;
    pose.x = j.x;
    pose.z = j.z;
    pose.angle = j.angle;
    pose.inclinaison = j.mort ? angleChute(j.tempsMort, CONFIG.joueur.dureeMort) : 0;
    calculerRotations(this.modele, this.animation, pose.rotations);

    // Rouge pendant la blessure, comme en jeu
    const blessure = CONFIG.surcouche.blessure;
    const blesse = j.mort || j.invulnerabilite > JOUEUR_BASE.invulnerabilite - blessure.duree;
    pose.surcouche[0] = blessure.couleur[0];
    pose.surcouche[1] = blessure.couleur[1];
    pose.surcouche[2] = blessure.couleur[2];
    pose.surcouche[3] = blesse ? blessure.force : 0;

    this.rendu.commencer();
    this.rendu.ajouter(pose);
    this.rendu.terminer();
    this.armure.poser(pose);
    ombres.ajouter(j.x, j.z, this.etat.personnage.rayonOmbre);
  }

}
