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
    niveaux: rangs(1, { description: '+1 projectile', modif: { projectiles: 1 } }),
  },
  silex: {
    nom: 'Silex taillé',
    description: 'Des coups critiques plus fréquents.',
    icone: 'item/flint',
    niveaux: rangs(5, { description: 'Coups critiques +5 %', modif: { critique: 0.05 } }),
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

  // --- Exclusifs aux personnages (voir personnages.ts) -----------------------
  cuirasse: {
    nom: 'Cuirasse en netherite',
    description: 'Guerrier : une armure presque impénétrable.',
    icone: 'item/netherite_chestplate',
    exclusif: true,
    niveaux: rangs(3, { description: 'Dégâts reçus −2', modif: { armure: 2 } }),
  },
  totem: {
    nom: "Totem d'immortalité",
    description: 'Guerrier : à sa mort, le totem le ramène à la vie (puis disparaît).',
    icone: 'item/totem_of_undying',
    exclusif: true,
    consommable: { reserveMax: 2, libelle: 'Totems en réserve' },
    niveaux: rangs(1, { description: '+1 vie : vous ressuscitez avec la moitié de vos PV', modif: { vies: 1 } }),
  },
  fleches_spectrales: {
    nom: 'Flèches spectrales',
    description: 'Archère : des tirs en plus, qui portent plus loin.',
    icone: 'item/spectral_arrow',
    exclusif: true,
    niveaux: rangs(2, { description: '+1 projectile, portée +10 %', modif: { projectiles: 1, portee: 0.1 } }),
  },
  patte_de_lapin: {
    nom: 'Patte de lapin',
    description: 'Archère : plus vive, elle ramasse l\'expérience de plus loin.',
    icone: 'item/rabbit_foot',
    exclusif: true,
    niveaux: rangs(3, { description: 'Vitesse +12 %, collecte +20 %', modif: { vitesse: 0.12, collecte: 0.2 } }),
  },
  meute: {
    nom: 'Appel de la meute',
    description: 'Invocateur : chaque invocation amène un allié de plus.',
    icone: 'item/lead',
    exclusif: true,
    niveaux: rangs(2, { description: '+1 allié par invocation', modif: { allies: 1 } }),
  },
  pacte: {
    nom: "Étoile du Nether",
    description: 'Invocateur : des alliés plus forts, invoqués plus souvent.',
    icone: 'item/nether_star',
    exclusif: true,
    niveaux: rangs(3, { description: 'Dégâts +10 %, délai −5 %', modif: { degats: 0.1, cadence: -0.05 } }),
  },
  grimoire: {
    nom: 'Grimoire enchanté',
    description: 'Mage : des sorts plus vastes et plus puissants.',
    icone: 'item/enchanted_book',
    exclusif: true,
    niveaux: rangs(3, { description: 'Zone +15 %, dégâts +10 %', modif: { zone: 0.15, degats: 0.1 } }),
  },
  cristal: {
    nom: "Éclat d'améthyste",
    description: 'Mage : les sorts se rechargent plus vite.',
    icone: 'item/amethyst_shard',
    exclusif: true,
    niveaux: rangs(3, { description: 'Délai des armes −10 %', modif: { cadence: -0.1 } }),
  },
} satisfies Record<string, DefinitionPassif>;

export type IdPassif = keyof typeof definitions;

export const PASSIFS: Readonly<Record<IdPassif, DefinitionPassif>> = definitions;

export const IDS_PASSIFS = Object.keys(definitions) as IdPassif[];
