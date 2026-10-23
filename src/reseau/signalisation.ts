/**
 * Messagerie sans serveur à nous, pour se trouver avant de jouer : la poignée
 * de main WebRTC d'un salon (code court, « K7F-3QX ») et les amis (présence,
 * invitations). Elle passe par des relais Nostr publics, de simples boîtes aux
 * lettres ; une fois les joueurs reliés, la partie va directement de l'un à
 * l'autre.
 *
 * Chaque conversation est un canal tiré d'un nom (« salon/K7F-3QX »…) : une
 * étiquette (ce que les relais voient) et une clé AES-GCM (ce qu'ils ne
 * voient pas). Sans le nom, on ne trouve pas le canal et on ne lit rien.
 * Messages éphémères (les relais ne les gardent pas), publiés sur tous les
 * relais à la fois (le premier arrivé l'emporte, les doublons sont ignorés),
 * signés d'une clé jetable (Nostr l'exige, elle n'identifie personne).
 */
import { schnorr } from '@noble/curves/secp256k1.js';

const ADRESSES = ['wss://relay.damus.io', 'wss://nos.lol', 'wss://relay.primal.net', 'wss://relay.nostr.band', 'wss://nostr.mom'];
/** Sorte d'événement Nostr éphémère (20000 à 29999 : relayé, jamais gardé). */
const SORTE = 25417;
/** ms avant de rouvrir un relais perdu (connexion longue des amis). */
const RECONNEXION = 15000;
/** Lettres et chiffres qu'on ne confond pas (ni O ni 0, ni I, L ni 1). */
const ALPHABET = 'ABCDEFGHJKMNPQRSTUVWXYZ23456789';

/** Un canal : son étiquette chez les relais et sa clé de chiffrement. */
export interface Canal {
  readonly etiquette: string;
  readonly cle: CryptoKey;
}

/** Canal d'un nom (toujours le même pour le même nom). */
export async function canal(nom: string): Promise<Canal> {
  const encodeur = new TextEncoder();
  const etiquette = hex(await empreinte(encodeur.encode(`block-survivors/etiquette/${nom}`))).slice(0, 32);
  const cle = await crypto.subtle.importKey('raw', await empreinte(encodeur.encode(`block-survivors/cle/${nom}`)), 'AES-GCM', false, ['encrypt', 'decrypt']);
  return { etiquette, cle };
}

/** Code au hasard de `longueur` caractères (multiple de 3), groupés par 3 : « ABC-DEF ». */
export function codeAuHasard(longueur: number): string {
  const octets = crypto.getRandomValues(new Uint8Array(longueur));
  let code = '';
  for (let i = 0; i < longueur; i++) code += ALPHABET[octets[i] % ALPHABET.length];
  return code.match(/.{3}/g)!.join('-');
}

/** Code tapé par le joueur, remis en forme (minuscules, espaces, tirets oubliés) ; null s'il n'en est pas un. */
export function normaliserCode(saisie: string, longueur: number): string | null {
  const brut = saisie.toUpperCase().replace(/[^A-Z0-9]/g, '');
  if (brut.length !== longueur || [...brut].some((c) => !ALPHABET.includes(c))) return null;
  return brut.match(/.{3}/g)!.join('-');
}

interface Ecoute {
  readonly canal: Canal;
  readonly rappel: (message: unknown) => void;
}

/** Connexion aux relais : on y écoute des canaux et on y publie. */
export class Relais {
  private readonly sockets = new Map<string, WebSocket>();
  private readonly ecoutes = new Map<string, Ecoute>();
  private readonly vus = new Set<string>();
  private readonly secret = schnorr.utils.randomSecretKey();
  private readonly publique = hex(schnorr.getPublicKey(this.secret));
  private ferme = false;

  /** Résolue dès qu'un relais répond (une erreur si aucun). */
  static ouvrir(): Promise<Relais> {
    const relais = new Relais();
    return new Promise((resoudre, rejeter) => {
      let echecs = 0;
      for (const adresse of ADRESSES) {
        relais.connecter(adresse, () => resoudre(relais), () => {
          if (++echecs === ADRESSES.length) rejeter(new Error('Aucun relais joignable : vérifiez votre connexion à Internet'));
        });
      }
    });
  }

  /** Nombre de relais connectés en ce moment. */
  get connectes(): number {
    let n = 0;
    for (const s of this.sockets.values()) if (s.readyState === WebSocket.OPEN) n++;
    return n;
  }

  ecouter(c: Canal, rappel: (message: unknown) => void): void {
    this.ecoutes.set(c.etiquette, { canal: c, rappel });
    const demande = this.demande(c);
    for (const s of this.sockets.values()) if (s.readyState === WebSocket.OPEN) s.send(demande);
  }

  oublier(c: Canal): void {
    if (!this.ecoutes.delete(c.etiquette)) return;
    const fin = JSON.stringify(['CLOSE', c.etiquette.slice(0, 16)]);
    for (const s of this.sockets.values()) if (s.readyState === WebSocket.OPEN) s.send(fin);
  }

  /** Publie `message` (chiffré) sur le canal, sur tous les relais connectés. */
  async publier(c: Canal, message: object): Promise<void> {
    if (this.ferme) return;
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const chiffre = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, c.cle, new TextEncoder().encode(JSON.stringify(message))));
    const evenement = {
      pubkey: this.publique,
      created_at: Math.floor(Date.now() / 1000),
      kind: SORTE,
      tags: [['t', c.etiquette]],
      content: base64(iv) + '.' + base64(chiffre),
    };
    const id = await empreinte(new TextEncoder().encode(JSON.stringify([0, evenement.pubkey, evenement.created_at, evenement.kind, evenement.tags, evenement.content])));
    const signe = { ...evenement, id: hex(id), sig: hex(schnorr.sign(id, this.secret)) };
    this.vus.add(signe.id);
    const texte = JSON.stringify(['EVENT', signe]);
    for (const s of this.sockets.values()) if (s.readyState === WebSocket.OPEN) s.send(texte);
  }

  fermer(): void {
    this.ferme = true;
    for (const s of this.sockets.values()) s.close();
  }

  private demande(c: Canal): string {
    return JSON.stringify(['REQ', c.etiquette.slice(0, 16), { kinds: [SORTE], '#t': [c.etiquette], since: Math.floor(Date.now() / 1000) - 30 }]);
  }

  /** Ouvre un relais (et le rouvre s'il se perd après avoir servi). */
  private connecter(adresse: string, surOuverture: () => void, surEchec: () => void): void {
    let s: WebSocket;
    try {
      s = new WebSocket(adresse);
    } catch {
      surEchec();
      return;
    }
    this.sockets.set(adresse, s);
    let ouvert = false;
    s.onopen = () => {
      ouvert = true;
      for (const e of this.ecoutes.values()) s.send(this.demande(e.canal));
      surOuverture();
    };
    s.onerror = () => {
      if (!ouvert) surEchec();
    };
    s.onclose = () => {
      if (ouvert && !this.ferme) setTimeout(() => this.connecter(adresse, () => {}, () => {}), RECONNEXION);
    };
    s.onmessage = (e) => void this.recevoir(e.data);
  }

  private async recevoir(donnees: unknown): Promise<void> {
    let message: unknown;
    try {
      message = JSON.parse(String(donnees));
    } catch {
      return;
    }
    if (!Array.isArray(message) || message[0] !== 'EVENT') return;
    const evenement = message[2] as { id?: unknown; content?: unknown; tags?: unknown } | undefined;
    if (!evenement || typeof evenement.id !== 'string' || typeof evenement.content !== 'string' || this.vus.has(evenement.id)) return;
    const etiquette = Array.isArray(evenement.tags) ? (evenement.tags as unknown[][]).find((t) => t[0] === 't')?.[1] : undefined;
    const ecoute = typeof etiquette === 'string' ? this.ecoutes.get(etiquette) : undefined;
    if (!ecoute) return;
    this.vus.add(evenement.id);
    const [iv, chiffre] = evenement.content.split('.');
    let contenu: unknown;
    try {
      const clair = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: depuisBase64(iv) }, ecoute.canal.cle, depuisBase64(chiffre));
      contenu = JSON.parse(new TextDecoder().decode(clair));
    } catch {
      return;  // pas chiffré avec la clé de ce canal : ignoré
    }
    if (!this.ferme) ecoute.rappel(contenu);
  }
}

// --- Salon : la poignée de main WebRTC -------------------------------------

/** Longueur d'un code de salon (sans le tiret). */
export const LONGUEUR_CODE_SALON = 6;

export type MessageSalle =
  /** Joueur → hôte : « je veux entrer » (`de` : son identifiant jetable). */
  | { readonly t: 'demande'; readonly de: string }
  /** Hôte → joueur `a` : la description WebRTC de l'hôte. */
  | { readonly t: 'offre'; readonly a: string; readonly sdp: string }
  /** Joueur → hôte : sa réponse. */
  | { readonly t: 'reponse'; readonly de: string; readonly sdp: string };

/** Salle de rencontre d'un code de salon : ce qui y est publié arrive à tous ceux qui l'ont ouverte. */
export class Salle {
  /** Message arrivé (les siens exclus, chacun une seule fois). */
  surMessage: (message: MessageSalle) => void = () => {};

  private readonly relais: Relais;
  private readonly canal: Canal;

  private constructor(relais: Relais, c: Canal) {
    this.relais = relais;
    this.canal = c;
  }

  static async ouvrir(code: string): Promise<Salle> {
    const [relais, c] = await Promise.all([Relais.ouvrir(), canal(`salon/${code}`)]);
    const salle = new Salle(relais, c);
    relais.ecouter(c, (m) => salle.surMessage(m as MessageSalle));
    return salle;
  }

  publier(message: MessageSalle): Promise<void> {
    return this.relais.publier(this.canal, message);
  }

  fermer(): void {
    this.relais.fermer();
  }
}

async function empreinte(octets: Uint8Array<ArrayBuffer>): Promise<Uint8Array<ArrayBuffer>> {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', octets));
}

function hex(octets: Uint8Array): string {
  let texte = '';
  for (let i = 0; i < octets.length; i++) texte += octets[i].toString(16).padStart(2, '0');
  return texte;
}

function base64(octets: Uint8Array): string {
  let binaire = '';
  for (let i = 0; i < octets.length; i++) binaire += String.fromCharCode(octets[i]);
  return btoa(binaire);
}

function depuisBase64(texte: string): Uint8Array<ArrayBuffer> {
  const binaire = atob(texte);
  const octets = new Uint8Array(binaire.length);
  for (let i = 0; i < binaire.length; i++) octets[i] = binaire.charCodeAt(i);
  return octets;
}
