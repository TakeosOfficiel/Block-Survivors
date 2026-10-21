/**
 * Carte de l'écran de chargement, façon Minecraft : la vraie map de la
 * partie vue de dessus, posée en perspective comme une maquette qui tourne
 * lentement (rotation en CSS, fluide même quand le jeu travaille). Ses chunks
 * apparaissent un à un depuis le point de départ à mesure que la barre
 * avance, avec un petit « pop » ; la tête du héros se dresse au départ. En
 * fond, la même map, floue et assombrie.
 */
import { CONFIG } from '../config';
import type { VueDessus } from '../monde/vueDessus';
import { element } from './dom';

interface Chunk {
  readonly x: number;  // première colonne
  readonly z: number;
  readonly l: number;  // colonnes de large
  readonly p: number;
}

/** Courbe d'apparition avec un léger rebond (le chunk dépasse un peu puis se pose). */
function rebond(t: number): number {
  const k = 1.7;
  const u = t - 1;
  return 1 + (k + 1) * u * u * u + k * u * u;
}

export class CarteChargement {
  private readonly plateau = element('chargement-plateau');
  private readonly toile = element<HTMLCanvasElement>('chargement-carte');
  private readonly ctx = this.toile.getContext('2d');
  private readonly fond = element<HTMLCanvasElement>('chargement-fond');
  private readonly marqueur = element('chargement-tete');
  private readonly tete = element<HTMLCanvasElement>('chargement-tete-image');
  /** Pixels de canvas par colonne de la map. */
  private echelle = 1;
  private largeur = 0;
  private profondeur = 0;
  /** Chunks, du plus proche au plus loin du départ. */
  private chunks: Chunk[] = [];
  private image: HTMLCanvasElement | null = null;
  /** Avancement du chargement quand la vue est arrivée : les chunks se dévoilent sur le reste de la barre. */
  private partDepart = 0;
  /** Chunks à dévoiler (selon la barre), et déjà dévoilés (ils arrivent un par un, même si la barre saute). */
  private voulus = 0;
  private devoiles = 0;
  /** Instant (ms) où chaque chunk dévoilé est apparu. */
  private apparitions: number[] = [];
  private dernierDevoilement = 0;
  private boucle = 0;

  /** Taille de la map connue : sa grille de chunks vides s'affiche, en attendant ses couleurs. */
  annoncer(largeur: number, profondeur: number): void {
    const C = CONFIG.chargement.carte;
    this.largeur = largeur;
    this.profondeur = profondeur;
    this.echelle = Math.max(1, Math.floor(C.pixelsMax / Math.max(largeur, profondeur)));
    this.toile.width = largeur * this.echelle;
    this.toile.height = profondeur * this.echelle;
    // Des « chunks » de 16 colonnes, plus petits sur les petites maps pour garder une belle grille
    const taille = Math.max(C.tailleChunkMin, Math.min(16, Math.round(Math.max(largeur, profondeur) / C.chunksParCote)));
    this.chunks = [];
    for (let z = 0; z < profondeur; z += taille) {
      for (let x = 0; x < largeur; x += taille) {
        this.chunks.push({ x, z, l: Math.min(taille, largeur - x), p: Math.min(taille, profondeur - z) });
      }
    }
    this.image = null;
    this.voulus = 0;
    this.devoiles = 0;
    this.apparitions = [];
    this.trier(largeur / 2, profondeur / 2);
    this.marqueur.classList.add('cache');
    this.fond.classList.remove('visible');
    this.plateau.classList.remove('cache');
    this.dessiner();
  }

  /**
   * Couleurs de la map arrivées (`part` = avancement actuel) : les chunks
   * apparaissent à partir d'ici, en partant du départ du joueur, où se dresse
   * sa tête (`skin` : image de skin de joueur).
   */
  montrer(vue: VueDessus, part: number, skin: CanvasImageSource | null): void {
    if (vue.largeur !== this.largeur || vue.profondeur !== this.profondeur) this.annoncer(vue.largeur, vue.profondeur);
    const image = document.createElement('canvas');
    image.width = vue.largeur;
    image.height = vue.profondeur;
    image.getContext('2d')?.putImageData(new ImageData(new Uint8ClampedArray(vue.couleurs), vue.largeur, vue.profondeur), 0, 0);
    this.image = image;

    // En fond, la même map, floue (le flou est fait une fois pour toutes par le navigateur)
    this.fond.width = vue.largeur;
    this.fond.height = vue.profondeur;
    this.fond.getContext('2d')?.drawImage(image, 0, 0);
    this.fond.classList.add('visible');

    if (vue.depart) {
      this.trier(vue.depart.x, vue.depart.z);
      if (skin) this.placerTete(skin, vue.depart.x, vue.depart.z);
    }
    this.partDepart = part;
    this.voulus = 0;
    this.devoiles = 0;
    this.apparitions = [];
    this.progression(part);
  }

  /** Avancement du chargement (0 à 1) : les chunks correspondants vont apparaître. */
  progression(part: number): void {
    if (!this.image || this.chunks.length === 0) return;
    const reste = Math.max(1e-6, 1 - this.partDepart);
    this.voulus = Math.max(this.voulus, Math.min(this.chunks.length, Math.ceil(((part - this.partDepart) / reste) * this.chunks.length)));
    this.animer();
  }

  cacher(): void {
    this.plateau.classList.add('cache');
    this.fond.classList.remove('visible');
    cancelAnimationFrame(this.boucle);
    this.boucle = 0;
  }

  /** Du plus proche au plus loin du point (colonnes), comme le chargement des chunks autour du joueur. */
  private trier(cx: number, cz: number): void {
    const distance = (c: Chunk) => Math.hypot(c.x + c.l / 2 - cx, c.z + c.p / 2 - cz);
    this.chunks.sort((a, b) => distance(a) - distance(b));
  }

  /** Tête du héros (visage et chapeau de la skin), debout sur son point de départ. */
  private placerTete(skin: CanvasImageSource, x: number, z: number): void {
    const ctx = this.tete.getContext('2d');
    if (!ctx) return;
    this.tete.width = 8;
    this.tete.height = 8;
    ctx.clearRect(0, 0, 8, 8);
    ctx.drawImage(skin, 8, 8, 8, 8, 0, 0, 8, 8);    // visage
    ctx.drawImage(skin, 40, 8, 8, 8, 0, 0, 8, 8);   // chapeau (seconde couche)
    this.marqueur.style.left = `${((x + 0.5) / this.largeur) * 100}%`;
    this.marqueur.style.top = `${((z + 0.5) / this.profondeur) * 100}%`;
  }

  /** Redessine tant que des chunks arrivent ou finissent leur apparition. */
  private animer(): void {
    if (this.boucle) return;
    const image = () => {
      const enCours = this.dessiner();
      this.boucle = enCours ? requestAnimationFrame(image) : 0;
    };
    this.boucle = requestAnimationFrame(image);
  }

  /** Dessine la carte ; vrai s'il reste quelque chose à animer. */
  private dessiner(): boolean {
    const ctx = this.ctx;
    if (!ctx) return false;
    const C = CONFIG.chargement.carte;
    const e = this.echelle;
    const maintenant = performance.now();

    // Les chunks arrivent un par un, à cadence régulière, même quand la barre fait un bond
    const intervalle = (C.dureeArrivee * 1000) / Math.max(1, this.chunks.length);
    const ecoule = maintenant - this.dernierDevoilement;
    if (this.devoiles < this.voulus && ecoule >= intervalle) {
      const fin = Math.min(this.voulus, this.devoiles + Math.floor(ecoule / intervalle));
      while (this.devoiles < fin) this.apparitions[this.devoiles++] = maintenant;
      this.dernierDevoilement = maintenant;
    }
    if (this.devoiles > 0 && this.marqueur.classList.contains('cache')) this.marqueur.classList.remove('cache');

    let enCours = this.devoiles < this.voulus;
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, this.toile.width, this.toile.height);
    this.chunks.forEach((c, i) => {
      const x = c.x * e, z = c.z * e, l = c.l * e, p = c.p * e;
      if (i >= this.devoiles || !this.image) {
        // Chunk pas encore chargé : une case à peine visible, séparée de ses voisines
        ctx.fillStyle = C.couleurAttente;
        ctx.fillRect(x + 1, z + 1, l - 2, p - 2);
        return;
      }
      const t = Math.min(1, (maintenant - this.apparitions[i]) / (C.apparition * 1000));
      if (t < 1) enCours = true;
      // « Pop » : le chunk grandit depuis son centre avec un léger rebond, en s'éclaircissant
      const s = rebond(t);
      const ls = l * s, ps = p * s;
      ctx.drawImage(this.image, c.x, c.z, c.l, c.p, x + (l - ls) / 2, z + (p - ps) / 2, ls, ps);
      if (t < 1) {
        ctx.globalCompositeOperation = 'source-atop';  // seulement sur les blocs, pas sur le vide
        ctx.fillStyle = `rgba(255, 255, 255, ${0.55 * (1 - t)})`;
        ctx.fillRect(x + (l - ls) / 2, z + (p - ps) / 2, ls, ps);
        ctx.globalCompositeOperation = 'source-over';
      }
    });
    return enCours;
  }
}
