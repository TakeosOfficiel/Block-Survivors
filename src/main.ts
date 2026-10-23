/**
 * BLOCK SURVIVORS — point d'entrée.
 * Charge l'interface (police, sprites, sons), affiche le menu, puis génère
 * l'arène choisie, charge ses textures et lance la partie.
 *
 * Note légale : les textures, sons et noms Minecraft appartiennent à Mojang ;
 * projet de fan non commercial.
 */
import './ui/styles.css';
import { Sons } from './audio/sons';
import { texturesInterface, texturesRequises } from './assets/manifeste';
import { memoriserPhoto } from './assets/photosArenes';
import { chargerPolice } from './assets/police';
import { BibliothequeTextures } from './assets/textures';
import { PERSONNAGES } from './donnees/personnages';
import { Jeu } from './jeu';
import { choixDemande, memoriserChoix, partieDemandee } from './lancement';
import { importerArene } from './minecraft/importation';
import { genererMonde, type MondeGenere } from './monde/generation';
import { Rendu } from './moteur/rendu';
import { chargerOptions } from './options';
import { Collection } from './progression/collection';
import { Sauvegarde } from './progression/sauvegarde';
import { apparenceJoueur, chargerProfil, CHEMIN_SKIN_IMPORTE } from './profil';
import { ApercuHeros } from './ui/apercuHeros';
import { chargerImageSkin, EcranProfil } from './ui/ecranProfil';
import { EcranChargement } from './ui/ecranChargement';
import { EcranMenu } from './ui/ecranMenu';
import { suivreEchelleInterface } from './ui/echelle';
import { appliquerHabillage } from './ui/habillage';
import { enregistrerArmures, enregistrerIconesObjets } from './ui/iconesObjets';
import { PanneauOptions } from './ui/panneauOptions';
import { apparenceDistante, cheminSkinJoueur } from './reseau/apparence';
import { EcranSalon, type LancementMultijoueur } from './reseau/ecranSalon';
import { EcranProfils } from './ui/ecranProfils';

async function demarrer(): Promise<void> {
  suivreEchelleInterface();
  const ecran = new EcranChargement();
  const textures = new BibliothequeTextures();
  let choix = choixDemande();
  // Partie lancée directement (Rejouer) : la photo de son arène tout de suite
  if (partieDemandee()) ecran.decor(choix.arene);

  let rendu: Rendu;
  try {
    rendu = await Rendu.creer(document.body);
  } catch {
    ecran.erreur('Ni WebGPU ni WebGL2 ne sont disponibles sur ce navigateur.');
    return;
  }

  // Interface : police, sprites et sons, communs au menu et à la partie
  const options = chargerOptions();
  const sauvegarde = Sauvegarde.charger();
  const profil = chargerProfil();
  const collection = Collection.charger();
  const sons = new Sons(options.volume, options.volumes);
  sons.precharger();
  // Une seule barre : l'interface seule avant le menu, ou l'interface et la partie (Rejouer)
  ecran.planifier(1 + (partieDemandee() ? etapesPartie(choix) : 0));
  ecran.etape('Chargement de l\'interface…');
  await Promise.all([
    chargerPolice(),
    textures.charger(texturesInterface(), (faites, total) => ecran.progression(faites, total)),
  ]);
  appliquerHabillage(textures);
  enregistrerIconesObjets(textures);
  enregistrerArmures(textures);
  // Skin importée du joueur : enregistrée comme une texture du jeu
  if (profil.skinImporte) {
    const image = await chargerImageSkin(profil.skinImporte);
    if (image) textures.enregistrer(CHEMIN_SKIN_IMPORTE, image);
    else if (profil.skin === 'importe') profil.skin = 'classe';
  }
  // Clic sur tout bouton de l'interface, comme en jeu
  document.addEventListener('click', (e) => {
    if ((e.target as Element | null)?.closest('button')) sons.jouer('clic');
  });

  let jeu: Jeu | null = null;
  const panneauOptions = new PanneauOptions(options, (o) => {
    sons.definirVolumes(o.volume, o.volumes);
    jeu?.appliquerOptions(o);
  });

  // Menu, sauf si l'adresse demande de jouer tout de suite (Rejouer)
  let multijoueur: LancementMultijoueur | null = null;
  if (!partieDemandee()) {
    // Héros en 3D du menu : même moteur que le jeu (repli WebGL2 compris)
    const apercu = await ApercuHeros.creer(textures, rendu.api !== 'WebGPU');
    if (apercu) apercu.armure = () => collection.portees();
    ecran.masquer();
    // Le salon multijoueur s'ouvre depuis le menu ; la première partie lancée (solo ou à plusieurs) l'emporte
    new EcranProfils();
    const salon = new EcranSalon(sauvegarde, profil, collection, choix);
    const menu = new EcranMenu(textures, choix, panneauOptions, sauvegarde, new EcranProfil(textures, profil, apercu, collection), apercu, collection);
    const choixSolo = menu.attendreChoix();
    multijoueur = await Promise.race([choixSolo.then(() => null), salon.attendreLancement()]);
    if (multijoueur) {
      apercu?.detacher();
      choix = multijoueur.choix;
    } else {
      choix = await choixSolo;
      memoriserChoix(choix);
    }
    ecran.planifier(etapesPartie(choix));
  }
  ecran.decor(choix.arene);
  ecran.afficher();

  let monde: MondeGenere;
  if (choix.arene.source) {
    // Map construite dans Minecraft : sa taille et son décor viennent de la schématique
    try {
      const importee = await importerArene(choix.arene, textures, {
        etape: (texte) => ecran.etape(texte),
        progression: (faites, total) => ecran.progression(faites, total),
      });
      choix = { ...choix, arene: importee.arene };
      monde = importee.monde;
    } catch (erreur) {
      ecran.erreur(`Map « ${choix.arene.nom} » illisible : ${(erreur as Error).message}`);
      return;
    }
  } else {
    ecran.etape('Génération du monde…');
    await new Promise((resoudre) => requestAnimationFrame(resoudre));  // laisse s'afficher l'étape
    monde = genererMonde(choix.arene);
  }
  ecran.etape('Chargement des textures…');
  await textures.charger(
    texturesRequises(choix.arene, monde, apparenceJoueur(PERSONNAGES[choix.personnage], profil)),
    (faites, total) => ecran.progression(faites, total),
  );
  // À plusieurs : les skins des autres joueurs aussi (leurs skins importées d'abord enregistrées)
  for (const info of multijoueur?.session.joueurs ?? []) {
    if (info.skinImporte) {
      const image = await chargerImageSkin(info.skinImporte);
      if (image) textures.enregistrer(cheminSkinJoueur(info.index), image);
    }
    await textures.charger(texturesRequises(choix.arene, monde, apparenceDistante(info)));
  }

  jeu = new Jeu(rendu, textures, monde, choix, { sons, options, panneauOptions, sauvegarde, profil, collection, reseau: multijoueur?.session });
  jeu.appliquerOptions(options);
  ecran.etape('Préparation des modèles…');
  await jeu.prechauffer((faites, total) => ecran.progression(faites, total));
  // Photo de l'arène pour le prochain chargement, prise juste avant que la partie ne commence
  const photo = jeu.photographier();
  if (multijoueur) {
    ecran.etape('En attente des autres joueurs…');
    await multijoueur.session.synchroniserDepart();
  }
  ecran.masquer();
  jeu.demarrer();
  if (photo) memoriserPhoto(choix.arene.id, photo);

  // En développement uniquement : accès à la partie depuis la console du navigateur
  if (import.meta.env.DEV) Object.assign(window, { blockSurvivors: { jeu, rendu } });
}

/** Étapes du chargement d'une partie : le monde (une map importée en compte quatre), les textures, les modèles. */
function etapesPartie(choix: { readonly arene: { readonly source?: unknown } }): number {
  return (choix.arene.source ? 4 : 1) + 2;
}

void demarrer();
