/**
 * Apparence (depuis le menu) : skin (celle de la classe, le personnage de
 * l'éditeur, une des skins par défaut du jeu, ou un PNG importé) et gabarit
 * des bras, avec le héros en 3D qu'on fait tourner. Le panneau « Traits »
 * compose le skin de l'éditeur. Chaque changement est enregistré aussitôt.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { dessinerSkinPerso } from '../assets/skinPerso';
import { NOMS_SKINS, SKINS_VANILLA } from '../donnees/skins';
import { IDS_TRAITS, TRAITS, type IdTrait } from '../donnees/traits';
import type { ApparenceEntite } from '../donnees/types';
import {
  apparenceJoueur, CHEMIN_SKIN_IMPORTE, CHEMIN_SKIN_PERSO, cheminSkinVanilla, enregistrerProfil, type ChoixSkin, type Profil,
} from '../profil';
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
  private readonly traits = element('profil-traits');
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
      this.redessinerPerso();  // les bras du skin de l'éditeur changent de largeur
      this.changer();
    });
    element<HTMLButtonElement>('profil-aleatoire').addEventListener('click', () => {
      for (const id of IDS_TRAITS) this.profil.traits[id] = Math.floor(Math.random() * TRAITS[id].choix.length);
      this.profil.skin = 'perso';
      this.redessinerPerso();
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
    if (skin === 'perso') return 'personnage personnalisé';
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
      { id: 'perso', nom: 'Perso', apparence: { ...classe, modele: this.profil.brasFins ? 'humanoide_fin' : 'humanoide', skin: CHEMIN_SKIN_PERSO } },
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

    this.dessinerTraits();
    element('profil-nom').textContent = this.profil.pseudo || 'Sans pseudo';
    element('profil-sous-titre').textContent = `${classe.nom} · ${this.nomSkin()}`;
    this.apercu?.afficher(apparenceJoueur(classe, this.profil));
  }

  /** Une ligne par trait : intitulé, ◀ valeur ▶. Changer un trait choisit le skin de l'éditeur. */
  private dessinerTraits(): void {
    this.traits.replaceChildren(...IDS_TRAITS.map((id) => {
      const trait = TRAITS[id];
      const index = this.profil.traits[id];
      const ligne = document.createElement('div');
      ligne.className = 'profil-trait';
      const intitule = document.createElement('div');
      intitule.className = 'profil-trait-nom';
      intitule.textContent = trait.nom;
      const valeur = document.createElement('div');
      valeur.className = 'profil-trait-valeur';
      const nom = document.createElement('span');
      nom.textContent = `${trait.choix[index].nom} (${index + 1}/${trait.choix.length})`;
      const echantillon = 'couleur' in trait.choix[index] ? (trait.choix[index] as { couleur: number }).couleur : null;
      if (echantillon !== null) {
        const pastille = document.createElement('span');
        pastille.className = 'profil-trait-couleur';
        pastille.style.background = `#${echantillon.toString(16).padStart(6, '0')}`;
        nom.prepend(pastille);
      }
      valeur.append(this.fleche('◀', id, -1), nom, this.fleche('▶', id, 1));
      ligne.append(intitule, valeur);
      return ligne;
    }));
  }

  private fleche(texte: string, id: IdTrait, sens: number): HTMLButtonElement {
    const bouton = document.createElement('button');
    bouton.className = 'bouton-mc profil-fleche';
    bouton.textContent = texte;
    bouton.addEventListener('click', () => {
      const n = TRAITS[id].choix.length;
      this.profil.traits[id] = (this.profil.traits[id] + sens + n) % n;
      this.profil.skin = 'perso';
      this.redessinerPerso();
      this.changer();
    });
    return bouton;
  }

  /** Redessine le skin de l'éditeur et l'enregistre comme texture (aperçu, visages, partie). */
  private redessinerPerso(): void {
    this.textures.enregistrer(CHEMIN_SKIN_PERSO, dessinerSkinPerso(this.profil.traits, this.profil.brasFins));
    oublierPortraits();
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
