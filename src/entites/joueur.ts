/** Joueur : déplacement, collisions avec les bords, vie, modèle animé du personnage choisi. */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import type { Ombres } from '../effets/ombres';
import type { EtatJoueur, EtatPartie } from '../etat';
import { angleChute, avancerAnimation, calculerRotations, creerEtatAnimation, declencherAttaque } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { ecartAngle, facteurLissage } from '../utils/maths';
import type { CibleJoueur, SourceEntites } from './types';

export class Joueur implements SourceEntites, CibleJoueur {
  readonly actifs = 1;
  /** Ignore tous les dégâts (debug). */
  invincible = false;

  private readonly etat: EtatPartie;
  private readonly entrees: Entrees;
  private readonly modele: ModeleCompile;
  private readonly rendu: ModeleInstancie;
  private readonly pose: PoseModele;
  private readonly animation = creerEtatAnimation();

  constructor(etat: EtatPartie, entrees: Entrees, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.etat = etat;
    this.entrees = entrees;
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
    if (mods.regeneration > 0) j.pv = Math.min(j.pvMax, j.pv + mods.regeneration * dt);

    // Direction voulue d'après le clavier
    let dx = (this.entrees.action('droite') ? 1 : 0) - (this.entrees.action('gauche') ? 1 : 0);
    let dz = (this.entrees.action('bas') ? 1 : 0) - (this.entrees.action('haut') ? 1 : 0);
    const norme = Math.hypot(dx, dz);
    if (norme > 0) {
      dx /= norme;
      dz /= norme;
    }

    // Vitesse : approche exponentielle de la vitesse cible (réactif, sans à-coups)
    const k = facteurLissage(JOUEUR_BASE.reactivite, dt);
    const vitesse = this.etat.personnage.vitesse * mods.vitesse;
    j.vx += (dx * vitesse - j.vx) * k;
    j.vz += (dz * vitesse - j.vz) * k;
    j.x += j.vx * dt;
    j.z += j.vz * dt;

    this.resoudreCollisionsBords(j);

    // Orientation vers la direction de marche
    if (norme > 0) {
      j.angle += ecartAngle(j.angle, Math.atan2(dx, dz)) * facteurLissage(JOUEUR_BASE.vitesseRotation, dt);
    }
    avancerAnimation(this.animation, this.modele, dt, Math.hypot(j.vx, j.vz));
  }

  /** Joue le geste d'attaque (coup d'épée). */
  declencherGeste(): void {
    declencherAttaque(this.animation, this.modele);
  }

  subirDegats(degats: number): boolean {
    const j = this.etat.joueur;
    if (j.mort || this.invincible || j.invulnerabilite > 0) return false;
    // L'armure retire des dégâts, mais un coup en fait toujours au moins 1
    const subis = Math.max(1, degats - this.etat.modificateurs.armure);
    j.pv = Math.max(0, j.pv - subis);
    j.invulnerabilite = JOUEUR_BASE.invulnerabilite;
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

  /** Maintient le joueur dans la zone jouable ; la vitesse contre un mur est annulée. */
  private resoudreCollisionsBords(j: EtatJoueur): void {
    const limite = this.etat.arene.taille / 2 - JOUEUR_BASE.rayon;
    if (j.x < -limite) { j.x = -limite; j.vx = 0; }
    else if (j.x > limite) { j.x = limite; j.vx = 0; }
    if (j.z < -limite) { j.z = -limite; j.vz = 0; }
    else if (j.z > limite) { j.z = limite; j.vz = 0; }
  }
}
