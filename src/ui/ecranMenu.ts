/**
 * Menu principal : choix du personnage (visage, statistiques, arme de
 * départ) et de l'arène, talents, options, puis « Jouer » (ou Entrée).
 * Les personnages et arènes pas encore débloqués affichent leur condition.
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
import type { Sauvegarde } from '../progression/sauvegarde';
import { libelleVersion, miseAJourDisponible, URL_TELECHARGEMENT } from '../version';
import { caseImage } from './bilan';
import { element } from './dom';
import type { EcranProfil } from './ecranProfil';
import { compteurEmeraudes, EcranTalents } from './ecranTalents';
import type { PanneauOptions } from './panneauOptions';
import { portrait } from './portraits';

const NOMBRE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

/** Pont exposé par l'application de bureau (electron/preload.cjs) ; absent dans un navigateur. */
const bureau = (window as { bureau?: { quitter(): void } }).bureau;

export class EcranMenu {
  private readonly racine = element('menu');
  private readonly listePersonnages = element('menu-personnages');
  private readonly listeArenes = element('menu-arenes');
  private readonly textures: BibliothequeTextures;
  private readonly options: PanneauOptions;
  private readonly sauvegarde: Sauvegarde;
  private readonly talents: EcranTalents;
  private readonly profil: EcranProfil;
  private personnage: IdPersonnage;
  private arene: DefinitionArene;
  private surJouer: ((choix: ChoixPartie) => void) | null = null;

  constructor(textures: BibliothequeTextures, choix: ChoixPartie, options: PanneauOptions, sauvegarde: Sauvegarde, profil: EcranProfil) {
    this.textures = textures;
    this.options = options;
    this.sauvegarde = sauvegarde;
    this.talents = new EcranTalents(textures, sauvegarde);
    this.profil = profil;
    // Un choix mémorisé encore verrouillé revient au premier disponible
    this.personnage = sauvegarde.verrou('personnage', choix.personnage)
      ? IDS_PERSONNAGES.find((id) => !sauvegarde.verrou('personnage', id)) ?? choix.personnage
      : choix.personnage;
    this.arene = sauvegarde.verrou('arene', choix.arene.id)
      ? ARENES.find((a) => !sauvegarde.verrou('arene', a.id)) ?? choix.arene
      : choix.arene;

    element('menu-accroche').textContent = ACCROCHES[Math.floor(Math.random() * ACCROCHES.length)];
    element('menu-version').textContent = libelleVersion();
    void miseAJourDisponible().then((disponible) => {
      if (!disponible) return;
      const lien = element<HTMLAnchorElement>('menu-mise-a-jour');
      lien.href = URL_TELECHARGEMENT;
      lien.classList.remove('cache');
    });
    this.listePersonnages.append(...IDS_PERSONNAGES.map((id) => this.creerOptionPersonnage(id)));
    this.listeArenes.append(...ARENES.map((arene) => this.creerOptionArene(arene)));
    element<HTMLButtonElement>('menu-jouer').addEventListener('click', () => this.jouer());
    element<HTMLButtonElement>('menu-options').addEventListener('click', () => this.options.ouvrir());
    element<HTMLButtonElement>('menu-talents').addEventListener('click', () => {
      this.talents.ouvrir(this.personnage, () => this.afficherEmeraudes());
    });
    this.afficherEmeraudes();
    const ouvrirProfil = () => this.profil.ouvrir(PERSONNAGES[this.personnage], () => this.afficherProfil());
    element<HTMLButtonElement>('menu-profil').addEventListener('click', ouvrirProfil);
    element<HTMLButtonElement>('menu-profil-bouton').addEventListener('click', ouvrirProfil);
    if (bureau) {
      const quitter = element<HTMLButtonElement>('menu-quitter');
      quitter.classList.remove('cache');
      quitter.addEventListener('click', () => bureau.quitter());
    }
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
    if (e.code !== 'Enter' || e.repeat || this.options.ouvert || this.talents.ouvert || this.profil.ouvert) return;
    e.preventDefault();
    this.jouer();
  };

  private afficherEmeraudes(): void {
    element('menu-emeraudes').replaceChildren(...compteurEmeraudes(this.textures, this.sauvegarde.emeraudes));
  }

  /** Pastille du profil (visage de la skin jouée et pseudo). */
  private afficherProfil(): void {
    const visage = this.profil.visage(PERSONNAGES[this.personnage]);
    const image = element<HTMLImageElement>('menu-profil-visage');
    if (visage) image.src = visage;
    else image.removeAttribute('src');
    element('menu-profil-pseudo').textContent = this.profil.profil.pseudo || 'Sans pseudo';
  }

  private marquerChoix(): void {
    this.afficherProfil();
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

    const verrou = this.sauvegarde.verrou('personnage', id);
    return this.creerOption(id, caseImage(portrait(this.textures, def), def.nom, 'grande portrait'), def.nom, def.description, [stats, ...armes], () => {
      this.personnage = id;
    }, verrou?.texte);
  }

  private creerOptionArene(arene: DefinitionArene): HTMLElement {
    const verrou = this.sauvegarde.verrou('arene', arene.id);
    return this.creerOption(arene.id, caseImage(this.textures.url(arene.icone), arene.nom, 'grande'), arene.nom, arene.description, [], () => {
      this.arene = arene;
    }, verrou?.texte);
  }

  private creerOption(
    id: string, icone: HTMLElement, nom: string, description: string, details: HTMLElement[], choisir: () => void, verrou?: string,
  ): HTMLElement {
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
    if (verrou) {
      // Verrouillé : la condition remplace les détails, et rien ne se choisit
      const condition = texte(`Verrouillé : ${verrou}`);
      condition.className = 'option-verrou';
      textes.replaceChildren(elementNom, condition);
      bouton.classList.add('verrouillee');
      return bouton;
    }
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
