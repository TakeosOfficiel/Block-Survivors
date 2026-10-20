"""Génère electron/icone.png : bloc d'herbe isométrique en pixel art (dessin original, pas une texture de Mojang)."""
import random
from PIL import Image

random.seed(7)
N = 16  # pixels de texture par face


def face(base, ecart):
    """Texture N x N : couleur de base, luminosité variant au hasard pixel par pixel."""
    return [[tuple(max(0, min(255, c + d)) for c in base) for d in (random.randint(-ecart, ecart) for _ in range(N))]
            for _ in range(N)]


herbe = face((98, 158, 56), 16)
terre = face((134, 96, 67), 14)
for y in range(N):  # frange d'herbe en haut des côtés
    for x in range(N):
        if y < 3 or (y == 3 and random.random() < 0.6) or (y == 4 and random.random() < 0.25):
            terre[y][x] = herbe[y][x]

T = 256
img = Image.new('RGBA', (T, T), (0, 0, 0, 0))
px = img.load()
cx, a, h = T / 2, 112, 128   # centre, demi-largeur, hauteur d'arête
b = a / 2
haut = T / 2 - (2 * b + h) / 2
ombres = (0.8, 0.62)          # côté gauche, côté droit
for Y in range(T):
    for X in range(T):
        dx, dy = X + 0.5 - cx, Y + 0.5 - haut
        u, v = (dy / b + dx / a) / 2, (dy / b - dx / a) / 2
        if 0 <= u < 1 and 0 <= v < 1:
            px[X, Y] = herbe[int(v * N)][int(u * N)] + (255,)
            continue
        if -a <= dx < a:
            gauche = dx < 0
            t = (dx + a) / a if gauche else dx / a
            y0 = b + t * b if gauche else 2 * b - t * b
            if y0 <= dy < y0 + h:
                c = terre[int((dy - y0) / h * N)][int(t * N)]
                k = ombres[0] if gauche else ombres[1]
                px[X, Y] = tuple(int(x * k) for x in c) + (255,)
img.save('electron/icone.png')
