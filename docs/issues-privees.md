# Issues et PR privées citées

Iter Arcanum a été développé dans un dépôt privé, `trollou/mage-knight`, avant sa publication (spec 082). Les
documents y renvoient sous la forme « issue privée N » ou « PR privée N » : cette page donne leur titre. Les issues
encore ouvertes à la publication ont été recréées ici, et leurs renvois portent leur numéro public.

Page générée une fois par le script de publication ; elle n'est plus mise à jour.

## Issues

| N° privé | Titre |
|---|---|
| 3 | Spec 002 : boucle de partie et fenêtre minimale |
| 25 | Fin du tour : ne pas demander de défausser quand la main est vide |
| 42 | Exposer l'effet d'une carte jouée (CardId + CardPlay → Effect) dans mk-core |
| 43 | Carte de référence des coordonnées du bac à sable |
| 44 | Specs de parcours : fixer les points d'étape, pas le découpage en tours |
| 45 | Vérifier les noms français des jetons d'ennemis |
| 48 | réduire le nombre d'execution de la CI |
| 50 | Le texte de survol du plateau est caché sous le panneau latéral |
| 51 | Tests mk-ui : BoardInput et BoardMovement construits champ par champ à 23 endroits |
| 52 | Parcours J2 : la graine est à rechoisir à chaque nouveau tirage de la mise en place |
| 57 | Sorties de fmt, clippy et test trop longues pour les fils |
| 64 | Vérification à l'œil : lancer le jeu sur une partie préparée |
| 65 | Survol d'une case : « Il vous manque N points » alors que le déplacement est fini |
| 66 | Méthode : ne déléguer aux modèles moins chers que les étapes isolées |
| 76 | Numéros de copie des Blessures limités à 255 |
| 77 | Factoriser le joueur de test « premier choix » |
| 78 | verifier.sh : afficher tous les binaires de test en échec |
| 85 | Repos prolongé après un combat sans carte jouée : règle à trancher |
| 86 | Main vide sans carte jouée après un combat : aucune fin de tour possible |
| 87 | Seuil « assommé » figé à 5 : suivre la limite de main du niveau (spec 035) |
| 88 | Partager l'aide de test « gloire et niveau de départ » dans l'arène |
| 89 | Mettre en commun la mise en place du bac à sable avec ennemis dans les tests |
| 98 | Partie préparée déjà terminée pour vérifier la fenêtre de fin |
| 99 | Découper app.rs (≈ 1 500 lignes) |
| 105 | Noms français des cartes : actions avancées, sorts, artefacts, unités |
| 106 | Rendre l'étape rouge du TDD visible et vérifiable (bilan de la spec 050) |
| 110 | Tests : moins de comparaisons de la liste exacte des événements |
| 111 | Écran : déduire les cartes jouables des actions légales |
| 118 | Coder une grosse spec en deux fils : moteur, puis écran |
| 119 | Un cas de test par point de la section « Interface » des specs |
| 120 | Compter les tests rouges avec cargo test --no-fail-fast |
| 124 | Passer les autres libellés au pluriel « (s) » |
| 127 | Rédaction d'une spec : vérifier les clés de libellés qu'elle dit existantes |
| 129 | Gabarit de spec : vérifier que les libellés proposés sont possibles (clés, police) |
| 130 | Spec : vérifier que chaque élément d'écran se calcule depuis la vue du joueur |
| 131 | Parties préparées : vérifier qu'elles mènent à l'écran visé |
| 135 | Titre tronqué de la fenêtre « couleur du cristal » (spec 044) |
| 140 | Fenêtre Offres : titre propre au choix d'une action avancée de récompense (ruines) |
| 141 | Vue : exposer « action du tour prise » pour les boutons grisés |
| 142 | Méthode : vérifier l'accord des clés de libellé réutilisées avec paramètre |
| 145 | Rouge d'un parcours : le premier point rouge masque les suivants |
| 154 | Rédaction d'une spec : vérifier les cas « comme aujourd'hui » contre le code |
| 157 | Tests de mk-ui : comparer les boutons sans figer tous leurs champs |
| 158 | Rédaction d'une spec : vérifier ses exemples contre le contenu du jeu |
| 162 | Nouvelle partie à redesigner |
| 164 | ne pas autoriser de générer du mouvement inutilement après l'action |
| 165 | jetons ennemis révélés sur un site d'aventure, forteresse, tour de mage |
| 167 | héros vectorisés et illustrations compressées |
| 169 | boutons combats et interactions à mettre sur la carte |
| 170 | Script d'illustrations : reprendre automatiquement après un échec d'appel |
| 171 | Illustrations : garder les candidats d'un fil à l'autre |
| 175 | Vérifier qu'un exemple de spec est jouable dans sa situation |
| 176 | Écrire « un événement sans objet ne change rien » dans la recette « ajouter un événement » |
| 177 | Fenêtre de la défausse trop large et panneau du bas qui grandit avec le bandeau |
| 180 | affichage des images au survol souris |
| 187 | Audio d'ambiance : musique et bruitages |
| 190 | Spec : chercher tous les tests qui lisent un champ retiré avant de les lister |
| 195 | Permettre de lancer le jeu en anglais |
| 201 | cercles blancs au chargement des sites |
| 203 | Tests de mk-ui : une seule aide pour reprendre une partie depuis un journal d'événements |
| 204 | Écran de partie calculé deux fois par image (panneau de droite et plateau) |
| 212 | Écran de partie encore recalculé par le panneau du bas et la fenêtre de défausse |
| 215 | Spec 052 Moteur d'effets, ADR 0008 et plan (validés) |
| 217 | Fin du tour après retraite forcée : l'écran recalcule la vue au lieu de la demander à mk-core |
| 218 | Gabarit de plan : les étapes d'écran ont aussi un rouge, sur game_screen |
| 221 | Spec 053 Actions avancées : combat, unités et soins (validée) |
| 223 | Écran : un seul texte d'option (effet, modificateurs, texte écrit) au lieu de replis à chaque appel |
| 224 | Vérification à l'œil : la carte survolée monte, les clics sur la main ratent |
| 233 | Panneau de droite débordé quand la Source montre des dés pris et des dés épuisés |
| 234 | Texte de base de Rituel de sang trop long (six options énumérées) |
| 235 | Zone de jeu d'Effet maximum : ni la cible ni l'effet obtenu |
| 236 | Marge de title_min_width insuffisante : titre de fenêtre tronqué sous Xvfb malgré le test vert |
| 245 | Déluge de flammes : les boutons affichent les points de base |
| 246 | Sort lancé depuis l'offre : absent de la zone de jeu, et cas de combat à tester |
| 247 | Fenêtre Offres ouverte après « Lancer » avec un seul paiement |
| 256 | Publier le jeu dans un dépôt public (variante C) |
| 257 | Spec 057 : choisir l'arrivée d'un déplacement par effet sur le plateau |
| 258 | Spec 057 : cocher les ennemis d'Ailes de la nuit et les unités visées |
| 259 | Retirer GameScreen::discard, que l'écran ne dessine plus |
| 260 | Effets en cours : nommer la source par l'effet joué (Distorsion temporelle) |
| 272 | Épée de justice et Mages d'Altem : combinaisons du Compendium S6 non vérifiées |
| 275 | Partager le logger de capture des tests de logs |
| 276 | libellés mk-i18n manquants |
| 282 | Fin du tour : la case « Défausser » sort des cartes simples (bilan spec 077) |
| 284 | Vérifier à la rédaction qu'une clé de libellé « réutilisée » existe encore |
| 285 | Arène de test : choisir le héros d'une mise en place construite par événements |
| 292 | Plateau : le drapeau d'un site conquis est caché par la figurine du héros |
| 294 | verif-manuelle.sh charger : charger la partie sans clic à coordonnées en dur |
| 295 | Panneau du bas : largeur de la section Compétences calculée et testée dans layout.rs |
| 296 | Parties préparées : faux « Ce tour » et bandeaux de niveau au chargement |
| 302 | Section Compétences : débordement à quatre compétences, bouton de côté grisé sans valeur |
| 304 | Spec 059 Parcours du jalon 5 (implémentée) |
| 305 | optimisation de la CI |
| 307 | impossible de sortir d'une interaction sans annuler |
| 312 | magie pure : mana de l'effet renforcé mal affiché |
| 313 | ajout des scans des artefacts FR |
| 315 | Icône pixelisée dans la barre des tâches de Linux Mint (Cinnamon) |
| 317 | Compétences révélées avant la récompense sort de la fin du tour |
| 318 | Parcours de bout en bout : partir d'une partie préparée plutôt que d'une graine rare |
| 321 | Découper engine.rs (8 799 lignes) en modules du moteur |
| 322 | Parcours J3 : partir d'une mise en place arrangée plutôt que d'une graine rare |
| 331 | Découper engine/apply.rs et engine/events.rs par domaine |
| 332 | Relecture de spec : clés de libellés complètes et parties préparées atteignables |
| 337 | Découper event.rs par thème |
| 341 | Lister dans la fenêtre d'interaction toutes les unités recrutables du lieu, grisées si l'Influence manque |
| 342 | Outils de test des cités partagés dans l'arène |
| 343 | Découper decide et la table des faits du tour |
| 349 | Spec 069 Parcours du jalon 6 (implémentée) |
| 352 | Essai de faisabilité pendant la rédaction d'une spec de parcours |
| 357 | Gabarit de spec : relever les tests touchés par les appels, pas par les définitions |
| 358 | Partager la capture des logs des tests entre mk-ui et mk-persistance |
| 359 | verifier.sh --rouge : pouvoir se limiter à une crate ou un fichier de test |
| 364 | Vérif à l'œil : régler un curseur egui au clavier sous Xvfb |

## Pull requests

| N° privé | Titre |
|---|---|
| 53 | Réduit les minutes de CI (issue privée 48) |
| 62 | Spec 031 Déroulé d'un combat : spec validée et partie A (moteur) |
| 70 | Spec 032 Éléments et résistances : spec validée et implémentation |
| 72 | Spec 033 Capacités des ennemis : spec validée et implémentation |
| 104 | Spec 050 Offres et cartes gagnées |
| 108 | Spec 040 Interaction et réputation |
| 122 | Spec 041 Villages et monastères |
| 152 | Spec 070 Combat et offres (implémentée) |
| 156 | Spec 071 Icônes et raccourcis visibles |
| 161 | Spec 072 Illustrations générées |
| 174 | Spec 073 Panneau du bas et panneau de droite |
| 183 | Spec 074 Barre de menu |
| 193 | Spec 075 Plateau isométrique (validée) et plan de développement |
| 200 | Spec 075 Plateau isométrique, fil B : éléments debout |
| 216 | Implémente la spec 052 Moteur d'effets |
| 222 | Implémente la spec 053 Actions avancées : combat, unités et soins |
| 252 | Implémente la spec 057 Sorts : déplacement, unités et mana |
| 263 | Renomme le jeu en Iter Arcanum |
| 266 | Spec 055 Artefacts (implémentée) |
| 288 | Spec 051 Compétences : gain et premier lot |
| 298 | Spec 058 Compétences : second lot (implémentée) |
| 304 | Spec 059 Parcours du jalon 5 (implémentée) |
| 308 | Réécrit la ligne 082 du plan selon la variante C |
| 320 | Parcours : mise en place arrangée et épisodes posés (issue privée 318) |
| 329 | Spec 060 Tactiques |
| 336 | Spec 061 Joueur fantôme |
| 339 | Spec 062 Cités |
| 345 | Met les outils de test des cités dans l'arène commune (issue privée 342) |
| 347 | Spec 064 Conquête en solitaire |
| 349 | Spec 069 Parcours du jalon 6 (implémentée) |
| 351 | Spec 080 Préférences |
| 363 | Spec 081 Audio |
