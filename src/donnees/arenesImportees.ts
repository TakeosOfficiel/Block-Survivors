/**
 * Arènes construites dans Minecraft : chaque schématique du dossier cartes/
 * (.schem de WorldEdit / FAWE, .nbt de bloc de structure) devient une arène.
 * Un fichier JSON du même nom, facultatif, la règle :
 *
 *   {
 *     "nom": "Jardin des champignons",
 *     "description": "…",
 *     "icone": "block/red_mushroom_block",
 *     "base": "prairie",              // ennemis, boss et ambiance repris de cette arène
 *     "ciel": "#78a7ff",
 *     "brouillard": { "debut": 45, "fin": 120 },
 *     "lumiere": { "ambiante": 0.35, "soleil": 0.75 },
 *     "boss": ["geant"],
 *     "ennemis": [{ "ennemi": "zombie", "depuis": 0, "poids": 6 }]
 *   }
 *
 * La taille, le sol et le décor viennent de la schématique, lue au lancement.
 */
import type { ChoixEnnemi, DefinitionArene, ZoneEnnemis } from './types';
import type { IdEnnemi } from './ennemis';

interface ReglagesCarte {
  readonly nom?: string;
  readonly description?: string;
  readonly icone?: string;
  readonly base?: string;
  readonly ciel?: string | number;
  readonly brouillard?: { readonly debut: number; readonly fin: number };
  readonly lumiere?: Partial<{
    readonly ambiante: number;
    readonly soleil: number;
    readonly couleurAmbiante: string | number;
    readonly couleurSoleil: string | number;
  }>;
  readonly boss?: readonly IdEnnemi[];
  readonly miniBoss?: readonly IdEnnemi[];
  readonly ennemis?: readonly ChoixEnnemi[];
  readonly zones?: readonly ZoneEnnemis[];
}

const FICHIERS = import.meta.glob('../../cartes/*.{schem,nbt}', { query: '?inline', import: 'default', eager: true }) as Record<string, string>;
const REGLAGES = import.meta.glob('../../cartes/*.json', { import: 'default', eager: true }) as Record<string, ReglagesCarte>;

/** « #78a7ff » ou nombre → couleur 0xRRGGBB. */
function couleur(valeur: string | number | undefined, defaut: number): number {
  if (valeur === undefined) return defaut;
  return typeof valeur === 'number' ? valeur : parseInt(valeur.replace('#', ''), 16);
}

/** « map_mushroom » → « Map mushroom ». */
function nomLisible(id: string): string {
  const texte = id.replace(/[_-]+/g, ' ').trim();
  return texte.charAt(0).toUpperCase() + texte.slice(1);
}

export function arenesImportees(bases: readonly DefinitionArene[]): DefinitionArene[] {
  return Object.entries(FICHIERS).map(([chemin, fichier]) => {
    const id = chemin.split('/').pop()!.replace(/\.(schem|nbt)$/, '');
    const r: ReglagesCarte = REGLAGES[`../../cartes/${id}.json`] ?? {};
    const base = bases.find((a) => a.id === r.base) ?? bases[0];
    return {
      ...base,
      id,
      nom: r.nom ?? nomLisible(id),
      description: r.description ?? 'Map construite dans Minecraft.',
      icone: r.icone ?? 'block/grass_block_side',
      // Connues une fois la schématique lue (voir minecraft/importation.ts)
      largeur: 0,
      profondeur: 0,
      hauteurMur: 0,
      sol: [],
      taches: [],
      murs: [],
      decor: [],
      zoneDepart: 0,
      exterieur: null,
      ciel: couleur(r.ciel, base.ciel),
      brouillard: r.brouillard ?? base.brouillard,
      lumiere: {
        ambiante: r.lumiere?.ambiante ?? base.lumiere.ambiante,
        soleil: r.lumiere?.soleil ?? base.lumiere.soleil,
        couleurAmbiante: couleur(r.lumiere?.couleurAmbiante, base.lumiere.couleurAmbiante),
        couleurSoleil: couleur(r.lumiere?.couleurSoleil, base.lumiere.couleurSoleil),
      },
      boss: r.boss ?? base.boss,
      miniBoss: r.miniBoss ?? base.miniBoss,
      ennemis: r.ennemis ?? base.ennemis,
      // Les zones de l'arène de base n'ont pas de sens ici : seulement celles du réglage
      zones: r.zones ?? [],
      source: { fichier },
    };
  });
}
