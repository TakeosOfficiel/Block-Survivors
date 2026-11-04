# Cartes Minecraft

Chaque schématique de ce dossier devient une arène du jeu, au nom du fichier :

- `.schem` : WorldEdit / FAWE (`//copy` puis `//schem save nom`) ;
- `.nbt` : bloc de structure.

**Zone jouable** : le sol plat où se trouve un panneau portant le mot
`depart` (sinon, le centre de la schématique), et tout ce qui y est relié à
pied au même niveau. Les murs, clôtures et reliefs qui l'entourent la
délimitent ; tout le reste de la construction sert de décor.

**Réglages** (facultatifs) : un fichier `.json` du même nom.

```json
{
  "nom": "Jardin des champignons",
  "description": "Texte affiché dans le menu.",
  "icone": "block/red_mushroom_block",
  "base": "prairie",
  "ciel": "#78a7ff",
  "brouillard": { "debut": 45, "fin": 120 },
  "lumiere": { "ambiante": 0.35, "soleil": 0.75 },
  "boss": ["geant", "seigneur_wither"],
  "ennemis": [{ "ennemi": "zombie", "depuis": 0, "poids": 6 }]
}
```

`base` reprend les ennemis, les boss et l'ambiance d'une arène du jeu
(`prairie`, `mine` ou `nether`) ; les autres champs la remplacent.

**Panneaux de réglage** (facultatifs) : un panneau dont le texte contient
`[ennemis]` crée autour de lui une zone peuplée de ces ennemis seulement, par
exemple `[zone] Village [ennemis] zombie squelette [rayon] 15` (les mots-clés
peuvent être sur des lignes différentes). `[zone]` donne un nom, `[rayon]` le
demi-côté de la zone carrée en blocs (12 par défaut). Les ennemis s'écrivent
avec leur identifiant : `zombie`, `squelette`, `araignee`, `creeper`,
`araignee_venimeuse`, `poisson_argent`, `piglin_zombifie`, `squelette_wither`,
`blaze`, `cube_magma`… Un nom inconnu est signalé au début de la partie.
Un panneau `[boss] wither` (et/ou `[mini-boss] ravageur`) remplace les boss de
la map, qui arrivent dans l'ordre des noms ; un nom qui n'est pas un boss est
signalé de la même façon. Les boss et mini-boss arrivent sur le premier panneau
`[boss]` (qui peut ne porter aucun nom), au lieu d'un endroit hors champ.
