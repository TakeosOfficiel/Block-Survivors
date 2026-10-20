/**
 * BLOCK SURVIVORS — point d'entrée.
 * Charge les textures, construit la partie, lance la boucle.
 *
 * Note légale : les textures et noms Minecraft appartiennent à Mojang ;
 * projet de fan non commercial.
 */
import './ui/styles.css';
import { texturesRequises } from './assets/manifeste';
import { BibliothequeTextures } from './assets/textures';
import { CONFIG, SOURCES_TEXTURES } from './config';
import { ARENES } from './donnees/arenes';
import { IDS_PERSONNAGES, PERSONNAGE_PAR_DEFAUT, PERSONNAGES, type IdPersonnage } from './donnees/personnages';
import { Jeu } from './jeu';
import { Rendu } from './moteur/rendu';
import { EcranChargement } from './ui/ecranChargement';

async function demarrer(): Promise<void> {
  const ecran = new EcranChargement(SOURCES_TEXTURES[0].url(CONFIG.chargement.textureFond));

  let rendu: Rendu;
  try {
    rendu = new Rendu(document.body);
  } catch {
    ecran.erreur('WebGL est indisponible sur ce navigateur.');
    return;
  }

  const arene = ARENES[0];
  const personnage = personnageDemande();
  const textures = new BibliothequeTextures();
  await textures.charger(texturesRequises(arene, PERSONNAGES[personnage]), (faites, total) => ecran.progression(faites, total));

  const jeu = new Jeu(rendu, textures, arene, personnage);
  ecran.masquer();
  jeu.demarrer();

  // En développement uniquement : accès à la scène depuis la console du navigateur
  if (import.meta.env.DEV) Object.assign(window, { blockSurvivors: { jeu, rendu } });
}

/** Personnage choisi dans l'adresse (?personnage=archer), sinon celui par défaut. */
function personnageDemande(): IdPersonnage {
  const demande = new URLSearchParams(window.location.search).get('personnage');
  return IDS_PERSONNAGES.find((id) => id === demande) ?? PERSONNAGE_PAR_DEFAUT;
}

void demarrer();
