/**
 * Carte de l'écran de chargement, façon Minecraft : la vraie map de la
 * partie vue de dessus, qui apparaît chunk par chunk depuis le point de
 * départ à mesure que la barre avance. Tant qu'un chunk n'est pas chargé,
 * c'est une case sombre ; quand il arrive, il s'allume d'un bref éclair.
 * Rien ne bouge entre deux avancées de la barre : la carte ne coûte rien.
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

export class CarteChargement {
  private readonly toile = element<HTMLCanvasElement>('chargement-carte');
  private readonly ctx = this.toile.getContext('2d');
  /** Pixels de canvas par colonne de la map. */
  private echelle = 1;
  private largeur = 0;
  private profondeur = 0;
  /** Chunks, du plus proche au plus loin du départ. */
  private chunks: Chunk[] = [];
  private image: HTMLCanvasElement | null = null;
  private tete: CanvasImageSource | null = null;
  private depart: { readonly x: number; readonly z: number } | null = null;
  /** Avancement du chargement quand la vue est arrivée : les chunks se dévoilent sur le reste de la barre. */
  private partDepart = 0;
  private devoiles = 0;
  /** Instant (ms) où chaque chunk dévoilé s'est allumé. */
  private allumages: number[] = [];
  private boucle = 0;

  /** Taille de la map connue : une grille de chunks sombres, en attendant ses couleurs. */
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
    this.depart = null;
    this.devoiles = 0;
    this.allumages = [];
    this.trier(largeur / 2, profondeur / 2);
    this.toile.classList.remove('cache');
    this.dessiner();
  }

  /**
   * Couleurs de la map arrivées (`part` = avancement actuel) : les chunks
   * s'allument à partir d'ici, en partant du départ du joueur, dont la tête
   * (`tete`, skin 64 px de large) marque la place.
   */
  montrer(vue: VueDessus, part: number, tete: CanvasImageSource | null): void {
    if (vue.largeur !== this.largeur || vue.profondeur !== this.profondeur) this.annoncer(vue.largeur, vue.profondeur);
    const image = document.createElement('canvas');
    image.width = vue.largeur;
    image.height = vue.profondeur;
    image.getContext('2d')?.putImageData(new ImageData(new Uint8ClampedArray(vue.couleurs), vue.largeur, vue.profondeur), 0, 0);
    this.image = image;
    this.tete = tete;
    this.depart = vue.depart;
    if (vue.depart) this.trier(vue.depart.x, vue.depart.z);
    this.partDepart = part;
    this.devoiles = 0;
    this.allumages = [];
    this.progression(part);
  }

  /** Avancement du chargement (0 à 1) : dévoile les chunks correspondants. */
  progression(part: number): void {
    if (!this.image || this.chunks.length === 0) return;
    const reste = Math.max(1e-6, 1 - this.partDepart);
    const voulus = Math.min(this.chunks.length, Math.ceil(((part - this.partDepart) / reste) * this.chunks.length));
    if (voulus <= this.devoiles) return;
    const maintenant = performance.now();
    while (this.devoiles < voulus) this.allumages[this.devoiles++] = maintenant;
    this.animer();
  }

  cacher(): void {
    this.toile.classList.add('cache');
    cancelAnimationFrame(this.boucle);
    this.boucle = 0;
  }

  /** Du plus proche au plus loin du point (colonnes), comme le chargement des chunks autour du joueur. */
  private trier(cx: number, cz: number): void {
    const distance = (c: Chunk) => Math.hypot(c.x + c.l / 2 - cx, c.z + c.p / 2 - cz);
    this.chunks.sort((a, b) => distance(a) - distance(b));
  }

  /** Redessine tant qu'un éclair d'allumage est en cours. */
  private animer(): void {
    if (this.boucle) return;
    const image = () => {
      const enCours = this.dessiner();
      this.boucle = enCours ? requestAnimationFrame(image) : 0;
    };
    this.boucle = requestAnimationFrame(image);
  }

  /** Dessine la carte ; vrai si un éclair est encore en cours. */
  private dessiner(): boolean {
    const ctx = this.ctx;
    if (!ctx) return false;
    const C = CONFIG.chargement.carte;
    const e = this.echelle;
    const maintenant = performance.now();
    let enCours = false;
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, this.toile.width, this.toile.height);
    this.chunks.forEach((c, i) => {
      if (i >= this.devoiles || !this.image) {
        // Chunk pas encore chargé : case sombre, séparée de ses voisines
        ctx.fillStyle = C.couleurAttente;
        ctx.fillRect(c.x * e + 1, c.z * e + 1, c.l * e - 2, c.p * e - 2);
        return;
      }
      ctx.drawImage(this.image, c.x, c.z, c.l, c.p, c.x * e, c.z * e, c.l * e, c.p * e);
      const age = (maintenant - this.allumages[i]) / 1000;
      if (age < C.eclair) {
        enCours = true;
        // Seulement sur les blocs du chunk (pas sur le vide autour de la map)
        ctx.globalCompositeOperation = 'source-atop';
        ctx.fillStyle = `rgba(255, 255, 255, ${0.6 * (1 - age / C.eclair)})`;
        ctx.fillRect(c.x * e, c.z * e, c.l * e, c.p * e);
        ctx.globalCompositeOperation = 'source-over';
      }
    });
    // La tête du joueur à son point de départ, dès que son chunk est chargé
    if (this.depart && this.tete && this.devoiles > 0) {
      const cote = Math.max(8, Math.round(C.tailleTete * e));
      const x = Math.round((this.depart.x + 0.5) * e - cote / 2), z = Math.round((this.depart.z + 0.5) * e - cote / 2);
      ctx.fillStyle = '#000';
      ctx.fillRect(x - 1, z - 1, cote + 2, cote + 2);
      ctx.drawImage(this.tete, 8, 8, 8, 8, x, z, cote, cote);   // visage
      ctx.drawImage(this.tete, 40, 8, 8, 8, x, z, cote, cote);  // chapeau (seconde couche)
    }
    return enCours;
  }
}
