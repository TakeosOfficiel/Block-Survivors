/** Liste des textures à précharger avant une partie, déduite des données. */
import { CONFIG } from '../config';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_PROJECTILES, PROJECTILES } from '../donnees/projectiles';
import type { CheminTexture, DefinitionArene } from '../donnees/types';

export function texturesRequises(arene: DefinitionArene): Set<CheminTexture> {
  const chemins = new Set<CheminTexture>();
  const ajouterBloc = (id: IdBloc) => {
    const faces = BLOCS[id].faces;
    chemins.add(faces.haut).add(faces.cotes).add(faces.bas);
  };

  for (const choix of arene.sol) ajouterBloc(choix.bloc);
  for (const choix of arene.murs) ajouterBloc(choix.bloc);
  ajouterBloc(arene.exterieur.bloc);

  chemins.add(JOUEUR_BASE.skin);
  for (const id of IDS_ENNEMIS) chemins.add(ENNEMIS[id].skin);
  for (const id of IDS_PROJECTILES) chemins.add(PROJECTILES[id].texture);
  chemins.add(CONFIG.ombre.texture);
  for (const coeur of Object.values(CONFIG.hud.coeurs)) chemins.add(coeur);
  return chemins;
}
