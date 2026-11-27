# #20 — Obtention des cartes

- Statut : validée par Mathieu le 2026-11-27 (Q1 à Q3 tranchées comme recommandé, maquette du niveau pair retenue)
- Issue : #20 (milestone v0.2.0)
- Dépend de : spec #32 (cartes des fenêtres : la carte dessinée hors de la zone du joueur, « Prendre » en haut à droite),
  issues #19 et #69 (boutons de choix en haut à droite, PR #89), spec 050 (offres et cartes gagnées), spec 051
  (compétences, choix du niveau pair), spec 045 (ruines : action avancée et unité en récompense), spec 056 (Entraînement,
  Don pour la magie, Apprentissage), spec 055 (Livre de sagesse), spec 041, 043 et 062 (achats des sites) ; toutes
  implémentées
- Livret : MKUE p. 11, 6.b (niveau pair) ; p. 11, 5.c et 5.d (récompenses) ; aucune règle changée
- Crates touchées : arcanum-ui, arcanum-core (un champ de la vue, C9), assets/i18n
- Base : issue #20 (reprise de l'issue privée 371), idée de Mathieu.
- Maquette de la fenêtre du niveau pair (demandée par Mathieu, 2026-11-27) :
  [`images/issue-20-obtention-des-cartes/`](images/issue-20-obtention-des-cartes/), captures du jeu en 1 600 × 900 avec un
  essai de code jetable, non versionné (patch dans les fichiers du projet, `ihm/issue-20-maquette-jetable.patch`) : Tovak au
  niveau 2, Marche forcée et Vision nocturne révélées, Attaque givrante (compétence de Goldyx, le fantôme) dans l'offre
  commune, offre Magie pure, Chemins de traverse, Agilité.
  [Aucune compétence encadrée](images/issue-20-obtention-des-cartes/maquette-sans-choix.png) (« Prendre » grisé partout, ligne
  d'aide) ; [compétence révélée encadrée](images/issue-20-obtention-des-cartes/maquette-competence-revelee.png) (« Prendre »
  actif sur les trois) ; [compétence de l'offre commune encadrée](images/issue-20-obtention-des-cartes/maquette-offre-commune.png)
  (seul « Prendre » d'Agilité, en dernière place, est actif). Disposition retenue par la maquette : compétences révélées et
  offre commune côte à côte sur une rangée, la note de l'offre commune sous ses tuiles, la ligne d'aide à côté du titre
  « Actions avancées », la fenêtre posée en haut du plateau pour que les cartes tiennent sans défiler en 1 600 × 900.

## Objectif

Une carte qui n'est pas dans la zone du joueur se montre **toujours de la même façon**, la « carte hors zone » de la spec #32
(cadre de la main, bande, texte de base et renforcé, sans bouton d'effet). Et le joueur **obtient une carte toujours dans
une fenêtre dédiée**, qui ne montre que les cartes concernées :

- la fenêtre **Offres** ne s'ouvre plus d'office : elle ne s'ouvre qu'au bouton « Offres… » (ou touche O), pour consulter ;
- l'action avancée d'une récompense, l'unité d'une récompense et la carte d'Entraînement, de Don pour la magie renforcé ou
  du Livre de sagesse se prennent dans une **fenêtre de choix** comme celles de l'artefact et du sort ;
- un **niveau pair** se règle dans **une seule fenêtre**, compétence et action avancée ensemble, qui montre ce que le choix
  de la compétence impose à l'action avancée ;
- l'**infobulle** d'un achat de carte (monastère, tour de mage, cité, recrutement) montre la carte hors zone au lieu de
  son texte.

## Règles

Aucune règle nouvelle ni changée : spec d'interface. Les choix restent ceux du moteur, dans le même ordre (spec 045, C24 :
récompenses avant niveaux ; spec 051 : compétence avant action avancée).

- **Niveau pair** (MKUE p. 11, 6.b ; FAQ p3 ; Compendium, *Skill Tokens* S3) : une compétence révélée laisse prendre
  n'importe quelle action avancée de l'offre ; une compétence de l'offre commune impose celle en dernière place. Ce que la
  spec en fait : la fenêtre montre les deux choix ensemble, et le choix de la compétence n'est joué qu'avec celui de la carte
  (Q1). Le moteur dit, pour chaque compétence prenable, les cartes qu'elle laisse prendre (C9) : l'écran n'applique pas la
  règle lui-même.
- **Offre d'actions avancées vide** (spec 050, L1 ; spec 051 C9) : la compétence se prend seule.
- **Action avancée d'une récompense** (MKUE p. 11, 5.c ; spec 045) : de l'offre d'actions avancées, pas une action avancée
  de monastère de l'offre d'unités.
- **Unité d'une récompense** (MKUE p. 11, 5.d ; spec 045) : de l'offre d'unités ; une action avancée de monastère qui s'y
  trouve n'est pas une unité.

## Modèle

Aucun terme du livret nouveau. Termes d'interface ajoutés au glossaire (section « Obtention des cartes (spec #20) ») :

| Français | Code | Sens |
|---|---|---|
| Carte hors zone | `OfferCardView` (UI) | Une carte montrée hors de la zone du joueur (spec #32) : cadre de la main, nom, bande, texte de base, séparation et texte renforcé, sans bouton d'effet ; seul un bouton de choix (« Prendre ») peut s'y ajouter. |
| Fenêtre de choix | `CardChoiceWindow` (UI) | Fenêtre qui ne montre que les cartes à prendre maintenant, en cartes hors zone, chacune avec « Prendre » : artefact, sort, action avancée ou unité d'une récompense, carte d'un effet « gagner ». |
| Fenêtre du niveau pair | `LevelChoiceWindow` (UI) | Fenêtre d'un niveau pair : compétences proposées, puis actions avancées de l'offre ; une compétence encadrée dit quelles cartes se prennent. |

Changements de l'écran (`arcanum-ui`) :

- **`CardChoiceWindow`** remplace la structure `ArtifactChoice` de l'écran (`screen/types.rs`, sans rapport avec
  `arcanum_core::ArtifactChoice`) : `title`, `cards: Vec<OfferCardView>`, et **`close: Option<Button>`** (nouveau) :
  « Renoncer » (unité d'une récompense), « Annuler » (effet « gagner ») ou rien (artefact, sort, action avancée d'une
  récompense, qu'on ne refuse pas). `GameScreen::artifact_choice` et `spell_choice` en prennent le type ; s'y ajoutent
  `advanced_action_reward`, `unit_reward` et `gain_choice` (`Option<CardChoiceWindow>`). Une seule s'affiche à la fois, dans
  l'ordre du moteur.
- **`LevelChoiceWindow`** remplace `SkillChoiceWindow` (`GameScreen::skill_choice` devient `level_choice`) : le titre, les
  compétences révélées et l'offre commune comme aujourd'hui (`SkillOffer`, avec `selected: bool` et `select:
  Option<Command>`, la commande qui l'encadre ; `take` ne reste que pour l'offre d'actions avancées vide, C7), la note
  de l'offre commune, puis **`advanced_actions: Vec<LevelCard>`** (l'offre d'actions avancées dans son ordre : la carte
  hors zone et son « Prendre » en `PurchaseButton`, actif ou grisé selon la compétence encadrée, avec la note de l'offre
  commune en infobulle quand il est grisé, C2 à C4) et **`hint: Option<String>`** (« Choisissez d'abord une
  compétence. » tant qu'aucune n'est encadrée). Précisé à l'implémentation (fil moteur) : un `OfferCardView` ne sait
  pas dire un « Prendre » grisé ni son infobulle.
- **Compétence encadrée** : un état de l'écran, jamais du moteur, comme la tactique encadrée (spec 060,
  `ScreenMode::ChoosingTactic`) : `ScreenMode::ChoosingLevelSkill(Skill)`, posé par `Command::SelectSkill(Skill)`.
  `Command::TakeLevelReward(CardId)` joue `ChooseSkill` de la compétence encadrée puis `GainAdvancedAction` de la carte,
  l'une après l'autre (la seconde est légale dès la première jouée). Rien de cet état n'est sauvegardé.
- **`PurchaseButton::card: Option<OfferCardView>`** (nouveau) : la carte hors zone de l'achat, montrée en infobulle à la place
  de `tooltip` (C12 à C14) ; `tooltip` reste pour les achats sans carte (soin) ou de carte cachée.
- `OffersWindow` : plus de titre de choix ni de « Renoncer » (`decline`), toujours fermable ; ses boutons « Prendre » de
  choix (niveau, récompense, effet « gagner ») disparaissent. Restent les boutons des effets du tour qui agissent sur l'offre
  (Q2) : « Apprendre » (Apprentissage), « Lancer » (Don pour la magie), « Utiliser : … » (Appel aux armes) et « Recruter
  (gratuit) » (Appel à la gloire).

Changement de la vue (`arcanum-core`) :

- **`PlayerView::skill_choice_cards: Vec<(Skill, Vec<CardId>)>`** (nouveau) : pendant le choix d'un niveau pair, pour chaque
  compétence prenable (révélée, ou de l'offre commune et d'un autre héros), les actions avancées de l'offre qu'elle laisse
  prendre, dans l'ordre de l'offre ; vide hors de ce choix. Calculé par la règle qui donne aujourd'hui les
  `GainAdvancedAction` légales après `ChooseSkill` (`engine/legal/mod.rs`, « dernière place »), partagée, pas recopiée.

Pas de changement du format des sauvegardes : ni événement ni état du moteur ne changent (la vue n'est pas sauvegardée),
donc pas de migration (spec #52).

## Actions et événements

Aucun nouveau : `ChooseSkill`, `GainAdvancedAction`, `DeclineUnitReward`, `TakeArtifact`, `TakeSpell`, `PlayCard` et les
achats existants, inchangés.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Le dessin (disposition, encadré, infobulle) se vérifie
à l'œil (ADR 0006).

Exemple du niveau pair, celui des tests de la spec 051 (`crates/arcanum-ui/tests/competences.rs`, `level_two`) : Tovak, bac à
sable, graine 4, 3 de gloire et le tour fini. Compétences révélées : Bretteur de glace (`ColdSwordsmanship` : « Attaque 2, ou
Attaque de glace 2. ») et Motivation (`MotivationTovak`). Offre d'actions avancées, dans l'ordre : Boule écrasante
(`CrushingBolt`), Récit héroïque (`HeroicTale`), Régénération (`Regeneration`, en dernière place). Offre commune préparée avec
Bretteur de feu (`HotSwordsmanship`, compétence d'Arythea), comme `common_offer_in_skill_choice_window`.

**Fenêtre du niveau pair**

- **C1** — Au niveau 2, une seule fenêtre s'ouvre d'office, titre « Niveau 2 : choisir une compétence et une action
  avancée » : Bretteur de glace et Motivation (non encadrées), puis les trois actions avancées en cartes hors zone, « Prendre »
  grisé sur chacune, et la ligne « Choisissez d'abord une compétence. ». La fenêtre Offres ne s'ouvre pas, ni avant ni après
  le choix (`even_level_opens_one_level_window`) — MKUE p. 11, 6.b.
- **C2** — Bretteur de glace encadrée (`Command::SelectSkill`) : elle seule est encadrée, la ligne d'aide disparaît, et
  « Prendre » est actif sur les trois cartes (`revealed_skill_lets_any_advanced_action_be_taken`) — MKUE p. 11, 6.b.
- **C3** — Bretteur de feu (offre commune) encadrée : seul « Prendre » de Régénération est actif ; Boule écrasante et Récit
  héroïque restent grisées, avec en infobulle « Prendre une compétence de l'offre commune impose l'action avancée en dernière
  place. » (`skill.choice.common_note`) (`common_skill_leaves_only_lowest_advanced_action`) — MKUE p. 11, 6.b ; FAQ p3.
- **C4** — Encadrer une autre compétence après une première (Bretteur de feu puis Motivation) change l'encadré et rend
  « Prendre » aux trois cartes : rien n'est joué tant qu'une carte n'est pas prise (`changing_skill_plays_nothing`) — Q1.
- **C5** — Bretteur de glace encadrée, « Prendre » sur Récit héroïque : `ChooseSkill` (Bretteur de glace) puis
  `GainAdvancedAction` (Récit héroïque) sont joués ; Bretteur de glace est gagnée, Motivation va dans l'offre commune, Récit
  héroïque est piochée (comme spec 051 C5), et la fenêtre se ferme (`take_level_reward_plays_skill_then_card`).
- **C6** — Comme aujourd'hui (spec 051 C8, test `common_offer_in_skill_choice_window`), une compétence de Tovak dans l'offre
  commune est montrée grisée, avec « Compétence de votre héros : pas prenable ici », et ne s'encadre pas
  (`own_skill_cannot_be_selected`).
- **C7** — Offre d'actions avancées vide (spec 051 C9) : la section des actions avancées est absente et chaque compétence
  prenable porte « Prendre », qui joue `ChooseSkill` seul (`empty_offer_takes_skill_alone`).
- **C8** — Deux niveaux pairs d'un coup (spec 051 C10) : la fenêtre du niveau 4 suit celle du niveau 2, avec ses deux
  nouvelles compétences ; aucune compétence n'y est encadrée d'avance (`second_even_level_starts_unselected`).
- **C9** — La vue au niveau 2 : `skill_choice_cards` vaut [(Bretteur de glace, [Boule écrasante, Récit héroïque,
  Régénération]), (Motivation, [les trois]), (Bretteur de feu, [Régénération])] ; la compétence de Tovak de l'offre commune
  n'y est pas ; vide une fois le niveau réglé (`view_lists_cards_allowed_by_each_skill`, arcanum-core) — MKUE p. 11, 6.b.
- **C10** — Touches : 1 à 3 prennent la carte de cette place une fois une compétence encadrée (spec 071, C22) ; sans
  compétence encadrée, ou sur une carte grisée, la touche ne fait rien (`level_window_shortcuts`).

**Fenêtres de choix**

- **C11** — Action avancée d'une récompense (ruine verte et rouge conquise, partie de `ruines.rs`,
  `advanced_action_reward_window`) : fenêtre de choix « Récompense : prenez une action avancée » avec les seules cartes de
  l'offre d'actions avancées, « Prendre » sur chacune, sans bouton de fermeture ; la fenêtre Offres ne s'ouvre pas
  (`advanced_action_reward_in_choice_window`) — MKUE p. 11, 5.c.
- **C12** — Unité d'une récompense (partie `ruines-unite` : Paysans au seul jeton de commandement, Forestiers et Herboristes
  dans l'offre) : fenêtre de choix « Récompense : prenez une unité de l'offre » avec les unités de l'offre seulement, en
  cartes hors zone ; « Prendre » ouvre comme aujourd'hui le choix de l'unité à renvoyer (spec 045, test
  `unit_reward_window_without_free_token`) ; « Renoncer » joue `DeclineUnitReward` (`unit_reward_in_choice_window`) — MKUE
  p. 11, 5.d.
- **C13** — Une action avancée de monastère dans l'offre d'unités n'apparaît pas dans la fenêtre de l'unité d'une récompense
  (`unit_reward_leaves_out_monastery_card`) — spec 045.
- **C14** — Entraînement (`Training` : « Détruire une carte action : prendre une action avancée de même couleur dans la
  défausse »), Marche détruite, offre Chemins de traverse (verte), Boule de feu (rouge), Agilité (blanche) (test
  `training_chooses_destroyed_card_then_offer_card`) : fenêtre de choix titrée « Entraînement : choisissez la carte » avec
  la seule Chemins de traverse, « Prendre » qui joue la carte comme aujourd'hui, et « Annuler » (`Command::CancelChoice`) ;
  la fenêtre Offres ne s'ouvre pas (`training_gain_in_choice_window`) — spec 056 C29.
- **C15** — Don pour la magie renforcé (« Payer un mana : prendre un sort de cette couleur de l'offre dans la défausse »),
  option rouge, offre de sorts Éclair de feu, Tempête de neige, Secousse (test `strong_magic_talent_chooses_spell_in_offers`) :
  la fenêtre de choix ne montre qu'Éclair de feu (`magic_talent_gain_in_choice_window`) — spec 056.
- **C16** — Livre de sagesse (« Écarter une carte action : action avancée de même couleur de l'offre en main ») : même
  fenêtre, avec les seules actions avancées de la couleur de la carte écartée (`book_of_wisdom_gain_in_choice_window`) — spec
  055.
- **C17** — Les cartes d'une fenêtre de choix sont les cartes hors zone de la fenêtre Offres, à `take` près : la carte de
  Récit héroïque dans la fenêtre de l'action avancée d'une récompense vaut celle de la fenêtre Offres (même `name`, `color`,
  `lines`, `strong_heading`, `strong_colors`, `illustration`) (`choice_cards_match_offer_cards`) — spec #32.

**Fenêtre Offres**

- **C18** — La fenêtre Offres ne s'ouvre qu'avec « Offres… » ou la touche O : ni le niveau pair, ni une récompense, ni un
  effet « gagner » ne l'ouvrent ; ouverte, elle a toujours le titre « Offres » et se ferme (Échap, « Fermer »)
  (`offers_window_opens_only_on_request`).
- **C19** — Pendant un choix de carte (niveau pair, récompense, effet « gagner »), « Offres… » et la touche O restent sans
  effet, comme aujourd'hui pendant le choix (test `offers_shortcuts`) (`offers_stay_closed_during_a_choice`).
- **C20** — Comme aujourd'hui (spec 056 C26, test `learning_button_on_advanced_actions_offer` ; spec 054 C27 ; spec 057 C17,
  C18, test `offer_units_use_and_free_recruit`), « Apprendre », « Lancer », « Utiliser : … » et « Recruter (gratuit) »
  restent dans la fenêtre Offres ouverte par le joueur (Q2).

**Infobulle des achats**

- **C21** — Au monastère de `villages_et_monasteres.rs` (Bouclier de glace dans l'offre d'unités, comme la spec #32 C4),
  l'achat « Acheter : Bouclier de glace (6) » (`purchase.advanced_action`) porte la carte hors zone de Bouclier de glace (`IceShield`), égale à celle de la
  fenêtre Offres sans `take` ni `mark` (`monastery_purchase_shows_offer_card`) — spec #32.
- **C22** — À la tour de mage conquise (`tour-conquise`), chaque achat de sort porte la carte hors zone de son sort ; dans la
  cité verte conquise (`cite-verte-conquise`), chaque achat d'action avancée de l'offre aussi, et l'achat de la carte du
  dessus de la pioche, cachée, n'en porte pas (`spell_and_city_purchases_show_cards`).
- **C23** — Au recrutement (partie `unites` : Gardes d'Utem dans l'offre), l'achat « Recruter : Gardes d'Utem (…) »
  (`units.recruit`) porte la carte hors zone de l'unité, égale à celle de la fenêtre Offres ; l'artefact de la cité rouge et
  l'unité d'élite de la cité blanche, cachés, n'en portent pas (`recruit_purchase_shows_unit_card`).

### Tests existants modifiés

Cherchés à la rédaction, là où c'est lu (`grep -rn 'offers_window\|skill_choice\|ArtifactChoice\|tooltip'
crates/arcanum-ui/tests`) :

- `competences.rs` : `skill_choice_window_at_even_level` et `common_offer_in_skill_choice_window` lisent
  `screen.skill_choice` (devient `level_choice`), `SkillOffer::take` (devient `selected`), et attendent la fenêtre Offres
  après `ChooseSkill` ; réécrits sur la fenêtre du niveau pair (C1 à C6).
- `offres.rs` : `choice_window_offers_take_buttons`, `offers_shortcuts` (partie « pendant le choix ») et
  `numbered_choices_show_shortcut` lisent les « Prendre » du choix du niveau dans la fenêtre Offres ; ils passent à la fenêtre
  du niveau pair (C5, C10). La fonction d'aide `events(true)` joue déjà `ChooseSkill` : elle s'arrête avant, ou la fenêtre
  attend la compétence encadrée.
- `ruines.rs` : `advanced_action_reward_window`, `unit_reward_window`, `unit_reward_window_without_free_token` et
  `unit_reward_take_shows_shortcut` lisent `session.offers_window` pendant la récompense ; ils lisent la fenêtre de choix
  (C11, C12). La partie « choix du niveau 2 » de `advanced_action_reward_window` lit la fenêtre du niveau pair.
- `actions_avancees_mana.rs` : `training_chooses_destroyed_card_then_offer_card`, `strong_training_chooses_offer_card_then_mana`
  et `strong_magic_talent_chooses_spell_in_offers` lisent la carte dans `offers_window_in` (`offers_in`, `take_of`) ; ils
  lisent `GameScreen::gain_choice` (C14, C15).
- `artefacts.rs` : les tests du Livre de sagesse qui lisent la carte prise dans la fenêtre Offres (C16).
- `villages_et_monasteres.rs` : `panel_offers_advanced_actions_at_monastery` compare l'infobulle de l'achat aux textes de
  `card_effect_texts` ; elle compare `PurchaseButton::card` (C21).
- Le type `ArtifactChoice` de l'écran, renommé : aucun test ne le nomme (`artefacts.rs` ne lit que
  `arcanum_core::ArtifactChoices`) ; les tests qui lisent `GameScreen::artifact_choice` et `spell_choice`
  (`sites_fortifies.rs`, `villages_et_monasteres.rs`, `cartes_des_fenetres.rs`, `illustrations.rs`) ne changent pas
  d'assertion.
- `icones.rs` (`escape_buttons_show_shortcut`) : la fenêtre Offres garde « Fermer (Échap) » ; vérifié sans changement attendu.
- Sans changement attendu, vérifiés : `offers_window_lists_the_three_offers`, `offers_window_shows_no_deck_size`,
  `offers_window_shows_monastery_cards`, `sorts.rs` (« Lancer » dans Offres), `sorts_deplacement.rs`
  (`offer_units_use_and_free_recruit`), `learning_button_on_advanced_actions_offer`, `offer_card_has_its_illustration`.
- Parties préparées (`crates/arcanum-persistance/tests/parties_preparees.rs`) : `niveau-pair`, `competences`,
  `fantome-competence` et `ruines-unite` promettent la fenêtre Offres ou la fenêtre de compétence ; leur promesse et leur
  ligne de `docs/verification-manuelle.md` changent, pas leurs événements.

## Cas limites et refus

- **L1** — Fenêtre du niveau pair plus haute que l'écran (offre commune pleine, cartes à bande) : comme les fenêtres de la
  spec #32 (L2), elle ne dépasse jamais l'écran, son contenu défile, son titre reste visible.
- **L2** — Une compétence encadrée qui n'est plus prenable (ne devrait pas arriver : l'encadré n'existe que pendant la
  fenêtre) : l'encadré est ignoré, comme une tactique encadrée devenue illégale (spec 060).
- **L3** — Fenêtre de choix d'un effet « gagner » : « Annuler » revient avant le choix de la carte, comme le bandeau du choix
  aujourd'hui (spec 056) ; Échap aussi.
- **L4** — Achat grisé (trop cher) : l'infobulle montre la carte quand même, comme le texte aujourd'hui.

## Information cachée

Rien de nouveau n'est révélé : offres, compétences révélées et offre commune sont déjà dans la vue. Champ ajouté à la vue :
`PlayerView::skill_choice_cards` (Modèle, C9). Les cartes cachées (dessus de la pioche d'actions avancées, artefact et unité
d'élite des cités) n'ont pas de carte en infobulle (C22, C23).

## Libellés (i18n)

Nouvelles clés :

| Clé | fr | en |
|---|---|---|
| `level_choice.title` | Niveau {level} : choisir une compétence et une action avancée | Level {level}: choose a Skill and an Advanced Action |
| `level_choice.skill_first` | Choisissez d'abord une compétence. | Choose a Skill first. |
| `gain_choice.title` | {card} : choisissez la carte | {card}: choose the card |

Avec `{level}` = 2 : « Niveau 2 : choisir une compétence et une action avancée ». Avec `{card}` = « Entraînement » :
« Entraînement : choisissez la carte » / « Training: choose the card ».

Clés réutilisées, relues dans `assets/i18n/fr.json` de `main` : `skill.choice.revealed` « Compétences révélées » ;
`skill.choice.common` « Offre de compétences communes » ; `skill.choice.common_note` « Prendre une compétence de l'offre
commune impose l'action avancée en dernière place. » (aussi l'infobulle des cartes grisées, C3) ; `skill.choice.own`
« Compétence de votre héros : pas prenable ici » ; `offers.advanced_actions` « Actions avancées » ; `offers.take`
« Prendre » ; `reward.choose_advanced_action` « Récompense : prenez une action avancée » ; `reward.choose_unit`
« Récompense : prenez une unité de l'offre » ; `reward.decline_unit` « Renoncer » ; `discard_choice.cancel` « Annuler ».

Clé changée : `gain_choice.prompt`, « {card} : {effect}. Choisissez la carte dans la fenêtre Offres. » devient
« {card} : {effect}. Choisissez la carte dans la fenêtre. » (en : « Choose the card in the window. »).

Clés retirées, sans plus de lecteur : `offers.choose_advanced_action` (« Niveau {level} : choisissez une action avancée ») et
`skill.choice.title` (« Niveau {level} : choisir une compétence »), remplacées par `level_choice.title`.

## Interface

- **Fenêtre du niveau pair** (C1 à C8, `LevelChoiceWindow`, `PlayerView::skill_choice`, `common_skills`,
  `skill_choice_cards`, `offers`) : en haut du plateau, sans croix ; compétences révélées et offre commune côte à côte sur une
  rangée (la note de l'offre commune sous ses tuiles), chaque compétence en tuile (nom, fréquence, texte) qu'un clic encadre,
  comme une tactique (spec 060) ; dessous, « Actions avancées » suivi de la ligne d'aide tant qu'aucune compétence n'est
  encadrée, et l'offre en rangée de cartes hors zone (spec #32), « Prendre » en haut à droite de chaque carte (L4 de la spec
  #32) (disposition : vérification à l'œil, [maquette](images/issue-20-obtention-des-cartes/maquette-sans-choix.png)).
- **Fenêtres de choix** (C11 à C17, `CardChoiceWindow`, `PlayerView::offers`, mode de l'écran pour l'effet « gagner ») :
  celle de l'artefact et du sort de la spec #32, une rangée de cartes hors zone, « Renoncer » ou « Annuler » en bas quand il
  y en a un (vérification à l'œil).
- **Fenêtre Offres** (C18 à C20) : consultation, plus jamais ouverte d'office ; toujours « Offres » et « Fermer ».
- **Infobulle d'un achat** (C21 à C23, `PurchaseButton::card`) : la carte hors zone au survol du bouton d'achat, à la place
  des lignes de texte (vérification à l'œil).

Parties préparées, sans en ajouter (promesses mises à jour) : `niveau-pair` (vaincre les Rôdeurs, finir le tour : fenêtre du
niveau pair, encadrer une compétence, « Prendre » ou touches 1 à 3), `fantome-competence` (la compétence du fantôme dans
l'offre commune : encadrée, seule la dernière carte se prend, C3), `competences` (deux niveaux pairs d'un coup, C8),
`ruines-unite` (fenêtre de l'unité, « Renoncer », C12), `actions-avancees-mana` (Entraînement : fenêtre de choix, C14),
`monastere`, `tour-conquise`, `cite-verte-conquise`, `unites` (infobulles des achats, C21 à C23).

### Raccourcis clavier

Aucune touche nouvelle. Les touches 1 à 3 de « Prendre » (spec 071, C22) valent dans la fenêtre du niveau pair une fois une
compétence encadrée (C10) et dans les fenêtres de choix comme aujourd'hui ; Échap annule la fenêtre d'un effet « gagner »
(L3). `docs/raccourcis.md` : la ligne des touches 1 à 3 cite la fenêtre du niveau pair au lieu de la fenêtre Offres.

## Documentation à mettre à jour (PR d'implémentation)

- `docs/specs/050-offres-et-cartes-gagnees.md`, `051-competences.md`, `045-ruines.md`, `056-actions-avancees-mana.md` : la
  fenêtre Offres ouverte d'office renvoie à cette spec.
- `docs/verification-manuelle.md` : lignes des parties citées dans Interface.
- `docs/architecture.md` si elle décrit la fenêtre Offres, `ArtifactChoice` ou `SkillChoiceWindow` ;
  `docs/reference/index-par-notion.md`, colonne Spec de « Compétences » et des offres ; `docs/raccourcis.md`.

## Hors périmètre

- **Effets du tour qui agissent sur l'offre** (Apprentissage, Don pour la magie de base, Appel aux armes, Appel à la
  gloire) : restent dans la fenêtre Offres, selon Q2.
- **Fenêtre de la défausse, main, écran de combat** : cartes de la zone du joueur, inchangées.
- **Fenêtres des tactiques** (spec 060) : pas des cartes d'action, inchangées.
- **Tuile de compétence dessinée** (icône, cadre de jeton) : la tuile garde le texte de la fenêtre de compétence d'aujourd'hui.

## Questions ouvertes

Tranchées par Mathieu (2026-11-27, « ok pour tout ! ») : les trois propositions recommandées, et la disposition de la maquette.

- **Q1. Niveau pair : quand le choix de la compétence est-il joué ?** Aujourd'hui, « Prendre » une compétence la joue
  aussitôt, et la fin du tour interdit tout retour en arrière : on découvre seulement après que Bretteur de feu ne laisse que
  Régénération. **Retenu** : un clic encadre la compétence sans rien jouer, les cartes s'activent selon
  elle (C2, C3), et « Prendre » sur une carte joue les deux (C5) ; le joueur voit la conséquence avant de s'engager, ce que
  demande l'issue. Écartée : garder le clic qui joue la compétence aussitôt, la fenêtre se mettant à jour ensuite (pas de
  champ de vue nouveau, mais pas de retour possible).
- **Q2. Apprentissage, Don pour la magie, Appel aux armes et Appel à la gloire : fenêtre Offres ou fenêtres dédiées ?**
  Ces effets valent pour tout le tour (on apprend une fois les 6 Influence réunies, on recrute quand on veut) : ce n'est pas
  un choix à faire maintenant, qu'une fenêtre ouverte d'office imposerait. **Retenu** : les laisser dans la
  fenêtre Offres, que le joueur ouvre lui-même (C20) ; l'offre complète ne s'ouvre bien qu'au clic sur « Offres… ». Écartée :
  une fenêtre dédiée par effet, ouverte par un bouton de plus dans le panneau, à spécifier.
- **Q3. Effet « gagner » : seulement les cartes prenables ?** L'issue dit « uniquement les cartes concernées ». **Retenu** :
  seulement les cartes prenables (C14 : Chemins de traverse seule). Écartée : toute l'offre concernée, les
  cartes d'une autre couleur grisées.
