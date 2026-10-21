# Nouveautés de Block Survivors

Chaque version a son tag (`v0.6.0`…) et sa release GitHub avec l'exe.
Le lien « dernière version » pointe toujours sur la plus récente.

Règle des numéros : `0.x.0` pour des nouveautés, `0.x.y` pour des corrections.

## [0.35.3] — 2026-10-21

### Corrections
- **Losanges de la barre de chargement enfin centrés** : ils étaient 3 pixels trop bas et trop vers l'intérieur (leur position partait de l'intérieur de la bordure de la barre). Ils sont maintenant exactement au milieu de la barre et sur ses bords dorés (mesuré au pixel près).

## [0.35.2] — 2026-10-21

### Corrections
- **Retour des losanges** sur le cadre de l'écran de chargement, mais refaits proprement : pixels tous de la même taille, émeraude plus grande dans un bord d'or en relief, et centrés exactement en hauteur sur le liseré (aux angles) et sur le milieu de la barre (aux deux bouts). Le remplissage de la barre part maintenant de la pointe des losanges.

## [0.35.1] — 2026-10-21

### Corrections
- **Cadre de l'écran de chargement refait** : les losanges des coins et de la barre étaient mal dessinés (pixels de tailles inégales, bords en escalier, mal placés sur les traits). Ils sont remplacés par des plaques d'or carrées avec une émeraude sertie, dessinées au pixel près et centrées exactement sur les angles du liseré et sur les bouts de la barre.

## [0.35.0] — 2026-10-21

### Changements
- **Écran de chargement de nouveau en plein écran**, avec un cadre propre à Block Survivors :
  - la photo de l'arène remplit l'écran, nette (photos fournies refaites en pleine taille, sans zoom) ;
  - un liseré doré fait le tour de l'écran, avec un joyau émeraude serti d'or dans chaque coin ;
  - la barre de chargement est dans le même cadre doré, un joyau à chaque bout, et se remplit en vert émeraude ;
  - le nom de l'arène en doré et une astuce au-dessus de la barre.

## [0.34.0] — 2026-10-21

### Changements
- **Écran de chargement dans le style du jeu** :
  - la photo de l'arène est encadrée dans un panneau d'inventaire de Minecraft, avec le nom de l'arène et l'astuce dessous ;
  - la barre de chargement est la barre d'expérience du jeu.
- **Photo nette** : prise à la pleine résolution de l'écran, sans zoom, et affichée plus petite dans son cadre.
- **Au lancement du jeu**, seulement le logo et la barre : la photo n'apparaît qu'au chargement d'une partie.

## [0.33.0] — 2026-10-21

### Changements
- **Écran de chargement façon WoW** : une belle image de l'arène en plein écran, qui avance très doucement, avec son nom en doré, une astuce de jeu au hasard et une barre de chargement dorée en bas.
  - L'image est une vraie photo de la map : à la fin de chaque chargement, le jeu en prend une sous un bel angle et la garde. Elle s'affiche au chargement suivant de cette arène, toujours à jour, et fonctionne aussi avec les maps que tu importes.
  - Les arènes du jeu ont déjà leur photo, dès la première partie.
  - Au lancement du jeu, l'écran montre la dernière arène jouée.
- La carte vue de dessus est retirée.

## [0.32.0] — 2026-10-21

### Changements
- **Écran de chargement refait, plus joli** :
  - plus de fond de terre : un fond sombre, puis la map elle-même, floue et assombrie, dès qu'elle est connue ;
  - la map est posée en perspective comme une maquette qui tourne lentement sur elle-même (rotation fluide même quand le jeu travaille) ;
  - ses chunks apparaissent un à un avec un petit « pop », depuis le point de départ, sur une grille discrète ;
  - la tête de ton héros se dresse à son point de départ, toujours tournée vers toi ;
  - le nom de l'arène en jaune, l'étape en cours et une barre fine et lumineuse.

## [0.31.1] — 2026-10-21

### Corrections
- **Le jeu ne se met plus en pause quand la fenêtre perd le focus** (clic sur un autre écran, une autre fenêtre…) : la partie continue. Échap met toujours en pause. Les touches enfoncées sont relâchées, pour que le héros ne continue pas à courir tout seul.

## [0.31.0] — 2026-10-21

### Changements
- **Nouvel écran de chargement façon Minecraft** : la vraie map de la partie apparaît vue de dessus, chunk par chunk, en partant du point de départ (marqué par la tête de ton héros) à mesure que la barre avance. Chaque chunk qui arrive s'allume d'un bref éclair.
  - Les couleurs sont celles des vrais blocs de la map (moyenne de leur texture), avec le relief ombré comme sur une carte de Minecraft.
  - Marche sur les arènes du jeu comme sur les maps importées (on voit toute la construction, décor compris).
  - L'écran ne dessine que quand la barre avance : il ne ralentit plus le chargement et ne peut pas saccader.
- La scène 3D du feu de camp est retirée.

### Corrections
- **Vue F5 (3e personne)** : la caméra ne saute plus brusquement quand on bouge la souris. Le jeu lit maintenant le mouvement brut de la souris et ignore les faux sauts du curseur que Chrome produit parfois sous Windows.

## [0.30.1] — 2026-10-21

### Corrections
- **L'animation du chargement ne saccade plus** : la scène du feu de camp est maintenant dessinée à part (dans un « worker »), elle continue de tourner pendant que le jeu construit la map et prépare les modèles, au lieu de se figer plusieurs secondes.
- La scène ne passe jamais avant le chargement de la partie : si la carte graphique est occupée par le jeu, elle attend son tour.
- **Lancement du jeu** : la scène n'est plus préparée au démarrage, mais une fois le menu affiché.
- La petite scène de fin à 100 % (tout le monde se lève) est retirée : la partie démarre dès que le chargement est fini.

## [0.30.0] — 2026-10-21

### Nouveautés
- **Nouvel écran de chargement : « La pause des mobs »**. Le soir, autour d'un vrai feu de camp de Minecraft (fumée, braises qui sautent, flammes animées), les mobs font une pause avant la partie pendant que la vue tourne lentement autour d'eux :
  - ton héros (ta classe, ta skin et son arme) caresse un mouton qui dort ;
  - le zombie fait griller un cookie au bout d'un bâton, et le cookie dore au fur et à mesure du chargement ;
  - le squelette compte ses os, un os à la main ;
  - le creeper essaie d'approcher du feu, tremble et recule ;
  - autour : une tente, une pile de bûches, une lanterne, des fleurs, des champignons et des lucioles.
- Ce sont les mobs qui racontent le chargement (« Le squelette compte ses os… 40 % »).
- **À 100 %**, tout le monde se lève : le mouton se réveille et sautille, le zombie croque son cookie, le héros fait coucou, et la partie commence.

## [0.29.0] — 2026-10-21

### Changements
- **Chargement des maps encore bien plus rapide** (mesuré au deuxième lancement : environ 10 s → 4 s sur la map importée, 7,5 s → 3,5 s sur la plaine) :
  - les mobs, alliés et objets partagent maintenant leurs shaders : une quarantaine à préparer au lieu de plus de 300 ;
  - la construction des maps Minecraft est environ quatre fois plus rapide.

## [0.28.0] — 2026-10-21

### Changements
- **Chargement des parties deux fois plus rapide** (mesuré : environ 15 s → 7 s au deuxième lancement) :
  - les shaders sont tous préparés en même temps, en parallèle, au lieu d'un petit paquet après l'autre ;
  - les textures et les modèles de blocs téléchargés sont gardés sur l'ordinateur : après le premier lancement, plus rien ne passe par internet (et le jeu se charge même hors ligne).
- **Une seule barre de chargement**, qui avance de 0 à 100 % sans jamais repartir de zéro ; seul le texte de l'étape change.

## [0.27.0] — 2026-10-21

### Changements
- **Mises à jour façon Discord** : au lancement, une petite fenêtre du jeu cherche les mises à jour.
  - S'il y en a une, elle la télécharge en montrant la progression, l'installe toute seule et relance le jeu à jour : plus de bouton à cliquer.
  - Sinon, ou sans connexion, le jeu s'ouvre aussitôt.
  - Si le téléchargement traîne, un lien permet de jouer sans l'attendre.
  - Une version sortie pendant une partie se télécharge en arrière-plan et s'installe à la fermeture du jeu.

## [0.26.0] — 2026-10-21

### Changements
- **Tornade de feu simplifiée et plus lisible** :
  - trois grandes spirales de feu continues, séparées par des vides, qui dessinent un entonnoir net même quand on fige l'image ;
  - les irrégularités sont sur les bords (bas dentelé, haut en flammes courtes) ; plus de grandes pointes au sommet, de trous au milieu des rubans ni de traînées fines ;
  - le feu monte toujours le long des spirales.
- **Plus de cubes ni de plaques noires au sol** autour de la tornade : elle laisse une discrète trace de brûlure brune, à plat, qui s'estompe derrière elle. Seules quelques braises restent à son pied.
- **Tornade de magma** : rouge, traversée de zones jaunes incandescentes qui montent, avec quelques fragments de magma.

### Corrections
- Le vent fait de nouveau onduler l'herbe, les fougères, les fleurs et les feuilles sur les maps importées de Minecraft (comme sur la plaine).

## [0.25.0] — 2026-10-21

### Changements
- **Tornade de feu plus vivante**, même silhouette en entonnoir :
  - chaque ruban a son propre rayon, sa vitesse, son nombre de tours et sa longueur ; des ondes montent le long des rubans, qui palpitent ;
  - le feu défile vers le haut le long de la spirale ; le bas des rubans est dentelé, le haut garde les pointes de flammes ;
  - les fins arcs blancs au sommet ont disparu.
- **Tornade de magma** (Tempête de feu) : au lieu d'un cône plein, un vortex de feu rougeoyant avec des espaces entre les rubans, de fines veines de magma au cœur et quelques fragments incandescents qui montent en spirale.

## [0.24.0] — 2026-10-21

### Nouveautés
- **Tornade de feu refaite en volume**, façon Minecraft Dungeons :
  - sa forme est un vrai entonnoir de rubans en spirale, recouverts du feu animé du jeu ; elle se reconnaît même sans aucune particule ;
  - la base est étroite et le haut large ; les rubans du cœur tournent plus vite ; le sommet ondule en retard sur la base ;
  - des mottes de terre sont arrachées du sol et montent en spirale, un anneau de dalles noircies brûle sous elle ;
  - flammes, braises et fumée l'habillent par-dessus ;
  - elle pousse du sol à l'apparition et s'effondre dans un nuage de fumée à la fin ;
  - après la Tempête de feu, c'est un entonnoir de magma.

### Changements
- La vue à la 1re personne est retirée : F5 alterne entre la vue de dessus et la 3e personne.
- Invocations : un allié tué ne revient plus à la relance suivante. Sa place reste vide pendant 1,5 fois la recharge de son arme (environ 10 s pour un loup, 9 s pour un vex, 24 s pour un golem).

## [0.23.0] — 2026-10-21

### Nouveautés
- **Touche F5 : trois vues**, comme dans Minecraft (aussi dans les options, bouton « Vue ») :
  - **de dessus** : la vue habituelle ;
  - **3e personne façon WoW** : la caméra est en hauteur, assez loin derrière le héros, et tourne à la souris ; la molette la rapproche jusque dans son dos ou l'éloigne pour voir toute la mêlée ;
  - **1re personne** : la caméra est dans les yeux du héros.
  
  Dans les deux vues à la souris, cliquer dans le jeu capture la souris, qui fait tourner la caméra ; ZQSD avance dans le sens où l'on regarde. La vue choisie est retenue d'une partie à l'autre.
- **Fusée explosive en vrai modèle 3D**, comme dans Minecraft Dungeons : un tube rouge à rayures blanches en spirale, avec sa coiffe pointue et sa tuyère. Plus d'image d'item avec une mèche qui ne sert à rien : la fusée crache derrière elle des flammes, des étincelles et un peu de fumée.

### Corrections
- Les invocations (loups, golems, vex…) ne se superposent plus : elles s'écartent les unes des autres et ne restent plus dans le joueur, même quand elles attaquent la même cible.

## [0.22.0] — 2026-10-20

Fini de retélécharger le jeu à chaque version : il se met à jour tout seul.

### Nouveautés
- Le jeu s'installe maintenant avec un installateur (`BlockSurvivors-Setup-0.22.0.exe`) : un double-clic, il s'installe, crée un raccourci sur le bureau et dans le menu Démarrer, puis se lance.
- Mises à jour automatiques : quand une nouvelle version sort, le jeu la télécharge en arrière-plan. Un bouton apparaît sur l'accueil : « Version X prête ! Cliquez pour redémarrer le jeu et l'installer ». Sans clic, elle s'installe d'elle-même à la fermeture du jeu.

### À savoir
- Il faut télécharger l'installateur **une dernière fois** (l'ancien exe portable affiche le lien sur l'accueil). Ensuite, plus jamais.
- Les sauvegardes (émeraudes, talents, profil, réglages) sont gardées.
- À la première installation, Windows peut afficher « Windows a protégé votre ordinateur » : cliquer sur « Informations complémentaires » puis « Exécuter quand même ».
- L'ancien `BlockSurvivors.exe` portable peut être supprimé après l'installation.

## [0.21.0] — 2026-10-20

Une seule version : le travail des maps Minecraft (fait à côté) rejoint la refonte des classes.

### Nouveautés
- Maps construites dans Minecraft : chaque fichier `.schem` (WorldEdit / FAWE) ou `.nbt` (bloc de structure) du dossier `cartes/` devient une arène. Première map : le **Jardin des champignons**.
- Tous les blocs vanilla s'affichent avec leur vraie forme (escaliers, dalles, portillons, plantes, vitres teintées, champignons géants…), lus dans les fichiers officiels de Minecraft.
- La zone jouable est trouvée toute seule (le sol plat où l'on démarre, ou celui d'un panneau « depart ») ; le reste de la construction sert de décor autour de l'arène.

### Changements
- Textures de Minecraft 26.3 (au lieu de 1.21.4).
- Les arènes peuvent être rectangulaires.
- Les squelettes et les blazes ne tirent plus à l'aveugle : si tu es caché, ils se déplacent pour te viser.
- L'exe est aussi publié sur le dépôt public : le lien « nouvelle version disponible » s'ouvre pour tous les joueurs.

### Corrections
- Les flèches ne traversent plus les murs, dans les deux sens : un muret d'un bloc protège désormais, et aucun tir ne passe dans un coin entre deux blocs ni ne saute un mur en temps accéléré.

## [0.20.1] — 2026-10-20

### Corrections
- Les objets du jeu sont dessinés comme dans Minecraft, plus en images plates :
  - la fusée et les haches lancées sont des objets en relief d'un pixel d'épaisseur ;
  - le trident lancé utilise le vrai modèle de l'entité (manche, garde et trois pointes) ;
  - la Lame colossale est une épée en relief plantée dans le sol ;
  - les flèches de la Pluie de flèches sont les entités de flèche du jeu, plantées pointe en bas ;
  - les pommes au sol sont en relief et tournent sur elles-mêmes comme les objets lâchés du jeu ;
  - l'allay (Esprit protecteur) a son vrai modèle au lieu de celui du vex ;
  - les racines du Totem des racines (et la haie du Bosquet) sont de vrais blocs de racines de palétuvier.
- Les boules de feu restent tournées vers la caméra : c'est ainsi que le jeu dessine les boules de feu des blazes et des ghasts.

## [0.20.0] — 2026-10-20

Refonte des classes, dernière étape : 4 emplacements d'armes et éveils. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Éveils : une arme au rang maximal propose deux cartes d'éveil exclusives ; en choisir une retire l'autre. 50 éveils au total, deux par arme (par exemple l'Épée : Tranchant ou Aura de feu ; les Haches : fendeuses ou givrées ; les Éclairs : Canalisation ou Foudre en boule ; les Loups : Meute affamée ou Loups des neiges).
- Un éveil renforce l'arme (dégâts, zone, nombre, durée…) et peut ajouter un état à chaque coup : brûlure, poison, ralentissement ou étourdissement.
- Les cartes d'éveil ont un nom violet, et l'arme éveillée brille comme un objet enchanté dans la barre des armes.

### Changements
- 4 emplacements d'armes au lieu de 6 : chaque partie demande de choisir 3 armes en plus de celle de départ, ce qui varie les arsenaux.

## [0.19.0] — 2026-10-20

Refonte des classes, étape 11 : les talents à mécanique propre de l'Invocateur. Les huit arbres de talents sont désormais complets : plus aucun talent n'est « en préparation ». Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Maître-chien : Chef de meute (première morsure renforcée), Morsure marquante, Traque ou Curée, Réactivité, Meute soudée, Hurlement, Encerclement, Braises de blaze, Coordination, Alpha (un loup plus grand et plus fort), Ruée de la meute (loups fantômes), Lune de sang (12 loups pendant 8 s), Gardien alpha.
- Évocateur : Lien spirituel, Garde rapprochée, Allay chanteur (l'allay ramasse l'XP), Essaim, Évocation supérieure (un ravageur du jeu charge à travers la horde), Crocs de l'évocateur (rangées de crocs qui jaillissent du sol et claquent), Bosquet (haie de racines entre deux totems).
- Nouveaux modèles : le ravageur (géométrie et texture du jeu) et les crocs d'évocateur, avec leurs sons.

## [0.18.0] — 2026-10-20

Refonte des classes, étape 10 : les talents à mécanique propre du Mage. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Pyromancien : Cœur de blaze (les Boules de feu enflamment), Cendres protectrices, Lave en fusion, Propagation, Brasier ou Fournaise, Tempête de feu (un météore tombé dans une tornade la change en tornade de magma), Embrasement (15 ennemis en feu explosent ensemble), Phénix (une fois par partie, retour à 40 % des PV dans une explosion de feu), Pluie de feu (3 petits météores).
- Arcaniste : Résonance, Haute tension et Conduction (arcs électriques d'un ennemi à l'autre), Orbes givrées, Flux arcanique (les orbes repartent une fois), Surcharge, Accélération, Tempête d'orage (8 éclairs toutes les 20 s), Écho arcanique (un sort se répète à 60 %), Zéro absolu (une nova sur cinq gèle et fait double dégâts).
- Ces talents ne sont plus marqués « Effet en préparation ».

## [0.17.0] — 2026-10-20

Refonte des classes, étape 9 : les talents à mécanique propre de l'Archère. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Élite : Visée stable, Premier sang, Perforation nette, Tir groupé ou Tir rebondissant, Marque du chasseur (lueur dorée sur l'ennemi marqué), Concentration, Point faible, Puissance (une flèche spectrale géante qui traverse tout toutes les 12 volées), Œil de l'aigle (un critique ricoche vers l'ennemi le plus éloigné), Exécutrice.
- Rôdeuse : Pas du chasseur, Venin (concentré ou virulent), Ronces collantes, Contagion, Course chargée et Second vent, Piège à loups, Ronces venimeuses, Nuage toxique (nuage vert de poison), Vent arrière.
- Ces talents ne sont plus marqués « Effet en préparation ».

### Changements
- Une flèche qui traverse un ennemi perd 25 % de ses dégâts (Perforation nette réduit cette perte).

## [0.16.0] — 2026-10-20

Refonte des classes, étape 8 : les talents à mécanique propre du Guerrier. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Gladiateur : Enchaînement, Frénésie (et Champion de l'arène), Fureur, Œil du bourreau, Élan critique ou Saignée, Lame lourde (un ennemi projeté contre un mur est blessé), Soif de combat, Fureur du gladiateur (anneau rouge toutes les 5 attaques de mêlée), Exécution.
- Gardien : Posture défensive, Épines et Ronces d'acier, Rempart ou Inébranlable, Contre-coup (le Heurt renvoie les projectiles dès le rang 1), Peau de fer, Tenir la ligne, Répliques (une 2e onde de choc), Contre-attaque du bastion, Dernier rempart, Séisme (onde en étoile toutes les 3 ondes).
- Nouvel état : le saignement, en poussière de redstone rouge.
- Ces talents ne sont plus marqués « Effet en préparation ».

## [0.15.1] — 2026-10-20

### Corrections
- Fusée explosive : la fusée vole la pointe en avant (elle partait de travers).
- Lame colossale : l'épée géante tombe debout et reste plantée dans le sol, au lieu de paraître couchée.

## [0.15.0] — 2026-10-20

Refonte des classes, étape 7 : les nouveaux arbres de talents, première partie. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- Huit nouveaux arbres de talents (deux par classe), en graphe : chaque talent s'achète une fois relié à un talent complet au-dessus de lui, et des paliers ouvrent le bas de l'arbre (8 rangs achetés au-dessus, puis 18 pour les talents finaux).
- Nouvel écran des talents : schéma de l'arbre avec liens de redstone qui s'allument, fiche du talent (effet, prérequis, coût, achat), et paliers affichés avec les rangs déjà achetés.
- Les achats sont gardés dans les deux arbres d'une classe ; un seul arbre est utilisé en partie, à choisir sur l'écran des talents.
- Talents à choix : l'achat débloque les deux options, une seule joue à la fois (changement gratuit).
- Talents finaux : un seul actif par arbre, changeable gratuitement avant chaque partie.
- Armes de départ : les talents qui offrent une arme au départ peuvent être désactivés, une seule arme en plus par arbre.
- Talents déjà actifs en partie : bonus de dégâts par famille d'armes (mêlée, projectiles, feu, alliés), rayon de mêlée, recul, critiques, PV, armure, régénération en part des PV max, soins des pommes, brûlure (dégâts, durée, cumuls), poison, ralentissements, PV, vitesse et durée des alliés, bonus d'armes précises, rangs d'armes offerts au départ.
- Les talents à mécanique propre (Frénésie, talents finaux…) sont déjà achetables et marqués « Effet en préparation » : ils arrivent dans la prochaine version.
- Le résumé des talents du menu montre les talents actifs de l'arbre utilisé, talent final en évidence.

### Changements
- Les talents de l'ancienne version sont remboursés : toutes les émeraudes dépensées sont rendues, et un bandeau l'annonce sur l'accueil.

## [0.14.0] — 2026-10-20

Refonte des classes, étape 6 : l'arsenal de l'Invocateur. Les quatre classes ont maintenant chacune leur propre arsenal. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- L'Invocateur a son propre arsenal de 6 invocations, et ses cartes ne proposent plus que celles-ci : Meute de loups, Golem de fer, Vex, et trois nouvelles.
- Esprit de blaze : des blazes alliés flottent à distance des ennemis et tirent des salves de 3 boules de feu ; au dernier rang, leurs tirs enflamment.
- Totem des racines : un totem en bois de palétuvier, entouré de racines, se plante dans le plus gros groupe. Il attire les mobs, ralentit et enracine ceux qui y entrent (les racines blessent à partir du rang 4) ; quand les deux totems sont posés, le plus ancien est replanté sur le nouveau groupe.
- Esprit protecteur : l'allay du jeu suit l'Invocateur et lui donne régulièrement du bouclier ; quand le bouclier est plein, il soigne l'allié le plus blessé.
- Bouclier : des PV d'absorption, affichés en cœurs dorés après les cœurs (comme l'effet Absorption du jeu), perdus avant les PV. La réserve est limitée à 25 % des PV max.

## [0.13.0] — 2026-10-20

Refonte des classes, étape 5 : l'arsenal de l'Archère. Valeurs provisoires, à ajuster après les essais.

### Nouveautés
- L'Archère a son propre arsenal de 6 attaques, et ses cartes ne proposent plus que celles-ci : l'Arc et cinq nouvelles.
- Salve en éventail : une gerbe de 5 flèches tirée à l'arbalète dans un cône, jusqu'à 9 flèches.
- Pluie de flèches : des flèches tombent du ciel sur le plus gros groupe et restent plantées un instant dans le sol.
- Flèche ricocheuse : une flèche spectrale lumineuse file vers un ennemi puis rebondit sur les suivants (jusqu'à 5 rebonds), en perdant 15 % de ses dégâts à chaque fois.
- Fusée explosive : une fusée de feu d'artifice qui explose en gerbe d'étincelles de toutes les couleurs, avec le son du jeu ; au dernier rang, l'explosion se divise en 4 petites explosions.
- Piège de ronces : des buissons de baies sucrées posés sous l'Archère blessent et ralentissent ceux qui les traversent (le plus ancien est remplacé quand ils sont tous posés) ; au dernier rang, ils enracinent un instant ceux qui y entrent.

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
