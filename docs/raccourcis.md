# Raccourcis clavier

Registre des touches utilisées par le jeu, tenu à jour par chaque spec qui en ajoute (sous-section « Raccourcis clavier » du gabarit).
Avant d'attribuer une touche, vérifier ici qu'elle n'est pas déjà prise dans le même contexte.

Chaque bouton qui a une touche la montre en infobulle, « Raccourci : … » (spec 071) ; un élément du Menu l'écrit en face de son
libellé (spec 074). Ctrl+Z, Ctrl+S, Ctrl+O et Ctrl+N sont aussi dans le Menu.

Sous macOS, Ctrl est Cmd : la touche à presser et son libellé (« Cmd+Z ») ; le reste du tableau écrit « Ctrl » (spec #7).

Les touches n'agissent jamais quand un champ de saisie a le focus, sauf Entrée dans le champ de la graine (spec 065).

| Touche | Effet | Contexte | Spec |
|---|---|---|---|
| Flèches | Faire défiler le plateau de jeu, sans que son centre sorte des tuiles posées | Partie | [011](specs/011-affichage-plateau.md), [075](specs/075-plateau-isometrique.md) |
| `+` / `-` | Zoomer / dézoomer le plateau de jeu (dézoom arrêté à 0,5 × le cadrage de Recentrer, zoom maximal 120) | Partie | [011](specs/011-affichage-plateau.md), [075](specs/075-plateau-isometrique.md) |
| Début | Recentrer le plateau de jeu (aussi dans le Menu) | Partie | [011](specs/011-affichage-plateau.md), [074](specs/074-barre-de-menu.md) |
| Échap | Annuler le choix de la carte à défausser | Partie, en mode choix | [013](specs/013-jouer-des-cartes.md) |
| Échap | Annuler le choix du mana (effet renforcé, ou option payante comme Cristallisation), des dés à mettre sur une couleur, ou de la carte à renforcer avec Concentration | Partie, en mode choix du mana, des dés ou Concentration | [022](specs/022-effets-renforces.md), [025](specs/025-cartes-de-mana.md) |
| Échap | Annuler le choix de l'arrivée d'un déplacement par un effet (Voyage souterrain, Ailes du vent) | Partie, en mode choix de l'arrivée | [057](specs/057-sorts-deplacement-unites-mana.md) |
| Échap | Annuler le choix d'un artefact (cartes de l'Épée de justice, ennemis de la Bannière de terreur, bonus du Cor de colère, couleurs des amulettes) ou de l'unité qui reçoit une bannière | Partie, en mode choix d'un artefact ou d'une unité | [055](specs/055-artefacts.md) |
| Fin | Ouvrir le mode fin du tour (main vide : finir le tour directement) | Partie, si finir le tour est possible, hors mode choix | [016](specs/016-fin-du-tour.md) |
| Entrée | Finir le tour avec les cartes cochées | Partie, en mode fin du tour, si « Finir le tour » est actif | [016](specs/016-fin-du-tour.md) |
| Entrée | Valider l'attaque ou la parade en cours | Partie, en combat, avec une attaque ou une parade en cours, hors mode choix ; sans effet si elle ne suffit pas (bouton « Annuler ») | [031](specs/031-deroule-combat.md), [070](specs/070-combat-et-offres.md) |
| Échap | Annuler l'attaque ou la parade en cours (retour en arrière jusqu'avant « Attaquer » ou « Parer ») | Partie, en combat, attaque ou parade insuffisante (bouton « Annuler »), hors mode choix | [070](specs/070-combat-et-offres.md) |
| Échap | Quitter le mode fin du tour sans rien faire | Partie, en mode fin du tour | [016](specs/016-fin-du-tour.md) |
| Échap | Annuler l'annonce de la fin de la manche | Partie, en confirmation de l'annonce | [023](specs/023-fin-de-manche.md) |
| Ctrl+Z | Revenir en arrière | Partie, hors mode choix, si c'est possible | [024](specs/024-sauvegarde-et-retour-en-arriere.md) |
| Ctrl+S | Sauvegarder la partie | Partie, hors mode choix | [024](specs/024-sauvegarde-et-retour-en-arriere.md) |
| Ctrl+O | Ouvrir la fenêtre de chargement | Partie, hors mode choix ; au lancement, comme « Charger… » | [024](specs/024-sauvegarde-et-retour-en-arriere.md), [065](specs/065-nouvelle-partie.md) |
| Échap | Fermer la fenêtre de chargement | Fenêtre de chargement ouverte | [024](specs/024-sauvegarde-et-retour-en-arriere.md) |
| Échap | Fermer la fenêtre de fin (score final) | Partie terminée, fenêtre de fin ouverte | [063](specs/063-premiere-reconnaissance.md) |
| Ctrl+N | Ouvrir l'écran de nouvelle partie, à l'étape Scénario | Partie (terminée comprise), hors mode choix | [065](specs/065-nouvelle-partie.md), [078](specs/078-nouvelle-partie-en-etapes.md) |
| Entrée | Commencer la partie (jamais « Suivant ») | Écran de nouvelle partie ouvert, sur les deux étapes, graine valide, y compris depuis le champ de la graine | [065](specs/065-nouvelle-partie.md), [078](specs/078-nouvelle-partie-en-etapes.md) |
| ↑ / ↓ | Cocher le scénario ou le héros précédent ou suivant de la liste, sans boucler | Écran de nouvelle partie ouvert, hors champ de saisie | [078](specs/078-nouvelle-partie-en-etapes.md) |
| ← / → | Passer à l'étape d'avant ou d'après : Scénario, Héros, Options | Écran de nouvelle partie ouvert, hors champ de saisie | [078](specs/078-nouvelle-partie-en-etapes.md), [061](specs/061-joueur-fantome.md) |
| Échap | Fermer la fenêtre de nouvelle partie | Partie en cours, fenêtre de nouvelle partie ouverte (sans effet au lancement) | [065](specs/065-nouvelle-partie.md) |
| O | Ouvrir ou fermer la fenêtre Offres | Partie, hors mode choix et hors choix d'action avancée | [050](specs/050-offres-et-cartes-gagnees.md) |
| Échap | Fermer la fenêtre Offres | Fenêtre Offres ouverte, hors choix d'action avancée | [050](specs/050-offres-et-cartes-gagnees.md) |
| D | Ouvrir ou fermer la fenêtre de la défausse | Partie, hors mode choix, défausse non vide | [073](specs/073-panneaux.md) |
| Échap | Fermer la fenêtre de la défausse (avant la fenêtre Offres) | Fenêtre de la défausse ouverte | [073](specs/073-panneaux.md) |
| 1, 2, 3 | Prendre l'action avancée en 1re, 2e, 3e place | Choix d'action avancée en attente | [050](specs/050-offres-et-cartes-gagnees.md) |
| 1 à 6 | Encadrer la tactique de ce numéro (sans effet sur une place vide) ; les autres touches de jeu n'agissent pas | Fenêtre du choix de la tactique ouverte | [060](specs/060-tactiques.md) |
| Entrée | Prendre la tactique encadrée | Fenêtre du choix de la tactique ouverte, une tactique encadrée | [060](specs/060-tactiques.md) |
| Échap | Annuler le choix des dés d'Appel du mana ou des cartes de Méditation nocturne | Partie, en mode choix des dés ou des cartes d'une tactique | [060](specs/060-tactiques.md) |
| I | Interagir avec les habitants | Partie, quand le bouton est actif, hors mode choix | [040](specs/040-interaction-et-reputation.md) |
| 1, 2, 3 | Prendre le 1er, le 2e, le 3e artefact tiré (trois pour deux artefacts gagnés aux ruines, spec 045) | Choix d'artefact en attente (avant celui d'une action avancée) | [041](specs/041-villages-et-monasteres.md), [045](specs/045-ruines.md) |
| Échap | Annuler l'incendie du monastère | Partie, en confirmation de l'incendie | [041](specs/041-villages-et-monasteres.md) |
| 1, 2, 3 | Prendre le 1er, 2e, 3e sort de l'offre | Choix du sort de la récompense (après celui d'un artefact, avant celui d'une action avancée) | [043](specs/043-sites-fortifies.md) |
| Échap | Annuler le choix du mana d'un sort acheté | Partie, en choix du mana de l'achat | [043](specs/043-sites-fortifies.md) |
| E | Entrer dans le site d'aventure (ouvre la confirmation) | Partie, quand le bouton est actif, hors mode choix | [044](specs/044-sites-d-aventure.md) |
| Échap | Annuler l'entrée dans le site d'aventure | Partie, en confirmation de l'entrée | [044](specs/044-sites-d-aventure.md) |
| 1, 2, 3, 4 | Choisir rouge, bleu, blanc, vert pour le cristal | Choix de la couleur du cristal (avant celui d'un artefact) | [044](specs/044-sites-d-aventure.md) |
| E | Faire une offrande à l'autel (ouvre la fenêtre des paiements), comme « Entrer dans » | Partie, sur un autel, bouton actif, hors mode choix | [045](specs/045-ruines.md) |
| 1 à 9 | Choisir le paiement de l'offrande | Fenêtre de l'offrande | [045](specs/045-ruines.md) |
| Échap | Fermer la fenêtre de l'offrande | Fenêtre de l'offrande | [045](specs/045-ruines.md) |
| 1, 2, 3 | Prendre la 1re, 2e, 3e unité de l'offre (sans jeton libre : choisir l'unité à renvoyer) | Choix de l'unité de récompense (après tous les autres choix de la fin du tour) | [045](specs/045-ruines.md) |
| Échap | Revenir du choix de l'unité à renvoyer au choix de l'unité de récompense | Choix de l'unité à renvoyer pour une récompense | [045](specs/045-ruines.md) |
| Ctrl+Q | Quitter le jeu (confirmation si le tour a avancé depuis la sauvegarde automatique) | Toujours, lancement compris | [074](specs/074-barre-de-menu.md) |
| Entrée | Quitter sans sauvegarder | Confirmation de Quitter ouverte | [074](specs/074-barre-de-menu.md) |
| Échap | Annuler, rester dans la partie | Confirmation de Quitter ouverte | [074](specs/074-barre-de-menu.md) |
| F11 | Passer en plein écran ou en sortir | Toujours, lancement compris | [074](specs/074-barre-de-menu.md) |
| Ctrl+, | Ouvrir ou fermer la fenêtre des préférences | Toujours, lancement compris | [080](specs/080-preferences.md) |
| Échap | Fermer la fenêtre des préférences (avant toute autre fenêtre) | Fenêtre des préférences ouverte | [080](specs/080-preferences.md) |
