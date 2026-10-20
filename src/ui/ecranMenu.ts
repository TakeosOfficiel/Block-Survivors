/**
 * Menu principal : choix du personnage (visage, statistiques, arme de
 * départ) et de l'arène, options, puis « Jouer » (ou Entrée).
 */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ACCROCHES } from '../donnees/accroches';
import { ARENES } from '../donnees/arenes';
import { ARMES } from '../donnees/armes';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import type { DefinitionArene } from '../donnees/types';
import type { ChoixPartie } from '../lancement';
import { decrireBonus } from '../progression/ameliorations';
import { caseImage } from './bilan';
import { element } from './dom';
import type { PanneauOptions } from './panneauOptions';
import { portrait } from './portraits';

const NOMBRE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

export class EcranMenu {
  private readonly racine = element('menu');
  private readonly listePersonnages = element('menu-personnages');
  private readonly listeArenes = element('menu-arenes');
  private readonly textures: BibliothequeTextures;
  private readonly options: PanneauOptions;
  private personnage: IdPersonnage;
  private arene: DefinitionArene;
  private surJouer: ((choix: ChoixPartie) => void) | null = null;

  constructor(textures: BibliothequeTextures, choix: ChoixPartie, options: PanneauOptions) {
    this.textures = textures;
    this.options = options;
    this.personnage = choix.personnage;
    this.arene = choix.arene;

    element('menu-accroche').textContent = ACCROCHES[Math.floor(Math.random() * ACCROCHES.length)];
    this.listePersonnages.append(...IDS_PERSONNAGES.map((id) => this.creerOptionPersonnage(id)));
    this.listeArenes.append(...ARENES.map((arene) => this.creerOptionArene(arene)));
    element<HTMLButtonElement>('menu-jouer').addEventListener('click', () => this.jouer());
    element<HTMLButtonElement>('menu-options').addEventListener('click', () => this.options.ouvrir());
    window.addEventListener('keydown', this.surTouche);
    this.marquerChoix();
  }

  /** Affiche le menu et attend que le joueur lance une partie. */
  attendreChoix(): Promise<ChoixPartie> {
    this.racine.classList.remove('cache');
    return new Promise((resoudre) => {
      this.surJouer = resoudre;
    });
  }

  private jouer(): void {
    const rappel = this.surJouer;
    if (!rappel) return;
    this.surJouer = null;
    window.removeEventListener('keydown', this.surTouche);
    this.racine.classList.add('cache');
    rappel({ personnage: this.personnage, arene: this.arene });
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Enter' || e.repeat || this.options.ouvert) return;
    e.preventDefault();
    this.jouer();
  };

  private marquerChoix(): void {
    for (const option of this.listePersonnages.children) {
      option.classList.toggle('choisie', (option as HTMLElement).dataset['id'] === this.personnage);
    }
    for (const option of this.listeArenes.children) {
      option.classList.toggle('choisie', (option as HTMLElement).dataset['id'] === this.arene.id);
    }
  }

  private creerOptionPersonnage(id: IdPersonnage): HTMLElement {
    const def = PERSONNAGES[id];
    const stats = document.createElement('div');
    stats.className = 'option-stats';
    const coeur = this.textures.url(CONFIG.hud.coeurs.plein);
    const sante = document.createElement('span');
    if (coeur) {
      const image = document.createElement('img');
      image.src = coeur;
      image.alt = '';
      image.className = 'icone-texte';
      sante.append(image);
    }
    sante.append(`${NOMBRE.format(def.pv / 2)} cœurs`);
    stats.append(sante, texte(`Vitesse : ${NOMBRE.format(def.vitesse)}`));
    for (const ligne of decrireBonus(def.bonus)) stats.append(texte(ligne));

    const armes = def.armesDepart.map((idArme) => {
      const arme = ARMES[idArme];
      const ligne = document.createElement('div');
      ligne.className = 'option-arme';
      const url = this.textures.url(arme.icone);
      if (url) {
        const image = document.createElement('img');
        image.src = url;
        image.alt = '';
        ligne.append(image);
      }
      ligne.append(`Arme : ${arme.nom}`);
      return ligne;
    });

    return this.creerOption(id, caseImage(portrait(this.textures, def), def.nom, 'grande portrait'), def.nom, def.description, [stats, ...armes], () => {
      this.personnage = id;
    });
  }

  private creerOptionArene(arene: DefinitionArene): HTMLElement {
    return this.creerOption(arene.id, caseImage(this.textures.url(arene.icone), arene.nom, 'grande'), arene.nom, arene.description, [], () => {
      this.arene = arene;
    });
  }

  private creerOption(id: string, icone: HTMLElement, nom: string, description: string, details: HTMLElement[], choisir: () => void): HTMLElement {
    const bouton = document.createElement('button');
    bouton.className = 'option-menu';
    bouton.dataset['id'] = id;
    const textes = document.createElement('div');
    textes.className = 'option-textes';
    const elementNom = texte(nom);
    elementNom.className = 'option-nom';
    const elementDescription = texte(description);
    elementDescription.className = 'option-description';
    textes.append(elementNom, elementDescription, ...details);
    bouton.append(icone, textes);
    bouton.addEventListener('click', () => {
      choisir();
      this.marquerChoix();
    });
    // Double-clic : choisir et jouer
    bouton.addEventListener('dblclick', () => this.jouer());
    return bouton;
  }
}

function texte(contenu: string): HTMLElement {
  const element = document.createElement('span');
  element.textContent = contenu;
  return element;
}
