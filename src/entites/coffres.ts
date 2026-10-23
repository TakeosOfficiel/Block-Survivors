/**
 * Coffres posés sur la carte (données : donnees/coffres.ts). Un coffre
 * ordinaire apparaît tous les tant d'ennemis tués (à quelques blocs du
 * joueur, sur une case qu'il peut atteindre) et à la mort de chaque mini-boss ;
 * le boss lâche un coffre de l'Ender. Marcher dessus l'ouvre si l'on a assez
 * d'émeraudes (gratuit pour l'Ender) : le couvercle se lève, puis les cartes
 * d'amélioration s'ouvrent (écran des cartes, voir jeu.ts). Leur prix flotte
 * au-dessus d'eux.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { COFFRES, IDS_COFFRES, REGLES_COFFRES, type IdCoffre } from '../donnees/coffres';
import { MODELES } from '../donnees/modeles';
import { GERBES } from '../donnees/particules';
import { Etiquette } from '../effets/etiquette';
import type { Particules } from '../effets/particules';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import { compilerModele } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import type { Terrain } from '../monde/navigation';
import { Pool } from '../utils/pool';
import type { EquipeJoueurs } from './types';

interface Coffre {
  indexType: number;
  x: number;
  z: number;
  angle: number;
  /** −1 : fermé ; sinon s depuis l'ouverture. */
  ouvert: number;
}

/** Couleurs du prix au-dessus du coffre : payable, trop cher. */
const PRIX_ABORDABLE = '#55ff55';
const PRIX_TROP_CHER = '#ff5555';

/** Hauteur du coffre (blocs) : le modèle, décrit tête en bas, est retourné autour d'elle. */
const HAUTEUR = 14 / 16;

export class Coffres {
  private readonly etat: EtatPartie;
  private readonly terrain: Terrain;
  private readonly particules: Particules;
  private readonly retours: Retours;
  /** Public pour le multijoueur : l'hôte le lit, le client le remplit d'après l'hôte. */
  readonly pool: Pool<Coffre>;
  private readonly rendus: ModeleInstancie[];
  private readonly pose: PoseModele;
  /** Index des parties du couvercle et du loquet (elles pivotent ensemble). */
  private readonly partiesCouvercle: number[];
  private readonly etiquettes: Etiquette[];
  /**
   * Ce que montre chaque étiquette (prix × 2, +1 s'il est payable ; −1 : rien). Les étiquettes
   * vont par emplacement du pool, et un coffre change d'emplacement quand un autre disparaît :
   * on compare donc à ce qui est affiché, pas à ce qu'a vu le coffre.
   */
  private readonly affichees: Int32Array;
  /** Ennemis tués à atteindre pour le prochain coffre, et l'écart suivant. */
  private prochainSeuil = REGLES_COFFRES.tues.premier;
  private intervalle = REGLES_COFFRES.tues.intervalle;
  /** Coffres payants déjà ouverts (le prix monte à chacun). */
  ouverts = 0;
  /** Multijoueur (hôte) : n'importe quel joueur de l'équipe ouvre les coffres. */
  equipe: EquipeJoueurs | null = null;
  // Résultat de trouverCase (évite d'allouer un objet)
  private caseX = 0;
  private caseZ = 0;

  constructor(etat: EtatPartie, terrain: Terrain, particules: Particules, retours: Retours, scene: THREE.Scene, textures: BibliothequeTextures) {
    const r = REGLES_COFFRES;
    this.etat = etat;
    this.terrain = terrain;
    this.particules = particules;
    this.retours = retours;
    this.pool = new Pool<Coffre>(r.capacite, () => ({ indexType: 0, x: 0, z: 0, angle: 0, ouvert: -1 }));
    const modele = compilerModele(MODELES.coffre);
    this.rendus = IDS_COFFRES.map((id) => new ModeleInstancie(modele, COFFRES[id].apparence, textures, r.capacite, scene));
    this.pose = creerPoseModele(modele);
    this.partiesCouvercle = modele.parties.flatMap((p, i) => (p.nom === 'couvercle' || p.nom === 'loquet' ? [i] : []));
    this.etiquettes = Array.from({ length: r.capacite }, () => new Etiquette(scene, ''));
    this.affichees = new Int32Array(r.capacite).fill(-1);
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Pose un coffre à cet endroit (ou sur la case accessible la plus proche). */
  poser(id: IdCoffre, x: number, z: number): void {
    const c = this.pool.acquerir();
    if (!c) return;
    c.indexType = IDS_COFFRES.indexOf(id);
    c.x = Math.floor(x) + 0.5;
    c.z = Math.floor(z) + 0.5;
    if (!this.accessible(c.x, c.z) && this.trouverCase(x, z, 0, 6)) {
      c.x = this.caseX;
      c.z = this.caseZ;
    }
    // Le loquet face au joueur
    const j = this.etat.joueur;
    c.angle = Math.round(Math.atan2(j.x - c.x, j.z - c.z) / (Math.PI / 2)) * (Math.PI / 2);
    c.ouvert = -1;
    this.retours.son('coffre_apparition');
    this.particules.emettre(c.x, 0.6, c.z, GERBES.invocation);
  }

  maj(dt: number): void {
    const r = REGLES_COFFRES;
    const etat = this.etat;
    const j = etat.joueur;

    // Un coffre tous les tant d'ennemis tués, à quelques blocs du joueur
    if (etat.ennemisTues >= this.prochainSeuil) {
      this.prochainSeuil += this.intervalle;
      this.intervalle += r.tues.croissance;
      const [min, max] = r.distance;
      if (this.trouverCase(j.x, j.z, min, max)) this.poser('coffre', this.caseX, this.caseZ);
    }

    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const c = this.pool.elements[i];
      if (c.ouvert >= 0) {
        // Les cartes viennent une fois le couvercle levé
        if (c.ouvert < r.dureeOuverture && c.ouvert + dt >= r.dureeOuverture) {
          const def = COFFRES[IDS_COFFRES[c.indexType]];
          etat.choixCoffre.restants += def.choix;
          etat.choixCoffre.titre = def.titre;
        }
        c.ouvert += dt;
        if (c.ouvert >= r.dureeOuverture + r.resteOuvert) {
          this.particules.emettre(c.x, 0.5, c.z, GERBES.invocation);
          this.pool.liberer(i);
        }
        continue;
      }
      const k = this.equipe ? this.equipe.plusProche(c.x, c.z) : -1;
      const ouvreur = this.equipe && k >= 0 ? this.equipe.joueur(k) : j;
      if (ouvreur.mort || (c.x - ouvreur.x) ** 2 + (c.z - ouvreur.z) ** 2 > r.rayonOuverture * r.rayonOuverture) continue;
      const prix = this.prix(c);
      if (etat.emeraudes < prix) continue;
      // Ouverture : on paie, le couvercle se lève, les cartes suivent
      const def = COFFRES[IDS_COFFRES[c.indexType]];
      etat.emeraudes -= prix;
      if (def.payant) this.ouverts++;
      c.ouvert = 0;
      etat.stats.coffresOuverts++;
      this.retours.son(def.son);
      this.particules.emettre(c.x, 1, c.z, GERBES.niveau);
    }
  }

  majVisuel(temps: number): void {
    const r = REGLES_COFFRES;
    const etat = this.etat;
    const pose = this.pose;
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].commencer();
    for (let i = 0; i < r.capacite; i++) {
      const etiquette = this.etiquettes[i];
      if (i >= this.pool.nombre) {
        etiquette.placer(0, 0, 0, false);
        continue;
      }
      const c = this.pool.elements[i];
      // Couvercle : se lève en douceur à l'ouverture (vers l'arrière)
      const p = c.ouvert < 0 ? 0 : Math.min(1, c.ouvert / r.dureeOuverture);
      const ouverture = -(1 - (1 - p) * (1 - p)) * Math.PI * 0.45;
      pose.rotations.fill(0);
      for (let k = 0; k < this.partiesCouvercle.length; k++) pose.rotations[this.partiesCouvercle[k] * 3] = ouverture;
      // Retourné (voir le modèle) : loquet en −Z, d'où le demi-tour
      pose.x = c.x;
      pose.z = c.z;
      pose.y = HAUTEUR;
      pose.echelleY = -1;
      pose.angle = c.angle + Math.PI;
      this.rendus[c.indexType].ajouter(pose);

      // Prix au-dessus du coffre fermé, vert s'il est payable
      const prix = this.prix(c);
      const abordable = etat.emeraudes >= prix;
      const affichage = prix * 2 + (abordable ? 1 : 0);
      if (affichage !== this.affichees[i]) {
        this.affichees[i] = affichage;
        etiquette.ecrire(prix > 0 ? `${prix} émeraude${prix > 1 ? 's' : ''}` : 'Gratuit', abordable ? PRIX_ABORDABLE : PRIX_TROP_CHER);
      }
      etiquette.placer(c.x, 1.1 + Math.sin(temps * 3 + i) * 0.05, c.z, c.ouvert < 0);
    }
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].terminer();
  }

  /** Prix actuel du coffre : le même pour tous les coffres payants, il monte à chaque ouverture. */
  private prix(c: Coffre): number {
    if (!COFFRES[IDS_COFFRES[c.indexType]].payant) return 0;
    return REGLES_COFFRES.prix.base + REGLES_COFFRES.prix.increment * this.ouverts;
  }

  /** Case accessible au hasard entre `min` et `max` blocs de (x, z). */
  private trouverCase(x: number, z: number, min: number, max: number): boolean {
    for (let essai = 0; essai < 24; essai++) {
      const angle = Math.random() * Math.PI * 2;
      const d = min + Math.random() * (max - min);
      const cx = Math.floor(x + Math.sin(angle) * d) + 0.5;
      const cz = Math.floor(z + Math.cos(angle) * d) + 0.5;
      if (!this.accessible(cx, cz)) continue;
      this.caseX = cx;
      this.caseZ = cz;
      return true;
    }
    return false;
  }

  private accessible(x: number, z: number): boolean {
    const { carte, flux } = this.terrain;
    const demiX = this.etat.arene.largeur / 2 - 1;
    const demiZ = this.etat.arene.profondeur / 2 - 1;
    if (Math.abs(x) > demiX || Math.abs(z) > demiZ) return false;
    return !carte.bloqueMarche(x, z) && carte.degatsSol(x, z) === 0 && flux.atteignable(x, z);
  }
}
