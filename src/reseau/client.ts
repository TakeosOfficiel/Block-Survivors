/**
 * Partie à plusieurs, côté joueur qui a rejoint : il ne simule ni les ennemis
 * ni les armes. Il envoie ses touches à chaque pas, prédit sa propre marche
 * (même carte de collision que l'hôte) et la corrige peu à peu d'après la
 * position que l'hôte renvoie. Le reste vient des instantanés : les ennemis,
 * les orbes et les objets remplissent les pools locaux (affichés par leur
 * propre majVisuel), interpolés d'un instantané à l'autre ; les projectiles
 * et les chiffres de dégâts sont redessinés tels quels.
 */
import { CONFIG } from '../config';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { MODELES } from '../donnees/modeles';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { GERBES } from '../donnees/particules';
import type { Ombres } from '../effets/ombres';
import type { RenduProjectiles } from '../effets/renduProjectiles';
import type { Retours } from '../effets/retours';
import { Etiquette } from '../effets/etiquette';
import { effacerEtats, type Ennemi } from '../entites/ennemi';
import { Joueur } from '../entites/joueur';
import { creerEtatPartie, type EtatPartie } from '../etat';
import { avancerAnimation, reinitialiserAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import type { Amelioration } from '../progression/ameliorations';
import { ecartAngle } from '../utils/maths';
import { positionDepart, apparenceDistante } from './apparence';
import { CommandesDistantes } from './combattant';
import type { CrochetsReseau, ElementsPartie } from './partie';
import {
  BITS_ACTIONS, ENNEMI_BLESSE, ENNEMI_BRULURE, ENNEMI_MORT, ENNEMI_POISON, ENNEMI_RALENTI, ENNEMI_SAIGNEMENT, Ecrivain,
  INSTANTANE_PAUSE, JOUEUR_MORT, JOUEUR_PROTEGE, Lecteur, STYLES_CHIFFRES, TYPE_ENTREE, TYPE_INSTANTANE, type MessageFiable,
} from './protocole';
import type { ObservateurPartie, SessionReseau } from './session';

/** Entrées envoyées dont on garde la position prédite (pour les comparer à celle de l'hôte). */
const HISTORIQUE = 128;
const CAPACITE_PROJECTILES = 2048;

/** Un autre joueur, affiché d'après les instantanés. */
interface AutreJoueur {
  readonly etat: EtatPartie;
  readonly joueur: Joueur;
  readonly commandes: CommandesDistantes;
  readonly etiquette: Etiquette;
  /** Position et orientation de l'avant-dernier et du dernier instantané. */
  x0: number; z0: number; a0: number;
  x1: number; z1: number; a1: number;
}

/** Un ennemi affiché, d'après son identifiant chez l'hôte. */
interface SuiviEnnemi {
  e: Ennemi;
  x0: number; z0: number; a0: number;
  x1: number; z1: number; a1: number;
  /** Numéro du dernier instantané qui le contenait. */
  vu: number;
}

export class PartieClient implements CrochetsReseau, ObservateurPartie {
  readonly client = true;
  readonly facteurApparitions = 1;

  private readonly session: SessionReseau;
  private readonly elements: ElementsPartie;
  private readonly autres: (AutreJoueur | null)[] = [];
  private readonly listeAutres: AutreJoueur[] = [];
  private readonly suivis = new Map<number, SuiviEnnemi>();
  private readonly actifs: SuiviEnnemi[] = [];
  private readonly libres: SuiviEnnemi[] = [];
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
  private pause = false;
  private readonly projectiles = new Float32Array(CAPACITE_PROJECTILES * 5);
  private nbProjectiles = 0;
  private readonly alerte: HTMLDivElement;

  constructor(session: SessionReseau, elements: ElementsPartie) {
    this.session = session;
    this.elements = elements;
    const { etat, terrain, scene, textures } = elements;
    positionDepart(etat.arene, session.index, etat.joueur);
    // Les coups ne se décident que chez l'hôte : localement, le joueur n'est jamais blessé
    elements.joueur.invincible = true;
    const muet: Retours = { son: () => {}, secousse: () => {}, degats: () => {} };
    for (const info of session.joueurs) {
      if (info.index === session.index) continue;
      const etatAutre = creerEtatPartie(etat.arene, info.personnage);
      positionDepart(etat.arene, info.index, etatAutre.joueur);
      const commandes = new CommandesDistantes();
      const joueur = new Joueur(etatAutre, terrain.carte, commandes, muet, scene, textures, apparenceDistante(info));
      joueur.invincible = true;
      const j = etatAutre.joueur;
      const autre: AutreJoueur = {
        etat: etatAutre, joueur, commandes, etiquette: new Etiquette(scene, info.pseudo),
        x0: j.x, z0: j.z, a0: 0, x1: j.x, z1: j.z, a1: 0,
      };
      this.autres[info.index] = autre;
      this.listeAutres.push(autre);
    }
    for (let i = 0; i < CONFIG.ennemis.capacite; i++) {
      this.libres.push({ e: null as unknown as Ennemi, x0: 0, z0: 0, a0: 0, x1: 0, z1: 0, a1: 0, vu: 0 });
    }
    this.alerte = document.createElement('div');
    this.alerte.className = 'reseau-alerte cache';
    document.body.append(this.alerte);
    session.partie = this;
  }

  // --- Simulation locale -----------------------------------------------------

  simulerClient(dt: number): void {
    const { etat, joueur, entrees } = this.elements;
    joueur.maj(dt);
    etat.tempsJeu += dt;
    for (let i = 0; i < this.listeAutres.length; i++) this.listeAutres[i].joueur.maj(dt);

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
    w.u8((entrees.action('haut') ? BITS_ACTIONS.haut : 0) | (entrees.action('bas') ? BITS_ACTIONS.bas : 0)
      | (entrees.action('gauche') ? BITS_ACTIONS.gauche : 0) | (entrees.action('droite') ? BITS_ACTIONS.droite : 0));
    w.f32(joueur.lacetCommandes);
    this.session.envoyerDonnees(w.contenu());
  }

  majJoueurs(): void {}
  majArmes(): void {}
  finPas(): void {}

  vitesseTemps(cartesOuvertes: boolean): number {
    return this.pause || cartesOuvertes ? 0 : 1;
  }

  surCarte(amelioration: Amelioration): void {
    this.session.envoyer({ type: 'carte', amelioration });
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
    if (message.type !== 'fin') return;
    const { etat } = this.elements;
    const s = etat.stats;
    const r = message.stats;
    s.tuesParType.set(r.tuesParType);
    s.degatsParArme.set(r.degatsParArme);
    s.coupsParArme.set(r.coupsParArme);
    s.critiquesParArme.set(r.critiquesParArme);
    s.plusGrosCoupParArme.set(r.plusGrosCoupParArme);
    s.tuesParArme.set(r.tuesParArme);
    s.acquisitionArme.set(r.acquisitionArme);
    s.degatsSubis = r.degatsSubis;
    s.pvSoignes = r.pvSoignes;
    s.xpTotale = r.xpTotale;
    s.emeraudesRamassees = r.emeraudesRamassees;
    s.coffresOuverts = r.coffresOuverts;
    s.bossVaincus = r.bossVaincus;
    if (etat.issue === null) {
      etat.issue = message.issue;
      this.elements.conclure();
    }
  }

  surDepart(): void {
    this.alerte.textContent = 'Connexion avec l\'hôte perdue';
    this.alerte.classList.remove('cache');
    this.pause = true;
  }

  // --- Affichage ---------------------------------------------------------------

  avantImage(dtReel: number): void {
    if (this.enAttente) {
      this.appliquer(this.enAttente);
      this.enAttente = null;
      this.arrivee = performance.now() / 1000;
    }
    const periode = 1 / REGLES_MULTIJOUEUR.reseau.instantanesParSeconde;
    const t = Math.min(1, (performance.now() / 1000 - this.arrivee) / periode);

    for (let i = 0; i < this.listeAutres.length; i++) {
      const a = this.listeAutres[i];
      const j = a.etat.joueur;
      j.x = a.x0 + (a.x1 - a.x0) * t;
      j.z = a.z0 + (a.z1 - a.z0) * t;
      j.angle = a.a0 + ecartAngle(a.a0, a.a1) * t;
    }
    for (let i = 0; i < this.actifs.length; i++) {
      const s = this.actifs[i];
      const e = s.e;
      e.x = s.x0 + (s.x1 - s.x0) * t;
      e.z = s.z0 + (s.z1 - s.z0) * t;
      e.angle = s.a0 + ecartAngle(s.a0, s.a1) * t;
      if (this.pause) continue;
      if (e.mort >= 0) e.mort += dtReel;
      avancerAnimation(e.animation, this.modele(e.indexType), dtReel, e.vitesseSol / (e.definition.echelle ?? 1));
    }
  }

  majVisuelOmbres(ombres: Ombres): void {
    for (let i = 0; i < this.listeAutres.length; i++) {
      const a = this.listeAutres[i];
      a.joueur.majVisuel(ombres);
      const j = a.etat.joueur;
      a.etiquette.placer(j.x, a.joueur.hauteur, j.z, !j.mort);
    }
  }

  majVisuelProjectiles(rendu: RenduProjectiles): void {
    // Tous les projectiles viennent de l'hôte (les armes locales ne tirent pas) : on repart de zéro
    rendu.commencer();
    for (let i = 0; i < this.nbProjectiles; i++) {
      const o = i * 5;
      rendu.ajouter(this.projectiles[o], this.projectiles[o + 1], this.projectiles[o + 2], this.projectiles[o + 3], this.projectiles[o + 4]);
    }
  }

  majVisuelEffets(): void {}

  // --- Instantanés -------------------------------------------------------------

  private appliquer(donnees: ArrayBuffer): void {
    const l = this.lecteur;
    const { etat, ennemis, orbes, objets, chiffres, particules, retours } = this.elements;
    l.lire(donnees);
    l.u8();
    const numero = l.u32();
    if (numero <= this.dernierNumero) return;
    this.dernierNumero = numero;
    const accuse = l.u32();
    etat.tempsJeu = l.f32();
    this.pause = (l.u8() & INSTANTANE_PAUSE) !== 0;
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
      const autre = this.autres[index];
      const j = index === this.session.index ? moi : autre?.etat.joueur;
      if (!j) continue;
      j.pv = pv;
      j.pvMax = pvMax;
      j.bouclier = bouclier;
      j.mort = (drapeaux & JOUEUR_MORT) !== 0;
      j.protection = (drapeaux & JOUEUR_PROTEGE) !== 0 ? 1 : 0;
      j.tempsMort = tempsMort;
      j.invulnerabilite = invulnerabilite;
      if (autre) {
        autre.x0 = j.x;
        autre.z0 = j.z;
        autre.a0 = j.angle;
        autre.x1 = x;
        autre.z1 = z;
        autre.a1 = angle;
        autre.commandes.bits = bits;
        autre.joueur.lacetCommandes = lacet;
      } else {
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
    for (let i = this.actifs.length - 1; i >= 0; i--) {
      const s = this.actifs[i];
      if (s.vu === numero) continue;
      const e = s.e;
      if (e.mort >= 0) particules.emettre(e.x, 0.6, e.z, GERBES.mort);
      else if (e.meche > 0) {
        particules.emettre(e.x, 0.8, e.z, GERBES.explosion);
        retours.son('explosion');
      }
      const index = ennemis.pool.elements.indexOf(e);
      if (index >= 0 && index < ennemis.pool.nombre) ennemis.pool.liberer(index);
      this.suivis.delete(e.id);
      this.actifs[i] = this.actifs[this.actifs.length - 1];
      this.actifs.pop();
      this.libres.push(s);
    }

    // Orbes et objets : reconstruits à chaque instantané
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

    // Projectiles, redessinés jusqu'au prochain instantané
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

    // Chiffres de dégâts
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
    const s = this.libres.pop();
    const e = s ? this.elements.ennemis.pool.acquerir() : null;
    if (!s || !e) {
      if (s) this.libres.push(s);
      return null;
    }
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
    effacerEtats(e);
    reinitialiserAnimation(e.animation);
    s.e = e;
    this.suivis.set(id, s);
    this.actifs.push(s);
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
