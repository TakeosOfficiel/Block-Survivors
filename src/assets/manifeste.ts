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
import { ARBRES, RECOMPENSES } from '../donnees/talents';
import { IDS_PROJECTILES, PROJECTILES } from '../donnees/projectiles';
import { SKINS_VANILLA } from '../donnees/skins';
import type { ApparenceEntite, CheminTexture, DefinitionArene } from '../donnees/types';
import { cheminSkinVanilla } from '../profil';
import type { MondeGenere } from '../monde/generation';
import { BarreBoss } from '../ui/barreBoss';
import { SPRITES_INTERFACE } from '../ui/habillage';

/** Textures du menu et des écrans : sprites d'interface, visages, icônes. */
export function texturesInterface(): Set<CheminTexture> {
  const chemins = new Set<CheminTexture>(Object.values(SPRITES_INTERFACE));
  chemins.add(CONFIG.chargement.textureFond);
  for (const id of IDS_PERSONNAGES) {
    const p = PERSONNAGES[id];
    chemins.add(p.skin);
    if (p.objetEnMain) chemins.add(p.objetEnMain);
    for (const exclusif of p.cartesExclusives) chemins.add(PASSIFS[exclusif].icone);
  }
  for (const id of IDS_ARMES) chemins.add(ARMES[id].icone);
  for (const arene of ARENES) chemins.add(arene.icone);
  chemins.add(CONFIG.hud.coeurs.plein);
  chemins.add(RECOMPENSES.icone);
  for (const nom of SKINS_VANILLA) chemins.add(cheminSkinVanilla(nom, false)).add(cheminSkinVanilla(nom, true));
  for (const id of IDS_PERSONNAGES) {
    for (const arbre of ARBRES[id]) {
      chemins.add(arbre.icone);
      for (const talent of arbre.talents) chemins.add(talent.icone);
    }
  }
  return chemins;
}

export function texturesRequises(
  arene: DefinitionArene,
  monde: MondeGenere,
  personnage: ApparenceEntite,
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
    const m = arme.mecanique;
    if (m.type === 'balayage' || m.type === 'tourbillon') for (const image of m.effet) chemins.add(image);
    if (m.type === 'chute') chemins.add(m.image);
    if (m.type === 'tornade' || m.type === 'orbes') chemins.add(m.texture);
    if (m.type === 'nova') chemins.add(m.texture).add(m.sol);
    if (m.type === 'meteore') chemins.add(m.texture).add(m.feu);
    if (m.type === 'pluie' || m.type === 'piege') chemins.add(m.image);
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
