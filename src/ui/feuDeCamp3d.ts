/**
 * Scène 3D de l'écran de chargement : « La pause des mobs » (voir
 * donnees/feuDeCamp.ts). Le soir, autour du vrai feu de camp du jeu, le héros
 * caresse un mouton endormi, le zombie fait griller un cookie, le squelette
 * compte ses os et le creeper n'ose pas approcher. La vue tourne lentement.
 *
 * Aucun accès au DOM : la scène tourne dans un worker (feuDeCamp.worker.ts),
 * sur un OffscreenCanvas, pour rester fluide pendant que le fil principal
 * construit la partie. Les mobs reprennent les modèles du jeu
 * (donnees/modeles.ts) et le feu de camp son modèle de bloc vanilla.
 */
import * as THREE from 'three';
import {
  COUCHES_JOUEUR, DECO_FEU_DE_CAMP, FEU_DE_CAMP, FUMEE_FEU_DE_CAMP, MODELE_LAINE_MOUTON, MODELE_MOUTON, TEXTURES_FEU_DE_CAMP,
} from '../donnees/feuDeCamp';
import { MODELES, type IdModele } from '../donnees/modeles';
import type { DefinitionModele, MainModele } from '../donnees/types';
import type { Quad } from '../minecraft/modeles';
import { creerGeometrieCube, creerGeometrieObjet, PIXELS_PAR_BLOC } from '../moteur/geometrie';

const T = TEXTURES_FEU_DE_CAMP;
const F = FEU_DE_CAMP;

/** Le héros assis au coin du feu : son modèle, sa skin et l'objet qu'il tient. */
export interface HerosFeuDeCamp {
  readonly modele: IdModele;
  readonly skin: string;
  readonly objetEnMain?: string;
}

/** Images de la scène, par chemin de texture vanilla : la texture, et ses pixels (objets en relief). */
export interface ImagesFeuDeCamp {
  readonly textures: ReadonlyMap<string, THREE.Texture>;
  readonly pixels: ReadonlyMap<string, ImageData>;
}

/** Nombre pseudo-aléatoire stable dans [0, 1[ (le décor est le même à chaque fois). */
function hasard(n: number): number {
  const x = Math.sin(n * 127.1 + 0.5) * 43758.5453;
  return x - Math.floor(x);
}
/** Transition douce de 0 à 1 quand x passe de a à b. */
function lisse(a: number, b: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}

/** Mob en parties articulées : un groupe par articulation, nommé comme dans son modèle. */
interface Mob {
  readonly racine: THREE.Group;
  readonly parties: Readonly<Record<string, THREE.Group>>;
}
/** Couche d'un mob : un modèle et sa texture (la laine du mouton se pose sur sa peau). */
interface Couche {
  readonly definition: DefinitionModele;
  readonly texture: THREE.Texture | null;
}

function construireMob(couches: readonly Couche[]): Mob {
  const racine = new THREE.Group();
  const parties: Record<string, THREE.Group> = {};
  for (const { definition, texture } of couches) {
    const materiau = new THREE.MeshLambertMaterial({ map: texture, alphaTest: 0.5, side: THREE.DoubleSide });
    for (const [nom, cube] of Object.entries(definition.parties)) {
      // Seconde couche des joueurs et laine du mouton : sur l'articulation de la partie qu'elles habillent
      const base = COUCHES_JOUEUR[nom] && definition.parties[COUCHES_JOUEUR[nom]] ? COUCHES_JOUEUR[nom] : nom;
      let pivot = parties[base];
      if (!pivot) {
        pivot = new THREE.Group();
        pivot.position.set(cube.pivot[0] / PIXELS_PAR_BLOC, cube.pivot[1] / PIXELS_PAR_BLOC, cube.pivot[2] / PIXELS_PAR_BLOC);
        if (cube.rotation) pivot.rotation.set(cube.rotation[0], cube.rotation[1], cube.rotation[2]);
        racine.add(pivot);
        parties[base] = pivot;
      }
      const maillage = new THREE.Mesh(creerGeometrieCube(cube, definition.tailleTexture), materiau);
      maillage.castShadow = true;
      maillage.receiveShadow = true;
      pivot.add(maillage);
    }
  }
  return { racine, parties };
}

/** Pose un objet dans la main d'un mob, au point d'attache de son modèle (comme en jeu). */
function tenir(mob: Mob, main: MainModele, objet: THREE.Object3D): void {
  const [x, y, z] = main.position;
  objet.position.set(x / PIXELS_PAR_BLOC, y / PIXELS_PAR_BLOC, z / PIXELS_PAR_BLOC);
  objet.rotation.set(main.rotation[0], main.rotation[1], main.rotation[2]);
  mob.parties[main.partie]?.add(objet);
}

/** Deux plans en croix posés au sol (herbes, fleurs). */
function geometrieCroix(): THREE.BufferGeometry {
  const s = 0.45 / Math.SQRT2;
  const h = 0.9;
  const geometrie = new THREE.BufferGeometry();
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute([
    -s, 0, -s, s, 0, s, s, h, s, -s, h, -s,
    -s, 0, s, s, 0, -s, s, h, -s, -s, h, s,
  ], 3));
  geometrie.setAttribute('uv', new THREE.Float32BufferAttribute([0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1], 2));
  // Normales vers le haut : les plantes sont éclairées comme le sol, comme en jeu
  geometrie.setAttribute('normal', new THREE.Float32BufferAttribute(Array.from({ length: 8 }, () => [0, 1, 0]).flat(), 3));
  geometrie.setIndex([0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);
  return geometrie;
}

/** Même géométrie répétée aux positions données, en un seul appel de dessin. */
function instances(
  geometrie: THREE.BufferGeometry,
  materiau: THREE.Material | THREE.Material[],
  positions: readonly (readonly [number, number, number])[],
  ombres = true,
): THREE.InstancedMesh {
  const maillage = new THREE.InstancedMesh(geometrie, materiau, Math.max(1, positions.length));
  const matrice = new THREE.Matrix4();
  positions.forEach(([x, y, z], i) => maillage.setMatrixAt(i, matrice.makeTranslation(x, y, z)));
  maillage.count = positions.length;
  maillage.castShadow = ombres;
  maillage.receiveShadow = true;
  return maillage;
}

/** Feuillage d'un arbre en blocs, autour du sommet de son tronc. */
function feuillage(ax: number, ay: number, az: number, liste: [number, number, number][]): void {
  for (let x = -2; x <= 2; x++) {
    for (let z = -2; z <= 2; z++) {
      for (let y = 0; y <= 2; y++) {
        if (Math.abs(x) + Math.abs(z) + y * 1.4 > 3.6 || (x === 0 && z === 0 && y < 2)) continue;
        if (Math.abs(x) === 2 && Math.abs(z) === 2) continue;
        liste.push([ax + x, ay + y, az + z]);
      }
    }
  }
  liste.push([ax, ay + 3, az]);
}

/** Petite image dessinée à la volée (OffscreenCanvas : la scène tourne dans un worker). */
function dessin(largeur: number, hauteur: number, peindre: (ctx: OffscreenCanvasRenderingContext2D) => void): THREE.CanvasTexture<OffscreenCanvas> {
  const toile = new OffscreenCanvas(largeur, hauteur);
  const ctx = toile.getContext('2d');
  if (ctx) peindre(ctx);
  return new THREE.CanvasTexture(toile);
}

/** « z » du dormeur en pixels, avec son ombre portée, comme le texte du jeu. */
function spriteZ(): THREE.SpriteMaterial {
  const motif = ['#####', '...#.', '..#..', '.#...', '#####'];
  const texture = dessin(8, 8, (ctx) => {
    for (const [teinte, decalage] of [['#3a3a5a', 1], ['#ffffff', 0]] as const) {
      ctx.fillStyle = teinte;
      motif.forEach((ligne, y) => [...ligne].forEach((c, x) => { if (c === '#') ctx.fillRect(x + decalage, y + decalage, 1, 1); }));
    }
  });
  texture.magFilter = THREE.NearestFilter;
  texture.minFilter = THREE.NearestFilter;
  texture.colorSpace = THREE.SRGBColorSpace;
  return new THREE.SpriteMaterial({ map: texture, transparent: true, depthWrite: false });
}

/** Géométries du modèle de bloc (repère 0 à 1), une par texture, centrées en x et z. */
function geometriesBloc(quads: readonly Quad[]): Map<string, THREE.BufferGeometry> {
  const parTexture = new Map<string, { pos: number[]; uv: number[]; n: number[]; idx: number[] }>();
  for (const q of quads) {
    let g = parTexture.get(q.texture);
    if (!g) parTexture.set(q.texture, (g = { pos: [], uv: [], n: [], idx: [] }));
    const base = g.pos.length / 3;
    for (let k = 0; k < 4; k++) {
      g.pos.push(q.positions[k * 3] - 0.5, q.positions[k * 3 + 1], q.positions[k * 3 + 2] - 0.5);
      g.uv.push(q.uv[k * 2] / 16, 1 - q.uv[k * 2 + 1] / 16);
      g.n.push(...q.normale);
    }
    // Coins haut-gauche, haut-droit, bas-droit, bas-gauche vus de face
    g.idx.push(base, base + 3, base + 2, base, base + 2, base + 1);
  }
  const geometries = new Map<string, THREE.BufferGeometry>();
  for (const [texture, g] of parTexture) {
    const geometrie = new THREE.BufferGeometry();
    geometrie.setAttribute('position', new THREE.Float32BufferAttribute(g.pos, 3));
    geometrie.setAttribute('uv', new THREE.Float32BufferAttribute(g.uv, 2));
    geometrie.setAttribute('normal', new THREE.Float32BufferAttribute(g.n, 3));
    geometrie.setIndex(g.idx);
    geometries.set(texture, geometrie);
  }
  return geometries;
}

/** Un mob assis autour du feu, à son angle. */
interface Place {
  readonly mob: Mob;
  readonly angle: number;
}

type Lambert = (chemin: string, opts?: THREE.MeshLambertMaterialParameters) => THREE.MeshLambertMaterial;

export class SceneFeuDeCamp {
  private readonly renderer: THREE.WebGLRenderer;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(F.camera.champ, 16 / 9, 0.1, 200);
  private readonly images: ImagesFeuDeCamp;

  private heros!: Place;
  private zombie!: Place;
  private squelette!: Place;
  private mouton!: Mob;
  private creeper!: Mob;
  private cookie: THREE.Mesh<THREE.BufferGeometry, THREE.MeshLambertMaterial> | null = null;
  private readonly lettresZ: THREE.Sprite[] = [];
  private lumiereFeu!: THREE.PointLight;
  private lumiereLanterne!: THREE.PointLight;
  private lueur!: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>;
  private readonly fumees: THREE.Sprite[] = [];
  private readonly imagesFumee: (THREE.Texture | null)[] = [];
  private readonly braises: THREE.Sprite[] = [];
  private readonly lucioles: THREE.Mesh[] = [];
  /** Textures en bande verticale (flammes, braise du feu) et leur vitesse. */
  private readonly animees: { readonly texture: THREE.Texture; readonly images: number; readonly vitesse: number }[] = [];

  private progression = 0;
  private images_ = 0;
  /** Barrière posée après la dernière image : tant qu'elle n'est pas franchie, la carte graphique est occupée. */
  private barriere: WebGLSync | null = null;

  constructor(canvas: OffscreenCanvas, images: ImagesFeuDeCamp, quadsFeu: readonly Quad[], heros: HerosFeuDeCamp) {
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: false, powerPreference: 'low-power' });
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    // Ombres recalculées une image sur quelques-unes : tout bouge lentement
    this.renderer.shadowMap.autoUpdate = false;
    this.images = images;
    this.construire(quadsFeu, heros);
  }

  /** Taille d'affichage, en pixels. */
  redimensionner(largeur: number, hauteur: number): void {
    if (largeur < 1 || hauteur < 1) return;
    this.renderer.setSize(largeur, hauteur, false);
    this.camera.aspect = largeur / hauteur;
    this.camera.updateProjectionMatrix();
  }

  /** Avancement du chargement (0 à 1) : le cookie du zombie dore avec lui. */
  definirProgression(part: number): void {
    this.progression = part;
  }

  /** Change le héros assis au coin du feu (ses images doivent déjà être là). */
  changerHeros(heros: HerosFeuDeCamp): void {
    this.scene.remove(this.heros.mob.racine);
    this.heros = { mob: this.creerHeros(heros), angle: F.places.heros };
    this.scene.add(this.heros.mob.racine);
  }

  /**
   * Dessine l'image de l'instant `t` (secondes), sauf si la carte graphique
   * n'a pas fini la précédente : on ne fait jamais la queue derrière le
   * chargement de la partie (shaders, textures), qui passe en premier.
   */
  dessiner(t: number): boolean {
    const gl = this.renderer.getContext() as WebGL2RenderingContext;
    if (this.barriere) {
      if (gl.clientWaitSync(this.barriere, 0, 0) === gl.TIMEOUT_EXPIRED) return false;
      gl.deleteSync(this.barriere);
      this.barriere = null;
    }
    for (const a of this.animees) {
      const image = Math.floor(t * a.vitesse) % a.images;
      a.texture.offset.y = 1 - (image + 1) / a.images;
    }
    // La vue tourne lentement autour du feu
    const C = F.camera;
    const angle = t * C.vitesse + C.depart;
    this.camera.position.set(Math.sin(angle) * C.rayon, C.hauteur + Math.sin(t * 0.25) * 0.25, Math.cos(angle) * C.rayon);
    this.camera.lookAt(0, C.visee, 0);

    const L = F.lumieres;
    const vacille = Math.sin(t * 11) * 0.25 + Math.sin(t * 6.3) * 0.2 + Math.sin(t * 17.7) * 0.1;
    this.lumiereFeu.intensity = L.intensiteFeu * (1 + vacille * L.vacillement / 2.6);
    this.lueur.material.opacity = 0.33 + vacille * 0.08;
    this.lumiereLanterne.intensity = L.intensiteLanterne * (1 + Math.sin(t * 3.1) * 0.05);
    this.animerFeu(t);
    this.animerMobs(t);
    this.renderer.shadowMap.needsUpdate = this.images_++ % F.ombresToutesLes === 0;
    this.renderer.render(this.scene, this.camera);
    this.barriere = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    gl.flush();
    return true;
  }

  private tex(chemin: string): THREE.Texture | null {
    return this.images.textures.get(chemin) ?? null;
  }

  /** Objet en relief (plaque d'un pixel d'épaisseur, comme les objets tenus en jeu), de côté `taille` blocs. */
  private objetRelief(chemin: string, taille: number): THREE.Mesh<THREE.BufferGeometry, THREE.MeshLambertMaterial> | null {
    const pixels = this.images.pixels.get(chemin);
    const texture = this.tex(chemin);
    if (!pixels || !texture) return null;
    const geometrie = creerGeometrieObjet(pixels.data, pixels.width, pixels.height, taille / pixels.width);
    const maillage = new THREE.Mesh(geometrie, new THREE.MeshLambertMaterial({ map: texture, alphaTest: 0.5, side: THREE.DoubleSide }));
    maillage.castShadow = true;
    return maillage;
  }

  // --- Construction -------------------------------------------------------------------------------------
  private construire(quadsFeu: readonly Quad[], heros: HerosFeuDeCamp): void {
    const scene = this.scene;
    const L = F.lumieres;
    scene.background = dessin(2, 256, (ctx) => {
      const g = ctx.createLinearGradient(0, 0, 0, 256);
      g.addColorStop(0, F.ciel.haut);
      g.addColorStop(1, F.ciel.bas);
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, 2, 256);
    });
    (scene.background as THREE.Texture).colorSpace = THREE.SRGBColorSpace;
    scene.fog = new THREE.Fog(F.brouillard.couleur, F.brouillard.debut, F.brouillard.fin);
    scene.add(new THREE.HemisphereLight(L.ciel, L.sol, L.ambiance));
    const lune = new THREE.DirectionalLight(L.lune, L.intensiteLune);
    lune.position.set(-8, 14, -6);
    scene.add(lune);
    const lambert: Lambert = (chemin, opts = {}) =>
      new THREE.MeshLambertMaterial({ map: this.tex(chemin), alphaTest: 0.5, side: THREE.DoubleSide, ...opts });

    this.construireClairiere(lambert);
    this.construireCampement(lambert);
    this.construireFeu(quadsFeu, lambert);

    // Les bancs et les mobs assis en rond, tournés vers le feu
    const B = F.bancs;
    const banc = new THREE.BoxGeometry(B.longueur, B.epaisseur, B.epaisseur);
    const bout = lambert(T.bucheDessus), ecorce = lambert(T.buche);
    for (const angle of Object.values(F.places)) {
      const b = new THREE.Mesh(banc, [bout, bout, ecorce, ecorce, ecorce, ecorce]);
      b.position.set(Math.sin(angle) * B.rayon, 0.5 + B.epaisseur / 2, Math.cos(angle) * B.rayon);
      b.rotation.y = angle + Math.PI / 2;
      b.castShadow = true;
      b.receiveShadow = true;
      scene.add(b);
    }
    this.heros = { mob: this.creerHeros(heros), angle: F.places.heros };
    this.zombie = { mob: construireMob([{ definition: MODELES.zombie, texture: this.tex(T.zombie) }]), angle: F.places.zombie };
    this.squelette = { mob: construireMob([{ definition: MODELES.squelette, texture: this.tex(T.squelette) }]), angle: F.places.squelette };
    for (const place of [this.zombie, this.squelette]) scene.add(place.mob.racine);
    scene.add(this.heros.mob.racine);

    // Le zombie tend un bâton au-dessus des flammes, un cookie au bout ; sa diagonale prolonge le bras
    const baton = this.objetRelief(T.baton, F.baton.taille);
    if (baton) {
      baton.rotation.x = Math.PI * 3 / 4;
      baton.position.set(-1 / PIXELS_PAR_BLOC, -1, 0);
      this.zombie.mob.parties.brasDroit?.add(baton);
      this.cookie = this.objetRelief(T.cookie, F.baton.cookie);
      if (this.cookie) {
        this.cookie.position.set(0.02, F.baton.taille * 0.33, F.baton.taille * 0.33);
        baton.add(this.cookie);
      }
    }
    // Le squelette tient un os, comme il tiendrait son arc
    const mainSquelette = MODELES.squelette.main;
    const os = mainSquelette ? this.objetRelief(T.os, mainSquelette.taille / PIXELS_PAR_BLOC) : null;
    if (os && mainSquelette) tenir(this.squelette.mob, mainSquelette, os);

    // Le mouton couché contre le héros (pattes repliées, tête posée), des « z » au-dessus de sa tête
    const M = F.mouton;
    this.mouton = construireMob([
      { definition: MODELE_MOUTON, texture: this.tex(T.mouton) },
      { definition: MODELE_LAINE_MOUTON, texture: this.tex(T.laineMouton) },
    ]);
    for (const [nom, partie] of Object.entries(this.mouton.parties)) partie.visible = !nom.startsWith('patte');
    this.mouton.parties.tete?.position.set(0, 12 / PIXELS_PAR_BLOC, 11 / PIXELS_PAR_BLOC);
    this.mouton.parties.tete?.rotation.set(0.2, 0, 0);
    this.mouton.racine.position.set(M.x, 0.5 - 12 / PIXELS_PAR_BLOC + 1.75 / PIXELS_PAR_BLOC, M.z);
    this.mouton.racine.rotation.y = M.cap;
    scene.add(this.mouton.racine);
    const dodo = new THREE.Group();
    dodo.position.set(M.x + Math.sin(M.cap) * 0.7, 1.15, M.z + Math.cos(M.cap) * 0.7);
    const materiauZ = spriteZ();
    for (let k = 0; k < 3; k++) {
      const s = new THREE.Sprite(materiauZ.clone());
      this.lettresZ.push(s);
      dodo.add(s);
    }
    scene.add(dodo);

    this.creeper = construireMob([{ definition: MODELES.creeper, texture: this.tex(T.creeper) }]);
    this.creeper.racine.rotation.y = F.creeper.angle + Math.PI;
    scene.add(this.creeper.racine);

    // Lucioles
    const luciole = new THREE.BoxGeometry(0.06, 0.06, 0.06);
    const materiauLuciole = new THREE.MeshBasicMaterial({ color: 0xd8ff7a, fog: false });
    for (let k = 0; k < F.lucioles; k++) {
      const l = new THREE.Mesh(luciole, materiauLuciole);
      this.lucioles.push(l);
      scene.add(l);
    }
  }

  private creerHeros(heros: HerosFeuDeCamp): Mob {
    const definition = MODELES[heros.modele];
    const mob = construireMob([{ definition, texture: this.tex(heros.skin) }]);
    const objet = heros.objetEnMain && definition.main
      ? this.objetRelief(heros.objetEnMain, definition.main.taille / PIXELS_PAR_BLOC)
      : null;
    if (objet && definition.main) tenir(mob, definition.main, objet);
    return mob;
  }

  private construireClairiere(lambert: Lambert): void {
    const C = F.clairiere;
    const cube = new THREE.BoxGeometry(1, 1, 1);
    const dessus: [number, number, number][] = [];
    const chemin: [number, number, number][] = [];
    for (let x = -C.rayon; x <= C.rayon; x++) {
      for (let z = -C.rayon; z <= C.rayon; z++) {
        const d = Math.hypot(x, z);
        if (d > C.rayon) continue;
        (d < C.rayonTerre + hasard(x * 13 + z) * 0.6 ? chemin : dessus).push([x, 0, z]);
      }
    }
    const cote = lambert(T.herbeCote), terre = lambert(T.terre);
    const herbe = lambert(T.herbeDessus, { color: F.teintes.herbe });
    this.scene.add(instances(cube, [cote, cote, herbe, terre, cote, cote], dessus, false));
    this.scene.add(instances(cube, lambert(T.terreBattue), chemin, false));

    // Un cercle d'arbres, des fleurs et des champignons à leurs pieds, des touffes d'herbe
    const troncs: [number, number, number][] = [];
    const feuilles: [number, number, number][] = [];
    const deco: [number, number, number][][] = DECO_FEU_DE_CAMP.map(() => []);
    for (let k = 0; k < C.arbres; k++) {
      const angle = (k / C.arbres) * Math.PI * 2 + hasard(k) * 0.2;
      const d = C.distanceArbres[0] + hasard(k + 3) * (C.distanceArbres[1] - C.distanceArbres[0]);
      const x = Math.round(Math.sin(angle) * d), z = Math.round(Math.cos(angle) * d), h = 4 + Math.floor(hasard(k + 8) * 2);
      for (let y = 1; y <= h; y++) troncs.push([x, y, z]);
      feuillage(x, h - 1, z, feuilles);
      for (let n = 0; n < 2; n++) {
        const dx = Math.round((hasard(k * 5 + n) - 0.5) * 4), dz = Math.round((hasard(k * 9 + n) - 0.5) * 4);
        if (dx !== 0 || dz !== 0) deco[Math.floor(hasard(k + n * 0.3) * deco.length)].push([x + dx, 0.5, z + dz]);
      }
    }
    const bout = lambert(T.bucheDessus), ecorce = lambert(T.buche);
    this.scene.add(instances(cube, [ecorce, ecorce, bout, bout, ecorce, ecorce], troncs, false));
    this.scene.add(instances(cube, lambert(T.feuilles, { color: F.teintes.feuilles }), feuilles, false));
    const croix = geometrieCroix();
    const touffes: [number, number, number][] = [];
    for (let k = 0; k < C.touffes; k++) {
      const x = Math.round((hasard(k * 2.1) - 0.5) * 2 * C.rayon), z = Math.round((hasard(k * 4.3) - 0.5) * 2 * C.rayon);
      const d = Math.hypot(x, z);
      if (d >= C.rayonLibre && d <= C.rayon) touffes.push([x, 0.5, z]);
    }
    this.scene.add(instances(croix, lambert(T.touffe, { color: F.teintes.herbe }), touffes, false));
    DECO_FEU_DE_CAMP.forEach((chemin, i) => this.scene.add(instances(croix, lambert(chemin), deco[i], false)));

    // Étoiles et lune
    const etoiles: number[] = [];
    for (let k = 0; k < F.etoiles; k++) {
      const a = hasard(k) * Math.PI * 2, h = 0.15 + hasard(k + 0.3) * 0.8;
      etoiles.push(Math.cos(a) * Math.cos(h) * 60, Math.sin(h) * 60, Math.sin(a) * Math.cos(h) * 60);
    }
    const geometrieEtoiles = new THREE.BufferGeometry();
    geometrieEtoiles.setAttribute('position', new THREE.Float32BufferAttribute(etoiles, 3));
    this.scene.add(new THREE.Points(geometrieEtoiles, new THREE.PointsMaterial({ color: 0xffffff, size: 2, sizeAttenuation: false, fog: false })));
    const imageLune = this.tex(T.lune)?.clone() ?? null;
    if (imageLune) {
      // Le disque de la pleine lune occupe les pixels 10 à 22 de son image de 32
      imageLune.repeat.set(12 / 32, 12 / 32);
      imageLune.offset.set(10 / 32, 10 / 32);
      const lune = new THREE.Mesh(new THREE.PlaneGeometry(5, 5), new THREE.MeshBasicMaterial({ map: imageLune, fog: false, transparent: true }));
      lune.position.set(-30, 26, -40);
      lune.lookAt(0, 0, 0);
      this.scene.add(lune);
    }
  }

  private construireCampement(lambert: Lambert): void {
    // La tente : deux pans de laine et un fond
    const Te = F.tente;
    const tente = new THREE.Group();
    const laine = lambert(T.laineTente, { color: F.teintes.laineTente });
    for (const s of [-1, 1]) {
      const pan = new THREE.Mesh(new THREE.BoxGeometry(0.06, Te.cote, Te.profondeur), laine);
      pan.rotation.z = s * Te.pente;
      pan.position.set(s * Math.sin(Te.pente) * Te.cote / 2, Math.cos(Te.pente) * Te.cote / 2, 0);
      pan.castShadow = true;
      pan.receiveShadow = true;
      tente.add(pan);
    }
    const demi = Math.sin(Te.pente) * Te.cote, haut = Math.cos(Te.pente) * Te.cote;
    const forme = new THREE.Shape([new THREE.Vector2(-demi, 0), new THREE.Vector2(demi, 0), new THREE.Vector2(0, haut)]);
    const fond = new THREE.Mesh(new THREE.ShapeGeometry(forme), new THREE.MeshLambertMaterial({ color: F.teintes.fondTente, side: THREE.DoubleSide }));
    fond.position.z = -Te.profondeur / 2;
    tente.add(fond);
    tente.position.set(Math.sin(Te.angle) * Te.distance, 0.5, Math.cos(Te.angle) * Te.distance);
    tente.rotation.y = Te.angle + Math.PI;
    this.scene.add(tente);

    // Une pile de bûches
    const bout = lambert(T.bucheDessus), ecorce = lambert(T.buche);
    const buche = new THREE.BoxGeometry(1.2, 0.3, 0.3);
    const P = F.pile;
    const px = Math.sin(P.angle) * P.distance, pz = Math.cos(P.angle) * P.distance;
    const tx = Math.cos(P.angle) * 0.31, tz = -Math.sin(P.angle) * 0.31;
    for (const [k, etage] of [[-1, 0], [0, 0], [1, 0], [-0.5, 1], [0.5, 1], [0, 2]] as const) {
      const b = new THREE.Mesh(buche, [bout, bout, ecorce, ecorce, ecorce, ecorce]);
      b.position.set(px + tx * k, 0.65 + etage * 0.29, pz + tz * k);
      b.rotation.y = P.angle + Math.PI / 2;
      b.castShadow = true;
      b.receiveShadow = true;
      this.scene.add(b);
    }

    // Une souche, une lanterne posée dessus
    const S = F.souche;
    const sx = Math.sin(S.angle) * S.distance, sz = Math.cos(S.angle) * S.distance;
    const souche = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), [ecorce, ecorce, bout, bout, ecorce, ecorce]);
    souche.position.set(sx, 1, sz);
    souche.castShadow = true;
    souche.receiveShadow = true;
    this.scene.add(souche);
    const fer = new THREE.MeshLambertMaterial({ color: 0x34343c });
    const lanterne = new THREE.Group();
    const morceau = (l: number, h: number, p: number, y: number, materiau: THREE.Material) => {
      const m = new THREE.Mesh(new THREE.BoxGeometry(l, h, p), materiau);
      m.position.y = y;
      lanterne.add(m);
    };
    morceau(0.3, 0.34, 0.3, 0.23, new THREE.MeshBasicMaterial({ color: 0xf0902a }));
    morceau(0.36, 0.08, 0.36, 0.43, fer);
    morceau(0.36, 0.06, 0.36, 0.03, fer);
    morceau(0.12, 0.1, 0.04, 0.52, fer);
    lanterne.position.set(sx, 1.5, sz);
    this.scene.add(lanterne);
    const L = F.lumieres;
    this.lumiereLanterne = new THREE.PointLight(L.lanterne, L.intensiteLanterne, L.porteeLanterne, 1.5);
    this.lumiereLanterne.position.set(sx, 1.85, sz);
    this.scene.add(this.lumiereLanterne);
  }

  private construireFeu(quads: readonly Quad[], lambert: Lambert): void {
    // Le vrai modèle du bloc : bûches, braise et flammes (ces deux-là brillent dans le noir, comme en jeu)
    const feu = new THREE.Group();
    feu.position.y = 0.5;
    feu.scale.setScalar(F.tailleFeu);
    for (const [chemin, geometrie] of geometriesBloc(quads)) {
      const flammes = chemin.includes('fire');
      const brille = flammes || chemin.includes('lit');
      const texture = this.tex(chemin);
      if (texture && texture.repeat.y < 1) {
        this.animees.push({ texture, images: Math.round(1 / texture.repeat.y), vitesse: flammes ? F.vitesseFlammes : F.vitesseBraisesFeu });
      }
      const materiau = brille
        ? new THREE.MeshBasicMaterial({ map: texture, alphaTest: 0.3, side: THREE.DoubleSide })
        : lambert(chemin);
      const maillage = new THREE.Mesh(geometrie, materiau);
      maillage.castShadow = !flammes;
      maillage.receiveShadow = true;
      feu.add(maillage);
    }
    this.scene.add(feu);

    const L = F.lumieres;
    this.lumiereFeu = new THREE.PointLight(L.feu, L.intensiteFeu, L.porteeFeu, L.attenuationFeu);
    this.lumiereFeu.position.set(0, 1.6, 0);
    this.lumiereFeu.castShadow = true;
    this.lumiereFeu.shadow.mapSize.set(512, 512);
    this.lumiereFeu.shadow.bias = -0.002;
    this.scene.add(this.lumiereFeu);
    const halo = dessin(64, 64, (ctx) => {
      const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
      g.addColorStop(0, 'rgba(255, 140, 50, 1)');
      g.addColorStop(1, 'rgba(255, 140, 50, 0)');
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, 64, 64);
    });
    this.lueur = new THREE.Mesh(
      new THREE.PlaneGeometry(8, 8),
      new THREE.MeshBasicMaterial({ map: halo, transparent: true, opacity: 0.35, blending: THREE.AdditiveBlending, depthWrite: false }),
    );
    this.lueur.rotation.x = -Math.PI / 2;
    this.lueur.position.y = 0.52;
    this.scene.add(this.lueur);

    // Grande fumée grise qui monte, et quelques braises qui sautent
    for (const chemin of FUMEE_FEU_DE_CAMP) this.imagesFumee.push(this.tex(chemin));
    for (let k = 0; k < F.fumee.nombre; k++) {
      const s = new THREE.Sprite(new THREE.SpriteMaterial({ map: this.imagesFumee[0], color: F.fumee.couleur, transparent: true, depthWrite: false }));
      this.fumees.push(s);
      this.scene.add(s);
    }
    for (let k = 0; k < F.braises.nombre; k++) {
      const s = new THREE.Sprite(new THREE.SpriteMaterial({ map: this.tex(T.braise), transparent: true, depthWrite: false, fog: false }));
      s.scale.setScalar(F.braises.taille);
      this.braises.push(s);
      this.scene.add(s);
    }
  }

  // --- Animation ----------------------------------------------------------------------------------------
  private animerFeu(t: number): void {
    const Fu = F.fumee;
    this.fumees.forEach((s, k) => {
      const age = ((t + k * Fu.duree / Fu.nombre) % Fu.duree) / Fu.duree;
      s.position.set(Math.sin(t * 0.3 + k) * 0.25 + age * 0.9, 1.6 + age * Fu.montee, Math.cos(t * 0.27 + k * 2) * 0.25 - age * 0.4);
      s.scale.setScalar(0.55 + age * 0.9);
      s.material.map = this.imagesFumee[Math.min(this.imagesFumee.length - 1, Math.floor(age * this.imagesFumee.length))];
      s.material.opacity = Math.min(1, age * 6) * (1 - age) * Fu.opacite;
    });
    const Br = F.braises;
    this.braises.forEach((s, k) => {
      const phase = t / Br.duree + k / Br.nombre;
      const age = phase % 1, graine = Math.floor(phase) * Br.nombre + k;
      const vx = (hasard(graine) - 0.5) * 1.2, vz = (hasard(graine + 0.5) - 0.5) * 1.2;
      s.position.set(vx * age, 0.9 + age * 2.2 - age * age * 1.8, vz * age);
      s.material.opacity = 1 - age;
      s.visible = hasard(graine + 0.9) > 0.35;
    });
    this.lucioles.forEach((l, k) => {
      const r = 4.5 + hasard(k) * 6, angle = hasard(k + 2) * 6.3 + t * 0.1 * (k % 2 ? 1 : -1);
      l.position.set(Math.sin(angle) * r, 1 + Math.sin(t * 0.8 + k) * 0.6 + hasard(k + 4) * 1.5, Math.cos(angle) * r);
      l.visible = Math.sin(t * 2 + k * 1.7) > -0.3;
    });
  }

  private animerMobs(t: number): void {
    // Assis sur les bancs, tournés vers le feu, les jambes en avant
    const B = F.bancs;
    for (const { mob, angle } of [this.heros, this.zombie, this.squelette]) {
      mob.racine.position.set(Math.sin(angle) * B.assis, 0.5 + B.epaisseur - 0.73, Math.cos(angle) * B.assis);
      mob.racine.rotation.y = angle + Math.PI;
      mob.parties.jambeDroite?.rotation.set(-Math.PI / 2, 0.12, 0);
      mob.parties.jambeGauche?.rotation.set(-Math.PI / 2, -0.12, 0);
    }

    // Le héros caresse le mouton et le regarde
    const h = this.heros.mob.parties;
    h.brasGauche?.rotation.set(-0.55 + Math.sin(t * 2.2) * 0.13, 0, 0.42);
    h.brasDroit?.rotation.set(-0.3, 0, 0);
    h.tete?.rotation.set(0.35, -0.35, 0);

    // Le squelette compte ses os : il lève l'os en rythme, hoche la tête, tape du pied
    const s = this.squelette.mob.parties;
    const compte = Math.max(0, Math.sin(t * 7)) ** 3;
    s.brasDroit?.rotation.set(-0.75 - compte * 0.45, 0.1, 0);
    s.brasGauche?.rotation.set(-0.4, -0.3, 0);
    s.tete?.rotation.set(compte * 0.25, 0, 0);
    s.jambeGauche?.rotation.set(-Math.PI / 2 + Math.max(0, Math.sin(t * 7)) * 0.25, -0.12, 0);
    this.squelette.mob.racine.rotation.z = Math.sin(t * 3.5) * 0.04;

    // Le zombie fait tourner son cookie au-dessus des flammes ; le cookie dore avec la barre
    const zb = this.zombie.mob.parties;
    zb.brasDroit?.rotation.set(-1.22 + Math.sin(t * 1.4) * 0.05, 0, 0.12 + Math.sin(t * 2.8) * 0.04);
    zb.brasGauche?.rotation.set(-0.2, 0, 0);
    zb.tete?.rotation.set(0.15, 0, 0);
    this.cookie?.material.color.setScalar(1 - 0.55 * this.progression);

    // Le mouton dort : il respire, des « z » montent
    this.mouton.racine.scale.set(1, 1 + Math.sin(t * 1.5) * 0.02, 1);
    this.lettresZ.forEach((z, k) => {
      const a = (t * 0.45 + k / 3) % 1;
      z.position.set(Math.sin(a * 7) * 0.15 + a * 0.3, a * 1.2, 0);
      z.material.opacity = Math.min(1, a * 4) * (1 - lisse(0.7, 1, a));
      z.scale.setScalar(0.18 + a * 0.22);
    });

    // Le creeper avance d'un pas, sent la chaleur, tremble et recule vite
    const Cr = F.creeper;
    const c = t % Cr.cycle;
    const avance = lisse(0.5, 2.5, c) * (1 - lisse(3, 3.4, c));
    const dist = Cr.distance - avance * Cr.avance;
    const cr = this.creeper.racine;
    cr.position.set(Math.sin(Cr.angle) * dist, 0.5, Math.cos(Cr.angle) * dist);
    const marche = (c > 0.5 && c < 2.5) || (c > 3 && c < 3.4) ? Math.sin(t * 12) * 0.6 : 0;
    const cp = this.creeper.parties;
    cp.patteArriereDroite?.rotation.set(marche, 0, 0);
    cp.patteAvantGauche?.rotation.set(marche, 0, 0);
    cp.patteArriereGauche?.rotation.set(-marche, 0, 0);
    cp.patteAvantDroite?.rotation.set(-marche, 0, 0);
    const tremble = c > 2.5 && c < 3;
    cr.rotation.z = tremble ? Math.sin(t * 50) * 0.05 : 0;
    cp.tete?.rotation.set(0, 0, tremble ? Math.sin(t * 40) * 0.1 : 0);
  }
}
