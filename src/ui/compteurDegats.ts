/**
 * Compteur de dégâts : pour chaque arme, dégâts totaux, part du total, dégâts
 * par seconde depuis son obtention, coups, critiques, plus gros coup et
 * ennemis achevés. Un panneau compact en jeu (Tab pour l'afficher ou le
 * masquer, déplaçable par son titre) et un tableau complet dans le bilan.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { ArmeEquipee } from '../combat/armeEquipee';
import type { EtatPartie } from '../etat';
import { caseImage } from './bilan';
import { ecrireStockage, lireStockage } from '../options';
import { element } from './dom';

/** Une ligne du compteur. */
export interface LigneDegats {
  readonly arme: ArmeEquipee;
  readonly degats: number;
  readonly part: number;          // 0 à 1 du total
  readonly dps: number;
  readonly coups: number;
  readonly critiques: number;
  readonly plusGrosCoup: number;
  readonly tues: number;
}

/** Lignes triées de la plus forte à la plus faible ; le total en plus. */
export function lignesDegats(etat: EtatPartie, armes: readonly ArmeEquipee[]): { lignes: LigneDegats[]; total: number; dps: number } {
  const s = etat.stats;
  let total = 0;
  for (const arme of armes) total += s.degatsParArme[arme.index];
  const lignes = armes.map((arme) => {
    const i = arme.index;
    const degats = s.degatsParArme[i];
    const duree = Math.max(1, etat.tempsJeu - Math.max(0, s.acquisitionArme[i]));
    return {
      arme,
      degats,
      part: total > 0 ? degats / total : 0,
      dps: degats / duree,
      coups: s.coupsParArme[i],
      critiques: s.critiquesParArme[i],
      plusGrosCoup: s.plusGrosCoupParArme[i],
      tues: s.tuesParArme[i],
    };
  }).sort((a, b) => b.degats - a.degats);
  return { lignes, total, dps: total / Math.max(1, etat.tempsJeu) };
}

const ENTIER = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 });
const DECIMAL = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

/** « 845 », « 12,4 k », « 1,2 M » */
export function formaterDegats(valeur: number): string {
  if (valeur >= 1e6) return `${DECIMAL.format(valeur / 1e6)} M`;
  if (valeur >= 1e4) return `${DECIMAL.format(valeur / 1e3)} k`;
  return ENTIER.format(valeur);
}

/** Version courte pour le panneau en jeu : « 845 », « 1,8 k », « 2,4 M ». */
function abreger(valeur: number): string {
  if (valeur >= 1e6) return `${DECIMAL.format(valeur / 1e6)} M`;
  if (valeur >= 1e3) return `${DECIMAL.format(valeur / 1e3)} k`;
  return ENTIER.format(valeur);
}

function colonne(texte: string, classe?: string): HTMLElement {
  const span = document.createElement('span');
  if (classe) span.className = classe;
  span.textContent = texte;
  return span;
}

const CLE_POSITION = 'block-survivors.compteur-position';

/** Zoom d'interface courant (variable CSS --echelle). */
function echelleInterface(): number {
  return parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--echelle')) || 1;
}

/** Délai entre deux rafraîchissements du panneau (s réelles). */
const PERIODE = 0.5;

export class CompteurDegats {
  private readonly racine = element('compteur-degats');
  private readonly liste = element('compteur-degats-liste');
  private readonly pied = element('compteur-degats-total');
  private readonly etat: EtatPartie;
  private readonly armes: readonly ArmeEquipee[];
  private readonly textures: BibliothequeTextures;
  private minuterie = 0;
  private actif = true;

  constructor(etat: EtatPartie, armes: readonly ArmeEquipee[], textures: BibliothequeTextures) {
    this.etat = etat;
    this.armes = armes;
    this.textures = textures;
    this.rendreDeplacable();
  }

  /** Glisser le titre déplace le panneau ; la position est retenue d'une partie à l'autre. */
  private rendreDeplacable(): void {
    const titre = element('compteur-degats-titre');
    const position = lireStockage<{ x: number; y: number }>(CLE_POSITION);
    if (position && Number.isFinite(position.x) && Number.isFinite(position.y)) this.placer(position.x, position.y);
    let depart: { x: number; y: number; gauche: number; haut: number } | null = null;
    titre.addEventListener('pointerdown', (e) => {
      // Le HUD a un zoom d'interface : les déplacements de la souris sont divisés par lui
      depart = { x: e.clientX, y: e.clientY, gauche: this.racine.offsetLeft, haut: this.racine.offsetTop };
      titre.setPointerCapture(e.pointerId);
      e.preventDefault();
    });
    titre.addEventListener('pointermove', (e) => {
      if (!depart) return;
      const zoom = echelleInterface();
      this.placer(depart.gauche + (e.clientX - depart.x) / zoom, depart.haut + (e.clientY - depart.y) / zoom);
    });
    const lacher = () => {
      if (!depart) return;
      depart = null;
      ecrireStockage(CLE_POSITION, { x: this.racine.offsetLeft, y: this.racine.offsetTop });
    };
    titre.addEventListener('pointerup', lacher);
    titre.addEventListener('pointercancel', lacher);
  }

  /** Position (px d'interface) bornée à l'écran. */
  private placer(x: number, y: number): void {
    const zoom = echelleInterface();
    const largeur = window.innerWidth / zoom;
    const hauteur = window.innerHeight / zoom;
    this.racine.style.left = `${Math.min(Math.max(0, x), Math.max(0, largeur - this.racine.offsetWidth))}px`;
    this.racine.style.top = `${Math.min(Math.max(0, y), Math.max(0, hauteur - 40))}px`;
    this.racine.style.right = 'auto';
  }

  /** Affiché ou non (option, touche Tab). */
  definir(actif: boolean): void {
    this.actif = actif;
    this.racine.classList.toggle('cache', !actif);
    this.minuterie = 0;
  }

  get affiche(): boolean {
    return this.actif;
  }

  majVisuel(dtReel: number): void {
    if (!this.actif) return;
    this.minuterie -= dtReel;
    if (this.minuterie > 0) return;
    this.minuterie = PERIODE;
    const { lignes, total, dps } = lignesDegats(this.etat, this.armes);
    const maximum = Math.max(1, lignes[0]?.degats ?? 0);
    const entete = document.createElement('div');
    entete.className = 'compteur-ligne compteur-entete';
    entete.append(colonne(''), colonne('Arme'), colonne('Dégâts', 'nombre'), colonne('%', 'nombre'), colonne('/s', 'nombre'));
    this.liste.replaceChildren(entete, ...lignes.map((l) => {
      const def = l.arme.definition;
      const ligne = document.createElement('div');
      ligne.className = 'compteur-ligne';
      const barre = document.createElement('div');
      barre.className = 'compteur-barre';
      barre.style.width = `${(l.degats / maximum) * 100}%`;
      ligne.append(
        barre,
        caseImage(this.textures.url(def.icone), def.nom),
        colonne(def.nom, 'compteur-nom'),
        colonne(abreger(l.degats), 'nombre'),
        colonne(`${Math.round(l.part * 100)} %`, 'nombre'),
        colonne(abreger(l.dps), 'nombre'),
      );
      return ligne;
    }));
    this.pied.textContent = `Total ${abreger(total)} · ${abreger(dps)} dégâts/s`;
  }
}
