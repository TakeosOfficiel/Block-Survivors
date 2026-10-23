/**
 * Session multijoueur : le salon (qui est là, quel personnage, quelle arène),
 * puis l'aiguillage des messages vers la partie. L'hôte (index 0) garde une
 * connexion par joueur (en étoile, 3 au plus) ; un joueur n'a que celle de
 * l'hôte.
 */
import { IDS_PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { BonusTalents } from '../progression/sauvegarde';
import type { ChoixSkin } from '../profil';
import { VERSION } from '../version';
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { armureValide, skinValide, type ArmureJoueur, type InfoJoueur, type MessageFiable } from './protocole';
import { ConnexionPair } from './transport';

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

/** Invitation de l'hôte : le code à envoyer, puis la réponse du joueur à coller. */
export interface Invitation {
  readonly code: string;
  accepter(reponse: string): Promise<void>;
  annuler(): void;
}

export class SessionReseau {
  readonly role: RoleReseau;
  /** Index de ce joueur dans la partie (0 : l'hôte). */
  index = 0;
  /** Joueurs du salon, l'hôte compris (dans l'ordre des index). */
  joueurs: InfoJoueur[] = [];
  /** Identifiant de l'arène choisie par l'hôte. */
  arene = '';
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
  private readonly profil: ProfilReseau;
  private prets = new Set<number>();
  private surPrets: (() => void) | null = null;
  private surDepartPartie: (() => void) | null = null;

  private constructor(role: RoleReseau, profil: ProfilReseau) {
    this.role = role;
    this.profil = profil;
  }

  static heberger(profil: ProfilReseau, arene: string): SessionReseau {
    const session = new SessionReseau('hote', profil);
    session.arene = arene;
    session.joueurs = [{ index: 0, pseudo: profil.pseudo, personnage: profil.personnage, skin: profil.skin, brasFins: profil.brasFins, skinImporte: profil.skinImporte, bonus: profil.bonus, armure: profil.armure,
    }];
    return session;
  }

  /** Joueur : répond au code d'invitation ; renvoie la session et le code de réponse à rendre à l'hôte. */
  static async rejoindre(profil: ProfilReseau, codeOffre: string): Promise<{ session: SessionReseau; code: string }> {
    const { pair, code } = await ConnexionPair.repondre(codeOffre);
    const session = new SessionReseau('client', profil);
    session.hote = pair;
    pair.observer({
      surMessage: (m) => session.recevoirDeHote(m as MessageFiable),
      surDonnees: (d) => session.partie?.surDonnees(0, d),
      surFermeture: () => {
        session.partie?.surDepart(0);
        session.surPerte('Connexion avec l\'hôte perdue');
      },
    });
    void pair.attendre(120000).then(() => {
      const p = session.profil;
      pair.envoyer({
        type: 'bonjour', version: VERSION, pseudo: p.pseudo, personnage: p.personnage, skin: p.skin, brasFins: p.brasFins, skinImporte: p.skinImporte,
        bonus: p.bonus, armure: p.armure,
      } satisfies MessageFiable);
    }, () => session.surPerte('Connexion impossible : l\'hôte n\'a pas validé la réponse à temps'));
    return { session, code };
  }

  get nombreJoueurs(): number {
    return this.joueurs.length;
  }

  /** Hôte : prépare une place pour un nouveau joueur. */
  async inviter(): Promise<Invitation> {
    const { pair, code } = await ConnexionPair.offrir();
    return {
      code,
      accepter: async (reponse) => {
        await pair.accepter(reponse);
        await pair.attendre();
        this.attendreBonjour(pair);
      },
      annuler: () => pair.fermer(),
    };
  }

  /** Hôte : tout le monde charge l'arène. */
  lancer(): void {
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
    this.hote?.fermer();
    for (const pair of this.pairs.values()) pair.fermer();
  }

  // --- Hôte ----------------------------------------------------------------

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
