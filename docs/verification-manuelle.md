# Vérification manuelle du client

Les tests couvrent `game_screen` et `board_screen`, pas le dessin egui. Chaque spec qui touche l'UI se termine donc par une
vérification à l'œil, avec des captures jointes à la PR ou déposées dans les fichiers du projet. Sans écran (conteneur cloud),
`scripts/verif-manuelle.sh` fait tout : serveur X virtuel (Xvfb), clics et touches (xdotool), captures (ImageMagick).

Linux seulement. Sous Windows, lancer `cargo run -p arcanum-ui` et regarder.

## En bref

```sh
S=scripts/verif-manuelle.sh
$S demarrer                 # installe ce qui manque, compile, lance Xvfb puis le jeu (écran de nouvelle partie)
$S touche Return            # « Commencer » : Arythea, Première reconnaissance, graine au hasard
$S capture spec-0XX-debut   # capture tout l'écran
$S cliquer 1343 378         # clic gauche en (x, y)
$S touche End               # touche(s) xdotool : End, Return, Escape, plus, minus…
$S survoler 600 500         # survol sans clic (infobulles, trajet)
$S molette 3                # zoom avant de 3 crans (négatif : arrière)
$S capture spec-0XX-panneau 340x900+1260+0   # une partie de l'écran : LxH+X+Y
$S logs                     # logs techniques du lancement
$S arreter
```

`$S aide` liste les commandes.

## Partir d'une partie préparée

Une partie normale change à chaque lancement : pour vérifier un écran précis (un combat, une main avec des Blessures), partir
d'une **partie préparée** au lieu de jouer jusqu'à la bonne situation (issue privée 64).

```sh
$S charger combat-deux-ennemis   # lance le jeu et charge cette partie
```

`charger` arrête d'abord un jeu déjà lancé, puis démarre avec un dossier de sauvegardes neuf (`target/verif-manuelle/sauvegardes/`,
`ARCANUM_SAVE_DIR`) qui ne contient que cette partie, et appuie sur Ctrl+O, qui sur l'écran de nouvelle partie fait comme
« Charger… » (spec 065). Puis Tab donne le focus au bouton « Charger » de la seule ligne et Entrée le clique : aucune
coordonnée à relever (issue privée 294). Les sauvegardes faites ensuite (Ctrl+S) vont dans ce dossier.

| Nom | Situation au chargement |
|---|---|
| `combat-deux-ennemis` | Bac à sable, Arythea, jour, début du tour. Figurine sur la plaine (2, −2), voisine des Rôdeurs (3, −3) et des Creuseurs (3, −2) : défier l'un, l'autre ou les deux, puis engager le combat. Main : Marche ×2, Promesse, Concentration, Rage. Source : vert, rouge, bleu, blanc. |
| `combat-elements` | Même départ, figurine sur la plaine (2, −2), voisine des Cogneurs (3, −3, résistance physique) et des Dragons de glace (3, −2, attaque de glace 6) : éléments et résistances (spec 032). Main : Polyvalence, Rage, Détermination, Endurance, Marche. Un cristal rouge et un bleu. |
| `combat-capacites` | Même départ, figurine sur la plaine (2, −2), voisine des Invocateurs orques (3, −3 ; la pile marron commence par un Minotaure, brutal) et des Chevaucheurs de loups (3, −2, vifs) : capacités des ennemis (spec 033). Main : Détermination, Rage, Marche, Endurance, Célérité. Un cristal bleu. |
| `main-blessures` | Même départ, figurine sur le village (2, −3), loin des ennemis. Main : Blessure ×2, Marche, Promesse, Rage. |
| `soins` | Même départ, figurine sur le village (2, −3). Main : Blessure ×3, Tranquillité, Rage : soin, pioche et repos (spec 034). |
| `combat-assomme` | Même départ, figurine sur la plaine (2, −2), voisine d'un Minotaure (3, −3, brutal : 5 Blessures s'il n'est pas paré). Main : Marche ×2, Endurance, Promesse, Rage : héros assommé (spec 034). |
| `niveau-3` | Même départ, figurine sur le village (2, −3), voisine des Rôdeurs (3, −3). Gloire 6, niveau 2. Main : Célérité, Marche, Rage, Promesse, Endurance : vaincre les Rôdeurs (Célérité renforcée par le dé blanc) affiche « Niveau 3 à la fin du tour » ; finir le tour montre le bandeau du niveau 3 et les nouvelles valeurs (spec 035). |
| `niveau-pair` | Même départ que `niveau-3`, avec les offres de la vraie mise en place. Gloire 1, niveau 1 : vaincre les Rôdeurs mène au niveau 2 ; finir le tour ouvre la fenêtre Offres sans croix, « Prendre » sous chaque action avancée (touches 1 à 3) ; la carte prise est piochée aussitôt. Bouton « Offres… » et touche O hors du choix (spec 050). |
| `competences` | **Première reconnaissance**, **Tovak**, jour, début du tour. Figurine sur le village (2, −3), voisine des Rôdeurs (3, −3) ; offres de la vraie mise en place. Gloire 15, niveau 3 (niveau 4 à la fin du tour). Compétences : Bretteur de glace, Motivation (retournée), Magie inutile (« De côté : 2 », « De côté : 3 », spec 058), pile de 7. Section Compétences à droite de la main ; en combat, Bretteur de glace en phase d'attaque ; finir le tour ouvre la fenêtre du choix de compétence, puis la fenêtre Offres (spec 051). |
| `competences-cote` | Bac à sable, **Tovak**, jour, début du tour, niveau 1. Figurine sur le village (2, −3), voisine des **Cogneurs** (3, −3, résistance physique). Main : Blessure, Rage, Marche, Endurance, Célérité. Compétences : Flexibilité, Magie inutile (bouton « De côté » : choix de la carte dans la main, bandeau « … : choisir la carte à jouer de côté » ; à +3, dés de la Source grisés et rappel dans les effets en cours), Botte secrète (en combat contre les Cogneurs : bandeau du choix de l'ennemi), Surcharge de mana (quatre boutons de couleur, puis « Au centre » ; au tour suivant, l'avantage dans les effets en cours) (spec 058). |
| `competences-goldyx` | Bac à sable, **Goldyx**, jour, début du tour. Figurine sur le village (2, −3), le lac voisin en (2, −4). Cristaux rouge, bleu et blanc. Compétences : Pouvoir universel (la carte, puis le mana : « Jeton vert · Mouvement 4 »), Vol (cases d'arrivée en évidence sur le plateau, un clic), Embrasure de la Source (le dé à relancer dans la Source, ou « Ne rien relancer » ; au tour suivant, le dé en plus et « Garder ce dé tel quel » près de la Source), Fortune clinquante (pendant l'interaction : « Influence d'interaction : 3 » sous l'Influence) (spec 058). |
| `competences-norowas` | Bac à sable, **Norowas**, jour, début du tour, niveau 5. Figurine sur le village (2, −3), voisine des Rôdeurs (3, −3). Paysans, Arbalétriers d'Utem et Forestiers prêts ; Herboristes et Gardes d'Utem dans l'offre. Compétences : Charisme (en combat, second bouton sur la capacité : infobulle « Avec Charisme : Parade 5 »), Marche en avant, Loyauté sans faille (« Jeton de Loyauté : recrutement −5 » sous l'en-tête des unités ; en interaction, « Recruter sous Loyauté : … »), Temps calme (spec 058). |
| `competences-arythea` | Bac à sable, Arythea, jour, début du tour. Figurine sur le village (2, −3), voisine des Rôdeurs (3, −3). Main : Blessure ×2, Rage, Marche, Célérité ; un cristal bleu. Compétences : Souffrance sublimée (une Blessure de côté), Invocation et Polarisation (lignes « Invocation : défausser … » et « Polarisation : … comme … » de la fenêtre du mana, Rage renforcée par exemple), Rituel de guérison (« Jeter 0/1/2 Blessure(s) », puis « Au centre » ; au tour suivant, « Avantage de Rituel de guérison » : une Blessure de côté à 3) (spec 058). |
| `interaction` | Bac à sable, **Norowas**, jour, début du tour. Figurine sur le village (2, −3), voisine des Rôdeurs (3, −3). Main : Nobles manières, Menace, Promesse, Marche, Rage ; un cristal blanc et un rouge. Réputation 0 : « Interagir avec les habitants » (touche I), Nobles manières renforcée (cristal blanc) donne gloire et réputation ; ou défier et vaincre les Rôdeurs pour voir « Réputation +1 » dans l'historique du combat (spec 040). |
| `unites` | Bac à sable, Arythea, jour, niveau 3 (deux jetons de commandement), début du tour. Figurine sur le village (2, −3), voisine des Rôdeurs (3, −3). Zone des unités : Paysans disponibles, Golems gardiens blessés ; des Gardes d'Utem dans l'offre. Main : Promesse, Menace, Tranquillité, Rage, Détermination ; un cristal vert. Tranquillité renforcée (cristal vert) donne « Soin : 2 » et « Soigner Golems gardiens (2) » ; interagir puis recruter les Gardes en renvoyant une unité ; ou défier les Rôdeurs et assigner leurs dégâts aux unités (spec 042). |
| `monastere` | Bac à sable, Arythea, jour, début du tour. Campagnes 3, 4 et 5 explorées, figurine sur le monastère de la campagne 5 (5, −6), voisine de Rôdeurs (5, −5). Offre d'unités : Herboristes, Moines du Nord, Forestiers et une action avancée du monastère ; Paysans dans la zone des unités. Main : Promesse, Menace, Rage, Détermination, une Blessure ; réputation 0. Interagir, acheter un point de soin (2) et l'action avancée (6) ; ou incendier le monastère, gagner le combat sans les unités, prendre un artefact à la fin du tour et voir le Bouclier sur le plateau (spec 041). |
| `forteresse` | Bac à sable, Arythea, jour, début du tour. Campagnes 3 (en 2A) et 4 (en 2B) explorées : Gardes face cachée sur la forteresse (7, −5), Mages de feu face cachée sur la tour de mage (2, −6). Figurine sur la plaine (5, −4), à deux pas de la forteresse. Main : Marche, Endurance, Célérité, Rage, Détermination ; un cristal rouge et un bleu ; Épéistes d'Utem dans la zone. Avancer à côté (6, −5) retourne les Gardes (plus de retour en arrière) ; assaillir (Réputation −1), parer et attaquer : conquête et Bouclier, ou retraite sur la case de départ (spec 043). |
| `tour-conquise` | Même plateau que `forteresse`, la forteresse déjà conquise, la tour de mage conquise **ce tour** ; figurine sur la tour (2, −6). Offres de la vraie mise en place ; offre d'unités : Golems gardiens, Illusionnistes, Épéistes d'Utem. Main : Promesse, Menace, Rage, Marche, Détermination ; un cristal rouge et un blanc. Interagir, acheter un sort (7 + un mana de sa couleur), recruter ; finir le tour ouvre la fenêtre du sort en récompense (spec 043). Limite de main augmentée : voir `forteresse` après la conquête. |
| `donjon` | Bac à sable, Arythea, jour, début du tour. Campagne 7 en 2A, figurine sur son donjon (6, −3) ; un Minotaure en tête de la pile marron. Paysans dans la zone, un jeton or, un cristal rouge et un bleu. Main : Détermination, Rage, Marche, Célérité, Endurance. Entrer (combat aussitôt, règles de Nuit : le jeton or ne paie rien, les Paysans ne participent pas), parer (Détermination renforcée) et attaquer (Rage renforcée et une carte de côté) : Bouclier, dé de récompense (bleu : un artefact), finir le tour (spec 044). |
| `sombre-terrier` | Même départ, tuile royaume 1 en 2B. **S'arrête sur la fenêtre de la couleur du cristal** (issue privée 131) : Sombre Terrier (1, −6) conquis ce tour, Épéistes d'Utem dans la zone, Marche jouée de côté, fin du tour lancée ; graine 15 : le dé rouge a donné un cristal rouge, le dé or attend sa couleur (touches 1 à 4). Le choix lance le dernier dé (blanc), puis les deux artefacts tirés attendent le choix (spec 044). Main : Rage, Détermination, Célérité, Endurance. |
| `clairiere` | Bac à sable, Arythea, jour, début du tour. Figurine sur la clairière de la campagne 1 (1, −3), une Blessure en main et une dans la défausse. Main : Marche, Rage, Promesse, Blessure, Endurance. Finir le tour propose de jeter une Blessure (main ou défausse) ; le tour suivant commence avec un jeton de mana or (spec 044). |
| `lac` | Bac à sable, Arythea, jour, début du tour. Figurine sur le village de la campagne 1 (2, −3), Rôdeurs en (1, −4), dé bleu dans la Source. Main : Pont de glace, Temporisation, Intimidation, Marche, Rage. Pont de glace renforcé rend le lac (2, −4) franchissable pour 1 ; son infobulle prévient de la retraite forcée ; défier les Rôdeurs depuis le lac ; finir le tour sur le lac annonce la retraite forcée, propose de garder Temporisation et baisse la réputation d'Intimidation ; zone de jeu : « Effets en cours » (spec 052). |
| `agilite` | Bac à sable, Arythea, jour, début du tour. Figurine sur le village (2, −3), Rôdeurs voisins en (3, −3). Main : Agilité, Embuscade, Diplomatie, Marche, Promesse. Jouer Agilité, Embuscade, Diplomatie, défier : en phase de parade, « Influence en Parade » après Promesse ; en phase d'attaque, « Mouvement en Attaque » après Marche ; Embuscade renforce la première carte d'Attaque ou de Parade (spec 052). |
| `actions-avancees` | Bac à sable, Arythea, jour, début du tour, niveau 3. Figurine sur la plaine (2, −2), Rôdeurs et Invocateurs orques ensemble en (3, −3). Illusionnistes et Paysans disponibles, un cristal bleu. Main : Bouclier de glace, Au cœur du combat, Coup de sang, Magie pure, Rage. Défier, Au cœur du combat (effets en cours), Illusionnistes sur les Invocateurs (choix de l'ennemi, « n'attaque pas »), parade des Rôdeurs avec Bouclier de glace renforcé (« Armure 1 (au lieu de 3) »), Magie pure renforcée (deux manas) (spec 053). |
| `actions-avancees-mana` | Bac à sable, Arythea, jour, début du tour. Figurine sur la plaine (2, −2). Offre d'actions avancées : Chemins de traverse, Boule de feu, Agilité ; offre de sorts : Éclair de feu, Tempête de neige, Secousse. Un cristal rouge et un bleu. Main : Effet maximum, Entraînement, Rituel de sang, Tempête de mana, Décomposition, Marche, Rage. Effet maximum sur Marche (Mouvement 6), Entraînement en détruisant Marche (Prendre dans la fenêtre Offres), Tempête de mana sur un dé, Décomposition sur Rage, Rituel de sang renforcé (fenêtre des jetons puis du cristal) (spec 056). |
| `sorts` | Bac à sable, Arythea, **nuit**. Assaut de la tour de mage (campagne 4 en 2B) défendue par des Mages de feu, figurine sur la tour (2, −6), venue de (2, −5), combat engagé en phase à distance. Main : Exposition, Démolition, Frisson, Bouclier ardent, Tourbillon, Don pour la magie, Rage. Offre de sorts : Éclair de feu, Tempête de neige, Secousse. Un cristal de chaque couleur, un jeton noir. Démolition (« fortifications des sites ignorées »), Exposition (« plus fortifié », « plus de résistances »), Frisson, bouton Renforcer et son paiement noir, Don pour la magie en défaussant Rage puis « Lancer » Éclair de feu dans la fenêtre Offres ; en parade, Bouclier ardent puis « Ajouter l'Attaque de feu 4 » en phase d'attaque (spec 054). |
| `sorts-deplacement` | Bac à sable, Arythea, **nuit**, début du tour. Figurine sur le village (2, −3). Source : vert, rouge, bleu, blanc, noir. Main : Voyage souterrain, Méditation, Puits de mana, Appel aux armes, Distorsion spatiale ; Marche, Rage et Endurance dans la défausse. Un cristal de chaque couleur. Paysans et Forestiers dans l'offre d'unités. Voyage souterrain (paiement dans le bandeau, puis arrivée cliquée sur le plateau, issue privée 257), Méditation (deux cartes au hasard, fenêtre « dessus » ou « dessous »), Puits de mana (dé gardé dans la Source), Appel aux armes (« Utiliser » dans l'offre d'unités), Distorsion spatiale renforcée (« Tour supplémentaire » dans la barre de menu, cartes écartées sur la ligne de la défausse) (spec 057). |
| `sorts-cibles` | Bac à sable, Arythea, **nuit**, début du tour. Figurine sur la plaine (2, −2), voisine des Rôdeurs, des Épéistes et des Chevaucheurs de loups (3, −3). Source : vert, rouge, bleu, blanc, noir ; deux jetons noirs, un cristal blanc. Main : Ailes du vent, Restauration, Marche, Rage, Endurance. Arbalétriers d'Utem, Paysans et Golems gardiens épuisés. Restauration renforcée (Renaissance) : unités cochées dans le panneau du bas ; défier les orques, engager le combat, jouer Marche puis Ailes du vent renforcé (Ailes de la nuit) : ennemis cochés sur l'écran de combat, coût en Mouvement dans le bandeau (spec 057, issue privée 258). |
| `artefacts` | Bac à sable, Arythea, jour, début du tour. Figurine sur la plaine (2, −2), Rôdeurs et Soldats voisins en (3, −3). Main : Bannière de gloire, Bannière de terreur, Anneau de rubis, Épée de justice, Cor de colère, Sac de gemmes sans fond, Livre de sagesse, Amulette des ténèbres, Marche, Rage (dix cartes au plus : la fin du tour énumère les sous-ensembles de la main). Paysans portant la Bannière de protection (sous la carte), Gardes d'Utem sans bannière ; Coup de sang dans l'offre d'actions avancées. Boutons Jouer et Détruire, « Assigner » puis clic sur une unité, Anneau de rubis renforcé (effets en cours), Amulette des ténèbres (couleurs), Livre de sagesse sur Rage, Sac de gemmes (dés lancés) ; défier : Cor de colère renforcé (bonus), Bannière de terreur renforcée (ennemis cochés), Épée de justice en phase d'attaque (cartes cochées) (spec 055). |
| `ruines-autel` | Bac à sable, Arythea, jour, début du tour. Campagne 8 en 2A, figurine sur sa ruine (7, −5), jeton autel bleu face visible. Un cristal bleu, un jeton bleu, un dé bleu dans la Source. Main : Marche, Rage, Détermination, Célérité, Endurance. Faire l'offrande (touche E, fenêtre des paiements, touche 1) : 7 gloire, le drapeau sur la ruine, « Ruines conquises » (spec 045). |
| `ruines-nuit` | Même plateau, **nuit**. Figurine sur la clairière (6, −5), voisine de la ruine (7, −5) face cachée (jeton gris et violet, récompense une unité). Main : Marche ×2, Endurance, Célérité, Rage. Un trajet vers la plaine (7, −4) passe par la ruine : il s'y arrête (message), le jeton se retourne ; entrer, combattre (spec 045). |
| `ruines-unite` | Même plateau, jour. **S'arrête sur le choix de l'unité de la récompense** (issue privée 131) : ruine grise et violette conquise ce tour, Paysans au seul jeton de commandement, Forestiers et Herboristes dans l'offre, Marche jouée de côté, fin du tour lancée. Prendre une unité en renvoyant les Paysans, ou renoncer (spec 045). |
| `cite-bleue` | Bac à sable, Arythea, **nuit**, début du tour. Tuile royaume 6 posée en A + B : cité bleue niveau 5 (4, −5), Mages de glace, Mages de feu et Glaceurs face cachée. Figurine sur le village (2, −3), à deux pas. Main : Marche ×2, Détermination, Rage, Endurance ; un cristal bleu et un rouge ; Catapultes dans la zone. Avancer sur la colline (3, −4) retourne les défenseurs de nuit aussi (plus de retour en arrière) ; infobulle de la cité ; assaut (Endurance, coût 2) ; combat : « Attaque 7 (au lieu de 5) », Glaceurs doublement fortifiés ; finir le combat sans tout vaincre : Bouclier et retraite, infobulle « Vos Boucliers : 1 » (spec 062). |
| `cite-blanche` | Bac à sable, Arythea, jour, début du tour. Tuile royaume 7 en A + B : cité blanche niveau 5 (4, −5), Gardiens et deux Canonniers retournés. Figurine sur la plaine (5, −6), voisine. Main : Marche, Bouclier de glace, Rage, Détermination, Endurance ; un cristal bleu. Marche puis la cité : assaut ; combat : « Armure 8 (au lieu de 7) », Gardiens doublement fortifiés (spec 062). |
| `cite-blanche-conquise` | Bac à sable, Arythea, jour, début du tour. Cité blanche conquise (3 Boucliers, Seigneur), figurine dessus (4, −5). Offres de la vraie mise en place ; offre d'unités : Bûcherons, Catapultes ; pioche d'élites : Canonniers d'Amotep en tête. Main : Promesse, Menace, Marche, Rage, Détermination ; un cristal blanc et un rouge ; réputation 0. Interagir : « Boucliers dans la cité : Influence +3 », recruter les Bûcherons, « Ajouter une unité d'élite à l'offre (2) » ; panneau : « Limite de main : 7 (+2 cité) » (spec 062). |
| `cite-rouge-conquise` | Comme `cite-blanche-conquise`, cité rouge (royaume 8). Promesse et Menace renforcées (cristaux blanc et rouge) et les 3 Boucliers : 12 Influence. Acheter un artefact, finir le tour : fenêtre du choix des artefacts (spec 062). |
| `cite-verte-conquise` | Comme `cite-blanche-conquise`, cité verte (royaume 5). Acheter une action avancée de l'offre (6), puis la carte du dessus de la pioche (6, plus de retour en arrière) (spec 062). |
| `cite-en-vue` | **Première reconnaissance**, Arythea, jour, début du tour. Figurine sur le village (2, −3) ; la tuile de cité (royaume 5) est la prochaine de la pioche. Main : Marche ×2, Endurance, Rage, Promesse : explorer l'emplacement (1, 1) donne 1 gloire et annonce le dernier tour ; finir ce tour puis le dernier ouvre la fenêtre de fin (spec 063). |
| `fin-de-partie` | **Première reconnaissance** terminée : `cite-en-vue` jouée jusqu'au bout (Marche, exploration de (1, 1), fin du tour, Endurance, fin du dernier tour). La fenêtre de fin s'ouvre dès le chargement, avec le décompte et le bouton « Nouvelle partie… » (specs 063 et 065). |
| `conquete-cote` | **Conquête en solitaire** (sans fantôme), Arythea, jour, début du tour. Tuiles de campagne 3 à 7 posées en (1, 1), (0, 2), (1, 2), (0, 3), (1, 3) ; prochaine tuile : royaume 2. Figurine sur la colline (5, −5), voisine de (2, 0), sur la côte, et de (2, 1), à l'intérieur. Main : Marche, Détermination, Rage, Célérité, Endurance. Marche, puis survoler les deux emplacements : « Pas de tuile royaume sur la côte… » sur (2, 0), « Cliquez pour révéler une tuile royaume » sur (2, 1) ; explorer (2, 1) (spec 064, C32). |
| `conquete-derniere-cite` | **Conquête en solitaire**, Arythea, jour, manche 3, Lève-tôt prise, fantôme Goldyx. Cité bleue conquise en (2, 1) (3 Boucliers) ; cité rouge niveau 8 (royaume 8) en A + B (4, −5), défendue par le seul Minotaure, face visible, 3 Boucliers du joueur dessus ; figurine voisine (3, −4). Main : Marche, Détermination, Rage, Célérité, Endurance ; un cristal bleu et un rouge. Panneau « Cités conquises : 1/2 » ; Marche, assaut ; parer 5 (Détermination renforcée, cristal bleu) ; attaquer 5 (Rage renforcée, cristal rouge, et Célérité de côté) ; finir le combat : bandeau « Toutes les cités sont conquises… », « 2/2 » ; fin du tour sans tour du fantôme, « Dernier tour » ; fin du dernier tour : fenêtre de fin, « Mission accomplie ! » et les douze lignes (spec 064, C28 à C30). |
| `conquete-accomplie` | `conquete-derniere-cite` jouée jusqu'au bout (Marche, assaut, Minotaure vaincu posé en phase d'attaque, fin du combat, fin du tour, Célérité de côté, fin du dernier tour). La fenêtre de fin s'ouvre au chargement : « Mission accomplie ! », manches non jouées (3) : 90 ; la fermer, la rouvrir par « Score final » (spec 064, C30). |
| `conquete-echouee` | **Conquête en solitaire**, Arythea, nuit, manche 6, fantôme Goldyx qui a retourné tout son deck et annoncé la fin de la manche : ce tour est le dernier. Cité bleue conquise en (2, 1). Finir le tour : fenêtre de fin, « La dernière manche est jouée. », « Mission échouée… », cités conquises (1) : 10, les quatre autres lignes à 0 (spec 064, C31). |
| `tactiques-jour` | **Première reconnaissance**, **Tovak**, manche 1, jour. Figurine sur le village (2, −3). **S'arrête sur la fenêtre du choix de la tactique** : les six tactiques de Jour en 2 lignes de 3, un clic (ou les touches 1 à 6) encadre, « Prendre cette tactique » (ou Entrée) prend ; la main est visible dessous. Faux départ (cases « Défausser » dans la main, bandeau), Vol de mana (dés cliquables dans la Source), Départ flamboyant, Au bon moment (bouton dans le panneau) (spec 060). |
| `tactiques-manche-3` | Même départ, **manche 3** (Jour), Planification et Préparation retirées aux manches d'avant. **S'arrête sur la fenêtre du choix** : un trou à la place 4 (Planification), les autres tactiques à leur place (spec 060). |
| `tactiques-de-vole` | `tactiques-manche-3`, Vol de mana pris et le dé bleu posé dessus : il n'est plus dans la Source, « Dé volé : bleu » sous la tactique dans le panneau ; Endurance renforcée le paie (la ligne « Dé volé : bleu » s'ajoute au choix du mana quand il y a un autre paiement) ; la tactique est alors « Retournée : a servi » (spec 060). |
| `tactiques-nuit` | Même départ, manche 2, **nuit**. Provisions prise, deux cartes posées dessous (« Dessous : 2 carte(s) » dans le panneau) ; finir le tour ouvre, avant le tour 3, la fenêtre de Provisions (poser la carte du dessus, ou retourner et prendre les 2 cartes) (spec 060). |
| `tactiques-longue-nuit` | Même départ, manche 2, nuit. Longue Nuit prise, deck de 2 cartes, Menace dans la défausse ; Marche jouée de côté, fin du tour en défaussant 3 cartes : **s'arrête sur la fenêtre de Longue Nuit** (« Remettre 3 cartes dans le deck » ou « Non, arrêter la pioche ») ; refusée, le bouton de Longue Nuit revient dans le panneau, deck vide (spec 060). |
| `fantome-premier` | **Première reconnaissance avec joueur fantôme** (Goldyx, cristaux vert, vert, bleu), Tovak sur le village, manche 1. **S'arrête sur la fenêtre du choix de la tactique** ; prendre Au bon moment (6) fait jouer le fantôme en premier : sa section du panneau se remplit (deck, défausse cliquable, dernier tour), ligne « Le joueur fantôme joue en premier » (spec 061). |
| `fantome-annonce` | Même départ, le fantôme a déjà retourné 14 cartes (deck à 2), Lève-tôt prise, début du tour 1. Finir le tour lui fait retourner ses 2 cartes ; finir le suivant le fait annoncer (« A annoncé la fin de la manche : c'est votre dernier tour. » en orange dans sa section, plus de bouton « Finir la manche… ») ; le dernier tour mène à la manche 2, l'action avancée ajoutée à son deck (17) et un cristal gagné (spec 061). |
| `fantome-competence` | Même départ, gloire 1, niveau 1, Lève-tôt prise, les Rôdeurs à côté du village. Les vaincre (+2) puis finir le tour mènent au choix de la compétence du niveau 2 ; la compétence du fantôme apparaît alors dans l'offre commune (spec 061). |

Les parties sont dans `scripts/parties/<nom>.json`, des sauvegardes ordinaires (spec 024) construites événement par événement,
comme l'arène des tests : tuiles de campagne 1 et 2 posées, le reste de la pioche de tuiles de la vraie mise en place (graine 0),
la suite du hasard reprise après elle. Elles sont **générées et vérifiées** par `crates/arcanum-persistance/tests/parties_preparees.rs` : le test
échoue si un fichier ne correspond plus à sa construction (format des événements changé, par exemple), ou si la partie chargée
n'est plus dans la situation du tableau. Régénérer avec
`ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test parties_preparees` et relire le diff.

Une partie commence au début du tour (issue privée 296) : rien dans « Ce tour » ni bandeau de niveau venus de sa construction. Les
événements qui posent la situation (cristaux, unités, sites conquis avant) se terminent par `turn_start`, la gloire et le niveau
se posent par `settled_progress`, et seule une conquête « de ce tour » (récompense de fin de tour) vient après le début du tour.
Un test le vérifie pour toutes les parties.

Ajouter une partie : une fonction de plus dans ce test (ses événements et ce qu'elle promet), une ligne dans ce tableau, puis
régénérer.

**Une partie mène à l'écran visé, et son test le prouve** (issue privée 131, bilan de la spec 044 : la main de `sombre-terrier` ne
pouvait pas gagner le combat, la fenêtre des cristaux n'a été vue que par les tests) :

- ce qu'elle promet (`check`) est l'**écran visé** lui-même (choix en attente, actions légales), pas seulement son point de départ ;
- un écran long ou incertain à atteindre (combat à gagner, dé à obtenir) : la partie s'arrête **juste devant**, construite en
  jouant les actions (`decide`) depuis la situation de départ, comme `sombre-terrier` ou `fin-de-partie` ;
- si elle tire du hasard, elle garde les positions des générateurs (`rngs`), pour que la suite au chargement soit celle du tableau ;
- sinon, quand le tableau décrit un chemin à jouer à la main, `check` le rejoue (mêmes actions) jusqu'à l'écran visé.

## Ce qu'il faut savoir

- **Écran** : 1600×900 par défaut, la fenêtre est placée en (0, 0) et occupe tout l'écran. La barre de menu occupe les 26 pixels du haut (bouton Menu en (26, 12), spec 074), le panneau latéral les 340 pixels
  de droite (x ≥ 1260), le plateau de jeu le reste.
- **Repérer où cliquer** : faire une capture, la lire (outil Read sur le PNG), puis cliquer aux coordonnées vues. Chaque clic,
  touche ou survol attend 0,5 s que le jeu se redessine ; `$S attendre 2` pour plus.
- **Carte de la main survolée** : sous le pointeur, une carte remonte d'une cinquantaine de pixels (spec 073) et ses boutons d'effet
  changent de place. Une capture prise sans survol donne donc des coordonnées fausses : le clic tombe à côté, sans erreur. Pour jouer
  un effet de carte : `$S survoler x y` sur la carte, capturer, lire les coordonnées du bouton sur cette capture, puis `$S cliquer`
  dessus (le curseur reste sur la carte, la mise en page ne bouge plus). Si rien ne se passe, refaire une capture après le survol (issue privée 224).
- **Lancement** : l'écran de nouvelle partie (spec 065), centré ; Entrée commence la partie cochée. Pour rejouer la même mise en
  place, saisir une graine dans son champ.
- **Partie différente à chaque lancement** : sans graine saisie, elle vient de l'horloge au clic sur « Commencer », la main et les tuiles changent. Relire une
  capture avant chaque clic plutôt que de rejouer des coordonnées d'un lancement précédent. La graine est écrite dans les logs. Pour une situation
  précise, partir d'une partie préparée (ci-dessus).
- **Captures** : dans `/mnt/project-files/captures/` quand le dossier du projet existe (fil cloud), sinon dans
  `target/verif-manuelle/captures/`. Nommer `spec-0XX-<ce-qu-on-voit>.png`, comme les captures existantes, et les citer dans la
  section « Vérification manuelle » de la spec ou dans la PR.
- **État** : PID, logs du jeu et de Xvfb dans `target/verif-manuelle/` (jamais versionné). `demarrer` sur un jeu déjà lancé ne fait
  rien ; `arreter` puis `demarrer` pour une nouvelle partie.
- **Variables** : `ARCANUM_VERIF_ECRAN` (`:99`), `ARCANUM_VERIF_TAILLE` (`1600x900`), `ARCANUM_VERIF_CAPTURES` (dossier des captures),
  `ARCANUM_VERIF_LANGUE` (`fr`) : langue du jeu, passée en `--lang` (spec 076). Sous Xvfb la langue du système vaut souvent `C` ou
  `en`, ce qui passerait les captures en anglais ; `ARCANUM_VERIF_LANGUE=en` pour vérifier le jeu en anglais.
  `ARCANUM_CONFIG_DIR` : dossier des préférences (spec 080). Non posé, le script en prend un neuf à chaque lancement
  (`target/verif-manuelle/config`), pour qu'une préférence de développement (plein écran, langue) ne fausse pas les captures ;
  posé, il est gardé (relancer avec les mêmes préférences). Sans gestionnaire de fenêtres, le plein écran au lancement ne se
  voit pas : lancer `openbox` sur l'écran virtuel et lire `xprop -id <fenêtre> _NET_WM_STATE`.

- **Focus clavier** (issue privée 364) : Tab parcourt les widgets dans l'ordre où ils sont dessinés, **écran du fond d'abord**, puis
  la fenêtre ouverte par-dessus ; Maj+Tab part de la fin, donc de la fenêtre. Le widget qui a le focus est entouré d'un trait
  clair (poignée d'un curseur, bouton, champ surligné). Un clic sur un curseur ne lui donne **pas** le focus (egui 0.37) : il
  faut y venir au clavier. Pour un curseur horizontal, seules ← et → changent la valeur ; ↑ et ↓ passent le focus au widget
  voisin. Chaque curseur de volume a deux arrêts de Tab : la poignée, puis le champ du nombre (où l'on peut taper la valeur).
  Exemple, régler les effets sonores (spec 081, C21) : depuis l'écran de nouvelle partie,

  ```sh
  $S touche ctrl+comma                       # Préférences
  $S touche shift+Tab shift+Tab shift+Tab    # Fermer, champ « 80 % », poignée des effets sonores
  $S touche Right Right                      # 82 %
  ```

  puis lire `preferences.json` du dossier `MK_CONFIG_DIR` (`target/verif-manuelle/config/` par défaut) : la nouvelle valeur y
  est écrite environ 1 s après la dernière touche. Sous Xvfb il n'y a pas de sortie audio : l'échantillon ne s'entend pas.

## Pièges déjà rencontrés

- Sans `libxkbcommon-x11-0`, winit ne trouve pas le clavier et le jeu s'arrête au lancement : le script l'installe.
- Sans gestionnaire de fenêtres, la fenêtre n'a pas le focus et les touches sont perdues : le script la place, l'agrandit et lui
  donne le focus au lancement.
- Pas de GPU sous Xvfb : rendu OpenGL logiciel (`LIBGL_ALWAYS_SOFTWARE=1`, posé par le script).
- Les installations passent par `apt-get` (root dans le conteneur cloud, `sudo` ailleurs).
- Les actions du lieu (interagir, piller, incendier, entrer, offrande, défier, engager le combat) sont des boutons ronds posés
  au-dessus de leur case (spec 075) : les repérer sur une capture, comme les cases.
- Le glissement de la figurine (150 ms par pas, essai d'animation de la spec 075) est plus bref qu'une capture : on ne voit que
  l'arrivée. La fluidité se juge sur un poste avec écran.
- Un réglage au clavier qui « ne marche pas » vient souvent du focus, pas du jeu : Tab avait le focus sur l'écran du fond, ou le
  clic n'en donne pas (curseurs egui). Voir « Focus clavier » ci-dessus (issue privée 364).
