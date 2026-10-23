/**
 * Écran des options (depuis le menu ou la pause) : volume général et volume
 * de chaque famille de sons, compteur de FPS,
 * secousses de l'écran, ombres et vent, chiffres des dégâts. Chaque changement est appliqué et enregistré aussitôt.
 * En bas, exporter le profil actif ou en importer un (progression/transfert.ts).
 */
import type { CategorieSon } from '../donnees/types';
import { exporterProfil, importerProfil } from '../progression/transfert';
import { relancerSurLeMenu } from '../profils';
import { CATEGORIES_SONS, enregistrerOptions, MODES_VUE, NIVEAUX_SHADERS, type ModeVue, type NiveauShaders, type Options } from '../options';

const LIBELLES_CATEGORIES: Readonly<Record<CategorieSon, string>> = {
  joueur: 'Joueur',
  armes: 'Armes',
  creatures: 'Créatures',
  interface: 'Interface',
};

const LIBELLES_VUES: Readonly<Record<ModeVue, string>> = {
  dessus: 'DE DESSUS',
  troisieme: '3E PERSONNE',
};

/** Passe à la vue suivante (bouton des options et touche F5). */
export function vueSuivante(options: Options): void {
  options.vue = MODES_VUE[(MODES_VUE.indexOf(options.vue) + 1) % MODES_VUE.length];
}

const LIBELLES_SHADERS: Readonly<Record<NiveauShaders, string>> = {
  aucun: 'NON',
  legers: 'OUI',
  complets: 'OUI (fines)',
};

import { element } from './dom';

export class PanneauOptions {
  private readonly racine = element('options');
  private readonly volume = element<HTMLInputElement>('option-volume');
  private readonly texteVolume = element('option-volume-texte');
  /** Curseur et texte de chaque famille de sons. */
  private readonly curseurs: { categorie: CategorieSon; entree: HTMLInputElement; texte: HTMLElement }[];
  private readonly boutonFps = element<HTMLButtonElement>('option-fps');
  private readonly boutonSecousses = element<HTMLButtonElement>('option-secousses');
  private readonly boutonShaders = element<HTMLButtonElement>('option-shaders');
  private readonly boutonDegats = element<HTMLButtonElement>('option-degats');
  private readonly boutonCompteur = element<HTMLButtonElement>('option-compteur');
  private readonly boutonVue = element<HTMLButtonElement>('option-vue');
  private readonly boutonEffetsAllies = element<HTMLButtonElement>('option-effets-allies');
  private readonly options: Options;
  private readonly appliquer: (options: Options) => void;
  private surFermeture: (() => void) | null = null;

  /** `appliquer` reçoit les options après chaque changement. */
  constructor(options: Options, appliquer: (options: Options) => void) {
    this.options = options;
    this.appliquer = appliquer;

    this.volume.addEventListener('input', () => {
      this.options.volume = Number(this.volume.value) / 100;
      this.changer();
    });
    // Un curseur par famille de sons, deux par rangée comme l'écran « Musique et sons » du jeu
    this.curseurs = CATEGORIES_SONS.map((categorie) => {
      const curseur = document.createElement('label');
      curseur.className = 'curseur-mc';
      const entree = document.createElement('input');
      Object.assign(entree, { type: 'range', min: '0', max: '100', step: '5' });
      const texte = document.createElement('span');
      curseur.append(entree, texte);
      entree.addEventListener('input', () => {
        this.options.volumes[categorie] = Number(entree.value) / 100;
        this.changer();
      });
      element('option-volumes').append(curseur);
      return { categorie, entree, texte };
    });
    this.boutonFps.addEventListener('click', () => {
      this.options.afficherFps = !this.options.afficherFps;
      this.changer();
    });
    this.boutonSecousses.addEventListener('click', () => {
      this.options.secousses = !this.options.secousses;
      this.changer();
    });
    this.boutonShaders.addEventListener('click', () => {
      const i = NIVEAUX_SHADERS.indexOf(this.options.shaders);
      this.options.shaders = NIVEAUX_SHADERS[(i + 1) % NIVEAUX_SHADERS.length];
      this.changer();
    });
    this.boutonDegats.addEventListener('click', () => {
      this.options.chiffresDegats = !this.options.chiffresDegats;
      this.changer();
    });
    this.boutonCompteur.addEventListener('click', () => this.modifier((o) => { o.compteurDegats = !o.compteurDegats; }));
    this.boutonVue.addEventListener('click', () => this.modifier(vueSuivante));
    this.boutonEffetsAllies.addEventListener('click', () => this.modifier((o) => { o.effetsAllies = !o.effetsAllies; }));
    element<HTMLButtonElement>('options-fermer').addEventListener('click', () => this.fermer());
    this.brancherSauvegarde();
    // Enregistré avant les autres écrans : Échap ferme d'abord les options
    window.addEventListener('keydown', this.surTouche);
    this.rafraichir();
    this.appliquer(this.options);
  }

  /** Exporter : le profil actif en fichier chiffré (.bssave). Importer : un nouveau profil, puis le jeu redémarre dessus. */
  private brancherSauvegarde(): void {
    const fichier = element<HTMLInputElement>('option-fichier');
    const texte = element('option-sauvegarde-texte');
    element<HTMLButtonElement>('option-exporter').addEventListener('click', async () => {
      await exporterProfil();
      texte.textContent = 'Profil exporté : dans l\'autre version du jeu, « Importer un profil ».';
    });
    element<HTMLButtonElement>('option-importer').addEventListener('click', () => fichier.click());
    fichier.addEventListener('change', async () => {
      const choisi = fichier.files?.[0];
      fichier.value = '';
      if (!choisi) return;
      if (!window.confirm('Ajouter le profil de ce fichier et jouer avec ? Le jeu va redémarrer (une partie en cours sera perdue).')) return;
      const erreur = await importerProfil(choisi);
      if (erreur) {
        texte.textContent = erreur;
        return;
      }
      relancerSurLeMenu();
    });
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  ouvrir(surFermeture?: () => void): void {
    this.surFermeture = surFermeture ?? null;
    this.rafraichir();
    this.racine.classList.remove('cache');
  }

  fermer(): void {
    if (!this.ouvert) return;
    this.racine.classList.add('cache');
    const rappel = this.surFermeture;
    this.surFermeture = null;
    rappel?.();
  }

  /** Change une option depuis le jeu (ex. touche Tab), enregistrée et appliquée comme un clic. */
  modifier(changement: (options: Options) => void): void {
    changement(this.options);
    this.changer();
  }

  private changer(): void {
    enregistrerOptions(this.options);
    this.appliquer(this.options);
    this.rafraichir();
  }

  private rafraichir(): void {
    const o = this.options;
    const pourcent = Math.round(o.volume * 100);
    this.volume.value = String(pourcent);
    this.texteVolume.textContent = pourcent === 0 ? 'Volume général : désactivé' : `Volume général : ${pourcent} %`;
    for (const { categorie, entree, texte } of this.curseurs) {
      const p = Math.round(o.volumes[categorie] * 100);
      entree.value = String(p);
      texte.textContent = `${LIBELLES_CATEGORIES[categorie]} : ${p === 0 ? 'NON' : `${p} %`}`;
    }
    this.boutonFps.textContent = `Compteur de FPS : ${o.afficherFps ? 'OUI' : 'NON'}`;
    this.boutonSecousses.textContent = `Secousses de l'écran : ${o.secousses ? 'OUI' : 'NON'}`;
    this.boutonShaders.textContent = `Ombres et vent : ${LIBELLES_SHADERS[o.shaders]}`;
    this.boutonDegats.textContent = `Chiffres des dégâts : ${o.chiffresDegats ? 'OUI' : 'NON'}`;
    this.boutonCompteur.textContent = `Compteur de dégâts (Tab) : ${o.compteurDegats ? 'OUI' : 'NON'}`;
    this.boutonVue.textContent = `Vue (F5) : ${LIBELLES_VUES[o.vue]}`;
    this.boutonEffetsAllies.textContent = `Effets des autres joueurs (multi) : ${o.effetsAllies ? 'OUI' : 'NON'}`;
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Escape' || !this.ouvert) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    if (!e.repeat) this.fermer();
  };
}
