/**
 * Session multijoueur : le salon (qui est là, quel personnage, quelle arène),
 * puis l'aiguillage des messages vers la partie. L'hôte (index 0) garde une
 * connexion par joueur (en étoile, 3 au plus) ; un joueur n'a que celle de
 * l'hôte.
 *
 * Entrée dans le salon par son code (signalisation.ts) : le joueur demande à
 * entrer, l'hôte lui prépare une offre, le joueur répond. Les messages de la
 * salle pouvant se perdre, le joueur répète sa demande (ou sa réponse) jusqu'à
 * être relié, et l'hôte renvoie la même offre à chaque demande répétée.
 */
import { IDS_PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { BonusTalents } from '../progression/sauvegarde';
import type { ChoixSkin } from '../profil';
import { VERSION } from '../version';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { armureValide, skinValide, type ArmureJoueur, type InfoJoueur, type MessageFiable } from './protocole';
import { nouveauCode, Salle, type MessageSalle } from './signalisation';
import { ConnexionPair } from './transport';

/** ms entre deux répétitions d'un message de la salle, et avant d'abandonner la recherche du salon. */
const REPETITION = 2500;
const RECHERCHE_MAX = 30000;

export type RoleReseau = 'hote' | 'client';

/** Ce que ce joueur apporte au salon. */
export interface ProfilReseau {
  readonly pseudo: string;
  readonly personnage: IdPersonnage;
  readonly skin: ChoixSkin;
  readonly skinImporte: string | null;
  readonly brasFins: boolean;
  readonly bonus: BonusTalents;
  readonly armure: ArmureJoueur;
}

/** Ce qui reçoit les messages pendant la partie (côté hôte : `index` du joueur ; côté joueur : 0). */
export interface ObservateurPartie {
  surMessage(index: number, message: MessageFiable): void;
  surDonnees(index: number, donnees: ArrayBuffer): void;
  surDepart(index: number): void;
}

/** Hôte : un joueur en train d'entrer (son offre, puis sa connexion). */
interface Arrivee {
  pair: ConnexionPair | null;
  offre: string | null;
  acceptee: boolean;
}

export class SessionReseau {
  readonly role: RoleReseau;
  /** Index de ce joueur dans la partie (0 : l'hôte). */
  index = 0;
  /** Joueurs du salon, l'hôte compris (dans l'ordre des index). */
  joueurs: InfoJoueur[] = [];
  /** Identifiant de l'arène choisie par l'hôte. */
  arene = '';
  /** Code du salon (hôte), à donner aux autres joueurs. */
  code = '';
  /** Appelé à chaque changement du salon (pour l'écran). */
  surChangement: () => void = () => {};
  /** Joueur : appelé quand l'hôte lance la partie ; avec la raison quand la connexion se perd. */
  surLancement: () => void = () => {};
  surPerte: (raison: string) => void = () => {};
  /** Branché par la partie une fois chargée ; les messages arrivés avant sont ignorés. */
  partie: ObservateurPartie | null = null;

  /** Hôte : connexion de chaque joueur distant, par index. */
  private readonly pairs = new Map<number, ConnexionPair>();
  /** Joueur : connexion à l'hôte. */
  private hote: ConnexionPair | null = null;
  /** Hôte : salle du code, ouverte tant que le salon accueille ; joueurs en train d'entrer, par identifiant. */
  private salle: Salle | null = null;
  private readonly arrivees = new Map<string, Arrivee>();
  private readonly profil: ProfilReseau;
  private prets = new Set<number>();
  private surPrets: (() => void) | null = null;
  private surDepartPartie: (() => void) | null = null;

  private constructor(role: RoleReseau, profil: ProfilReseau) {
    this.role = role;
    this.profil = profil;
  }

  /** Hôte : ouvre un salon sous un nouveau code ; les joueurs qui le tapent entrent d'eux-mêmes. */
  static async heberger(profil: ProfilReseau, arene: string): Promise<SessionReseau> {
    const session = new SessionReseau('hote', profil);
    session.arene = arene;
    session.joueurs = [{ index: 0, pseudo: profil.pseudo, personnage: profil.personnage, skin: profil.skin, brasFins: profil.brasFins, skinImporte: profil.skinImporte, bonus: profil.bonus, armure: profil.armure,
    }];
    session.code = nouveauCode();
    session.salle = await Salle.ouvrir(session.code);
    session.salle.surMessage = (m) => session.surSalle(m);
    return session;
  }

  /** Joueur : entre dans le salon du code ; résolue une fois relié à l'hôte. */
  static async rejoindre(profil: ProfilReseau, code: string): Promise<SessionReseau> {
    const salle = await Salle.ouvrir(code);
    const session = new SessionReseau('client', profil);
    const id = Array.from(crypto.getRandomValues(new Uint8Array(6)), (o) => o.toString(16).padStart(2, '0')).join('');
    return new Promise((resoudre, rejeter) => {
      let reponse: string | null = null;
      const finir = (erreur?: Error) => {
        clearInterval(repetition);
        clearTimeout(abandon);
        salle.fermer();
        if (!erreur) {
          session.direBonjour();
          resoudre(session);
        } else {
          session.hote?.fermer();
          rejeter(erreur);
        }
      };
      const repetition = setInterval(() => void salle.publier(reponse ? { t: 'reponse', de: id, sdp: reponse } : { t: 'demande', de: id }), REPETITION);
      const abandon = setTimeout(() => finir(new Error('Salon introuvable : vérifiez le code, et que l\'hôte est bien dans son salon')), RECHERCHE_MAX);
      let offreRecue = false;
      salle.surMessage = (m) => {
        if (m.t !== 'offre' || m.a !== id || offreRecue) return;
        offreRecue = true;
        clearTimeout(abandon);
        void ConnexionPair.repondre(m.sdp).then(({ pair, sdp }) => {
          reponse = sdp;
          session.brancherHote(pair);
          void salle.publier({ t: 'reponse', de: id, sdp });
          pair.attendre().then(() => finir(), () => finir(new Error(
            'Connexion directe impossible avec l\'hôte (box trop fermées) : essayez en réseau local, ou avec Tailscale ou ZeroTier')));
        });
      };
      void salle.publier({ t: 'demande', de: id });
    });
  }

  get nombreJoueurs(): number {
    return this.joueurs.length;
  }

  /** Hôte : tout le monde charge l'arène ; le salon n'accueille plus personne. */
  lancer(): void {
    this.fermerSalle();
    const message: MessageFiable = { type: 'lancer', arene: this.arene, joueurs: this.joueurs };
    for (const pair of this.pairs.values()) pair.envoyer(message);
  }

  /**
   * Fin du chargement. Joueur : prévient l'hôte et attend le départ. Hôte :
   * attend que tous les joueurs soient prêts (au plus 60 s), puis donne le départ.
   */
  synchroniserDepart(): Promise<void> {
    if (this.role === 'client') {
      return new Promise((resoudre) => {
        this.surDepartPartie = resoudre;
        this.hote?.envoyer({ type: 'pret' } satisfies MessageFiable);
      });
    }
    return new Promise<void>((resoudre) => {
      const verifier = () => {
        for (const index of this.pairs.keys()) if (!this.prets.has(index)) return;
        this.surPrets = null;
        resoudre();
      };
      this.surPrets = verifier;
      setTimeout(() => {
        this.surPrets = null;
        resoudre();
      }, 60000);
      verifier();
    }).then(() => {
      for (const pair of this.pairs.values()) pair.envoyer({ type: 'depart' } satisfies MessageFiable);
    });
  }

  /** Joueur → hôte, ou hôte → un joueur (`index`), ou hôte → tous (`index` absent). */
  envoyer(message: MessageFiable, index?: number): void {
    if (this.hote) this.hote.envoyer(message);
    else if (index !== undefined) this.pairs.get(index)?.envoyer(message);
    else for (const pair of this.pairs.values()) pair.envoyer(message);
  }

  envoyerDonnees(donnees: Uint8Array<ArrayBuffer>, index = 0): void {
    if (this.hote) this.hote.envoyerDonnees(donnees);
    else this.pairs.get(index)?.envoyerDonnees(donnees);
  }

  /** Index des joueurs distants encore connectés (hôte). */
  indexDistants(): IterableIterator<number> {
    return this.pairs.keys();
  }

  fermer(): void {
    this.fermerSalle();
    this.hote?.fermer();
    for (const pair of this.pairs.values()) pair.fermer();
  }

  // --- Hôte ----------------------------------------------------------------

  private surSalle(m: MessageSalle): void {
    if (m.t === 'demande') void this.preparerArrivee(m.de);
    else if (m.t === 'reponse') void this.accepterArrivee(m.de, m.sdp);
  }

  /** Une demande : une offre pour ce joueur (la même s'il répète sa demande, la première a pu se perdre). */
  private async preparerArrivee(id: string): Promise<void> {
    const deja = this.arrivees.get(id);
    if (deja) {
      if (deja.offre && !deja.acceptee) void this.salle?.publier({ t: 'offre', a: id, sdp: deja.offre });
      return;
    }
    if (this.joueurs.length >= REGLES_MULTIJOUEUR.joueursMax) return;
    const arrivee: Arrivee = { pair: null, offre: null, acceptee: false };
    this.arrivees.set(id, arrivee);
    const { pair, sdp } = await ConnexionPair.offrir();
    if (!this.salle) {
      pair.fermer();
      return;
    }
    arrivee.pair = pair;
    arrivee.offre = sdp;
    void this.salle.publier({ t: 'offre', a: id, sdp });
  }

  private async accepterArrivee(id: string, sdp: string): Promise<void> {
    const arrivee = this.arrivees.get(id);
    if (!arrivee?.pair || arrivee.acceptee) return;
    arrivee.acceptee = true;
    try {
      await arrivee.pair.accepter(sdp);
      await arrivee.pair.attendre();
      this.attendreBonjour(arrivee.pair);
    } catch {
      arrivee.pair.fermer();
    }
  }

  /** Plus d'arrivées : la salle se ferme, les connexions pas encore abouties aussi. */
  private fermerSalle(): void {
    this.salle?.fermer();
    this.salle = null;
    for (const arrivee of this.arrivees.values()) {
      if (arrivee.pair && ![...this.pairs.values()].includes(arrivee.pair)) arrivee.pair.fermer();
    }
    this.arrivees.clear();
  }

  private attendreBonjour(pair: ConnexionPair): void {
    let index = -1;
    pair.observer({
      surMessage: (brut) => {
        const m = brut as MessageFiable;
        if (index < 0) {
          if (m.type === 'bonjour') index = this.accueillir(pair, m);
          return;
        }
        if (m.type === 'pret') {
          this.prets.add(index);
          this.surPrets?.();
        } else {
          this.partie?.surMessage(index, m);
        }
      },
      surDonnees: (d) => {
        if (index >= 0) this.partie?.surDonnees(index, d);
      },
      surFermeture: () => {
        if (index < 0) return;
        this.pairs.delete(index);
        this.joueurs = this.joueurs.filter((j) => j.index !== index);
        this.partie?.surDepart(index);
        this.surPrets?.();
        this.diffuserSalon();
      },
    });
  }

  /** Vérifie le joueur et lui donne une place ; renvoie son index (−1 : refusé). */
  private accueillir(pair: ConnexionPair, m: Extract<MessageFiable, { type: 'bonjour' }>): number {
    const refuser = (raison: string) => {
      pair.envoyer({ type: 'refus', raison } satisfies MessageFiable);
      setTimeout(() => pair.fermer(), 500);
      return -1;
    };
    if (m.version !== VERSION) return refuser(`Versions différentes : l'hôte a la ${VERSION}, vous la ${String(m.version)}`);
    if (!IDS_PERSONNAGES.includes(m.personnage)) return refuser('Personnage inconnu');
    let index = 1;
    while (this.pairs.has(index)) index++;
    if (index >= REGLES_MULTIJOUEUR.joueursMax) return refuser('La partie est complète');
    this.pairs.set(index, pair);
    this.joueurs.push({
      index, pseudo: String(m.pseudo).slice(0, 16), personnage: m.personnage, ...skinValide(m.skin, m.skinImporte), brasFins: m.brasFins === true,
      bonus: m.bonus, armure: armureValide(m.armure),
    });
    this.joueurs.sort((a, b) => a.index - b.index);
    pair.envoyer({ type: 'accueil', index } satisfies MessageFiable);
    this.diffuserSalon();
    return index;
  }

  private diffuserSalon(): void {
    const message: MessageFiable = { type: 'salon', arene: this.arene, joueurs: this.joueurs };
    for (const pair of this.pairs.values()) pair.envoyer(message);
    this.surChangement();
  }

  /** Hôte : l'arène a changé dans le salon. */
  changerArene(arene: string): void {
    this.arene = arene;
    this.diffuserSalon();
  }

  // --- Joueur --------------------------------------------------------------

  private brancherHote(pair: ConnexionPair): void {
    this.hote = pair;
    pair.observer({
      surMessage: (m) => this.recevoirDeHote(m as MessageFiable),
      surDonnees: (d) => this.partie?.surDonnees(0, d),
      surFermeture: () => {
        this.partie?.surDepart(0);
        this.surPerte('Connexion avec l\'hôte perdue');
      },
    });
  }

  /** Relié : qui il est et ses bonus permanents (l'hôte vérifie sa version). */
  private direBonjour(): void {
    const p = this.profil;
    this.hote?.envoyer({
      type: 'bonjour', version: VERSION, pseudo: p.pseudo, personnage: p.personnage, skin: p.skin, brasFins: p.brasFins, skinImporte: p.skinImporte,
      bonus: p.bonus, armure: p.armure,
    } satisfies MessageFiable);
  }

  private recevoirDeHote(m: MessageFiable): void {
    switch (m.type) {
      case 'accueil':
        this.index = m.index;
        break;
      case 'refus':
        this.surPerte(m.raison);
        this.hote?.fermer();
        break;
      case 'salon':
        this.arene = m.arene;
        this.joueurs = m.joueurs;
        this.surChangement();
        break;
      case 'lancer':
        this.arene = m.arene;
        this.joueurs = m.joueurs;
        this.surLancement();
        break;
      case 'depart':
        this.surDepartPartie?.();
        this.surDepartPartie = null;
        break;
      default:
        this.partie?.surMessage(0, m);
    }
  }
}
