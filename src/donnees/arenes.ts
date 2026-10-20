/**
 * Arènes : sol, taches de sol, murs, décor (structures placées ou dispersées),
 * ambiance et population d'ennemis par zone. Ajouter une arène = ajouter une entrée.
 * Coordonnées en blocs, origine au centre (le joueur y démarre).
 */
import type { DefinitionArene } from './types';

const prairie: DefinitionArene = {
  id: 'prairie',
  nom: 'Prairie du village abandonné',
  icone: 'block/grass_block_side',
  description: 'Un village en ruine au milieu des champs.',
  taille: 60,
  hauteurMur: 2,
  sol: [{ bloc: 'grass_block', poids: 1 }],
  taches: [
    { type: 'taches', bloc: 'dirt', nombre: 6, rayon: [1, 2] },
    { type: 'rectangle', bloc: 'dirt_path', rect: [-30, -1, 30, 1] },
    { type: 'rectangle', bloc: 'dirt_path', rect: [-1, -30, 1, 30] },
    { type: 'rectangle', bloc: 'gravel', rect: [-4, -4, 4, 4] },
  ],
  murs: [
    { bloc: 'cobblestone', poids: 3 },
    { bloc: 'mossy_cobblestone', poids: 1 },
  ],
  decor: [
    { type: 'structure', structure: 'maison_ruine', x: -16, z: -15 },
    { type: 'structure', structure: 'maison_ruine', x: 9, z: -15 },
    { type: 'structure', structure: 'maison_ruine', x: -16, z: 8 },
    { type: 'structure', structure: 'grange_ruine', x: 8, z: 8 },
    { type: 'structure', structure: 'puits', x: -12, z: 3 },
    { type: 'dispersion', structure: 'meule', nombre: 4, espacement: 2, zone: [-20, -20, 20, 20] },
    { type: 'dispersion', structure: 'arbre_chene', nombre: 18, espacement: 2 },
    { type: 'dispersion', structure: 'herbes', nombre: 140, espacement: 0 },
    { type: 'dispersion', structure: 'pissenlit', nombre: 25, espacement: 0 },
    { type: 'dispersion', structure: 'coquelicot', nombre: 25, espacement: 0 },
  ],
  zoneDepart: 5,
  exterieur: { bloc: 'grass_block', assombrissement: 0.65, taille: 400 },
  ciel: 0x78a7ff,
  brouillard: { debut: 45, fin: 120 },
  lumiere: { ambiante: 0.35, soleil: 0.75, couleurAmbiante: 0xffffff, couleurSoleil: 0xffffff },
  boss: ['geant', 'seigneur_wither'],
  ennemis: [
    { ennemi: 'zombie', depuis: 0, poids: 6 },
    { ennemi: 'araignee', depuis: 20, poids: 6 },
    { ennemi: 'creeper', depuis: 60, poids: 4 },
    { ennemi: 'squelette', depuis: 90, poids: 2 },
  ],
  zones: [
    {
      nom: 'Village',
      rect: [-20, -20, 20, 20],
      ennemis: [
        { ennemi: 'zombie', depuis: 0, poids: 10 },
        { ennemi: 'squelette', depuis: 45, poids: 6 },
        { ennemi: 'creeper', depuis: 120, poids: 2 },
      ],
    },
  ],
};

const mine: DefinitionArene = {
  id: 'mine',
  nom: 'Mine souterraine',
  icone: 'block/iron_ore',
  description: 'Galeries sombres, piliers de minerai et toiles d\'araignée.',
  taille: 60,
  hauteurMur: 3,
  sol: [
    { bloc: 'stone', poids: 6 },
    { bloc: 'andesite', poids: 2 },
    { bloc: 'gravel', poids: 1 },
    { bloc: 'cobblestone', poids: 1 },
  ],
  taches: [{ type: 'taches', bloc: 'gravel', nombre: 8, rayon: [1, 3] }],
  murs: [
    { bloc: 'stone', poids: 6 },
    { bloc: 'coal_ore', poids: 1 },
    { bloc: 'iron_ore', poids: 1 },
  ],
  decor: [
    { type: 'structure', structure: 'ruine_mine', x: -4, z: -26 },
    { type: 'dispersion', structure: 'pilier_charbon', nombre: 7, espacement: 2 },
    { type: 'dispersion', structure: 'pilier_fer', nombre: 5, espacement: 2 },
    { type: 'dispersion', structure: 'pilier_diamant', nombre: 2, espacement: 2 },
    { type: 'dispersion', structure: 'etai', nombre: 8, espacement: 1 },
    { type: 'dispersion', structure: 'lampe', nombre: 14, espacement: 2 },
    { type: 'dispersion', structure: 'toiles', nombre: 14, espacement: 0, zone: [-30, -30, -4, -4] },
    { type: 'dispersion', structure: 'toiles', nombre: 4, espacement: 1 },
    { type: 'dispersion', structure: 'eboulis', nombre: 8, espacement: 1 },
  ],
  zoneDepart: 5,
  exterieur: { bloc: 'stone', assombrissement: 0.25, taille: 400 },
  ciel: 0x0b0b10,
  brouillard: { debut: 28, fin: 70 },
  lumiere: { ambiante: 0.22, soleil: 0.4, couleurAmbiante: 0x9aa0c0, couleurSoleil: 0xffe0b0 },
  boss: ['reine_araignee', 'geant'],
  ennemis: [
    { ennemi: 'zombie', depuis: 0, poids: 5 },
    { ennemi: 'araignee_venimeuse', depuis: 0, poids: 5 },
    { ennemi: 'poisson_argent', depuis: 30, poids: 4 },
    { ennemi: 'squelette', depuis: 40, poids: 5 },
    { ennemi: 'creeper', depuis: 75, poids: 3 },
  ],
  zones: [
    {
      nom: 'Galerie des toiles',
      rect: [-30, -30, -4, -4],
      ennemis: [
        { ennemi: 'araignee_venimeuse', depuis: 0, poids: 12 },
        { ennemi: 'poisson_argent', depuis: 15, poids: 4 },
      ],
    },
    {
      nom: 'Filon profond',
      rect: [6, 6, 30, 30],
      ennemis: [
        { ennemi: 'poisson_argent', depuis: 0, poids: 6 },
        { ennemi: 'squelette', depuis: 20, poids: 8 },
        { ennemi: 'creeper', depuis: 50, poids: 4 },
      ],
    },
  ],
};

const nether: DefinitionArene = {
  id: 'nether',
  nom: 'Zone volcanique',
  icone: 'block/netherrack',
  description: 'Lacs de lave, sable des âmes et ruines de forteresse.',
  taille: 60,
  hauteurMur: 3,
  sol: [{ bloc: 'netherrack', poids: 1 }],
  taches: [
    { type: 'taches', bloc: 'soul_sand', nombre: 9, rayon: [2, 4] },
    { type: 'taches', bloc: 'magma', nombre: 10, rayon: [1, 2] },
    { type: 'taches', bloc: 'lava', nombre: 6, rayon: [2, 4], zone: [-2, -30, 30, 30] },
  ],
  murs: [{ bloc: 'nether_bricks', poids: 1 }],
  decor: [
    { type: 'structure', structure: 'rempart', x: -27, z: -20 },
    { type: 'structure', structure: 'rempart', x: -27, z: -4 },
    { type: 'structure', structure: 'rempart', x: -27, z: 12 },
    { type: 'structure', structure: 'tour_nether', x: -18, z: -13 },
    { type: 'structure', structure: 'tour_nether', x: -18, z: 18 },
    { type: 'dispersion', structure: 'pilier_obsidienne', nombre: 8, espacement: 2 },
    { type: 'dispersion', structure: 'amas_glowstone', nombre: 8, espacement: 1 },
  ],
  zoneDepart: 6,
  exterieur: { bloc: 'netherrack', assombrissement: 0.45, taille: 400 },
  ciel: 0x2a0606,
  brouillard: { debut: 25, fin: 80 },
  lumiere: { ambiante: 0.4, soleil: 0.45, couleurAmbiante: 0xffb0a0, couleurSoleil: 0xff9060 },
  boss: ['magma_geant', 'seigneur_wither'],
  ennemis: [
    { ennemi: 'piglin_zombifie', depuis: 0, poids: 7 },
    { ennemi: 'cube_magma', depuis: 15, poids: 5 },
    { ennemi: 'blaze', depuis: 45, poids: 4 },
    { ennemi: 'squelette_wither', depuis: 90, poids: 3 },
  ],
  zones: [
    {
      nom: 'Forteresse',
      rect: [-30, -30, -12, 30],
      ennemis: [
        { ennemi: 'squelette_wither', depuis: 0, poids: 10 },
        { ennemi: 'blaze', depuis: 20, poids: 5 },
      ],
    },
    {
      nom: 'Lacs de lave',
      rect: [6, -30, 30, 30],
      ennemis: [
        { ennemi: 'cube_magma', depuis: 0, poids: 8 },
        { ennemi: 'blaze', depuis: 30, poids: 4 },
      ],
    },
  ],
};

export const ARENES: readonly DefinitionArene[] = [prairie, mine, nether];

export const ARENE_PAR_DEFAUT = 'prairie';
