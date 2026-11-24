# 038 — Un générateur par usage

- Statut : implémentée (2026-10-28)
- Jalon : J3 (technique, rien ne change pour le joueur)
- Dépend de : 000 (générateur à graine), 024 (sauvegarde, retour en arrière), 030 (piles d'ennemis)
- Livret : aucun (MKUE p. 3, *Enemy and Ruin token piles*, seulement pour la liste des piles)
- Crates touchées : arcanum-core, arcanum-persistance
- Décision : ADR 0007 « Un générateur par usage » (piste choisie par Mathieu le 2026-10-28) ; issue privée 52

## Objectif

Aujourd'hui, un seul générateur tire tout le hasard de la partie : un tirage ajouté par une spec décale tous les tirages qui le
suivent, et il faut rechoisir la graine des parcours (issue privée 52 : 10 → 73 pour la spec 030). Après cette spec, chaque **usage** du
hasard (pioche de tuiles, Source, deck, chaque pile d'ennemis) a son propre générateur, dérivé de la graine. Une spec qui ajoute un
usage ne change plus rien aux autres, pour la même graine. Rien ne change pour le joueur, sinon que les anciennes sauvegardes ne se
chargent plus.

## Règles

Aucune règle du livret. Ce que la spec fixe :

- **Usages.** Un usage regroupe les tirages qui forment une même suite dans le jeu. Liste des usages de cette spec :

  | Usage | Numéro de flux | Tirages |
  |---|---|---|
  | Pioche de tuiles | 1 | formation de la pioche à la mise en place (spec 012) ; tuile écartée tirée au hasard quand la pioche est vide (spec 015, MKUE p. 7, 5.e) |
  | Source | 2 | lancer de la mise en place et de chaque nouvelle manche (spec 020, 023) ; dés relancés en fin de tour (spec 022) |
  | Deck du joueur *n* | 100 + *n* | mélange du deck à la mise en place et à chaque nouvelle manche (spec 013, 023) |
  | Pile d'ennemis | 200 + couleur | mélange de la pile à la mise en place (spec 030) : vert 200, gris 201, violet 202, marron 203, rouge 204, blanc 205 |

  Les specs à venir ajoutent leurs usages à ce tableau (ruines, offres, artefacts, compétences, joueur fantôme…), avec un numéro
  libre ; un numéro livré ne change jamais (ADR 0007). Les numéros laissent de la place : 100 à 199 pour les usages par joueur, 200 à
  299 pour les piles de jetons.
- **Dérivation** : le générateur d'un usage est ChaCha8 (`GameRng`) initialisé par la graine de la partie, sur le flux du numéro de
  l'usage (`set_stream`). Pas de nouvelle dépendance.
- **Pile par couleur, deck par joueur** (Q1) : chaque pile d'ennemis a son usage, pour qu'un remélange de pile (spec 031) ne décale pas les
  autres piles ; chaque joueur a celui de son deck, pour le jour où il y aura plusieurs joueurs.

## Modèle

Nouveaux termes, ajoutés au glossaire (« Termes techniques du moteur ») :

| Français | Code | Sens |
|---|---|---|
| Usage du hasard | `RngUsage` | Famille de tirages qui partagent un générateur : Source, pioche de tuiles, deck d'un joueur, pile d'ennemis d'une couleur. Chaque usage a un numéro de flux fixe. |
| Générateurs de la partie | `GameRngs` | Un générateur par usage, dérivés de la graine, créés au premier tirage. |
| Positions des générateurs | `rng_positions` | Pour chaque usage déjà tiré, où en est son générateur ; remplace « position du générateur » (`rng_position`). |

Données :

- `GameConfig::rngs()` donne les générateurs de la partie (aucun créé), à la place de `GameConfig::rng()`.
- `new_game` et `decide` prennent les générateurs de la partie au lieu d'un générateur, et tirent chaque hasard dans le générateur de
  son usage.
- La boucle de partie garde les générateurs de la partie ; elle donne et reprend leurs positions (une par usage déjà tiré) au lieu
  d'une seule.
- La sauvegarde écrit `rng_positions` : un objet JSON dont les clés sont les usages en texte (`"tiles"`, `"source"`, `"deck/0"`,
  `"enemy_pile/green"`…) et les valeurs les positions, en texte comme aujourd'hui (entiers de 128 bits).

Exemple de sauvegarde (extrait) :

```
{
  "version": 1,
  "summary": { … },
  "rng_positions": {
    "tiles": "64",
    "source": "48",
    "deck/0": "16",
    "enemy_pile/green": "16",
    …
  },
  "events": [ … ]
}
```

## Actions et événements

Aucune action ni événement nouveau. Les événements gardent leur forme : le hasard y est déjà résolu (ADR 0001). Seul le générateur
dans lequel chaque tirage est pris change.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses).

- **C1** — Étant donné deux usages, quand on tire autant qu'on veut dans l'un, alors la suite de l'autre ne change pas
  (`drawing_in_one_usage_does_not_shift_another`).
- **C2** — Étant donné deux usages différents et la même graine, alors leurs suites diffèrent ; et pour la même graine et le même
  usage, la suite est toujours la même (`each_usage_has_its_own_sequence`).
- **C3** — Les numéros de flux sont ceux du tableau des usages (`usage_streams_are_frozen`). Un renumérotage, qui changerait toutes les
  parties, fait échouer ce test.
- **C4** — Mise en place : la pioche de tuiles, le deck, la Source et chaque pile d'ennemis sont ceux qu'on obtient en tirant dans un
  générateur neuf de leur usage, quel que soit l'ordre des tirages dans `new_game` (`setup_draws_each_usage_from_a_fresh_generator`).
  C'est ce qui garantit qu'un usage ajouté à la mise en place ne décale pas les autres.
- **C5** — Fin du tour : les dés pris sont relancés dans le générateur de la Source ; le deck et les piles d'ennemis n'avancent pas
  (`end_of_turn_rerolls_dice_from_the_source_generator`).
- **C6** — Nouvelle manche : la Source est lancée dans le générateur de la Source, le deck mélangé dans celui du deck ; la pioche de
  tuiles et les piles d'ennemis n'avancent pas (`new_round_draws_source_and_deck_from_their_own_generators`).
- **C7** — Exploration quand la pioche de tuiles est vide : la tuile écartée est tirée dans le générateur de la pioche de tuiles ; la
  Source et le deck n'avancent pas (`removed_tile_is_drawn_from_the_tiles_generator`).
- **C8** — Spec 024 C8, généralisé : une décision qui n'est pas de non-retour ne fait avancer **aucun** générateur
  (`only_no_return_decisions_use_rng`, adapté).
- **C9** — Revenir en arrière puis rejouer donne les mêmes positions des générateurs que sans retour (spec 024 C9, adapté :
  `undo_then_redo_is_transparent`).
- **C10** — Sauvegarde puis reprise : la partie reprise a les mêmes positions des générateurs, et la suite tire les mêmes dés et
  mélanges que la partie d'origine (tests de reprise de la spec 024 et de `arcanum-persistance`, adaptés).
- **C11** — Un usage jamais tiré n'a pas de position dans la sauvegarde ; à la reprise, son générateur part du début de son flux,
  comme il l'aurait fait dans la partie d'origine (`unused_usage_resumes_from_the_start_of_its_stream`).
- **C12** — Parcours J1 et J2 : leurs graines sont rechoisies une dernière fois pour ce changement si besoin (à
  l'implémentation : J2 73 → 13 ; la graine 10 du J1 tient toujours), et le commentaire de la graine du
  parcours J2 ne parle plus de la rechercher à chaque nouveau tirage (parcours de la spec 019 et de la spec 028, au vert).

## Cas limites et refus

- **L1** — Un tirage de plus **dans un usage existant** décale encore la suite de cet usage (une carte qui relance un dé de la Source
  décale les relances suivantes). C'est voulu (ADR 0007, Conséquences) ; le test « graine à rechoisir » des parcours reste en place
  pour ce cas.
- **L2** — Une sauvegarde au format d'avant (champ `rng_position`) est refusée au chargement avec le message existant de sauvegarde
  invalide (`LoadError::Invalid`), sans migration (ADR 0001, révision). La version de fichier reste 1, comme le veut la spec 024.
- **L3** — Une sauvegarde qui cite un usage inconnu (`"rng_positions": { "inconnu": "3" }`) est refusée au chargement
  (`LoadError::Invalid`) : on ne devine pas un usage (`unknown_usage_in_save_is_refused`).
- **L4** — Les tests qui construisent un état à la main avec `ChaCha8Rng::seed_from_u64` (mélanger un deck pour un scénario de règles)
  ne changent pas : ils ne passent pas par le moteur. Ceux qui appellent `decide(…, &mut config.rng())` passent à
  `&mut config.rngs()` ; leurs graines sont vérifiées, et rechoisies si une précondition ne tient plus (signalé dans la PR).

## Information cachée

Aucun changement. Les positions des générateurs ne sont pas dans la vue du joueur ; elles sont dans la sauvegarde, comme la position
d'aujourd'hui (spec 024 : un joueur qui ouvre le fichier peut tricher, accepté en solo).

## Libellés (i18n)

Aucun.

## Interface

Aucune.

### Raccourcis clavier

Aucun.

## Hors périmètre

- L'outil de test qui cherche une graine satisfaisant les préconditions d'un parcours (piste B de l'issue privée 52, écartée par
  l'ADR 0007).
- Les usages des specs à venir (ruines, offres, artefacts, compétences, joueur fantôme, remélange des piles d'ennemis) : chaque spec
  ajoute les siens au tableau.
- Copier les générateurs pour une simulation (MCTS) : avec les joueurs automatiques.

## Questions ouvertes

Aucune.

Tranchée : **Q1** (Mathieu, 2026-10-28) : un générateur par pile d'ennemis (et un par deck de joueur), pas un seul pour toutes les
piles.
