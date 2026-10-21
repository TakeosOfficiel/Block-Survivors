/**
 * Mécanique « tornade » (Tornade de feu) : un entonnoir de cubes de feu qui
 * tourbillonne et avance vers l'ennemi le plus proche ; toutes les
 * `intervalle` secondes, elle blesse les ennemis dans son rayon et les aspire
 * vers son centre (recul négatif). En volume, façon Minecraft Dungeons : la
 * silhouette est un entonnoir de rubans en spirale texturés de feu animé
 * (lisible sans aucune particule), avec des débris du sol arrachés et un sol
 * brûlé dessous ; flammes, braises et fumée ne font que l'habiller. Elle
 * pousse du sol puis s'effondre en fumée.
 * Talent Tempête de feu : un météore tombé dedans la change en tornade de
 * magma, plus large et plus durable (`renforcer`).
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import { CubesTextures } from '../effets/cubesTextures';
import { Rubans } from '../effets/rubans';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Tornade {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  /** Direction de dérive quand aucun ennemi n'est proche. */
  dirX: number;
  dirZ: number;
  temps: number;
  prochainCoup: number;
  prochainesBraises: number;
  /** Part des dégâts (Écho arcanique), et tornade de magma (Tempête de feu) : zone et durée en plus. */
  facteur: number;
  magma: boolean;
  zone: number;
  duree: number;
}

export class Tornades {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(128);
  private readonly tornades: Pool<Tornade>;
  /** Rubans de feu (et de magma après la Tempête de feu), débris et sol brûlé. */
  private readonly feu: Rubans;
  private readonly magma: Rubans;
  private readonly debris: CubesTextures;
  private readonly sol: CubesTextures;
  /** Copies répétées le long des rubans des textures animées, et leurs originaux (qui portent l'image courante). */
  private readonly animees: { readonly copie: THREE.Texture; readonly source: THREE.Texture }[] = [];

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.tornade;
    this.contexte = contexte;
    this.tornades = new Pool<Tornade>(c.capacite, () => ({
      arme: null, x: 0, z: 0, dirX: 0, dirZ: 1, temps: 0, prochainCoup: 0, prochainesBraises: 0, facteur: 1, magma: false, zone: 1, duree: 0,
    }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'tornade');
    if (m?.type !== 'tornade') throw new Error('Aucune arme de type tornade');
    textures.animer(m.texture, c.imagesParSeconde);
    textures.animer(m.feu, c.imagesFeu);
    const rubans = c.capacite * (c.exterieurs + c.interieurs);
    this.feu = new Rubans(scene, this.repetee(textures, m.feu), 0xff8a2a, rubans, c.segments, c.pasU, 'tornades:feu');
    this.magma = new Rubans(scene, this.repetee(textures, m.texture), 0xc0401a, rubans, c.segments, c.pasU, 'tornades:magma');
    this.debris = new CubesTextures(scene, textures, m.debris, c.capacite * c.debris, false, 'tornades:débris');
    this.sol = new CubesTextures(scene, textures, m.sol, c.capacite * c.dalles, false, 'tornades:sol');
  }

  /** Copie de la texture animée qui se répète le long des rubans (l'image courante reste celle de l'original). */
  private repetee(textures: BibliothequeTextures, chemin: string): THREE.Texture | null {
    const source = textures.obtenir(chemin);
    if (!source) return null;
    const copie = source.clone();
    copie.wrapS = THREE.RepeatWrapping;
    copie.needsUpdate = true;
    this.animees.push({ copie, source });
    return copie;
  }

  /** Lance les tornades vers l'ennemi le plus proche ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'tornade') return false;
    const j = this.contexte.etat.joueur;
    const cible = this.contexte.ennemis.plusProche(j.x, j.z, CONFIG.armes.tornade.portee);
    if (!cible) return false;
    const angle = Math.atan2(cible.x - j.x, cible.z - j.z);
    const s = arme.stats;
    for (let k = 0; k < s.nombre; k++) {
      const t = this.tornades.acquerir();
      if (!t) break;
      const a = angle + (k - (s.nombre - 1) / 2) * 0.8;
      t.arme = arme;
      t.dirX = Math.sin(a);
      t.dirZ = Math.cos(a);
      t.x = j.x + t.dirX;
      t.z = j.z + t.dirZ;
      t.temps = 0;
      t.prochainCoup = 0;
      t.prochainesBraises = 0;
      t.facteur = this.contexte.talents.facteurDeclenchement;
      t.magma = false;
      t.zone = 1;
      t.duree = s.duree;
    }
    return true;
  }

  /** Tempête de feu : les tornades touchées par un météore en (x, z) deviennent des tornades de magma. */
  renforcer(x: number, z: number): void {
    const c = CONFIG.armes.tornade.magma;
    for (let i = 0; i < this.tornades.nombre; i++) {
      const t = this.tornades.elements[i];
      if (!t.arme || t.magma) continue;
      const rayon = t.arme.stats.zone * t.zone + c.marge;
      if ((t.x - x) ** 2 + (t.z - z) ** 2 > rayon * rayon) continue;
      t.magma = true;
      t.zone *= 1 + c.zone;
      t.duree += c.duree;
      this.contexte.particules.emettre(t.x, 1, t.z, GERBES.impactMeteore);
    }
  }

  maj(dt: number): void {
    const c = CONFIG.armes.tornade;
    const { ennemis, particules, carte } = this.contexte;
    for (let i = this.tornades.nombre - 1; i >= 0; i--) {
      const t = this.tornades.elements[i];
      const arme = t.arme;
      const m = arme?.definition.mecanique;
      t.temps += dt;
      if (!arme || m?.type !== 'tornade' || t.temps >= t.duree) {
        if (arme) particules.emettre(t.x, 1, t.z, GERBES.finTornade);
        t.arme = null;
        this.tornades.liberer(i);
        continue;
      }
      // Avance vers l'ennemi le plus proche, sinon continue sur sa lancée
      const cible = ennemis.plusProche(t.x, t.z, c.recherche);
      if (cible) {
        const dx = cible.x - t.x;
        const dz = cible.z - t.z;
        const d = longueur(dx, dz);
        if (d > 0.3) {
          t.dirX = dx / d;
          t.dirZ = dz / d;
        }
      }
      const nx = t.x + t.dirX * m.vitesse * dt;
      const nz = t.z + t.dirZ * m.vitesse * dt;
      if (!carte.bloqueMarche(nx, nz)) {
        t.x = nx;
        t.z = nz;
      }
      t.prochainCoup -= dt;
      if (t.prochainCoup <= 0) {
        t.prochainCoup += m.intervalle;
        const n = ennemis.chercher(t.x, t.z, arme.stats.zone * t.zone, this.tampon);
        for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, t.x, t.z, arme.stats.recul, t.facteur);
      }
      t.prochainesBraises -= dt;
      if (t.prochainesBraises <= 0) {
        t.prochainesBraises = c.periodeBraises;
        particules.emettre(t.x, 0.6, t.z, GERBES.tornade);
        // Braises arrachées au bord de l'entonnoir, à une hauteur au hasard
        const h = Math.random();
        const a = Math.random() * Math.PI * 2;
        const r = (arme.stats.zone * t.zone / 1.5) * (0.18 + 0.9 * h ** 1.3);
        particules.emettre(t.x + Math.sin(a) * r, 0.2 + h * c.hauteur, t.z + Math.cos(a) * r, GERBES.braisesTornade);
        if (t.magma) particules.emettre(t.x, 1.2, t.z, GERBES.braisesMagma);
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.tornade;
    const { feu, magma, debris, sol } = this;
    for (const a of this.animees) a.copie.offset.y = a.source.offset.y;
    feu.commencer();
    magma.commencer();
    debris.commencer();
    sol.commencer();
    const rubansParTornade = c.exterieurs + c.interieurs;
    for (let i = 0; i < this.tornades.nombre; i++) {
      const t = this.tornades.elements[i];
      if (!t.arme) continue;
      const largeur = (t.arme.stats.zone * t.zone) / 1.5;
      const rubans = t.magma ? magma : feu;
      // Pousse du sol (les rubans s'allongent vers le haut), puis s'effondre :
      // l'entonnoir s'écarte, s'affaisse et s'amincit
      const pousse = Math.min(1, t.temps / c.apparition);
      const chute = Math.max(0, 1 - (t.duree - t.temps) / c.effondrement);
      const ecartement = 1 + chute * 1.6;
      const affaissement = chute * chute * c.hauteur;
      const retrecissement = 1 - chute * 0.6;
      const longueurVisible = Math.min(1, pousse * 1.15);

      for (let r = 0; r < rubansParTornade && rubans.place; r++) {
        const interieur = r >= c.exterieurs;
        const groupe = interieur ? c.interieurs : c.exterieurs;
        const phase = ((interieur ? r - c.exterieurs + 0.5 : r) / groupe) * Math.PI * 2;
        const facteurRayon = interieur ? 0.45 : 1;
        const vitesse = c.rotation * (interieur ? 1.6 : 1);
        const facteurBande = (interieur ? 0.8 : 1) * retrecissement;
        for (let k = 0; k <= c.segments; k++) {
          const h = (k / c.segments) * longueurVisible;
          // Le haut ondule en retard sur la base
          const ondulation = h * h * c.oscillation * largeur;
          const cx = t.x + Math.sin(t.temps * 1.9 + h * 2.2) * ondulation;
          const cz = t.z + Math.cos(t.temps * 1.4 + h * 2.2) * ondulation;
          const y = 0.05 + h * c.hauteur - affaissement * (0.4 + h);
          const rayon = largeur * (0.18 + 0.9 * h ** 1.3) * facteurRayon * ecartement;
          const angle = t.temps * vitesse * (1 + 0.4 * h) + phase + h * c.tours * Math.PI * 2;
          const bande = (c.bande[0] + (c.bande[1] - c.bande[0]) * h) * facteurBande;
          const sin = Math.sin(angle);
          const cos = Math.cos(angle);
          // Le bord haut penche vers l'extérieur : la paroi de l'entonnoir s'évase
          const rayonHaut = rayon + bande * 0.35;
          rubans.point(
            k,
            cx + sin * rayon, Math.max(0.02, y - bande * 0.3), cz + cos * rayon,
            cx + sin * rayonHaut, Math.max(0.05, y + bande * 0.7), cz + cos * rayonHaut,
          );
        }
        rubans.fermer();
      }

      // Débris du sol arrachés : montent en spirale, s'écartent, retombent au sommet
      const tailleDebris = 0.2 * pousse * retrecissement;
      for (let k = 0; k < c.debris; k++) {
        const phase = (t.temps * 0.55 + k / c.debris) % 1;
        const angle = t.temps * c.rotation * 1.2 + k * 2.4;
        const r = (0.3 + phase * 0.95) * largeur * ecartement;
        const y = 0.1 + Math.sin(phase * Math.PI) * c.hauteur * 0.9 - affaissement;
        debris.ajouter(t.x + Math.sin(angle) * r, Math.max(0.1, y), t.z + Math.cos(angle) * r, tailleDebris, angle * 1.7, t.temps * 5 + k);
      }

      // Sol brûlé : anneau de dalles noircies qui tourne lentement sous la tornade
      const tailleDalle = 0.34 * Math.min(1, pousse * 1.5) * (1 - chute * 0.5);
      for (let k = 0; k < c.dalles; k++) {
        const angle = t.temps * 0.8 + (k / c.dalles) * Math.PI * 2;
        const r = largeur * (k % 2 === 0 ? 0.55 : 0.85);
        sol.ajouterDalle(t.x + Math.sin(angle) * r, t.z + Math.cos(angle) * r, tailleDalle, 0.04, angle);
      }
    }
    feu.terminer();
    magma.terminer();
    debris.terminer();
    sol.terminer();
  }
}
