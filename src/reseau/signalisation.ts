/**
 * Mise en relation par un code de salon court (« K7F-3QX ») : l'hôte le donne,
 * le joueur le tape, et c'est tout. Le jeu n'a pas de serveur : la poignée de
 * main WebRTC (offre de l'hôte, réponse du joueur) passe par des relais Nostr
 * publics, de simples boîtes aux lettres. Une fois les deux joueurs reliés,
 * plus rien ne passe par eux : la partie va directement d'un joueur à l'autre.
 *
 * Messages éphémères (les relais ne les gardent pas), publiés sur tous les
 * relais à la fois (le premier arrivé l'emporte), sous une étiquette tirée du
 * code et chiffrés (AES-GCM) avec une clé tirée du code : sans le code, on ne
 * trouve pas le salon et on ne lit rien. Chaque ouverture signe avec une clé
 * jetable (Nostr exige une signature, elle n'identifie personne).
 */
import { schnorr } from '@noble/curves/secp256k1.js';

const RELAIS = ['wss://relay.damus.io', 'wss://nos.lol', 'wss://relay.primal.net', 'wss://relay.nostr.band', 'wss://nostr.mom'];
/** Sorte d'événement Nostr éphémère (20000 à 29999 : relayé, jamais gardé). */
const SORTE = 25417;
/** Lettres et chiffres qu'on ne confond pas (ni O ni 0, ni I, L ni 1). */
const ALPHABET = 'ABCDEFGHJKMNPQRSTUVWXYZ23456789';
const LONGUEUR_CODE = 6;

export type MessageSalle =
  /** Joueur → hôte : « je veux entrer » (`de` : son identifiant jetable). */
  | { readonly t: 'demande'; readonly de: string }
  /** Hôte → joueur `a` : la description WebRTC de l'hôte. */
  | { readonly t: 'offre'; readonly a: string; readonly sdp: string }
  /** Joueur → hôte : sa réponse. */
  | { readonly t: 'reponse'; readonly de: string; readonly sdp: string };

/** Nouveau code de salon, « ABC-DEF ». */
export function nouveauCode(): string {
  const octets = crypto.getRandomValues(new Uint8Array(LONGUEUR_CODE));
  let code = '';
  for (let i = 0; i < LONGUEUR_CODE; i++) code += ALPHABET[octets[i] % ALPHABET.length];
  return `${code.slice(0, 3)}-${code.slice(3)}`;
}

/** Code tapé par le joueur, remis en forme (minuscules, espaces, tiret oublié) ; null s'il n'en est pas un. */
export function normaliserCode(saisie: string): string | null {
  const brut = saisie.toUpperCase().replace(/[^A-Z0-9]/g, '');
  if (brut.length !== LONGUEUR_CODE || [...brut].some((c) => !ALPHABET.includes(c))) return null;
  return `${brut.slice(0, 3)}-${brut.slice(3)}`;
}

/** Salle de rencontre d'un code : ce qui y est publié arrive à tous ceux qui l'ont ouverte. */
export class Salle {
  /** Message arrivé (les siens exclus, chacun une seule fois). */
  surMessage: (message: MessageSalle) => void = () => {};

  private readonly etiquette: string;
  private readonly cle: CryptoKey;
  private readonly secret = schnorr.utils.randomSecretKey();
  private readonly publique: string;
  private readonly sockets: WebSocket[] = [];
  private readonly vus = new Set<string>();
  private fermee = false;

  private constructor(etiquette: string, cle: CryptoKey) {
    this.etiquette = etiquette;
    this.cle = cle;
    this.publique = hex(schnorr.getPublicKey(this.secret));
  }

  /** Ouvre la salle du code sur tous les relais ; résolue dès qu'un relais écoute (sinon une erreur). */
  static async ouvrir(code: string): Promise<Salle> {
    const encodeur = new TextEncoder();
    const etiquette = hex(await empreinte(encodeur.encode(`block-survivors/salon/${code}`))).slice(0, 32);
    const cle = await crypto.subtle.importKey('raw', await empreinte(encodeur.encode(`block-survivors/cle/${code}`)), 'AES-GCM', false, ['encrypt', 'decrypt']);
    const salle = new Salle(etiquette, cle);
    await salle.connecter();
    return salle;
  }

  /** Publie sur tous les relais connectés. */
  async publier(message: MessageSalle): Promise<void> {
    if (this.fermee) return;
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const chiffre = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, this.cle, new TextEncoder().encode(JSON.stringify(message))));
    const contenu = base64(iv) + '.' + base64(chiffre);
    const evenement = {
      pubkey: this.publique,
      created_at: Math.floor(Date.now() / 1000),
      kind: SORTE,
      tags: [['t', this.etiquette]],
      content: contenu,
    };
    const id = await empreinte(new TextEncoder().encode(JSON.stringify([0, evenement.pubkey, evenement.created_at, evenement.kind, evenement.tags, evenement.content])));
    const signe = { ...evenement, id: hex(id), sig: hex(schnorr.sign(id, this.secret)) };
    this.vus.add(signe.id);
    const texte = JSON.stringify(['EVENT', signe]);
    for (const s of this.sockets) if (s.readyState === WebSocket.OPEN) s.send(texte);
  }

  fermer(): void {
    this.fermee = true;
    for (const s of this.sockets) s.close();
  }

  private connecter(): Promise<void> {
    return new Promise((resoudre, rejeter) => {
      let echecs = 0;
      const abonnement = JSON.stringify(['REQ', 'salle', { kinds: [SORTE], '#t': [this.etiquette], since: Math.floor(Date.now() / 1000) - 30 }]);
      const echouer = () => {
        if (++echecs === RELAIS.length) rejeter(new Error('Aucun relais joignable : vérifiez votre connexion à Internet'));
      };
      for (const url of RELAIS) {
        let s: WebSocket;
        try {
          s = new WebSocket(url);
        } catch {
          echouer();
          continue;
        }
        let ouvert = false;
        s.onopen = () => {
          ouvert = true;
          s.send(abonnement);
          resoudre();
        };
        s.onerror = () => {
          if (!ouvert) echouer();
        };
        s.onmessage = (e) => void this.recevoir(e.data);
        this.sockets.push(s);
      }
    });
  }

  private async recevoir(donnees: unknown): Promise<void> {
    let message: unknown;
    try {
      message = JSON.parse(String(donnees));
    } catch {
      return;
    }
    if (!Array.isArray(message) || message[0] !== 'EVENT') return;
    const evenement = message[2] as { id?: unknown; content?: unknown } | undefined;
    if (!evenement || typeof evenement.id !== 'string' || typeof evenement.content !== 'string' || this.vus.has(evenement.id)) return;
    this.vus.add(evenement.id);
    const [iv, chiffre] = evenement.content.split('.');
    try {
      const clair = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: depuisBase64(iv) }, this.cle, depuisBase64(chiffre));
      const contenu = JSON.parse(new TextDecoder().decode(clair)) as MessageSalle;
      if (!this.fermee) this.surMessage(contenu);
    } catch {
      // Pas chiffré avec ce code : ignoré
    }
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
