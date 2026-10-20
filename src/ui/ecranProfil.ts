/**
 * Profil (depuis le menu) : pseudo, skin (celle de la classe, une des skins
 * par défaut du jeu, ou un PNG importé) et gabarit des bras, avec un aperçu
 * de face. Chaque changement est enregistré aussitôt.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { NOMS_SKINS, SKINS_VANILLA } from '../donnees/skins';
import type { ApparenceEntite } from '../donnees/types';
import {
  apparenceJoueur, CHEMIN_SKIN_IMPORTE, cheminSkinVanilla, enregistrerProfil, LONGUEUR_PSEUDO, nettoyerPseudo,
  type ChoixSkin, type Profil,
} from '../profil';
import { caseImage } from './bilan';
import { element } from './dom';
import { oublierPortraits, portrait } from './portraits';

/** Taille du patron de skin attendu (et de ses multiples HD). */
const TAILLE_SKIN = 64;

export class EcranProfil {
  private readonly racine = element('profil');
  private readonly pseudo = element<HTMLInputElement>('profil-pseudo');
  private readonly skins = element('profil-skins');
  private readonly apercu = element<HTMLCanvasElement>('profil-apercu');
  private readonly boutonBras = element<HTMLButtonElement>('profil-bras');
  private readonly fichier = element<HTMLInputElement>('profil-fichier');
  private readonly erreur = element('profil-erreur');
  private readonly textures: BibliothequeTextures;
  readonly profil: Profil;
  /** Skin de la classe sélectionnée au menu (premier choix de la liste). */
  private apparenceClasse: ApparenceEntite | null = null;
  private surFermeture: (() => void) | null = null;

  constructor(textures: BibliothequeTextures, profil: Profil) {
    this.textures = textures;
    this.profil = profil;
    this.pseudo.maxLength = LONGUEUR_PSEUDO;
    this.pseudo.addEventListener('input', () => {
      this.profil.pseudo = nettoyerPseudo(this.pseudo.value);
      enregistrerProfil(this.profil);
    });
    // Les touches tapées dans le pseudo ne doivent pas lancer la partie (Entrée) ni fermer (Échap)
    this.pseudo.addEventListener('keydown', (e) => {
      e.stopPropagation();
      if (e.code === 'Enter') this.pseudo.blur();
    });
    this.boutonBras.addEventListener('click', () => {
      this.profil.brasFins = !this.profil.brasFins;
      this.changer();
    });
    element<HTMLButtonElement>('profil-importer').addEventListener('click', () => this.fichier.click());
    this.fichier.addEventListener('change', () => void this.importer());
    element<HTMLButtonElement>('profil-fermer').addEventListener('click', () => this.fermer());
    window.addEventListener('keydown', (e) => {
      if (this.ouvert && e.code === 'Escape') this.fermer();
    });
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  ouvrir(classe: ApparenceEntite, surFermeture: () => void): void {
    this.apparenceClasse = classe;
    this.surFermeture = surFermeture;
    this.pseudo.value = this.profil.pseudo;
    this.erreur.textContent = '';
    this.racine.classList.remove('cache');
    this.dessiner();
  }

  /** Visage du profil (menu) : celui de la skin choisie, ou celui de la classe. */
  visage(classe: ApparenceEntite): string | null {
    return portrait(this.textures, this.apparence(classe));
  }

  private fermer(): void {
    this.profil.pseudo = nettoyerPseudo(this.pseudo.value);
    enregistrerProfil(this.profil);
    this.racine.classList.add('cache');
    this.surFermeture?.();
  }

  private changer(): void {
    enregistrerProfil(this.profil);
    this.dessiner();
  }

  private apparence(classe: ApparenceEntite): ApparenceEntite {
    return apparenceJoueur(classe, this.profil);
  }

  private dessiner(): void {
    const classe = this.apparenceClasse;
    if (!classe) return;
    const choix: { id: ChoixSkin; nom: string; apparence: ApparenceEntite }[] = [
      { id: 'classe', nom: 'Classe', apparence: classe },
      ...SKINS_VANILLA.map((nom) => ({
        id: nom as ChoixSkin,
        nom: NOMS_SKINS[nom],
        apparence: { ...classe, modele: 'humanoide' as const, skin: cheminSkinVanilla(nom, false) },
      })),
    ];
    if (this.profil.skinImporte) choix.push({ id: 'importe', nom: 'Importée', apparence: { ...classe, modele: 'humanoide', skin: CHEMIN_SKIN_IMPORTE } });

    this.skins.replaceChildren(...choix.map(({ id, nom, apparence }) => {
      const bouton = document.createElement('button');
      bouton.className = 'option-menu profil-skin';
      bouton.classList.toggle('choisie', this.profil.skin === id);
      bouton.append(caseImage(portrait(this.textures, apparence), nom, 'portrait'), nom);
      bouton.addEventListener('click', () => {
        this.profil.skin = id;
        this.changer();
      });
      return bouton;
    }));

    // Le gabarit des bras ne concerne pas la skin de la classe (fixée par la classe)
    this.boutonBras.textContent = `Bras : ${this.profil.brasFins ? 'fins' : 'larges'}`;
    this.boutonBras.disabled = this.profil.skin === 'classe';

    const apparence = this.apparence(classe);
    const image = this.textures.obtenir(apparence.skin)?.image as HTMLImageElement | undefined;
    dessinerApercu(this.apercu, image ?? null, apparence.modele === 'humanoide_fin');
  }

  /** Lit un PNG de skin choisi par le joueur. */
  private async importer(): Promise<void> {
    const fichier = this.fichier.files?.[0];
    this.fichier.value = '';
    if (!fichier) return;
    const adresse = await new Promise<string | null>((resoudre) => {
      const lecteur = new FileReader();
      lecteur.onload = () => resoudre(typeof lecteur.result === 'string' ? lecteur.result : null);
      lecteur.onerror = () => resoudre(null);
      lecteur.readAsDataURL(fichier);
    });
    const image = adresse ? await chargerImageSkin(adresse) : null;
    if (!adresse || !image) {
      this.erreur.textContent = 'Skin refusée : il faut un PNG carré de 64×64 pixels (format du jeu).';
      return;
    }
    this.erreur.textContent = '';
    this.textures.enregistrer(CHEMIN_SKIN_IMPORTE, image);
    oublierPortraits();
    this.profil.skinImporte = adresse;
    this.profil.skin = 'importe';
    this.changer();
  }
}

/** Image d'une skin (data URL), ou null si elle n'a pas le format du jeu (carrée, 64 px ou multiple). */
export function chargerImageSkin(adresse: string): Promise<HTMLImageElement | null> {
  return new Promise((resoudre) => {
    const image = new Image();
    image.onload = () => {
      const valide = image.width === image.height && image.width >= TAILLE_SKIN && image.width % TAILLE_SKIN === 0;
      resoudre(valide ? image : null);
    };
    image.onerror = () => resoudre(null);
    image.src = adresse;
  });
}

/**
 * Aperçu de face (16 × 32 pixels de skin) : faces avant de la tête, du corps,
 * des bras et des jambes, puis leur seconde couche (chapeau, veste, manches…).
 */
function dessinerApercu(toile: HTMLCanvasElement, image: HTMLImageElement | null, brasFins: boolean): void {
  const ctx = toile.getContext('2d');
  if (!ctx) return;
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, toile.width, toile.height);
  if (!image) return;
  const e = image.width / TAILLE_SKIN;           // skins HD : pixels par pixel de patron
  const k = toile.width / 16;                     // pixels d'aperçu par pixel de patron
  const bras = brasFins ? 3 : 4;
  const morceau = (u: number, v: number, l: number, h: number, x: number, y: number) => {
    ctx.drawImage(image, u * e, v * e, l * e, h * e, x * k, y * k, l * k, h * k);
  };
  // [u, v, largeur, hauteur, x, y] de la couche de base, puis décalage de la seconde couche
  const parties: readonly [number, number, number, number, number, number, number, number][] = [
    [8, 8, 8, 8, 4, 0, 32, 0],                    // tête / chapeau
    [20, 20, 8, 12, 4, 8, 0, 16],                 // corps / veste
    [44, 20, bras, 12, 4 - bras, 8, 0, 16],       // bras droit (à gauche à l'écran) / manche
    [36, 52, bras, 12, 12, 8, 16, 0],             // bras gauche / manche
    [4, 20, 4, 12, 4, 20, 0, 16],                 // jambe droite / jambière
    [20, 52, 4, 12, 8, 20, -16, 0],               // jambe gauche / jambière
  ];
  for (const [u, v, l, h, x, y] of parties) morceau(u, v, l, h, x, y);
  for (const [u, v, l, h, x, y, du, dv] of parties) morceau(u + du, v + dv, l, h, x, y);
}
