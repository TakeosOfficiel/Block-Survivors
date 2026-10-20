/**
 * Talents (depuis le menu) : pour chaque personnage, deux arbres côte à côte ;
 * un seul est actif pendant les parties. Clic sur un talent = acheter un rang
 * avec les émeraudes gagnées en jouant.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import { ARBRES, RECOMPENSES, SEUILS_PALIERS } from '../donnees/talents';
import type { DefinitionTalent } from '../donnees/types';
import type { Sauvegarde } from '../progression/sauvegarde';
import { caseImage } from './bilan';
import { element } from './dom';
import { portrait } from './portraits';

export class EcranTalents {
  private readonly racine = element('talents');
  private readonly onglets = element('talents-personnages');
  private readonly arbres = element('talents-arbres');
  private readonly compteur = element('talents-emeraudes');
  private readonly textures: BibliothequeTextures;
  private readonly sauvegarde: Sauvegarde;
  private personnage: IdPersonnage = IDS_PERSONNAGES[0];
  /** Appelé à la fermeture (le menu remet ses compteurs à jour). */
  private surFermeture: (() => void) | null = null;

  constructor(textures: BibliothequeTextures, sauvegarde: Sauvegarde) {
    this.textures = textures;
    this.sauvegarde = sauvegarde;
    element<HTMLButtonElement>('talents-fermer').addEventListener('click', () => this.fermer());
    window.addEventListener('keydown', (e) => {
      if (this.ouvert && e.code === 'Escape') this.fermer();
    });
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  ouvrir(personnage: IdPersonnage, surFermeture: () => void): void {
    this.personnage = personnage;
    this.surFermeture = surFermeture;
    this.racine.classList.remove('cache');
    this.dessiner();
  }

  private fermer(): void {
    this.racine.classList.add('cache');
    this.surFermeture?.();
  }

  /** Reconstruit tout l'écran (quelques dizaines d'éléments : simple et sûr). */
  private dessiner(): void {
    this.dessinerCompteur();
    this.onglets.replaceChildren(...IDS_PERSONNAGES.map((id) => {
      const def = PERSONNAGES[id];
      const bouton = document.createElement('button');
      bouton.className = 'option-menu talents-onglet';
      bouton.classList.toggle('choisie', id === this.personnage);
      const verrou = this.sauvegarde.verrou('personnage', id);
      bouton.disabled = verrou !== null;
      bouton.title = verrou ? `Verrouillé : ${verrou.texte}` : def.nom;
      bouton.append(caseImage(portrait(this.textures, def), def.nom, 'portrait'), def.nom);
      bouton.addEventListener('click', () => {
        this.personnage = id;
        this.dessiner();
      });
      return bouton;
    }));
    this.arbres.replaceChildren(...[0, 1].map((index) => this.dessinerArbre(index)));
  }

  private dessinerCompteur(): void {
    this.compteur.replaceChildren(...compteurEmeraudes(this.textures, this.sauvegarde.emeraudes));
  }

  private dessinerArbre(index: number): HTMLElement {
    const p = this.personnage;
    const def = ARBRES[p][index];
    const actif = this.sauvegarde.arbreActif(p) === index;
    const panneau = document.createElement('section');
    panneau.className = 'panneau-mc talents-arbre';
    panneau.classList.toggle('actif', actif);

    // En-tête : icône, nom, description, choix de l'arbre
    const entete = document.createElement('div');
    entete.className = 'talents-entete';
    const textes = document.createElement('div');
    textes.className = 'option-textes';
    textes.append(texte(def.nom, 'option-nom'), texte(def.description, 'option-description'));
    const choisir = document.createElement('button');
    choisir.className = 'bouton-mc talents-choisir';
    choisir.textContent = actif ? 'Arbre actif' : 'Choisir';
    choisir.disabled = actif;
    choisir.addEventListener('click', () => {
      this.sauvegarde.choisirArbre(p, index);
      this.dessiner();
    });
    entete.append(caseImage(this.textures.url(def.icone), def.nom, 'grande'), textes, choisir);

    // Paliers, du plus haut au plus bas (on « monte » dans l'arbre)
    const paliers = document.createElement('div');
    paliers.className = 'talents-paliers';
    for (let palier = SEUILS_PALIERS.length - 1; palier >= 0; palier--) {
      const ouvert = this.sauvegarde.palierOuvert(p, index, palier);
      const rangee = document.createElement('div');
      rangee.className = 'talents-rangee';
      rangee.classList.toggle('ferme', !ouvert);
      const etiquette = ouvert ? `Palier ${palier + 1}` : `Palier ${palier + 1} : ${SEUILS_PALIERS[palier]} rangs requis`;
      rangee.append(texte(etiquette, 'talents-palier'));
      const cases = document.createElement('div');
      cases.className = 'talents-cases';
      for (const talent of def.talents) if (talent.palier === palier) cases.append(this.dessinerTalent(index, talent));
      rangee.append(cases);
      paliers.append(rangee);
    }

    // Remise à zéro : toutes les émeraudes de l'arbre sont rendues
    const valeur = this.sauvegarde.valeur(p, index);
    const reinitialiser = document.createElement('button');
    reinitialiser.className = 'bouton-mc talents-reinitialiser';
    reinitialiser.textContent = valeur > 0 ? `Réinitialiser (+${valeur})` : 'Réinitialiser';
    reinitialiser.disabled = valeur === 0;
    reinitialiser.addEventListener('click', () => {
      this.sauvegarde.reinitialiser(p, index);
      this.dessiner();
    });

    panneau.append(entete, paliers, reinitialiser);
    return panneau;
  }

  private dessinerTalent(arbre: number, talent: DefinitionTalent): HTMLElement {
    const p = this.personnage;
    const rang = this.sauvegarde.rang(p, arbre, talent);
    const max = talent.couts.length;
    const refus = this.sauvegarde.refusAchat(p, arbre, talent);
    const bouton = document.createElement('button');
    bouton.className = 'talent';
    bouton.classList.toggle('max', refus === 'max');
    bouton.classList.toggle('possede', rang > 0);
    bouton.disabled = refus === 'palier' || refus === 'max';

    const icone = caseImage(this.textures.url(talent.icone), talent.nom);
    const pastille = texte(`${rang}/${max}`, 'talent-rang');
    icone.append(pastille);
    const textes = document.createElement('div');
    textes.className = 'talent-textes';
    textes.append(texte(talent.nom, 'talent-nom'), texte(talent.effet, 'talent-effet'));
    const prix = document.createElement('span');
    prix.className = 'talent-prix';
    if (refus === 'max') {
      prix.textContent = 'Maximum';
    } else {
      prix.append(...compteurEmeraudes(this.textures, talent.couts[rang]));
      prix.classList.toggle('trop-cher', refus === 'prix');
    }
    textes.append(prix);
    bouton.append(icone, textes);
    bouton.addEventListener('click', () => {
      if (this.sauvegarde.acheter(p, arbre, talent)) this.dessiner();
    });
    return bouton;
  }
}

/** Icône d'émeraude suivie d'un nombre. */
export function compteurEmeraudes(textures: BibliothequeTextures, nombre: number): Node[] {
  const url = textures.url(RECOMPENSES.icone);
  const noeuds: Node[] = [];
  if (url) {
    const image = document.createElement('img');
    image.src = url;
    image.alt = 'émeraudes';
    image.className = 'icone-texte';
    noeuds.push(image);
  }
  noeuds.push(document.createTextNode(new Intl.NumberFormat('fr-FR').format(nombre)));
  return noeuds;
}

function texte(contenu: string, classe: string): HTMLElement {
  const span = document.createElement('span');
  span.className = classe;
  span.textContent = contenu;
  return span;
}
