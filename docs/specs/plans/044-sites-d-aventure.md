# Plan de développement — spec 044 Sites d'aventure

- Statut : accepté par Mathieu avec la spec (2026-11-05).
- Spec : [`044-sites-d-aventure.md`](../044-sites-d-aventure.md). Branche : `spec/044-sites-d-aventure`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests en échec et
  leur nombre (`scripts/verifier.sh --rouge`), section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR (celle qui porte la spec) : entrer sans récompense, ou conquérir sans fin de tour qui la donne, laisserait le jeu
bancal.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : plus de 8 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 8 (C1 à C38, persistance, parties préparées, docs du moteur). Il pousse sa dernière étape verte et
   s'arrête.
2. **Fil écran** : étapes 9 à 12 (libellés, C39 à C49, vérification à l'œil, docs de l'écran, bilan), à partir de la spec, de ce
   plan et de la branche.

## Choix de modèle

- **Jetons par case** : `GameState::enemy_tokens : BTreeMap<Hex, Vec<EnemyToken>>` ; `face_down` reste par case (seuls les
  défenseurs des sites fortifiés sont face cachée, un par case). La vue garde la même forme (liste par case).
- **Ennemi du combat** : `CombatEnemy` est retrouvé par son `token` (unique) ; `DeclareAttack { targets: Vec<EnemyToken> }`,
  `DeclareBlock { target }`, `AssignDamage { enemy }` et les événements de combat nomment le jeton. Étape 1, sans comportement
  nouveau sauf C1 : les tests existants changent de forme (à lister dans la PR).
- **Ennemis errants** : `rampaging_enemies` exclut les cases des sites d'aventure (comme les sites fortifiés, spec 043).
- **Site d'aventure du tour** : `PlayerState::adventure : Option<Hex>` posé par `AdventureSiteEntered` ; `burning` et
  `adventure` sur un donjon ou un tombeau excluent les unités par une même fonction (`units_excluded`).
- **Journée du combat** : `GameState::combat_time_of_day(player)` = Nuit si combat en cours et `adventure` sur un donjon ou un
  tombeau, sinon `time_of_day` ; `mana_payments`, `token_paying` et `is_depleted` la lisent à la place de `time_of_day`.
- **Récompenses** : `PlayerState::crystal_rewards : u8` (dés à lancer) et `crystal_choices : u8` (or à choisir) ; `EndTurn` lance
  les dés un à un (`RngUsage::RewardDice`, numéro de flux libre) et s'arrête au premier or, comme l'artefact ; `ChooseCrystalColor`
  reprend la suite (dés restants, artefact, sort, niveau, pioche) en réutilisant le chemin de `TakeArtifact`.
- **Clairière** : champ `glade : Option<WoundPile>` d'`EndTurn` ; `ManaTokenGained` ajouté après chaque `TurnStarted` (fin du
  tour et nouvelle manche) par une fonction commune.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Ennemis par jeton** : liste de jetons par case, cibles et événements de combat par jeton (refactor), tests existants adaptés | C1 | C1 : viser le premier des deux jetons de la case les vise tous les deux (cible par case) | `state.rs`, `combat.rs`, `engine.rs`, `action.rs`, `event.rs`, `view.rs`, tests arcanum-core, arcanum-ui (appels), nouveau `tests/sites_d_aventure.rs` | non | fil principal |
| 2 | **Entrer** : `EnterAdventureSite`, conditions, tirage des jetons, combat aussitôt ; jetons d'aventure pas errants | C2 à C6, C17, C20 | C2 : `EnterAdventureSite` absente des actions légales | `action.rs`, `event.rs`, `engine.rs`, `state.rs`, tests | non | fil principal |
| 3 | **Donjon et tombeau** : sans unités, journée du combat (règles de Nuit) | C7 à C10, L7 | C8 : le jeton or paie encore dans le donjon | `engine.rs`, `combat.rs`, `state.rs`, `view.rs`, tests | non | fil principal |
| 4 | **Issue** : conquête, dé du donjon, défausse ou jetons restés, Sombre Terrier complété, donjon conquis pour la gloire | C11 à C16, C18, C19, L1 à L3, L5 | C11 : pas de `SiteConquered` après le combat du donjon | `engine.rs`, `event.rs`, `rng.rs`, tests | non | fil principal |
| 5 | **Récompenses de fin de tour** : cristaux aléatoires un à un, noir, or et `ChooseCrystalColor`, ordre avant l'artefact | C21 à C26, L6 | C21 : aucun `RewardDieRolled` en fin de tour | `action.rs`, `engine.rs`, `event.rs`, tests | non | fil principal |
| 6 | **Clairière** : `glade` d'`EndTurn`, `WoundThrownAway`, jeton au début du tour | C27 à C33, L8, L9 | C27 : pas d'`EndTurn` avec `glade` sur la clairière | `action.rs`, `engine.rs`, `event.rs`, tests | non | fil principal |
| 7 | **Décompte, sauvegarde, vue** : L'Aventurier, sauvegarde, vue | C34 à C37 | C34 : L'Aventurier vaut 0 | `game_end.rs`, `view.rs`, tests arcanum-core et arcanum-persistance | non | fil principal |
| 8 | `FirstChoice` (C38), parties préparées `donjon` et `sombre-terrier`, parcours relus, docs du moteur (`architecture.md`, glossaire) | C38 | C38 : `FirstChoice` entre dans le donjon | `tests/premier_choix/`, `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/`, `docs/` | non | fil principal |
| 9 | Libellés `fr.json`, `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 10 | **Écran** : bouton et confirmation d'entrée, jetons sur les sites, drapeau, infobulles, marques de combat, récompenses (dé, cristaux, fenêtre de couleur), clairière en fin de tour et au début du tour, fenêtre de fin | C39 à C49 | C39 : pas de bouton « Entrer dans » sur le donjon | `arcanum-ui/src/board/`, `arcanum-ui/src/screen/`, `session.rs`, `app/`, `arcanum-ui/tests/` | non | fil principal |
| 11 | Vérification à l'œil (parties `donjon` et `sombre-terrier`), captures | — | — | `docs/verification-manuelle.md` | non | fil principal |
| 12 | Docs : `architecture.md` (écran), `raccourcis.md`, `index-par-notion.md`, `PLAN.md` (implémentée) ; bilan de fin de spec | — | — | `docs/` | oui | fil principal |

## Risques repérés

- **Étape 1 large** : beaucoup de tests de combat nomment l'ennemi par sa case ; changement mécanique mais étendu (arcanum-core et
  arcanum-ui). Le faire d'un bloc, vert avant l'étape 2, et lister les fichiers de test touchés dans la PR.
- **Parcours** : un tour qui commence sur une clairière gagne un jeton de mana (bac à sable : `(1, -3)`), ce qui ajoute un
  `ManaTokenGained` aux journaux ; les points d'étape qui comptent les jetons sont à relire.
- **Nombre d'actions** : `EndTurn` triple au plus sur une clairière (sans, main, défausse) ; sans gêne.
- **Ordre des actions légales** : `ChooseCrystalColor` en tête des choix de fin de tour, `EnterAdventureSite` après
  `BurnMonastery` ; les tests et joueurs choisissent par valeur.
- **Générateurs** : `RewardDice` est un nouvel usage ; les parcours existants ne lancent aucun dé de récompense, leurs graines ne
  bougent pas (spec 038).
