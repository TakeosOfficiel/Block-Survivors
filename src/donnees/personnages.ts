/**
 * Personnages jouables. Ajouter un personnage = ajouter une entrée.
 * Les cartes exclusives sont des bonus passifs marqués `exclusif` (passifs.ts).
 * Chaque classe a son arsenal : quand il est complet, les cartes ne proposent
 * que ses attaques (les bonus passifs restent communs).
 */
import type { IdArme } from './armes';
import type { DefinitionPersonnage } from './types';

const HUMAIN_SECOURS = {
  tete: 0xa8805e, corps: 0x2fa8a8, brasDroit: 0xa8805e, brasGauche: 0xa8805e, jambeDroite: 0x3b3b9c, jambeGauche: 0x3b3b9c,
};

const definitions = {
  guerrier: {
    nom: 'Guerrier',
    role: 'Mêlée, robuste',
    objetEnMain: 'item/iron_sword',
    description: "Robuste et cuirassé, il taille la horde à l'épée.",
    modele: 'humanoide',
    skin: 'entity/player/wide/steve',
    couleursSecours: HUMAIN_SECOURS,
    rayonOmbre: 0.5,
    pv: 24,
    vitesse: 4.8,
    armesDepart: ['epee'],
    arsenal: ['epee', 'haches', 'trident', 'onde_choc', 'lame_colossale', 'tourbillon_lames', 'heurt_bouclier'],
    arsenalComplet: true,
    bonus: { armure: 1 },
    cartesExclusives: ['cuirasse', 'totem'],
  },
  archer: {
    nom: 'Archère',
    role: 'Rapide, à distance',
    objetEnMain: 'item/bow',
    description: 'Rapide et précise, elle tire de loin.',
    modele: 'humanoide_fin',
    skin: 'entity/player/slim/alex',
    couleursSecours: { ...HUMAIN_SECOURS, corps: 0x5f8f3a, jambeDroite: 0x5a4030, jambeGauche: 0x5a4030 },
    rayonOmbre: 0.5,
    pv: 18,
    vitesse: 5.6,
    armesDepart: ['arc'],
    arsenal: ['arc', 'salve', 'pluie_fleches', 'fleche_ricochet', 'fusee', 'piege_ronces'],
    arsenalComplet: true,
    bonus: { portee: 0.15 },
    cartesExclusives: ['fleches_spectrales', 'patte_de_lapin'],
  },
  mage: {
    nom: 'Mage',
    role: 'Zones et éléments',
    objetEnMain: 'item/blaze_rod',
    description: 'Fragile, mais ses sorts couvrent de larges zones.',
    modele: 'humanoide',
    skin: 'entity/player/wide/zuri',
    couleursSecours: { ...HUMAIN_SECOURS, corps: 0x7a3fa0 },
    rayonOmbre: 0.5,
    pv: 16,
    vitesse: 5,
    armesDepart: ['boules_feu'],
    arsenal: ['boules_feu', 'eclairs', 'tornade_feu', 'nova_glace', 'meteore', 'orbes_arcaniques'],
    arsenalComplet: true,
    bonus: { zone: 0.15, degats: 0.1 },
    cartesExclusives: ['grimoire', 'cristal'],
  },
  invocateur: {
    nom: 'Invocateur',
    role: 'Alliés et totems',
    objetEnMain: 'item/bone',
    description: 'Fragile seul, il commande des loups, des vex et des golems.',
    modele: 'humanoide',
    skin: 'entity/player/wide/efe',
    couleursSecours: { ...HUMAIN_SECOURS, corps: 0x4a6a8a },
    rayonOmbre: 0.5,
    pv: 18,
    vitesse: 5.1,
    armesDepart: ['loups'],
    arsenal: ['loups', 'golems', 'vex', 'esprit_blaze', 'totem_racines', 'esprit_protecteur'],
    arsenalComplet: true,
    bonus: { regeneration: 0.05 },
    cartesExclusives: ['meute', 'pacte'],
  },
} satisfies Record<string, DefinitionPersonnage>;

export type IdPersonnage = keyof typeof definitions;

export const PERSONNAGES: Readonly<Record<IdPersonnage, DefinitionPersonnage>> = definitions;

export const IDS_PERSONNAGES = Object.keys(definitions) as IdPersonnage[];

export const PERSONNAGE_PAR_DEFAUT: IdPersonnage = 'guerrier';

/** Armes réservées aux classes dont l'arsenal est complet. */
const ARMES_RESERVEES = new Set<IdArme>(
  IDS_PERSONNAGES.filter((id) => PERSONNAGES[id].arsenalComplet).flatMap((id) => PERSONNAGES[id].arsenal),
);

/** L'arme peut-elle être proposée en carte à ce personnage ? */
export function armeProposable(personnage: DefinitionPersonnage, arme: IdArme): boolean {
  return personnage.arsenal.includes(arme) || (!personnage.arsenalComplet && !ARMES_RESERVEES.has(arme));
}
