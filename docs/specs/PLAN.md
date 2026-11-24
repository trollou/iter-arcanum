# Plan des specs

- Statut : **validé par Mathieu** (2026-10-23), J4 à J6 revus et validés le 2026-10-30.
- Références : `CLAUDE.md`, `docs/decisions/0001` à `0004`, `docs/glossaire.md`, livret `docs/regles/mkue-rulebook.pdf` (transcrit dans `docs/regles/livret/`)

## 1. Principe : des tranches jouables, pas des chapitres

On ne rédige pas le livret chapitre par chapitre (on aurait 200 pages de specs avant la première ligne de code).
On découpe en **jalons** : chaque jalon se termine par quelque chose qu'on peut **lancer et jouer** dans la fenêtre egui,
même très incomplet. Chaque jalon contient quelques specs courtes ; chaque spec suit le cycle
spec validée → tests rouges → code minimal → refactor.

Une spec, c'est :

- **un comportement cohérent**, implémentable en une à trois sessions de travail ;
- **1 à 3 pages**, avec des cas numérotés (C1, C2…) qui deviennent chacun un test ;
- **une seule à la fois** en cours d'implémentation. Je peux rédiger la suivante pendant que tu relis le code de la précédente.

## 2. Jalons et specs

Les numéros de page renvoient au livret MKUE.

**Ce tableau est l'index des specs.** La colonne **Statut** dit où en est chacune (voir § 4) ; le reste à faire, ce sont les lignes
qui ne sont pas `implémentée`, dans l'ordre des tableaux : le numéro d'une spec est un identifiant, pas un rang. Le nom
d'une spec rédigée renvoie à son fichier.

**À tenir à jour** : toute PR qui rédige une spec, change son statut ou la renumérote met à jour sa ligne ici (statut, lien), dans la
même PR que l'en-tête « Statut » de la spec. Le statut dit ce qui est sur `main` une fois la PR fusionnée : une PR qui implémente une
spec la passe à `implémentée`.

Une issue intégrée au plan (une ligne ici renvoie à elle) porte le label GitHub `planifié` jusqu'à l'implémentation de sa spec
(`docs/issues.md`).

Chaque jalon de règles (J1 à J6) se termine par une **spec de parcours**, numérotée en 9 (028 pour J2, la 029 étant prise). Elle écrit au moins un
parcours de bout en bout (`crates/arcanum-core/tests/parcours_*.rs`, ADR 0006), qui joue ce que promet la ligne **Jouable** du jalon. J0 n'a pas de
parcours, puisqu'il n'a aucune règle de jeu.

### J0 — Squelette (aucune règle de jeu)

| Spec | Statut | Contenu | Crates |
|---|---|---|---|
| [000 Noyau du moteur](000-noyau-moteur.md) | implémentée | `State`, `Action`, `Event`, `apply` pure, décision en attente, actions légales, générateur à graine, vue joueur, journal. Traduit les ADR 0001 et 0002 en comportements testables, sur un « jeu jouet » (ex. piocher une carte). | arcanum-core |
| [001 Libellés](001-libelles.md) | implémentée | Chargement d'un `<langue>.json`, clés hiérarchiques, paramètres nommés, clé manquante visible, test de cohérence entre langues (ADR 0003). | arcanum-i18n |
| [002 Boucle de partie et fenêtre minimale](002-boucle-et-fenetre.md) | implémentée | Fenêtre eframe qui affiche l'état du jeu jouet, le joueur humain implémente `Player`, la boucle de partie ne bloque pas l'UI. | arcanum-ui |
| [003 Logs techniques](003-logs-techniques.md) | implémentée | Façade de logs commune aux crates, niveaux, fichier de logs dans le dossier de données de l'utilisateur (Windows et Linux), écriture des manques de libellés relevés par arcanum-i18n. À ne pas confondre avec le journal des événements (`EventLog`). | arcanum-ui, arcanum-i18n |
| [004 Icône du jeu](004-icone-du-jeu.md) | implémentée | Icône de la fenêtre et du programme (Windows et Linux), dessinée par nous en SVG. Ajoutée après la validation du plan. | arcanum-ui |

**Jouable :** une fenêtre qui affiche une main et un bouton « piocher ».

### J1 — Se déplacer et explorer

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [010 Plateau de jeu hexagonal](010-plateau-hexagonal.md) | implémentée | Coordonnées hex, tuiles de 7 hexagones, terrains, orientation, voisinage. | p. 2-3, 7 |
| [011 Affichage du plateau de jeu](011-affichage-plateau.md) | implémentée | Dessin egui des tuiles, terrains et héros (formes et couleurs provisoires), zoom et déplacement de la vue, survol d'un hexagone. Pas encore d'action. | — |
| [012 Scénario et mise en place](012-scenario-mise-en-place.md) | implémentée | Définition d'un **scénario** comme donnée (voir ci-dessous) et mise en place pilotée par lui. Configuration de départ = scénario + héros + graine (+ variantes plus tard), c'est elle que la sauvegarde enregistre. Premier scénario : un « bac à sable » d'entraînement sans ennemis. Affichage de la zone interdite par la forme du plateau de jeu (brouillard ou hachures). | p. 1, 3, 14 |
| [013 Jouer des cartes](013-jouer-des-cartes.md) | implémentée | Jouer une carte pour son effet de base, ou de côté (Mouvement/Influence/Attaque/Parade 1), cumul des effets. Sans mana. | p. 4 |
| [014 Déplacement](014-deplacement.md) | implémentée | Coût des terrains de jour et de nuit, terrains infranchissables. Les points non utilisés sont perdus en fin de tour (016) ou au début de la phase d'action (première action). | p. 6-7 |
| [015 Exploration](015-exploration.md) | implémentée | Révéler une tuile adjacente au bord du plateau de jeu, coût 2, pioche de tuiles. | p. 7 |
| [016 Fin du tour](016-fin-du-tour.md) | implémentée | Défausse des cartes jouées, repioche jusqu'à la limite de main. | p. 11 |
| [017 Cases atteignables](017-cases-atteignables.md) | implémentée | Mettre en évidence toutes les cases atteignables avec les points disponibles (trajet le moins cher), tracé du trajet au survol, clic qui suit le trajet. | p. 7 |
| [019 Parcours du jalon 1](019-parcours-j1.md) | implémentée | Parcours de bout en bout du J1 (ADR 0006), avec le joueur scripté apporté par la 028. Mise en place du bac à sable, puis plusieurs tours : jouer des cartes, se déplacer de jour, explorer, finir le tour et repiocher. | — |

**Jouable :** voir le plateau dès 011, puis promener son héros et révéler le plateau de jeu, sans ennemis.

#### Comment on évite de figer un scénario unique

Un scénario n'est pas codé en dur dans la mise en place : c'est une valeur (définie en Rust dans `arcanum-core` : ses règles spéciales sont du code, spec 029) que la mise en place et le moteur consultent. La spec 012 fixe sa forme ; chaque scénario ajouté ensuite n'est qu'une nouvelle valeur, plus ses règles spéciales.

- **Paramètres** : forme du plateau de jeu (coin, ouverte), face de la tuile de départ, composition de la pioche de tuiles (campagne, royaume, cités), nombre de manches, nombre de joueurs, joueur fantôme oui/non, unités d'élite oui/non, niveaux de cité.
- **Règles spéciales** : points d'extension nommés et typés, pas du code libre. Par exemple *condition de fin de partie*, *calcul du score*, *choix de tactique du joueur fantôme*, *restrictions* (pas de combat entre joueurs…). Chaque scénario choisit une implémentation pour chaque point ; une règle de scénario nouvelle = un nouveau point d'extension, introduit par une spec.
- **Variantes** (p. 16-17) : même mécanisme, cumulables avec le scénario, hors périmètre pour l'instant mais la configuration de départ leur réserve une place.
- **Tests** : la mise en place est testée sur au moins deux scénarios différents dès 012 (le bac à sable et un scénario de test), pour garantir qu'aucun n'est câblé en dur.

### J2 — Mana et fin de manche

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [020 Source et dés de mana](020-source-et-des-de-mana.md) | implémentée | Lancer à la mise en place, règle « au moins la moitié de couleurs de base », dés épuisés (noir de jour, or de nuit), Source affichée. Prendre un dé passe dans la 022 (spec 020, Q1). | p. 3, 5 |
| [021 Cristaux](021-cristaux.md) | implémentée | Inventaire, limite de 3 par couleur, gagner un cristal en finissant son tour sur une mine (avancée de la 044). Utiliser un cristal et les jetons de mana passent dans la 022 (spec 021, Q1). | p. 5, 11 |
| [022 Effets renforcés](022-effets-renforces.md) | implémentée | Payer la couleur pour l'effet renforcé avec un dé de la Source (un par tour, relancé en fin de tour) ou un cristal ; or et noir selon Jour/Nuit. Cartes qui produisent du mana et jetons de mana : spec 025 (spec 022, Q1). | p. 4-6, 11 |
| [025 Cartes de mana](025-cartes-de-mana.md) | implémentée | Jetons de mana dans la zone de jeu, rendus à la réserve en fin de tour ; effets qui font gagner un jeton ou un cristal (jeton à la place d'un 4e cristal) ; dés supplémentaires et dés mis sur une couleur (Source de mana, Attraction de mana) ; Concentration, Pouvoir de concentration, Cristallisation, Euphorie cristalline. À faire juste après la 022. | p. 5, 11 |
| [023 Fin de manche](023-fin-de-manche.md) | implémentée | Deck vide, annonce de fin de manche, alternance Jour/Nuit, nouvelle manche (sans tactiques ni offres). Nombre de manches du scénario (4 pour le bac à sable) et fin de partie (spec 023, Q1). Affichage « Manche 2/6 · Nuit (tour 1) » (numéro de manche ; remplace la proposition de la spec 016 Q5). | p. 4, 11 |
| [024 Sauvegarde et retour en arrière](024-sauvegarde-et-retour-en-arriere.md) | implémentée | Revenir en arrière décision par décision jusqu'au point de non-retour (révélation ou début du tour). Sauvegarde = journal (configuration dans `GameStarted`) + position du générateur, rechargement, sauvegarde automatique, pas de migration avant la fin du J6 (instantanés en mémoire seulement : spec 024, Q1). Crée arcanum-persistance. | p. 5, ADR 0001 |
| [029 Contenu du jeu en données](029-contenu-en-donnees.md) | implémentée | Cartes, decks de départ et tuiles décrits dans des fichiers de contenu (RON) embarqués dans arcanum-core et validés par des tests, au lieu de constantes Rust. À implémenter après la fusion des 022 et 023, avant la 030 : les ennemis et les sites seront écrits directement en données. Technique, rien ne change pour le joueur. | — |
| [028 Parcours du jalon 2](028-parcours-j2.md) | implémentée | Parcours sur plusieurs manches : effets renforcés payés par un dé, un cristal ou un jeton, cristal gagné sur une mine, fin de manche et passage à la nuit, puis sauvegarde, rechargement et annulation. Premier parcours écrit : apporte le joueur scripté (ADR 0006). | — |

**Jouable :** plusieurs manches de déplacement avec la mana, sauvegarde et annulation.

### J3 — Combat

Le plus gros morceau, découpé finement.

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [030 Ennemis errants et provocation](030-ennemis-errants.md) | implémentée | Jetons d'ennemis, révélation, provoquer en passant à côté. | p. 7-8 |
| [031 Déroulé d'un combat](031-deroule-combat.md) | implémentée | Défier, engager le combat ; phases : distance/siège, parade, dégâts, attaque. Victoire, gloire (total), Blessures en main ; fortifié (jeton). Éléments et capacités ignorés en attendant 032 et 033 (spec 031, Q1). | p. 8-10 |
| [032 Éléments et résistances](032-elements-resistances.md) | implémentée | Physique, Feu, Glace, Feu glacé ; parade efficace ou inefficace ; résistances, partagées par le groupe ciblé (bandeau « Le groupe partage », maquette de la spec 031). Polyvalence renforcée, avec la première attaque de siège (spec 032, Q1). | p. 9-10, 24 |
| [033 Capacités des ennemis](033-capacites-ennemis.md) | implémentée | Vif, Brutal, Toxique, Paralysant, Invocation (créature marron tirée au début de la phase de parade) ; pile de secours quand une pile d'ennemis est épuisée ; Endurance givrée renforcée. Capacités des extensions hors périmètre. | p. 9-10, 24 |
| [034 Blessures et soins](034-blessures-et-soins.md) | implémentée | Assommé, repos standard et prolongé, soins et pioche (Tranquillité, Rajeunissement), reprise d'Euphorie cristalline avec une Blessure (les Blessures en main et le repos prolongé arrivent avec la 031). | p. 6, 9, 11 |
| [035 Gloire et niveaux](035-gloire-et-niveaux.md) | implémentée | Piste de gloire, changement de niveau, limite de main, armure, jetons de commandement (sans compétences ni actions avancées). | p. 6, 11 |
| [036 Trajet en un clic](036-trajet-en-un-clic.md) | implémentée | Cliquer une case atteignable lointaine fait suivre le trajet case par case (spec 017, Q1). Le calcul de la distance et le trajet suivi évitent les provocations (spec 030, C19). Demandé par Mathieu (2026-10-27). | p. 7 |
| [037 Survol du plateau près du curseur](037-survol-infobulle.md) | implémentée | Les lignes de survol d'une case (coût, jeton d'ennemi, provocation) s'affichent dans une infobulle à côté du curseur au lieu du bas du panneau latéral, souvent hors de vue (issue privée 50). Interface seulement. | — |
| [038 Un générateur par usage](038-generateurs-par-usage.md) | implémentée | Un générateur par usage du hasard (Source, pioche de tuiles, deck, chaque pile d'ennemis), dérivé de la graine : un nouveau tirage ne décale plus les autres ni les graines des parcours (issue privée 52, ADR 0007). Sauvegarde : une position par usage. Technique, rien ne change pour le joueur. | — |
| [039 Parcours du jalon 3](039-parcours-j3.md) | implémentée (révision du départ, issue privée 322) | Provoquer un ennemi errant, le combattre (parade, dégâts, attaque), se soigner, en défier un second, gagner de la gloire et monter de niveau, puis passer à la manche 2 avec ses Blessures ; reprise d’une sauvegarde en plein combat. | — |

**Jouable :** combattre les ennemis errants et monter de niveau.

### J4 — Une vraie partie courte, puis sites et unités

Revu le 2026-10-30 (fil « Revoir le plan de dev avant le jalon 4 ») : le premier scénario du livret passe en tête, et les offres
avant les sites, car presque tous les sites donnent ou vendent des cartes. Les numéros déjà prévus sont gardés (ils sont cités
dans le code et les specs) : l'ordre des lignes fait foi, pas le numéro.

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [063 Première reconnaissance en solo](063-premiere-reconnaissance.md) | implémentée | Scénario par défaut à la place du bac à sable (gardé pour les tests et les parties préparées) : 1 gloire par tuile révélée, cités ni conquises ni visitées, fin quand une cité est révélée (dernier tour), décompte standard sans titres sur ce qui existe, écran de fin. Sans joueur fantôme, permis pour une première partie (spec 061 pour l'ajouter). | p. 15, 18 |
| [065 Nouvelle partie](065-nouvelle-partie.md) | implémentée | Écran de nouvelle partie : héros (les quatre), scénario (Première reconnaissance ou bac à sable), graine. Une illustration SVG par héros : figurine sur le plateau, portrait dans le panneau (demandé par Mathieu, 2026-10-30). | p. 1 |
| [050 Offres et cartes gagnées](050-offres-et-cartes-gagnees.md) | implémentée | Offres d'unités, d'actions avancées et de sorts : mise en place, renouvellement, unités d'élite après une tuile royaume ; gagner une carte (offre ou deck) ; action avancée des niveaux pairs. Toutes les cartes en données ; un effet pas encore codé est `Unsupported`, la carte se joue de côté (p. 4, 3.e). | p. 3, 4, 5, 11 |
| [040 Interaction et réputation](040-interaction-et-reputation.md) | implémentée | Piste de réputation et bonus d'interaction, réputation des ennemis errants vaincus ; interagir avec les habitants d'un village ou d'un monastère (l'action du tour, sans achat : 041 et 042) ; Menace renforcée et Nobles manières (spec 022, Q2). | p. 6, 8, 10 |
| [042 Unités](042-unites.md) | implémentée | Recrutement, activation, unités en combat, unités blessées ; capacités simples (Attaque, Parade, Mouvement, Influence, Soin, mana), les autres `Unsupported` jusqu'au J5. L'offre d'unités est dans la 050. | p. 5, 8-10 |
| [041 Villages et monastères](041-villages-et-monasteres.md) | implémentée | Soins achetés, actions avancées des monastères dans l'offre d'unités et leur achat, monastère incendié (combat sans unités, jeton Bouclier, artefact en récompense de fin de tour), Le Collectionneur et Le Conquérant ; pillage d'un village. | p. 8 + cartes de site |
| [043 Sites fortifiés](043-sites-fortifies.md) | implémentée | Forteresses, tours de mage : défenseur face cachée, assaut, conquête, retraite, sort en récompense ; recruter et acheter des sorts dans un site conquis, limite de main près d'une forteresse. Retraite forcée et case sécurisée : spec 052 (spec 043, Q1). | p. 7-11 + cartes de site |
| [044 Sites d'aventure](044-sites-d-aventure.md) | implémentée | Donjon, tombeau, antre, Sombre Terrier, clairière magique ; récompenses de combat (dé de récompense, cristaux aléatoires). Ennemis du combat désignés par leur jeton (deux jetons sur une case, spec 044, Q1). | p. 11 + cartes de site |
| [045 Ruines](045-ruines.md) | implémentée | Jetons de ruines (pile, face cachée la nuit, révélés au Jour ou en y entrant) ; autel : offrande de 3 mana pour 7 gloire ; ennemis tirés selon le jeton, restés sans remplacement ; récompenses du jeton (artefacts *n* + 1 tirés, sort, action avancée, un cristal de chaque couleur, unité de l'offre). | p. 3, 10, 11 + carte de site |
| [049 Parcours du jalon 4](049-parcours-j4.md) | implémentée | Une partie de Première reconnaissance jusqu'au score : recruter une unité au village, explorer, conquérir la forteresse de la campagne 3 avec elle, gagner l'action avancée du niveau 2, explorer jusqu'à la cité, dernier tour, décompte vérifié ligne par ligne ; reprise d'une sauvegarde. | — |

**Jouable :** une partie complète de Première reconnaissance en solo, avec un héros au choix, villages, monastères, unités, sites et
récompenses. Une carte gagnée dont l'effet n'est pas encore codé se joue de côté.

### IHM — Refonte de l'interface

Ajouté le 2026-11-05 (fil « Discuter la refonte de l'IHM »), après le J4 et avant le J5. Interface seulement, aucune règle nouvelle :
la base de chaque spec est la synthèse [refonte-ihm.md](refonte-ihm.md), les images suivent l'[ADR 0009](../decisions/0009-illustrations-generees.md).
Pas de spec de parcours : rien ne change dans arcanum-core, la vérification se fait à l'œil (ADR 0006).

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [070 Combat et offres](070-combat-et-offres.md) | implémentée | Ennemi coché d'office s'il est seul, bouton « Parer », attaque ou parade insuffisante non proposée, carte d'ennemi simplifiée ; police des offres, plus de taille de pioche. | — |
| [071 Icônes et raccourcis visibles](071-icones-et-raccourcis.md) | implémentée | Jeu d'icônes SVG (Mouvement, Attaques par élément, distance, siège, Influence, cristaux, Armure, Gloire, héros et drapeaux) avec infobulles ; raccourcis affichés ; jouer de côté par boutons-icônes. | — |
| [072 Illustrations générées](072-illustrations-generees.md) | implémentée (affichage, script et premier lot de 41 images ; reste la fusion) | Script de génération, préambule de style, premier lot (unités, actions de base et Blessure) en images carrées, vignettes sur les cartes et les unités, repli sans image (ADR 0009). Héros et monstres restent en SVG (spec 072, Q5). | — |
| [073 Panneau du bas et panneau de droite](073-panneaux.md) | implémentée | Unités, main et compétences en bas, sur le plateau comme en combat (chevauchement animé, base et renforcé séparés, boutons Jouer et Renforcer en icônes : renvoyés là par la spec 071) ; à droite le joueur, la zone de jeu avec les événements du tour, Fin du tour et Offres toujours visibles. Grosse spec : deux fils (issue privée 118). | — |
| [074 Barre de menu](074-barre-de-menu.md) | implémentée | Menu (revenir en arrière, charger, sauvegarder, nouvelle partie, quitter, plein écran), scénario, jour ou nuit, manche et tour, pioches de tuiles. | — |
| [075 Plateau isométrique](075-plateau-isometrique.md) | implémentée (fil A, géométrie : vue inclinée, mer, tranche, éléments debout triés, caméra bornée ; fil B, éléments debout : figurines, brouillard, décor de relief, figurines au combat, jetons supprimés ; fil C : boutons d'action sur les cases, menu du site, section Ici retirée, essai d'animation du héros ; dessins D1 à D5 : toutes les figurines, [plan](plans/075-plateau-isometrique.md)) | Vue isométrique 2D, relief (forêts, collines, montagnes), sites et figurines debout (héros et monstres en SVG dessinés par le code, `assets/images/figure/`, spec 072 Q5 ; les héros vectorisés de l'issue privée 167 servent de modèle), monstres des sites en figurine ou dans le brouillard (issue privée 165), mer au-delà de la côte, zoom et défilement bornés, boutons d'action sur les cases à la place de la section Ici (issue privée 169), essai d'animation sur le héros (le reste des animations : spec à part si l'essai convainc). | — |

**Jouable :** la même partie qu'au J4, avec l'interface refaite.

### J5 — Les cartes jouent leur effet

Les specs d'interface 076, 077 et 078 (issues privées 195, 180 et 162) sont placées après les artefacts à la demande de Mathieu (2026-11-12) : numérotées avec l'IHM, elles ne touchent pas aux règles.

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [052 Moteur d'effets](052-moteur-d-effets.md) | implémentée | Effets en cours (modificateurs, ADR 0008) : coût des terrains et lacs franchissables, conversions (Mouvement en Attaque, Influence en Parade), bonus sur la prochaine carte, effets de fin de tour ; consultés par le déplacement, le combat et la fin du tour, suivis par la sauvegarde et le retour en arrière. Retraite forcée et case sécurisée (spec 043, Q1). Prouvé sur 7 actions avancées et les Bûcherons (spec 052, Q8). Carte jouée depuis une offre : 054 (spec 052, Q2). | p. 5, 7, 11 |
| [053 Actions avancées : combat, unités et soins](053-actions-avancees.md) | implémentée | Coup de sang, Bouclier de glace renforcé, Marche revigorante, Mélodie du vent, Magie pure (second mana), Récit héroïque, Régénération, Au cœur du combat, Besoin d'aide ; premiers modificateurs d'ennemis (armure réduite, n'attaque pas) et règle générale des résistances aux effets ; capacités d'unités encore `Unsupported` (Illusionnistes, Glaceurs d'Amotep, Mages et Gardiens d'Altem). Premier des deux lots (spec 053, Q1). | p. 8 + cartes |
| [056 Actions avancées : mana et cartes](056-actions-avancees-mana.md) | implémentée | Rituel de sang, Décomposition, Maîtrise du cristal, Tempête de mana, Effet maximum, Apprentissage, Entraînement, Don pour la magie renforcé (spec 053, Q1). Don pour la magie de base : 054. | cartes |
| [054 Sorts : lancer un sort, sorts de combat](054-sorts.md) | implémentée | Payer un sort (sa couleur ; renforcé de nuit, mana noir en plus) ; les 10 sorts de combat (Éclair de feu, Tempête de neige, Exposition, Secousse, Mur de flammes, Éclair de mana, Tourbillon, Bouclier ardent, Frisson, Démolition) ; ennemi visé, perdre fortifications et résistances, détruire un ennemi ; Don pour la magie de base, sort lancé depuis l'offre (spec 052, Q2). Premier des deux lots (spec 054, Q1). | p. 4 + cartes |
| [057 Sorts : déplacement, unités et mana](057-sorts-deplacement-unites-mana.md) | implémentée | Voyage souterrain, Ailes du vent, Restauration, Distorsion spatiale, Appel aux armes, Méditation, Fuite de mana, Puits de mana, Lecture d'esprit, Flux d'énergie (spec 054, Q1) ; ce qui vise les autres joueurs, ramené au solo. | p. 4 + cartes |
| [055 Artefacts](055-artefacts.md) | implémentée | Les 16 artefacts : jouer pour l'effet de base, détruire pour l'effet renforcé ; bannières assignées aux unités (bonus, capacités, perte, reprise en fin de manche, décompte) ; anneaux (mana illimité), Épée de justice (Attaques physiques doublées), Cor de colère et Sac de gemmes (dés lancés), Coupe d'or, Livre de sagesse, amulettes (règles de jour la nuit, de nuit le jour). | p. 4, 5 + cartes |
| [076 Choix de la langue](076-choix-de-la-langue.md) | implémentée | Interface seulement. Au lancement, la langue du système (anglais si ce n'est ni le français ni l'anglais) ; paramètre facultatif `--lang fr\|en` qui la force, repli en anglais avec un message d'erreur dans les logs si la langue demandée n'existe pas. Détection sous Windows par un appel API derrière `#[cfg(windows)]`, sans dépendance nouvelle s'il n'y a pas d'inconvénient. Les logs techniques restent en français ; une ligne de l'ADR 0003 (langue par défaut) à mettre à jour. Menu de préférences plus tard, à la place du paramètre (issue privée 195). | — |
| [077 Cartes agrandies](077-cartes-agrandies.md) | implémentée | Interface seulement. Illustration en bande sous le nom des cartes et des unités du panneau du bas (recadrée au milieu), cartes de 216 × 240 pour qu'une carte simple reste entière, Blessure en image seule ; pas d'image au survol. Lève la Q6 de la spec 072 (issue privée 180). Illustrations refaites pour la bande : issue #9. | — |
| [078 Nouvelle partie en étapes](078-nouvelle-partie-en-etapes.md) | implémentée | Interface seulement. L'écran de nouvelle partie (spec 065) devient un assistant plein écran à étapes : barre d'étapes cliquable (Scénario, Héros, puis Options plus tard : joueur fantôme, variantes), chaque étape avec la liste à gauche et une grande fiche à droite. Fiche du scénario : texte d'ambiance, durée, joueurs, type, objectif, règles spéciales, fin de partie, décompte, ce qui n'est pas encore dans le jeu. Fiche du héros : grand portrait, description en italique sous le nom (texte original d'après le walkthrough p. 20), style de jeu (cartes propres, compétences, profil), cartes propres lisibles avec leur effet ; pas de compétences avant la 051. Derniers choix retenus, « Commencer » dès l'étape Scénario, graine dans un volet « Avancé », Ctrl+N ouvre le même écran avec « Annuler ». Texte plus grand. Maquette : `/mnt/project-files/ihm/078/` (issue privée 162). | — |
| [051 Compétences : gain et premier lot](051-competences.md) | implémentée | Pile de compétences du héros (coopérative gardée, compétitive retirée en solo), deux révélées à chaque niveau pair : en garder une, ou en prendre une d'un autre héros dans l'offre de compétences communes contre l'action avancée en dernière place ; une fois par tour, une fois par manche (retournée) ; section Compétences du panneau du bas. Les 23 compétences sans règle nouvelle (points, cristaux et jetons, Motivation, Potion de soin, Inspiration) ; les 17 autres se gagnent sans servir (spec 051, Q1). | p. 3, 4, 5, 11, 15 |
| [058 Compétences : second lot](058-competences-second-lot.md) | implémentée | Les 17 compétences qui demandent une règle nouvelle : cartes de côté modifiées (Souffrance sublimée, Pouvoir universel, Flexibilité, Magie inutile), mana transformé (Invocation, Polarisation), Vol, Marche en avant, Fortune clinquante, Charisme, Loyauté sans faille, Botte secrète, Même pas mal, et les quatre coopératives posées au centre, dont l'avantage différé revient au joueur à son tour suivant en solo (Rituel de guérison, Embrasure de la Source, Temps calme, Surcharge de mana ; Compendium, *Skill Tokens* S5). | p. 5 + compétences |
| [059 Parcours du jalon 5](059-parcours-j5.md) | implémentée (révision du départ, issue privée 318) | Monter de niveau, choisir compétence et action avancée, les jouer ; acheter un sort et le lancer. | — |

**Jouable :** chaque carte et chaque compétence du jeu de base fait ce qu'elle dit.

### J6 — Une vraie partie solo

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [060 Tactiques](060-tactiques.md) | implémentée | Choix des tactiques de jour et de nuit (le joueur choisit le premier, tactique retirée en fin de manche, MKUE p. 19 ; retrait paramétré par le scénario), les douze effets ; Première reconnaissance seulement, pas le bac à sable. Ordre du tour et tactique du joueur fantôme : spec 061 (spec 060, Q1). | p. 4, 11, 19 |
| [061 Joueur fantôme](061-joueur-fantome.md) | implémentée | Automate de fin de manche (3 cartes retournées, plus une par cristal de la couleur de la dernière ; annonce deck vide), cristaux des cercles du héros, deck qui grossit de l'action avancée en dernière place, cristal du sort en dernière place, ses compétences dans l'offre commune ; sa tactique tirée parmi les restantes et l'ordre du tour ; ajouté à Première reconnaissance, option pour s'en passer et choix de son héros, « au hasard » par défaut (étape « Options » de la nouvelle partie). | p. 4, 6, 15, 18, 19 |
| [062 Cités](062-cites.md) | implémentée | Cités révélées avec leurs défenseurs selon le niveau donné par le scénario (bac à sable : une cité de niveau 5 ; Première reconnaissance : cités fermées), retournés à côté du héros même de nuit ; assaut contre tous les défenseurs restants, fortifiés, avec le bonus de la cité (armure, attaque, Brutal, Toxique) ; un Bouclier par défenseur vaincu, retraite sans perdre ses Boucliers, conquête et Seigneur ; cité conquise : interaction avec bonus des Boucliers, recrutement des unités à icône Cité, achat propre à sa couleur, limite de main +1 ou +2. | p. 7, 8, 10, 11, 18 ; cartes de cité |
| [064 Conquête en solitaire](064-conquete-en-solitaire.md) | implémentée | Le scénario solo de référence : 6 manches, forme en coin standard (pas de tuile royaume sur la côte), deux cités de niveaux 5 puis 8, joueur fantôme obligatoire, sorts compétitifs retirés ; toutes les cités conquises, un dernier tour sans le fantôme ; écran de fin « mission accomplie / échouée » et décompte solo (+10 par cité, +15 toutes, +30 par manche non jouée, deck du fantôme, +5 sans annonce). | p. 14, 15, 19 |
| [069 Parcours du jalon 6](069-parcours-j6.md) | implémentée | Conquête en solitaire de bout en bout. Parcours principal en mise en place arrangée (Tovak, fantôme Goldyx) : six manches, tactiques choisies puis retirées (chacune prise une fois), ordre du tour, tours et annonce du fantôme, cités révélées (niveaux 5 puis 8), pas de tuile royaume sur la côte, fin par la limite de manches et décompte de la mission échouée ; reprise d.une sauvegarde. Épisode posé : la seconde cité révélée, assaillie, retraite puis conquête, dernier tour sans le fantôme, mission accomplie et décompte solo. | p. 4, 10, 14, 15, 19 |

### Finitions — Animations, son, préférences, publication

Fonctionnalités venues d'issues, intégrées au plan à la demande de Mathieu (2026-11-13) ; le support de macOS (issue #7) suit le dépôt public, la Source dans la barre du haut (issue #10) précède les cartes en main épurées (spec 089, issues #11, #12 et #13), qui fixent la présentation que reprennent les cartes des fenêtres (spec 086), les illustrations des sorts suivent celles-ci et précèdent les illustrations refaites en bande (issue #9), les animations (issues #5 et #4), les moins urgentes, viennent ensuite, puis deux règles reportées (issues #8 et #6, Mathieu, 2026-11-18 : rien d'urgent), et l'aide des touches (issue #1) ferme le plan.
Interface et outillage seulement, sauf ces deux règles reportées : pas de parcours.

**Avant les specs de Finitions**, une fois le J6 fini (Mathieu, 2026-11-17), des issues traitées une à la fois, un fil par issue (`docs/issues.md`), dans cet ordre (planifiées le 2026-11-18) :

| Issue | Contenu |
|---|---|
| issue privée 322 Parcours du J3 | Partir d'une mise en place arrangée plutôt que d'une graine rare (6 graines sur 400), comme le J5 (ADR 0006, « Départ d'un parcours »). |
| issue privée 317 Compétences révélées trop tôt | Révéler les deux compétences de la montée de niveau après le choix des récompenses, pas dès la fin du tour (MKUE p. 11, *End of the Turn* 5 puis 6.b ; spec 051). |
| issue privée 307 Sortir du menu d'interaction | Fermer le menu d'interaction sans rien avoir dépensé ni acheté annule le début de l'interaction (comme Ctrl+Z), pour pouvoir encore se déplacer : commencer l'interaction est l'action du tour, après laquelle on ne se déplace plus (MKUE p. 7, 1). |

| Spec | Statut | Contenu | Livret |
|---|---|---|---|
| [080 Préférences](080-preferences.md) | implémentée | Premier écran de préférences, gardées entre deux sessions : langue (reprend le paramètre `--lang` de la spec 076), plein écran par défaut, volumes par type de son (spec 081). Préalable de l'audio (issue privée 187). | — |
| [081 Audio](081-audio.md) | implémentée | Musique et bruitages (issue privée 187). Une crate nouvelle (`arcanum-audio`) reçoit les événements du jeu et pilote musique de fond, ambiances et effets : musique qui passe progressivement de l'exploration au combat, sons superposés sans couper la musique (hibou à la nuit, coq au jour, parade, blessure, ennemi vaincu, fin de combat, sort lancé…), fondus, volumes indépendants par type de son, spatialisation éventuelle. Dépendance envisagée : Kira, à proposer et justifier (licence via `cargo deny`, Windows et Linux). Sons sous licence libre compatible (Freesound, OpenGameArt, Pixabay, Sonniss…), licence de chaque fichier vérifiée et consignée, rien d'officiel. Sans doute une grosse spec (deux fils, issue privée 118). | — |
| [082 Dépôt public](082-depot-public.md) | implémentée (publié le 2026-11-24 ; ménage du dépôt privé à suivre) | Publication sous le nom Iter Arcanum (issue privée 256), **variante C** (Mathieu, 2026-11-14) : pas de miroir, le projet passe dans un **dépôt public neuf** (`iter-arcanum`) qui devient le dépôt de travail : code, `assets/`, nos propres docs (specs, `PLAN.md`, ADR, architecture, glossaire, `docs/reference/bac-a-sable.*` et `index-par-notion.md`, CLAUDE.md, `.claude/`) et les issues. Le dépôt privé actuel ne garde que le matériel de tiers de `docs/regles/` (livret, FAQ, compendium, scans, cartes, walkthrough) et `scripts/faq-compendium/`, son historique servant d'archive. Le dépôt public part d'un **instantané** de `main` sans ce matériel (pas de `git filter-repo`), avec un garde-fou CI qui refuse tout fichier interdit (PDF, image hors `assets/`, `docs/regles/` hors liste). Avant publication : audit des citations longues du livret, de la FAQ ou du compendium (specs, ADR, glossaire, commentaires, `assets/i18n/`), remplacées par des renvois. Issues : les issues ouvertes sont recréées en public (GitHub ne transfère pas d'un dépôt privé vers un public) ; les renvois « issue #N » du code et des docs sont préfixés ou changés en liens pour éviter les collisions de numéros. Renommage des identifiants internes (crates `mk-*` en `arcanum-*`, `MageKnightApp`, `APP_ID`, icônes, logs, dossiers `iter-arcanum` sans reprise des anciens) ; `cargo deny check`. Les noms officiels des héros et des ennemis sont gardés (Mathieu, 2026-11-13). La rédaction vérifie qu'un fil cloud clone les deux dépôts côte à côte et fixe le chemin stable du livret dans CLAUDE.md (Windows et Linux) ; mise à jour de CLAUDE.md, `.claude/rules/`, `docs/ci.md`, `docs/issues.md` et de la mémoire du projet. | — |
| 085 Support de macOS | à rédiger | macOS rejoint Windows et Linux (issue #7) : détection de la langue du système propre à macOS (spec 076, derrière `#[cfg(target_os = "macos")]`), dossiers de sauvegarde et de logs, icône et paquet d'application, crate audio compatible (spec 081) ; ADR 0004 et CLAUDE.md (plateformes) à mettre à jour. Placée après le dépôt public : la CI macOS, dix fois plus chère en minutes sur un dépôt privé, tourne gratuitement sur le dépôt public. La rédaction dit comment vérifier à l'œil sans Mac. | — |
| 091 Illustrations en bande | à rédiger | Les illustrations existantes des cartes et des unités (spec 072), composées pour un carré de 256 × 256, sont refaites pour la bande d'environ 2,4:1 de la spec 077, qui les recadre au milieu et en perd 58 % : sujet centré dans la bande, scènes plus simples et plus lisibles (issue #9, second volet de l'issue privée 180). Reprend le script `scripts/illustrations/` et le style en bande validé par la spec 087 (ADR 0009) ; aucun code nouveau. La rédaction chiffre le coût (crédit Gemini) avant de lancer et dit quels lots refaire (cartes de base, actions avancées, artefacts, unités) ; planche validée par Mathieu. | — |
| 079 Animations de fin de combat | à rédiger | À la fin d'un combat, quand on revient sur le plateau, les figurines des monstres vaincus disparaissent avec une animation au lieu d'être retirées d'un coup (issue #5). Ajoute la ligne « Animations » du Menu et des préférences (spec 080) qui les coupe (spec 075, Q7). Peu urgent (Mathieu, 2026-11-13). | — |
| 084 Animations des héros et des monstres | à rédiger | Faire mieux que l'essai de la spec 075 (respiration de ± 2 % et glissement du héros, jugé peu convaincant) pour les figurines des héros et des monstres ; d'autres animations que celles de la spec 079 (issue #4). Peu urgent (Mathieu, 2026-11-13). | — |
| 088 Source dans la barre du haut | à rédiger | Interface seulement. La Source de mana (ses dés, spec 020) quitte le panneau de droite, où elle est en tête (spec 073, Q4), pour la barre de menu (spec 074), visible sur le plateau comme en combat (issue #10, Mathieu, 2026-11-13). La rédaction tranche la place et la taille des dés dans la barre, les dés épuisés et pris ce tour, les choix qui visent un dé (prendre un dé, Tempête de mana, Méditation…), la fenêtre étroite et ce que gagne le panneau de droite ; le panneau du bas (spec 077) n'est pas touché. | — |
| 089 Cartes en main épurées | à rédiger | Interface seulement (issue #11). Cartes de la main (spec 077) allégées : le texte complet de la carte passe derrière un bouton « i » à droite du nom, y compris ce qui ne vaut qu'à plusieurs joueurs, tandis que les boutons d'effet restent ceux que la situation permet (Mathieu, 2026-11-18 ; Lecture d'esprit, issue #12, dont seul le cristal joue en solo, paraissait inachevée) ; la séparation devient « —— (pastille du mana) Renforcé —— » et le coût du renfort n'est plus répété devant chaque bouton (un coût en plus s'écrit « coût : effet ») ; artefacts : séparation « —— si détruit —— », sans ligne « Renforcé » ; sorts : nom de base avec sa pastille, puis nom renforcé avec ses deux pastilles, chacun suivi de ses boutons. Corrige les cartes tronquées : boutons d'options qui débordent à droite (Magie pure, Illusionnistes ; issue #13), qui passent à la ligne. La rédaction tranche les artefacts (texte complet utile ou non), et vaut pour les unités. Maquette avant de trancher. | — |
| 090 Ennemis par catégorie | à rédiger | Interface seulement (issue #14). Les ennemis sont nommés par leur catégorie et non par la couleur de leur pile (« Ennemis : gardes de forteresse » au lieu de « Ennemis : gris » sur les ruines, et partout où `enemy.pile_color` sert), et chaque figurine de monstre sur le plateau (spec 075) repose sur un socle ou un cercle de la couleur de sa catégorie (vert, gris, violet, marron, rouge, blanc). La rédaction tranche la forme du cercle, sa lisibilité dans le brouillard et la nuit, et l'écran de combat. | — |
| 086 Cartes des fenêtres | à rédiger | Interface seulement. Fenêtre Offres et fenêtres de choix (artefact, sort) présentées comme les cartes de la main (spec 077) : bande d'illustration sous le nom, même disposition, au lieu de la vignette de 64 en haut à droite. Peut-être une taille de carte minimale, pour que la bande d'une colonne étroite (sorts) ne devienne pas minuscule. Maquette avant de trancher (Mathieu, 2026-11-13, spec 077 Q4). | — |
| 087 Illustrations des sorts | à rédiger | Lot d'illustrations générées pour les 20 sorts, aujourd'hui sans image et trop sobres à l'écran (Mathieu, 2026-11-13) : script et style de la spec 072 (un lot par famille, planche validée par Mathieu, ADR 0009). Faites d'emblée pour la bande de la spec 077 (même format que l'issue #9) ; affichées dans la main sans code nouveau, et dans la fenêtre Offres et le choix du sort avec la spec 086 qui précède. La rédaction chiffre le coût (crédit Gemini) et dit si les actions avancées et les artefacts suivent dans un même lot. | — |
| Issue #8 Bannière de terreur contre les Invocateurs | — | Traitée dans un fil d'issue (`docs/issues.md`), pas une spec. Un moment au début de la phase de parade, avant le tirage de la créature de l'Invocateur (spec 033), où la Bannière de terreur peut annuler son attaque : aucune créature tirée, Gloire +1 (Compendium, *Banner of Fear* Q5 ; l'attaque de la créature tirée reste hors d'atteinte, Q6). Lève la décision provisoire de la spec 055 (Q4, cas C17). | p. 9 |
| 092 Effet maximum complet | à rédiger | Règle. Étend Effet maximum (spec 056, Q1 et Q2) aux options non répétables que règle le compendium (défausses multipliées, modificateurs doublés, Concentration en chaîne, Entraînement répété, Don pour la magie, Rajeunissement : *Advanced Action Cards*, Maximal Effect S6 à S34) et à une option différente à chaque répétition (FAQ p1, Cristallisation renforcée ×2 donne deux cristaux de couleurs différentes ; Compendium S26, S31) (issue #6). Vers la fin des Finitions (Mathieu, 2026-11-18). | p. 5 |
| 083 Aide des touches | à rédiger | Élément « Raccourcis clavier… » du Menu (spec 074) qui ouvre une fenêtre listant la table des touches (`KEY_BINDINGS`) et les touches du plateau sans bouton (flèches, `+`/`-`), avec un libellé et un contexte par touche (environ 25 libellés fr et en) (issue #1). En toute fin de plan (Mathieu, 2026-11-13). | — |

### Hors périmètre (pour l'instant)

Combat entre joueurs (p. 12), assaut coopératif de cité (p. 13), variantes (p. 16-17), autres scénarios, joueurs automatiques (chacun aura sa spec plus tard), réseau.

## 3. Gabarit d'une spec

Voir `_gabarit.md`. En résumé : en-tête (statut, jalon, dépendances, pages du livret), objectif, règles résumées avec pages,
modèle et termes du glossaire, actions et événements, cas numérotés en *Étant donné / Quand / Alors*, cas limites et refus,
information cachée, clés i18n, UI, hors périmètre, questions ouvertes.

Une spec ne peut passer en « validée » que si ses questions ouvertes sont vides.

Toute spec qui touche l'UI remplit la sous-section « Raccourcis clavier » : chaque action à la souris doit se demander si elle mérite
une touche. Les touches prises sont tenues dans `docs/raccourcis.md`, pour qu'une spec n'en reprenne pas une déjà utilisée.

## 4. Organisation des fichiers

```
docs/specs/
├── README.md        # entrée du dossier : renvoie à ce plan
├── PLAN.md          # jalons, index des specs et leur statut
├── _gabarit.md
├── plans/           # plans de développement des grosses specs ; gabarit plans/_gabarit.md
├── 000-noyau-moteur.md
├── 001-libelles.md
└── …
```

Statuts : `à rédiger` → `brouillon` → `à valider` → `validée` → `implémentée`.

| Statut | Sens |
|---|---|
| `à rédiger` | prévue par ce plan, pas encore de fichier |
| `brouillon` | rédaction en cours |
| `à valider` | rédigée, attend la relecture de Mathieu (PR ouverte) |
| `validée` | questions ouvertes vides, validée par Mathieu ; tests et code peuvent commencer |
| `implémentée` | tests et code fusionnés sur `main` |

Une spec validée qui doit changer repasse en `à valider`. L'en-tête « Statut » de la spec et la colonne Statut du § 2 disent la même
chose.
