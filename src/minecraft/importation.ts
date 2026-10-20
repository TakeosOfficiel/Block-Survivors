/**
 * Arène construite dans Minecraft : lit la schématique, cuit les modèles de
 * ses blocs, trouve la zone jouable, puis prépare la carte de collision et
 * l'affichage de toute la construction (le décor autour de l'arène compris).
 *
 * Zone jouable (version « à plat ») : à partir du point de départ (panneau
 * « depart », sinon le centre de la schématique), toutes les cases au même
 * niveau, reliées à pied, sur un sol plein avec deux blocs libres au-dessus.
 * Ce qui l'entoure (collines, clôtures, champignons géants…) devient le décor.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { DefinitionArene } from '../donnees/types';
import { BLOQUE_MARCHE, BLOQUE_TIR, CarteMonde } from '../monde/carte';
import type { MondeGenere } from '../monde/generation';
import { Atlas, construireMaillage } from './maillage';
import { BibliothequeModeles } from './modeles';
import { physiqueDe, type Physique } from './proprietes';
import { lireSchematique, type Schematique } from './schematique';

export interface SuiviImportation {
  etape(texte: string): void;
  progression(faites: number, total: number): void;
}

export interface AreneImportee {
  readonly arene: DefinitionArene;
  readonly monde: MondeGenere;
  /** Blocs sans modèle connu (affichés en damier rose et noir). */
  readonly inconnus: readonly string[];
}

export async function importerArene(
  definition: DefinitionArene,
  textures: BibliothequeTextures,
  suivi: SuiviImportation,
): Promise<AreneImportee> {
  if (!definition.source) throw new Error(`L'arène ${definition.id} n'a pas de fichier Minecraft.`);
  suivi.etape('Lecture de la map Minecraft…');
  const reponse = await fetch(definition.source.fichier);
  const schem = await lireSchematique(await reponse.arrayBuffer());

  suivi.etape('Chargement des modèles de blocs…');
  const blocs = await new BibliothequeModeles().cuire(schem.palette);
  const physiques = blocs.map(physiqueDe);

  const zone = trouverZone(schem, physiques);
  const carte = construireCarte(schem, physiques, zone);

  suivi.etape('Chargement des textures de la map…');
  await textures.charger(BibliothequeModeles.textures(blocs), (faites, total) => suivi.progression(faites, total));
  suivi.etape('Construction de la map…');
  await new Promise((resoudre) => requestAnimationFrame(resoudre));
  const atlas = new Atlas(BibliothequeModeles.textures(blocs), textures);
  const decor = construireMaillage(schem, blocs, atlas, {
    x: zone.x0 + zone.largeur / 2,
    y: zone.niveau,
    z: zone.z0 + zone.profondeur / 2,
  });

  const arene: DefinitionArene = {
    ...definition,
    largeur: zone.largeur,
    profondeur: zone.profondeur,
    depart: {
      x: zone.departX - zone.x0 - zone.largeur / 2 + 0.5,
      z: zone.departZ - zone.z0 - zone.profondeur / 2 + 0.5,
    },
  };
  const inconnus = blocs.filter((b) => b.inconnu).map((b) => b.etat);
  if (inconnus.length > 0) console.warn('Blocs Minecraft sans modèle :', inconnus);
  return { arene, monde: { blocs: new Map(), carte, decor }, inconnus };
}

// --- Zone jouable -------------------------------------------------------------------

interface Zone {
  /** Niveau des pieds (y de la première case libre au-dessus du sol). */
  readonly niveau: number;
  /** Rectangle englobant, en cases de la schématique. */
  readonly x0: number;
  readonly z0: number;
  readonly largeur: number;
  readonly profondeur: number;
  /** Cases du rectangle atteintes par le parcours (1 = sol praticable, 2 = liquide au sol). */
  readonly cases: Uint8Array;
  readonly departX: number;
  readonly departZ: number;
}

const PRATICABLE = 1;
const LIQUIDE = 2;

function trouverZone(schem: Schematique, physiques: readonly Physique[]): Zone {
  const { largeur: L, hauteur: H, profondeur: P } = schem;
  const physique = (x: number, y: number, z: number): Physique | null =>
    x < 0 || y < 0 || z < 0 || x >= L || y >= H || z >= P ? null : physiques[schem.blocs[(y * P + z) * L + x]];
  const libre = (x: number, y: number, z: number) => {
    const p = physique(x, y, z);
    return p === null ? y >= H : p.collision === 'vide';
  };
  const debout = (x: number, y: number, z: number) =>
    physique(x, y - 1, z)?.collision === 'plein' && libre(x, y, z) && libre(x, y + 1, z);

  // Départ : panneau « depart », sinon le centre ; le niveau le plus haut où l'on tient debout
  const panneau = schem.panneaux.find((p) => /d[ée]part/i.test(p.texte));
  const departX = panneau?.x ?? Math.floor(L / 2);
  const departZ = panneau?.z ?? Math.floor(P / 2);
  let niveau = -1;
  if (panneau) {
    for (let dy = 0; dy <= 2 && niveau < 0; dy++) {
      if (debout(departX, panneau.y - dy, departZ)) niveau = panneau.y - dy;
    }
  }
  for (let y = H; y >= 1 && niveau < 0; y--) if (debout(departX, y, departZ)) niveau = y;
  if (niveau < 0) {
    throw new Error('Aucun sol praticable au point de départ : pose un panneau « depart » au sol, dans l\'arène.');
  }

  // Parcours en largeur des cases reliées au même niveau (on traverse les étangs sans s'y arrêter)
  const etat = new Uint8Array(L * P);
  const file = new Int32Array(L * P);
  let debut = 0;
  let fin = 0;
  etat[departZ * L + departX] = PRATICABLE;
  file[fin++] = departZ * L + departX;
  let minX = departX;
  let maxX = departX;
  let minZ = departZ;
  let maxZ = departZ;
  while (debut < fin) {
    const c = file[debut++];
    const x = c % L;
    const z = (c - x) / L;
    minX = Math.min(minX, x);
    maxX = Math.max(maxX, x);
    minZ = Math.min(minZ, z);
    maxZ = Math.max(maxZ, z);
    for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const nx = x + dx;
      const nz = z + dz;
      if (nx < 0 || nz < 0 || nx >= L || nz >= P || etat[nz * L + nx] !== 0) continue;
      if (debout(nx, niveau, nz)) etat[nz * L + nx] = PRATICABLE;
      else if (physique(nx, niveau - 1, nz)?.collision === 'liquide' && libre(nx, niveau, nz) && libre(nx, niveau + 1, nz)) etat[nz * L + nx] = LIQUIDE;
      else continue;
      file[fin++] = nz * L + nx;
    }
  }

  const largeur = maxX - minX + 1;
  const profondeur = maxZ - minZ + 1;
  const cases = new Uint8Array(largeur * profondeur);
  for (let z = 0; z < profondeur; z++) {
    for (let x = 0; x < largeur; x++) cases[z * largeur + x] = etat[(minZ + z) * L + minX + x];
  }
  return { niveau, x0: minX, z0: minZ, largeur, profondeur, cases, departX, departZ };
}

/**
 * Carte de collision du rectangle de la zone : hors parcours, une case est
 * infranchissable (et arrête les tirs si un bloc plein s'y dresse) ; dans le
 * parcours, le sol et les blocs traversés donnent leurs effets.
 */
function construireCarte(schem: Schematique, physiques: readonly Physique[], zone: Zone): CarteMonde {
  const { largeur: L, hauteur: H, profondeur: P } = schem;
  const physique = (x: number, y: number, z: number): Physique | null =>
    x < 0 || y < 0 || z < 0 || x >= L || y >= H || z >= P ? null : physiques[schem.blocs[(y * P + z) * L + x]];
  const carte = new CarteMonde(zone.largeur, zone.profondeur);
  for (let iz = 0; iz < zone.profondeur; iz++) {
    for (let ix = 0; ix < zone.largeur; ix++) {
      const c = iz * zone.largeur + ix;
      const x = zone.x0 + ix;
      const z = zone.z0 + iz;
      const corps = [physique(x, zone.niveau, z), physique(x, zone.niveau + 1, z)];
      const sol = physique(x, zone.niveau - 1, z);
      if (zone.cases[c] !== PRATICABLE) {
        carte.drapeaux[c] |= BLOQUE_MARCHE;
        if (corps.some((p) => p?.collision === 'plein')) carte.drapeaux[c] |= BLOQUE_TIR;
        if (zone.cases[c] === LIQUIDE) carte.degats[c] = sol?.degats ?? 0;
        continue;
      }
      let ralentit = sol?.ralentit ?? 1;
      let horsToiles = ralentit;
      let degats = sol?.degats ?? 0;
      for (const p of corps) {
        if (!p) continue;
        if (p.ralentit !== undefined) {
          ralentit = Math.min(ralentit, p.ralentit);
          if (!p.toile) horsToiles = Math.min(horsToiles, p.ralentit);
        }
        degats = Math.max(degats, p.degats ?? 0);
      }
      carte.ralentissement[c] = ralentit;
      carte.ralentissementHorsToiles[c] = horsToiles;
      carte.degats[c] = degats;
    }
  }
  return carte;
}

