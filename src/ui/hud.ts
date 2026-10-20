/**
 * HUD : chrono, ennemis vaincus, cœurs, expérience et niveau, armes et
 * passifs possédés, barre du boss, alertes, voile de blessure, FPS et
 * messages centraux.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { Arsenal } from '../combat/arsenal';
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { PROGRESSION } from '../donnees/progression';
import type { AfficheurAlerte } from '../entites/directeurBoss';
import type { Ennemis } from '../entites/ennemis';
import type { EtatPartie } from '../etat';
import type { Passifs } from '../progression/passifs';
import { formaterTemps } from '../utils/format';
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
  /** Secondes avant le prochain boss (−1 = plus aucun avant la fin). */
  tempsAvantBoss(): number;
  /** Vrai après 15:00 tant qu'un boss reste à vaincre (la victoire attend). */
  bossAVaincre(): boolean;
}

export class Hud implements AfficheurAlerte {
  private readonly elementFps = element('hud-fps');
  private readonly elementMessage = element('hud-message');
  private readonly elementAlerte = element('hud-alerte');
  private readonly elementChrono = element('hud-chrono');
  private readonly elementProchainBoss = element('hud-prochain-boss');
  private readonly elementTues = element('hud-tues-nombre');
  private readonly elementVignette = element('vignette');
  private readonly elementTotems = element('hud-totems');
  private readonly elementNombreTotems = element('hud-totems-nombre');
  private readonly elementAnimationTotem = element<HTMLImageElement>('hud-totem-anim');
  private readonly sources: SourcesHud;
  private readonly coeurs: BarreCoeurs;
  private readonly capacites: BarreCapacites;
  private readonly xp: BarreXp;
  private readonly boss: BarreBoss;

  private images = 0;
  private duree = 0;
  private minuterieMessage = 0;
  private minuterieAlerte = 0;
  private secondeAffichee = -1;
  private secondeBossAffichee = -2;
  private tuesAffiches = -1;
  private opaciteVignette = 0;
  private totemsAffiches = -1;
  private viesUtiliseesVues = 0;
  private tempsReel = 0;

  /** Dernière mesure (lue aussi par le debug). */
  fps = 0;
  msParImage = 0;

  constructor(textures: BibliothequeTextures, sources: SourcesHud) {
    this.sources = sources;
    this.coeurs = new BarreCoeurs(textures);
    this.capacites = new BarreCapacites(textures);
    this.xp = new BarreXp(textures);
    this.boss = new BarreBoss(textures);
    const icone = textures.url(CONFIG.hud.iconeTues);
    const image = element<HTMLImageElement>('hud-tues-icone');
    if (icone) image.src = icone;
    else image.classList.add('cache');
    const totem = textures.url(CONFIG.joueur.totem.icone);
    if (totem) {
      element<HTMLImageElement>('hud-totems-icone').src = totem;
      this.elementAnimationTotem.src = totem;
    }
  }

  /** Affiche ou masque le compteur de FPS (option). */
  set afficherFps(visible: boolean) {
    this.elementFps.classList.toggle('cache', !visible);
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
    const j = etat.joueur;
    this.tempsReel += dtReel;
    this.mesurerFps(dtBrut);
    this.coeurs.afficher(j.pv, j.pvMax, j.bouclier, dtReel);
    this.xp.afficher(j);
    this.capacites.afficher(arsenal, passifs);
    this.boss.afficher(ennemis.bossActif());
    this.afficherChrono(etat.tempsJeu);
    this.afficherProchainBoss();
    if (etat.ennemisTues !== this.tuesAffiches) {
      this.tuesAffiches = etat.ennemisTues;
      this.elementTues.textContent = String(etat.ennemisTues);
    }
    this.afficherVignette(etat);
    this.afficherTotems(etat);

    if (this.minuterieMessage > 0) {
      this.minuterieMessage -= dtReel;
      if (this.minuterieMessage <= 0) this.elementMessage.classList.remove('visible');
    }
    if (this.minuterieAlerte > 0) {
      this.minuterieAlerte -= dtReel;
      if (this.minuterieAlerte <= 0) this.elementAlerte.classList.add('cache');
    }
  }

  private afficherChrono(temps: number): void {
    const seconde = Math.floor(temps);
    if (seconde === this.secondeAffichee) return;
    this.secondeAffichee = seconde;
    this.elementChrono.textContent = formaterTemps(seconde);
    this.elementChrono.classList.toggle('final', PROGRESSION.dureePartie - seconde <= CONFIG.hud.dernierCompteARebours);
  }

  /** « Prochain boss dans 02:13 » sous le chrono ; rouge dans les 10 dernières secondes. */
  private afficherProchainBoss(): void {
    const dans = this.sources.tempsAvantBoss();
    const aVaincre = this.sources.bossAVaincre();
    // −2 : temps écoulé, boss encore vivant
    const seconde = aVaincre ? -2 : dans < 0 ? -1 : Math.ceil(dans);
    if (seconde === this.secondeBossAffichee) return;  // le DOM ne change qu'une fois par seconde
    this.secondeBossAffichee = seconde;
    this.elementProchainBoss.textContent = aVaincre
      ? 'Vainquez le boss pour gagner !'
      : dans < 0 ? '' : `Prochain boss dans ${formaterTemps(seconde)}`;
    this.elementProchainBoss.classList.toggle('cache', dans < 0 && !aVaincre);
    this.elementProchainBoss.classList.toggle('proche', aVaincre || (dans >= 0 && dans <= 10));
  }

  /** Totems restants, et animation du totem qui jaillit à chaque résurrection. */
  private afficherTotems(etat: EtatPartie): void {
    const j = etat.joueur;
    const restants = Math.max(0, etat.modificateurs.vies - j.viesUtilisees);
    if (restants !== this.totemsAffiches) {
      this.totemsAffiches = restants;
      this.elementTotems.classList.toggle('cache', restants === 0);
      this.elementNombreTotems.textContent = `×${restants}`;
    }
    if (j.viesUtilisees !== this.viesUtiliseesVues) {
      this.viesUtiliseesVues = j.viesUtilisees;
      const anim = this.elementAnimationTotem;
      anim.classList.remove('cache', 'joue');
      void anim.offsetWidth;  // relance l'animation CSS
      anim.classList.add('joue');
    }
  }

  /** Voile rouge : fort juste après un coup, pulsant quand la santé est basse. */
  private afficherVignette(etat: EtatPartie): void {
    const v = CONFIG.hud.vignette;
    const j = etat.joueur;
    const blessure = j.mort ? 0 : Math.min(1, Math.max(0, j.invulnerabilite / JOUEUR_BASE.invulnerabilite)) * v.blessure;
    const santeBasse = !j.mort && j.pv / j.pvMax <= v.seuilSanteBasse
      ? v.santeBasse * (0.6 + 0.4 * Math.sin(this.tempsReel * v.pulsation))
      : 0;
    const opacite = Math.max(blessure, santeBasse);
    if (Math.abs(opacite - this.opaciteVignette) < 0.01) return;
    this.opaciteVignette = opacite;
    this.elementVignette.style.opacity = opacite.toFixed(2);
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
