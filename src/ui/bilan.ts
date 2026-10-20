/**
 * Bilan d'une partie (pause et écran de fin) : chiffres clés, ennemis vaincus
 * par type avec leur visage, et compteur de dégâts détaillé de chaque arme.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { rangMax } from '../combat/armeEquipee';
import type { ArmeEquipee } from '../combat/armeEquipee';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import type { EtatPartie } from '../etat';
import { formaterTemps } from '../utils/format';
import { formaterDegats, lignesDegats } from './compteurDegats';
import { portrait } from './portraits';

const NOMBRE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 });
const DECIMAL = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

/** Remplace le contenu de `conteneur` par le bilan de la partie. */
export function remplirBilan(
  conteneur: HTMLElement,
  etat: EtatPartie,
  armes: readonly ArmeEquipee[],
  textures: BibliothequeTextures,
): void {
  const s = etat.stats;
  const degatsTotaux = s.degatsParArme.reduce((total, d) => total + d, 0);

  const chiffres = document.createElement('div');
  chiffres.className = 'bilan-grille';
  chiffres.append(
    ligne('Personnage', etat.personnage.nom),
    ligne('Arène', etat.arene.nom),
    ligne('Temps survécu', formaterTemps(etat.tempsJeu)),
    ligne('Niveau atteint', String(etat.joueur.niveau)),
    ligne('Expérience totale', NOMBRE.format(s.xpTotale)),
    ligne('Ennemis vaincus', NOMBRE.format(etat.ennemisTues)),
    ligne('Boss vaincus', String(s.bossVaincus)),
    ligne('Dégâts infligés', NOMBRE.format(degatsTotaux)),
    ligne('Dégâts subis', NOMBRE.format(s.degatsSubis)),
    ligne('PV soignés', NOMBRE.format(s.pvSoignes)),
    ligne('Totems utilisés', String(etat.joueur.viesUtilisees)),
  );

  // Ennemis vaincus : visage et nombre, pour chaque type rencontré
  const tues = document.createElement('div');
  tues.className = 'bilan-tues';
  IDS_ENNEMIS.forEach((id, i) => {
    const nombre = s.tuesParType[i];
    if (nombre === 0) return;
    const def = ENNEMIS[id];
    const element = document.createElement('div');
    element.className = 'bilan-tue';
    element.title = def.nom;
    element.append(caseImage(portrait(textures, def), def.nom, 'portrait'), texte('span', `×${NOMBRE.format(nombre)}`));
    tues.append(element);
  });
  if (tues.childElementCount === 0) tues.append(texte('span', 'Aucun pour l\'instant.', 'bilan-vide'));

  // Armes : tableau du compteur de dégâts (rang, total, part, DPS, coups, critiques, plus gros coup, achevés)
  const listeArmes = document.createElement('div');
  listeArmes.className = 'bilan-degats';
  const { lignes } = lignesDegats(etat, armes);
  const maximum = Math.max(1, lignes[0]?.degats ?? 0);
  listeArmes.append(texte('span', ''), ...['Arme', 'Dégâts', 'Part', 'Par s', 'Coups', 'Critiques', 'Max', 'Achevés']
    .map((titre, i) => texte('span', titre, i === 0 ? 'entete' : 'entete nombre')));
  for (const l of lignes) {
    const def = l.arme.definition;
    const rang = l.arme.rang >= rangMax(def) ? 'MAX' : `rang ${l.arme.rang}`;
    const critiques = l.coups > 0 ? `${Math.round((l.critiques / l.coups) * 100)} %` : '—';
    listeArmes.append(
      caseImage(textures.url(def.icone), def.nom),
      texte('span', `${def.nom} (${rang})`, 'nom'),
      texte('span', formaterDegats(l.degats), 'nombre'),
      texte('span', `${Math.round(l.part * 100)} %`, 'nombre'),
      texte('span', DECIMAL.format(l.dps), 'nombre'),
      texte('span', NOMBRE.format(l.coups), 'nombre'),
      texte('span', critiques, 'nombre'),
      texte('span', NOMBRE.format(l.plusGrosCoup), 'nombre'),
      texte('span', NOMBRE.format(l.tues), 'nombre'),
    );
    const jauge = document.createElement('div');
    jauge.className = 'jauge';
    const remplissage = document.createElement('div');
    remplissage.style.width = `${(l.degats / maximum) * 100}%`;
    jauge.append(remplissage);
    listeArmes.append(texte('span', ''), jauge);
  }

  conteneur.replaceChildren(
    chiffres,
    texte('h3', 'Ennemis vaincus', 'titre-panneau'),
    tues,
    texte('h3', 'Dégâts par arme', 'titre-panneau'),
    listeArmes,
  );
}

function ligne(libelle: string, valeur: string): HTMLElement {
  const element = document.createElement('div');
  element.className = 'bilan-ligne';
  element.append(texte('span', libelle), texte('span', valeur, 'valeur'));
  return element;
}

function texte(balise: string, contenu: string, classe?: string): HTMLElement {
  const element = document.createElement(balise);
  element.textContent = contenu;
  if (classe) element.className = classe;
  return element;
}

/** Case d'inventaire contenant une image (ou l'initiale du nom si elle manque). */
export function caseImage(url: string | null, nom: string, variante?: string): HTMLElement {
  const conteneur = document.createElement('div');
  conteneur.className = variante ? `case-mc ${variante}` : 'case-mc';
  if (url) {
    const image = document.createElement('img');
    image.src = url;
    image.alt = '';
    conteneur.append(image);
  } else {
    conteneur.textContent = nom.charAt(0);
  }
  return conteneur;
}
