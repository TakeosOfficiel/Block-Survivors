/**
 * Bonus passifs : modificateurs globaux du joueur. Ajouter un passif = ajouter
 * une entrée ; chaque rang ajoute sa `modif` aux modificateurs (voir Modificateurs).
 */
import type { DefinitionPassif, NiveauPassif } from './types';

/** `rangs` niveaux identiques. */
function rangs(nombre: number, niveau: NiveauPassif): NiveauPassif[] {
  return Array.from({ length: nombre }, () => niveau);
}

const definitions = {
  force: {
    nom: 'Poudre de blaze',
    description: 'Toutes les armes font plus de dégâts.',
    icone: 'item/blaze_powder',
    niveaux: rangs(5, { description: 'Dégâts +10 %', modif: { degats: 0.1 } }),
  },
  frenesie: {
    nom: 'Redstone',
    description: 'Les armes se déclenchent plus souvent.',
    icone: 'item/redstone',
    niveaux: rangs(5, { description: 'Délai des armes −7 %', modif: { cadence: -0.07 } }),
  },
  amplitude: {
    nom: 'Poudre de glowstone',
    description: 'Les attaques couvrent une plus grande zone.',
    icone: 'item/glowstone_dust',
    niveaux: rangs(5, { description: 'Zone +10 %', modif: { zone: 0.1 } }),
  },
  multitir: {
    nom: 'Carquois',
    description: 'Toutes les armes lancent un projectile de plus.',
    icone: 'item/arrow',
    niveaux: rangs(2, { description: '+1 projectile', modif: { projectiles: 1 } }),
  },
  longue_vue: {
    nom: 'Longue-vue',
    description: 'Les armes visent et portent plus loin.',
    icone: 'item/spyglass',
    niveaux: rangs(5, { description: 'Portée +10 %', modif: { portee: 0.1 } }),
  },
  rapidite: {
    nom: 'Sucre',
    description: 'Vous vous déplacez plus vite.',
    icone: 'item/sugar',
    niveaux: rangs(5, { description: 'Vitesse +8 %', modif: { vitesse: 0.08 } }),
  },
  vitalite: {
    nom: 'Pastèque scintillante',
    description: 'Augmente la santé maximale.',
    icone: 'item/glistering_melon_slice',
    niveaux: rangs(5, { description: '+2 cœurs', modif: { pvMax: 4 } }),
  },
  armure: {
    nom: 'Plastron en fer',
    description: 'Réduit les dégâts de chaque coup reçu.',
    icone: 'item/iron_chestplate',
    niveaux: rangs(5, { description: 'Dégâts reçus −1', modif: { armure: 1 } }),
  },
  regeneration: {
    nom: 'Larme de ghast',
    description: 'Régénère lentement la santé.',
    icone: 'item/ghast_tear',
    niveaux: rangs(5, { description: '+0,15 PV par seconde', modif: { regeneration: 0.15 } }),
  },
  aimant: {
    nom: "Œil de l'Ender",
    description: "Attire les orbes d'expérience de plus loin.",
    icone: 'item/ender_eye',
    niveaux: rangs(5, { description: 'Rayon de collecte +30 %', modif: { collecte: 0.3 } }),
  },
} satisfies Record<string, DefinitionPassif>;

export type IdPassif = keyof typeof definitions;

export const PASSIFS: Readonly<Record<IdPassif, DefinitionPassif>> = definitions;

export const IDS_PASSIFS = Object.keys(definitions) as IdPassif[];
