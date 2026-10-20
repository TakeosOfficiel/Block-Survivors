/**
 * Mécaniques des talents du Mage.
 * Pyromancien : Cœur de blaze, brûlures (Propagation, Brasier, Fournaise,
 * Cendres protectrices), Embrasement, Phénix. Lave en fusion, Tempête de feu
 * et Pluie de feu sont lues par le système des météores.
 * Arcaniste : Résonance, Surcharge, Accélération, Tempête d'orage, Écho
 * arcanique, Zéro absolu. Haute tension, Conduction, Orbes givrées et Flux
 * arcanique sont lus par les systèmes des éclairs et des orbes.
 */
import { ARMES, IDS_ARMES } from '../../donnees/armes';
import { GERBES } from '../../donnees/particules';
import type { Ennemi } from '../../entites/ennemi';
import type { NatureCoup } from '../../entites/ennemis';
import { calculerStats, type ArmeEquipee, type StatsCalculees } from '../armeEquipee';
import { indexArme, MecaniquesClasse, type Talents } from './talents';

/** Valeurs des talents (provisoires), par rang quand il y en a deux. */
const V = {
  coeurBlaze: { cumuls: 1, degatsParSeconde: 3, duree: 3 },
  cendres: { bouclier: 1, delaiInterne: 1 },
  propagation: { voisins: [1, 2], part: [0.5, 1], rayon: 3 },
  brasier: { degatsCritiques: 0.3 },
  fournaise: { critique: 0.08 },
  embrasement: { brulants: 15, multiplicateur: 1.5, rayon: 2, delaiInterne: 10, periode: 0.25 },
  phenix: { pv: 0.4, rayon: 5, degats: 40, protection: 2, force: 14 },
  resonance: { fenetre: 3, parSort: 0.05, maximum: 0.15 },
  surcharge: { etourdi: [0.3, 0.5] },
  acceleration: { sorts: 10, delai: 0.8, duree: [2, 3] },
  orage: { periode: 20, eclairs: 8, duree: 2, portee: 10 },
  echo: { sorts: 6, delai: 0.5, part: 0.6, delaiInterne: 3 },
  zeroAbsolu: { novas: 4 },
} as const;

const BOULES = indexArme('boules_feu');
const ECLAIRS = indexArme('eclairs');
const NOVA = indexArme('nova_glace');
/** Sorts du Mage (index dans IDS_ARMES). */
const SORTS: ReadonlySet<number> = new Set((['boules_feu', 'eclairs', 'tornade_feu', 'nova_glace', 'meteore', 'orbes_arcaniques'] as const).map(indexArme));

export class MecaniquesMage extends MecaniquesClasse {
  private readonly tampon = new Int32Array(256);
  private readonly derniersSorts = new Float32Array(IDS_ARMES.length).fill(-1e9);
  private delaiCendres = 0;
  private minuterieEmbrasement = 0;
  private delaiEmbrasement = 0;
  private phenixUtilise = false;
  private sortsAcceleration = 0;
  private acceleration = 0;
  private minuterieOrage: number = V.orage.periode;
  private eclairsOrage = 0;
  private prochainEclair = 0;
  private readonly statsEclair: StatsCalculees = { ...ARMES.eclairs.stats };
  private sortsEcho = 0;
  private echoPret = false;
  private delaiEcho = 0;
  private echoArme: ArmeEquipee | null = null;
  private echoMinuterie = 0;
  private novas = 0;
  private gelApplique = false;

  constructor(talents: Talents) {
    super(talents);
  }

  override maj(dt: number): void {
    const t = this.t;
    this.delaiCendres -= dt;
    this.delaiEmbrasement -= dt;
    this.delaiEcho -= dt;
    if (this.acceleration > 0) this.acceleration -= dt;

    // Embrasement : 15 ennemis en feu explosent ensemble
    if (t.rang('embrasement') > 0 && (this.minuterieEmbrasement -= dt) <= 0) {
      this.minuterieEmbrasement = V.embrasement.periode;
      if (this.delaiEmbrasement <= 0) this.embrasement();
    }

    // Tempête d'orage : 8 éclairs en 2 s toutes les 20 s
    if (t.rang('tempete_orage') > 0) {
      if ((this.minuterieOrage -= dt) <= 0) {
        this.minuterieOrage = V.orage.periode;
        this.eclairsOrage = V.orage.eclairs;
        this.prochainEclair = 0;
      }
      if (this.eclairsOrage > 0 && (this.prochainEclair -= dt) <= 0) {
        this.prochainEclair = V.orage.duree / V.orage.eclairs;
        this.eclairsOrage--;
        this.eclairOrage();
      }
    }

    // Écho arcanique : le sort se répète, plus faible
    if (this.echoArme && (this.echoMinuterie -= dt) <= 0) {
      const arme = this.echoArme;
      this.echoArme = null;
      if (t.arsenal.armes.includes(arme)) {
        t.facteurDeclenchement = V.echo.part;
        t.arsenal.declencher(arme);
        t.facteurDeclenchement = 1;
      }
    }
  }

  // --- Déclenchements -------------------------------------------------------

  override avantDeclenchement(arme: ArmeEquipee): void {
    if (arme.index === NOVA && this.t.rang('zero_absolu') > 0 && this.novas >= V.zeroAbsolu.novas) {
      this.t.novaGel = true;
      this.gelApplique = true;
    }
  }

  override apresDeclenchement(arme: ArmeEquipee, declenchee: boolean): void {
    const t = this.t;
    if (arme.index === NOVA && t.rang('zero_absolu') > 0) {
      if (declenchee) this.novas = this.gelApplique ? 0 : this.novas + 1;
      this.gelApplique = false;
      t.novaGel = false;
    }
    if (!declenchee || !SORTS.has(arme.index)) return;
    this.derniersSorts[arme.index] = t.etat.tempsJeu;
    // Accélération : tous les 10 sorts
    if (t.rang('acceleration') > 0 && ++this.sortsAcceleration >= V.acceleration.sorts) {
      this.sortsAcceleration = 0;
      this.acceleration = t.valeur('acceleration', V.acceleration.duree);
    }
    // Écho arcanique : après 6 sorts, le suivant se répète 0,5 s plus tard
    if (t.rang('echo_arcanique') > 0) {
      if (this.echoPret && this.delaiEcho <= 0 && !this.echoArme) {
        this.echoPret = false;
        this.delaiEcho = V.echo.delaiInterne;
        this.echoArme = arme;
        this.echoMinuterie = V.echo.delai;
        const j = t.etat.joueur;
        t.contexte.particules.emettre(j.x, 1.2, j.z, GERBES.impactArcane);
      } else if (!this.echoPret && ++this.sortsEcho >= V.echo.sorts) {
        this.sortsEcho = 0;
        this.echoPret = true;
      }
    }
  }

  override delai(arme: ArmeEquipee): number {
    return this.acceleration > 0 && SORTS.has(arme.index) ? V.acceleration.delai : 1;
  }

  // --- Coups portés ---------------------------------------------------------

  override multiplicateur(_e: Ennemi, arme: number, nature: NatureCoup): number {
    const t = this.t;
    if (arme < 0 || nature !== 'arme' || t.rang('resonance') === 0) return 1;
    // Résonance : +5 % par sort différent lancé dans les 3 s
    let sorts = 0;
    for (const index of SORTS) if (t.etat.tempsJeu - this.derniersSorts[index] <= V.resonance.fenetre) sorts++;
    return 1 + Math.min(V.resonance.maximum, sorts * V.resonance.parSort);
  }

  override bonusCritique(e: Ennemi, _arme: number, _nature: NatureCoup): number {
    return e.brulure.cumuls > 0 && this.t.rang('fournaise') > 0 ? V.fournaise.critique : 0;
  }

  override facteurCritique(e: Ennemi, _arme: number, _nature: NatureCoup, base: number): number {
    return e.brulure.cumuls > 0 && this.t.rang('brasier') > 0 ? base + V.brasier.degatsCritiques : base;
  }

  override apresCoup(e: Ennemi, _degats: number, arme: number, critique: boolean, nature: NatureCoup): void {
    const t = this.t;
    if (nature !== 'arme' || e.mort >= 0) return;
    // Cœur de blaze : les Boules de feu enflamment
    if (arme === BOULES && t.rang('coeur_blaze') > 0) {
      t.contexte.ennemis.bruler(e, V.coeurBlaze.cumuls, V.coeurBlaze.degatsParSeconde, V.coeurBlaze.duree, arme);
    }
    // Surcharge : un éclair critique étourdit
    if (arme === ECLAIRS && critique) {
      const duree = t.valeur('surcharge', V.surcharge.etourdi);
      if (duree > 0) t.contexte.ennemis.etourdir(e, duree);
    }
  }

  override surMort(e: Ennemi, _arme: number, _nature: NatureCoup): void {
    const t = this.t;
    const brulure = e.brulure;
    if (brulure.cumuls === 0) return;
    // Cendres protectrices : un ennemi en feu tué donne du bouclier
    if (t.rang('cendres_protectrices') > 0 && this.delaiCendres <= 0) {
      this.delaiCendres = V.cendres.delaiInterne;
      t.bouclier(V.cendres.bouclier);
    }
    // Propagation : ses brûlures passent à ses voisins (la moitié des cumuls par transmission)
    const voisins = t.valeur('propagation', V.propagation.voisins);
    if (voisins === 0) return;
    const ennemis = t.contexte.ennemis;
    const part = t.valeur('propagation', V.propagation.part);
    const cumuls = Math.max(1, Math.round(brulure.cumuls * part * 0.5));
    const dps = brulure.degatsParSeconde / (1 + t.etat.modificateurs.degatsBrulure);
    const duree = brulure.temps / (1 + t.etat.modificateurs.dureeBrulure);
    const n = ennemis.chercher(e.x, e.z, V.propagation.rayon, this.tampon);
    let faits = 0;
    for (let k = 0; k < n && faits < voisins; k++) {
      const o = ennemis.obtenir(this.tampon[k]);
      if (o === e || o.mort >= 0) continue;
      ennemis.bruler(o, cumuls, dps, Math.max(1, duree), brulure.arme);
      t.contexte.particules.emettre(o.x, 0.8, o.z, GERBES.brulure);
      faits++;
    }
  }

  override eviterMort(): boolean {
    // Phénix : une fois par partie, explosion de feu et retour à 40 % des PV
    const t = this.t;
    if (t.rang('phenix') === 0 || this.phenixUtilise) return false;
    this.phenixUtilise = true;
    const j = t.etat.joueur;
    j.pv = Math.ceil(j.pvMax * V.phenix.pv);
    j.protection = Math.max(j.protection, V.phenix.protection);
    t.blesserZone(j.x, j.z, V.phenix.rayon, V.phenix.degats, BOULES, 0, this.tampon);
    const n = t.contexte.ennemis.chercher(j.x, j.z, V.phenix.rayon, this.tampon);
    for (let k = 0; k < n; k++) t.contexte.ennemis.bruler(t.contexte.ennemis.obtenir(this.tampon[k]), 2, 3, 4, BOULES);
    t.contexte.ennemis.repousser(j.x, j.z, V.phenix.rayon, V.phenix.force);
    t.contexte.particules.emettre(j.x, 1, j.z, GERBES.explosion);
    t.contexte.particules.emettre(j.x, 1, j.z, GERBES.totem);
    t.cercle(j.x, j.z, V.phenix.rayon * 0.7, 'brulure', 24);
    t.contexte.retours.son('totem');
    t.contexte.retours.son('explosion');
    t.contexte.retours.secousse(0.6);
    return true;
  }

  /** Embrasement : chaque ennemi en feu explose (1,5 × ce qui reste de sa brûlure). */
  private embrasement(): void {
    const t = this.t;
    const ennemis = t.contexte.ennemis;
    const n = ennemis.brulants(this.tampon);
    if (n < V.embrasement.brulants) return;
    this.delaiEmbrasement = V.embrasement.delaiInterne;
    // Les explosions se suivent : on fige d'abord la liste (ses index restent valides pendant le pas)
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      const b = e.brulure;
      if (b.cumuls === 0) continue;
      const restant = b.degatsParSeconde * b.cumuls * Math.max(0, b.temps) * V.embrasement.multiplicateur;
      const arme = b.arme;
      b.cumuls = 0;
      t.contexte.particules.emettre(e.x, 0.8, e.z, GERBES.explosionArme);
      t.blesserZone(e.x, e.z, V.embrasement.rayon, restant, arme, 3, TAMPON_EXPLOSION);
    }
    t.contexte.retours.son('explosion');
    t.contexte.retours.secousse(0.5);
  }

  /** Un éclair de la Tempête d'orage : les dégâts de l'arme Éclairs (rang actuel, ou rang 1). */
  private eclairOrage(): void {
    const t = this.t;
    const j = t.etat.joueur;
    const cible = t.contexte.ennemis.auHasard(j.x, j.z, V.orage.portee);
    if (!cible) return;
    let stats: StatsCalculees | null = null;
    for (const a of t.arsenal.armes) if (a.index === ECLAIRS) stats = a.stats;
    if (!stats) stats = calculerStats(ARMES.eclairs, 1, t.etat.modificateurs, this.statsEclair, t.etat.talents.armes.eclairs ?? null);
    const m = ARMES.eclairs.mecanique;
    const couleur = m.type === 'foudre' ? m.couleur : 0xffffff;
    t.systemes.foudre.eclairTalent(ECLAIRS, cible.x, cible.z, stats.degats, stats.zone, couleur);
  }
}

const TAMPON_EXPLOSION = new Int32Array(128);
