/**
 * Entrées : état brut du clavier, de la molette et de la souris capturée.
 * Les autres systèmes lisent ces valeurs ; aucune logique de jeu ici.
 */
import { CONFIG, type ActionDeplacement } from '../config';

export class Entrees {
  private readonly touches = new Map<string, boolean>();  // event.code → enfoncée
  private readonly codesControles = new Set<string>(Object.values(CONFIG.controles).flat());
  private molette = 0;  // défilement cumulé (pixels) pas encore consommé
  /** Déplacement de la souris capturée (pixels) pas encore consommé. */
  private sourisX = 0;
  private sourisY = 0;
  /** Le premier mouvement après une capture est souvent faux (saut du curseur) : on l'ignore. */
  private ignorerProchainMouvement = false;

  constructor() {
    window.addEventListener('keydown', this.surToucheEnfoncee);
    window.addEventListener('keyup', this.surToucheRelachee);
    // Fenêtre qui perd le focus : on relâche tout pour éviter un joueur qui court seul
    window.addEventListener('blur', this.toutRelacher);
    window.addEventListener('wheel', this.surMolette, { passive: false });
    document.addEventListener('mousemove', this.surSouris);
    document.addEventListener('pointerlockchange', () => { this.ignorerProchainMouvement = true; });
  }

  /** Vrai si la souris est capturée (vue à la 3e personne). */
  get sourisCapturee(): boolean {
    return document.pointerLockElement !== null;
  }

  /** Capture la souris sur l'élément (le curseur disparaît, ses mouvements tournent la caméra). */
  capturerSouris(element: HTMLElement): void {
    if (this.sourisCapturee) return;
    // Mouvement brut de la souris (sans l'accélération du système) : évite aussi les sauts
    // de caméra de Chrome sous Windows ; repli sur la capture simple s'il n'est pas proposé
    const capturer = (options?: { unadjustedMovement: boolean }) => {
      try {
        // Navigateurs récents : une promesse, refusée si le geste de l'utilisateur manque
        const resultat = (element.requestPointerLock as (o?: object) => unknown).call(element, options);
        return resultat instanceof Promise ? resultat : Promise.resolve();
      } catch (erreur) {
        return Promise.reject(erreur);
      }
    };
    capturer({ unadjustedMovement: true })
      .catch((erreur: unknown) => (erreur as Error | null)?.name === 'NotSupportedError' ? capturer() : undefined)
      .catch(() => {
        // Capture refusée : la caméra garde son orientation
      });
  }

  libererSouris(): void {
    if (this.sourisCapturee) document.exitPointerLock();
  }

  /** Déplacement de la souris capturée depuis le dernier appel, puis remise à zéro. */
  consommerSouris(sortie: { x: number; y: number }): void {
    sortie.x = this.sourisX;
    sortie.y = this.sourisY;
    this.sourisX = 0;
    this.sourisY = 0;
  }

  /** Vrai si au moins une des touches liées à l'action est enfoncée. */
  action(nom: ActionDeplacement): boolean {
    const codes = CONFIG.controles[nom];
    for (let i = 0; i < codes.length; i++) {
      if (this.touches.get(codes[i])) return true;
    }
    return false;
  }

  /** Défilement accumulé depuis le dernier appel, puis remise à zéro. */
  consommerMolette(): number {
    const valeur = this.molette;
    this.molette = 0;
    return valeur;
  }

  private readonly surToucheEnfoncee = (e: KeyboardEvent): void => {
    if (!this.codesControles.has(e.code)) return;
    this.touches.set(e.code, true);
    e.preventDefault();
  };

  private readonly surToucheRelachee = (e: KeyboardEvent): void => {
    this.touches.set(e.code, false);
  };

  private readonly toutRelacher = (): void => {
    this.touches.clear();
  };

  private readonly surSouris = (e: MouseEvent): void => {
    if (!this.sourisCapturee) return;
    // Sauts aberrants (curseur qui « rebondit » au bord de la fenêtre) : ignorés
    const saut = CONFIG.camera.sautSourisMax;
    if (this.ignorerProchainMouvement || Math.abs(e.movementX) > saut || Math.abs(e.movementY) > saut) {
      this.ignorerProchainMouvement = false;
      return;
    }
    this.sourisX += e.movementX;
    this.sourisY += e.movementY;
  };

  /** Ctrl + molette reste le zoom du navigateur. */
  private readonly surMolette = (e: WheelEvent): void => {
    if (e.ctrlKey) return;
    e.preventDefault();
    this.molette += e.deltaMode === WheelEvent.DOM_DELTA_LINE ? e.deltaY * 16
                  : e.deltaMode === WheelEvent.DOM_DELTA_PAGE ? e.deltaY * 400
                  : e.deltaY;
  };
}
