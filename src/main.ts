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
  const textures = new BibliothequeTextures();
  await textures.charger(texturesRequises(arene), (faites, total) => ecran.progression(faites, total));

  const jeu = new Jeu(rendu, textures, arene);
  ecran.masquer();
  jeu.demarrer();
}

void demarrer();
