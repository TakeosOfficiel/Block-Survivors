/** Liste des textures à précharger avant une partie, déduite des données. */
import { CONFIG } from '../config';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import { JOUEUR_BASE } from '../donnees/joueur';
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
  chemins.add(CONFIG.ombre.texture);
  return chemins;
}
