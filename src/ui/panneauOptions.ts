/**
 * Écran des options (depuis le menu ou la pause) : volume, compteur de FPS,
 * secousses de l'écran. Chaque changement est appliqué et enregistré aussitôt.
 */
import { enregistrerOptions, NIVEAUX_SHADERS, STYLES_HERBE, type NiveauShaders, type Options, type StyleHerbe } from '../options';

const LIBELLES_SHADERS: Readonly<Record<NiveauShaders, string>> = {
  aucun: 'Désactivés',
  legers: 'Légers',
  complets: 'Complets',
};

const LIBELLES_HERBE: Readonly<Record<StyleHerbe, string>> = {
  minecraft: 'Minecraft',
  touffes: 'Touffes ondulantes',
  pixels: 'Pixels 3D',
  stylisee: 'Stylisée',
  realiste: 'Réaliste',
};
import { element } from './dom';

export class PanneauOptions {
  private readonly racine = element('options');
  private readonly volume = element<HTMLInputElement>('option-volume');
  private readonly texteVolume = element('option-volume-texte');
  private readonly boutonFps = element<HTMLButtonElement>('option-fps');
  private readonly boutonSecousses = element<HTMLButtonElement>('option-secousses');
  private readonly boutonHerbe = element<HTMLButtonElement>('option-herbe');
  private readonly boutonShaders = element<HTMLButtonElement>('option-shaders');
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
    this.boutonHerbe.addEventListener('click', () => {
      const i = STYLES_HERBE.indexOf(this.options.herbe);
      this.options.herbe = STYLES_HERBE[(i + 1) % STYLES_HERBE.length];
      this.changer();
    });
    element<HTMLButtonElement>('options-fermer').addEventListener('click', () => this.fermer());
    // Enregistré avant les autres écrans : Échap ferme d'abord les options
    window.addEventListener('keydown', this.surTouche);
    this.rafraichir();
    this.appliquer(this.options);
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

  private changer(): void {
    enregistrerOptions(this.options);
    this.appliquer(this.options);
    this.rafraichir();
  }

  private rafraichir(): void {
    const o = this.options;
    const pourcent = Math.round(o.volume * 100);
    this.volume.value = String(pourcent);
    this.texteVolume.textContent = pourcent === 0 ? 'Sons : désactivés' : `Sons : ${pourcent} %`;
    this.boutonFps.textContent = `Compteur de FPS : ${o.afficherFps ? 'OUI' : 'NON'}`;
    this.boutonSecousses.textContent = `Secousses de l'écran : ${o.secousses ? 'OUI' : 'NON'}`;
    this.boutonHerbe.textContent = `Herbe : ${LIBELLES_HERBE[o.herbe]}`;
    this.boutonShaders.textContent = `Shaders : ${LIBELLES_SHADERS[o.shaders]}`;
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Escape' || !this.ouvert) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    if (!e.repeat) this.fermer();
  };
}
