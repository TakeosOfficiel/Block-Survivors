/**
 * Menu principal en trois colonnes : la classe (choix et détails), le héros
 * (aperçu 3D, pseudo, apparence, talents) et l'arène (boss, déblocages),
 * puis le grand bouton « Jouer » (ou Entrée).
 */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ACCROCHES } from '../donnees/accroches';
import { ARENES } from '../donnees/arenes';
import { ARMES } from '../donnees/armes';
import { CALENDRIER_BOSS } from '../donnees/boss';
import { ENNEMIS, type IdEnnemi } from '../donnees/ennemis';
import { PASSIFS } from '../donnees/passifs';
import { IDS_PERSONNAGES, PERSONNAGES, type IdPersonnage } from '../donnees/personnages';
import { PROGRESSION } from '../donnees/progression';
import { ARBRES } from '../donnees/talents';
import type { DefinitionArene } from '../donnees/types';
import type { ChoixPartie } from '../lancement';
import { decrireBonus } from '../progression/ameliorations';
import type { Sauvegarde } from '../progression/sauvegarde';
import { apparenceJoueur, enregistrerProfil, LONGUEUR_PSEUDO, nettoyerPseudo, pseudoAleatoire } from '../profil';
import { formaterTemps } from '../utils/format';
import { libelleVersion, miseAJourDisponible } from '../version';
import type { ApercuHeros } from './apercuHeros';
import { caseImage } from './bilan';
import { element } from './dom';
import { EcranNouveautes } from './ecranNouveautes';
import type { EcranProfil } from './ecranProfil';
import { compteurEmeraudes, EcranTalents } from './ecranTalents';
import type { PanneauOptions } from './panneauOptions';

const NOMBRE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

/** Pont exposé par l'application de bureau (electron/preload.cjs) ; absent dans un navigateur. */
const bureau = (window as { bureau?: { quitter(): void } }).bureau;

export class EcranMenu {
  private readonly racine = element('menu');
  private readonly listePersonnages = element('menu-personnages');
  private readonly listeArenes = element('menu-arenes');
  private readonly pseudo = element<HTMLInputElement>('menu-pseudo');
  private readonly textures: BibliothequeTextures;
  private readonly options: PanneauOptions;
  private readonly sauvegarde: Sauvegarde;
  private readonly talents: EcranTalents;
  private readonly nouveautes = new EcranNouveautes();
  private readonly profil: EcranProfil;
  private readonly apercu: ApercuHeros | null;
  private personnage: IdPersonnage;
  private arene: DefinitionArene;
  private surJouer: ((choix: ChoixPartie) => void) | null = null;

  constructor(
    textures: BibliothequeTextures,
    choix: ChoixPartie,
    options: PanneauOptions,
    sauvegarde: Sauvegarde,
    profil: EcranProfil,
    apercu: ApercuHeros | null,
  ) {
    this.textures = textures;
    this.options = options;
    this.sauvegarde = sauvegarde;
    this.talents = new EcranTalents(textures, sauvegarde);
    this.profil = profil;
    this.apercu = apercu;
    this.personnage = choix.personnage;
    // Une arène mémorisée encore verrouillée revient à la première disponible
    this.arene = sauvegarde.verrou('arene', choix.arene.id)
      ? ARENES.find((a) => !sauvegarde.verrou('arene', a.id)) ?? choix.arene
      : choix.arene;

    element('menu-accroche').textContent = ACCROCHES[Math.floor(Math.random() * ACCROCHES.length)];
    element('menu-version').textContent = libelleVersion();
    void miseAJourDisponible().then((publiee) => {
      if (!publiee) return;
      const lien = element<HTMLAnchorElement>('menu-mise-a-jour');
      lien.href = publiee.telechargement;
      lien.textContent = `Version ${publiee.version} disponible ! Cliquez pour la télécharger`;
      lien.classList.remove('cache');
    });

    // Passage aux nouveaux arbres : les émeraudes des anciens talents ont été rendues
    const rendu = sauvegarde.annoncerRemboursement();
    if (rendu > 0) {
      const bouton = element<HTMLButtonElement>('menu-remboursement');
      bouton.textContent = `Nouveaux arbres de talents : ${rendu} émeraudes rendues. Cliquez pour les dépenser`;
      bouton.classList.remove('cache');
      bouton.addEventListener('click', () => {
        bouton.classList.add('cache');
        this.talents.ouvrir(this.personnage, () => this.rafraichir());
      });
    }

    this.listePersonnages.append(...IDS_PERSONNAGES.map((id) => this.creerBoutonClasse(id)));
    this.creerPseudo();
    element<HTMLButtonElement>('menu-jouer').addEventListener('click', () => this.jouer());
    element<HTMLButtonElement>('menu-options').addEventListener('click', () => this.options.ouvrir());
    element<HTMLButtonElement>('menu-nouveautes').addEventListener('click', () => this.nouveautes.ouvrir());
    element<HTMLButtonElement>('menu-talents').addEventListener('click', () => {
      this.talents.ouvrir(this.personnage, () => this.rafraichir());
    });
    element<HTMLButtonElement>('menu-profil-bouton').addEventListener('click', () => {
      this.profil.ouvrir(PERSONNAGES[this.personnage], () => this.rafraichir());
    });
    if (bureau) {
      const quitter = element<HTMLButtonElement>('menu-quitter');
      quitter.classList.remove('cache');
      quitter.addEventListener('click', () => bureau.quitter());
    }
    window.addEventListener('keydown', this.surTouche);
    this.rafraichir();
  }

  /** Affiche le menu et attend que le joueur lance une partie. */
  attendreChoix(): Promise<ChoixPartie> {
    this.racine.classList.remove('cache');
    this.rafraichir();
    this.nouveautes.signalerNouvelleVersion();
    return new Promise((resoudre) => {
      this.surJouer = resoudre;
    });
  }

  private jouer(): void {
    const rappel = this.surJouer;
    if (!rappel) return;
    this.surJouer = null;
    window.removeEventListener('keydown', this.surTouche);
    this.apercu?.detacher();
    this.racine.classList.add('cache');
    rappel({ personnage: this.personnage, arene: this.arene });
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (e.code !== 'Enter' || e.repeat || this.options.ouvert || this.talents.ouvert || this.profil.ouvert || this.nouveautes.ouvert) return;
    e.preventDefault();
    this.jouer();
  };

  /** Tout ce qui dépend du choix courant, de la sauvegarde ou du profil. */
  private rafraichir(): void {
    element('menu-emeraudes').replaceChildren(...compteurEmeraudes(this.textures, this.sauvegarde.emeraudes));
    for (const bouton of this.listePersonnages.children) {
      bouton.classList.toggle('choisie', (bouton as HTMLElement).dataset['id'] === this.personnage);
    }
    this.afficherClasse();
    this.afficherHeros();
    this.afficherArenes();
    const def = PERSONNAGES[this.personnage];
    element('menu-jouer-titre').textContent = `Jouer — ${this.arene.nom}`;
    element('menu-jouer-sous-titre').textContent = `${def.nom} · ${this.prochainsBoss(this.arene)[0]?.texte ?? ''}`;
  }

  // --- Classe ---------------------------------------------------------------

  private creerBoutonClasse(id: IdPersonnage): HTMLElement {
    const def = PERSONNAGES[id];
    const bouton = document.createElement('button');
    bouton.className = 'menu-classe';
    bouton.dataset['id'] = id;
    const icone = def.objetEnMain ?? ARMES[def.armesDepart[0]].icone;
    bouton.title = def.nom;
    bouton.append(caseImage(this.textures.url(icone), def.nom, 'grande'));
    bouton.addEventListener('click', () => {
      this.personnage = id;
      this.rafraichir();
    });
    bouton.addEventListener('dblclick', () => this.jouer());
    return bouton;
  }

  private afficherClasse(): void {
    const def = PERSONNAGES[this.personnage];
    element('menu-classe-nom').textContent = def.nom;
    element('menu-classe-description').textContent = def.description;

    const arme = ARMES[def.armesDepart[0]];
    element('menu-classe-tuiles').replaceChildren(
      tuile('Cœurs', NOMBRE.format(def.pv / 2), this.textures.url(CONFIG.hud.coeurs.plein)),
      tuile('Vitesse', NOMBRE.format(def.vitesse)),
      tuile('Commence avec', arme.nom, this.textures.url(arme.icone)),
    );
    element('menu-classe-bonus').replaceChildren(...decrireBonus(def.bonus).map((ligne) => texte(ligne)));
    element('menu-classe-exclusifs').replaceChildren(...def.cartesExclusives.map((id) => {
      const passif = PASSIFS[id];
      const caseIcone = caseImage(this.textures.url(passif.icone), passif.nom);
      caseIcone.title = `${passif.nom} : ${passif.description}`;
      return caseIcone;
    }));
    // Les deux arbres de la classe ; l'actif est en évidence
    const actif = this.sauvegarde.arbreActif(this.personnage);
    element('menu-classe-arbres').replaceChildren(...ARBRES[this.personnage].map((arbre, index) => {
      const ligne = document.createElement('div');
      ligne.className = 'menu-arbre';
      ligne.classList.toggle('actif', index === actif);
      const textes = document.createElement('div');
      textes.className = 'option-textes';
      textes.append(texte(index === actif ? `${arbre.nom} (actif)` : arbre.nom, 'option-nom'));
      ligne.title = arbre.description;
      ligne.append(caseImage(this.textures.url(arbre.icone), arbre.nom), textes);
      return ligne;
    }));
  }

  // --- Héros ----------------------------------------------------------------

  private creerPseudo(): void {
    this.pseudo.maxLength = LONGUEUR_PSEUDO;
    this.pseudo.value = this.profil.profil.pseudo;
    this.pseudo.addEventListener('input', () => {
      this.profil.profil.pseudo = nettoyerPseudo(this.pseudo.value);
      enregistrerProfil(this.profil.profil);
    });
    // Les touches tapées dans le pseudo ne lancent pas la partie
    this.pseudo.addEventListener('keydown', (e) => {
      e.stopPropagation();
      if (e.code === 'Enter') this.pseudo.blur();
    });
    element<HTMLButtonElement>('menu-pseudo-aleatoire').addEventListener('click', () => {
      this.profil.profil.pseudo = pseudoAleatoire();
      this.pseudo.value = this.profil.profil.pseudo;
      enregistrerProfil(this.profil.profil);
    });
  }

  private afficherHeros(): void {
    const def = PERSONNAGES[this.personnage];
    this.pseudo.value = this.profil.profil.pseudo;
    element('menu-heros-ligne').textContent = def.nom;
    element('menu-heros-skin').textContent = this.profil.nomSkin();
    // L'aperçu revient au menu après l'écran d'apparence
    if (this.apercu && !this.profil.ouvert && !this.racine.classList.contains('cache')) {
      this.apercu.attacher(element('menu-apercu'));
      this.apercu.afficher(apparenceJoueur(def, this.profil.profil));
    }

    // Arbre de talents actif et rangs achetés
    const index = this.sauvegarde.arbreActif(this.personnage);
    const arbre = ARBRES[this.personnage][index];
    const points = this.sauvegarde.points(this.personnage, index);
    element('menu-arbre-titre').textContent = `Talents : ${arbre.nom} (${points} rang${points > 1 ? 's' : ''})`;
    const p = this.personnage;
    const actifs = arbre.noeuds.filter((n) => this.sauvegarde.actif(p, index, n));
    element('menu-talents-actifs').replaceChildren(...(actifs.length > 0
      ? actifs.map((n) => {
        const rang = this.sauvegarde.rang(p, index, n);
        const choix = n.options?.[this.sauvegarde.option(p, index, n)];
        const nom = choix ? `${n.nom} : ${choix.nom}` : n.nom;
        const caseIcone = caseImage(this.textures.url(choix?.icone ?? n.icone), nom);
        caseIcone.title = `${nom} (${rang}/${n.rangMax}) : ${choix?.effet ?? n.effet}`;
        caseIcone.classList.toggle('talent-final', n.type === 'final');
        if (n.rangMax > 1) caseIcone.append(texte(String(rang), 'talent-rang'));
        return caseIcone;
      })
      : [texte('Aucun talent acheté : gagnez des émeraudes en jouant.', 'option-description')]));
  }

  // --- Arènes ---------------------------------------------------------------

  private afficherArenes(): void {
    this.listeArenes.replaceChildren(...ARENES.map((arene) => {
      const verrou = this.sauvegarde.verrou('arene', arene.id);
      const choisie = arene.id === this.arene.id;
      const bouton = document.createElement('button');
      bouton.className = 'menu-arene';
      bouton.classList.toggle('choisie', choisie);
      bouton.classList.toggle('verrouillee', verrou !== null);

      const entete = document.createElement('div');
      entete.className = 'menu-arene-entete';
      entete.append(caseImage(this.textures.url(arene.icone), arene.nom), texte(arene.nom, 'option-nom'));
      bouton.append(entete);
      if (verrou) {
        bouton.append(texte(`Verrouillée : ${verrou.texte}`, 'option-verrou'));
        bouton.disabled = true;
        return bouton;
      }
      if (choisie) {
        bouton.append(texte(arene.description, 'option-description'));
        bouton.append(texte(`Mobs : ${this.nomsMobs(arene)}`, 'menu-arene-mobs'));
        const boss = document.createElement('div');
        boss.className = 'menu-puces';
        boss.append(...this.prochainsBoss(arene).map((b) => texte(b.texte, 'menu-puce')));
        bouton.append(boss);
        const record = this.sauvegarde.record(arene.id);
        if (record > 0) bouton.append(texte(`Record : ${formaterTemps(record)}`, 'menu-arene-record'));
      }
      bouton.addEventListener('click', () => {
        this.arene = arene;
        this.rafraichir();
      });
      bouton.addEventListener('dblclick', () => this.jouer());
      return bouton;
    }));
  }

  /** Noms des mobs de l'arène (toutes zones), dans l'ordre d'arrivée. */
  private nomsMobs(arene: DefinitionArene): string {
    const ids = new Set<IdEnnemi>();
    const tous = [...arene.ennemis, ...arene.zones.flatMap((z) => z.ennemis)].sort((a, b) => a.depuis - b.depuis);
    for (const e of tous) ids.add(e.ennemi);
    return [...ids].map((id) => ENNEMIS[id].nom.toLowerCase()).join(', ');
  }

  /** « 5:00 · Géant », « 10:00 · Seigneur wither »… jusqu'à la fin de la partie. */
  private prochainsBoss(arene: DefinitionArene): { texte: string }[] {
    const liste: { texte: string }[] = [];
    const c = CALENDRIER_BOSS;
    for (let t = c.premierPassage, i = 0; t < PROGRESSION.dureePartie; t += c.intervalle, i++) {
      liste.push({ texte: `${formaterTemps(t)} · ${ENNEMIS[arene.boss[i % arene.boss.length]].nom}` });
    }
    return liste;
  }
}

function texte(contenu: string, classe?: string): HTMLElement {
  const span = document.createElement('span');
  if (classe) span.className = classe;
  span.textContent = contenu;
  return span;
}

/** Petite case « intitulé / valeur » (points de vie, arme de départ…). */
function tuile(intitule: string, valeur: string, icone?: string | null): HTMLElement {
  const boite = document.createElement('div');
  boite.className = 'menu-tuile';
  boite.append(texte(intitule, 'menu-tuile-intitule'));
  const ligne = texte('', 'menu-tuile-valeur');
  if (icone) {
    const image = document.createElement('img');
    image.src = icone;
    image.alt = '';
    image.className = 'icone-texte';
    ligne.append(image);
  }
  ligne.append(valeur);
  boite.append(ligne);
  return boite;
}
