/**
 * Apparence (depuis le menu) : skin (celle de la classe, une des skins par
 * défaut du jeu, ou un PNG importé) et gabarit des bras, avec le héros en 3D
 * qu'on fait tourner. Chaque changement est enregistré aussitôt.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { NOMS_SKINS, SKINS_VANILLA } from '../donnees/skins';
import type { ApparenceEntite } from '../donnees/types';
import { apparenceJoueur, CHEMIN_SKIN_IMPORTE, cheminSkinVanilla, enregistrerProfil, type ChoixSkin, type Profil } from '../profil';
import type { ApercuHeros } from './apercuHeros';
import { caseImage } from './bilan';
import { element } from './dom';
import { oublierPortraits, portrait } from './portraits';

/** Taille du patron de skin attendu (et de ses multiples HD). */
const TAILLE_SKIN = 64;

/** Classe affichée : son apparence et son nom. */
export type ClasseAffichee = ApparenceEntite & { readonly nom: string };

export class EcranProfil {
  private readonly racine = element('profil');
  private readonly skins = element('profil-skins');
  private readonly boutonBras = element<HTMLButtonElement>('profil-bras');
  private readonly fichier = element<HTMLInputElement>('profil-fichier');
  private readonly erreur = element('profil-erreur');
  private readonly textures: BibliothequeTextures;
  private readonly apercu: ApercuHeros | null;
  readonly profil: Profil;
  private classe: ClasseAffichee | null = null;
  private surFermeture: (() => void) | null = null;

  constructor(textures: BibliothequeTextures, profil: Profil, apercu: ApercuHeros | null) {
    this.textures = textures;
    this.profil = profil;
    this.apercu = apercu;
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

  ouvrir(classe: ClasseAffichee, surFermeture: () => void): void {
    this.classe = classe;
    this.surFermeture = surFermeture;
    this.erreur.textContent = '';
    this.racine.classList.remove('cache');
    this.apercu?.attacher(element('profil-apercu'));
    this.dessiner();
  }

  /** Nom de la skin choisie, pour le menu. */
  nomSkin(): string {
    const skin = this.profil.skin;
    if (skin === 'classe') return 'skin de la classe';
    if (skin === 'importe') return 'skin importée';
    return `skin ${NOMS_SKINS[skin]}`;
  }

  private fermer(): void {
    this.racine.classList.add('cache');
    this.surFermeture?.();
  }

  private changer(): void {
    enregistrerProfil(this.profil);
    this.dessiner();
  }

  private dessiner(): void {
    const classe = this.classe;
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

    element('profil-nom').textContent = this.profil.pseudo || 'Sans pseudo';
    element('profil-sous-titre').textContent = `${classe.nom} · ${this.nomSkin()}`;
    this.apercu?.afficher(apparenceJoueur(classe, this.profil));
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
