/**
 * Amis : chaque profil a un code ami (« ABC-DEF-GHJ »), à donner une fois.
 * Un ami ajouté voit si l'on est en ligne (au menu, dans un salon, en partie),
 * peut rejoindre notre salon d'un clic, nous y inviter, ou nous demander de
 * jouer. Tout passe par les relais de signalisation.ts, sur deux canaux tirés
 * du code ami : la présence (que publie son propriétaire) et la boîte aux
 * lettres (où les autres lui écrivent). Seul qui connaît le code les lit.
 *
 * La présence est éphémère : republiée toutes les 25 s ; un ami muet depuis
 * 70 s est hors ligne. Au démarrage, on demande à chaque ami de se manifester.
 * Rien ne se connecte tant que le profil n'a pas d'ami et que la page
 * Multijoueur n'a pas été ouverte.
 */
import { ecrireStockage, lireStockage } from '../options';
import { cleProfil } from '../profils';
import { canal, codeAuHasard, normaliserCode, Relais, type Canal } from './signalisation';

/** Longueur d'un code ami (sans les tirets). */
const LONGUEUR_CODE_AMI = 9;
/** ms entre deux publications de sa présence ; au-delà de `silence` ms sans nouvelles, un ami est hors ligne. */
const BATTEMENT = 25000;
const SILENCE = 70000;
/** Amis au plus. */
const AMIS_MAX = 50;

export interface Ami {
  readonly code: string;
  /** Dernier pseudo connu (« ? » tant qu'il ne s'est pas manifesté). */
  pseudo: string;
}

export type EtatPresence = 'menu' | 'salon' | 'partie';

export interface Presence {
  readonly etat: EtatPresence;
  /** Code de son salon s'il en héberge un qui a encore de la place, sinon null. */
  readonly salon: string | null;
  /** Joueurs dans son salon ou sa partie. */
  readonly joueurs: number;
  /** Dernières nouvelles (ms, horloge du navigateur). */
  readonly vu: number;
}

type MessageAmi =
  | { readonly t: 'presence'; readonly pseudo: string; readonly etat: EtatPresence; readonly salon: string | null; readonly joueurs: number }
  /** Boîte aux lettres : de la part de `de` (son code ami). */
  | { readonly t: 'qui'; readonly de: string }
  | { readonly t: 'ajout'; readonly de: string; readonly pseudo: string }
  | { readonly t: 'invitation'; readonly de: string; readonly pseudo: string; readonly salon: string }
  | { readonly t: 'demande'; readonly de: string; readonly pseudo: string };

interface DonneesAmis {
  version: 1;
  code: string;
  amis: Ami[];
}

export function normaliserCodeAmi(saisie: string): string | null {
  return normaliserCode(saisie, LONGUEUR_CODE_AMI);
}

export class ServiceAmis {
  readonly code: string;
  readonly amis: Ami[];
  /** Présence de chaque ami (par code), quand il s'est manifesté. */
  readonly presences = new Map<string, Presence>();
  /** Liste ou présences changées (pour l'écran). */
  surChangement: () => void = () => {};
  /** Un ami nous invite dans son salon. */
  surInvitation: (ami: Ami, salon: string) => void = () => {};
  /** Un ami voudrait jouer avec nous. */
  surDemande: (ami: Ami) => void = () => {};
  /** Quelqu'un nous a ajouté (pas encore dans notre liste). */
  surAjout: (code: string, pseudo: string) => void = () => {};

  /** Pseudo du profil, lu à chaque envoi (il peut changer dans l'Apparence). */
  private readonly pseudo: () => string;
  private etat: EtatPresence = 'menu';
  private salon: string | null = null;
  private joueurs = 1;
  private relais: Relais | null = null;
  private connexion: Promise<void> | null = null;
  private presence: Canal | null = null;
  private readonly canauxAmis = new Map<string, Canal>();
  private derniereReponse = 0;
  private enLigneAvant = '';

  constructor(pseudo: () => string) {
    this.pseudo = pseudo;
    const lu = lireStockage<Partial<DonneesAmis>>(cleProfil('amis'));
    this.code = typeof lu?.code === 'string' && normaliserCodeAmi(lu.code) === lu.code ? lu.code : codeAuHasard(LONGUEUR_CODE_AMI);
    this.amis = Array.isArray(lu?.amis)
      ? lu.amis.filter((a) => typeof a?.code === 'string' && normaliserCodeAmi(a.code) === a.code).map((a) => ({ code: a.code, pseudo: String(a.pseudo ?? '?').slice(0, 16) }))
      : [];
    this.enregistrer();
    if (this.amis.length > 0) void this.demarrer();
  }

  /** Vrai une fois relié aux relais. */
  get connecte(): boolean {
    return (this.relais?.connectes ?? 0) > 0;
  }

  /** Se relie aux relais (une seule fois) ; sans effet si c'est déjà fait. */
  demarrer(): Promise<void> {
    this.connexion ??= this.relier().catch(() => {
      this.connexion = null;  // relais injoignables : on réessaiera à la prochaine ouverture
    });
    return this.connexion;
  }

  enLigne(code: string): Presence | null {
    const p = this.presences.get(code);
    return p && Date.now() - p.vu < SILENCE ? p : null;
  }

  /** Ajoute un ami par son code ; renvoie une erreur à afficher, ou null. */
  ajouter(saisie: string, pseudo = '?'): string | null {
    const code = normaliserCodeAmi(saisie);
    if (!code) return 'Un code ami a 9 lettres ou chiffres, comme ABC-DEF-GHJ';
    if (code === this.code) return 'C\'est votre propre code';
    if (this.amis.some((a) => a.code === code)) return 'Cet ami est déjà dans votre liste';
    if (this.amis.length >= AMIS_MAX) return 'Liste d\'amis pleine';
    this.amis.push({ code, pseudo });
    this.enregistrer();
    void this.demarrer().then(async () => {
      await this.suivre(code);
      await this.ecrire(code, { t: 'ajout', de: this.code, pseudo: this.pseudo() });
      await this.ecrire(code, { t: 'qui', de: this.code });
    });
    this.surChangement();
    return null;
  }

  retirer(code: string): void {
    const i = this.amis.findIndex((a) => a.code === code);
    if (i < 0) return;
    this.amis.splice(i, 1);
    this.presences.delete(code);
    const c = this.canauxAmis.get(code);
    if (c) this.relais?.oublier(c);
    this.canauxAmis.delete(code);
    this.enregistrer();
    this.surChangement();
  }

  /** Où l'on en est (publié tout de suite aux amis) ; `salon` : code du salon hébergé s'il reste de la place. */
  definirEtat(etat: EtatPresence, salon: string | null = null, joueurs = 1): void {
    if (etat === this.etat && salon === this.salon && joueurs === this.joueurs) return;
    this.etat = etat;
    this.salon = salon;
    this.joueurs = joueurs;
    void this.publierPresence();
  }

  inviter(code: string, salon: string): Promise<void> {
    return this.ecrire(code, { t: 'invitation', de: this.code, pseudo: this.pseudo(), salon });
  }

  demanderAJouer(code: string): Promise<void> {
    return this.ecrire(code, { t: 'demande', de: this.code, pseudo: this.pseudo() });
  }

  // --- Relais ----------------------------------------------------------------

  private async relier(): Promise<void> {
    const [relais, boite, presence] = await Promise.all([Relais.ouvrir(), canal(`ami/${this.code}/boite`), canal(`ami/${this.code}/presence`)]);
    this.relais = relais;
    this.presence = presence;
    relais.ecouter(boite, (m) => this.lireBoite(m as MessageAmi));
    await Promise.all(this.amis.map((a) => this.suivre(a.code)));
    await this.publierPresence();
    for (const a of this.amis) void this.ecrire(a.code, { t: 'qui', de: this.code });
    setInterval(() => void this.publierPresence(), BATTEMENT);
    // Un ami qui se tait passe hors ligne
    setInterval(() => this.verifierEnLigne(), 5000);
  }

  /** Écoute la présence d'un ami. */
  private async suivre(code: string): Promise<void> {
    if (!this.relais || this.canauxAmis.has(code)) return;
    const c = await canal(`ami/${code}/presence`);
    this.canauxAmis.set(code, c);
    this.relais.ecouter(c, (m) => this.lirePresence(code, m as MessageAmi));
  }

  private async ecrire(code: string, message: MessageAmi): Promise<void> {
    await this.demarrer();
    if (!this.relais) return;
    await this.relais.publier(await canal(`ami/${code}/boite`), message);
  }

  private async publierPresence(): Promise<void> {
    if (!this.relais || !this.presence) return;
    await this.relais.publier(this.presence, { t: 'presence', pseudo: this.pseudo(), etat: this.etat, salon: this.salon, joueurs: this.joueurs } satisfies MessageAmi);
  }

  private lirePresence(code: string, m: MessageAmi): void {
    if (m.t !== 'presence') return;
    const ami = this.amis.find((a) => a.code === code);
    if (!ami) return;
    const pseudo = String(m.pseudo).slice(0, 16);
    if (pseudo && pseudo !== ami.pseudo) {
      ami.pseudo = pseudo;
      this.enregistrer();
    }
    const etat: EtatPresence = m.etat === 'salon' || m.etat === 'partie' ? m.etat : 'menu';
    const salon = typeof m.salon === 'string' ? m.salon : null;
    this.presences.set(code, { etat, salon, joueurs: Number(m.joueurs) || 1, vu: Date.now() });
    this.enLigneAvant = this.signatureEnLigne();
    this.surChangement();
  }

  private lireBoite(m: MessageAmi): void {
    if (m.t === 'presence' || typeof m.de !== 'string' || m.de === this.code) return;
    const ami = this.amis.find((a) => a.code === m.de);
    if (m.t === 'qui') {
      // Un ami (re)vient : on se manifeste, sans répondre en rafale
      if (ami && Date.now() - this.derniereReponse > 2000) {
        this.derniereReponse = Date.now();
        void this.publierPresence();
      }
      return;
    }
    const pseudo = String(m.pseudo).slice(0, 16);
    if (m.t === 'ajout') {
      if (!ami) this.surAjout(m.de, pseudo);
      return;
    }
    // Invitations et demandes : seulement de nos amis
    if (!ami) return;
    if (m.t === 'invitation' && typeof m.salon === 'string') this.surInvitation(ami, m.salon);
    else if (m.t === 'demande') this.surDemande(ami);
  }

  private verifierEnLigne(): void {
    const signature = this.signatureEnLigne();
    if (signature === this.enLigneAvant) return;
    this.enLigneAvant = signature;
    this.surChangement();
  }

  private signatureEnLigne(): string {
    return this.amis.filter((a) => this.enLigne(a.code)).map((a) => a.code).join();
  }

  private enregistrer(): void {
    ecrireStockage(cleProfil('amis'), { version: 1, code: this.code, amis: this.amis } satisfies DonneesAmis);
  }
}
