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
import { chargerPolice } from './assets/police';
import { BibliothequeTextures } from './assets/textures';
import { CONFIG, SOURCES_TEXTURES } from './config';
import { PERSONNAGES } from './donnees/personnages';
import { Jeu } from './jeu';
import { choixDemande, memoriserChoix, partieDemandee } from './lancement';
import { genererMonde } from './monde/generation';
import { Rendu } from './moteur/rendu';
import { chargerOptions } from './options';
import { Sauvegarde } from './progression/sauvegarde';
import { apparenceJoueur, chargerProfil, CHEMIN_SKIN_IMPORTE } from './profil';
import { ApercuHeros } from './ui/apercuHeros';
import { chargerImageSkin, EcranProfil } from './ui/ecranProfil';
import { EcranChargement } from './ui/ecranChargement';
import { EcranMenu } from './ui/ecranMenu';
import { suivreEchelleInterface } from './ui/echelle';
import { appliquerHabillage } from './ui/habillage';
import { PanneauOptions } from './ui/panneauOptions';

async function demarrer(): Promise<void> {
  suivreEchelleInterface();
  const ecran = new EcranChargement(SOURCES_TEXTURES[0].url(CONFIG.chargement.textureFond));

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
  const sons = new Sons(options.volume, options.volumes);
  sons.precharger();
  const textures = new BibliothequeTextures();
  ecran.etape('Chargement de l\'interface…');
  await Promise.all([
    chargerPolice(),
    textures.charger(texturesInterface(), (faites, total) => ecran.progression(faites, total)),
  ]);
  appliquerHabillage(textures);
  // Skin importée par le joueur : enregistrée comme une texture du jeu
  if (profil.skinImporte) {
    const image = await chargerImageSkin(profil.skinImporte);
    if (image) textures.enregistrer(CHEMIN_SKIN_IMPORTE, image);
    else if (profil.skin === 'importe') profil.skin = 'classe';
  }
  document.getElementById('menu-rendu')?.replaceChildren(rendu.api);
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
  let choix = choixDemande();
  if (!partieDemandee()) {
    // Héros en 3D du menu : même moteur que le jeu (repli WebGL2 compris)
    const apercu = await ApercuHeros.creer(textures, rendu.api !== 'WebGPU');
    ecran.masquer();
    choix = await new EcranMenu(textures, choix, panneauOptions, sauvegarde, new EcranProfil(textures, profil, apercu), apercu).attendreChoix();
    memoriserChoix(choix);
    ecran.afficher();
  }

  ecran.etape('Génération du monde…');
  await new Promise((resoudre) => requestAnimationFrame(resoudre));  // laisse s'afficher l'étape
  const monde = genererMonde(choix.arene);
  ecran.etape('Chargement des textures…');
  await textures.charger(
    texturesRequises(choix.arene, monde, apparenceJoueur(PERSONNAGES[choix.personnage], profil)),
    (faites, total) => ecran.progression(faites, total),
  );

  jeu = new Jeu(rendu, textures, monde, choix, { sons, options, panneauOptions, sauvegarde, profil });
  jeu.appliquerOptions(options);
  ecran.masquer();
  jeu.demarrer();

  // En développement uniquement : accès à la partie depuis la console du navigateur
  if (import.meta.env.DEV) Object.assign(window, { blockSurvivors: { jeu, rendu } });
}

void demarrer();
