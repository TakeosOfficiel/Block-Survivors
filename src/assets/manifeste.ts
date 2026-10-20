/** Listes des textures à précharger (menu, puis partie), déduites des données. */
import { CONFIG } from '../config';
import { ALLIES, IDS_ALLIES } from '../donnees/allies';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import { IDS_OBJETS, OBJETS } from '../donnees/objets';
import { IDS_PARTICULES, PARTICULES } from '../donnees/particules';
import { IDS_PASSIFS, PASSIFS } from '../donnees/passifs';
import { PROGRESSION } from '../donnees/progression';
import { ARENES } from '../donnees/arenes';
import { IDS_PERSONNAGES, PERSONNAGES } from '../donnees/personnages';
import { IDS_PROJECTILES, PROJECTILES } from '../donnees/projectiles';
import type { CheminTexture, DefinitionArene, DefinitionPersonnage } from '../donnees/types';
import type { MondeGenere } from '../monde/generation';
import { BarreBoss } from '../ui/barreBoss';
import { SPRITES_INTERFACE } from '../ui/habillage';

/** Textures du menu et des écrans : sprites d'interface, visages, icônes. */
export function texturesInterface(): Set<CheminTexture> {
  const chemins = new Set<CheminTexture>(Object.values(SPRITES_INTERFACE));
  chemins.add(CONFIG.chargement.textureFond);
  for (const id of IDS_PERSONNAGES) chemins.add(PERSONNAGES[id].skin);
  for (const id of IDS_ARMES) chemins.add(ARMES[id].icone);
  for (const arene of ARENES) chemins.add(arene.icone);
  chemins.add(CONFIG.hud.coeurs.plein);
  return chemins;
}

export function texturesRequises(
  arene: DefinitionArene,
  monde: MondeGenere,
  personnage: DefinitionPersonnage,
): Set<CheminTexture> {
  const chemins = new Set<CheminTexture>();
  const ajouterBloc = (id: IdBloc) => {
    const faces = BLOCS[id].faces;
    chemins.add(faces.haut).add(faces.cotes).add(faces.bas);
  };

  for (const id of monde.blocs.keys()) ajouterBloc(id);
  ajouterBloc(arene.exterieur.bloc);

  chemins.add(personnage.skin);
  if (personnage.objetEnMain) chemins.add(personnage.objetEnMain);
  for (const id of IDS_ENNEMIS) {
    const ennemi = ENNEMIS[id];
    chemins.add(ennemi.skin);
    if (ennemi.objetEnMain) chemins.add(ennemi.objetEnMain);
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
  for (const id of IDS_PARTICULES) for (const image of PARTICULES[id].images) chemins.add(image);
  for (const id of IDS_PASSIFS) chemins.add(PASSIFS[id].icone);
  chemins.add(PROGRESSION.soin.icone);
  chemins.add(CONFIG.orbes.texture);
  for (const sprite of Object.values(CONFIG.hud.coeurs)) chemins.add(sprite);
  for (const sprite of Object.values(CONFIG.hud.barreXp)) chemins.add(sprite);
  chemins.add(CONFIG.hud.iconeTues);
  return chemins;
}
