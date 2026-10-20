/** Skins par défaut du jeu (entity/player/wide|slim/<nom>), proposées dans le profil. */
export const SKINS_VANILLA = ['steve', 'alex', 'ari', 'efe', 'kai', 'makena', 'noor', 'sunny', 'zuri'] as const;

export type NomSkin = (typeof SKINS_VANILLA)[number];

export const NOMS_SKINS: Readonly<Record<NomSkin, string>> = {
  steve: 'Steve', alex: 'Alex', ari: 'Ari', efe: 'Efe', kai: 'Kai', makena: 'Makena', noor: 'Noor', sunny: 'Sunny', zuri: 'Zuri',
};
