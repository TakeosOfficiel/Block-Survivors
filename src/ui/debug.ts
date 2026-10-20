/**
 * Panneau debug (F3 ou ²), compact : mesures, vitesse du temps, invincibilité,
 * frappe de zone, niveaux, boss ; rangées d'icônes pour faire apparaître
 * chaque ennemi, donner chaque arme ou bonus (exclusifs compris) et poser
 * chaque objet ; changement de personnage et d'arène.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG, SOURCES_TEXTURES } from '../config';
import { ARENES } from '../donnees/arenes';
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
import { ENNEMIS, IDS_ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { IDS_OBJETS, OBJETS, type IdObjet } from '../donnees/objets';
import { IDS_PASSIFS, PASSIFS, type IdPassif } from '../donnees/passifs';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { DefinitionArene } from '../donnees/types';
import type { EtatPartie } from '../etat';
import type { CameraJeu } from '../moteur/camera';
import type { Rendu } from '../moteur/rendu';
import { formaterTemps } from '../utils/format';
import { borner } from '../utils/maths';
import { element } from './dom';
import type { Hud } from './hud';
import { portrait } from './portraits';

/** Ce que le panneau observe et les actions qu'il déclenche. */
export interface SourcesDebug {
  readonly rendu: Rendu;
  readonly etat: EtatPartie;
  readonly hud: Hud;
  readonly camera: CameraJeu;
  readonly textures: BibliothequeTextures;
  compterEntites(): number;
  compterParticules(): number;
  definirInvincibilite(actif: boolean): void;
  apparaitreEnnemi(id: IdEnnemi): void;
  ameliorerArme(id: IdArme): void;
  ajouterBonus(id: IdPassif): void;
  poserObjet(id: IdObjet): void;
  gagnerNiveau(): void;
  apparaitreBoss(): void;
  allerVersFin(): void;
  changerPersonnage(id: IdPersonnage): void;
  changerArene(arene: DefinitionArene): void;
  frapper(): void;
}

export class PanneauDebug {
  private readonly sources: SourcesDebug;
  private readonly dom = {
    panneau: element('debug'),
    fps: element('dbg-fps'),
    ms: element('dbg-ms'),
    entites: element('dbg-entites'),
    particules: element('dbg-particules'),
    appels: element('dbg-appels'),
    triangles: element('dbg-triangles'),
    textures: element('dbg-textures'),
    temps: element('dbg-temps'),
    pv: element('dbg-pv'),
    tues: element('dbg-tues'),
    position: element('dbg-position'),
    zoom: element('dbg-zoom'),
    vitesse: element('dbg-vitesse'),
    ralentir: element<HTMLButtonElement>('dbg-ralentir'),
    accelerer: element<HTMLButtonElement>('dbg-accelerer'),
    invincible: element<HTMLButtonElement>('dbg-invincible'),
    apparitions: element('dbg-apparitions'),
    armes: element('dbg-armes'),
    bonus: element('dbg-bonus'),
    objets: element('dbg-objets'),
    frappe: element<HTMLButtonElement>('dbg-frappe'),
    niveau: element<HTMLButtonElement>('dbg-niveau'),
    boss: element<HTMLButtonElement>('dbg-boss'),
    fin: element<HTMLButtonElement>('dbg-fin'),
    personnages: element('dbg-personnages'),
    arenes: element('dbg-arenes'),
  };

  private visible: boolean = CONFIG.debug.actifAuDemarrage;
  private invincible = false;
  private indexVitesseTemps: number = CONFIG.debug.vitessesTemps.indexOf(1);
  private minuterie = 0;

  /** Multiplicateur du temps de simulation. */
  vitesseTemps = 1;

  constructor(sources: SourcesDebug) {
    this.sources = sources;

    this.lierBouton(this.dom.ralentir, () => this.changerVitesseTemps(-1));
    this.lierBouton(this.dom.accelerer, () => this.changerVitesseTemps(1));
    this.lierBouton(this.dom.invincible, () => this.basculerInvincibilite());
    this.lierBouton(this.dom.frappe, () => this.sources.frapper());
    this.lierBouton(this.dom.niveau, () => this.sources.gagnerNiveau());
    this.lierBouton(this.dom.boss, () => this.sources.apparaitreBoss());
    this.lierBouton(this.dom.fin, () => this.sources.allerVersFin());

    // Une icône par ennemi, arme, bonus et objet, générées depuis les tables de données
    const { textures } = sources;
    for (const id of IDS_ENNEMIS) {
      const def = ENNEMIS[id];
      this.dom.apparitions.append(this.creerIcone(def.nom, portrait(textures, def), () => this.sources.apparaitreEnnemi(id)));
    }
    for (const id of IDS_ARMES) {
      const def = ARMES[id];
      this.dom.armes.append(this.creerIcone(`${def.nom} : ajouter / +1 rang`, textures.url(def.icone), () => this.sources.ameliorerArme(id)));
    }
    for (const id of IDS_PASSIFS) {
      const def = PASSIFS[id];
      const bouton = this.creerIcone(`${def.nom}${def.exclusif ? ' (exclusif)' : ''} : ajouter / +1 rang`, textures.url(def.icone), () => this.sources.ajouterBonus(id));
      bouton.classList.toggle('exclusif', def.exclusif === true);
      this.dom.bonus.append(bouton);
    }
    for (const id of IDS_OBJETS) {
      const def = OBJETS[id];
      this.dom.objets.append(this.creerIcone(`${def.nom} : poser à côté`, textures.url(def.icone), () => this.sources.poserObjet(id)));
    }
    for (const id of IDS_PERSONNAGES) {
      const def = PERSONNAGES[id];
      this.dom.personnages.append(this.creerIcone(`${def.nom} (relance la partie)`, portrait(textures, def), () => this.sources.changerPersonnage(id)));
    }
    for (const arene of ARENES) {
      this.dom.arenes.append(this.creerIcone(`${arene.nom} (relance la partie)`, textures.url(arene.icone), () => this.sources.changerArene(arene)));
    }

    window.addEventListener('keydown', this.surTouche);
    this.changerVitesseTemps(0);
    this.appliquerVisibilite();
  }

  majVisuel(dtReel: number): void {
    if (!this.visible) return;
    this.minuterie += dtReel;
    if (this.minuterie < CONFIG.debug.periodeRafraichissement) return;
    this.minuterie = 0;
    this.rafraichir();
  }

  /** Bouton carré à icône (nom en infobulle), ou à initiale si l'image manque. */
  private creerIcone(nom: string, url: string | null, action: () => void): HTMLButtonElement {
    const bouton = this.creerBouton(url ? '' : nom.charAt(0), action);
    bouton.className = 'icone';
    bouton.title = nom;
    if (url) {
      const image = document.createElement('img');
      image.src = url;
      image.alt = nom;
      bouton.append(image);
    }
    return bouton;
  }

  private creerBouton(texte: string, action: () => void): HTMLButtonElement {
    const bouton = document.createElement('button');
    bouton.textContent = texte;
    this.lierBouton(bouton, action);
    return bouton;
  }

  /** Le bouton rend le focus après le clic : Espace/Entrée ne le redéclenchent pas. */
  private lierBouton(bouton: HTMLButtonElement, action: () => void): void {
    bouton.addEventListener('click', () => {
      action();
      bouton.blur();
    });
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if ((CONFIG.debug.touchesOverlay as readonly string[]).includes(e.code)) {
      e.preventDefault();
      if (!e.repeat) this.basculer();
      return;
    }
    // Raccourcis actifs seulement quand le panneau est ouvert
    if (!this.visible || e.repeat) return;
    if (e.key === '+' || e.key === '=') this.changerVitesseTemps(1);
    else if (e.key === '-' || e.key === '_') this.changerVitesseTemps(-1);
    else if (e.code === 'KeyI') this.basculerInvincibilite();
    else if (e.code === 'KeyK') this.sources.frapper();
    else if (e.code === 'KeyL') this.sources.gagnerNiveau();
    else if (e.code === 'KeyB') this.sources.apparaitreBoss();
  };

  private basculer(): void {
    this.visible = !this.visible;
    this.appliquerVisibilite();
  }

  private appliquerVisibilite(): void {
    this.dom.panneau.classList.toggle('cache', !this.visible);
    if (this.visible) this.rafraichir();
  }

  private changerVitesseTemps(sens: number): void {
    const liste = CONFIG.debug.vitessesTemps;
    this.indexVitesseTemps = borner(this.indexVitesseTemps + sens, 0, liste.length - 1);
    this.vitesseTemps = liste[this.indexVitesseTemps];
    this.dom.vitesse.textContent = `×${this.vitesseTemps}`;
  }

  private basculerInvincibilite(): void {
    this.invincible = !this.invincible;
    this.sources.definirInvincibilite(this.invincible);
    this.dom.invincible.classList.toggle('actif', this.invincible);
  }

  private rafraichir(): void {
    const { rendu, etat, hud, camera, textures } = this.sources;
    const infos = rendu.renderer.info.render;
    const bilan = textures.bilan;
    const detailSources = SOURCES_TEXTURES
      .filter((s) => bilan.parSource.has(s.id))
      .map((s) => `${s.nom} ${bilan.parSource.get(s.id)}`)
      .join(', ');

    this.dom.fps.textContent = String(hud.fps);
    this.dom.ms.textContent = hud.msParImage.toFixed(1);
    this.dom.entites.textContent = String(this.sources.compterEntites());
    this.dom.particules.textContent = String(this.sources.compterParticules());
    this.dom.appels.textContent = `${infos.drawCalls} (${rendu.api})`;
    this.dom.triangles.textContent = String(infos.triangles);
    this.dom.textures.textContent = `${bilan.chargees} / ${bilan.demandees}${detailSources ? ` (${detailSources})` : ''}`;
    this.dom.temps.textContent = formaterTemps(etat.tempsJeu);
    this.dom.pv.textContent = `${Math.ceil(etat.joueur.pv)} / ${etat.joueur.pvMax}`;
    this.dom.tues.textContent = String(etat.ennemisTues);
    this.dom.position.textContent = `${etat.joueur.x.toFixed(1)} / ${etat.joueur.z.toFixed(1)}`;
    this.dom.zoom.textContent = camera.distance.toFixed(1);
  }
}
