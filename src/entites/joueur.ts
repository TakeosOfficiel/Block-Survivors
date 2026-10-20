/** Joueur : déplacement, collisions avec les bords, modèle animé et ombre. */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import { JOUEUR_BASE } from '../donnees/joueur';
import { MODELES } from '../donnees/modeles';
import type { EtatJoueur, EtatPartie } from '../etat';
import { ecartAngle, facteurLissage } from '../utils/maths';
import { AnimateurModele } from './animateur';
import { ModeleCubique } from './modeleCubique';
import { creerOmbre } from './ombre';
import type { SourceEntites } from './types';

export class Joueur implements SourceEntites {
  readonly actifs = 1;

  private readonly etat: EtatPartie;
  private readonly entrees: Entrees;
  /** Suit la position ; le modèle à l'intérieur porte l'orientation, l'ombre reste à plat. */
  private readonly conteneur = new THREE.Group();
  private readonly modele: ModeleCubique;
  private readonly animateur: AnimateurModele;

  constructor(etat: EtatPartie, entrees: Entrees, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.etat = etat;
    this.entrees = entrees;

    const definition = MODELES[JOUEUR_BASE.modele];
    this.modele = new ModeleCubique(definition, textures.obtenir(JOUEUR_BASE.skin), JOUEUR_BASE.couleursSecours);
    this.animateur = new AnimateurModele(this.modele, definition);

    this.conteneur.name = 'joueur';
    this.conteneur.add(this.modele.racine, creerOmbre(textures.obtenir(CONFIG.ombre.texture), JOUEUR_BASE.rayonOmbre));
    scene.add(this.conteneur);
    this.majVisuel(0);
  }

  /** Un pas de simulation. */
  maj(dt: number): void {
    const j = this.etat.joueur;

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
    j.vx += (dx * JOUEUR_BASE.vitesse - j.vx) * k;
    j.vz += (dz * JOUEUR_BASE.vitesse - j.vz) * k;
    j.x += j.vx * dt;
    j.z += j.vz * dt;

    this.resoudreCollisionsBords(j);

    // Orientation vers la direction de marche
    if (norme > 0) {
      j.angle += ecartAngle(j.angle, Math.atan2(dx, dz)) * facteurLissage(JOUEUR_BASE.vitesseRotation, dt);
    }
  }

  /** Recopie l'état dans la scène et anime le modèle (dtJeu en temps de jeu). */
  majVisuel(dtJeu: number): void {
    const j = this.etat.joueur;
    this.conteneur.position.set(j.x, 0, j.z);
    this.modele.racine.rotation.y = j.angle;
    this.animateur.maj(dtJeu, Math.hypot(j.vx, j.vz));
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
