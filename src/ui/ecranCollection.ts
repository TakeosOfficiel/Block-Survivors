/**
 * Collection (depuis le menu) : un onglet par monde, avec sa progression, son
 * armure (pièces gagnées ou grisées, bonus de pièce et d'ensemble) et ses
 * bagues et amulettes (découvertes, ou « ??? » tant qu'on ne les a pas
 * trouvées) ; puis les objets de tous les mondes.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { ARMURES, MONDES, NOMS_MONDES, NOMS_PIECES, PIECES_ARMURE } from '../donnees/armures';
import { IDS_OBJETS_EQUIPEMENT, OBJETS_EQUIPEMENT, RARETES_OBJETS, type IdObjetEquipement } from '../donnees/equipement';
import type { IdMonde } from '../donnees/types';
import type { Collection } from '../progression/collection';
import { cheminIconeObjet } from '../progression/equipement';
import { caseImage } from './bilan';
import { element } from './dom';
import { iconePiece } from './iconesObjets';

export class EcranCollection {
  private readonly racine = element('collection');
  private readonly onglets = element('collection-onglets');
  private readonly contenu = element('collection-contenu');
  private readonly textures: BibliothequeTextures;
  private readonly collection: Collection;
  private monde: IdMonde = 'prairie';
  private surFermeture: (() => void) | null = null;

  constructor(textures: BibliothequeTextures, collection: Collection) {
    this.textures = textures;
    this.collection = collection;
    element<HTMLButtonElement>('collection-fermer').addEventListener('click', () => this.fermer());
    window.addEventListener('keydown', (e) => {
      if (this.ouvert && e.code === 'Escape') this.fermer();
    });
  }

  get ouvert(): boolean {
    return !this.racine.classList.contains('cache');
  }

  /** `monde` : onglet ouvert d'abord (celui de l'arène choisie). */
  ouvrir(monde: IdMonde, surFermeture: () => void): void {
    this.monde = monde;
    this.surFermeture = surFermeture;
    this.racine.classList.remove('cache');
    this.dessiner();
  }

  private fermer(): void {
    this.racine.classList.add('cache');
    this.surFermeture?.();
  }

  private dessiner(): void {
    this.onglets.replaceChildren(...MONDES.map((monde) => {
      const bouton = document.createElement('button');
      bouton.className = 'option-menu talents-onglet collection-onglet';
      bouton.classList.toggle('choisie', monde === this.monde);
      const avancee = this.collection.avancee(monde);
      bouton.textContent = `${NOMS_MONDES[monde]} · ${Math.round(avancee.part * 100)} %`;
      bouton.addEventListener('click', () => {
        this.monde = monde;
        this.dessiner();
      });
      return bouton;
    }));

    const monde = this.monde;
    const armure = ARMURES[monde];
    const avancee = this.collection.avancee(monde);

    // Progression du monde
    const entete = document.createElement('div');
    entete.className = 'collection-entete';
    const barre = document.createElement('div');
    barre.className = 'collection-barre';
    const rempli = document.createElement('div');
    rempli.style.width = `${Math.round(avancee.part * 100)}%`;
    barre.append(rempli);
    entete.append(
      texte('div', `${NOMS_MONDES[monde]} : ${avancee.pieces} / ${PIECES_ARMURE.length} pièces · ${avancee.objets} / ${avancee.totalObjets} objets`, 'collection-titre'),
      barre,
    );

    // Armure du monde
    const panneauArmure = document.createElement('div');
    panneauArmure.className = 'panneau-mc collection-panneau';
    panneauArmure.append(texte('h3', armure.nom, 'titre-panneau'));
    for (const piece of PIECES_ARMURE) {
      const def = armure.pieces[piece];
      const possedee = this.collection.possede(monde, piece);
      panneauArmure.append(this.tuile(
        this.textures.url(iconePiece(monde, piece)),
        possedee ? def.nom : `${NOMS_PIECES[piece]} : à gagner`,
        possedee ? def.description : `Gagnez une partie dans « ${NOMS_MONDES[monde]} ».`,
        possedee,
        null,
      ));
    }
    const complet = PIECES_ARMURE.every((p) => this.collection.possede(monde, p));
    panneauArmure.append(texte('div', armure.ensemble.description, complet ? 'collection-ensemble obtenu' : 'collection-ensemble'));

    // Objets du monde, puis de tous les mondes
    const panneauObjets = document.createElement('div');
    panneauObjets.className = 'panneau-mc collection-panneau';
    panneauObjets.append(texte('h3', 'Bagues et amulettes du monde', 'titre-panneau'));
    for (const id of IDS_OBJETS_EQUIPEMENT.filter((i) => OBJETS_EQUIPEMENT[i].monde === monde)) panneauObjets.append(this.tuileObjet(id));
    panneauObjets.append(texte('h3', 'De tous les mondes', 'titre-panneau'));
    for (const id of IDS_OBJETS_EQUIPEMENT.filter((i) => OBJETS_EQUIPEMENT[i].monde === null)) panneauObjets.append(this.tuileObjet(id));

    const corps = document.createElement('div');
    corps.className = 'collection-corps';
    corps.append(panneauArmure, panneauObjets);
    this.contenu.replaceChildren(entete, corps);
  }

  private tuileObjet(id: IdObjetEquipement): HTMLElement {
    const def = OBJETS_EQUIPEMENT[id];
    const trouve = this.collection.decouvert(id);
    return this.tuile(
      this.textures.url(cheminIconeObjet(id)),
      trouve ? def.nom : '???',
      trouve ? def.description(1) : `${RARETES_OBJETS[def.rarete].nom} · à trouver dans un coffre`,
      trouve,
      def.rarete,
    );
  }

  private tuile(icone: string | null, nom: string, description: string, obtenu: boolean, rarete: string | null): HTMLElement {
    const tuile = document.createElement('div');
    tuile.className = `collection-tuile${obtenu ? '' : ' verrouillee'}`;
    const textes = document.createElement('div');
    textes.append(
      texte('div', nom, rarete ? `collection-nom rarete-objet-${rarete}` : 'collection-nom'),
      texte('div', description, 'collection-description'),
    );
    tuile.append(caseImage(icone, nom), textes);
    return tuile;
  }
}

function texte(balise: string, contenu: string, classe: string): HTMLElement {
  const e = document.createElement(balise);
  e.className = classe;
  e.textContent = contenu;
  return e;
}
