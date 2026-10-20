/** Liste des textures à précharger avant une partie, déduite des données. */
import { CONFIG } from '../config';
import { ALLIES, IDS_ALLIES } from '../donnees/allies';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { IDS_OBJETS, OBJETS } from '../donnees/objets';
import { IDS_PASSIFS, PASSIFS } from '../donnees/passifs';
import { PROGRESSION } from '../donnees/progression';
import { IDS_PROJECTILES, PROJECTILES } from '../donnees/projectiles';
import type { CheminTexture, DefinitionArene, DefinitionPersonnage } from '../donnees/types';
import { BarreBoss } from '../ui/barreBoss';

export function texturesRequises(arene: DefinitionArene, personnage: DefinitionPersonnage): Set<CheminTexture> {
  const chemins = new Set<CheminTexture>();
  const ajouterBloc = (id: IdBloc) => {
    const faces = BLOCS[id].faces;
    chemins.add(faces.haut).add(faces.cotes).add(faces.bas);
  };

  for (const choix of arene.sol) ajouterBloc(choix.bloc);
  for (const choix of arene.murs) ajouterBloc(choix.bloc);
  ajouterBloc(arene.exterieur.bloc);

  chemins.add(personnage.skin);
  for (const id of IDS_ENNEMIS) {
    const ennemi = ENNEMIS[id];
    chemins.add(ennemi.skin);
    if (ennemi.boss) for (const sprite of BarreBoss.sprites(ennemi.boss.couleurBarre)) chemins.add(sprite);
  }
  for (const id of IDS_OBJETS) chemins.add(OBJETS[id].icone);
  for (const id of IDS_PROJECTILES) chemins.add(PROJECTILES[id].texture);
  for (const id of IDS_ALLIES) chemins.add(ALLIES[id].skin);
  for (const id of IDS_ARMES) {
    const arme = ARMES[id];
    chemins.add(arme.icone);
    if (arme.mecanique.type === 'balayage') for (const image of arme.mecanique.effet) chemins.add(image);
  }
  chemins.add(CONFIG.ombre.texture);
  for (const id of IDS_PASSIFS) chemins.add(PASSIFS[id].icone);
  chemins.add(PROGRESSION.soin.icone);
  chemins.add(CONFIG.orbes.texture);
  for (const sprite of Object.values(CONFIG.hud.coeurs)) chemins.add(sprite);
  for (const sprite of Object.values(CONFIG.hud.barreXp)) chemins.add(sprite);
  return chemins;
}
