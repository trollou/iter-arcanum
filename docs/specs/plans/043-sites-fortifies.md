# Plan de développement — spec 043 Sites fortifiés

- Statut : accepté par Mathieu avec la spec (2026-11-04).
- Spec : [`043-sites-fortifies.md`](../043-sites-fortifies.md). Branche : `spec/043-sites-fortifies`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests en échec et
  leur nombre (`scripts/verifier.sh --rouge`), section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR (celle qui porte la spec) : assaillir sans conquête, ou conquérir sans pouvoir rien faire du site, laisserait le jeu
bancal.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : plus de 8 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 8 (C1 à C34, persistance, parties préparées, docs du moteur). Il pousse sa dernière étape verte et
   s'arrête.
2. **Fil écran** : étapes 9 à 12 (libellés, C35 à C45, vérification à l'œil, docs de l'écran, bilan), à partir de la spec, de ce
   plan et de la branche.

## Choix de modèle

- **Défenseurs face cachée** : `GameState::enemy_tokens` garde le jeton (le moteur le connaît) ; un ensemble `face_down` (cases)
  dit lesquels le joueur ne voit pas. La vue remplace ceux-là par leur couleur (`PlayerView::face_down_defenders : case →
  EnemyColor`) et ne les met pas dans `enemy_tokens`.
- **Défenseur ou ennemi errant** : un jeton est un défenseur quand sa case porte un site fortifié non conquis ; une fonction
  `rampaging_tokens(state)` (cases des seuls ennemis errants) remplace `enemy_tokens.keys()` dans `provoked`, `challengeable` et
  `routes_avoiding`.
- **Coût de déplacement** : `move_cost` reste une fonction de la case ; une couche au-dessus (`move_cost_at(state, hex)`) rend
  `Points` pour un site conquis, sans toucher aux tests de la spec 014 qui ne voient pas l'état.
- **Assaut** : `PlayerState::assault : Option<Assault { site, from }>` ; `AssaultStarted` met `site` dans `enemies_to_fight` et
  termine le déplacement comme une provocation (spec 030), d'où le moment « à engager » sans code de plus.
- **Fortification** : `CombatEnemy` gagne `site_fortified` (posé par `CombatStarted` quand la case est celle de l'assaut) ;
  `fortification()` = site + capacité (0, 1 ou 2). Les règles de la phase d'attaque à distance (cibles, siège) lisent ce nombre.
- **Récompense sort** : `PlayerState::spell_rewards` ; `EndTurn` s'arrête avant la pioche comme pour l'artefact ; `TakeSpell`
  reprend comme `GainAdvancedAction` (spec 050).
- **Limite de main** : `GameState::draw_limit(player)` = limite du niveau + bonus de forteresse ; `hand_limit` (non modifiée)
  reste pour le seuil d'assommé.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Défenseurs** : `DefenderPlaced` à l'exploration et à la mise en place, face cachée, vue filtrée | C1 à C3 | C1 : pas de `DefenderPlaced` après `TileRevealed` | `engine.rs`, `event.rs`, `state.rs`, `view.rs`, nouveau `tests/sites_fortifies.rs` | non | fil principal |
| 2 | **Retourner les défenseurs** : pas, exploration, manche de Jour ; ils ne sont pas des ennemis errants ; point de non-retour | C4 à C10 | C4 : pas d'`EnemyTokenRevealed` après le pas voisin | `engine.rs`, `combat.rs`, `movement.rs`, `event.rs`, tests | non | fil principal |
| 3 | **Assaut** : `MoveHero` ou `FollowRoute` qui finit sur un site non conquis (trajet arrêté par une révélation), `AssaultStarted`, Réputation −1, moment « à engager », révélation à l'engagement | C11 à C14 (C13b compris), C17, C18, L1, L7 | C11 : le pas vers la forteresse absent des actions légales | `engine.rs`, `movement.rs`, `action.rs`, `event.rs`, tests | non | fil principal |
| 4 | **Fortification du site** : siège seulement, doublement fortifié, unités, gloire | C15, C16, C19, C20, L2, L3 | C16 : Gardes encore cible du siège en phase d'attaque à distance | `combat.rs`, tests | non | fil principal |
| 5 | **Issue** : `SiteConquered`, `HeroWithdrew`, site conquis ordinaire pour le déplacement et les trajets | C21 à C25, L4, L5 | C21 : pas de `SiteConquered` après `CombatEnded` | `engine.rs`, `movement.rs`, `event.rs`, tests | non | fil principal |
| 6 | **Site conquis** : interaction, recrutement par site, `BuySpell` et son mana, bonus de limite de main | C26 à C30, L6, L8 | C27 : `BuySpell` absente des actions légales | `action.rs`, `event.rs`, `engine.rs`, `state.rs`, tests | non | fil principal |
| 7 | **Récompense sort, décompte, sauvegarde** : `TakeSpell`, ordre des choix de fin de tour, Le Conquérant, sauvegarde | C31 à C34 | C31 : la pioche de fin de tour n'attend pas | `engine.rs`, `game_end.rs`, tests arcanum-core et arcanum-persistance | non | fil principal |
| 8 | `FirstChoice` (pas d'assaut), parties préparées `forteresse` et `tour-conquise`, parcours relus, docs du moteur (`architecture.md`, glossaire) | — | — | `tests/premier_choix/`, `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/`, `docs/` | non | fil principal |
| 9 | Libellés `fr.json`, `en.json` (tableau de la spec), `board.hover.fortified` changée | — | — | `assets/i18n/*.json` | oui | Haiku |
| 10 | **Écran** : défenseurs et infobulles, cibles d'assaut en un clic (pastille rouge, trajet rouge), drapeau des sites conquis (SVG par héros, `assets/images/flag/`), marques de combat, retraite, achats de sorts, interaction en forteresse, récompense, limite de main | C35 à C45, C38b | C35 : un défenseur face cachée n'est pas dessiné | `assets/images/flag/`, `arcanum-ui/src/board/`, `arcanum-ui/src/screen/`, `session.rs`, `app/`, `arcanum-ui/tests/` | non | fil principal |
| 11 | Vérification à l'œil (parties `forteresse` et `tour-conquise`), captures | — | — | `docs/verification-manuelle.md` | non | fil principal |
| 12 | Docs : `architecture.md` (écran), `raccourcis.md`, `index-par-notion.md`, `PLAN.md` (implémentée) ; bilan de fin de spec | — | — | `docs/` | oui | fil principal |

## Risques repérés

- **Parcours** : les parcours qui explorent les campagnes 3 et 4 tirent un jeton gris et un violet de plus ; les ennemis errants
  ne changent pas (un générateur par pile, spec 038) mais un parcours qui provoque ou défie près de la forteresse doit être relu.
- **Retour en arrière** : un pas de jour à côté d'un défenseur caché devient un point de non-retour ; les parcours et tests qui
  reviennent en arrière sur un pas près de la forteresse du bac à sable sont à relire.
- **Test de la spec 014 qui change de sens** : L5 (`entering_a_fortified_site_is_not_available_yet`) devient l'assaut pour la
  forteresse et la tour de mage ; à signaler dans la PR, la cité garde le refus.
- **Ordre des actions légales** : `TakeSpell` s'insère entre `TakeArtifact` et `GainAdvancedAction`, `BuySpell` entre
  `BuyAdvancedAction` et `BuyHealing` ; les tests et joueurs choisissent par valeur.
- **Nombre d'actions** : `BuySpell` multiplie sorts et moyens de paiement (3 sorts × dés, cristal, jeton) : au plus une vingtaine
  d'actions, sans gêne.
