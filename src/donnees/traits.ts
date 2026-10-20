/**
 * Traits de l'éditeur de personnage : chaque trait est une liste de choix
 * (nom + couleur ou style). Le skin 64×64 est dessiné à partir de ces choix
 * (apparence/skinPerso.ts). Ajouter un choix = ajouter une entrée.
 */

export interface ChoixCouleur {
  readonly nom: string;
  readonly couleur: number;
}

export interface ChoixStyle<S extends string> {
  readonly nom: string;
  readonly style: S;
}

/** Couleurs des 16 teintures du jeu, pour les vêtements. */
const TEINTURES: readonly ChoixCouleur[] = [
  { nom: 'Blanc', couleur: 0xf9fffe },
  { nom: 'Gris clair', couleur: 0x9d9d97 },
  { nom: 'Gris', couleur: 0x474f52 },
  { nom: 'Noir', couleur: 0x1d1d21 },
  { nom: 'Marron', couleur: 0x835432 },
  { nom: 'Rouge', couleur: 0xb02e26 },
  { nom: 'Orange', couleur: 0xf9801d },
  { nom: 'Jaune', couleur: 0xfed83d },
  { nom: 'Vert clair', couleur: 0x80c71f },
  { nom: 'Vert', couleur: 0x5e7c16 },
  { nom: 'Cyan', couleur: 0x169c9c },
  { nom: 'Bleu clair', couleur: 0x3ab3da },
  { nom: 'Bleu', couleur: 0x3c44aa },
  { nom: 'Violet', couleur: 0x8932b8 },
  { nom: 'Magenta', couleur: 0xc74ebd },
  { nom: 'Rose', couleur: 0xf38baa },
];

export type StyleCoiffure = 'chauve' | 'courte' | 'meche' | 'longue' | 'herissee' | 'queue' | 'bouclee';
export type StyleBarbe = 'aucune' | 'moustache' | 'bouc' | 'courte' | 'longue';
export type StyleHaut = 'tshirt' | 'chemise' | 'sweat' | 'veste';

export const TRAITS = {
  peau: {
    nom: 'Couleur de peau',
    choix: [
      { nom: 'Claire', couleur: 0xf2cfae },
      { nom: 'Rosée', couleur: 0xe8b896 },
      { nom: 'Dorée', couleur: 0xd9a066 },
      { nom: 'Mate', couleur: 0xb97a4b },
      { nom: 'Brune', couleur: 0x8d5a3b },
      { nom: 'Foncée', couleur: 0x5e3a26 },
    ] as readonly ChoixCouleur[],
  },
  yeux: {
    nom: 'Yeux',
    choix: [
      { nom: 'Bleus', couleur: 0x3a6fd8 },
      { nom: 'Verts', couleur: 0x3f9e4a },
      { nom: 'Marron', couleur: 0x6b3f1f },
      { nom: 'Gris', couleur: 0x7a8a99 },
      { nom: 'Violets', couleur: 0x7d3fbf },
      { nom: 'Ambre', couleur: 0xd08a1c },
    ] as readonly ChoixCouleur[],
  },
  coiffure: {
    nom: 'Coiffure',
    choix: [
      { nom: 'Courte', style: 'courte' },
      { nom: 'Mèche', style: 'meche' },
      { nom: 'Longue', style: 'longue' },
      { nom: 'Hérissée', style: 'herissee' },
      { nom: 'Queue de cheval', style: 'queue' },
      { nom: 'Bouclée', style: 'bouclee' },
      { nom: 'Chauve', style: 'chauve' },
    ] as readonly ChoixStyle<StyleCoiffure>[],
  },
  cheveux: {
    nom: 'Couleur des cheveux',
    choix: [
      { nom: 'Noirs', couleur: 0x1e1a18 },
      { nom: 'Bruns', couleur: 0x3b2516 },
      { nom: 'Châtains', couleur: 0x6a4026 },
      { nom: 'Auburn', couleur: 0x8a3a1c },
      { nom: 'Roux', couleur: 0xc4561f },
      { nom: 'Blonds', couleur: 0xd8b45a },
      { nom: 'Platine', couleur: 0xe8e0c8 },
      { nom: 'Blancs', couleur: 0xf0f0f0 },
      { nom: 'Bleus', couleur: 0x3a5fd0 },
      { nom: 'Roses', couleur: 0xe070b0 },
    ] as readonly ChoixCouleur[],
  },
  barbe: {
    nom: 'Pilosité faciale',
    choix: [
      { nom: 'Aucune', style: 'aucune' },
      { nom: 'Moustache', style: 'moustache' },
      { nom: 'Bouc', style: 'bouc' },
      { nom: 'Barbe courte', style: 'courte' },
      { nom: 'Barbe longue', style: 'longue' },
    ] as readonly ChoixStyle<StyleBarbe>[],
  },
  haut: {
    nom: 'Haut',
    choix: [
      { nom: 'T-shirt', style: 'tshirt' },
      { nom: 'Chemise', style: 'chemise' },
      { nom: 'Sweat', style: 'sweat' },
      { nom: 'Veste ouverte', style: 'veste' },
    ] as readonly ChoixStyle<StyleHaut>[],
  },
  couleurHaut: { nom: 'Couleur du haut', choix: TEINTURES },
  pantalon: { nom: 'Pantalon', choix: TEINTURES },
  chaussures: { nom: 'Chaussures', choix: TEINTURES },
} as const;

export type IdTrait = keyof typeof TRAITS;

export const IDS_TRAITS = Object.keys(TRAITS) as IdTrait[];

/** Index choisi pour chaque trait. */
export type Traits = Record<IdTrait, number>;

/** Un personnage de départ agréable (cheveux châtains, haut cyan, pantalon bleu). */
export const TRAITS_PAR_DEFAUT: Traits = {
  peau: 1, yeux: 0, coiffure: 0, cheveux: 2, barbe: 0, haut: 0, couleurHaut: 10, pantalon: 12, chaussures: 2,
};
