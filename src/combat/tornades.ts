/**
 * Mécanique « tornade » (Tornade de feu) : un entonnoir de feu qui
 * tourbillonne et avance vers l'ennemi le plus proche ; toutes les
 * `intervalle` secondes, elle blesse les ennemis dans son rayon et les aspire
 * vers son centre (recul négatif). En volume, façon Minecraft Dungeons : la
 * silhouette est faite de quelques spirales continues, texturées de feu animé
 * et séparées par des vides, lisible même figée ; les irrégularités sont sur
 * leurs bords. Elle laisse derrière elle une trace de brûlure à plat qui
 * s'estompe ; quelques braises à son pied l'habillent. Elle pousse du sol
 * puis s'effondre en fumée.
 * Talent Tempête de feu : un météore tombé dedans la change en tornade de
 * magma, plus large et plus durable (`renforcer`), aux zones jaunes
 * incandescentes et chargée de quelques fragments de magma.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import { CubesTextures } from '../effets/cubesTextures';
import { Rubans } from '../effets/rubans';
import { borner, longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';
import { alea as tirer } from '../utils/aleatoire';

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
  /** Tire au sort les petits écarts de ses spirales (deux tornades ne se ressemblent pas). */
  graine: number;
  /** Distance parcourue depuis la dernière brûlure posée (blocs). */
  depuisTrace: number;
}

/** Brûlure laissée au sol, qui s'estompe (rétrécit) avec l'âge. */
interface Trace {
  x: number;
  z: number;
  age: number;
  taille: number;
  rotation: number;
}

/** Nombre pseudo-aléatoire stable dans [0, 1[ tiré de `n`. */
function alea(n: number): number {
  const x = Math.sin(n) * 43758.5453;
  return x - Math.floor(x);
}

/** Forme d'une tornade à cette image (apparition et effondrement compris). */
interface FormeTornade {
  largeur: number;
  ecartement: number;
  affaissement: number;
  retrecissement: number;
  longueurVisible: number;
}

/** Réutilisés pour chaque tornade et chaque brûlure : rien n'est alloué pendant le rendu. */
const forme: FormeTornade = { largeur: 0, ecartement: 1, affaissement: 0, retrecissement: 1, longueurVisible: 1 };
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const echelle = new THREE.Vector3();
const AXE_Y = new THREE.Vector3(0, 1, 0);

export class Tornades {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(128);
  private readonly tornades: Pool<Tornade>;
  private readonly traces: Pool<Trace>;
  /** Spirales de feu (celles des tornades de magma ont leurs zones jaunes), fragments de magma, brûlures. */
  private readonly feu: Rubans;
  private readonly fragments: CubesTextures;
  private readonly brulures: THREE.InstancedMesh;
  /** Copie du feu animé répétée le long des spirales, et son original (qui porte l'image courante). */
  private readonly feuRepete: THREE.Texture | null;
  private readonly feuSource: THREE.Texture | null;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.tornade;
    this.contexte = contexte;
    this.tornades = new Pool<Tornade>(c.capacite, () => ({
      arme: null, x: 0, z: 0, dirX: 0, dirZ: 1, temps: 0, prochainCoup: 0, prochainesBraises: 0, facteur: 1, magma: false, zone: 1, duree: 0,
      graine: 0, depuisTrace: 0,
    }));
    this.traces = new Pool<Trace>(c.trace.capacite, () => ({ x: 0, z: 0, age: 0, taille: 0, rotation: 0 }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'tornade');
    if (m?.type !== 'tornade') throw new Error('Aucune arme de type tornade');
    textures.animer(m.texture, c.imagesParSeconde);
    textures.animer(m.feu, c.imagesFeu);
    this.feuSource = textures.obtenir(m.feu);
    this.feuRepete = this.feuSource?.clone() ?? null;
    if (this.feuRepete) {
      this.feuRepete.wrapS = THREE.RepeatWrapping;
      this.feuRepete.needsUpdate = true;
    }
    this.feu = new Rubans(scene, this.feuRepete, 0xff8a2a, c.capacite * c.spirales, c.segments, c.pasU, 'tornades:feu', c.vHaut);
    this.fragments = new CubesTextures(scene, textures, m.texture, c.capacite * c.magma.fragments, true, 'tornades:magma');
    // Brûlures : carrés à plat de la texture brune, éclairés comme le sol, sans relief
    const map = textures.obtenir(m.brulure);
    this.brulures = new THREE.InstancedMesh(
      new THREE.PlaneGeometry(1, 1).rotateX(-Math.PI / 2),
      new THREE.MeshLambertMaterial(map ? { map } : { color: 0x4c3a2f }),
      c.trace.capacite,
    );
    this.brulures.name = 'tornades:brûlures';
    this.brulures.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.brulures.frustumCulled = false;
    this.brulures.count = 0;
    scene.add(this.brulures);
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
      t.graine = tirer() * 1000;
      t.depuisTrace = Infinity;  // première brûlure dès l'apparition
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
        t.depuisTrace += longueur(nx - t.x, nz - t.z);
        t.x = nx;
        t.z = nz;
      }
      if (t.depuisTrace >= c.trace.espacement) {
        t.depuisTrace = 0;
        this.bruler(t);
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
        // Quelques braises au pied, rien de plus : l'attention reste sur le vortex
        particules.emettre(t.x, 0.15, t.z, GERBES.tornade);
        if (t.magma) particules.emettre(t.x, 0.3, t.z, GERBES.braisesMagma);
      }
    }
    for (let i = this.traces.nombre - 1; i >= 0; i--) {
      const trace = this.traces.elements[i];
      trace.age += dt;
      if (trace.age >= c.trace.duree) this.traces.liberer(i);
    }
  }

  /** Pose une brûlure sous la tornade, de la taille de son pied. */
  private bruler(t: Tornade): void {
    const trace = this.traces.acquerir();
    if (!trace || !t.arme) return;
    const c = CONFIG.armes.tornade.trace;
    trace.x = t.x + (tirer() - 0.5) * 0.3;
    trace.z = t.z + (tirer() - 0.5) * 0.3;
    trace.age = 0;
    trace.taille = ((t.arme.stats.zone * t.zone) / 1.5) * c.taille * (0.85 + tirer() * 0.3);
    trace.rotation = tirer() * Math.PI * 2;
  }

  majVisuel(): void {
    const c = CONFIG.armes.tornade;
    const { feu, fragments } = this;
    // Image courante de l'animation, et le feu qui monte le long des spirales
    if (this.feuRepete && this.feuSource) {
      this.feuRepete.offset.y = this.feuSource.offset.y;
      this.feuRepete.offset.x = -((this.contexte.etat.tempsJeu * c.defilement) % 1);
    }
    feu.commencer();
    fragments.commencer();
    for (let i = 0; i < this.tornades.nombre; i++) {
      const t = this.tornades.elements[i];
      if (!t.arme) continue;
      const largeur = (t.arme.stats.zone * t.zone) / 1.5;
      // Pousse du sol (les spirales s'allongent vers le haut), puis s'effondre :
      // l'entonnoir s'écarte, s'affaisse et s'amincit
      const pousse = Math.min(1, t.temps / c.apparition);
      const chute = Math.max(0, 1 - (t.duree - t.temps) / c.effondrement);
      forme.largeur = largeur;
      forme.ecartement = 1 + chute * 1.6;
      forme.affaissement = chute * chute * c.hauteur;
      forme.retrecissement = 1 - chute * 0.6;
      forme.longueurVisible = Math.min(1, pousse * 1.15);
      for (let r = 0; r < c.spirales; r++) this.spirale(t, forme, r);

      if (t.magma) {
        // Quelques fragments de magma incandescents emportés dans le vortex
        for (let k = 0; k < c.magma.fragments; k++) {
          const h = (t.temps * 0.7 + k / c.magma.fragments) % 1;
          const angle = t.temps * c.rotation * 1.1 + k * 2.4;
          const r = largeur * (0.18 + 0.9 * h ** 1.3) * 0.9 * forme.ecartement;
          const taille = c.magma.tailleFragment * (1 - h * 0.5) * forme.retrecissement * Math.min(1, pousse * 2);
          fragments.ajouter(
            t.x + Math.sin(angle) * r, 0.2 + h * c.hauteur * 0.9 - forme.affaissement, t.z + Math.cos(angle) * r,
            taille, angle * 1.3, t.temps * 4 + k,
          );
        }
      }
    }
    feu.terminer();
    fragments.terminer();

    // Brûlures : à plat sur le sol, elles rétrécissent en s'estompant
    const tr = c.trace;
    let n = 0;
    for (let i = 0; i < this.traces.nombre; i++) {
      const trace = this.traces.elements[i];
      const reste = 1 - trace.age / tr.duree;
      const taille = trace.taille * Math.min(1, trace.age / 0.15) * Math.sqrt(reste);
      if (taille <= 0) continue;
      // Hauteurs légèrement différentes : les brûlures qui se chevauchent ne scintillent pas
      position.set(trace.x, tr.hauteur + (i % 8) * 0.001, trace.z);
      orientation.setFromAxisAngle(AXE_Y, trace.rotation);
      matrice.compose(position, orientation, echelle.set(taille, 1, taille));
      this.brulures.setMatrixAt(n++, matrice);
    }
    this.brulures.count = n;
    this.brulures.instanceMatrix.needsUpdate = true;
  }

  /**
   * Une spirale principale de l'entonnoir : un ruban continu, régulier dans son
   * tracé (seuls de petits écarts tirés de la graine), dont les irrégularités
   * sont sur les bords : bas dentelé, haut découpé par les flammes de la
   * texture. Sur une tornade de magma, des zones jaunes incandescentes
   * remontent le long du rouge.
   */
  private spirale(t: Tornade, f: FormeTornade, r: number): void {
    const rubans = this.feu;
    if (!rubans.place) return;
    const c = CONFIG.armes.tornade;
    const g = t.graine + r * 17.31;
    const ir = c.irregularite;
    const rayonRuban = 1 - ir + 2 * ir * alea(g);
    const tours = c.tours * (1 - ir + 2 * ir * alea(g + 2.2));
    const decalage = (r / c.spirales) * Math.PI * 2 + (alea(g + 4.4) - 0.5) * 0.4;
    const o = c.ondes;
    const m = c.magma;
    for (let k = 0; k <= c.segments; k++) {
      const s = k / c.segments;
      const h = s * f.longueurVisible;
      // Le haut ondule en retard sur la base
      const ondulation = h * h * c.oscillation * f.largeur;
      const cx = t.x + Math.sin(t.temps * 1.9 + h * 2.2) * ondulation;
      const cz = t.z + Math.cos(t.temps * 1.4 + h * 2.2) * ondulation;
      const y = 0.05 + h * c.hauteur - f.affaissement * (0.4 + h);
      // Une onde douce monte le long de la spirale
      const onde = Math.sin(h * o.frequence - t.temps * o.vitesse + g);
      const rayon = (f.largeur * (0.18 + 0.9 * h ** 1.3) * rayonRuban + f.largeur * o.amplitude * onde) * f.ecartement;
      const angle = t.temps * c.rotation * (1 + 0.4 * h) + decalage + h * tours * Math.PI * 2;
      // Bande pleine sur toute la longueur (pas de traînée fine), à peine plus mince tout en bas
      const bande = (c.bande[0] + (c.bande[1] - c.bande[0]) * h) * f.retrecissement
        * Math.min(1, 0.6 + s * 8) * (1 + c.palpitation * onde);
      const sin = Math.sin(angle);
      const cos = Math.cos(angle);
      // Le bord haut penche vers l'extérieur (la paroi s'évase) ; le bord bas est dentelé
      const rayonHaut = rayon + bande * 0.35;
      const dent = k % 2 === 0 ? 0 : bande * c.dents;
      let rouge = 1;
      let vert = 1;
      let bleu = 1;
      if (t.magma) {
        // Rouge sombre, traversé de zones jaunes incandescentes qui montent
        const chaleur = borner((Math.sin(h * m.frequenceChaleur - t.temps * m.vitesseChaleur + g) - 0.2) * 1.6, 0, 1);
        rouge = m.rouge[0] + (m.jaune[0] - m.rouge[0]) * chaleur;
        vert = m.rouge[1] + (m.jaune[1] - m.rouge[1]) * chaleur;
        bleu = m.rouge[2] + (m.jaune[2] - m.rouge[2]) * chaleur;
      }
      rubans.point(
        k,
        cx + sin * rayon, Math.max(0.02, y - bande * 0.3 + dent), cz + cos * rayon,
        cx + sin * rayonHaut, Math.max(0.05, y + bande * 0.7), cz + cos * rayonHaut,
        rouge, vert, bleu,
      );
    }
    rubans.fermer();
  }
}
