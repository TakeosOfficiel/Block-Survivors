/**
 * Scène 3D de l'écran de chargement : « La pause des mobs » (voir
 * donnees/feuDeCamp.ts). Le soir, autour du vrai feu de camp du jeu, le héros
 * caresse un mouton endormi, le zombie fait griller un cookie, le squelette
 * compte ses os et le creeper n'ose pas approcher. La vue tourne lentement.
 * À 100 %, tout le monde se lève et se tourne vers nous.
 *
 * Un petit moteur WebGL séparé, sur son propre canvas sous le texte du
 * chargement : il démarre tout de suite (pas de compilation de shaders
 * WebGPU à attendre) et ne dessine que lorsque l'écran est affiché. Les mobs
 * reprennent les modèles du jeu (donnees/modeles.ts) et le feu de camp son
 * modèle de bloc vanilla.
 */
import * as THREE from 'three';
import { lirePixels } from '../assets/pixels';
import type { BibliothequeTextures } from '../assets/textures';
import {
  COUCHES_JOUEUR, DECO_FEU_DE_CAMP, FEU_DE_CAMP, FUMEE_FEU_DE_CAMP, MODELE_LAINE_MOUTON, MODELE_MOUTON, TEXTURES_FEU_DE_CAMP,
} from '../donnees/feuDeCamp';
import { MODELES } from '../donnees/modeles';
import type { ApparenceEntite, DefinitionModele, MainModele } from '../donnees/types';
import { BibliothequeModeles, type Quad } from '../minecraft/modeles';
import { creerGeometrieCube, creerGeometrieObjet, PIXELS_PAR_BLOC } from '../moteur/geometrie';

const T = TEXTURES_FEU_DE_CAMP;
const F = FEU_DE_CAMP;

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
/** Ramène un angle dans [−π, π]. */
function ecartAngle(a: number): number {
  return Math.atan2(Math.sin(a), Math.cos(a));
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

/** Objet en relief (plaque d'un pixel d'épaisseur, comme les objets tenus en jeu), de côté `taille` blocs. */
function objetRelief(texture: THREE.Texture | null, taille: number): THREE.Mesh | null {
  const image = texture?.image as (CanvasImageSource & { width: number; height: number }) | undefined;
  if (!texture || !image) return null;
  try {
    const pixels = lirePixels(image);
    const geometrie = creerGeometrieObjet(pixels.data, image.width, image.height, taille / image.width);
    const maillage = new THREE.Mesh(geometrie, new THREE.MeshLambertMaterial({ map: texture, alphaTest: 0.5, side: THREE.DoubleSide }));
    maillage.castShadow = true;
    return maillage;
  } catch {
    return null;  // image illisible : pas d'objet plutôt qu'une erreur
  }
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

/** Petit dessin en pixels (« z » du dormeur, « ! » du réveil) avec son ombre portée, comme le texte du jeu. */
function spritePixels(motif: readonly string[], couleur: string, ombre: string): THREE.SpriteMaterial {
  const toile = document.createElement('canvas');
  toile.width = 8;
  toile.height = 8;
  const ctx = toile.getContext('2d');
  if (ctx) {
    for (const [teinte, decalage] of [[ombre, 1], [couleur, 0]] as const) {
      ctx.fillStyle = teinte;
      motif.forEach((ligne, y) => [...ligne].forEach((c, x) => { if (c === '#') ctx.fillRect(x + decalage, y + decalage, 1, 1); }));
    }
  }
  const texture = new THREE.CanvasTexture(toile);
  texture.magFilter = THREE.NearestFilter;
  texture.minFilter = THREE.NearestFilter;
  texture.colorSpace = THREE.SRGBColorSpace;
  return new THREE.SpriteMaterial({ map: texture, transparent: true, depthWrite: false });
}
const MOTIF_Z = ['#####', '...#.', '..#..', '.#...', '#####'];
const MOTIF_ALERTE = ['#', '#', '#', '#', '#', '.', '#'];

/** Dégradé vertical du ciel. */
function degrade(haut: string, bas: string): THREE.CanvasTexture {
  const toile = document.createElement('canvas');
  toile.width = 2;
  toile.height = 256;
  const ctx = toile.getContext('2d');
  if (ctx) {
    const g = ctx.createLinearGradient(0, 0, 0, 256);
    g.addColorStop(0, haut);
    g.addColorStop(1, bas);
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 2, 256);
  }
  const texture = new THREE.CanvasTexture(toile);
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}

/** Tache de lumière ronde (lueur du feu sur le sol). */
function halo(): THREE.CanvasTexture {
  const toile = document.createElement('canvas');
  toile.width = 64;
  toile.height = 64;
  const ctx = toile.getContext('2d');
  if (ctx) {
    const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
    g.addColorStop(0, 'rgba(255, 140, 50, 1)');
    g.addColorStop(1, 'rgba(255, 140, 50, 0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 64, 64);
  }
  return new THREE.CanvasTexture(toile);
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

/** Les trois mobs assis autour du feu. */
interface Place {
  readonly mob: Mob;
  readonly angle: number;
}

export class SceneChargement {
  private readonly renderer: THREE.WebGLRenderer;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(F.camera.champ, 16 / 9, 0.1, 200);
  private readonly textures: BibliothequeTextures;
  private readonly conteneur: HTMLElement;
  private readonly tex = (chemin: string): THREE.Texture | null => this.textures.obtenir(chemin);

  private heros!: Place;
  private zombie!: Place;
  private squelette!: Place;
  private mouton!: Mob;
  private creeper!: Mob;
  private cookie: THREE.Object3D | null = null;
  private readonly miettes: THREE.Mesh[] = [];
  private readonly posCookie = new THREE.Vector3();
  private readonly dodo = new THREE.Group();
  private readonly lettresZ: THREE.Sprite[] = [];
  private alerte!: THREE.Sprite;
  private lumiereFeu!: THREE.PointLight;
  private lumiereLanterne!: THREE.PointLight;
  private lueur!: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>;
  private readonly fumees: THREE.Sprite[] = [];
  private readonly imagesFumee: (THREE.Texture | null)[] = [];
  private readonly braises: THREE.Sprite[] = [];
  private readonly lucioles: THREE.Mesh[] = [];

  private progression = 0;
  private readonly debut = performance.now();
  private dernier = 0;
  private boucle = 0;
  private images = 0;
  private actif = false;
  /** Instant (s) où la fin à 100 % a commencé, ou null. */
  private finDepuis: number | null = null;

  private constructor(renderer: THREE.WebGLRenderer, conteneur: HTMLElement, textures: BibliothequeTextures) {
    this.renderer = renderer;
    this.conteneur = conteneur;
    this.textures = textures;
    new ResizeObserver(() => this.redimensionner()).observe(conteneur);
  }

  /**
   * Prépare la scène dans `conteneur` (textures, modèle du feu de camp) ;
   * null si WebGL est indisponible : l'écran garde alors son fond de terre.
   */
  static async creer(conteneur: HTMLElement, textures: BibliothequeTextures, heros: ApparenceEntite): Promise<SceneChargement | null> {
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: false, powerPreference: 'low-power' });
    } catch {
      return null;
    }
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    // Ombres recalculées une image sur quelques-unes : tout bouge lentement
    renderer.shadowMap.autoUpdate = false;
    renderer.setPixelRatio(1);
    renderer.domElement.className = 'chargement-scene';

    // Le modèle du feu de camp et les textures de la scène en même temps
    const [[bloc]] = await Promise.all([
      new BibliothequeModeles().cuire([F.bloc]),
      textures.charger([
        ...Object.values(T), ...DECO_FEU_DE_CAMP, ...FUMEE_FEU_DE_CAMP,
        heros.skin, ...(heros.objetEnMain ? [heros.objetEnMain] : []),
      ]),
    ]);
    const texturesFeu = BibliothequeModeles.textures([bloc]);
    await textures.charger(texturesFeu);
    for (const chemin of texturesFeu) textures.animer(chemin, chemin.includes('fire') ? F.vitesseFlammes : F.vitesseBraisesFeu);

    const scene = new SceneChargement(renderer, conteneur, textures);
    scene.construire(bloc.parties[0]?.[0]?.modele.quads ?? [], heros);
    conteneur.prepend(renderer.domElement);
    scene.redimensionner();
    return scene;
  }

  /** Avancement du chargement (0 à 1) : le cookie du zombie dore avec lui. */
  definirProgression(part: number): void {
    this.progression = part;
  }

  /** Change le héros assis au coin du feu (classe ou skin choisis entre deux chargements). */
  async changerHeros(apparence: ApparenceEntite): Promise<void> {
    await this.textures.charger([apparence.skin, ...(apparence.objetEnMain ? [apparence.objetEnMain] : [])]);
    this.scene.remove(this.heros.mob.racine);
    this.heros = { mob: this.creerHeros(apparence), angle: F.places.heros };
    this.scene.add(this.heros.mob.racine);
  }

  demarrer(): void {
    if (this.actif) return;
    this.actif = true;
    this.finDepuis = null;
    this.dernier = performance.now();
    const image = (maintenant: number) => {
      if (!this.actif) return;
      // Cadence plafonnée : le chargement de la partie garde la main
      if (maintenant - this.dernier >= 1000 / F.imagesParSeconde - 2) this.dessiner(maintenant);
      this.boucle = requestAnimationFrame(image);
    };
    this.boucle = requestAnimationFrame(image);
  }

  arreter(): void {
    this.actif = false;
    cancelAnimationFrame(this.boucle);
  }

  /** La fin à 100 % (tout le monde se lève, le mouton se réveille) ; se résout quand elle est jouée. */
  finale(): Promise<void> {
    if (!this.actif) return Promise.resolve();
    this.finDepuis = (performance.now() - this.debut) / 1000;
    return new Promise((resoudre) => window.setTimeout(resoudre, F.finale.duree * 1000));
  }

  private redimensionner(): void {
    const { width, height } = this.conteneur.getBoundingClientRect();
    if (width < 1 || height < 1) return;
    this.renderer.setSize(width, height, false);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }

  // --- Construction -------------------------------------------------------------------------------------
  private construire(quadsFeu: readonly Quad[], heros: ApparenceEntite): void {
    const scene = this.scene;
    const L = F.lumieres;
    scene.background = degrade(F.ciel.haut, F.ciel.bas);
    scene.fog = new THREE.Fog(F.brouillard.couleur, F.brouillard.debut, F.brouillard.fin);
    scene.add(new THREE.HemisphereLight(L.ciel, L.sol, L.ambiance));
    const lune = new THREE.DirectionalLight(L.lune, L.intensiteLune);
    lune.position.set(-8, 14, -6);
    scene.add(lune);
    const lambert = (chemin: string, opts: THREE.MeshLambertMaterialParameters = {}) =>
      new THREE.MeshLambertMaterial({ map: this.tex(chemin), alphaTest: 0.5, side: THREE.DoubleSide, ...opts });

    this.construireClairiere(lambert);
    this.construireCampement(lambert);
    this.construireFeu(quadsFeu, lambert);

    // Les bancs et les mobs assis en rond
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
    for (const place of [this.heros, this.zombie, this.squelette]) scene.add(place.mob.racine);

    // Le zombie tend un bâton au-dessus des flammes, un cookie au bout ; sa diagonale prolonge le bras
    const baton = objetRelief(this.tex(T.baton), F.baton.taille);
    if (baton) {
      baton.rotation.x = Math.PI * 3 / 4;
      baton.position.set(-1 / PIXELS_PAR_BLOC, -1, 0);
      this.zombie.mob.parties.brasDroit?.add(baton);
      this.cookie = objetRelief(this.tex(T.cookie), F.baton.cookie);
      if (this.cookie) {
        this.cookie.position.set(0.02, F.baton.taille * 0.33, F.baton.taille * 0.33);
        baton.add(this.cookie);
      }
    }
    const miette = new THREE.BoxGeometry(0.06, 0.06, 0.06);
    for (let k = 0; k < 8; k++) {
      const m = new THREE.Mesh(miette, new THREE.MeshLambertMaterial({ color: k % 2 ? 0xa0642c : 0x5a3418 }));
      m.userData.direction = new THREE.Vector3(hasard(k + 60) - 0.5, hasard(k + 61) * 0.8, hasard(k + 62) - 0.5);
      m.visible = false;
      this.miettes.push(m);
      scene.add(m);
    }
    // Le squelette tient un os, comme il tiendrait son arc
    const os = objetRelief(this.tex(T.os), (MODELES.squelette.main?.taille ?? 12) / PIXELS_PAR_BLOC);
    if (os && MODELES.squelette.main) tenir(this.squelette.mob, MODELES.squelette.main, os);

    // Le mouton couché contre le héros, des « z » au-dessus de sa tête
    const M = F.mouton;
    this.mouton = construireMob([
      { definition: MODELE_MOUTON, texture: this.tex(T.mouton) },
      { definition: MODELE_LAINE_MOUTON, texture: this.tex(T.laineMouton) },
    ]);
    this.mouton.racine.position.set(M.x, 0, M.z);
    this.mouton.racine.rotation.y = M.cap;
    scene.add(this.mouton.racine);
    const tete = new THREE.Vector3(M.x + Math.sin(M.cap) * 0.7, 1.15, M.z + Math.cos(M.cap) * 0.7);
    const materiauZ = spritePixels(MOTIF_Z, '#ffffff', '#3a3a5a');
    for (let k = 0; k < 3; k++) {
      const s = new THREE.Sprite(materiauZ.clone());
      this.lettresZ.push(s);
      this.dodo.add(s);
    }
    this.dodo.position.copy(tete);
    scene.add(this.dodo);
    this.alerte = new THREE.Sprite(spritePixels(MOTIF_ALERTE, '#ffff55', '#3f3f15'));
    this.alerte.position.set(tete.x, 1.75, tete.z);
    this.alerte.visible = false;
    scene.add(this.alerte);

    this.creeper = construireMob([{ definition: MODELES.creeper, texture: this.tex(T.creeper) }]);
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

  private creerHeros(apparence: ApparenceEntite): Mob {
    const definition = MODELES[apparence.modele];
    const mob = construireMob([{ definition, texture: this.tex(apparence.skin) }]);
    const objet = apparence.objetEnMain && definition.main
      ? objetRelief(this.tex(apparence.objetEnMain), definition.main.taille / PIXELS_PAR_BLOC)
      : null;
    if (objet && definition.main) tenir(mob, definition.main, objet);
    return mob;
  }

  private construireClairiere(lambert: (chemin: string, opts?: THREE.MeshLambertMaterialParameters) => THREE.MeshLambertMaterial): void {
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

  private construireCampement(lambert: (chemin: string, opts?: THREE.MeshLambertMaterialParameters) => THREE.MeshLambertMaterial): void {
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

  private construireFeu(quads: readonly Quad[], lambert: (chemin: string, opts?: THREE.MeshLambertMaterialParameters) => THREE.MeshLambertMaterial): void {
    // Le vrai modèle du bloc : bûches, braise et flammes (ces deux-là brillent dans le noir, comme en jeu)
    const feu = new THREE.Group();
    feu.position.y = 0.5;
    feu.scale.setScalar(F.tailleFeu);
    for (const [chemin, geometrie] of geometriesBloc(quads)) {
      const brille = chemin.includes('fire') || chemin.includes('lit');
      const materiau = brille
        ? new THREE.MeshBasicMaterial({ map: this.tex(chemin), alphaTest: 0.3, side: THREE.DoubleSide })
        : lambert(chemin);
      const maillage = new THREE.Mesh(geometrie, materiau);
      maillage.castShadow = !chemin.includes('fire');
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
    this.lueur = new THREE.Mesh(
      new THREE.PlaneGeometry(8, 8),
      new THREE.MeshBasicMaterial({ map: halo(), transparent: true, opacity: 0.35, blending: THREE.AdditiveBlending, depthWrite: false }),
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
  private dessiner(maintenant: number): void {
    const dt = Math.min(0.05, (maintenant - this.dernier) / 1000);
    this.dernier = maintenant;
    const t = (maintenant - this.debut) / 1000;
    const fini = this.finDepuis !== null;
    const apres = fini ? t - (this.finDepuis ?? t) : 0;
    this.textures.majAnimations(t);

    // La vue tourne lentement autour du feu
    const C = F.camera;
    const a = t * C.vitesse + C.depart;
    this.camera.position.set(Math.sin(a) * C.rayon, C.hauteur + Math.sin(t * 0.25) * 0.25, Math.cos(a) * C.rayon);
    this.camera.lookAt(0, C.visee, 0);

    const L = F.lumieres;
    const vacille = Math.sin(t * 11) * 0.25 + Math.sin(t * 6.3) * 0.2 + Math.sin(t * 17.7) * 0.1;
    this.lumiereFeu.intensity = L.intensiteFeu * (1 + vacille * L.vacillement / 2.6);
    this.lueur.material.opacity = 0.33 + vacille * 0.08;
    this.lumiereLanterne.intensity = L.intensiteLanterne * (1 + Math.sin(t * 3.1) * 0.05);
    this.animerFeu(t);
    this.animerMobs(t, dt, fini, apres);
    this.renderer.shadowMap.needsUpdate = this.images++ % F.ombresToutesLes === 0;
    this.renderer.render(this.scene, this.camera);
  }

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

  private animerMobs(t: number, dt: number, fini: boolean, apres: number): void {
    const debout = fini ? lisse(0.4, 0.9, apres) : 0;
    const versCamera = lisse(0.6, 1.2, apres) * Math.min(1, dt * 6);
    const B = F.bancs;
    // Assis sur les bancs, tournés vers le feu ; à 100 % ils se lèvent, font un pas et se tournent vers nous
    for (const { mob, angle } of [this.heros, this.zombie, this.squelette]) {
      const p = mob.parties;
      const r = B.assis - debout * B.avanceDebout;
      const x = Math.sin(angle) * r, z = Math.cos(angle) * r;
      mob.racine.position.set(x, 0.5 + B.epaisseur - 0.73 + debout * 0.33, z);
      if (fini) mob.racine.rotation.y += ecartAngle(Math.atan2(this.camera.position.x - x, this.camera.position.z - z) - mob.racine.rotation.y) * versCamera;
      else mob.racine.rotation.y = angle + Math.PI;
      p.jambeDroite?.rotation.set(-Math.PI / 2 * (1 - debout), fini ? 0 : 0.12, 0);
      p.jambeGauche?.rotation.set(-Math.PI / 2 * (1 - debout), fini ? 0 : -0.12, 0);
    }

    // Le héros caresse le mouton ; à 100 % il fait coucou
    const h = this.heros.mob.parties;
    h.brasGauche?.rotation.set(fini ? -0.1 : -0.55 + Math.sin(t * 2.2) * 0.13, 0, fini ? 0 : 0.42);
    const coucou = fini ? lisse(1, 1.4, apres) : 0;
    h.brasDroit?.rotation.set(fini ? 0 : -0.3, 0, -coucou * (2.7 + Math.sin(t * 9) * 0.3));
    h.tete?.rotation.set(fini ? 0 : 0.35, fini ? 0 : -0.35, 0);

    // Le squelette compte ses os : il lève l'os en rythme, hoche la tête, tape du pied
    const s = this.squelette.mob.parties;
    const compte = Math.max(0, Math.sin(t * 7)) ** 3;
    s.brasDroit?.rotation.set(fini ? -0.2 : -0.75 - compte * 0.45, 0.1, 0);
    s.brasGauche?.rotation.set(fini ? 0 : -0.4, -0.3, 0);
    s.tete?.rotation.set(fini ? 0 : compte * 0.25, 0, 0);
    if (!fini) s.jambeGauche?.rotation.set(-Math.PI / 2 + Math.max(0, Math.sin(t * 7)) * 0.25, -0.12, 0);
    this.squelette.mob.racine.rotation.z = fini ? 0 : Math.sin(t * 3.5) * 0.04;

    // Le zombie fait dorer son cookie ; à 100 % il le lève bien haut et le croque
    const zb = this.zombie.mob.parties;
    const leve = fini ? lisse(0.2, 0.8, apres) : 0;
    zb.brasDroit?.rotation.set(-1.22 + Math.sin(t * 1.4) * 0.05 * (1 - leve) - leve * 1.5, 0, 0.12 + Math.sin(t * 2.8) * 0.04);
    zb.brasGauche?.rotation.set(fini ? -Math.PI / 2 : -0.2, 0, 0);
    zb.tete?.rotation.set(fini ? -leve * 0.4 : 0.15, 0, 0);
    if (this.cookie) {
      const mat = (this.cookie as THREE.Mesh<THREE.BufferGeometry, THREE.MeshLambertMaterial>).material;
      mat.color.setScalar(1 - 0.55 * this.progression);
      const croque = fini && apres > 1.3;
      if (fini && !croque) this.cookie.getWorldPosition(this.posCookie);
      this.cookie.visible = !croque;
      for (const m of this.miettes) {
        m.visible = croque && apres < 2.3;
        if (!m.visible) continue;
        const a = apres - 1.3;
        m.position.copy(m.userData.direction as THREE.Vector3).multiplyScalar(a * 1.6).add(this.posCookie);
        m.position.y -= a * a * 3;
      }
    }

    // Le mouton dort (des « z » montent) ; à 100 % « ! », il lève la tête, se met debout et fait un bond
    const mp = this.mouton.parties;
    this.dodo.visible = !fini;
    if (!fini) {
      this.lettresZ.forEach((z, k) => {
        const a = (t * 0.45 + k / 3) % 1;
        z.position.set(Math.sin(a * 7) * 0.15 + a * 0.3, a * 1.2, 0);
        z.material.opacity = Math.min(1, a * 4) * (1 - lisse(0.7, 1, a));
        z.scale.setScalar(0.18 + a * 0.22);
      });
    }
    this.alerte.visible = fini && apres > 0.1 && apres < 1.4;
    this.alerte.scale.setScalar(0.45 * Math.min(1, Math.max(0, apres - 0.1) * 6));
    const leveMouton = fini ? lisse(0.6, 0.9, apres) : 0;
    const couche = 0.5 - 12 / PIXELS_PAR_BLOC + 1.75 / PIXELS_PAR_BLOC;
    for (const nom of ['patteArriereDroite', 'patteArriereGauche', 'patteAvantDroite', 'patteAvantGauche']) {
      const patte = mp[nom];
      if (patte) patte.visible = leveMouton > 0;
    }
    mp.tete?.position.set(0, (12 + leveMouton * 6) / PIXELS_PAR_BLOC, (11 - leveMouton * 3) / PIXELS_PAR_BLOC);
    mp.tete?.rotation.set(0.2 - leveMouton * 0.4, 0, 0);
    const bond = apres > 1.1 && apres < 1.5 ? Math.sin((apres - 1.1) * Math.PI / 0.4) * 0.35 : 0;
    this.mouton.racine.position.y = couche + leveMouton * (0.5 - couche) + bond;
    this.mouton.racine.scale.set(1, fini ? 1 : 1 + Math.sin(t * 1.5) * 0.02, 1);

    // Le creeper avance d'un pas, sent la chaleur, tremble et recule vite ; à 100 % il sautille
    const Cr = F.creeper;
    const c = t % Cr.cycle;
    const avance = fini ? 0.2 : lisse(0.5, 2.5, c) * (1 - lisse(3, 3.4, c));
    const dist = Cr.distance - avance * Cr.avance;
    const saut = fini && apres > 0.5 && apres < 1.85 ? Math.abs(Math.sin(apres * 7)) * 0.35 : 0;
    const cr = this.creeper.racine;
    cr.position.set(Math.sin(Cr.angle) * dist, 0.5 + saut, Math.cos(Cr.angle) * dist);
    if (fini) cr.rotation.y += ecartAngle(Math.atan2(this.camera.position.x - cr.position.x, this.camera.position.z - cr.position.z) - cr.rotation.y) * Math.min(1, dt * 4);
    else cr.rotation.y = Cr.angle + Math.PI;
    const marche = !fini && ((c > 0.5 && c < 2.5) || (c > 3 && c < 3.4)) ? Math.sin(t * 12) * 0.6 : 0;
    const cp = this.creeper.parties;
    cp.patteArriereDroite?.rotation.set(marche, 0, 0);
    cp.patteAvantGauche?.rotation.set(marche, 0, 0);
    cp.patteArriereGauche?.rotation.set(-marche, 0, 0);
    cp.patteAvantDroite?.rotation.set(-marche, 0, 0);
    const tremble = !fini && c > 2.5 && c < 3;
    cr.rotation.z = tremble ? Math.sin(t * 50) * 0.05 : 0;
    cp.tete?.rotation.set(0, 0, tremble ? Math.sin(t * 40) * 0.1 : 0);
  }
}
