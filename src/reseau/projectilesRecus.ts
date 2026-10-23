/**
 * Flèches et projectiles des ennemis dans les instantanés. L'hôte envoie leur
 * position, leur direction et leur vitesse ; le joueur les fait avancer en
 * ligne droite entre deux instantanés (un instantané tous les 1/15 s : sans
 * cela, une flèche rapide saute de deux blocs à la fois) et cesse d'en
 * dessiner une dès qu'elle atteint le décor.
 */
import { CONFIG } from '../config';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { ProjectilesEnnemis } from '../entites/projectilesEnnemis';
import type { CarteMonde } from '../monde/carte';
import type { Ecrivain, Lecteur } from './protocole';

const CAPACITE = 2048;
/** Valeurs par projectile : rendu, x, y, z, angle, vitesse. */
const CHAMPS = 6;
/** Vitesse envoyée au 1/4 de bloc par seconde (au plus 63 blocs/s). */
const PAS_VITESSE = 4;
/** Au-delà de ce retard (en instantanés), un projectile n'avance plus : le prochain instantané va le replacer. */
const AVANCE_MAX = 1.5;

/** Hôte : les projectiles des ennemis à moins de √r2 de (cx, cz), tant qu'il reste de la place. */
export function ecrireProjectiles(w: Ecrivain, projectiles: ProjectilesEnnemis, cx: number, cz: number, r2: number): void {
  const compte = w.reserver16();
  let n = 0;
  const pool = projectiles.pool;
  for (let i = 0; i < pool.nombre && w.reste > 2 * 1024; i++) {
    const p = pool.elements[i];
    if ((p.x - cx) ** 2 + (p.z - cz) ** 2 > r2) continue;
    w.u8(p.indexRendu);
    w.coordonnee(p.x);
    w.u8(Math.min(255, Math.round(p.definition.hauteur * 32)));
    w.coordonnee(p.z);
    w.angle(Math.atan2(p.vx, p.vz));
    w.u8(Math.min(255, Math.round(Math.hypot(p.vx, p.vz) * PAS_VITESSE)));
    n++;
  }
  w.ecrire16(compte, n);
}

/** Joueur : les projectiles du dernier instantané, avancés à chaque image. */
export class ProjectilesRecus {
  private readonly carte: CarteMonde;
  private readonly valeurs = new Float32Array(CAPACITE * CHAMPS);
  private nombre = 0;

  constructor(carte: CarteMonde) {
    this.carte = carte;
  }

  lire(l: Lecteur): void {
    const n = l.u16();
    this.nombre = 0;
    for (let i = 0; i < n; i++) {
      const rendu = l.u8();
      const x = l.coordonnee();
      const y = l.u8() / 32;
      const z = l.coordonnee();
      const angle = l.angle();
      const vitesse = l.u8() / PAS_VITESSE;
      if (this.nombre >= CAPACITE) continue;
      const o = this.nombre++ * CHAMPS;
      this.valeurs[o] = rendu;
      this.valeurs[o + 1] = x;
      this.valeurs[o + 2] = y;
      this.valeurs[o + 3] = z;
      this.valeurs[o + 4] = angle;
      this.valeurs[o + 5] = vitesse;
    }
  }

  /** `depuis` : s écoulées depuis l'arrivée de l'instantané. */
  dessiner(rendu: RenduProjectiles, depuis: number): void {
    const t = Math.min(depuis, AVANCE_MAX / REGLES_MULTIJOUEUR.reseau.instantanesParSeconde);
    const rayon = CONFIG.projectiles.rayonDecor;
    for (let i = 0; i < this.nombre; i++) {
      const o = i * CHAMPS;
      const angle = this.valeurs[o + 4];
      const avance = this.valeurs[o + 5] * t;
      const x = this.valeurs[o + 1] + Math.sin(angle) * avance;
      const z = this.valeurs[o + 3] + Math.cos(angle) * avance;
      // Arrivé dans un mur chez l'hôte : il disparaît là, pas de l'autre côté
      if (this.carte.toucheTir(x, z, rayon)) continue;
      rendu.ajouter(this.valeurs[o], x, this.valeurs[o + 2], z, angle);
    }
  }
}
