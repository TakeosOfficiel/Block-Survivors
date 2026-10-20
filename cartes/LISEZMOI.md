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
