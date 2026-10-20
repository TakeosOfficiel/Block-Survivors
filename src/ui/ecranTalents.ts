/**
 * Talents (depuis le menu) : pour chaque personnage, deux arbres en graphe ;
 * un seul est actif pendant les parties. Un clic sur un talent ouvre sa fiche :
 * achat d'un rang avec les émeraudes gagnées en jouant, choix de l'option
 * d'un talent à choix, talent final actif, arme de départ active.
 *
 * Schéma : les liens de redstone s'allument quand le talent du dessus est
 * complet ; les lignes pointillées sont les paliers (rangs à acheter au-dessus).
 */
import type { BibliothequeTextures } from '../assets/textures';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import { ARBRES, coutsTalent, MECANIQUES_PRETES, PALIERS, RECOMPENSES } from '../donnees/talents';
import type { BrancheArbre, NoeudTalent, TypeNoeud } from '../donnees/types';
import type { Sauvegarde } from '../progression/sauvegarde';
import { caseImage } from './bilan';
import { element } from './dom';
import { portrait } from './portraits';

/** Placement des talents sur le schéma (px). */
const SCHEMA = { ecartX: 66, ecartY: 68, marge: 18, taille: 46, largeur: 540, hauteur: 486 } as const;

const COULEURS_BRANCHES: Record<BrancheArbre, string> = {
  tronc: '#b3a397', gauche: '#6fb2ff', centre: '#ffa24a', droite: '#67d993', libre: '#c99bff',
};

const TYPES: Record<TypeNoeud, string> = {
  bonus: 'Bonus', capacite: 'Capacité', choix: 'Talent à choix', final: 'Talent final',
};

const SVG = 'http://www.w3.org/2000/svg';

export class EcranTalents {
  private readonly racine = element('talents');
  private readonly onglets = element('talents-personnages');
  private readonly contenu = element('talents-arbres');
  private readonly compteur = element('talents-emeraudes');
  private readonly textures: BibliothequeTextures;
  private readonly sauvegarde: Sauvegarde;
  private personnage: IdPersonnage = IDS_PERSONNAGES[0];
  /** Arbre affiché (0 ou 1) et talent dont la fiche est ouverte. */
  private arbre = 0;
  private choisi: string | null = null;
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
    this.surFermeture = surFermeture;
    this.racine.classList.remove('cache');
    this.changerPersonnage(personnage);
  }

  private fermer(): void {
    this.racine.classList.add('cache');
    this.surFermeture?.();
  }

  private changerPersonnage(personnage: IdPersonnage): void {
    this.personnage = personnage;
    this.changerArbre(this.sauvegarde.arbreActif(personnage));
  }

  private changerArbre(arbre: number): void {
    this.arbre = arbre;
    this.choisi = ARBRES[this.personnage][arbre].noeuds[0].id;
    this.dessiner();
  }

  /** Reconstruit tout l'écran (une centaine d'éléments : simple et sûr). */
  private dessiner(): void {
    this.compteur.replaceChildren(...compteurEmeraudes(this.textures, this.sauvegarde.emeraudes));
    this.onglets.replaceChildren(...IDS_PERSONNAGES.map((id) => {
      const def = PERSONNAGES[id];
      const bouton = document.createElement('button');
      bouton.className = 'option-menu talents-onglet';
      bouton.classList.toggle('choisie', id === this.personnage);
      const verrou = this.sauvegarde.verrou('personnage', id);
      bouton.disabled = verrou !== null;
      bouton.title = verrou ? `Verrouillé : ${verrou.texte}` : def.nom;
      bouton.append(caseImage(portrait(this.textures, def), def.nom, 'portrait'), def.nom);
      bouton.addEventListener('click', () => this.changerPersonnage(id));
      return bouton;
    }));

    const volets = document.createElement('div');
    volets.className = 'talents-volets';
    volets.append(this.dessinerSchema(), this.dessinerFiche());
    this.contenu.replaceChildren(this.dessinerEntete(), volets);
  }

  /** Les deux arbres en onglets, l'arbre actif en partie, la remise à zéro. */
  private dessinerEntete(): HTMLElement {
    const p = this.personnage;
    const actif = this.sauvegarde.arbreActif(p);
    const entete = document.createElement('div');
    entete.className = 'talents-entete';
    const onglets = document.createElement('div');
    onglets.className = 'talents-arbres-onglets';
    ARBRES[p].forEach((def, index) => {
      const bouton = document.createElement('button');
      bouton.className = 'option-menu talents-onglet-arbre';
      bouton.classList.toggle('choisie', index === this.arbre);
      bouton.title = def.description;
      const textes = document.createElement('div');
      textes.className = 'option-textes';
      const rangs = this.sauvegarde.points(p, index);
      textes.append(
        texte(index === actif ? `${def.nom} ★` : def.nom, 'option-nom'),
        texte(`${def.description} · ${rangs} rang${rangs > 1 ? 's' : ''}`, 'option-description'),
      );
      bouton.append(caseImage(this.textures.url(def.icone), def.nom), textes);
      bouton.addEventListener('click', () => this.changerArbre(index));
      onglets.append(bouton);
    });

    const actions = document.createElement('div');
    actions.className = 'talents-actions';
    const utiliser = document.createElement('button');
    utiliser.className = 'bouton-mc';
    utiliser.textContent = this.arbre === actif ? '★ Utilisé en partie' : 'Utiliser en partie';
    utiliser.disabled = this.arbre === actif;
    utiliser.addEventListener('click', () => {
      this.sauvegarde.choisirArbre(p, this.arbre);
      this.dessiner();
    });
    const valeur = this.sauvegarde.valeur(p, this.arbre);
    const reinitialiser = document.createElement('button');
    reinitialiser.className = 'bouton-mc';
    reinitialiser.textContent = valeur > 0 ? `Réinitialiser (+${valeur})` : 'Réinitialiser';
    reinitialiser.title = 'Rend toutes les émeraudes dépensées dans cet arbre';
    reinitialiser.disabled = valeur === 0;
    reinitialiser.addEventListener('click', () => {
      this.sauvegarde.reinitialiser(p, this.arbre);
      this.dessiner();
    });
    actions.append(utiliser, reinitialiser);
    entete.append(onglets, actions);
    return entete;
  }

  private dessinerSchema(): HTMLElement {
    const p = this.personnage;
    const a = this.arbre;
    const def = ARBRES[p][a];
    const parId = new Map(def.noeuds.map((n) => [n.id, n]));
    const schema = document.createElement('div');
    schema.className = 'talents-schema';
    schema.style.width = `${SCHEMA.largeur}px`;
    schema.style.height = `${SCHEMA.hauteur}px`;

    // Liens de redstone : allumés quand le talent du dessus est complet
    const svg = document.createElementNS(SVG, 'svg');
    svg.setAttribute('width', String(SCHEMA.largeur));
    svg.setAttribute('height', String(SCHEMA.hauteur));
    const centre = (n: NoeudTalent) => [
      SCHEMA.marge + n.colonne * SCHEMA.ecartX + SCHEMA.taille / 2,
      SCHEMA.marge + n.ligne * SCHEMA.ecartY + SCHEMA.taille / 2,
    ];
    for (const n of def.noeuds) {
      for (const id of n.prerequis ?? []) {
        const parent = parId.get(id);
        if (!parent) continue;
        const [x1, y1] = centre(parent);
        const [x2, y2] = centre(n);
        const ligne = document.createElementNS(SVG, 'line');
        ligne.setAttribute('x1', String(x1));
        ligne.setAttribute('y1', String(y1));
        ligne.setAttribute('x2', String(x2));
        ligne.setAttribute('y2', String(y2));
        ligne.setAttribute('class', this.sauvegarde.complet(p, a, parent) ? 'lien allume' : 'lien');
        svg.append(ligne);
      }
    }
    schema.append(svg);

    // Paliers : rangs achetés au-dessus / rangs requis
    for (const palier of PALIERS) {
      const atteint = this.sauvegarde.palierAtteint(p, a, palier.ligne);
      const porte = document.createElement('div');
      porte.className = 'talents-porte';
      porte.classList.toggle('ouverte', atteint);
      porte.style.top = `${SCHEMA.marge + palier.ligne * SCHEMA.ecartY - 15}px`;
      const achetes = Math.min(palier.rangs, this.sauvegarde.points(p, a, palier.ligne));
      porte.append(texte(atteint ? `${palier.rangs} rangs ✔` : `${achetes} / ${palier.rangs} rangs`, 'talents-porte-texte'));
      schema.append(porte);
    }

    for (const n of def.noeuds) schema.append(this.dessinerNoeud(n));
    return schema;
  }

  private dessinerNoeud(n: NoeudTalent): HTMLElement {
    const p = this.personnage;
    const a = this.arbre;
    const rang = this.sauvegarde.rang(p, a, n);
    const bouton = document.createElement('button');
    bouton.type = 'button';
    bouton.className = `talent-noeud ${n.type}`;
    bouton.classList.toggle('libre', n.libre === true);
    bouton.classList.toggle('ferme', rang === 0 && !this.sauvegarde.ouvert(p, a, n));
    bouton.classList.toggle('possede', rang > 0);
    bouton.classList.toggle('complet', rang >= n.rangMax);
    bouton.classList.toggle('inactif', rang > 0 && !this.sauvegarde.actif(p, a, n));
    bouton.classList.toggle('choisi', n.id === this.choisi);
    bouton.style.left = `${SCHEMA.marge + n.colonne * SCHEMA.ecartX}px`;
    bouton.style.top = `${SCHEMA.marge + n.ligne * SCHEMA.ecartY}px`;
    bouton.setAttribute('aria-label', `${n.nom} (${rang}/${n.rangMax})`);

    const caseNoeud = document.createElement('span');
    caseNoeud.className = 'talent-case';
    if (n.options) {
      // Deux moitiés en diagonale ; l'option choisie ressort une fois le talent acheté
      const option = this.sauvegarde.option(p, a, n);
      n.options.forEach((o, i) => {
        const moitie = document.createElement('span');
        moitie.className = `talent-moitie ${i === 0 ? 'a' : 'b'}`;
        moitie.classList.toggle('eteinte', rang > 0 && i !== option);
        moitie.append(image(this.textures.url(o.icone)));
        caseNoeud.append(moitie);
      });
    } else {
      caseNoeud.append(image(this.textures.url(n.icone)));
    }
    bouton.append(caseNoeud);
    if (n.rangMax > 1 || rang > 0) bouton.append(texte(n.rangMax > 1 ? `${rang}/${n.rangMax}` : '✔', 'talent-noeud-rang'));
    if (n.armeDepart) bouton.append(texte('+', 'talent-noeud-arme'));
    const pied = texte('', 'talent-noeud-pied');
    pied.style.background = COULEURS_BRANCHES[n.branche];
    bouton.append(pied);
    bouton.addEventListener('click', () => {
      this.choisi = n.id;
      this.dessiner();
    });
    return bouton;
  }

  /** Fiche du talent choisi : effet, prérequis, coût, achat et réglages. */
  private dessinerFiche(): HTMLElement {
    const p = this.personnage;
    const a = this.arbre;
    const def = ARBRES[p][a];
    const n = def.noeuds.find((x) => x.id === this.choisi) ?? def.noeuds[0];
    const rang = this.sauvegarde.rang(p, a, n);
    const fiche = document.createElement('div');
    fiche.className = 'talents-fiche';

    const titre = document.createElement('div');
    titre.className = 'talents-fiche-titre';
    titre.append(texte(n.nom, 'talents-fiche-nom'));
    const meta = texte(`${TYPES[n.type]} · rang ${rang}/${n.rangMax} · `, 'talents-fiche-meta');
    const branche = texte(def.branches[n.branche] ?? '', 'talents-fiche-branche');
    branche.style.color = COULEURS_BRANCHES[n.branche];
    meta.append(branche);
    titre.append(meta);
    fiche.append(titre);

    // Effet ; pour un talent à choix, les deux options (choix gratuit une fois acheté)
    if (n.options) {
      const option = this.sauvegarde.option(p, a, n);
      n.options.forEach((o, i) => {
        const carte = document.createElement('button');
        carte.type = 'button';
        carte.className = 'talents-option';
        carte.classList.toggle('choisie', rang > 0 && i === option);
        carte.disabled = rang === 0;
        carte.append(image(this.textures.url(o.icone)));
        const textes = document.createElement('div');
        textes.append(texte(o.nom, 'talents-option-nom'), texte(o.effet, 'talents-fiche-effet'));
        if (o.mecanique && !MECANIQUES_PRETES.has(o.mecanique)) textes.append(texte('Effet en préparation', 'talents-preparation'));
        carte.append(textes);
        carte.addEventListener('click', () => {
          this.sauvegarde.choisirOption(p, a, n, i);
          this.dessiner();
        });
        fiche.append(carte);
      });
    } else {
      fiche.append(texte(n.effet, 'talents-fiche-effet'));
      if (n.mecanique && !MECANIQUES_PRETES.has(n.mecanique)) {
        fiche.append(texte('Effet en préparation : il arrive dans une prochaine version.', 'talents-preparation'));
      }
    }

    // Conditions et prix
    const infos = document.createElement('dl');
    infos.className = 'talents-fiche-infos';
    const ajouter = (terme: string, valeur: string, classe = '') => {
      const dt = document.createElement('dt');
      dt.textContent = terme;
      const dd = document.createElement('dd');
      dd.textContent = valeur;
      if (classe) dd.className = classe;
      infos.append(dt, dd);
    };
    ajouter('Prérequis', this.prerequis(n), this.sauvegarde.ouvert(p, a, n) ? '' : 'refuse');
    const couts = coutsTalent(n);
    ajouter('Coût', `${couts.join(' / ')} émeraudes`);
    if (n.type === 'final') ajouter('Activation', 'Un seul talent final actif par arbre, changeable avant chaque partie.');
    if (n.armeDepart) ajouter('Activation', 'Une seule arme de départ en plus par arbre ; désactivable sans perdre l’achat.');
    if (n.options) ajouter('Activation', 'L’achat débloque les deux options ; une seule joue à la fois.');
    fiche.append(infos);

    // Boutons : achat, puis réglages d'un talent acheté
    const boutons = document.createElement('div');
    boutons.className = 'talents-fiche-boutons';
    const refus = this.sauvegarde.refusAchat(p, a, n);
    const acheter = document.createElement('button');
    acheter.className = 'bouton-mc';
    if (refus === 'max') {
      acheter.textContent = 'Complet';
    } else {
      acheter.append(rang === 0 ? 'Acheter ' : 'Rang suivant ', ...compteurEmeraudes(this.textures, couts[rang]));
      acheter.title = refus === 'verrou' ? 'Prérequis non remplis' : refus === 'prix' ? 'Pas assez d’émeraudes' : '';
    }
    acheter.disabled = refus !== null;
    acheter.addEventListener('click', () => {
      if (this.sauvegarde.acheter(p, a, n)) this.dessiner();
    });
    boutons.append(acheter);

    if (rang > 0 && n.type === 'final') {
      const actif = this.sauvegarde.actif(p, a, n);
      const activer = document.createElement('button');
      activer.className = 'bouton-mc';
      activer.textContent = actif ? 'Talent final actif' : 'Activer ce talent final';
      activer.disabled = actif;
      activer.addEventListener('click', () => {
        this.sauvegarde.choisirFinal(p, a, n);
        this.dessiner();
      });
      boutons.append(activer);
    }
    if (rang > 0 && n.armeDepart) {
      const actif = this.sauvegarde.actif(p, a, n);
      const arme = document.createElement('button');
      arme.className = 'bouton-mc';
      arme.textContent = actif ? 'Désactiver cette arme de départ' : 'Activer cette arme de départ';
      arme.addEventListener('click', () => {
        this.sauvegarde.choisirArmeDepart(p, a, actif ? null : n);
        this.dessiner();
      });
      boutons.append(arme);
    }
    fiche.append(boutons);
    if (rang > 0 && !this.sauvegarde.actif(p, a, n)) fiche.append(texte('Acheté mais inactif dans cet arbre.', 'talents-preparation'));
    return fiche;
  }

  /** Texte des conditions d'achat du talent. */
  private prerequis(n: NoeudTalent): string {
    const def = ARBRES[this.personnage][this.arbre];
    const parties: string[] = [];
    if (n.libre) parties.push('Talent libre');
    else if (n.prerequis && n.prerequis.length > 0) {
      const noms = n.prerequis.map((id) => def.noeuds.find((x) => x.id === id)?.nom ?? id);
      parties.push(`${noms.join(' ou ')} complet`);
    } else parties.push('Racine de l’arbre');
    const palier = [...PALIERS].reverse().find((x) => n.ligne >= x.ligne);
    if (palier) parties.push(`${palier.rangs} rangs achetés au-dessus`);
    return parties.join(' · ');
  }
}

/** Icône d'émeraude suivie d'un nombre. */
export function compteurEmeraudes(textures: BibliothequeTextures, nombre: number): Node[] {
  const url = textures.url(RECOMPENSES.icone);
  const noeuds: Node[] = [];
  if (url) {
    const icone = document.createElement('img');
    icone.src = url;
    icone.alt = 'émeraudes';
    icone.className = 'icone-texte';
    noeuds.push(icone);
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

function image(url: string | null): HTMLElement {
  const img = document.createElement('img');
  if (url) img.src = url;
  img.alt = '';
  return img;
}
