# 078 — Nouvelle partie en étapes

- Statut : implémentée (validée le 2026-11-13, Q1 à Q5 tranchées par Mathieu)
- Jalon : IHM (après 077)
- Dépend de : 065 (écran de nouvelle partie), 074 (barre de menu), 076 (libellés fr et en) ; toutes implémentées
- Livret : MKUE p. 1 (*Game Flow*, 1 et 2), p. 18 (*First Reconnaissance*, dont *Solo Variant*), p. 3 (decks de départ) ;
  walkthrough p. 20 (`docs/regles/MK_walkthrough_EN.pdf`, *Mage Knights* : source des descriptions des héros, réécrites)
- Crates touchées : arcanum-ui, `assets/i18n/` (fr.json et en.json) ; arcanum-core seulement pour retirer `ScenarioId::summary_key`
- Origine : issue privée 162 (décisions du 2026-11-12 en commentaire) ; maquette validée `/mnt/project-files/ihm/078/maquette/`
  (artefact https://claude.ai/artifact/ASCTBfFUPz394F8d6JcfxZ)

## Objectif

L'écran de nouvelle partie de la spec 065 (une petite fenêtre avec des boutons radio) devient un **assistant plein écran à
deux étapes**, Scénario puis Héros. Chaque étape montre la liste à gauche et une **grande fiche** à droite : le joueur lit ce
que propose le scénario (objectif, règles spéciales, fin de partie, décompte) et ce que fait le héros (description, style de
jeu, ses deux cartes propres avec leur effet) avant de choisir. « Commencer » reste possible dès la première étape, la graine
passe dans un volet « Avancé », et Ctrl+N ouvre le même écran en cours de partie. Rien ne change aux règles ni à ce que
« Commencer » lance.

## Règles

- **Choisir un scénario, puis un héros** (MKUE p. 1, *Game Flow*, 1 et 2) : l'ordre des étapes suit le livret. Les
  conséquences de « Commencer » (graine, sauvegarde de la partie abandonnée, copie de la sauvegarde automatique) sont celles
  de la spec 065, inchangées.
- **Fiche de Première reconnaissance** (MKUE p. 18) : elle résume ce que fait le jeu, c'est-à-dire la variante solo de la
  spec 063 (4 manches, sans joueur fantôme, 1 gloire par tuile, dernier tour après la cité, score sans titres). Ce que la
  variante solo du livret demande et que le jeu n'a pas encore (joueur fantôme et choix des tactiques, specs 061 et 060) est
  dit dans une ligne « Pas encore dans le jeu », **sans renvoi au livret** (issue privée 162). Le texte d'ambiance est **original** :
  il reprend l'idée de la mission du livret (trouver la capitale d'une contrée inconnue) sans en traduire la phrase
  (CLAUDE.md, *Assets et droits* ; la maquette en suivait le livret de trop près).
- **Fiche du bac à sable** : scénario d'essai propre au jeu (spec 012), absent du livret.
- **Fiche du héros** : la description de chaque héros est un **texte original** écrit d'après l'histoire du walkthrough p. 20 :
  elle dit l'allégeance du héros au Conseil du Vide (*Council of the Void*, MKUE p. 22), qui structure le background du jeu,
  et peut nommer la Rupture, la Mue ou le Neuvième Cercle (glossaire) ; le style de jeu est celui validé sur la maquette
  (compétences et profil général, issue privée 162). Les **cartes propres** (MKUE p. 3 ; `content/heroes.ron`) sont montrées
  avec leurs effets de base et renforcé, au même texte que dans la fenêtre des offres. Les compétences ne sont pas affichées
  avant la spec 051.

## Modèle

Termes du glossaire déjà présents (section « Nouvelle partie ») : Graine, Cartes propres, Écran de nouvelle partie
(`NewGameScreen`), Illustration du héros (`hero_image`), Session du client (`ClientSession`).

Nouveaux termes, ajoutés au glossaire par cette PR (section « Nouvelle partie », hors livret) :

- **Étape** (`NewGameStep`) : Scénario ou Héros ; l'écran en montre une à la fois, la barre d'étapes passe de l'une à l'autre.
- **Fiche du scénario** (`ScenarioSheet`) et **fiche du héros** (`HeroSheet`) : la grande zone de droite de l'étape.
- **Accroche** : la ligne courte sous le nom d'un scénario ou d'un héros dans la liste de gauche.

Rien ne change dans l'état du jeu ni dans la vue. Dans arcanum-ui :

- le **formulaire** (`NewGameForm`) gagne l'étape affichée et l'état du volet « Avancé » (ouvert ou fermé) ;
- l'**écran** (`NewGameScreen`, toujours calculé sans egui) gagne : la barre d'étapes (deux onglets, l'actif marqué), le
  récapitulatif « Votre partie : {scénario} · {héros} », le titre de la liste, la liste de l'étape (scénarios ou héros avec
  leur accroche, et pour les héros l'illustration et la couleur de repli), la fiche de l'étape, le bouton « Suivant » (étape
  Scénario seulement), le volet « Avancé » (ouvert ou non, son libellé). Il perd la ligne de cartes propres des héros et le
  résumé des scénarios ;
- la **fiche du scénario** : nom, texte d'ambiance, trois faits (Durée, Joueurs, Type : libellé et valeur), quatre rubriques
  dans l'ordre Objectif, Règles spéciales, Fin de partie, Décompte (titre et puces), et la ligne « Pas encore dans le jeu »
  si le scénario en a une ;
- la **fiche du héros** : nom, illustration (`hero_image`, la même que la figurine et le portrait du panneau) et couleur de
  repli, description, puces du style de jeu, et ses deux cartes propres dans l'ordre de `heroes.ron`, chacune avec son nom,
  la carte remplacée, sa couleur, le texte de l'effet de base, le titre « Renforcé (mana {couleur}) » et le texte de l'effet
  renforcé ;
- les textes des fiches sont des libellés (section Libellés) : une clé par rubrique, une puce par ligne (lignes séparées par
  `\n` dans le JSON, pour que chaque langue ait le nombre de puces qu'il lui faut). Une table d'arcanum-ui dit, pour chaque
  scénario, s'il a une ligne « Pas encore dans le jeu » (Première reconnaissance : oui ; bac à sable : non).
- `ScenarioId::summary_key` et les clés `scenario.*.summary` disparaissent (remplacés par l'accroche et la fiche) ;
  `new_game.own_cards` et `new_game.replaces` aussi (remplacés par les fiches des cartes).

Aucune action ni aucun événement : l'écran construit une `GameConfig` comme aujourd'hui.

## Actions et événements

Aucun.

## Comportements attendus

Situation de départ des tests : comme la spec 065 (`crates/arcanum-ui/tests/nouvelle_partie.rs`), le formulaire et
`new_game_screen(form, contexte, catalog)` avec le `fr.json` du dépôt, la `ClientSession` pour les commandes et les touches,
un dossier de sauvegardes temporaire pour C16.

Étapes :

- **C1** — Au lancement, sans sauvegarde automatique : étape Scénario, Première reconnaissance et Arythea cochés, graine vide,
  volet « Avancé » fermé ; la barre d'étapes a deux onglets, « Scénario » (actif) puis « Héros » ; le récapitulatif est
  « Première reconnaissance · Arythea » (`new_game_opens_on_scenario_step`) — MKUE p. 1. Remplace C1 de la spec 065
  (`new_game_screen_defaults`), dont l'ordre des héros et des scénarios reste vérifié par C2 et C7.
- **C2** — Étape Scénario : la liste a pour titre « Scénarios » et donne, dans l'ordre de `ScenarioId::ALL`, Première
  reconnaissance (accroche « Conseillé pour une première partie ») puis Bac à sable (« Essai libre, sans score »)
  (`scenario_list_shows_taglines`). Remplace C3 de la spec 065 (`scenario_shows_summary`).
- **C3** — Fiche de Première reconnaissance : nom, ambiance (`scenario.first_reconnaissance.flavor`), faits Durée
  « 4 manches : Jour, Nuit, Jour, Nuit », Joueurs « Solo », Type « Entraînement » ; rubriques Objectif (2 puces), Règles
  spéciales (2), Fin de partie (2), Décompte (2), dans cet ordre ; ligne « Pas encore dans le jeu » présente
  (`first_reconnaissance_sheet`) — MKUE p. 18 ; spec 063.
- **C4** — Fiche du bac à sable : Type « Essai libre », rubriques à une puce chacune, pas de ligne « Pas encore dans le jeu »
  (`sandbox_sheet_has_nothing_missing`).
- **C5** — Cliquer le bac à sable : il est coché, Première reconnaissance ne l'est plus, la fiche devient celle du bac à sable,
  le récapitulatif devient « Bac à sable · Arythea » ; l'étape reste Scénario (`choosing_a_scenario_updates_sheet_and_recap`).
- **C6** — « Suivant : choisir le héros » (étape Scénario) passe à l'étape Héros ; l'étape Héros n'a pas de bouton « Suivant »
  ni de bouton de retour, la barre d'étapes suffit (`next_goes_to_hero_step`).
- **C7** — Cliquer l'onglet « Héros », puis « Scénario », change l'étape affichée et l'onglet actif ; le scénario, le héros, la
  graine et le volet restent ce qu'ils étaient (`step_bar_switches_steps`).

Héros :

- **C8** — Étape Héros : la liste a pour titre « Héros » et donne, dans l'ordre de `Hero::ALL` (Arythea, Goldyx, Norowas,
  Tovak), l'illustration du héros (`hero/<héros>.svg`) avec sa couleur de repli, son nom et son accroche (Arythea : « Feu,
  siège et nuit ») ; cliquer Tovak le coche, change la fiche et le récapitulatif (`hero_list_shows_portraits_and_taglines`).
  Remplace C4 et C4b de la spec 065 (`choosing_hero_and_scenario`, `hero_choices_show_their_illustration`).
- **C9** — Fiche d'Arythea : nom, illustration `hero/arythea.svg`, description (`hero.arythea.description`), titre « Style de
  jeu » et ses 4 puces (`hero_sheet_shows_description_and_style`).
- **C9b** — Portraits (Mathieu, 2026-11-13, à l'implémentation) : la fiche et la liste de chaque héros portent aussi son
  portrait généré `hero-art/<héros>.png` (famille d'illustrations `hero-art`, ADR 0009), affiché à la place du médaillon SVG
  quand le fichier est là ; sans lui, le médaillon (`hero_portraits_are_hero_art`, `every_hero_portrait_exists`). Ce sont
  les portraits de référence des héros, déplacés de `scripts/illustrations/heroes/` vers `assets/images/hero-art/`.
- **C10** — Cartes propres d'Arythea, dans l'ordre de `heroes.ron` : Polyvalence (`BattleVersatility`, rouge) « à la place
  de Rage », Base « Attaque 2 ou Parade 2 ou Attaque à distance 1 », titre « Renforcé (mana rouge) », texte « Attaque 4 ou
  Parade 4 ou Attaque de feu 3 ou Parade de feu 3 ou Attaque à distance 3 ou Attaque de siège 2 » ; puis Attraction de mana
  (`ManaPull`, blanc) « à la place de Source de mana », titre « Renforcé (mana blanc) ». Les textes sont ceux que la carte
  montre dans la fenêtre des offres (même fonction), sans le préfixe « Renforcé : » (`hero_sheet_shows_own_cards_with_effects`)
  — MKUE p. 3 ; `basic_actions.ron`. Remplace C2 de la spec 065 (`hero_shows_own_cards`).
- **C11** — Pour chacun des quatre héros, aucune des huit cartes propres n'a de texte « pas encore codé » ni « pas encore
  disponible » (`every_own_card_effect_is_written`) : vrai aujourd'hui, toutes ont `basic` et `strong` en `Options`
  (`basic_actions.ron`).

Commencer et graine :

- **C12** — Étape Scénario, sans passer par l'étape Héros : « Commencer » lance la partie avec le héros coché (Arythea au
  premier lancement) (`start_from_scenario_step_uses_checked_hero`). Depuis l'étape Héros, comme C5 de la spec 065
  (`start_uses_chosen_hero_and_scenario`, inchangé).
- **C13** — Volet « Avancé » : fermé, il ne montre que son libellé « Avancé » ; ouvert, le champ de la graine et son aide
  (`new_game.seed`, `new_game.seed_hint`, inchangés). Refermé avec la graine « 42 » saisie, il garde la graine, son libellé
  devient « Avancé (graine saisie) », et « Commencer » lance la partie avec la graine 42 (`closed_advanced_keeps_the_seed`).
- **C14** — Graine invalide (« abc ») : le volet est ouvert et ne peut pas être refermé tant qu'elle l'est, la ligne
  `new_game.seed_invalid` est sous le champ et « Commencer » est grisé sur les deux étapes (`invalid_seed_keeps_advanced_open`).
  Complète C8 de la spec 065 (`invalid_seed_disables_start`, inchangé).

Ouverture et derniers choix :

- **C15** — En cours de partie (Bac à sable, Tovak), Ctrl+N ouvre l'écran à l'étape Scénario, bac à sable et Tovak cochés,
  graine vide, volet fermé ; « Annuler » et l'avertissement d'abandon sont là comme aujourd'hui
  (`new_game_window_prefills_current_choice`, complété de l'étape et du volet) — spec 065, C11.
- **C16** — Au lancement, la sauvegarde automatique est lisible (partie terminée ou non) : son scénario et son héros sont
  cochés au lieu de Première reconnaissance et Arythea ; illisible ou absente, les choix de C1 (`launch_preselects_last_game`)
  — Q1. « Continuer » reste réservé à une partie non terminée (spec 065, C15 et C16, inchangés).
- **C17** — Chaque clé nouvelle existe en français et en anglais, et chaque scénario de `ScenarioId::ALL` et chaque héros de
  `Hero::ALL` a toutes ses clés de fiche, chacune non vide (`every_scenario_and_hero_has_its_sheet_texts`, et
  `every_new_game_key_exists_in_french` complété).

Touches (Q3) :

- **C18** — Hors champ de saisie : ↓ coche l'élément suivant de la liste de l'étape et ↑ le précédent, sans boucler (↓ sur
  Tovak ne fait rien) ; → passe à l'étape Héros, ← à l'étape Scénario (`arrows_choose_and_switch_steps`). Dans le champ de la
  graine, les flèches restent au champ.

Les autres cas de la spec 065 sont inchangés et gardent leurs tests : graine (C5 à C9), panneau (C10), Annuler et Échap
(C12), partie abandonnée sauvegardée (C13), fenêtre de fin (C14), Continuer et Charger (C15 à C18), L1 à L7.

## Cas limites et refus

- **L1** — Étape Héros, puis cocher un autre scénario par l'onglet « Scénario » : le héros coché ne change pas.
- **L2** — Fenêtre de chargement ouverte depuis l'écran (« Charger… », Ctrl+O) : par-dessus l'écran plein, comme aujourd'hui
  par-dessus la fenêtre (spec 065, L5) ; la fermer ramène à l'étape où l'on était.
- **L3** — Fenêtre de 1280 × 800 (taille d'ouverture du jeu) : la fiche du héros la plus longue (Goldyx) tient en largeur ; si
  elle ne tient pas en hauteur, la fiche défile, la barre d'étapes et le pied (boutons) restent en place (vérification à l'œil).
- **L4** — Échap au lancement ne fait rien, en cours de partie ferme l'écran (spec 065, L3 et C12, inchangés), quelle que soit
  l'étape.

## Information cachée

Aucune : l'écran ne montre que du contenu public (textes, cartes propres) et les choix du joueur. La graine tirée de l'horloge
n'est toujours jamais affichée (spec 065, C9) ; une graine saisie l'est dans son champ, comme aujourd'hui.

## Libellés (i18n)

Clés réutilisées, relues dans `assets/i18n/fr.json` :

- `new_game.title` « Nouvelle partie » (titre de l'écran), `new_game.scenario` « Scénario » et `new_game.hero` « Héros »
  (onglets de la barre d'étapes), `new_game.start`, `new_game.cancel`, `new_game.continue`, `new_game.load`,
  `new_game.seed`, `new_game.seed_hint`, `new_game.seed_invalid`, `new_game.abandon_warning` : inchangées.
- `mana.red` « rouge », `mana.white` « blanc »… dans `new_game.card_strong` (ci-dessous).
- Les textes d'effet des cartes (`effect.*`, `effect.or` « {first} ou {second} ») : ceux de la fenêtre des offres.

Clés retirées : `new_game.own_cards`, `new_game.replaces`, `scenario.sandbox.summary`,
`scenario.first_reconnaissance.summary`.

Clés nouvelles de l'écran (`new_game.recap_choice` ajoutée à l'implémentation : la clé `panel.scenario_and_hero` que la
spec pensait réutiliser n'existe plus depuis la spec 074) :

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `new_game.recap` | — | Votre partie : | Your game: |
| `new_game.recap_choice` | `scenario`, `hero` | {scenario} · {hero} | {scenario} · {hero} |
| `new_game.scenarios` | — | Scénarios | Scenarios |
| `new_game.heroes` | — | Héros | Heroes |
| `new_game.next` | — | Suivant : choisir le héros | Next: choose your hero |
| `new_game.advanced` | — | Avancé | Advanced |
| `new_game.advanced_filled` | — | Avancé (graine saisie) | Advanced (seed entered) |
| `new_game.fact_length` | — | Durée | Length |
| `new_game.fact_players` | — | Joueurs | Players |
| `new_game.fact_kind` | — | Type | Type |
| `new_game.objective` | — | Objectif | Goal |
| `new_game.special_rules` | — | Règles spéciales | Special rules |
| `new_game.game_end` | — | Fin de partie | Game end |
| `new_game.scoring` | — | Décompte | Scoring |
| `new_game.style` | — | Style de jeu | Play style |
| `new_game.own_cards_title` | — | Cartes propres | Own cards |
| `new_game.card_replaces` | `card` | à la place de {card} | instead of {card} |
| `new_game.card_basic` | — | Base | Basic |
| `new_game.card_strong` | `color` | Renforcé (mana {color}) | Powered ({color} mana) |

Avec `{card}` = « Rage » : « à la place de Rage » (en : « instead of Rage »). Avec `{color}` = `mana.red` : « Renforcé (mana
rouge) » (en : « Powered (red mana) ») ; « blanc », « vert », « bleu » s'accordent avec « mana » (masculin).

Clés nouvelles des scénarios (groupe `scenario.<id>`, à côté de `name`) ; `\n` sépare les puces :

| Clé | fr | en |
|---|---|---|
| `scenario.first_reconnaissance.tagline` | Conseillé pour une première partie | Recommended for a first game |
| `….flavor` | Une contrée inexplorée s'étend devant vous. Votre mission : la parcourir jusqu'à trouver sa capitale, et revenir avec toute la gloire et les trésors que vous saurez y gagner. | An unexplored land lies before you. Your mission: cross it until you find its capital, and come back with all the fame and treasure you can win along the way. |
| `….length` | 4 manches : Jour, Nuit, Jour, Nuit | 4 rounds: Day, Night, Day, Night |
| `….players` | Solo | Solo |
| `….kind` | Entraînement | Training |
| `….objective` | Explorer la région jusqu'à révéler une cité.\nGagner le plus de gloire possible en chemin. | Explore the land until you reveal a city.\nGain as much Fame as you can on the way. |
| `….special_rules` | Chaque tuile révélée rapporte 1 gloire.\nLes cités ne peuvent être ni conquises ni visitées. | Each revealed tile gives 1 Fame.\nCities can be neither conquered nor entered. |
| `….game_end` | Une cité est révélée : vous finissez ce tour, puis jouez un dernier tour.\nSinon, la partie s'arrête à la fin de la 4e manche. | A city is revealed: finish this turn, then play one last turn.\nOtherwise, the game ends after the 4th round. |
| `….scoring` | Votre gloire, plus les Hauts Faits.\nPas de titres en solo. | Your Fame, plus the Achievements.\nNo titles in solo play. |
| `….missing` | Pas encore dans le jeu : le joueur fantôme et le choix des tactiques de la variante solo. On peut s'en passer pour une première partie. | Not in the game yet: the Dummy player and Tactic selection of the solo variant. You can do without them for a first game. |
| `scenario.sandbox.tagline` | Essai libre, sans score | Free practice, no score |
| `….flavor` | Un terrain d'essai sans enjeu : déplacez-vous, explorez et découvrez les cartes à votre rythme. | A practice ground with nothing at stake: move, explore and discover the cards at your own pace. |
| `….length` | 4 manches : Jour, Nuit, Jour, Nuit | 4 rounds: Day, Night, Day, Night |
| `….players` | Solo | Solo |
| `….kind` | Essai libre | Free practice |
| `….objective` | Aucun : apprendre les règles en jouant. | None: learn the rules by playing. |
| `….special_rules` | Pas de gloire pour les tuiles révélées. | No Fame for revealed tiles. |
| `….game_end` | À la fin de la 4e manche. | After the 4th round. |
| `….scoring` | Pas de score final. | No final score. |

« Joueur fantôme », « Hauts Faits », « Tactiques », « manche » : termes du glossaire (en : *Dummy player*, *Achievements*,
*Tactics*, *round*, ceux du livret).

Clés nouvelles des héros (groupe `hero.<id>`, à côté de `name`) ; `\n` sépare les puces du style :

| Clé | fr | en |
|---|---|---|
| `hero.arythea.tagline` | Feu, siège et nuit | Fire, siege and night |
| `hero.arythea.description` | On murmure que son culte a déclenché la Rupture. Revenue du chaos plus puissante que jamais, elle met sa lame au service du Conseil du Vide, pourvu que le sang coule pour son dieu. | Her cult is whispered to have caused the Breaking. Back from the chaos stronger than ever, she lends her blade to the Council of the Void, as long as blood flows for her god. |
| `hero.arythea.style` | Offensive : attaques de feu et de siège, idéale pour prendre d'assaut les sites fortifiés.\nPlus forte la nuit : se déplace et négocie mieux dans l'obscurité, tire parti du mana noir.\nRetourne la douleur à son avantage : ses Blessures peuvent servir au lieu de l'encombrer.\nMoins à l'aise pour recruter ou soigner : plutôt pour un joueur qui aime le risque. | Offensive: fire and siege attacks, ideal to assault fortified sites.\nStronger at night: moves and negotiates better in the dark, makes use of black mana.\nTurns pain to her advantage: her Wounds can help instead of clogging her hand.\nLess at ease recruiting or healing: for players who like risk. |
| `hero.goldyx.tagline` | Cristaux et mana | Crystals and mana |
| `hero.goldyx.description` | Sorti de la Mue plus fort qu'aucun de ses semblables, il ne se fie à personne. Le Conseil du Vide l'a envoyé en mission, mais seuls les trésors et les ennemis dignes de lui l'intéressent. | Out of the Surge stronger than any of his kind, he trusts no one. The Council of the Void sent him on a mission, but only treasures and worthy foes interest him. |
| `hero.goldyx.style` | Maître du mana : gagne cristaux et jetons à volonté, et renforce presque toutes ses cartes.\nExcellent avec les sorts, qu'il alimente sans peine.\nPeut voler par-dessus le terrain et bloquer la Source aux autres.\nSouvent jugé le plus puissant, mais demande de bien planifier son mana. | Master of mana: gains crystals and tokens at will, and powers almost all his cards.\nExcellent with Spells, which he fuels with ease.\nCan fly over the terrain and block the Source for others.\nOften considered the strongest, but his mana needs careful planning. |
| `hero.norowas.tagline` | Unités et diplomatie | Units and diplomacy |
| `hero.norowas.description` | Ancien chef des armées elfiques, il a rallié le Conseil du Vide pour imposer son ordre aux royaumes nés de la Rupture, sans pitié ni hésitation. | Once the leader of the elven armies, he joined the Council of the Void to impose his order on the kingdoms born of the Breaking, without mercy or hesitation. |
| `hero.norowas.style` | Chef d'armée : ses unités frappent et parent plus fort, et le font avancer plus vite.\nDiplomate : Influence élevée de jour, gloire et réputation en plus quand il recrute.\nGarde ses troupes en état : soigne ou prépare une unité, recrute moins cher.\nPlus faible seul au combat en début de partie : il doit vite s'entourer. | Warlord: his Units attack and block harder, and help him move faster.\nDiplomat: high Influence by day, extra Fame and Reputation when he recruits.\nKeeps his troops fit: heals or readies a Unit, recruits for less.\nWeaker alone in combat early on: he must surround himself quickly. |
| `hero.tovak.tagline` | Combat sans magie, pour débuter | Combat without magic, for beginners |
| `hero.tovak.description` | Le Neuvième Cercle vendait ses épées au plus offrant ; sous sa poigne, il est devenu une puissance. Il épouse les conquêtes du Conseil du Vide tant qu'elles frappent les factions qui méprisent les siens. | The Ninth Circle sold its swords to the highest bidder; under his grip, it became a power. He embraces the Council of the Void's conquests as long as they strike the factions that scorn his people. |
| `hero.tovak.style` | Guerrier solide : attaque, parade et tir sans dépendre du mana.\nBrise les résistances ennemies et encaisse les Blessures sans faiblir.\nTire le meilleur des cartes jouées de côté : peu de mana, beaucoup d'effet.\nLe plus simple à prendre en main : conseillé pour une première partie. | Solid warrior: attacks, blocks and shoots without relying on mana.\nBreaks enemy resistances and takes Wounds without weakening.\nGets the most out of cards played sideways: little mana, lots of effect.\nThe easiest to learn: recommended for a first game. |

Les accroches et les styles de jeu sont ceux de la maquette validée (issue privée 162), sauf l'accroche de Tovak (« Combat, sans
magie · débutant » devient une phrase). Les descriptions remplacent celles de la maquette, marquées « provisoires ».

Vérifications (issue privée 129) : `scenario.<id>` et `hero.<id>` sont déjà des groupes (`name`), on y ajoute des libellés frères ;
`new_game.fact_length` et les autres sont des libellés à plat, pas des filles de `new_game.scenario` (un libellé). Aucun
caractère nouveau : « · » s'affiche déjà (`load.entry`) ; les « › » et « ▸ ▾ » de la maquette ne sont **pas**
écrits en texte, le séparateur des onglets est dessiné et le volet « Avancé » est un en-tête repliable d'egui, qui dessine
sa propre flèche.

## Interface

Plein écran, au lancement comme en cours de partie (Ctrl+N, Menu, fenêtre de fin) : l'écran occupe toute la fenêtre sous la
barre de menu (Q4), le plateau et les panneaux ne sont pas dessinés tant qu'il est ouvert. Disposition de la maquette :

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ Nouvelle partie   (1 Scénario) — (2 Héros)                Votre partie : [Première reco… · Arythea] │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ SCÉNARIOS                  ┌─────────────────────────────────────────────────────────────────┐ │
│ ┃Première reconnaissance   │ Première reconnaissance                                         │ │
│ ┃Conseillé pour une …      │ Une contrée inexplorée s'étend devant vous… (italique)          │ │
│  Bac à sable               │ [DURÉE 4 manches…] [JOUEURS Solo] [TYPE Entraînement]           │ │
│  Essai libre, sans score   │ Objectif              Règles spéciales                          │ │
│                            │ • …                    • …                                       │ │
│                            │ Fin de partie         Décompte                                  │ │
│                            │ • …                    • …                                       │ │
│                            │ [Pas encore dans le jeu : le joueur fantôme…]                   │ │
│                            └─────────────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ [Continuer : …] [Charger…]                  ▸ Avancé   [Commencer] [Suivant : choisir le héros] │
└──────────────────────────────────────────────────────────────────────────────────────────────┘

Étape Héros, fiche :
│ (  portrait   )  Arythea                                                                       │
│ (  ~300 pts   )  Cultiste du sang sortie plus puissante… (italique)                            │
│                  Style de jeu : • … (4 puces)                                                  │
│                  Cartes propres                                                                │
│                  ┌▀▀▀▀▀ rouge ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀┐ ┌▀▀▀▀▀ blanc ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀┐                      │
│                  │ Polyvalence  à la place de Rage │ │ Attraction de mana  à la place…│     │
│                  │ BASE  Attaque 2 ou Parade 2 …   │ │ BASE  …                        │     │
│                  │ RENFORCÉ (MANA ROUGE)  …        │ │ RENFORCÉ (MANA BLANC)  …       │     │
```

- **En-tête** : titre `new_game.title`, barre d'étapes (deux onglets numérotés 1 et 2, l'actif en or ; séparateur dessiné),
  récapitulatif `new_game.recap` puis `new_game.recap_choice` à droite (C1, C5, C7). Pas d'onglet « Options » avant
  qu'une spec en ajoute une (Q2).
- **Liste de gauche** (environ 340 points de large) : titre `new_game.scenarios` ou `new_game.heroes`, puis un bouton par
  élément, nom (environ 18 points) et accroche (environ 14 points) ; l'élément coché a un liseré à gauche, or pour un
  scénario, couleur du héros (`hero_color`) pour un héros. Les héros ont leur portrait (C9b) d'environ 56 points, aux coins arrondis (C2, C8).
- **Fiche de droite**, sur un fond plus clair, défilante si besoin (L3) : celle du scénario (C3, C4) ou celle du héros (C9,
  C10). Faits en trois cases côte à côte, rubriques sur deux colonnes, titres de rubrique en or, puces ; ligne « Pas encore
  dans le jeu » dans un encadré ambré. Fiche du héros : portrait (C9b) d'environ 300 points, aux coins arrondis, à gauche (plus petite si la
  fenêtre est basse), nom (environ 34 points), description en italique, style de jeu, puis les deux cartes propres côte à
  côte, chacune avec un bandeau de la couleur de la carte en haut.
- **Pied** : à gauche « Continuer : … » et « Charger… » (lancement) ou l'avertissement d'abandon et le message d'échec de
  sauvegarde (partie en cours) ; à droite l'en-tête repliable « Avancé » (C13, C14), « Annuler » (partie en cours),
  « Commencer », et à l'étape Scénario « Suivant : choisir le héros », bouton principal en or (C6, C12). Le volet ouvert
  s'affiche entre la fiche et le pied, sur toute la largeur : libellé de la graine, champ (20 caractères), aide.
- **Texte plus grand** que la fenêtre de la spec 065 : tailles indicatives de la maquette (en points egui), vérification à
  l'œil ; couleurs et espacements aussi.

Vérification à l'œil (`scripts/verif-manuelle.sh`, en fr puis avec `ARCANUM_VERIF_LANGUE=en`) : au lancement, les deux fiches de
scénario, « Suivant », les quatre fiches de héros ; ouvrir et refermer « Avancé », saisir « abc » puis « 42 » ; ↑/↓ et ←/→ ;
commencer depuis l'étape Scénario ; en partie, Ctrl+N, Échap ; fenêtre de 1280 × 800, fiche de Goldyx (L3). Captures jointes
à la PR d'implémentation.

### Raccourcis clavier

Reportés dans `docs/raccourcis.md` :

- **↑ / ↓** (nouveau, Q3) : cocher l'élément précédent ou suivant de la liste de l'étape (scénario ou héros), sans boucler.
  Écran de nouvelle partie ouvert, hors champ de saisie (C18).
- **← / →** (nouveau, Q3) : passer à l'étape Scénario ou à l'étape Héros. Même condition (C18).
- **Entrée** : commencer, sur les deux étapes, graine valide, y compris depuis le champ de la graine (spec 065, inchangé ; elle
  ne fait pas « Suivant »).
- **Échap** et **Ctrl+N** : inchangés (spec 065).

## Effets sur les specs déjà implémentées

- **065** : la fenêtre devient l'écran plein écran ; C1 à C4b remplacés par C1, C2, C3, C8, C9, C10 ; C11 complété par C15.
  Tests modifiés dans `crates/arcanum-ui/tests/nouvelle_partie.rs` : `new_game_screen_defaults`, `hero_shows_own_cards`,
  `scenario_shows_summary`, `choosing_hero_and_scenario`, `hero_choices_show_their_illustration` (remplacés),
  `new_game_window_prefills_current_choice` et `every_new_game_key_exists_in_french` (complétés). Champs retirés de
  `NewGameScreen` : `hero_title`, `scenario_title`, `HeroChoice::own_cards`, `ScenarioChoice::summary` ; leurs seuls lecteurs
  sont `crates/arcanum-ui/src/app/windows.rs` (dessin de la fenêtre, remplacé) et ces tests (`grep` du 2026-11-13).
- **074** : la barre de menu reste au-dessus de l'écran en cours de partie, avec ses commandes de partie grisées comme
  aujourd'hui (`menu_greys_game_commands_behind_new_game_window`, inchangé) ; toujours pas de barre au lancement (Q4).
- **012 / 063** : `ScenarioId::summary_key` disparaît d'arcanum-core (seul lecteur : `new_game.rs`).

## Hors périmètre

- Étape « Options » (joueur fantôme, variantes, niveaux de cité) : avec la spec qui en apporte une (061 joueur fantôme d'abord).
- Compétences des héros sur leur fiche : spec 051.
- Se souvenir des choix sans sauvegarde automatique (fichier de préférences) : spec 080.
- Illustration des cartes propres sur la fiche du héros (seulement leur texte ici) ; nouveaux portraits de héros.
- Fiche de « Conquête en solitaire » : la spec 064 ajoute ses clés, l'écran n'a rien à changer.

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-11-13) :

- **Q1 — Derniers choix au lancement** : scénario et héros de la sauvegarde automatique lisible, terminée ou non (C16) ; le
  reste avec les préférences (spec 080).
- **Q2 — Onglet « Options »** : pas affiché tant qu'aucune spec n'apporte d'option.
- **Q3 — Touches** : ↑/↓ pour changer de scénario ou de héros, ←/→ pour changer d'étape, hors champ de saisie (C18).
- **Q4 — Barre de menu** : gardée au-dessus de l'écran en cours de partie, commandes de partie grisées comme aujourd'hui.
- **Q5 — Textes nouveaux** : les descriptions des héros disent leur allégeance au Conseil du Vide ; Mathieu a choisi la
  version « background » (Rupture, Mue, Neuvième Cercle) parmi deux propositions, reportée dans Libellés. L'ambiance réécrite
  de Première reconnaissance est gardée telle que proposée.
