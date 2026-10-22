/**
 * Progression entre les parties : maîtrise de chaque classe (l'XP ramassée en
 * partie la fait monter, chaque niveau donne un point de talent), arbres de
 * talents (deux par classe, un seul actif par partie ; chacun a tous les
 * points de la classe) et déblocages des personnages et des arènes.
 *
 * Chaque arbre est un graphe : un talent relié s'achète quand un des talents
 * reliés au-dessus de lui est complet ; un talent libre s'ouvre avec son
 * palier. Les achats restent acquis ; dans l'arbre actif, tout talent acheté
 * est actif, sauf trois réglages faits avant la partie : un seul talent final,
 * une option par talent à choix, une seule arme de départ en plus.
 */
import type { IdPersonnage } from './personnages';
import type { DefinitionArbre, DefinitionDeblocage, DefinitionMaitrise } from './types';

/** Paliers : la ligne `ligne` et les suivantes s'ouvrent avec `rangs` rangs achetés au-dessus d'elle. */
export const PALIERS: readonly { readonly ligne: number; readonly rangs: number }[] = [
  { ligne: 4, rangs: 8 },
  { ligne: 6, rangs: 18 },
];

/** Mécaniques propres déjà en jeu ; les autres sont annoncées « en préparation » sur l'écran des talents. */
export const MECANIQUES_PRETES: ReadonlySet<string> = new Set<string>([
  // Guerrier
  'enchainement', 'frenesie', 'bourreau', 'elan_critique', 'saignee', 'lame_lourde', 'soif_combat', 'fureur',
  'fureur_gladiateur', 'champion_arene', 'execution', 'posture_defensive', 'epines', 'rempart', 'inebranlable',
  'ronces_acier', 'contre_coup', 'peau_de_fer', 'tenir_la_ligne', 'repliques', 'contre_attaque_bastion',
  'dernier_rempart', 'seisme',
  // Archère
  'visee_stable', 'premier_sang', 'perforation_nette', 'tir_groupe', 'tir_rebondissant', 'marque_chasseur',
  'concentration', 'point_faible', 'puissance', 'oeil_aigle', 'executrice', 'pas_chasseur', 'venin',
  'venin_concentre', 'venin_virulent', 'ronces_collantes', 'contagion', 'course_chargee', 'piege_a_loups',
  'second_vent', 'ronces_venimeuses', 'nuage_toxique', 'vent_arriere',
  // Mage
  'coeur_blaze', 'cendres_protectrices', 'lave_fusion', 'propagation', 'brasier', 'fournaise', 'tempete_feu',
  'embrasement', 'phenix', 'pluie_feu', 'resonance', 'haute_tension', 'orbes_givrees', 'conduction',
  'flux_arcanique', 'surcharge', 'acceleration', 'tempete_orage', 'echo_arcanique', 'zero_absolu',
  // Invocateur
  'chef_meute', 'morsure_marquante', 'reactivite', 'meute_soudee', 'traque', 'curee', 'hurlement', 'encerclement',
  'braises_blaze', 'coordination', 'alpha', 'ruee_meute', 'lune_de_sang', 'gardien_alpha', 'lien_spirituel',
  'garde_rapprochee', 'allay_chanteur', 'essaim', 'evocation_superieure', 'crocs_evocateur', 'bosquet',
]);

export const ARBRES: Readonly<Record<IdPersonnage, readonly [DefinitionArbre, DefinitionArbre]>> = {
  guerrier: [
    {
      id: 'gladiateur',
      nom: 'Gladiateur',
      description: 'Frapper fort, frapper juste.',
      icone: 'item/diamond_sword',
      branches: { tronc: 'Tronc', gauche: 'Puissance', centre: 'Enchaînements', droite: 'Précision', libre: 'Tourbillon' },
      noeuds: [
        { id: 'enchainement', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Enchaînement', icone: 'item/iron_sword', effet: "Un coup d'épée qui touche 3 ennemis ou plus réduit de 15 % le délai du coup suivant.", rangMax: 1, mecanique: 'enchainement' },
        { id: 'lame', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Lame affûtée', icone: 'item/flint', effet: 'Dégâts de mêlée +4 / 8 / 12 %.', rangMax: 3, prerequis: ['enchainement'], modif: { degatsMelee: 0.04 } },
        { id: 'precis', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Coup précis', icone: 'item/diamond', effet: 'Coups critiques +3 / 6 / 9 %.', rangMax: 3, prerequis: ['enchainement'], modif: { critique: 0.03 } },
        { id: 'maitre', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: "Maître d'armes", icone: 'item/diamond_sword', effet: "L'Épée en fer commence avec +1 / +2 rangs.", rangMax: 2, prerequis: ['lame'], rangsArme: 'epee' },
        { id: 'frenesie', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Frénésie', icone: 'item/redstone', effet: '10 éliminations en 3 s : délai de mêlée −15 / 25 % pendant 4 s, et +1 PV.', rangMax: 2, prerequis: ['lame', 'precis'], mecanique: 'frenesie' },
        { id: 'bourreau', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Œil du bourreau', icone: 'item/netherite_sword', effet: 'Critiques +20 / 35 % de dégâts contre un ennemi sous 30 % de ses PV.', rangMax: 2, prerequis: ['precis'], mecanique: 'bourreau' },
        { id: 'allonge', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Allonge', icone: 'item/iron_axe', effet: 'Rayon des attaques de mêlée +8 / 16 %.', rangMax: 2, prerequis: ['maitre'], modif: { zoneMelee: 0.08 } },
        { id: 'elan', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Élan critique / Saignée', icone: 'item/golden_sword', effet: 'Au choix : Élan critique ou Saignée.', rangMax: 1, prerequis: ['frenesie'], options: [
        { nom: 'Élan critique', icone: 'item/golden_sword', effet: 'Un critique renforce le prochain balayage : +40 % de zone et +30 % de dégâts (délai interne 1,5 s).', mecanique: 'elan_critique' },
        { nom: 'Saignée', icone: 'item/red_dye', effet: 'Les critiques de mêlée font saigner : 4 dégâts/s pendant 3 s (3 cumuls).', mecanique: 'saignee' },
      ] },
        { id: 'sangfroid', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Sang-froid', icone: 'item/compass_00', effet: '+1 relance des cartes par partie.', rangMax: 1, prerequis: ['bourreau'], relances: 1 },
        { id: 'tourbillon', ligne: 4, colonne: 0, type: 'capacite', branche: 'libre', nom: 'Tourbillon', icone: 'item/golden_axe', effet: 'Commence la partie avec le Tourbillon de lames.', rangMax: 1, libre: true, armeDepart: 'tourbillon_lames' },
        { id: 'lourde', ligne: 4, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Lame lourde', icone: 'block/anvil', effet: 'Recul +20 / 40 %. Un ennemi projeté contre un mur subit 10 / 20 dégâts.', rangMax: 2, prerequis: ['allonge'], modif: { recul: 0.2 }, mecanique: 'lame_lourde' },
        { id: 'ecraseur', ligne: 4, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Écraseur', icone: 'item/mace', effet: 'Lame colossale et Onde de choc : dégâts +10 / 20 %.', rangMax: 2, prerequis: ['elan'], armes: [{ arme: 'lame_colossale', part: { degats: 0.1 } }, { arme: 'onde_choc', part: { degats: 0.1 } }] },
        { id: 'danse', ligne: 5, colonne: 0, type: 'bonus', branche: 'libre', nom: 'Danse des lames', icone: 'item/stone_sword', effet: 'Tourbillon de lames : +1 / +2 frappes par rotation.', rangMax: 2, prerequis: ['tourbillon'], armes: [{ arme: 'tourbillon_lames', ajout: { duree: 0.3 } }] },
        { id: 'soif', ligne: 5, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Soif de combat', icone: 'item/cooked_beef', effet: 'Chaque élimination en mêlée rend 0,1 / 0,2 PV (dans la limite de régénération).', rangMax: 2, prerequis: ['lourde'], mecanique: 'soif_combat' },
        { id: 'fureur', ligne: 5, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Fureur', icone: 'item/blaze_powder', effet: 'Après une Frénésie : dégâts +5 / 10 % pendant 5 s.', rangMax: 2, prerequis: ['ecraseur'], mecanique: 'fureur' },
        { id: 'f_fureur', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Fureur du gladiateur', icone: 'item/netherite_axe', effet: "Toutes les 5 attaques de mêlée qui touchent, un anneau rouge (rayon 3,5) inflige 60 % des dégâts de l'épée. Délai interne 3 s.", rangMax: 1, prerequis: ['soif'], mecanique: 'fureur_gladiateur' },
        { id: 'f_champion', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: "Champion de l'arène", icone: 'item/golden_helmet', effet: 'La Frénésie se déclenche dès 6 éliminations et dure 6 s.', rangMax: 1, prerequis: ['fureur'], mecanique: 'champion_arene' },
        { id: 'f_execution', ligne: 6, colonne: 5, type: 'final', branche: 'centre', nom: 'Exécution', icone: 'block/wither_rose', effet: "Une fois toutes les 2 s, l'épée achève un ennemi normal sous 15 % de ses PV. Contre un boss sous 15 % : +30 % de dégâts.", rangMax: 1, prerequis: ['fureur'], mecanique: 'execution' },
      ],
    },
    {
      id: 'gardien',
      nom: 'Gardien',
      description: 'Encaisser, tenir, survivre.',
      icone: 'item/iron_chestplate',
      branches: { tronc: 'Tronc', gauche: 'Épines', centre: 'Bouclier', droite: 'Soins', libre: 'Séisme' },
      noeuds: [
        { id: 'posture', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Posture défensive', icone: 'block/iron_block', effet: 'Quand il est touché, le Guerrier repousse les ennemis proches (délai interne 3 s).', rangMax: 1, mecanique: 'posture_defensive' },
        { id: 'robustesse', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Robustesse', icone: 'item/apple', effet: '+1 cœur par rang.', rangMax: 3, prerequis: ['posture'], modif: { pvMax: 2 } },
        { id: 'souffle', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Second souffle', icone: 'item/ghast_tear', effet: 'Régénération +0,5 % des PV max par seconde, par rang.', rangMax: 3, prerequis: ['posture'], modif: { regenerationPart: 0.005 } },
        { id: 'plates', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Plates de fer', icone: 'item/iron_chestplate', effet: 'Armure +1 / +2.', rangMax: 2, prerequis: ['robustesse'], modif: { armure: 1 } },
        { id: 'bouclier', ligne: 2, colonne: 3, type: 'capacite', branche: 'centre', nom: 'Bouclier lourd', icone: 'item/iron_door', effet: 'Commence avec le Heurt de bouclier (rang 1, puis rang 2).', rangMax: 2, prerequis: ['robustesse', 'souffle'], armeDepart: 'heurt_bouclier' },
        { id: 'pansements', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Pansements', icone: 'item/paper', effet: 'Les pommes ramassées soignent +25 / 50 %.', rangMax: 2, prerequis: ['souffle'], modif: { soinsObjets: 0.25 } },
        { id: 'epines', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Épines', icone: 'block/cactus_side', effet: 'Quand il est touché, des pointes infligent 10 / 18 dégâts autour de lui (rayon 2,5, délai interne 2 s).', rangMax: 2, prerequis: ['plates'], mecanique: 'epines' },
        { id: 'rempart', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Rempart / Inébranlable', icone: 'block/stone_bricks', effet: 'Au choix : Rempart ou Inébranlable.', rangMax: 1, prerequis: ['bouclier'], options: [
        { nom: 'Rempart', icone: 'block/stone_bricks', effet: 'Toutes les 12 s, un bouclier absorbe 4 PV.', mecanique: 'rempart' },
        { nom: 'Inébranlable', icone: 'block/obsidian', effet: 'Insensible au recul ; dégâts subis des boss −10 %.', mecanique: 'inebranlable' },
      ] },
        { id: 'vigilance', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Vigilance', icone: 'item/spyglass', effet: "Rayon de collecte de l'XP +20 %.", rangMax: 1, prerequis: ['pansements'], modif: { collecte: 0.2 } },
        { id: 'ronces', ligne: 4, colonne: 1, type: 'bonus', branche: 'gauche', nom: "Ronces d'acier", icone: 'block/iron_bars', effet: 'Épines : rayon +0,5 / 1 bloc, et elles ralentissent de 30 % pendant 1 s.', rangMax: 2, prerequis: ['epines'], mecanique: 'ronces_acier' },
        { id: 'contrecoup', ligne: 4, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Contre-coup', icone: 'item/iron_ingot', effet: 'Heurt de bouclier : dégâts +20 / 40 % ; il renvoie les projectiles dès le rang 1.', rangMax: 2, prerequis: ['rempart'], armes: [{ arme: 'heurt_bouclier', part: { degats: 0.2 } }], mecanique: 'contre_coup' },
        { id: 'sismique', ligne: 4, colonne: 5, type: 'capacite', branche: 'libre', nom: 'Sismique', icone: 'block/cracked_stone_bricks', effet: "Commence la partie avec l'Onde de choc.", rangMax: 1, libre: true, armeDepart: 'onde_choc' },
        { id: 'peau', ligne: 5, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Peau de fer', icone: 'item/iron_helmet', effet: 'Sous 50 % de PV : armure +1 / +2.', rangMax: 2, prerequis: ['ronces'], mecanique: 'peau_de_fer' },
        { id: 'ligne', ligne: 5, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Tenir la ligne', icone: 'item/brick', effet: 'Immobile depuis 1 s : dégâts subis −10 / 20 %.', rangMax: 2, prerequis: ['contrecoup'], mecanique: 'tenir_la_ligne' },
        { id: 'repliques', ligne: 5, colonne: 5, type: 'bonus', branche: 'libre', nom: 'Répliques', icone: 'item/clay_ball', effet: "L'Onde de choc laisse une 2e fissure 0,5 s après, à 40 / 60 % des dégâts.", rangMax: 2, prerequis: ['sismique'], mecanique: 'repliques' },
        { id: 'f_bastion', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Contre-attaque du bastion', icone: 'item/netherite_chestplate', effet: '30 dégâts bloqués ou absorbés chargent une frappe au sol (rayon 4) : 40 dégâts + les dégâts absorbés (80 au maximum). Délai interne 6 s.', rangMax: 1, prerequis: ['peau'], mecanique: 'contre_attaque_bastion' },
        { id: 'f_dernier', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: 'Dernier rempart', icone: 'item/totem_of_undying', effet: 'Une fois par partie, sous 25 % de PV : invulnérable 3 s, et repousse tout autour.', rangMax: 1, prerequis: ['ligne'], mecanique: 'dernier_rempart' },
        { id: 'f_seisme', ligne: 6, colonne: 5, type: 'final', branche: 'libre', nom: 'Séisme', icone: 'block/chiseled_stone_bricks', effet: 'Toutes les 3 Ondes de choc, la suivante part dans 6 directions.', rangMax: 1, prerequis: ['repliques'], mecanique: 'seisme' },
      ],
    },
  ],
  archer: [
    {
      id: 'elite',
      nom: "Tireuse d'élite",
      description: 'Chaque flèche compte.',
      icone: 'item/spectral_arrow',
      branches: { tronc: 'Tronc', gauche: 'Perforation', centre: 'Marque', droite: 'Critiques', libre: 'Fusées' },
      noeuds: [
        { id: 'stable', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Visée stable', icone: 'item/bow', effet: "Les flèches de l'Arc qui touchent à plus de 8 blocs font +15 % de dégâts.", rangMax: 1, mecanique: 'visee_stable' },
        { id: 'pointes', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Pointes de fer', icone: 'item/arrow', effet: 'Dégâts des projectiles +4 / 8 / 12 %.', rangMax: 3, prerequis: ['stable'], modif: { degatsProjectiles: 0.04 } },
        { id: 'faucon', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Œil de faucon', icone: 'item/spyglass', effet: 'Portée +5 / 10 / 15 % et vitesse des flèches +10 / 20 / 30 %.', rangMax: 3, prerequis: ['stable'], modif: { portee: 0.05, vitesseProjectiles: 0.1 } },
        { id: 'arc', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Arc enchanté', icone: 'item/enchanted_book', effet: "L'Arc commence avec +1 / +2 rangs.", rangMax: 2, prerequis: ['pointes'], rangsArme: 'arc' },
        { id: 'sang', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Premier sang', icone: 'item/red_dye', effet: '+20 / 35 % de dégâts contre un ennemi à plus de 90 % de ses PV.', rangMax: 2, prerequis: ['pointes', 'faucon'], mecanique: 'premier_sang' },
        { id: 'visee', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Visée', icone: 'item/flint', effet: 'Coups critiques +4 / 8 %.', rangMax: 2, prerequis: ['faucon'], modif: { critique: 0.04 } },
        { id: 'perfo', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Perforation nette', icone: 'item/spectral_arrow', effet: 'Une flèche perd 12 %, puis 0 %, de ses dégâts par ennemi traversé (au lieu de 25 %). Rang 2 : +1 perforation.', rangMax: 2, prerequis: ['arc'], mecanique: 'perforation_nette' },
        { id: 'tir', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Tir groupé / Tir rebondissant', icone: 'item/crossbow_standby', effet: 'Au choix : Tir groupé ou Tir rebondissant.', rangMax: 1, prerequis: ['sang'], options: [
        { nom: 'Tir groupé', icone: 'item/crossbow_standby', effet: 'Salve en éventail : cône 30 % plus serré et +2 flèches.', armes: [{ arme: 'salve', ajout: { nombre: 2 } }], mecanique: 'tir_groupe' },
        { nom: 'Tir rebondissant', icone: 'item/amethyst_shard', effet: 'Flèche ricocheuse : +1 rebond, et les rebonds portent 2 blocs plus loin.', armes: [{ arme: 'fleche_ricochet', ajout: { perforation: 1 } }], mecanique: 'tir_rebondissant' },
      ] },
        { id: 'tete', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'En pleine tête', icone: 'block/target_top', effet: 'Dégâts critiques +0,25.', rangMax: 1, prerequis: ['visee'], modif: { degatsCritiques: 0.25 } },
        { id: 'artificiere', ligne: 4, colonne: 0, type: 'capacite', branche: 'libre', nom: 'Artificière', icone: 'item/firework_rocket', effet: 'Commence la partie avec la Fusée explosive.', rangMax: 1, libre: true, armeDepart: 'fusee' },
        { id: 'lourdes', ligne: 4, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Flèches lourdes', icone: 'item/iron_nugget', effet: 'Recul des flèches +50 / 100 %.', rangMax: 2, prerequis: ['perfo'], modif: { recul: 0.5 } },
        { id: 'marque', ligne: 4, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Marque du chasseur', icone: 'item/glow_ink_sac', effet: "Un critique marque l'ennemi 4 s : il subit +10 / 20 % de dégâts de l'Archère.", rangMax: 2, prerequis: ['tir'], mecanique: 'marque_chasseur' },
        { id: 'poudre', ligne: 5, colonne: 0, type: 'bonus', branche: 'libre', nom: 'Poudre à canon', icone: 'item/gunpowder', effet: 'Rayon de la Fusée explosive +15 / 30 %.', rangMax: 2, prerequis: ['artificiere'], armes: [{ arme: 'fusee', part: { zone: 0.15 } }] },
        { id: 'concentration', ligne: 5, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Concentration', icone: 'item/clock_00', effet: 'Immobile depuis 1 s : délai des tirs −8 / 15 %.', rangMax: 2, prerequis: ['lourdes'], mecanique: 'concentration' },
        { id: 'faible', ligne: 5, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Point faible', icone: 'block/target_side', effet: 'Un boss marqué subit encore +10 / 15 %.', rangMax: 2, prerequis: ['marque'], mecanique: 'point_faible' },
        { id: 'f_puissance', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Puissance', icone: 'item/end_crystal', effet: "Tous les 12 tirs, la prochaine flèche de l'Arc devient une flèche spectrale géante qui traverse tout sur 30 blocs (3 × les dégâts de l'Arc + 20). Délai interne 4 s.", rangMax: 1, prerequis: ['concentration'], mecanique: 'puissance' },
        { id: 'f_aigle', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: "Œil de l'aigle", icone: 'item/ender_eye', effet: "Les critiques ricochent une fois vers l'ennemi le plus éloigné, à 100 % des dégâts.", rangMax: 1, prerequis: ['faible'], mecanique: 'oeil_aigle' },
        { id: 'f_executrice', ligne: 6, colonne: 5, type: 'final', branche: 'centre', nom: 'Exécutrice', icone: 'block/wither_rose', effet: 'Les flèches achèvent les ennemis normaux sous 10 % de leurs PV. Contre un boss, chaque critique ajoute +2 % de dégâts contre lui (10 cumuls, 20 % au maximum) ; les cumuls durent 6 s et se rafraîchissent à chaque critique.', rangMax: 1, prerequis: ['faible'], mecanique: 'executrice' },
      ],
    },
    {
      id: 'rodeuse',
      nom: 'Rôdeuse',
      description: 'Rapide, insaisissable, prévoyante.',
      icone: 'item/rabbit_foot',
      branches: { tronc: 'Tronc', gauche: 'Pièges', centre: 'Poison', droite: 'Course', libre: 'Pluie' },
      noeuds: [
        { id: 'chasseur', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Pas du chasseur', icone: 'item/rabbit_foot', effet: 'Après 2 s de déplacement continu : vitesse +10 % (aucun bonus de dégâts).', rangMax: 1, mecanique: 'pas_chasseur' },
        { id: 'pas', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Pas légers', icone: 'item/sugar', effet: 'Vitesse +3 / 6 / 9 %.', rangMax: 3, prerequis: ['chasseur'], modif: { vitesse: 0.03 } },
        { id: 'flair', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Flair', icone: 'item/ender_pearl', effet: "Rayon de collecte de l'XP +10 / 20 / 30 %.", rangMax: 3, prerequis: ['chasseur'], modif: { collecte: 0.1 } },
        { id: 'ronces', ligne: 2, colonne: 1, type: 'capacite', branche: 'gauche', nom: 'Ronces aiguisées', icone: 'item/sweet_berries', effet: 'Commence la partie avec le Piège de ronces.', rangMax: 1, prerequis: ['pas'], armeDepart: 'piege_ronces' },
        { id: 'venin', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Venin', icone: 'item/spider_eye', effet: '20 / 35 % des flèches empoisonnent : 2 dégâts/s pendant 4 s (5 cumuls).', rangMax: 2, prerequis: ['pas', 'flair'], mecanique: 'venin' },
        { id: 'instinct', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Instinct', icone: 'item/compass_00', effet: '+1 relance des cartes par partie.', rangMax: 1, prerequis: ['flair'], relances: 1 },
        { id: 'buissons', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Buissons épais', icone: 'block/sweet_berry_bush_stage3', effet: '+1 piège en même temps ; dégâts des pièges +15 / 30 %.', rangMax: 2, prerequis: ['ronces'], armes: [{ arme: 'piege_ronces', ajout: { nombre: 1 }, part: { degats: 0.15 } }] },
        { id: 'virulence', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Venin concentré / Venin virulent', icone: 'item/fermented_spider_eye', effet: 'Au choix : Venin concentré ou Venin virulent.', rangMax: 1, prerequis: ['venin'], options: [
        { nom: 'Venin concentré', icone: 'item/fermented_spider_eye', effet: 'Poison +50 % de dégâts, mais 3 cumuls au maximum.', mecanique: 'venin_concentre' },
        { nom: 'Venin virulent', icone: 'item/poisonous_potato', effet: '8 cumuls au maximum, mais −20 % de dégâts par cumul.', mecanique: 'venin_virulent' },
      ] },
        { id: 'herboriste', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Herboriste', icone: 'item/glow_berries', effet: 'Régénération +0,75 % des PV max par seconde.', rangMax: 1, prerequis: ['instinct'], modif: { regenerationPart: 0.0075 } },
        { id: 'collantes', ligne: 4, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Ronces collantes', icone: 'item/string', effet: 'En entrant dans un piège, un ennemi est enraciné 0,5 / 1 s (un boss est seulement ralenti).', rangMax: 2, prerequis: ['buissons'], mecanique: 'ronces_collantes' },
        { id: 'contagion', ligne: 4, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Contagion', icone: 'item/slime_ball', effet: 'Un ennemi empoisonné qui meurt transmet 1 / 2 cumuls à un voisin.', rangMax: 2, prerequis: ['virulence'], mecanique: 'contagion' },
        { id: 'course', ligne: 4, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Course chargée', icone: 'item/feather', effet: 'Chaque 1,5 bloc parcouru donne une charge (10 au maximum). À 10 charges, la prochaine volée tire +3 flèches avec +15 / 30 % de dégâts.', rangMax: 2, prerequis: ['herboriste'], mecanique: 'course_chargee' },
        { id: 'loups', ligne: 5, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Piège à loups', icone: 'block/tripwire_hook', effet: "Un ennemi qui sort d'un piège reste ralenti 1 / 2 s.", rangMax: 2, prerequis: ['collantes'], mecanique: 'piege_a_loups' },
        { id: 'toxines', ligne: 5, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Toxines', icone: 'item/pufferfish', effet: 'Dégâts du poison +10 / 20 %.', rangMax: 2, prerequis: ['contagion'], modif: { degatsPoison: 0.1 } },
        { id: 'vent', ligne: 5, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Second vent', icone: 'item/phantom_membrane', effet: 'La Course chargée se remplit à 7 charges au lieu de 10.', rangMax: 1, prerequis: ['course'], mecanique: 'second_vent' },
        { id: 'pluie', ligne: 5, colonne: 6, type: 'capacite', branche: 'libre', nom: 'Averse', icone: 'item/arrow', effet: 'Commence la partie avec la Pluie de flèches.', rangMax: 1, libre: true, armeDepart: 'pluie_fleches' },
        { id: 'f_ronces', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Ronces venimeuses', icone: 'item/suspicious_stew', effet: 'Les ennemis piégés subissent +30 % de dégâts des projectiles, et leur poison se propage aux autres ennemis du piège (une fois par seconde).', rangMax: 1, prerequis: ['loups'], mecanique: 'ronces_venimeuses' },
        { id: 'f_nuage', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: 'Nuage toxique', icone: 'item/lingering_potion', effet: 'Un ennemi qui meurt avec son maximum de cumuls de poison (3, 5 ou 8 selon Venin) laisse un nuage de 2 s (rayon 2) : 1 cumul de poison toutes les 0,5 s aux ennemis dedans. Délai interne 1 s, 3 nuages au maximum.', rangMax: 1, prerequis: ['toxines'], mecanique: 'nuage_toxique' },
        { id: 'f_vent', ligne: 6, colonne: 5, type: 'final', branche: 'droite', nom: 'Vent arrière', icone: 'item/elytra', effet: 'Quand la Course chargée est pleine, la volée part aussi vers l’arrière.', rangMax: 1, prerequis: ['vent'], mecanique: 'vent_arriere' },
      ],
    },
  ],
  mage: [
    {
      id: 'pyromancien',
      nom: 'Pyromancien',
      description: 'Tout brûle.',
      icone: 'item/fire_charge',
      branches: { tronc: 'Tronc', gauche: 'Cendres', centre: 'Propagation', droite: 'Météores', libre: 'Tornade' },
      noeuds: [
        { id: 'coeur', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Cœur de blaze', icone: 'item/blaze_rod', effet: 'Les Boules de feu enflamment : 1 cumul de brûlure (3 dégâts/s pendant 3 s).', rangMax: 1, mecanique: 'coeur_blaze' },
        { id: 'braises', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Braises', icone: 'item/blaze_powder', effet: 'Dégâts des armes de feu et des brûlures +4 / 8 / 12 %.', rangMax: 3, prerequis: ['coeur'], modif: { degatsFeu: 0.04, degatsBrulure: 0.04 } },
        { id: 'souffle', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Souffle ardent', icone: 'item/glowstone_dust', effet: 'Zone +3 / 6 / 9 %.', rangMax: 3, prerequis: ['coeur'], modif: { zone: 0.03 } },
        { id: 'peau', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Peau de magma', icone: 'item/nether_wart', effet: '+1 cœur par rang.', rangMax: 2, prerequis: ['braises'], modif: { pvMax: 2 } },
        { id: 'charges', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Charges de feu', icone: 'item/fire_charge', effet: 'Les Boules de feu commencent avec +1 / +2 rangs.', rangMax: 2, prerequis: ['braises', 'souffle'], rangsArme: 'boules_feu' },
        { id: 'meteore', ligne: 2, colonne: 5, type: 'capacite', branche: 'droite', nom: 'Météore', icone: 'block/magma', effet: 'Commence la partie avec le Météore.', rangMax: 1, prerequis: ['souffle'], armeDepart: 'meteore' },
        { id: 'cendres', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Cendres protectrices', icone: 'item/charcoal', effet: 'Tuer un ennemi en feu donne 1 PV de bouclier (une fois par seconde, dans la limite des boucliers).', rangMax: 1, prerequis: ['peau'], mecanique: 'cendres_protectrices' },
        { id: 'combustion', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Combustion vive / Combustion lente', icone: 'item/magma_cream', effet: 'Au choix : Combustion vive ou Combustion lente.', rangMax: 1, prerequis: ['charges'], options: [
        { nom: 'Combustion vive', icone: 'item/magma_cream', effet: 'Brûlure : 4 cumuls au maximum au lieu de 3.', modif: { cumulsBrulure: 1 } },
        { nom: 'Combustion lente', icone: 'item/clock_00', effet: 'Les brûlures durent 50 % plus longtemps.', modif: { dureeBrulure: 0.5 } },
      ] },
        { id: 'lave', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Lave en fusion', icone: 'item/lava_bucket', effet: 'Feu au sol du Météore : +2 s et +20 % de dégâts.', rangMax: 1, prerequis: ['meteore'], armes: [{ arme: 'meteore', ajout: { duree: 2 } }], mecanique: 'lave_fusion' },
        { id: 'cratere', ligne: 3, colonne: 6, type: 'bonus', branche: 'droite', nom: 'Cratère', icone: 'item/gunpowder', effet: "Rayon d'impact du Météore +20 %.", rangMax: 1, prerequis: ['meteore'], armes: [{ arme: 'meteore', part: { zone: 0.2 } }] },
        { id: 'tornade', ligne: 4, colonne: 0, type: 'capacite', branche: 'libre', nom: 'Tornade de feu', icone: 'item/wind_charge', effet: 'Commence la partie avec la Tornade de feu.', rangMax: 1, libre: true, armeDepart: 'tornade_feu' },
        { id: 'propagation', ligne: 4, colonne: 2, type: 'bonus', branche: 'centre', nom: 'Propagation', icone: 'item/flint_and_steel', effet: 'Un ennemi qui meurt en brûlant transmet 50 / 100 % de ses brûlures à 1 / 2 voisins. Chaque transmission perd la moitié des cumuls.', rangMax: 2, prerequis: ['combustion'], mecanique: 'propagation' },
        { id: 'brasier', ligne: 4, colonne: 4, type: 'choix', branche: 'centre', nom: 'Brasier / Fournaise', icone: 'item/soul_campfire', effet: 'Au choix : Brasier ou Fournaise.', rangMax: 1, prerequis: ['combustion'], options: [
        { nom: 'Brasier', icone: 'item/soul_campfire', effet: 'Dégâts critiques +0,3 contre les ennemis en feu.', mecanique: 'brasier' },
        { nom: 'Fournaise', icone: 'block/furnace_front_on', effet: 'Coups critiques +8 % contre les ennemis en feu.', mecanique: 'fournaise' },
      ] },
        { id: 'vortex', ligne: 5, colonne: 0, type: 'bonus', branche: 'libre', nom: 'Vortex', icone: 'item/breeze_rod', effet: 'La Tornade aspire les ennemis 30 / 60 % plus fort.', rangMax: 2, prerequis: ['tornade'], armes: [{ arme: 'tornade_feu', part: { recul: 0.3 } }] },
        { id: 'infernale', ligne: 5, colonne: 2, type: 'bonus', branche: 'centre', nom: 'Chaleur infernale', icone: 'item/netherite_scrap', effet: 'Dégâts des brûlures +10 / 20 %.', rangMax: 2, prerequis: ['propagation'], modif: { degatsBrulure: 0.1 } },
        { id: 'ardent', ligne: 5, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Cœur ardent', icone: 'item/campfire', effet: 'Délai des armes de feu −4 / 8 %.', rangMax: 2, prerequis: ['brasier'], modif: { cadenceFeu: -0.04 } },
        { id: 'tempete', ligne: 5, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Tempête de feu', icone: 'item/dragon_breath', effet: 'Un Météore qui tombe dans une Tornade la transforme en tornade de magma (+50 % de rayon, +2 s).', rangMax: 1, prerequis: ['lave'], mecanique: 'tempete_feu' },
        { id: 'f_embrasement', ligne: 6, colonne: 1, type: 'final', branche: 'centre', nom: 'Embrasement', icone: 'block/tnt_side', effet: "Quand 15 ennemis brûlent en même temps, ils explosent tous et infligent d'un coup 1,5 × leurs brûlures restantes autour d'eux. Délai interne 10 s.", rangMax: 1, prerequis: ['infernale'], mecanique: 'embrasement' },
        { id: 'f_phenix', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: 'Phénix', icone: 'item/nether_star', effet: 'Une fois par partie, à 0 PV : explosion de feu (rayon 5), puis retour à 40 % des PV.', rangMax: 1, prerequis: ['ardent'], mecanique: 'phenix' },
        { id: 'f_pluie', ligne: 6, colonne: 5, type: 'final', branche: 'droite', nom: 'Pluie de feu', icone: 'item/firework_star', effet: 'Le Météore tombe en pluie : 3 météores plus petits (60 % des dégâts chacun).', rangMax: 1, prerequis: ['tempete'], mecanique: 'pluie_feu' },
      ],
    },
    {
      id: 'arcaniste',
      nom: 'Arcaniste',
      description: 'La foudre et le savoir.',
      icone: 'item/enchanted_book',
      branches: { tronc: 'Tronc', gauche: 'Foudre', centre: 'Orbes', droite: 'Glace' },
      noeuds: [
        { id: 'resonance', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Résonance', icone: 'item/amethyst_shard', effet: 'Chaque sort différent lancé dans les 3 s donne +5 % de dégâts (15 % au maximum).', rangMax: 1, mecanique: 'resonance' },
        { id: 'concentration', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Concentration', icone: 'item/clock_00', effet: 'Délai des armes −2 / 4 / 6 %.', rangMax: 3, prerequis: ['resonance'], modif: { cadence: -0.02 } },
        { id: 'vigueur', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Vigueur', icone: 'item/glistering_melon_slice', effet: '+1 cœur par rang.', rangMax: 3, prerequis: ['resonance'], modif: { pvMax: 2 } },
        { id: 'appel', ligne: 2, colonne: 1, type: 'capacite', branche: 'gauche', nom: 'Appel de la foudre', icone: 'block/lightning_rod', effet: 'Commence avec les Éclairs (rang 1, puis rang 2).', rangMax: 2, prerequis: ['concentration'], armeDepart: 'eclairs' },
        { id: 'savoir', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Savoir', icone: 'item/book', effet: '+1 / +2 relances des cartes par partie.', rangMax: 2, prerequis: ['concentration', 'vigueur'], relances: 1 },
        { id: 'egide', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Égide', icone: 'item/experience_bottle', effet: 'Armure +1 / +2.', rangMax: 2, prerequis: ['vigueur'], modif: { armure: 1 } },
        { id: 'tension', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Haute tension', icone: 'item/copper_ingot', effet: 'Chaque éclair frappe aussi un 2e ennemi proche, à 30 / 50 %.', rangMax: 2, prerequis: ['appel'], mecanique: 'haute_tension' },
        { id: 'orbes', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Orbes renforcées / Orbes givrées', icone: 'item/ender_pearl', effet: 'Au choix : Orbes renforcées ou Orbes givrées.', rangMax: 1, prerequis: ['savoir'], options: [
        { nom: 'Orbes renforcées', icone: 'item/ender_pearl', effet: 'Orbes arcaniques : +1 orbe et +20 % de dégâts.', armes: [{ arme: 'orbes_arcaniques', ajout: { nombre: 1 }, part: { degats: 0.2 } }] },
        { nom: 'Orbes givrées', icone: 'item/snowball', effet: 'Les orbes ralentissent de 30 % pendant 1 s.', mecanique: 'orbes_givrees' },
      ] },
        { id: 'gel', ligne: 3, colonne: 5, type: 'capacite', branche: 'droite', nom: 'Gel', icone: 'block/packed_ice', effet: 'Commence la partie avec la Nova de glace.', rangMax: 1, prerequis: ['egide'], armeDepart: 'nova_glace' },
        { id: 'conduction', ligne: 4, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Conduction', icone: 'block/redstone_torch', effet: 'Un éclair qui frappe un ennemi ralenti rebondit sur 2 / 3 voisins à 60 % (sans rebondir à nouveau).', rangMax: 2, prerequis: ['tension'], mecanique: 'conduction' },
        { id: 'flux', ligne: 4, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Flux arcanique', icone: 'item/echo_shard', effet: 'Les orbes reviennent une fois et touchent à nouveau, à 50 / 80 %.', rangMax: 2, prerequis: ['orbes'], mecanique: 'flux_arcanique' },
        { id: 'froid', ligne: 4, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Froid mordant', icone: 'block/blue_ice', effet: 'Ralentissements +40 % de durée.', rangMax: 1, prerequis: ['gel'], modif: { dureeRalenti: 0.4 } },
        { id: 'surcharge', ligne: 5, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Surcharge', icone: 'item/glowstone_dust', effet: 'Les éclairs critiques étourdissent 0,3 / 0,5 s (pas les boss).', rangMax: 2, prerequis: ['conduction'], mecanique: 'surcharge' },
        { id: 'acceleration', ligne: 5, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Accélération', icone: 'item/recovery_compass_00', effet: 'Tous les 10 sorts, délai de tous les sorts −20 % pendant 2 / 3 s.', rangMax: 2, prerequis: ['flux'], mecanique: 'acceleration' },
        { id: 'profond', ligne: 5, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Gel profond', icone: 'block/ice', effet: 'Les ennemis ralentis subissent +10 / 20 % de dégâts.', rangMax: 2, prerequis: ['froid'], modif: { degatsRalentis: 0.1 } },
        { id: 'f_orage', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: "Tempête d'orage", icone: 'item/heart_of_the_sea', effet: "Toutes les 20 s, 8 éclairs tombent en 2 s sur des ennemis à moins de 10 blocs. Chacun inflige les dégâts d'un éclair de l'arme Éclairs (son rang actuel, rang 1 si elle n'est pas équipée), critiques compris. Ce sont des réactions : ni Haute tension ni Conduction.", rangMax: 1, prerequis: ['surcharge'], mecanique: 'tempete_orage' },
        { id: 'f_echo', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: 'Écho arcanique', icone: 'item/disc_fragment_5', effet: "Tous les 6 sorts, le prochain sort compatible se répète 0,5 s plus tard à 60 %. Délai interne 3 s ; l'écho ne compte pas.", rangMax: 1, prerequis: ['acceleration'], mecanique: 'echo_arcanique' },
        { id: 'f_zero', ligne: 6, colonne: 5, type: 'final', branche: 'droite', nom: 'Zéro absolu', icone: 'block/powder_snow', effet: 'Toutes les 4 Novas, la suivante gèle (enracine 1 s ; un boss est ralenti de 25 %) et fait ×2 dégâts.', rangMax: 1, prerequis: ['profond'], mecanique: 'zero_absolu' },
      ],
    },
  ],
  invocateur: [
    {
      id: 'maitre_chien',
      nom: 'Maître-chien',
      description: 'Une meute toujours plus grande.',
      icone: 'item/bone',
      branches: { tronc: 'Tronc', gauche: 'Meute', centre: 'Marques', droite: 'Alpha', libre: 'Blaze' },
      noeuds: [
        { id: 'chef', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Chef de meute', icone: 'item/bone', effet: 'Les loups arrivent en courant : leur première morsure fait +50 % de dégâts.', rangMax: 1, mecanique: 'chef_meute' },
        { id: 'crocs', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Crocs aiguisés', icone: 'item/wolf_armor', effet: 'Dégâts des alliés +4 / 8 / 12 %.', rangMax: 3, prerequis: ['chef'], modif: { degatsAllies: 0.04 } },
        { id: 'piste', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Sur la piste', icone: 'item/sugar', effet: 'Vitesse +3 / 6 / 9 % ; vitesse des alliés +5 / 10 / 15 %.', rangMax: 3, prerequis: ['chef'], modif: { vitesse: 0.03, vitesseAllies: 0.05 } },
        { id: 'dressage', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Dressage', icone: 'item/lead', effet: 'La Meute de loups commence avec +1 / +2 rangs.', rangMax: 2, prerequis: ['crocs'], rangsArme: 'loups' },
        { id: 'morsure', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Morsure marquante', icone: 'item/name_tag', effet: 'Une morsure marque la cible 3 s : +15 / 25 % de dégâts pour les autres alliés.', rangMax: 2, prerequis: ['crocs', 'piste'], mecanique: 'morsure_marquante' },
        { id: 'reactivite', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Réactivité', icone: 'item/heart_of_the_sea', effet: "Après la mort d'un allié, le prochain déclenchement de son arme arrive 25 / 40 % plus tôt.", rangMax: 2, prerequis: ['piste'], mecanique: 'reactivite' },
        { id: 'soudee', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Meute soudée', icone: 'item/leather', effet: '+3 / 4 % de dégâts par allié en vie (24 / 32 % au maximum).', rangMax: 2, prerequis: ['dressage'], mecanique: 'meute_soudee' },
        { id: 'traque', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Traque / Curée', icone: 'item/compass_00', effet: 'Au choix : Traque ou Curée.', rangMax: 1, prerequis: ['morsure'], options: [
        { nom: 'Traque', icone: 'item/compass_00', effet: 'Les marques durent 2 s de plus, et les loups visent d’abord les cibles marquées.', mecanique: 'traque' },
        { nom: 'Curée', icone: 'item/cooked_beef', effet: 'Tuer une cible marquée soigne les alliés proches de 10 % (1 fois par allié toutes les 8 s).', mecanique: 'curee' },
      ] },
        { id: 'os', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Os à ronger', icone: 'item/bone_meal', effet: 'PV des loups +20 %.', rangMax: 1, prerequis: ['reactivite'], armes: [{ arme: 'loups', pv: 0.2 }] },
        { id: 'familier', ligne: 4, colonne: 0, type: 'capacite', branche: 'libre', nom: 'Familier de blaze', icone: 'item/blaze_rod', effet: "Commence la partie avec l'Esprit de blaze.", rangMax: 1, libre: true, armeDepart: 'esprit_blaze' },
        { id: 'hurlement', ligne: 4, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Hurlement', icone: 'item/goat_horn', effet: 'Toutes les 10 s, les alliés proches attaquent 10 / 20 % plus vite pendant 3 s.', rangMax: 2, prerequis: ['soudee'], mecanique: 'hurlement' },
        { id: 'encerclement', ligne: 4, colonne: 4, type: 'bonus', branche: 'centre', nom: 'Encerclement', icone: 'item/rabbit_hide', effet: 'Un ennemi mordu par 3 loups différents en moins de 2 s subit +15 / 30 % de dégâts pendant 3 s (rafraîchi, non cumulable).', rangMax: 2, prerequis: ['traque'], mecanique: 'encerclement' },
        { id: 'braises', ligne: 5, colonne: 0, type: 'bonus', branche: 'libre', nom: 'Braises de blaze', icone: 'item/blaze_powder', effet: 'Les tirs du blaze enflamment (1 / 2 cumuls).', rangMax: 2, prerequis: ['familier'], mecanique: 'braises_blaze' },
        { id: 'coordination', ligne: 5, colonne: 2, type: 'bonus', branche: 'gauche', nom: 'Coordination', icone: 'item/bell', effet: 'Vex et blazes visent d’abord les cibles marquées, avec +10 / 20 % de dégâts.', rangMax: 2, prerequis: ['hurlement'], mecanique: 'coordination' },
        { id: 'alpha', ligne: 5, colonne: 4, type: 'bonus', branche: 'droite', nom: 'Alpha', icone: 'item/diamond_horse_armor', effet: 'Le premier loup est un alpha : PV ×2 et +30 / 50 % de dégâts.', rangMax: 2, prerequis: ['encerclement'], mecanique: 'alpha' },
        { id: 'f_ruee', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Ruée de la meute', icone: 'item/saddle', effet: '15 coups sur des cibles marquées déclenchent un hurlement : les loups vont 2 fois plus vite avec +50 % de dégâts pendant 3 s, et 2 loups fantômes les rejoignent. Délai interne 12 s.', rangMax: 1, prerequis: ['coordination'], mecanique: 'ruee_meute' },
        { id: 'f_lune', ligne: 6, colonne: 3, type: 'final', branche: 'droite', nom: 'Lune de sang', icone: 'item/redstone', effet: "Toutes les 60 s, pendant 8 s, le quota de loups passe de 6 à 12. Les loups manquants apparaissent aussitôt autour de l'Invocateur, avec 50 % des PV d'un loup, et disparaissent à la fin (ce n'est pas une mort). Pendant la Lune, les « +1 allié » en surplus deviennent de vrais loups au lieu de leur bonus.", rangMax: 1, prerequis: ['alpha'], mecanique: 'lune_de_sang' },
        { id: 'f_galpha', ligne: 6, colonne: 5, type: 'final', branche: 'droite', nom: 'Gardien alpha', icone: 'item/golden_apple', effet: "L'alpha provoque les ennemis, grandit de 25 % et revient une fois après sa mort, à chaque invocation.", rangMax: 1, prerequis: ['alpha'], mecanique: 'gardien_alpha' },
      ],
    },
    {
      id: 'evocateur',
      nom: 'Évocateur',
      description: 'Esprits et golems à son service.',
      icone: 'item/echo_shard',
      branches: { tronc: 'Tronc', gauche: 'Golem', centre: 'Vex', droite: 'Soutien', libre: 'Racines' },
      noeuds: [
        { id: 'lien', ligne: 0, colonne: 3, type: 'capacite', branche: 'tronc', nom: 'Lien spirituel', icone: 'item/ender_eye', effet: "Quand un allié est tué (PV à 0), l'Invocateur gagne un bouclier de 2 PV (délai interne 5 s). Une expiration ou un remplacement ne compte pas.", rangMax: 1, mecanique: 'lien_spirituel' },
        { id: 'robustesse', ligne: 1, colonne: 2, type: 'bonus', branche: 'tronc', nom: 'Robustesse', icone: 'item/iron_ingot', effet: '+1 cœur par rang.', rangMax: 3, prerequis: ['lien'], modif: { pvMax: 2 } },
        { id: 'pacte', ligne: 1, colonne: 4, type: 'bonus', branche: 'tronc', nom: 'Pacte durable', icone: 'item/writable_book', effet: 'Alliés : PV +10 / 20 / 30 % et durée +1 / 2 / 3 s.', rangMax: 3, prerequis: ['lien'], modif: { pvAllies: 0.1, dureeAllies: 1 } },
        { id: 'garde', ligne: 2, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Garde rapprochée', icone: 'item/iron_chestplate', effet: 'Armure +1 / +2 tant qu’un allié est à moins de 4 blocs.', rangMax: 2, prerequis: ['robustesse'], mecanique: 'garde_rapprochee' },
        { id: 'lies', ligne: 2, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Esprits liés', icone: 'item/echo_shard', effet: 'Les Vex commencent avec +1 / +2 rangs.', rangMax: 2, prerequis: ['robustesse', 'pacte'], rangsArme: 'vex' },
        { id: 'protecteur', ligne: 2, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Lien protecteur', icone: 'block/amethyst_cluster', effet: 'Esprit protecteur : boucliers +1 / +2 et soins +20 % (1 soin par allié toutes les 8 s).', rangMax: 2, prerequis: ['pacte'], armes: [{ arme: 'esprit_protecteur', ajout: { degats: 1 }, part: { zone: 0.2 } }] },
        { id: 'longevite', ligne: 3, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Longévité', icone: 'item/glow_berries', effet: 'Régénération +0,5 % des PV max par seconde, par rang.', rangMax: 2, prerequis: ['garde'], modif: { regenerationPart: 0.005 } },
        { id: 'vex', ligne: 3, colonne: 3, type: 'choix', branche: 'centre', nom: 'Vex furieux / Vex spectraux', icone: 'item/iron_sword', effet: 'Au choix : Vex furieux ou Vex spectraux.', rangMax: 1, prerequis: ['lies'], options: [
        { nom: 'Vex furieux', icone: 'item/iron_sword', effet: 'Vex : vitesse +30 %.', armes: [{ arme: 'vex', vitesse: 0.3 }] },
        { nom: 'Vex spectraux', icone: 'item/phantom_membrane', effet: 'Vex : PV +50 % et durée +2 s.', armes: [{ arme: 'vex', pv: 0.5, ajout: { duree: 2 } }] },
      ] },
        { id: 'allay', ligne: 3, colonne: 5, type: 'bonus', branche: 'droite', nom: 'Allay chanteur', icone: 'item/music_disc_cat', effet: "L'allay ramasse les orbes d'XP proches.", rangMax: 1, prerequis: ['protecteur'], mecanique: 'allay_chanteur' },
        { id: 'golem', ligne: 4, colonne: 1, type: 'capacite', branche: 'gauche', nom: 'Gardien de fer', icone: 'block/carved_pumpkin', effet: 'Commence avec le Golem de fer (rang 1, puis rang 2).', rangMax: 2, prerequis: ['longevite'], armeDepart: 'golems' },
        { id: 'nuee', ligne: 4, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Nuée', icone: 'item/feather', effet: 'Vex : +1 / +2 alliés par invocation (dans le quota).', rangMax: 2, prerequis: ['vex'], armes: [{ arme: 'vex', ajout: { nombre: 1 } }] },
        { id: 'racines', ligne: 4, colonne: 6, type: 'capacite', branche: 'libre', nom: 'Racines anciennes', icone: 'block/mangrove_roots_side', effet: 'Commence la partie avec le Totem des racines.', rangMax: 1, libre: true, armeDepart: 'totem_racines' },
        { id: 'forge', ligne: 5, colonne: 1, type: 'bonus', branche: 'gauche', nom: 'Forge', icone: 'block/iron_block', effet: 'Golem : PV et dégâts +15 / 30 %.', rangMax: 2, prerequis: ['golem'], armes: [{ arme: 'golems', pv: 0.15, part: { degats: 0.15 } }] },
        { id: 'essaim', ligne: 5, colonne: 3, type: 'bonus', branche: 'centre', nom: 'Essaim', icone: 'item/gunpowder', effet: 'Un vex tué (PV à 0) explose : 6 / 10 dégâts (rayon 1,5). Un vex dont la durée expire n’explose pas.', rangMax: 2, prerequis: ['nuee'], mecanique: 'essaim' },
        { id: 'profondes', ligne: 5, colonne: 5, type: 'bonus', branche: 'libre', nom: 'Racines profondes', icone: 'block/rooted_dirt', effet: 'Totem des racines : durée +2 / 4 s et PV +30 %.', rangMax: 2, prerequis: ['racines'], armes: [{ arme: 'totem_racines', pv: 0.3, ajout: { duree: 2 } }] },
        { id: 'f_sup', ligne: 6, colonne: 1, type: 'final', branche: 'gauche', nom: 'Évocation supérieure', icone: 'item/totem_of_undying', effet: "Chaque déclenchement d'une arme d'invocation compte 1, quel que soit le nombre de créatures (l'allay et le ravageur ne comptent pas). À 8, un ravageur charge à travers la horde pendant 8 s (40 dégâts, gros recul). Un seul à la fois : le compteur attend son départ. Délai interne 30 s.", rangMax: 1, prerequis: ['forge'], mecanique: 'evocation_superieure' },
        { id: 'f_crocs', ligne: 6, colonne: 3, type: 'final', branche: 'centre', nom: "Crocs de l'évocateur", icone: 'item/flint', effet: "Toutes les 6 s, une rangée de crocs d'évocateur jaillit du sol vers le groupe le plus proche (20 dégâts).", rangMax: 1, prerequis: ['essaim'], mecanique: 'crocs_evocateur' },
        { id: 'f_bosquet', ligne: 6, colonne: 5, type: 'final', branche: 'libre', nom: 'Bosquet', icone: 'block/azalea_leaves', effet: 'Deux totems à moins de 8 blocs font pousser entre eux une haie de racines qui ralentit.', rangMax: 1, prerequis: ['profondes'], mecanique: 'bosquet' },
      ],
    },
  ],

};

/**
 * Maîtrise : niveau 1 à 700 XP ramassées avec la classe, chaque niveau suivant
 * en demande 175 de plus (une partie complète en rapporte environ 9 000). Un
 * niveau = un point de talent, un point = un rang : un arbre (31 rangs) se
 * complète en une douzaine de parties complètes.
 */
export const MAITRISE: DefinitionMaitrise = {
  icone: 'item/experience_bottle',
  premierNiveau: 700,
  increment: 175,
};

/** XP de maîtrise totale pour atteindre ce niveau. */
export function xpPourNiveau(niveau: number): number {
  return niveau * MAITRISE.premierNiveau + MAITRISE.increment * (niveau * (niveau - 1)) / 2;
}

/** Niveau de maîtrise atteint avec cette XP. */
export function niveauMaitrise(xp: number): number {
  let niveau = 0;
  while (xpPourNiveau(niveau + 1) <= xp) niveau++;
  return niveau;
}

/** Ce qui n'est pas disponible dès la première partie, et comment l'obtenir. */
export const DEBLOCAGES: readonly DefinitionDeblocage[] = [
  { type: 'arene', id: 'mine', nom: 'Mine souterraine', condition: { type: 'survie', duree: 300, arene: 'prairie' }, texte: 'Survivre 5:00 dans la Prairie' },
  { type: 'arene', id: 'nether', nom: 'Zone volcanique', condition: { type: 'survie', duree: 300, arene: 'mine' }, texte: 'Survivre 5:00 dans la Mine' },
];
