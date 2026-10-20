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
  const sons = new Sons(options.volume);
  sons.precharger();
  const textures = new BibliothequeTextures();
  ecran.etape('Chargement de l\'interface…');
  await Promise.all([
    chargerPolice(),
    textures.charger(texturesInterface(), (faites, total) => ecran.progression(faites, total)),
  ]);
  appliquerHabillage(textures);
  document.getElementById('menu-rendu')?.replaceChildren(rendu.api);
  // Clic sur tout bouton de l'interface, comme en jeu
  document.addEventListener('click', (e) => {
    if ((e.target as Element | null)?.closest('button')) sons.jouer('clic');
  });

  let jeu: Jeu | null = null;
  const panneauOptions = new PanneauOptions(options, (o) => {
    sons.definirVolume(o.volume);
    jeu?.appliquerOptions(o);
  });

  // Menu, sauf si l'adresse demande de jouer tout de suite (Rejouer)
  let choix = choixDemande();
  if (!partieDemandee()) {
    ecran.masquer();
    choix = await new EcranMenu(textures, choix, panneauOptions).attendreChoix();
    memoriserChoix(choix);
    ecran.afficher();
  }

  ecran.etape('Génération du monde…');
  await new Promise((resoudre) => requestAnimationFrame(resoudre));  // laisse s'afficher l'étape
  const monde = genererMonde(choix.arene);
  ecran.etape('Chargement des textures…');
  await textures.charger(
    texturesRequises(choix.arene, monde, PERSONNAGES[choix.personnage]),
    (faites, total) => ecran.progression(faites, total),
  );

  jeu = new Jeu(rendu, textures, monde, choix, { sons, options, panneauOptions });
  jeu.appliquerOptions(options);
  ecran.masquer();
  jeu.demarrer();

  // En développement uniquement : accès à la partie depuis la console du navigateur
  if (import.meta.env.DEV) Object.assign(window, { blockSurvivors: { jeu, rendu } });
}

void demarrer();
