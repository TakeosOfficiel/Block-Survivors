/**
 * Menu principal : la photo de l'arène choisie en plein écran, les classes à
 * gauche (avec les détails de la choisie), le héros en 3D au centre (pseudo,
 * apparence, talents), les arènes en vignettes photo à droite (boss, record,
 * déblocages), puis le grand bouton « Jouer » (ou Entrée).
 */
import { photoArene } from '../assets/photosArenes';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
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
import { EcranTalents } from './ecranTalents';
import { EcranCollection } from './ecranCollection';
import type { Collection } from '../progression/collection';
import { bandeauMaitrise } from './maitrise';
import type { PanneauOptions } from './panneauOptions';

const NOMBRE = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

/** Pont exposé par l'application de bureau (electron/preload.cjs) ; absent dans un navigateur. */
const bureau = (window as {
  bureau?: {
    quitter(): void;
    /** Absents des anciens exe portables, qui ne se mettent pas à jour seuls. */
    surMiseAJourPrete?(rappel: (version: string) => void): void;
    installerMiseAJour?(): void;
  };
}).bureau;

export class EcranMenu {
  private readonly racine = element('menu');
  private readonly listePersonnages = element('menu-personnages');
  private readonly listeArenes = element('menu-arenes');
  private readonly fond = element('menu-fond');
  /** Arène dont la photo de fond est demandée (une demande plus récente l'emporte). */
  private fondVoulu = '';
  private readonly pseudo = element<HTMLInputElement>('menu-pseudo');
  private readonly textures: BibliothequeTextures;
  private readonly options: PanneauOptions;
  private readonly sauvegarde: Sauvegarde;
  private readonly collection: EcranCollection;
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
    collection: Collection,
  ) {
    this.textures = textures;
    this.options = options;
    this.sauvegarde = sauvegarde;
    this.talents = new EcranTalents(textures, sauvegarde);
    this.collection = new EcranCollection(textures, collection);
    this.profil = profil;
    this.apercu = apercu;
    this.personnage = choix.personnage;
    // Une arène mémorisée encore verrouillée revient à la première disponible
    this.arene = sauvegarde.verrou('arene', choix.arene.id)
      ? ARENES.find((a) => !sauvegarde.verrou('arene', a.id)) ?? choix.arene
      : choix.arene;

    element('menu-version').textContent = libelleVersion();
    const lien = element<HTMLAnchorElement>('menu-mise-a-jour');
    if (bureau?.surMiseAJourPrete && bureau.installerMiseAJour) {
      // Application installée : les mises à jour s'installent au lancement ; une version sortie
      // pendant la partie s'installe à la fermeture (un clic redémarre le jeu dessus tout de suite)
      const installer = bureau.installerMiseAJour;
      bureau.surMiseAJourPrete((version) => {
        lien.removeAttribute('href');
        lien.textContent = `Version ${version} téléchargée : elle s'installera à la fermeture du jeu (cliquez pour redémarrer maintenant)`;
        lien.classList.remove('cache');
      });
      lien.addEventListener('click', (e) => {
        e.preventDefault();
        installer();
      });
    } else {
      // Navigateur ou ancien exe portable : lien vers la page de téléchargement
      void miseAJourDisponible().then((publiee) => {
        if (!publiee) return;
        lien.href = publiee.telechargement;
        lien.textContent = `Version ${publiee.version} disponible ! Cliquez pour la télécharger`;
        lien.classList.remove('cache');
      });
    }

    // Passage à la maîtrise : les talents achetés en émeraudes ont été remis à zéro
    if (sauvegarde.annoncerReinitialisation()) {
      const bouton = element<HTMLButtonElement>('menu-remboursement');
      bouton.textContent = 'Nouveau : chaque classe a sa maîtrise, qui monte avec l’XP ramassée ; un niveau = un point de talent. Talents remis à zéro, cliquez pour voir';
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
    element<HTMLButtonElement>('menu-collection').addEventListener('click', () => {
      this.collection.ouvrir(this.arene.monde, () => this.rafraichir());
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

  /** Classe et arène choisies en ce moment. */
  get choix(): ChoixPartie {
    return { personnage: this.personnage, arene: this.arene };
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
    const p = this.personnage;
    element('menu-maitrise').replaceChildren(bandeauMaitrise(
      this.textures, PERSONNAGES[p].nom, this.sauvegarde.maitrise(p), this.sauvegarde.pointsLibres(p, this.sauvegarde.arbreActif(p)),
    ));
    for (const bouton of this.listePersonnages.children) {
      bouton.classList.toggle('choisie', (bouton as HTMLElement).dataset['id'] === this.personnage);
    }
    this.afficherClasse();
    this.afficherHeros();
    this.afficherArenes();
    this.afficherFond();
    element('menu-jouer-titre').textContent = 'Jouer';
    element('menu-jouer-sous-titre').textContent = `${PERSONNAGES[this.personnage].nom} · ${this.arene.nom}`;
  }

  /** La photo de l'arène choisie, en fond (fondu à chaque changement d'arène). */
  private afficherFond(): void {
    const id = this.arene.id;
    if (id === this.fondVoulu) return;
    this.fondVoulu = id;
    this.fond.classList.remove('visible');
    void photoArene(id).then((adresse) => {
      if (!adresse || this.fondVoulu !== id) return;
      const image = new Image();
      image.onload = () => {
        if (this.fondVoulu !== id) return;
        this.fond.style.backgroundImage = `url("${adresse}")`;
        this.fond.classList.add('visible');
      };
      image.src = adresse;
    });
  }

  // --- Classe ---------------------------------------------------------------

  private creerBoutonClasse(id: IdPersonnage): HTMLElement {
    const def = PERSONNAGES[id];
    const bouton = document.createElement('button');
    bouton.className = 'menu-classe';
    bouton.dataset['id'] = id;
    const icone = document.createElement('span');
    icone.className = 'menu-classe-icone';
    const url = this.textures.url(def.objetEnMain ?? ARMES[def.armesDepart[0]].icone);
    if (url) icone.append(image(url));
    const textes = document.createElement('span');
    textes.append(texte(def.nom, 'menu-classe-nom'), texte(def.role, 'menu-classe-role'));
    bouton.append(icone, textes);
    bouton.addEventListener('click', () => {
      this.personnage = id;
      this.rafraichir();
    });
    bouton.addEventListener('dblclick', () => this.jouer());
    return bouton;
  }

  private afficherClasse(): void {
    const def = PERSONNAGES[this.personnage];
    element('menu-classe-description').textContent = def.description;

    const arme = ARMES[def.armesDepart[0]];
    element('menu-classe-tuiles').replaceChildren(
      puce(NOMBRE.format(def.pv / 2), this.textures.url(CONFIG.hud.coeurs.plein), 'Cœurs'),
      puce(`Vitesse ${NOMBRE.format(def.vitesse)}`),
      puce(arme.nom, this.textures.url(arme.icone), 'Arme de départ'),
    );
    element('menu-classe-bonus').replaceChildren(...decrireBonus(def.bonus).map((ligne) => texte(ligne)));
    element('menu-classe-exclusifs').replaceChildren(...def.cartesExclusives.map((id) => {
      const passif = PASSIFS[id];
      const caseIcone = caseImage(this.textures.url(passif.icone), passif.nom);
      caseIcone.title = `${passif.nom} : ${passif.description}`;
      return caseIcone;
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
    element('menu-apercu').title = `${this.profil.nomSkin()} · glisser pour tourner`;
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
      : [texte('Aucun talent acheté : jouez cette classe pour monter sa maîtrise.', 'menu-talents-vide')]));
  }

  // --- Arènes ---------------------------------------------------------------

  private afficherArenes(): void {
    this.listeArenes.replaceChildren(...ARENES.map((arene) => {
      const verrou = this.sauvegarde.verrou('arene', arene.id);
      const bouton = document.createElement('button');
      bouton.className = 'menu-arene';
      bouton.classList.toggle('choisie', arene.id === this.arene.id);
      bouton.classList.toggle('verrouillee', verrou !== null);
      bouton.title = arene.description;
      bouton.append(texte(arene.nom, 'menu-arene-nom'));
      void photoArene(arene.id).then((adresse) => {
        if (adresse) bouton.style.backgroundImage = `url("${adresse}")`;
      });
      if (verrou) {
        bouton.append(texte(`Verrouillée · ${verrou.texte}`, 'menu-arene-verrou'));
        bouton.disabled = true;
        return bouton;
      }
      bouton.addEventListener('click', () => {
        this.arene = arene;
        this.rafraichir();
      });
      bouton.addEventListener('dblclick', () => this.jouer());
      return bouton;
    }));

    // Détails de l'arène choisie : mobs, boss, record
    const a = this.arene;
    const boss = document.createElement('div');
    boss.className = 'menu-arene-boss';
    boss.append(...this.prochainsBoss(a).map((b) => texte(b.texte)));
    const details: HTMLElement[] = [texte(a.description), texte(`Mobs : ${this.nomsMobs(a)}`, 'menu-arene-mobs'), boss];
    const record = this.sauvegarde.record(a.id);
    if (record > 0) details.push(texte(`Record : ${formaterTemps(record)}`, 'menu-arene-record'));
    const recordSansFin = this.sauvegarde.recordSansFin(a.id);
    if (recordSansFin > 0) details.push(texte(`Record sans fin : ${formaterTemps(recordSansFin)}`, 'menu-arene-record'));
    element('menu-arene-details').replaceChildren(...details);
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

/** Pastille arrondie : icône (facultative) et valeur ; `titre` en infobulle. */
function puce(valeur: string, icone?: string | null, titre?: string): HTMLElement {
  const pastille = texte('', 'menu-puce');
  if (icone) pastille.append(image(icone));
  pastille.append(valeur);
  if (titre) pastille.title = titre;
  return pastille;
}

function image(url: string): HTMLImageElement {
  const img = document.createElement('img');
  img.src = url;
  img.alt = '';
  return img;
}
