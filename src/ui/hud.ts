/**
 * HUD : cœurs, expérience et niveau, armes et passifs possédés, barre du boss,
 * alertes, FPS et messages centraux.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { Arsenal } from '../combat/arsenal';
import { CONFIG } from '../config';
import type { AfficheurAlerte } from '../entites/directeurBoss';
import type { Ennemis } from '../entites/ennemis';
import type { EtatPartie } from '../etat';
import type { Passifs } from '../progression/passifs';
import { BarreBoss } from './barreBoss';
import { BarreCapacites } from './barreCapacites';
import { BarreXp } from './barreXp';
import { BarreCoeurs } from './coeurs';
import { element } from './dom';

/** Ce que le HUD affiche. */
export interface SourcesHud {
  readonly etat: EtatPartie;
  readonly arsenal: Arsenal;
  readonly passifs: Passifs;
  readonly ennemis: Ennemis;
}

export class Hud implements AfficheurAlerte {
  private readonly elementFps = element('hud-fps');
  private readonly elementMessage = element('hud-message');
  private readonly elementAlerte = element('hud-alerte');
  private readonly sources: SourcesHud;
  private readonly coeurs: BarreCoeurs;
  private readonly capacites: BarreCapacites;
  private readonly xp: BarreXp;
  private readonly boss: BarreBoss;

  private images = 0;
  private duree = 0;
  private minuterieMessage = 0;
  private minuterieAlerte = 0;

  /** Dernière mesure (lue aussi par le debug). */
  fps = 0;
  msParImage = 0;

  constructor(textures: BibliothequeTextures, sources: SourcesHud) {
    this.sources = sources;
    this.coeurs = new BarreCoeurs(textures);
    this.capacites = new BarreCapacites(textures);
    this.xp = new BarreXp(textures);
    this.boss = new BarreBoss(textures);
  }

  /** Affiche un message central pendant `duree` secondes (Infinity = jusqu'au prochain message). */
  afficherMessage(texte: string, duree: number = CONFIG.hud.dureeMessage): void {
    this.elementMessage.textContent = texte;
    this.elementMessage.classList.add('visible');
    this.minuterieMessage = duree;
  }

  /** Bandeau d'alerte rouge clignotant (arrivée d'un boss). */
  alerter(texte: string, duree: number): void {
    this.elementAlerte.textContent = texte;
    this.elementAlerte.classList.remove('cache');
    this.minuterieAlerte = duree;
  }

  /** dtBrut (non borné) mesure les FPS réels ; dtReel (borné) fait avancer les minuteries. */
  majVisuel(dtBrut: number, dtReel: number): void {
    const { etat, arsenal, passifs, ennemis } = this.sources;
    this.mesurerFps(dtBrut);
    this.coeurs.afficher(etat.joueur.pv, etat.joueur.pvMax);
    this.xp.afficher(etat.joueur);
    this.capacites.afficher(arsenal, passifs);
    this.boss.afficher(ennemis.bossActif());

    if (this.minuterieMessage > 0) {
      this.minuterieMessage -= dtReel;
      if (this.minuterieMessage <= 0) this.elementMessage.classList.remove('visible');
    }
    if (this.minuterieAlerte > 0) {
      this.minuterieAlerte -= dtReel;
      if (this.minuterieAlerte <= 0) this.elementAlerte.classList.add('cache');
    }
  }

  private mesurerFps(dtBrut: number): void {
    this.images++;
    this.duree += dtBrut;
    if (this.duree < CONFIG.hud.periodeFps) return;
    this.fps = Math.round(this.images / this.duree);
    this.msParImage = (this.duree * 1000) / this.images;
    this.images = 0;
    this.duree = 0;
    this.elementFps.textContent = `FPS : ${this.fps}`;
  }
}
