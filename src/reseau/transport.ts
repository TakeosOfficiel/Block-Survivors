/**
 * Transport du multijoueur : une connexion WebRTC (DataChannel) entre l'hôte et
 * un joueur. La mise en relation (offre de l'hôte, réponse du joueur) passe
 * par la salle du code de salon (signalisation.ts) ; chaque description
 * contient tous les candidats ICE (collecte terminée avant de l'envoyer : pas
 * d'échange au fil de l'eau).
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

  /** Hôte : prépare une connexion et l'offre à envoyer au joueur. */
  static async offrir(): Promise<{ pair: ConnexionPair; sdp: string }> {
    const pair = new ConnexionPair();
    await pair.pc.setLocalDescription(await pair.pc.createOffer());
    await collecteTerminee(pair.pc);
    return { pair, sdp: pair.pc.localDescription!.sdp };
  }

  /** Joueur : accepte l'offre de l'hôte et prépare la réponse à lui renvoyer. */
  static async repondre(offre: string): Promise<{ pair: ConnexionPair; sdp: string }> {
    const pair = new ConnexionPair();
    await pair.pc.setRemoteDescription({ type: 'offer', sdp: offre });
    await pair.pc.setLocalDescription(await pair.pc.createAnswer());
    await collecteTerminee(pair.pc);
    return { pair, sdp: pair.pc.localDescription!.sdp };
  }

  /** Hôte : termine la mise en relation avec la réponse du joueur. */
  async accepter(reponse: string): Promise<void> {
    await this.pc.setRemoteDescription({ type: 'answer', sdp: reponse });
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
