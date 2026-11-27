# Index des règles par notion

Pour trouver vite les paragraphes du livret qui traitent d'une notion, sans lire les 24 pages. Les chemins `livret/`, `faq/` et
`cartes/` de cette page partent de `../mage-knight/docs/regles/`, dans le dépôt privé voisin. Complète le sommaire par page de `livret/README.md`. Les termes sont ceux de `docs/glossaire.md` ; le terme anglais du livret est entre parenthèses quand il aide à
chercher (`grep -rn "Forced Withdrawal" ../mage-knight/docs/regles/livret/`).

- Une référence `p. 7, 5.e` renvoie à `livret/p07.md`, paragraphe `5.`, sous-paragraphe `e.` ; la numérotation repart de 1 à chaque
  section (`###`), d'où le nom de la section quand une page en a plusieurs.
- La colonne **Spec** donne la spec qui a implémenté la notion, ou celle qui est prévue (en italique) : « 042 » pour une spec d'avant
  le 2026-11-24, « #N » pour une spec nommée d'après son issue (`docs/issues.md`).
  **À tenir à jour** dans la PR qui rédige ou implémente une spec : passer son numéro en romain une fois implémentée,
  et l'ajouter aux notions qu'elle couvre.
- Ce n'est qu'un index : la règle fait foi dans la page citée, jamais ici.
- Quand le livret ne tranche pas, ou qu'une règle semble en contredire une autre : la FAQ officielle (`faq/README.md`, tableau
  « Ce qui sert déjà »), puis le compendium de BGG via le tableau **Compendium** en fin de page (mode d'emploi :
  `faq/compendium/README.md`). Une contradiction avec le livret devient une question ouverte pour Mathieu.
- Le texte des cartes, jetons et cartes de site (hors livret) est transcrit dans `cartes/` : y chercher au lieu d'ouvrir les
  scans (`grep -rn "Mana Draw" ../mage-knight/docs/regles/cartes/`).

## Partie, manche, tour

| Notion | Où | Spec |
|---|---|---|
| Déroulé d'une partie (vue d'ensemble), choix du scénario et du héros | p. 1 *Game Flow* ; résumé p. 23 *Game* | [065](../specs/065-nouvelle-partie.md) (nouvelle partie), [078](../specs/078-nouvelle-partie-en-etapes.md) (écran en étapes, fiches) |
| Mise en place, zones communes (Source, offres, pioche de tuiles, plateau de jeu) | p. 3 *Game Setup* ; illustration p. 2 | 012, 020 |
| Zone du joueur (carte de héros, inventaire, jetons de niveau, deck, main de départ) | p. 3 *Player Area* | 012, 013 |
| Manche de Jour ou de Nuit, préparation d'une nouvelle manche | p. 4, 1 et 2.a à 2.g ; résumé p. 23 *One Round* | 023, [035](../specs/035-gloire-et-niveaux.md) (pioche jusqu'à la limite du niveau) |
| Choix des tactiques, ordre du tour | p. 4, 3 ; p. 19 *Solo Conquest* ; texte des tactiques : `cartes/tactiques.md` | [060](../specs/060-tactiques.md) (ordre du tour : [061](../specs/061-joueur-fantome.md)) |
| Ordre des tours, annonce de fin de manche, dernier tour | p. 4, 4 ; p. 6, 3 | 023 |
| Fin de partie (conditions du scénario, limite de manches) | p. 4, 4.c, 4.d et 5 | [063](../specs/063-premiere-reconnaissance.md) |
| Effets « avant votre tour » / « pendant le tour d'un autre joueur » | p. 6, 1 | — |
| Jeton d'ordre du tour retourné (tour sauté) | p. 6, 2 | hors périmètre (multijoueur) |
| Passer son tour (deck vide) | p. 6, 3 et 4 | 023 |
| Tour normal : déplacement puis une seule action | p. 6, 6 ; résumé p. 23 *One Turn* | 014 |
| Actions obligatoires (assaut, ennemi provoqué, autre joueur) | p. 6, 6.b | 030, [043](../specs/043-sites-fortifies.md) |
| Points de mouvement et d'influence perdus au début de l'action | p. 6, 6.e | [040](../specs/040-interaction-et-reputation.md) |
| Jouer ou défausser au moins une carte par tour | p. 6, 5.a | 016 |
| Repos (Standard Rest, Slow Recovery) | p. 6, 7 ; p. 11 *Wounds and Healing*, 1.b | 031 (repos prolongé), [034](../specs/034-blessures-et-soins.md) |
| Annoncer la fin de son tour, rendre les dés | p. 6, 8 | 016, 022 |
| Fin du tour : ordre des étapes | p. 11 *End of the Turn*, 1 à 7 ; résumé p. 23 | 016 |
| Retour en arrière (Reverting), points de non-retour | p. 5 *Reverting* 1, 2 et 2.a ; combat : p. 9 *Attack Phase*, 3.a | 024 |
| Organigramme du tour | p. 24 *Turn Sequence Overview* | — |

## Cartes et effets

| Notion | Où | Spec |
|---|---|---|
| Cartes : actions de base et avancées, sorts, artefacts, blessures | p. 4 *Deed Cards*, 1 à 3 | 013 |
| Effet de base, effet renforcé (payer la couleur) | p. 4 *Deed Cards*, 3.a à 3.c | 013, 022, [073](../specs/073-panneaux.md) (blocs séparés sur la carte), [#11](../specs/issue-11-cartes-en-main-epurees.md) (texte derrière « i », séparation « Renforcé » et son mana), [#32](../specs/issue-32-cartes-des-fenetres.md) (mêmes cartes dans les fenêtres Offres et de choix), [#20](../specs/issue-20-obtention-des-cartes.md) (fenêtres de choix, infobulle des achats) |
| Jouer une carte de côté (Mouvement, Influence, Attaque, Parade 1) | p. 4 *Deed Cards*, 3.e ; p. 7, 2.c ; p. 8, 2.d ; p. 9 *Block Phase*, 4.b ; p. 10 *Attack Phase*, 1.b || 013, [071](../specs/071-icones-et-raccourcis.md) (boutons-icônes) |
| Cumul des effets semblables | p. 4 *Deed Cards*, 4 | 013 |
| Coût d'une carte : payer une mana, défausser ou écarter une autre carte | p. 4 *Deed Cards*, 6 | 013 |
| Durée d'un effet (jusqu'à la fin du tour) | p. 5 *Effects*, 1.b | [052](../specs/052-moteur-d-effets.md) |
| Effets composites (Concentration, Magic Talent…) | p. 5 *"Composite" Effects* | 025, [052](../specs/052-moteur-d-effets.md), [056](../specs/056-actions-avancees-mana.md) (Effet maximum), 092 (Effet maximum complet, à rédiger) |
| Effets « gagner » (jeton de mana, cristal, carte) | p. 5 *"Gain" Effects* | 021, 025, [050](../specs/050-offres-et-cartes-gagnees.md), [056](../specs/056-actions-avancees-mana.md) (carte gagnée dans la défausse ou la main) |
| Défausser, détruire (throw away) | p. 5 *Discarding and Throwing Away* | 013, [073](../specs/073-panneaux.md) (défausse consultable), [056](../specs/056-actions-avancees-mana.md) (détruire une carte action) |
| Défausse à la fin du tour et repioche jusqu'à la limite de main | p. 11 *End of the Turn*, 3 et 7 | 016, [035](../specs/035-gloire-et-niveaux.md) (limite du niveau) |
| Limite de main augmentée (forteresse, cité, tactique Planning) | p. 11 *End of the Turn*, 7.b ; p. 4, 2.g et 4.f | [043](../specs/043-sites-fortifies.md), [057](../specs/057-sorts-deplacement-unites-mana.md) (Méditation), [060](../specs/060-tactiques.md), [062](../specs/062-cites.md) |
| Pas de remélange quand le deck est vide (sauf Long Night) | p. 11 *End of the Turn*, 7.d | 016 |
| Contenu des cartes de départ | `decks-de-depart.md` (hors livret), `content/heroes.ron` | 013, 029 |
| Texte des cartes d'action de base (communes et propres aux héros) | `cartes/actions-de-base.md` (hors livret) | 013, 022 |
| Texte des actions avancées, sorts, artefacts | `cartes/actions-avancees.md`, `cartes/sorts.md`, `cartes/artefacts.md` (hors livret) | [050](../specs/050-offres-et-cartes-gagnees.md) (en données, jouables de côté), [052](../specs/052-moteur-d-effets.md), [053](../specs/053-actions-avancees.md), [056](../specs/056-actions-avancees-mana.md), [054](../specs/054-sorts.md) (sorts de combat), [057](../specs/057-sorts-deplacement-unites-mana.md) (autres sorts), [055](../specs/055-artefacts.md) (artefacts) |

## Mana

| Notion | Où | Spec |
|---|---|---|
| Couleurs de mana (base, or, noir), Jour et Nuit | p. 5 *Using Mana*, 1 et 2 | 020 |
| Mana fondamental (dé, jeton de mana) et cristaux | p. 5 *Using Mana*, 1.a et 1.b | 021, 022 |
| Source : lancer, au moins la moitié de couleurs de base | p. 3 *Day/Night board* ; p. 4, 2.b | 020, [#10](../specs/issue-10-source-barre-du-haut.md) (Source dans la barre de menu) |
| Prendre un dé de la Source (un par tour), le relancer | p. 5 *Using Mana*, 3.a et 3.b ; p. 6, 8.a ; p. 11, 1 | 022, 025 (dés de plus, dés mis sur une couleur), [056](../specs/056-actions-avancees-mana.md) (Tempête de mana) |
| Dés épuisés (noir de jour, or de nuit) | p. 5 *Using Mana*, 3.c | 020 |
| Inventaire : 3 cristaux par couleur au plus | p. 5 *Using Mana*, 1.b ; p. 5 *"Gain" Effects*, 2 | 021 |
| Cristal transformé en mana fondamental | p. 5 *Using Mana*, 1.b | 022 |
| Jetons de mana rendus à la réserve en fin de tour | p. 11 *End of the Turn*, 3.a | 025 |
| Cristal gagné sur une mine | p. 11 *End of the Turn*, 4.b | 021 |
| Sorts : effet de base et effet renforcé de nuit (mana noir) | p. 4 *Deed Cards*, 3.b | [054](../specs/054-sorts.md) |

## Plateau de jeu, déplacement, exploration

| Notion | Où | Spec |
|---|---|---|
| Formes du plateau de jeu (en coin, ouverte, n colonnes), côte | p. 14 *Map Shape* (image `livret/images/p14-formes-de-carte.jpg`) | 012, [075](../specs/075-plateau-isometrique.md) (mer au-delà de la côte, implémentée) |
| Pioche de tuiles : campagne au-dessus, royaume dessous, cités | p. 3 *Tile deck* | 012, [074](../specs/074-barre-de-menu.md) (nombres dans la barre de menu) |
| Contenu des tuiles (terrains, sites imprimés) | `tuiles.md` (hors livret), `content/tiles.ron` | 010, 029 |
| Coordonnées des cases et emplacements du bac à sable (portail, villages, Lac, mines, forteresse…) | `bac-a-sable.md` (hors livret, généré et vérifié par un test) | 012, 015 |
| Coûts de déplacement, terrains infranchissables, Jour et Nuit | p. 7, 3 ; plateau Jour/Nuit (scans `misc/daynight_*.jpg`) | 014, 036 (case par case) |
| Limites : site fortifié, ennemi errant, sites d'aventure | p. 7, 4 | 014, 030, [043](../specs/043-sites-fortifies.md) |
| Provoquer un ennemi errant | p. 7, 4.c ; p. 8 *Combat*, 1.e | 030, [057](../specs/057-sorts-deplacement-unites-mana.md) (déplacements qui ne provoquent pas) |
| Explorer : emplacements, coût 2, orientation, campagne et royaume | p. 7, 5.a à 5.e | 015 |
| Pioche de tuiles vide : tuiles écartées | p. 7, 5.e | 015, [074](../specs/074-barre-de-menu.md) |
| Tuile révélée : sites, monastère, cité | p. 7, 5.f et 5.g | [041](../specs/041-villages-et-monasteres.md) (monastère révélé), [062](../specs/062-cites.md) |
| Alterner déplacement et exploration, jouer des cartes en cours de route | p. 7, 6 | 014, 015, 017, [036](../specs/036-trajet-en-un-clic.md) (trajet en un clic) |
| Effets qui modifient le déplacement (réduction de coût, téléportation) | p. 7, 7.a à 7.c | [052](../specs/052-moteur-d-effets.md), [057](../specs/057-sorts-deplacement-unites-mana.md) (déplacement par un effet, cases à 2 comme voisines) |
| Case sécurisée (safe space) | p. 7, 7.d | [052](../specs/052-moteur-d-effets.md) (specs 034 et 043, hors périmètre) |
| Retraite forcée (Forced Withdrawal) | p. 11 *End of the Turn*, 2 ; p. 7, 8.e ; p. 10 *Combat Outcomes*, 3.d | [052](../specs/052-moteur-d-effets.md) (specs 034 et 043, hors périmètre) |
| Autres joueurs sur le plateau, portail, cité conquise | p. 7, 8 | hors périmètre (multijoueur) |
| Gloire en révélant une tuile (Première reconnaissance) | p. 18 *First Reconnaissance*, Special Rules | [063](../specs/063-premiere-reconnaissance.md) |

## Combat contre les ennemis

| Notion | Où | Spec |
|---|---|---|
| Déclencher un combat (assaut, site d'aventure, monastère, ennemi errant) ; défier | p. 8 *Combat with Enemies*, 1 ; p. 6, 6.c | 030, 031, [043](../specs/043-sites-fortifies.md), [044](../specs/044-sites-d-aventure.md) |
| Un seul combat par tour, ennemis de plusieurs cases | p. 8 *Combat with Enemies*, 2 | 030, 031, [043](../specs/043-sites-fortifies.md) |
| Les quatre phases | p. 8 *Combat with Enemies*, 3 et 4 ; résumé p. 23 *Combat with Enemies* | 031, [070](../specs/070-combat-et-offres.md) (écran de combat) |
| Effets rouges ou bleus contre les résistances au Feu ou à la Glace | p. 8 *Combat with Enemies*, 5 ; FAQ p3 ; Compendium, *Combat* Q7, S33 | [053](../specs/053-actions-avancees.md) (règle générale), [054](../specs/054-sorts.md) (résistances perdues, détruire), [055](../specs/055-artefacts.md) |
| Pas de soin pendant un combat | p. 8 *Combat with Enemies*, 6 ; p. 11 *Wounds and Healing*, 3.d | [034](../specs/034-blessures-et-soins.md) |
| Phase d'attaque à distance et de siège, ennemis fortifiés | p. 9 *Ranged and Siege Attack Phase* | 031, 032 (siège), [043](../specs/043-sites-fortifies.md), [070](../specs/070-combat-et-offres.md) (cible seule cochée d'office, attaque insuffisante annulée) |
| Attaque inefficace (moitié) contre une résistance | p. 9 *Ranged and Siege Attack Phase*, 4 | 032 |
| Résistances des ennemis (physique, feu, glace, les deux), partagées par le groupe ciblé | p. 24 *Defensive* ; p. 9 *Ranged and Siege Attack Phase*, 4.a et 7.b ; Compendium, *Combat* S12 || 032, [071](../specs/071-icones-et-raccourcis.md) (icônes) |
| Éléments des attaques et des parades (physique, feu, glace, feu glacé) | p. 24 *Offensive* ; p. 9 *Block Phase*, 4 et 5 | 032 |
| Ennemi vaincu, gloire gagnée | p. 9 *Ranged and Siege Attack Phase*, 5 | 031, [035](../specs/035-gloire-et-niveaux.md) |
| Phase de parade, parade efficace ou inefficace selon l'élément | p. 9 *Block Phase*, 1 à 10 | 031, 032, [070](../specs/070-combat-et-offres.md) (parade insuffisante annulée) |
| Phase d'assignation des dégâts : unités, héros, armure | p. 9 *Assign Damage Phase*, 1 à 5 ; p. 10, 6 et 7 | 031 (héros), 033 (Brutal, Toxique, Paralysant), [034](../specs/034-blessures-et-soins.md) (assommé), [035](../specs/035-gloire-et-niveaux.md) (armure du niveau), [042](../specs/042-unites.md) (unités) |
| Assommé (Knock out) | p. 9 *Assign Damage Phase*, 5.c ; FAQ p2 ; Compendium, *Combat* S20, Q22, S45 | [034](../specs/034-blessures-et-soins.md), [035](../specs/035-gloire-et-niveaux.md) (seuil : limite de main du niveau) |
| Phase d'attaque | p. 10 *Attack Phase* | 031 |
| Issue du combat (sites, ennemis non vaincus, retraite) | p. 10 *Combat Outcomes* | 031, [043](../specs/043-sites-fortifies.md), [044](../specs/044-sites-d-aventure.md) |
| Assaut d'une cité, bonus des défenseurs, Seigneur de la cité | p. 10 *City Assaults* | [062](../specs/062-cites.md) |
| Récompenses de combat (cristaux, artefacts, sorts, unités) | p. 11 *End of the Turn*, 5 | [050](../specs/050-offres-et-cartes-gagnees.md) (gagner une carte), [041](../specs/041-villages-et-monasteres.md) (artefact du monastère incendié), [043](../specs/043-sites-fortifies.md), [044](../specs/044-sites-d-aventure.md), [045](../specs/045-ruines.md) (ruines, unité en récompense), [#20](../specs/issue-20-obtention-des-cartes.md) (fenêtres de choix) |
| Capacités des ennemis : Vif, Brutal, Toxique, Paralysant, Invocation… | p. 24 *Enemy Token Abilities* (image `livret/images/p24-icones-ennemis.jpg`) ; valeurs de chaque jeton : `cartes/ennemis.md` ; p. 9 *Block Phase*, 2 et 6.a ; p. 10, 6 ; Compendium, *Combat* S20, Q22, S23, *ORC SUMMONERS* S1 | 033, [#27](../specs/issue-27-icones-capacites-ennemis.md) (icônes, règle en infobulle) |
| Fortifié, doublement fortifié, non fortifié | p. 9 *Ranged and Siege Attack Phase*, 3.a ; p. 24 *Defensive* | 031 (jeton), 032 (siège), [043](../specs/043-sites-fortifies.md) ; non fortifié : extensions, hors périmètre |
| Attaques multiples d'un ennemi | p. 24 *Multiple Attacks* | aucun ennemi du jeu de base (extensions, hors périmètre ; spec 033) |
| Jetons d'ennemis : piles par couleur, défausse, remélange, pile de secours | p. 3 *Enemy and Ruin token piles* ; valeurs : `cartes/ennemis.md` ; Compendium, *Scenarios* S5 | 030, 031 (défausse, remélange), 033 (pile de secours), [#14](../specs/issue-14-ennemis-par-categorie.md) (piles nommées par catégorie, socle coloré des figurines) |

## Blessures, soins, progression

| Notion | Où | Spec |
|---|---|---|
| Blessures : en main, jamais jouées ni défaussées sauf exception | p. 11 *Wounds and Healing*, 1 ; p. 4 *Deed Cards*, 3.d et 6.a | 031, [034](../specs/034-blessures-et-soins.md) |
| Unités blessées | p. 11 *Wounds and Healing*, 2 ; p. 5 *Using Units*, 3.c | [042](../specs/042-unites.md) |
| Soins (points de soin, coût par unité, villages et monastères) | p. 11 *Wounds and Healing*, 3 ; p. 8 *Interacting*, 4.b | [034](../specs/034-blessures-et-soins.md) (Soin X), [041](../specs/041-villages-et-monasteres.md) (soins achetés), [042](../specs/042-unites.md) (réserve de points de soin, unités) |
| Clairière magique | p. 11 *End of the Turn*, 4.a ; carte de site : `cartes/sites-et-cites.md` | [044](../specs/044-sites-d-aventure.md) |
| Ruines : jetons de ruines, autel, ennemis, récompenses | p. 3 *Enemy and Ruin token piles* ; p. 10 *Combat Outcomes* 2.b, 3.c ; carte *Ancient Ruins* : `cartes/sites-et-cites.md` ; jetons : `cartes/ennemis.md` ; Compendium, *Miscellaneous* S6, S11, S16, *Movement* S8, *End of Round* S6 | [045](../specs/045-ruines.md) |
| Gloire, piste de gloire, changement de niveau | p. 9 *Ranged and Siege Attack Phase*, 5.b ; p. 11 *End of the Turn*, 6 ; scan `misc/fame_board.jpg` ; Compendium, *End of Turn* S6, S7, *Miscellaneous* S8 | [035](../specs/035-gloire-et-niveaux.md) (compétences des niveaux pairs : [051](../specs/051-competences.md)) |
| Niveau : armure, limite de main, jeton de commandement | p. 3 *Player Area* ; p. 11 *End of the Turn*, 6.a ; scans `misc/level_token_*.png` | [035](../specs/035-gloire-et-niveaux.md) |
| Compétences : types, gain aux niveaux pairs, offre commune | p. 3 *Pile of Skill tokens* ; p. 4, 2.g ; p. 5 *Using Skills* ; p. 11 *End of the Turn*, 6.b ; texte : `cartes/heros-et-competences.md` ; FAQ p2, p3 ; Compendium, *Skill Tokens* | [051](../specs/051-competences.md) (mécanisme, premier lot), [058](../specs/058-competences-second-lot.md) (second lot), [#20](../specs/issue-20-obtention-des-cartes.md) (fenêtre du niveau pair) |
| Compétences en solo (coopérative gardée, celles du joueur fantôme) | p. 15 *Skills in a Solo Game* ; Compendium, *Skill Tokens* S5 | [051](../specs/051-competences.md), [061](../specs/061-joueur-fantome.md) |

## Unités, interaction, sites

| Notion | Où | Spec |
|---|---|---|
| Unités : disponibles, épuisées, blessées, activation, renvoyer | p. 5 *Using Units* ; texte des cartes : `cartes/unites.md` | [042](../specs/042-unites.md) |
| Bannières | p. 5 *Using Units*, 8 | [055](../specs/055-artefacts.md) ; Bannière de terreur contre les Invocateurs : issue #8 |
| Offre d'unités : composition, renouvellement, élites après une tuile royaume | p. 3 *Unit offer* ; p. 4, 2.c | [050](../specs/050-offres-et-cartes-gagnees.md) |
| Interaction : influence, réputation (bonus ou malus, case X) | p. 8 *Interacting with Locals*, 1 à 3 et 5 | [040](../specs/040-interaction-et-reputation.md) |
| Recruter une unité (type de site), acheter des soins, des actions avancées, des sorts | p. 8 *Interacting with Locals*, 4.a à 4.d | [041](../specs/041-villages-et-monasteres.md) (soins, actions avancées du monastère), [042](../specs/042-unites.md) (recruter au village et au monastère) |
| Interaction dans une cité, par couleur de cité | p. 8 *Interacting with Locals*, 3.a et 4.e | [062](../specs/062-cites.md) |
| Réputation perdue (assaut, monastère brûlé) ou gagnée (ennemis errants) | p. 8 *Combat with Enemies*, 1.a et 1.c ; p. 10 *Combat Outcomes*, 2.a | [040](../specs/040-interaction-et-reputation.md) (ennemis errants, Menace, Nobles manières), [041](../specs/041-villages-et-monasteres.md) (monastère incendié), [043](../specs/043-sites-fortifies.md) (assaut) |
| Offres d'actions avancées et de sorts : renouvellement | p. 4, 2.d et 2.e ; p. 5 *"Gain" Effects*, 3 | [050](../specs/050-offres-et-cartes-gagnees.md) |
| Sites (règles détaillées de chaque site) | Cartes de description, hors livret : `cartes/sites-et-cites.md` | [041](../specs/041-villages-et-monasteres.md) (village, monastère), [043](../specs/043-sites-fortifies.md) à [045](../specs/045-ruines.md) |
| Cités (niveaux, cartes de cité) | p. 18 *City Levels* ; défenseurs par niveau : `cartes/sites-et-cites.md` | [062](../specs/062-cites.md) |

## Solo et scénarios

| Notion | Où | Spec |
|---|---|---|
| Joueur fantôme : mise en place, tour, nouvelle manche | p. 15 *Dummy Player* ; p. 4, 3.c (ordre du tour) ; p. 6, 3 ; Compendium, *Dummy Player* S1 à S3 | [061](../specs/061-joueur-fantome.md) |
| Décompte standard des titres | p. 15 *Standard Achievements Scoring* ; carte de score : `cartes/sites-et-cites.md` | [063](../specs/063-premiere-reconnaissance.md) (sans les bonus de titre), [041](../specs/041-villages-et-monasteres.md) (artefacts, Boucliers des monastères) |
| Première reconnaissance et sa variante solo | p. 18 *First Reconnaissance* | [063](../specs/063-premiere-reconnaissance.md) (sans joueur fantôme), [061](../specs/061-joueur-fantome.md) |
| Conquête en solitaire (scénario solo de référence, tactiques du joueur fantôme) | p. 19 *Solo Conquest* ; p. 14 *Wedge Map* (pas de tuile royaume sur la côte) ; Compendium, *Scenarios* O30, O31 ; *Final Scoring* S3 | [064](../specs/064-conquete-en-solitaire.md) |
| Autres scénarios | p. 19 à 22 | hors périmètre |
| Variantes (enchères, plateau, Jour et Nuit, partie amicale, Rampage, combat interactif, deckbuilding) | p. 16 et 17 | hors périmètre |
| Combat entre joueurs, assaut coopératif de cité, équipes | p. 12 et 13 ; p. 15 *Team Rules* | hors périmètre |

## Compendium (FAQ non officielle de BGG)

Section du compendium à lire pour chaque groupe de notions ci-dessus. Fichiers dans `faq/compendium/` ; une carte, une compétence,
une unité ou un ennemi se cherche par son nom anglais dans `faq/compendium/index.md`. Les pages de livret citées par le compendium
sont celles des livrets d'origine, pas de MKUE.

| Notions | Sections du compendium |
|---|---|
| Mise en place, tuiles de départ | `01-setup.md` |
| Joueur fantôme, solo | `02-dummy-player.md` ; `21-scenarios.md` ; `32-tactic-cards.md` (tactiques) |
| Début du tour, effets entre les tours | `03-turn-start.md` ; `19-miscellaneous.md` |
| Mana : Source, dés, jetons, cristaux, jour et nuit | `04-mana.md` |
| Déplacement, ennemis provoqués, safe space | `05-movement.md` |
| Exploration, pose des tuiles | `06-map-revealing.md` |
| Combat : phases, résistances, fortifications, capacités des ennemis, dégâts, assauts | `07-combat.md` ; `09-arcane-immunity.md` ; `33-enemy-tokens.md` ; sujets de `35-index-of-topics.md` (Brutal, Elusive, Swiftness…) |
| Cités, assaut de cité | `10-cities.md` |
| Interaction, influence, réputation | `11-influence.md` |
| Soins, blessures, repos, tour minimum | `12-healing.md` ; `13-resting.md` ; `14-minimum-turn.md` |
| Fin du tour, retraite forcée, fin de manche | `15-end-of-turn.md` ; `16-forced-withdrawal.md` ; `17-end-of-round.md` |
| Décompte final | `18-final-scoring.md` |
| Cartes d'action, sorts, artefacts | `23-basic-action-cards.md` ; `24-advanced-action-cards.md` ; `25-dual-colored-action-cards.md` ; `26-spell-cards.md` ; `27-artifact-cards.md` |
| Compétences, niveaux | `28-skill-tokens.md` |
| Unités, bannières | `30-silver-units.md` ; `31-gold-units.md` ; `27-artifact-cards.md` (bannières) |
| Variantes, scénarios, Volkare, combat entre joueurs | `20-variants.md` ; `21-scenarios.md` ; `22-volkare.md` ; `08-pvp.md` ; `29-faction-tokens.md` |
| Erreurs de règles fréquentes, éditions | `34-general-questions.md` |
