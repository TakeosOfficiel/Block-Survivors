# Nouveautés de Block Survivors

Chaque version a son tag (`v0.6.0`…) et sa release GitHub avec l'exe.
Le lien « dernière version » pointe toujours sur la plus récente.

Règle des numéros : `0.x.0` pour des nouveautés, `0.x.y` pour des corrections.

## [0.12.0] — 2026-10-20

Refonte des classes, étape 4 : l'arsenal du Mage. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Le Mage a son propre arsenal de 6 attaques, et ses cartes ne proposent plus que celles-ci : Boules de feu, Éclairs, et quatre nouvelles.
- Tornade de feu : une colonne de cubes de magma tourbillonne vers les ennemis, les blesse et les aspire, en crachant des braises. Jusqu'à 2 tornades.
- Nova de glace : un anneau d'éclats de glace jaillit autour du Mage, blesse et ralentit (45 % au rang 4). Au dernier rang, il laisse de la neige poudreuse qui ralentit ceux qui la traversent. Bruit de verre brisé.
- Météore : un bloc de magma tombe en biais sur le plus gros groupe (son ombre annonce l'impact), puis le sol brûle avec le feu animé du jeu et blesse ceux qui restent dedans.
- Orbes arcaniques : des cubes d'améthyste tournent autour du Mage puis foncent chacun sur un ennemi, en changeant de cible si la leur meurt. À partir du rang 4, ils blessent aussi au contact pendant l'orbite.

### Changements
- Boules de feu et Éclairs sont désormais réservés au Mage. L'Archère et l'Invocateur gardent les armes non réservées en attendant leur propre arsenal.

## [0.11.0] — 2026-10-20

Refonte des classes, étape 3 : l'arsenal du Guerrier. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Le Guerrier a son propre arsenal de 7 attaques, et ses cartes ne proposent plus que celles-ci : Épée en fer, Haches tournoyantes, Trident, et quatre nouvelles.
- Onde de choc : il frappe le sol, une fissure file vers le plus gros groupe et soulève les blocs du sol de l'arène, qui jaillissent puis retombent. Jusqu'à 3 fissures en éventail ; au dernier rang, les ennemis touchés sont ralentis.
- Lame colossale : une épée en netherite géante tombe du ciel sur le plus gros groupe (son ombre annonce l'impact), avec éboulis, secousse et le son de l'enclume.
- Tourbillon de lames : il tourne sur lui-même et frappe tout autour plusieurs fois, entouré de balayages ; au dernier rang, il aspire les ennemis.
- Heurt de bouclier : coup de bouclier devant lui qui repousse fort et interrompt les attaques (la mèche du creeper repart de zéro) ; il étourdit à partir du rang 4 et renvoie les flèches ennemies contre leurs tireurs au dernier rang.

### Changements
- Haches tournoyantes et Trident sont désormais réservés au Guerrier. Les autres classes gardent les autres armes en attendant leur propre arsenal.

## [0.10.0] — 2026-10-20

Refonte des classes, étape 2 : les états des ennemis. Ils serviront aux nouvelles attaques et aux talents ; on peut déjà les essayer depuis le panneau F3.

### Nouveautés
- Brûlure et poison : dégâts toutes les 0,5 s en cumuls (3 et 5 au maximum), chiffres orange ou verts, jamais critiques. Le mob vacille en orange (brûlure) ou se teinte de vert, avec flammes ou tourbillons d'effet de potion.
- Ralentissement et enracinement : teinte bleue et particules de l'effet Lenteur. Un ennemi enraciné ne marche plus mais peut encore attaquer.
- Étourdissement : ni marche ni attaque, étoiles magiques au-dessus de la tête.
- Marque : les alliés invoqués font plus de dégâts à la cible, lueur dorée au-dessus d'elle.
- Les boss résistent : ralentis moitié moins (25 % au plus), seulement ralentis au lieu d'être enracinés, jamais étourdis.
- Panneau F3 : une rangée « États » pour infliger chaque état aux ennemis proches.

## [0.9.0] — 2026-10-20

Refonte des classes, étape 1 : le socle. Les chiffres sont provisoires et seront ajustés après les essais.

### Changements
- Plafonds : délai des armes −50 % au maximum, +2 projectiles en plus par arme, 60 % de critiques, régénération limitée à 2 % des PV max par seconde, et l'armure retire au plus la moitié d'un coup.
- « +1 projectile » ne s'applique plus qu'aux armes à projectiles (arc, trident, boules de feu, éclairs) : le Carquois n'ajoute plus de golem ni de hache.
- Invocations : chaque type a son quota (6 loups, 6 vex, 2 golems), et aucun ne prend la place d'un autre. Chaque invocation fait apparaître un groupe ; au quota, elle renouvelle à la place la durée des alliés présents.
- Appel de la meute et Hurlement donnent +1 allié par invocation (au lieu d'un projectile) ; un allié qui dépasserait le quota donne +10 % de dégâts à la place.
- Loups, vex et golem rééquilibrés pour que la durée compte : le golem n'est plus permanent dès le rang 1.

## [0.8.2] — 2026-10-20

### Changements
- Compteur de dégâts (Tab) refait : cadre violet du jeu, une barre par arme avec nom, dégâts, dégâts/s et part sur une seule ligne, plus de chiffres coupés.

### Corrections
- Portraits du poisson d'argent et du cube de magma dans le bilan (vus de face).
- Texture du cube de magma : il utilisait l'ancienne disposition de la planche, il a maintenant son vrai visage.

## [0.8.1] — 2026-10-20

### Changements
- « Nouveautés » ne s'ouvre plus tout seul : une pastille « Nouveau ! » sur le bouton signale une nouvelle version.
- Avertissement de mise à jour : le jeu lit un petit fichier public qui ne contient que le numéro de la dernière version (aucun code publié).

## [0.8.0] — 2026-10-20

### Nouveautés
- Éditeur de personnage (Apparence → Perso) : couleur de peau, yeux, coiffure, couleur des cheveux, barbe, haut, couleur du haut, pantalon et chaussures, avec flèches et bouton Aléatoire. Le skin est dessiné au format du jeu et utilisé en partie.
- Les mobs au corps à corps attaquent les invocations (golem, loups, vex) : elles ont des PV, une barre de vie, et peuvent mourir avant la fin. Le golem, avec son armure, attire la horde loin de toi.

### Corrections
- La seconde couche des skins (chapeau, veste, manches, jambières) s'affiche enfin, pour toutes les skins.

## [0.7.0] — 2026-10-20

### Nouveautés
- Compteur de dégâts en jeu (touche Tab), déplaçable à la souris : dégâts, part et dégâts par seconde de chaque arme, en colonnes.
- Bilan détaillé par arme : dégâts, part, dégâts par seconde, coups, critiques, plus gros coup, ennemis achevés.
- Les cœurs se remplissent pixel par pixel et les PV exacts s'affichent à côté : la régénération se voit.

### Changements
- Pour gagner, il faut avoir vaincu tous les boss : après 15:00, la partie continue tant qu'un boss est en vie.

### Corrections
- L'XP et les objets ne tombent plus dans la lave, l'eau ou les murs : ils vont sur la case accessible la plus proche.

## [0.6.0] — 2026-10-20

### Nouveautés
- Nouvelle classe : l'Invocateur et sa meute de loups.
- Nouvel accueil en trois colonnes : la classe, le héros en 3D (glisser pour le tourner) et l'arène avec ses boss.
- Pseudo affiché au-dessus du héros, avec un bouton « Aléatoire ».
- Écran Apparence : skin de la classe, une des 9 skins du jeu ou ton propre PNG, bras larges ou fins.
- Chiffres des dégâts au-dessus des ennemis, critiques dorés et plus gros.
- Volumes séparés : général, joueur, armes, créatures, interface.
- Trident : il part en ligne et traverse tout, puis revient dans la main à partir du rang 3 (Loyauté).
- Écran « Nouveautés » sur l'accueil, ouvert tout seul après une mise à jour.

### Changements
- Ombres et vent activés d'office pour tout le monde.
- Les trois classes sont jouables dès le départ.
- Totem : consommable sans rangs, chaque carte donne un totem de plus (2 en réserve au maximum).
- Les araignées ne sont plus ralenties par les toiles.

### Corrections
- Plus d'à-coup à l'arrivée d'un boss ou d'un nouveau mob : tout est préparé pendant le chargement.
- Flèches : une seule flèche au lieu de deux bout à bout.

## [0.5.0] — 2026-10-20

### Nouveautés
- Émeraudes gagnées à chaque partie, et deux arbres de talents par classe.
- Déblocage de la Mine et du Nether en jouant.
- Coups critiques et bonus « Silex taillé ».
- Particules du jeu : explosions, nuages de mort, flammes, paillettes du totem…

### Changements
- Armes rééquilibrées : Éclairs et Boules de feu moins écrasants, Arc et Golem renforcés, Haches qui touchent au contact.
- Squelettes, blazes et creepers moins punitifs ; le Nether démarre plus doucement.

## [0.4.0] — 2026-10-20

### Nouveautés
- Version affichée sur l'accueil.
- Totem d'immortalité : une vie en plus.
- Objets tenus en main (arcs des squelettes, armes des personnages).
- Mobs propres à chaque arène et compte à rebours du prochain boss.
- Rendu WebGPU (avec repli WebGL2), ombres et végétation au vent.
- Relance des cartes (R) et panneau de debug F3 plus complet.

### Corrections
- Les orbes d'XP ne tournent plus en boucle autour du joueur.
- Arcs tenus à l'endroit.

## [0.3.0] — 2026-10-20

### Nouveautés
- Application Windows (BlockSurvivors.exe), sans rien installer.
- L'épée vise automatiquement l'ennemi le plus proche.

### Corrections
- Plus de parties noires sur les squelettes.
- Interface à l'échelle de la fenêtre, sans défilement.

## [0.2.0] — 2026-10-20

### Nouveautés
- Première version jouable : textures Minecraft, ennemis, armes automatiques, niveaux et cartes, personnages, boss, trois arènes, menu, sons.
