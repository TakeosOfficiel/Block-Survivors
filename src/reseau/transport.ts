/**
 * Transport du multijoueur : une connexion WebRTC (DataChannel) entre l'hôte et
 * un joueur, sans serveur de signalisation. La mise en relation passe par deux
 * codes copiés-collés (Discord, SMS…) : l'offre de l'hôte, puis la réponse du
 * joueur. Chaque code contient la description de session et tous les candidats
 * ICE (collecte terminée avant de l'afficher : pas d'échange au fil de l'eau),
 * compressée puis encodée en base64url.
 *
 * Deux canaux négociés d'avance (mêmes identifiants des deux côtés) :
 * - `fiable` : ordonné, sans perte (salon, cartes, fin de partie), en JSON ;
 * - `rapide` : sans ordre ni renvoi (entrées, instantanés), en binaire.
 *
 * Seuls des serveurs STUN publics servent (adresse publique vue d'Internet) :
 * pas de relais TURN, donc deux NAT symétriques ne se joignent pas.
 */

/** Serveurs STUN publics (ils ne voient passer que la découverte d'adresse, jamais le jeu). */
const SERVEURS_ICE: RTCIceServer[] = [
  { urls: ['stun:stun.l.google.com:19302', 'stun:stun.cloudflare.com:3478'] },
];
/** Collecte des candidats ICE au plus aussi longue (ms) : au-delà, le code part avec ceux trouvés. */
const DELAI_COLLECTE = 4000;
/** Préfixe des codes : version du format, pour refuser proprement un code d'une autre version. */
const PREFIXE = 'BS1.';
/** Au-delà de ce volume en attente d'envoi (octets), les messages `rapide` sont abandonnés. */
const ENGORGEMENT = 256 * 1024;

export type CanalTransport = 'fiable' | 'rapide';

export interface ObservateurTransport {
  /** Message `fiable` (déjà décodé du JSON). */
  surMessage(message: unknown): void;
  /** Message `rapide` (binaire). */
  surDonnees(donnees: ArrayBuffer): void;
  /** Connexion perdue ou fermée. */
  surFermeture(): void;
}

/** Une connexion avec un autre joueur. */
export class ConnexionPair {
  private readonly pc: RTCPeerConnection;
  private readonly fiable: RTCDataChannel;
  private readonly rapide: RTCDataChannel;
  private observateur: ObservateurTransport | null = null;
  private ouverte: Promise<void>;
  private fermee = false;

  private constructor() {
    this.pc = new RTCPeerConnection({ iceServers: SERVEURS_ICE });
    this.fiable = this.pc.createDataChannel('fiable', { negotiated: true, id: 0, ordered: true });
    this.rapide = this.pc.createDataChannel('rapide', { negotiated: true, id: 1, ordered: false, maxRetransmits: 0 });
    this.rapide.binaryType = 'arraybuffer';
    this.ouverte = Promise.all([attendreOuverture(this.fiable), attendreOuverture(this.rapide)]).then(() => undefined);
    this.fiable.onmessage = (e) => {
      let message: unknown;
      try {
        message = JSON.parse(String(e.data));
      } catch {
        return;  // message illisible : ignoré
      }
      this.observateur?.surMessage(message);
    };
    this.rapide.onmessage = (e) => {
      if (e.data instanceof ArrayBuffer) this.observateur?.surDonnees(e.data);
    };
    const fermer = () => this.signalerFermeture();
    this.fiable.onclose = fermer;
    this.rapide.onclose = fermer;
    this.pc.onconnectionstatechange = () => {
      if (this.pc.connectionState === 'failed' || this.pc.connectionState === 'closed') fermer();
    };
  }

  /** Hôte : prépare une connexion et le code d'offre à envoyer au joueur. */
  static async offrir(): Promise<{ pair: ConnexionPair; code: string }> {
    const pair = new ConnexionPair();
    await pair.pc.setLocalDescription(await pair.pc.createOffer());
    await collecteTerminee(pair.pc);
    return { pair, code: await encoderCode('offre', pair.pc.localDescription!.sdp) };
  }

  /** Joueur : accepte le code d'offre de l'hôte et prépare le code de réponse à lui renvoyer. */
  static async repondre(codeOffre: string): Promise<{ pair: ConnexionPair; code: string }> {
    const sdp = await decoderCode(codeOffre, 'offre');
    const pair = new ConnexionPair();
    await pair.pc.setRemoteDescription({ type: 'offer', sdp });
    await pair.pc.setLocalDescription(await pair.pc.createAnswer());
    await collecteTerminee(pair.pc);
    return { pair, code: await encoderCode('reponse', pair.pc.localDescription!.sdp) };
  }

  /** Hôte : termine la mise en relation avec le code de réponse du joueur. */
  async accepter(codeReponse: string): Promise<void> {
    const sdp = await decoderCode(codeReponse, 'reponse');
    await this.pc.setRemoteDescription({ type: 'answer', sdp });
  }

  /** Résolue quand les deux canaux sont ouverts ; rejetée après `delai` ms. */
  attendre(delai = 20000): Promise<void> {
    return Promise.race([
      this.ouverte,
      new Promise<void>((_, rejeter) => setTimeout(() => rejeter(new Error('Connexion impossible (délai dépassé)')), delai)),
    ]);
  }

  observer(observateur: ObservateurTransport): void {
    this.observateur = observateur;
  }

  get active(): boolean {
    return !this.fermee && this.fiable.readyState === 'open';
  }

  envoyer(message: object): void {
    if (this.fiable.readyState === 'open') this.fiable.send(JSON.stringify(message));
  }

  /** Message `rapide` : abandonné si la connexion est engorgée (le suivant le remplacera). */
  envoyerDonnees(donnees: Uint8Array<ArrayBuffer>): void {
    if (this.rapide.readyState === 'open' && this.rapide.bufferedAmount < ENGORGEMENT) this.rapide.send(donnees);
  }

  fermer(): void {
    this.pc.close();
    this.signalerFermeture();
  }

  private signalerFermeture(): void {
    if (this.fermee) return;
    this.fermee = true;
    this.observateur?.surFermeture();
  }
}

function attendreOuverture(canal: RTCDataChannel): Promise<void> {
  if (canal.readyState === 'open') return Promise.resolve();
  return new Promise((resoudre) => canal.addEventListener('open', () => resoudre(), { once: true }));
}

/** Attend la fin de la collecte des candidats ICE (ou le délai). */
function collecteTerminee(pc: RTCPeerConnection): Promise<void> {
  if (pc.iceGatheringState === 'complete') return Promise.resolve();
  return new Promise((resoudre) => {
    const fin = () => {
      clearTimeout(minuterie);
      pc.removeEventListener('icegatheringstatechange', verifier);
      resoudre();
    };
    const verifier = () => {
      if (pc.iceGatheringState === 'complete') fin();
    };
    const minuterie = setTimeout(fin, DELAI_COLLECTE);
    pc.addEventListener('icegatheringstatechange', verifier);
  });
}

// --- Codes à copier-coller -----------------------------------------------

type SorteCode = 'offre' | 'reponse';

async function encoderCode(sorte: SorteCode, sdp: string): Promise<string> {
  const texte = JSON.stringify({ s: sorte === 'offre' ? 'o' : 'r', d: sdp });
  const compresse = await transformer(new TextEncoder().encode(texte), new CompressionStream('deflate-raw'));
  return PREFIXE + versBase64Url(compresse);
}

async function decoderCode(code: string, attendu: SorteCode): Promise<string> {
  const nettoye = code.replace(/\s+/g, '');
  if (!nettoye.startsWith(PREFIXE)) throw new Error('Ce n\'est pas un code de Block Survivors (ou d\'une autre version)');
  let contenu: { s?: unknown; d?: unknown };
  try {
    const octets = await transformer(depuisBase64Url(nettoye.slice(PREFIXE.length)), new DecompressionStream('deflate-raw'));
    contenu = JSON.parse(new TextDecoder().decode(octets)) as { s?: unknown; d?: unknown };
  } catch {
    throw new Error('Code illisible : vérifiez qu\'il est copié en entier');
  }
  const sorte = contenu.s === 'o' ? 'offre' : contenu.s === 'r' ? 'reponse' : null;
  if (sorte !== attendu || typeof contenu.d !== 'string') {
    throw new Error(attendu === 'offre' ? 'Il faut le code d\'invitation de l\'hôte' : 'Il faut le code de réponse du joueur');
  }
  return contenu.d;
}

async function transformer(octets: Uint8Array<ArrayBuffer>, flux: CompressionStream | DecompressionStream): Promise<Uint8Array<ArrayBuffer>> {
  const sortie = new Blob([octets]).stream().pipeThrough(flux);
  return new Uint8Array(await new Response(sortie).arrayBuffer());
}

function versBase64Url(octets: Uint8Array): string {
  let binaire = '';
  for (let i = 0; i < octets.length; i++) binaire += String.fromCharCode(octets[i]);
  return btoa(binaire).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function depuisBase64Url(texte: string): Uint8Array<ArrayBuffer> {
  const binaire = atob(texte.replace(/-/g, '+').replace(/_/g, '/'));
  const octets = new Uint8Array(binaire.length);
  for (let i = 0; i < binaire.length; i++) octets[i] = binaire.charCodeAt(i);
  return octets;
}
