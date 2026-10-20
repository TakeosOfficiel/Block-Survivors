/**
 * Lecture des sons vanilla.
 *
 * Le serveur de ressources de Mojang ne renvoie pas d'en-têtes CORS : la Web
 * Audio API ne peut pas décoder ses fichiers. On passe donc par des éléments
 * <audio>, dont la lecture n'exige pas CORS, en réserve tournante par son.
 * La hauteur varie avec playbackRate, comme en jeu. Si une source échoue, la
 * suivante de SOURCES_SONS prend le relais ; sans aucune, le son est muet.
 */
import { SOURCES_SONS } from '../config';
import { IDS_SONS, SONS, type IdSon } from '../donnees/sons';
import type { CategorieSon } from '../donnees/types';
import { borner } from '../utils/maths';

/** Ce dont les systèmes du jeu ont besoin pour jouer un son. */
export interface LecteurSons {
  /** `hauteur` multiplie la hauteur tirée au hasard (1 = normale). */
  jouer(id: IdSon, hauteur?: number): void;
}

/** Nouvelles tentatives sur une même source avant de passer à la suivante (coupures réseau). */
const ESSAIS_PAR_SOURCE = 2;
const DELAI_NOUVEL_ESSAI_MS = 1500;

interface Canal {
  readonly id: IdSon;
  /** Éléments créés à la demande, jusqu'à `simultanes` (le premier dès le préchargement). */
  readonly elements: HTMLAudioElement[];
  suivant: number;
  /** Instant de la dernière lecture (s). */
  dernier: number;
  /** Index dans SOURCES_SONS ; au-delà, le son est muet. */
  source: number;
  essais: number;
  adresse: string;
}

export class Sons implements LecteurSons {
  private readonly canaux = new Map<IdSon, Canal>();
  private volume: number;
  private volumes: Readonly<Record<CategorieSon, number>>;

  constructor(volume: number, volumes: Readonly<Record<CategorieSon, number>>) {
    this.volume = volume;
    this.volumes = volumes;
  }

  /** Lance le téléchargement de tous les sons (un élément par son). */
  precharger(): void {
    for (const id of IDS_SONS) this.canal(id);
  }

  /** Volume général et volume de chaque famille de sons (0 à 1). */
  definirVolumes(volume: number, volumes: Readonly<Record<CategorieSon, number>>): void {
    this.volume = borner(volume, 0, 1);
    this.volumes = { ...volumes };
  }

  jouer(id: IdSon, hauteur = 1): void {
    const def = SONS[id];
    const volume = this.volume * this.volumes[def.categorie];
    if (volume <= 0) return;
    const canal = this.canal(id);
    if (canal.source >= SOURCES_SONS.length) return;
    const maintenant = performance.now() / 1000;
    if (maintenant - canal.dernier < def.intervalleMin) return;
    canal.dernier = maintenant;

    // Réserve tournante, agrandie à la demande (le fichier est alors déjà en cache)
    if (canal.suivant >= canal.elements.length) canal.elements.push(this.creerElement(canal));
    const element = canal.elements[canal.suivant];
    canal.suivant = (canal.suivant + 1) % def.simultanes;
    const [min, max] = def.hauteur;
    element.volume = borner(def.volume * volume, 0, 1);
    element.playbackRate = borner(hauteur * (min + Math.random() * (max - min)), 0.25, 4);
    element.currentTime = 0;
    // Refusé tant que l'utilisateur n'a pas interagi avec la page : sans gravité
    element.play().catch(() => undefined);
  }

  private canal(id: IdSon): Canal {
    let canal = this.canaux.get(id);
    if (canal) return canal;
    canal = { id, elements: [], suivant: 0, dernier: -Infinity, source: 0, essais: 0, adresse: this.adresse(id, 0) };
    canal.elements.push(this.creerElement(canal));
    this.canaux.set(id, canal);
    return canal;
  }

  private creerElement(canal: Canal): HTMLAudioElement {
    const element = new Audio();
    element.preload = 'auto';
    element.preservesPitch = false;
    element.addEventListener('error', () => {
      // Une seule réaction par échec de l'adresse courante, quel que soit l'élément qui le signale
      if (element.src === canal.adresse) this.echec(canal);
    });
    element.src = canal.adresse;
    return element;
  }

  /** Réessaie la même source après un délai, puis passe à la suivante. */
  private echec(canal: Canal): void {
    if (canal.essais < ESSAIS_PAR_SOURCE) {
      canal.essais++;
      const adresse = canal.adresse;
      window.setTimeout(() => {
        if (canal.adresse === adresse) this.appliquer(canal, adresse);
      }, DELAI_NOUVEL_ESSAI_MS);
      return;
    }
    canal.essais = 0;
    canal.source++;
    if (canal.source < SOURCES_SONS.length) this.appliquer(canal, this.adresse(canal.id, canal.source));
  }

  private appliquer(canal: Canal, adresse: string): void {
    canal.adresse = adresse;
    for (const element of canal.elements) {
      element.src = adresse;
      element.load();
    }
  }

  private adresse(id: IdSon, source: number): string {
    const def = SONS[id];
    const url = SOURCES_SONS[source]?.(def.chemin, def.hachage) ?? '';
    // element.src est toujours absolue : on compare avec la même forme
    return new URL(url, window.location.href).href;
  }
}
