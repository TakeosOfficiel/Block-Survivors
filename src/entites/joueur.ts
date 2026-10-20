/** Joueur : déplacement, collisions avec les bords, vie, modèle animé du personnage choisi. */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import type { Ombres } from '../effets/ombres';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import type { CarteMonde, Point } from '../monde/carte';
import { angleChute, avancerAnimation, calculerRotations, creerEtatAnimation, declencherAttaque } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { ecartAngle, facteurLissage, longueur } from '../utils/maths';
import type { CibleJoueur, SourceEntites } from './types';

const corrige: Point = { x: 0, z: 0 };

export class Joueur implements SourceEntites, CibleJoueur {
  readonly actifs = 1;
  /** Ignore tous les dégâts (debug). */
  invincible = false;

  private readonly etat: EtatPartie;
  private readonly carte: CarteMonde;
  private readonly entrees: Entrees;
  private readonly retours: Retours;
  private readonly modele: ModeleCompile;
  private readonly rendu: ModeleInstancie;
  private readonly pose: PoseModele;
  private readonly animation = creerEtatAnimation();

  constructor(
    etat: EtatPartie,
    carte: CarteMonde,
    entrees: Entrees,
    retours: Retours,
    scene: THREE.Scene,
    textures: BibliothequeTextures,
  ) {
    this.etat = etat;
    this.carte = carte;
    this.entrees = entrees;
    this.retours = retours;
    const personnage = etat.personnage;
    this.modele = compilerModele(MODELES[personnage.modele]);
    this.rendu = new ModeleInstancie(this.modele, personnage, textures.obtenir(personnage.skin), 1, scene);
    this.pose = creerPoseModele(this.modele);
  }

  /** Un pas de simulation. */
  maj(dt: number): void {
    const j = this.etat.joueur;
    if (j.mort) {
      j.tempsMort += dt;
      return;
    }
    if (j.invulnerabilite > 0) j.invulnerabilite -= dt;
    const mods = this.etat.modificateurs;
    if (mods.regeneration > 0) {
      const avant = j.pv;
      j.pv = Math.min(j.pvMax, j.pv + mods.regeneration * dt);
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

    // Vitesse : approche exponentielle de la vitesse cible (réactif, sans à-coups)
    const k = facteurLissage(JOUEUR_BASE.reactivite, dt);
    const vitesse = this.etat.personnage.vitesse * mods.vitesse * this.carte.vitesseSol(j.x, j.z);
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
    const degatsSol = this.carte.degatsSol(j.x, j.z);
    if (degatsSol > 0) this.subirDegats(degatsSol);

    // Orientation vers la direction de marche
    if (norme > 0) {
      j.angle += ecartAngle(j.angle, Math.atan2(dx, dz)) * facteurLissage(JOUEUR_BASE.vitesseRotation, dt);
    }
    avancerAnimation(this.animation, this.modele, dt, longueur(j.vx, j.vz));
  }

  /** Joue le geste d'attaque (coup d'épée). */
  declencherGeste(): void {
    declencherAttaque(this.animation, this.modele);
  }

  subirDegats(degats: number): boolean {
    const j = this.etat.joueur;
    if (j.mort || this.invincible || j.invulnerabilite > 0 || this.etat.issue !== null) return false;
    // L'armure retire des dégâts, mais un coup en fait toujours au moins 1
    const subis = Math.min(j.pv, Math.max(1, degats - this.etat.modificateurs.armure));
    j.pv -= subis;
    j.invulnerabilite = JOUEUR_BASE.invulnerabilite;
    this.etat.stats.degatsSubis += subis;
    this.retours.son('blessure_joueur');
    this.retours.secousse(0.18 + Math.min(subis, 10) * 0.03);
    if (j.pv === 0) {
      j.mort = true;
      j.vx = 0;
      j.vz = 0;
    }
    return true;
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
    ombres.ajouter(j.x, j.z, this.etat.personnage.rayonOmbre);
  }

}
