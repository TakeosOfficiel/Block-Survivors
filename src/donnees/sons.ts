/**
 * Sons vanilla. Les sons ne sont pas dans le .jar : ils sont servis par
 * empreinte (hachage SHA-1 de l'index d'assets 1.21.4). Ajouter un son =
 * ajouter une entrée.
 */
import type { DefinitionSon } from './types';

const definitions = {
  // --- Joueur et progression ---
  orbe: { categorie: 'joueur', chemin: 'random/orb', hachage: '8a04a60d5c28fc60df472a877ca57f37eabc78d7', volume: 0.3, hauteur: [0.55, 1.45], intervalleMin: 0.035, simultanes: 4 },
  niveau: { categorie: 'interface', chemin: 'random/levelup', hachage: '19034765ba8ba5389b35804ab213537ab5cf706f', volume: 0.6, hauteur: [1, 1], intervalleMin: 0.2, simultanes: 1 },
  blessure_joueur: { categorie: 'joueur', chemin: 'damage/hit1', hachage: 'c43077ac1f9ceda7e9e1c152f839baf207833aa8', volume: 0.7, hauteur: [0.9, 1.1], intervalleMin: 0.1, simultanes: 2 },
  ramassage: { categorie: 'joueur', chemin: 'random/pop', hachage: 'd6ae1c04d0a7376a33d1df12e1b8057cfbab6bc2', volume: 0.4, hauteur: [0.8, 1.4], intervalleMin: 0.05, simultanes: 2 },
  manger: { categorie: 'joueur', chemin: 'random/burp', hachage: '389e837fa8c7f8f09499b43d8710489dfd8c53dd', volume: 0.5, hauteur: [0.9, 1.1], intervalleMin: 0.1, simultanes: 1 },
  totem: { categorie: 'joueur', chemin: 'item/totem/use_totem', hachage: 'f3fa2f7ac95f0bd8419b9638b2e6b289265e4b18', volume: 0.7, hauteur: [1, 1], intervalleMin: 0.5, simultanes: 1 },
  victoire: { categorie: 'interface', chemin: 'ui/toast/challenge_complete', hachage: 'bd01dff39a7bd1e0e7f6847f7aee981f157e4f94', volume: 0.6, hauteur: [1, 1], intervalleMin: 1, simultanes: 1 },
  clic: { categorie: 'interface', chemin: 'random/click', hachage: '3455ca942556c7d5eac7dd5e458e7fb3bad564c9', volume: 0.35, hauteur: [1, 1], intervalleMin: 0.03, simultanes: 2 },

  // --- Armes ---
  balayage: { categorie: 'armes', chemin: 'entity/player/attack/sweep1', hachage: '89741b2005ca27c98454e0cc000afe33b5af86fb', volume: 0.4, hauteur: [0.9, 1.1], intervalleMin: 0.1, simultanes: 2 },
  arc: { categorie: 'armes', chemin: 'random/bow', hachage: '4a6c47aa4f8c2cfe3237b250ed19b8714d611fd2', volume: 0.3, hauteur: [0.8, 1.25], intervalleMin: 0.06, simultanes: 3 },
  trident: { categorie: 'armes', chemin: 'item/trident/throw1', hachage: '57bc0d3c05470580efb254ec51262eae17ff0c4c', volume: 0.4, hauteur: [0.9, 1.1], intervalleMin: 0.1, simultanes: 2 },
  boule_feu: { categorie: 'armes', chemin: 'mob/ghast/fireball4', hachage: 'e2aae5ec9916c6c4ae328520f4d72a50afff5aed', volume: 0.25, hauteur: [0.9, 1.1], intervalleMin: 0.1, simultanes: 2 },
  tonnerre: { categorie: 'armes', chemin: 'ambient/weather/thunder1', hachage: 'a20f0a846d7becf5d08756207f86acf24fac1a8d', volume: 0.18, hauteur: [0.9, 1.2], intervalleMin: 0.3, simultanes: 2 },
  golem_invocation: { categorie: 'armes', chemin: 'mob/irongolem/throw', hachage: 'f30080c54b46aa2dcaa260018cb07ef0561408ac', volume: 0.5, hauteur: [1, 1], intervalleMin: 0.3, simultanes: 1 },
  golem_coup: { categorie: 'armes', chemin: 'mob/irongolem/hit1', hachage: '9d528a24b3aaf8e4210b570316ef4fe125f813d8', volume: 0.35, hauteur: [0.9, 1.1], intervalleMin: 0.15, simultanes: 2 },
  fissure: { categorie: 'armes', chemin: 'dig/stone1', hachage: '4ec831592ae2bcbac1b4047a9f9f72a2de5ca834', volume: 0.5, hauteur: [0.6, 0.8], intervalleMin: 0.15, simultanes: 2 },
  lame_colossale: { categorie: 'armes', chemin: 'random/anvil_land', hachage: '781d4bfc6ee0aff2a61b0d78f0da34610a28aa62', volume: 0.35, hauteur: [0.7, 0.85], intervalleMin: 0.2, simultanes: 2 },
  bouclier: { categorie: 'armes', chemin: 'item/shield/block1', hachage: 'f3b8dcde479d572a5c6973d5b73314eecd785dd6', volume: 0.5, hauteur: [0.8, 1.2], intervalleMin: 0.15, simultanes: 2 },
  tornade: { categorie: 'armes', chemin: 'mob/blaze/breathe1', hachage: 'f9e4ff760b11244522bce898a4a0ebab1dd4520e', volume: 0.45, hauteur: [0.8, 1], intervalleMin: 0.3, simultanes: 2 },
  nova: { categorie: 'armes', chemin: 'random/glass1', hachage: 'fa35a51bb74a4a6cf0c48161342db5a8577f13e5', volume: 0.4, hauteur: [0.9, 1.2], intervalleMin: 0.2, simultanes: 2 },
  orbes: { categorie: 'armes', chemin: 'block/amethyst/shimmer', hachage: 'a62c6054fe1ac8137b22f29959b562c0d821b6ed', volume: 0.6, hauteur: [0.9, 1.3], intervalleMin: 0.2, simultanes: 2 },
  explosion: { categorie: 'armes', chemin: 'random/explode1', hachage: 'fa85b9e7e54c36c9f9d0249ca739444b5e3bb66c', volume: 0.45, hauteur: [0.8, 1.1], intervalleMin: 0.08, simultanes: 3 },

  loup_aboiement: { categorie: 'armes', chemin: 'mob/wolf/bark1', hachage: '527618d2fbb9712950357c4d5e95cfb86ffee417', volume: 0.35, hauteur: [0.9, 1.2], intervalleMin: 0.3, simultanes: 2 },
  loup_morsure: { categorie: 'armes', chemin: 'mob/wolf/growl1', hachage: '0eaaba6d7fee5159a61b36aea268381bface391b', volume: 0.2, hauteur: [0.9, 1.2], intervalleMin: 0.15, simultanes: 2 },
  vex_invocation: { categorie: 'armes', chemin: 'mob/evocation_illager/prepare_summon', hachage: 'e792be7a228007d8a782b0d6caf9da9ebf77a86b', volume: 0.35, hauteur: [1, 1.1], intervalleMin: 0.5, simultanes: 1 },
  vex_charge: { categorie: 'armes', chemin: 'mob/vex/charge1', hachage: 'e4208f313e7c1431d9e04c80974fa4ed4a783275', volume: 0.2, hauteur: [0.9, 1.2], intervalleMin: 0.15, simultanes: 2 },

  golem_blesse: { categorie: 'armes', chemin: 'mob/irongolem/hit1', hachage: '9d528a24b3aaf8e4210b570316ef4fe125f813d8', volume: 0.25, hauteur: [0.8, 1], intervalleMin: 0.2, simultanes: 2 },
  golem_mort: { categorie: 'armes', chemin: 'mob/irongolem/death', hachage: 'aa6c662548316c68591a63f568be1f997d162ef1', volume: 0.5, hauteur: [1, 1], intervalleMin: 0.3, simultanes: 1 },
  loup_blesse: { categorie: 'armes', chemin: 'mob/wolf/hurt1', hachage: 'a08590d43a21ee739bd7b9be85b4a81564e6f74a', volume: 0.25, hauteur: [0.9, 1.2], intervalleMin: 0.15, simultanes: 2 },
  loup_mort: { categorie: 'armes', chemin: 'mob/wolf/death', hachage: '945dab2b484c34f108e1ea8eebca5e5e2a9e6424', volume: 0.4, hauteur: [0.9, 1.1], intervalleMin: 0.2, simultanes: 2 },
  vex_blesse: { categorie: 'armes', chemin: 'mob/vex/hurt1', hachage: '909ded09c154faea85e71d5456921be64cbdcd49', volume: 0.2, hauteur: [0.9, 1.2], intervalleMin: 0.15, simultanes: 2 },
  vex_mort: { categorie: 'armes', chemin: 'mob/vex/death1', hachage: 'd33b4c2a9bcaa175655a69f9b0a425bf1c136b71', volume: 0.3, hauteur: [0.9, 1.1], intervalleMin: 0.2, simultanes: 2 },

  // --- Ennemis ---
  meche: { categorie: 'creatures', chemin: 'random/fuse', hachage: 'b9a550ba1e028c027557572aeb928b760864efe4', volume: 0.5, hauteur: [0.9, 1.1], intervalleMin: 0.2, simultanes: 2 },
  boss: { categorie: 'creatures', chemin: 'mob/wither/spawn', hachage: '52049ed59e074474287b5dfee00477ce571b3385', volume: 0.45, hauteur: [1, 1], intervalleMin: 1, simultanes: 1 },
  zombie_blesse: { categorie: 'creatures', chemin: 'mob/zombie/hurt1', hachage: 'e46ef10953dadb96e7faab3dddcfecc5551b652b', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  zombie_mort: { categorie: 'creatures', chemin: 'mob/zombie/death', hachage: '51deaef407de6444854e4da39b165faa0d41370d', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  squelette_blesse: { categorie: 'creatures', chemin: 'mob/skeleton/hurt1', hachage: '6df446a07ce7f7aa292e8eeeab9f96543acd805b', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  squelette_mort: { categorie: 'creatures', chemin: 'mob/skeleton/death', hachage: '324bdca11921a46b0c87e61bf7a93047e70eb750', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  araignee_blesse: { categorie: 'creatures', chemin: 'mob/spider/say1', hachage: '8a687fb7ea955d58a8e758a7e6ca8147c48dd477', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  araignee_mort: { categorie: 'creatures', chemin: 'mob/spider/death', hachage: 'eec5d186206792d2c650e1dbaa9c8796c56d2ea3', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  creeper_blesse: { categorie: 'creatures', chemin: 'mob/creeper/say1', hachage: '6fc976d65fc742451db2049a2a38a1007b976bc1', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  piglin_blesse: { categorie: 'creatures', chemin: 'mob/zombified_piglin/zpighurt1', hachage: '7be727ea16b9a261665b8419397115f13a0966b6', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  piglin_mort: { categorie: 'creatures', chemin: 'mob/zombified_piglin/zpigdeath', hachage: '6b82e81c0122b8edad442925a532f7574ce7c8d2', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  blaze_blesse: { categorie: 'creatures', chemin: 'mob/blaze/hit1', hachage: '6e958c47928b592951feacce8015e0b308e39934', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  blaze_mort: { categorie: 'creatures', chemin: 'mob/blaze/death', hachage: 'cd7bf7e231c8fbf316bbcbd77278ac021e428834', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  wither_squelette_blesse: { categorie: 'creatures', chemin: 'mob/wither_skeleton/hurt1', hachage: 'aca61491c1ed4f91b2f97ad221ba279cb7629579', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  wither_squelette_mort: { categorie: 'creatures', chemin: 'mob/wither_skeleton/death1', hachage: '5c016e18ac7a942da00fa06522166b6210c8b3e6', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  magma_blesse: { categorie: 'creatures', chemin: 'mob/magmacube/jump1', hachage: '1e00e965fd5a742ae5557c5dfb9091d65fb1db59', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  magma_mort: { categorie: 'creatures', chemin: 'mob/magmacube/big1', hachage: 'c72dc968c642afa06e6e93d996185ada18f82154', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  poisson_blesse: { categorie: 'creatures', chemin: 'mob/silverfish/hit1', hachage: '8112d96f4f017bd0fb7d01b1b1167727c78a72b4', volume: 0.25, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  poisson_mort: { categorie: 'creatures', chemin: 'mob/silverfish/kill', hachage: 'e22688b99a704b528f438bf70af8a91c47f1d4b1', volume: 0.25, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
  creeper_mort: { categorie: 'creatures', chemin: 'mob/creeper/death', hachage: 'bd090d4d049f06217d1436552525d96ea1fa53b0', volume: 0.3, hauteur: [0.8, 1.2], intervalleMin: 0.07, simultanes: 3 },
} satisfies Record<string, DefinitionSon>;

export type IdSon = keyof typeof definitions;

export const SONS: Readonly<Record<IdSon, DefinitionSon>> = definitions;

export const IDS_SONS = Object.keys(definitions) as IdSon[];
