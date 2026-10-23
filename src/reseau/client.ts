/**
 * Partie à plusieurs, côté joueur qui a rejoint : seul l'hôte décide des coups,
 * des morts et du butin. Le joueur envoie ses touches à chaque pas, prédit sa
 * propre marche (même carte de collision que l'hôte) et la corrige peu à peu
 * d'après la position que l'hôte renvoie.
 *
 * Les ennemis, les orbes, les objets et les coffres viennent des instantanés :
 * ils remplissent les pools locaux (affichés par leur propre majVisuel),
 * interpolés d'un instantané à l'autre. Les armes, elles, tournent ici aussi,
 * les siennes et celles des autres (leurs cartes arrivent de l'hôte), mais
 * pour l'effet seulement : ici, un coup ne blesse personne. Les vrais dégâts
 * arrivent en chiffres avec les instantanés. L'option « Effets des autres
 * joueurs » coupe la simulation de leurs armes.
 */
import { CONFIG } from '../config';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { MODELES } from '../donnees/modeles';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { GERBES } from '../donnees/particules';
import type { Ombres } from '../effets/ombres';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { Retours } from '../effets/retours';
import { effacerEtats, type Ennemi } from '../entites/ennemi';
import { avancerAnimation, reinitialiserAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import type { Options } from '../options';
import type { Amelioration } from '../progression/ameliorations';
import type { BilanPartie } from '../progression/sauvegarde';
import { ecartAngle } from '../utils/maths';
import { positionDepart } from './apparence';
import { SuiviLocal } from './bandeau';
import { Combattant, Commutateur, type PartageCombat } from './combattant';
import { ameliorationValide } from './hote';
import type { CrochetsReseau, ElementsPartie } from './partie';
import { enregistrerRecordsEquipe } from './records';
import {
  appliquerStatistiques, BITS_ACTIONS, ENNEMI_BLESSE, ENNEMI_BRULURE, ENNEMI_MORT, ENNEMI_POISON, ENNEMI_RALENTI, ENNEMI_SAIGNEMENT,
  Ecrivain, JOUEUR_EN_CHOIX, JOUEUR_MORT, JOUEUR_PROTEGE, Lecteur, STYLES_CHIFFRES, TYPE_ENTREE, TYPE_INSTANTANE, type MessageFiable,
} from './protocole';
import type { ObservateurPartie, SessionReseau } from './session';

/** Entrées envoyées dont on garde la position prédite (pour les comparer à celle de l'hôte). */
const HISTORIQUE = 128;
const CAPACITE_PROJECTILES = 2048;
/** Sons des autres joueurs qu'on n'entend pas ici (ils concernent leur corps à eux). */
const SONS_PERSONNELS = new Set(['blessure_joueur', 'totem', 'orbe', 'emeraude', 'manger', 'niveau']);

/** Position et orientation d'un autre joueur aux deux derniers instantanés. */
interface Trajet {
  x0: number; z0: number; a0: number;
  x1: number; z1: number; a1: number;
}

/** Un ennemi affiché, d'après son identifiant chez l'hôte. */
interface SuiviEnnemi extends Trajet {
  readonly e: Ennemi;
  /** Numéro du dernier instantané qui le contenait. */
  vu: number;
}

export class PartieClient implements CrochetsReseau, ObservateurPartie {
  readonly client = true;
  readonly facteurApparitions = 1;
  pauseLocale = false;

  private readonly session: SessionReseau;
  private readonly elements: ElementsPartie;
  /** Les autres joueurs : leur modèle et (pour l'effet) leurs armes, par index et en liste. */
  private readonly autres: (Combattant | null)[] = [];
  private readonly listeAutres: Combattant[] = [];
  /** Trajet de chaque autre joueur, par index de joueur. */
  private readonly trajets: Trajet[] = [];
  /** Suivi de chaque ennemi affiché, à la même place que lui dans le pool des ennemis (échangés ensemble). */
  private readonly suivisPool: SuiviEnnemi[] = [];
  private readonly suivis = new Map<number, SuiviEnnemi>();
  private readonly modeles: (ModeleCompile | null)[] = IDS_ENNEMIS.map(() => null);
  private readonly ecrivain = new Ecrivain(16);
  private readonly lecteur = new Lecteur();
  /** Entrées envoyées : numéro, x et z prédits à l'envoi. */
  private readonly historique = new Float64Array(HISTORIQUE * 3);
  private numeroEntree = 0;
  /** Dernier instantané reçu, appliqué au début de l'image suivante. */
  private enAttente: ArrayBuffer | null = null;
  private dernierNumero = 0;
  private arrivee = 0;
  private readonly projectiles = new Float32Array(CAPACITE_PROJECTILES * 5);
  private nbProjectiles = 0;
  private readonly suiviLocal = new SuiviLocal();
  private cartesOuvertes = false;
  private effetsAllies = true;
  /** Réanimation du joueur local (0 à 1), d'après l'hôte. */
  private reanimation = 0;
  /** La reprise en mode sans fin vient de l'hôte (ne pas la lui redemander). */
  private repriseRecue = false;
  private dtReel = 0;
  private minuterieBouclier = 0;

  constructor(session: SessionReseau, elements: ElementsPartie) {
    this.session = session;
    this.elements = elements;
    const { etat, ennemis } = elements;
    positionDepart(etat.arene, session.index, etat.joueur);
    // Les coups ne se décident que chez l'hôte : ici, le joueur n'est jamais blessé et les armes ne blessent personne
    elements.joueur.invincible = true;
    ennemis.coupsActifs = false;

    const partage: PartageCombat = {
      etat,
      carte: elements.terrain.carte,
      ennemis,
      projectilesEnnemis: elements.projectilesEnnemis,
      orbes: elements.orbes,
      particules: elements.particules,
      scene: elements.scene,
      camera: elements.camera,
      textures: elements.textures,
      commutateur: new Commutateur(etat, ennemis, elements.mecaniques),
    };
    const retours = elements.retours;
    const retoursAutres: Retours = {
      son: (id, hauteur) => {
        if (!SONS_PERSONNELS.has(id)) retours.son(id, hauteur);
      },
      secousse: () => {},
      degats: () => {},
    };
    for (const info of session.joueurs) {
      if (info.index === session.index) continue;
      const c = new Combattant(info, this.listeAutres.length + 1, partage, retoursAutres);
      c.joueur.invincible = true;
      const j = c.etatJoueur;
      this.autres[info.index] = c;
      this.listeAutres.push(c);
      this.trajets[info.index] = { x0: j.x, z0: j.z, a0: 0, x1: j.x, z1: j.z, a1: 0 };
    }
    for (let i = 0; i < CONFIG.ennemis.capacite; i++) {
      this.suivisPool.push({ e: ennemis.pool.elements[i], x0: 0, z0: 0, a0: 0, x1: 0, z1: 0, a1: 0, vu: 0 });
    }
    session.partie = this;
  }

  // --- Simulation locale -----------------------------------------------------

  simulerClient(dt: number): void {
    const { etat, joueur, entrees } = this.elements;
    // Pendant le choix des cartes, le joueur reste sur place (protégé chez l'hôte)
    if (!this.pauseLocale) joueur.maj(dt);
    etat.tempsJeu += dt;
    for (let i = 0; i < this.listeAutres.length; i++) this.listeAutres[i].majJoueur(dt);

    // Entrée envoyée à chaque pas, avec la position prédite qu'on lui associe
    const n = ++this.numeroEntree;
    const h = (n % HISTORIQUE) * 3;
    this.historique[h] = n;
    this.historique[h + 1] = etat.joueur.x;
    this.historique[h + 2] = etat.joueur.z;
    const w = this.ecrivain;
    w.position = 0;
    w.u8(TYPE_ENTREE);
    w.u32(n);
    w.u8(this.pauseLocale ? 0 : (entrees.action('haut') ? BITS_ACTIONS.haut : 0) | (entrees.action('bas') ? BITS_ACTIONS.bas : 0)
      | (entrees.action('gauche') ? BITS_ACTIONS.gauche : 0) | (entrees.action('droite') ? BITS_ACTIONS.droite : 0));
    w.f32(joueur.lacetCommandes);
    this.session.envoyerDonnees(w.contenu());
  }

  voit(): boolean {
    return false;
  }

  majJoueurs(): void {}

  /** Armes des autres joueurs, pour l'effet (option « Effets des autres joueurs »). */
  majArmes(dt: number): void {
    if (!this.effetsAllies) return;
    for (let i = 0; i < this.listeAutres.length; i++) {
      const c = this.listeAutres[i];
      if (!c.enChoix && !c.etatJoueur.mort) c.majArmes(dt);
    }
  }

  finPas(): void {}

  vitesseTemps(cartesOuvertes: boolean): number {
    const etat = this.elements.etat;
    this.cartesOuvertes = cartesOuvertes;
    this.pauseLocale = etat.issue === null && (cartesOuvertes || etat.niveauxEnAttente > 0 || etat.choixCoffre.restants > 0);
    return 1;
  }

  surCarte(amelioration: Amelioration): void {
    this.session.envoyer({ type: 'carte', amelioration });
  }

  surContinuer(): void {
    if (!this.repriseRecue) this.session.envoyer({ type: 'continuer' });
    this.repriseRecue = false;
  }

  appliquerOptions(options: Options): void {
    this.effetsAllies = options.effetsAllies;
  }

  surFinPartie(bilan: BilanPartie): void {
    enregistrerRecordsEquipe(bilan);
  }

  surDegats(): void {}

  // --- Messages de l'hôte ----------------------------------------------------

  surDonnees(_index: number, donnees: ArrayBuffer): void {
    if (donnees.byteLength < 5) return;
    const vue = new DataView(donnees);
    if (vue.getUint8(0) !== TYPE_INSTANTANE) return;
    // Le plus récent seulement (canal sans ordre)
    if (vue.getUint32(1) <= this.dernierNumero) return;
    this.enAttente = donnees;
  }

  surMessage(_index: number, message: MessageFiable): void {
    const { etat } = this.elements;
    switch (message.type) {
      case 'stats':
        appliquerStatistiques(etat.stats, message.stats);
        break;
      case 'fin':
        appliquerStatistiques(etat.stats, message.stats);
        if (etat.issue === null) {
          etat.tempsJeu = message.temps;
          etat.issue = message.issue;
          this.elements.conclure();
        }
        break;
      case 'coffre':
        // Coffre ouvert par n'importe qui : nos cartes, tirées ici comme en solo
        etat.choixCoffre.restants += message.nombre;
        etat.choixCoffre.titre = message.titre;
        break;
      case 'carteJouee': {
        const c = this.autres[message.index];
        if (!c || !ameliorationValide(message.amelioration)) break;
        try {
          c.appliquerCarte(message.amelioration);
        } catch {
          // Arsenal désaccordé : seul l'effet des armes en pâtit
        }
        break;
      }
      case 'reprise':
        if (etat.issue === 'victoire') {
          this.repriseRecue = true;
          this.elements.continuerSansFin();
        }
        break;
    }
  }

  surDepart(): void {
    this.suiviLocal.signalerConnexionPerdue();
  }

  // --- Affichage ---------------------------------------------------------------

  avantImage(dtReel: number): void {
    this.dtReel = dtReel;
    if (this.enAttente) {
      this.appliquer(this.enAttente);
      this.enAttente = null;
      this.arrivee = performance.now() / 1000;
    }
    const periode = 1 / REGLES_MULTIJOUEUR.reseau.instantanesParSeconde;
    const t = Math.min(1, (performance.now() / 1000 - this.arrivee) / periode);

    for (let i = 0; i < this.listeAutres.length; i++) {
      const c = this.listeAutres[i];
      const a = this.trajets[c.info.index];
      const j = c.etatJoueur;
      j.x = a.x0 + (a.x1 - a.x0) * t;
      j.z = a.z0 + (a.z1 - a.z0) * t;
      j.angle = a.a0 + ecartAngle(a.a0, a.a1) * t;
    }
    const pool = this.elements.ennemis.pool;
    for (let i = 0; i < pool.nombre; i++) {
      const s = this.suivisPool[i];
      const e = s.e;
      e.x = s.x0 + (s.x1 - s.x0) * t;
      e.z = s.z0 + (s.z1 - s.z0) * t;
      e.angle = s.a0 + ecartAngle(s.a0, s.a1) * t;
      if (e.mort >= 0) e.mort += dtReel;
      avancerAnimation(e.animation, this.modele(e.indexType), dtReel, e.vitesseSol / (e.definition.echelle ?? 1));
    }
    // Les armes (pour l'effet) cherchent leurs cibles dans la grille
    this.elements.ennemis.reconstruireGrille();

    if (this.suiviLocal.maj(this.cartesOuvertes, this.elements.etat.joueur, this.reanimation)) this.elements.choisirCarte();
  }

  majVisuelOmbres(ombres: Ombres): void {
    for (let i = 0; i < this.listeAutres.length; i++) this.listeAutres[i].majVisuelOmbres(ombres, this.dtReel, this.effetsAllies);
    // Bouclier du joueur local pendant qu'il choisit ses cartes
    if (this.pauseLocale && (this.minuterieBouclier -= this.dtReel) <= 0) {
      this.minuterieBouclier = REGLES_MULTIJOUEUR.choixCartes.periodeBouclier;
      const j = this.elements.etat.joueur;
      this.elements.particules.emettre(j.x, 1, j.z, GERBES.arcane);
    }
  }

  majVisuelProjectiles(rendu: RenduProjectiles): void {
    // Flèches des ennemis, telles que l'hôte les dessine
    for (let i = 0; i < this.nbProjectiles; i++) {
      const o = i * 5;
      rendu.ajouter(this.projectiles[o], this.projectiles[o + 1], this.projectiles[o + 2], this.projectiles[o + 3], this.projectiles[o + 4]);
    }
    if (this.effetsAllies) for (let i = 0; i < this.listeAutres.length; i++) this.listeAutres[i].majVisuelProjectiles(rendu);
  }

  majVisuelEffets(dtJeu: number): void {
    if (this.effetsAllies) for (let i = 0; i < this.listeAutres.length; i++) this.listeAutres[i].majVisuelEffets(dtJeu);
  }

  // --- Instantanés -------------------------------------------------------------

  private appliquer(donnees: ArrayBuffer): void {
    const l = this.lecteur;
    const { etat, ennemis, orbes, objets, coffres, chiffres, particules, retours } = this.elements;
    l.lire(donnees);
    l.u8();
    const numero = l.u32();
    if (numero <= this.dernierNumero) return;
    this.dernierNumero = numero;
    const accuse = l.u32();
    etat.tempsJeu = l.f32();
    etat.ennemisTues = l.u32();
    const niveau = l.u16();
    const moi = etat.joueur;
    // Niveau commun : chaque niveau gagné ouvre ses cartes, tirées et appliquées ici comme en solo
    if (niveau > moi.niveau) etat.niveauxEnAttente += niveau - moi.niveau;
    moi.niveau = niveau;
    moi.xp = l.f32();
    moi.xpNiveau = l.f32();
    etat.emeraudes = l.u32();

    // Équipe
    const nbJoueurs = l.u8();
    for (let k = 0; k < nbJoueurs; k++) {
      const index = l.u8();
      const x = l.f32();
      const z = l.f32();
      const angle = l.angle();
      const pv = l.f32();
      const pvMax = l.f32();
      const bouclier = l.f32();
      const drapeaux = l.u8();
      const tempsMort = l.f32();
      const invulnerabilite = l.f32();
      const bits = l.u8();
      const lacet = l.f32();
      const reanimation = l.part();
      const autre = this.autres[index];
      const j = index === this.session.index ? moi : autre?.etatJoueur;
      if (!j) continue;
      j.pv = pv;
      j.pvMax = pvMax;
      j.bouclier = bouclier;
      j.mort = (drapeaux & JOUEUR_MORT) !== 0;
      j.protection = (drapeaux & JOUEUR_PROTEGE) !== 0 ? 1 : 0;
      j.tempsMort = tempsMort;
      j.invulnerabilite = invulnerabilite;
      if (autre) {
        const a = this.trajets[index];
        a.x0 = j.x;
        a.z0 = j.z;
        a.a0 = j.angle;
        a.x1 = x;
        a.z1 = z;
        a.a1 = angle;
        autre.commandes.bits = bits;
        autre.joueur.lacetCommandes = lacet;
        autre.enChoix = (drapeaux & JOUEUR_EN_CHOIX) !== 0;
        autre.reanimation = reanimation;
      } else {
        this.reanimation = reanimation;
        this.corrigerPrediction(accuse, x, z);
      }
    }

    // Ennemis
    let n = l.u16();
    for (let i = 0; i < n; i++) {
      const id = l.u32();
      const type = l.u8();
      const x = l.coordonnee();
      const z = l.coordonnee();
      const angle = l.angle();
      const pv = l.part();
      const drapeaux = l.u8();
      const meche = l.part();
      const mort = l.u8() / 100;
      const vitesse = l.u8() / 16;
      let s: SuiviEnnemi | null | undefined = this.suivis.get(id);
      if (!s) {
        s = this.nouvelEnnemi(id, type, x, z, angle);
        if (!s) continue;
      }
      const e = s.e;
      s.x0 = e.x;
      s.z0 = e.z;
      s.a0 = e.angle;
      s.x1 = x;
      s.z1 = z;
      s.a1 = angle;
      s.vu = numero;
      e.pv = pv;
      e.pvMax = 1;
      e.mort = (drapeaux & ENNEMI_MORT) !== 0 ? Math.max(e.mort, mort) : -1;
      e.blessure = (drapeaux & ENNEMI_BLESSE) !== 0 ? 0.1 : 0;
      e.brulure.cumuls = (drapeaux & ENNEMI_BRULURE) !== 0 ? 1 : 0;
      e.poison.cumuls = (drapeaux & ENNEMI_POISON) !== 0 ? 1 : 0;
      e.saignement.cumuls = (drapeaux & ENNEMI_SAIGNEMENT) !== 0 ? 1 : 0;
      e.ralentiTemps = (drapeaux & ENNEMI_RALENTI) !== 0 ? 1 : 0;
      const c = e.definition.comportement;
      e.meche = c.type === 'explosif' ? meche * c.duree : 0;
      e.vitesseSol = vitesse;
    }
    // Disparus de l'instantané : morts (gerbe), explosés (creeper) ou trop loin
    const pool = ennemis.pool;
    for (let i = pool.nombre - 1; i >= 0; i--) {
      const s = this.suivisPool[i];
      if (s.vu === numero) continue;
      const e = s.e;
      if (e.mort >= 0) particules.emettre(e.x, 0.6, e.z, GERBES.mort);
      else if (e.meche > 0) {
        particules.emettre(e.x, 0.8, e.z, GERBES.explosion);
        retours.son('explosion');
      }
      this.suivis.delete(e.id);
      // Le suivi suit son ennemi dans l'échange du pool
      const dernier = pool.nombre - 1;
      this.suivisPool[i] = this.suivisPool[dernier];
      this.suivisPool[dernier] = s;
      pool.liberer(i);
    }

    // Orbes, objets et coffres : reconstruits à chaque instantané
    n = l.u16();
    orbes.pool.nombre = 0;
    for (let i = 0; i < n; i++) {
      const x = l.coordonnee();
      const z = l.coordonnee();
      const valeur = l.u16();
      const o = orbes.pool.acquerir();
      if (!o) continue;
      o.x = x;
      o.z = z;
      o.valeur = valeur;
      o.aimantee = false;
      o.phase = x * 3.7 + z * 5.3;
    }
    n = l.u16();
    objets.pool.nombre = 0;
    for (let i = 0; i < n; i++) {
      const type = l.u8();
      const x = l.coordonnee();
      const z = l.coordonnee();
      const valeur = l.u8();
      const o = objets.pool.acquerir();
      if (!o) continue;
      o.indexType = type;
      o.x = x;
      o.z = z;
      o.valeur = valeur;
      o.phase = x + z;
      o.aimante = false;
    }
    coffres.ouverts = l.u16();
    n = l.u16();
    coffres.pool.nombre = 0;
    for (let i = 0; i < n; i++) {
      const type = l.u8();
      const x = l.coordonnee();
      const z = l.coordonnee();
      const angle = l.angle();
      const ouvert = l.f32();
      const o = coffres.pool.acquerir();
      if (!o) continue;
      o.indexType = type;
      o.x = x;
      o.z = z;
      o.angle = angle;
      o.ouvert = ouvert;
    }

    // Flèches des ennemis, redessinées jusqu'au prochain instantané
    n = l.u16();
    this.nbProjectiles = 0;
    for (let i = 0; i < n; i++) {
      const index = l.u8();
      const x = l.coordonnee();
      const y = l.u8() / 32;
      const z = l.coordonnee();
      const angle = l.angle();
      if (this.nbProjectiles >= CAPACITE_PROJECTILES) continue;
      const o = this.nbProjectiles++ * 5;
      this.projectiles[o] = index;
      this.projectiles[o + 1] = x;
      this.projectiles[o + 2] = y;
      this.projectiles[o + 3] = z;
      this.projectiles[o + 4] = angle;
    }

    // Chiffres de dégâts (les vrais, décidés par l'hôte)
    n = l.u16();
    for (let i = 0; i < n; i++) {
      const x = l.coordonnee();
      const y = l.u8() / 16;
      const z = l.coordonnee();
      const valeur = l.f32();
      const style = STYLES_CHIFFRES[l.u8()] ?? 'normal';
      chiffres.ajouter(x, y, z, valeur, style);
    }
  }

  /**
   * L'hôte renvoie notre position après l'entrée `accuse` : on la compare à
   * celle prédite au même moment, et on corrige une part de l'écart (le reste
   * aux instantanés suivants) ; trop loin, on se replace d'un coup.
   */
  private corrigerPrediction(accuse: number, x: number, z: number): void {
    const j = this.elements.etat.joueur;
    const h = (accuse % HISTORIQUE) * 3;
    const p = REGLES_MULTIJOUEUR.prediction;
    let ex = x - j.x;
    let ez = z - j.z;
    if (accuse > 0 && this.historique[h] === accuse) {
      ex = x - this.historique[h + 1];
      ez = z - this.historique[h + 2];
    }
    if (ex * ex + ez * ez > p.teleportation * p.teleportation) {
      j.x = x;
      j.z = z;
      this.historique.fill(0);
      return;
    }
    const cx = ex * p.correction;
    const cz = ez * p.correction;
    j.x += cx;
    j.z += cz;
    for (let i = 0; i < HISTORIQUE; i++) {
      this.historique[i * 3 + 1] += cx;
      this.historique[i * 3 + 2] += cz;
    }
  }

  private nouvelEnnemi(id: number, type: number, x: number, z: number, angle: number): SuiviEnnemi | null {
    if (type >= IDS_ENNEMIS.length) return null;
    const pool = this.elements.ennemis.pool;
    const e = pool.acquerir();
    if (!e) return null;
    const s = this.suivisPool[pool.nombre - 1];
    e.id = id;
    e.indexType = type;
    e.definition = ENNEMIS[IDS_ENNEMIS[type]];
    e.x = x;
    e.z = z;
    e.angle = angle;
    e.vx = 0;
    e.vz = 0;
    e.mort = -1;
    e.meche = 0;
    e.blessure = 0;
    e.immunites.fill(0);
    effacerEtats(e);
    reinitialiserAnimation(e.animation);
    this.suivis.set(id, s);
    return s;
  }

  private modele(indexType: number): ModeleCompile {
    let modele = this.modeles[indexType];
    if (!modele) {
      modele = compilerModele(MODELES[ENNEMIS[IDS_ENNEMIS[indexType]].modele]);
      this.modeles[indexType] = modele;
    }
    return modele;
  }
}
