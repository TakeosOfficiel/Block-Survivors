/**
 * Modèles cubiques : aucun modèle 3D importé, tout est construit en boîtes.
 *
 * Repère : le modèle regarde vers +Z, son côté droit est en −X, ses pieds en y = 0.
 * 1 pixel de texture = 1/16 de bloc.
 *
 * `uv` est l'origine du patron de la boîte ; ses faces s'en déduisent selon le
 * layout vanilla (l = largeur, p = profondeur) :
 *   dessus (u+p, v)   dessous (u+p+l, v)
 *   droite (u, v+p)   avant (u+p, v+p)   gauche (u+p+l, v+p)   arrière (u+2p+l, v+p)
 * Pour un skin 64x64 cela redonne exactement le layout standard
 * (tête : top(8,0) bottom(16,0) right(0,8) front(8,8) left(16,8) back(24,8), etc.).
 */
import type { DefinitionModele } from './types';

const definitions = {
  // Joueur, zombie… : 6 boîtes (tête 8x8x8, corps 8x12x4, bras et jambes 4x12x4)
  humanoide: {
    tailleTexture: [64, 64],
    parties: {
      tete:        { taille: [8, 8, 8],  pivot: [0, 24, 0],    centre: [0, 4, 0],   uv: [0, 0] },
      corps:       { taille: [8, 12, 4], pivot: [0, 24, 0],    centre: [0, -6, 0],  uv: [16, 16] },
      brasDroit:   { taille: [4, 12, 4], pivot: [-5, 22, 0],   centre: [-1, -4, 0], uv: [40, 16] },
      brasGauche:  { taille: [4, 12, 4], pivot: [5, 22, 0],    centre: [1, -4, 0],  uv: [32, 48] },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],  uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],  uv: [16, 48] },
    },
    // Valeurs de Minecraft : 0,6662 rad par pas d'animation, bras ×1, jambes ×1,4
    marche: {
      frequence: 2.665,
      vitesseReference: 5.8,
      lissage: 8,
      membres: [
        { partie: 'brasDroit',   amplitude: 1.0, dephasage: Math.PI },
        { partie: 'brasGauche',  amplitude: 1.0, dephasage: 0 },
        { partie: 'jambeDroite', amplitude: 1.4, dephasage: 0 },
        { partie: 'jambeGauche', amplitude: 1.4, dephasage: Math.PI },
      ],
    },
    repos: {
      amplitude: 0.05,
      frequenceZ: 1.8,
      frequenceX: 1.34,
      membres: [
        { partie: 'brasDroit',  signe: 1 },
        { partie: 'brasGauche', signe: -1 },
      ],
    },
  },
} satisfies Record<string, DefinitionModele>;

export type IdModele = keyof typeof definitions;

export const MODELES: Readonly<Record<IdModele, DefinitionModele>> = definitions;
