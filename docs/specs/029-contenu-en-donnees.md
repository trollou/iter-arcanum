# 029 — Contenu du jeu en données

- Statut : implémentée (2026-10-25)
- Jalon : J2 (fin), avant J3
- Dépend de : 010 (tuiles), 013 (cartes d'action de base, decks de départ), 022 (effets renforcés, à fusionner avant l'implémentation)
- Livret : aucune règle nouvelle ; le contenu vient des tuiles et des cartes (`docs/reference/tuiles.md`, `docs/reference/decks-de-depart.md`)
- Crates touchées : arcanum-core (nouvelles dépendances `serde` et `ron`, Q1)

## Objectif

Le contenu « plat » du jeu (couleur et effets des cartes, composition des decks de départ, cases des tuiles, puis ennemis et sites) est
décrit dans des **fichiers de contenu** lisibles, au lieu de constantes Rust. Un fichier par famille, seule source de vérité pour le code,
pour Mathieu et pour les fils de travail qui rédigent une spec. Le joueur ne voit aucune différence : la spec est purement technique.

Aujourd'hui, une tuile prend 12 lignes de Rust dans `board/content.rs` et existe aussi en tableau dans `docs/reference/tuiles.md` ; les
cartes sont réparties entre `card.rs` (effets) et `hero.rs` (decks), et les decks sont aussi décrits dans `decks-de-depart.md`. Le
contenu à venir (une cinquantaine d'ennemis, les sites, les unités, les actions avancées, les sorts) ferait grossir ces fichiers Rust
bien plus vite que la logique.

## Ce qui change dans les règles du projet

`CLAUDE.md` dit : « Contenu du jeu écrit en Rust dans arcanum-core, pour être vérifié par le compilateur. Le sortir dans des fichiers de données
seulement si un besoin réel apparaît, avec une spec. » Cette spec est cette demande. Ce qu'on perd, et comment on le compense :

| Avec le contenu en Rust | Avec le contenu en données |
|---|---|
| Une faute de frappe dans un nom de carte ne compile pas. | Le fichier est lu par serde dans un test : un nom inconnu fait échouer la CI, avec le fichier et la ligne. |
| Une carte oubliée est attrapée par un `match` exhaustif. | La validation vérifie que chaque identifiant a exactement une définition (C2, C3). |
| Aucune dépendance. | `serde` (déjà dans le workspace) et `ron` (nouvelle, voir Q1). |

Ce qui ne change pas : **arcanum-core reste sans I/O**. Les fichiers sont embarqués dans le binaire à la compilation (`include_str!`) et lus en
mémoire au premier accès. Aucun fichier n'est ouvert pendant une partie, et l'exécutable n'a besoin d'aucun dossier à côté de lui.

## Modèle

- **Contenu du jeu** (`GameContent`, nouveau terme du glossaire) : l'ensemble des définitions lues depuis les fichiers de contenu.
  « Catalogue » est déjà pris par les libellés (`Catalog`, spec 001), d'où un autre nom.
- **Fichier de contenu** : un fichier texte d'une famille de contenu, dans `crates/arcanum-core/content/` (Q2).
- Les **identifiants restent des enums Rust** (`BasicAction`, `Hero`, `TileId`, puis les ennemis…). Le code continue de les nommer et le
  compilateur de les vérifier ; les fichiers ne décrivent que ce qu'il y a derrière un identifiant.
- Les **mécaniques restent en Rust**. Un fichier choisit parmi les effets que le code sait jouer (`Move(2)`, `Attack(…)`), il n'en invente
  pas. Une mécanique nouvelle = une variante d'enum + son code, apportée par la spec qui en a besoin, puis utilisée dans le fichier (Q5).
- Les **noms affichés restent dans `assets/i18n/`** (ADR 0003) : les fichiers de contenu ne contiennent aucun texte pour le joueur.

Fichiers prévus par cette spec (noms indicatifs) :

| Fichier | Contenu | Remplace |
|---|---|---|
| `basic_actions.ron` | Couleur, effet de base et effet renforcé des 20 cartes d'action de base | `BasicAction::definition` dans `card.rs` |
| `heroes.ron` | Deck standard (16 cartes) et les deux remplacements de chaque héros | `STANDARD_DECK` et `replacements` dans `hero.rs` |
| `tiles.ron` | Les 21 tuiles : case centrale et six côtés | `BASE_GAME_TILES` dans `board/content.rs` |

Les ennemis (spec 030), les sites (041 à 044), les unités (042) et les cartes du J5 seront écrits directement dans de nouveaux fichiers par
leurs specs. Les scénarios restent en Rust : leurs règles spéciales sont du code (points d'extension, spec 012).

### À quoi ressemble un fichier (RON, Q1)

```ron
// tiles.ron — transcription de docs/reference/tuiles.md, vérifiée par Mathieu le 2026-10-24.
// Côtés dans l'ordre E, N-E, N-O, O, S-O, S-E.
[
    (
        id: Countryside(1),
        center: (Forest, Some(Site(MagicalGlade))),
        sides: [
            (Plains, Some(Site(Village))),
            (Lake, None),
            (Forest, Some(Rampaging(OrcMarauders))),
            (Forest, None),
            (Plains, None),
            (Plains, None),
        ],
    ),
]
```

```ron
// basic_actions.ron (extrait ; l'effet renforcé arrive avec la spec 022)
{
    March: (color: Green, basic: Options([Move(2)])),
    Improvisation: (
        color: Red,
        basic: Options([Move(3), Influence(3), Attack(3), Block(3)]),
        cost: DiscardAnotherCard,
    ),
    // Version révisée Lost Legion / Ultimate.
    ColdToughness: (color: Blue, basic: Options([Attack(2), Block(3, Ice)])),
}
```

La même case en JSON, pour comparer : `{"terrain": "Forest", "feature": {"Site": "MagicalGlade"}}`, sans commentaire possible.

La forme exacte (raccourcis comme `Attack(3)` pour une attaque physique de mêlée) se fixe à l'implémentation, tant qu'elle reste lisible
sans connaître le code.

## Actions et événements

Aucun : le contenu n'est pas de l'état de partie. Les événements portent toujours des identifiants, jamais des définitions, donc une
sauvegarde ne dépend pas du format des fichiers.

## Comportements attendus

- **C1** — Les fichiers embarqués se lisent sans erreur et passent la validation (`embedded_content_is_valid`).
- **C2** — Chaque carte d'action de base a exactement une définition ; chaque héros a exactement une entrée
  (`every_basic_action_has_one_definition`, `every_hero_has_one_entry`).
- **C3** — Les 21 tuiles du jeu de base sont définies, chacune une seule fois (`base_game_has_twenty_one_tiles`).
- **C4** — Le deck de départ de chaque héros a 16 cartes et ses remplacements visent des cartes du deck standard
  (`starting_decks_have_sixteen_cards`). MKUE p. 3.
- **C5** — **Non-régression** : tous les tests existants d'arcanum-core et arcanum-ui passent **sans modification**. Ils décrivent déjà les cartes,
  les decks et les tuiles ; c'est eux qui prouvent que la migration n'a rien changé. Toute modification d'un test existant serait signalée
  et justifiée (CLAUDE.md).
- **C6** — L'accès au contenu ne lit aucun fichier : il fonctionne quel que soit le dossier courant. Garanti par `include_str!`, sans
  test dédié (Q3).

## Cas limites et refus

La validation est une fonction pure `GameContent::parse(…) -> Result<GameContent, ContentError>` (erreur typée, `thiserror`), testée sur
des textes de test, pas sur les vrais fichiers :

- **L1** — Syntaxe invalide ou nom inconnu (`Marhc`) → `ContentError::Syntax`, avec le nom du fichier, la ligne et la colonne.
- **L2** — Identifiant sans définition → `ContentError::Missing` qui le nomme.
- **L3** — Identifiant défini deux fois → `ContentError::Duplicate` qui le nomme.
- **L4** — Deck standard qui n'a pas 16 cartes, ou remplacement d'une carte absente du deck standard → `ContentError::InvalidDeck`.
- **L5** — Effet hors des bornes du moteur (option vide, 0 point) → `ContentError::InvalidEffect`.

En jeu, le contenu est chargé une fois dans une variable statique (`LazyLock`). Si la lecture échouait, ce serait une panique : c'est un
invariant justifié en commentaire, car C1 tourne en CI sur les fichiers mêmes qui sont embarqués (Q3).

## Information cachée

Sans objet : le contenu du jeu est public (cartes et tuiles imprimées). Ce qui est caché reste l'ordre des pioches, qui est de l'état.

## Libellés (i18n)

Aucun. Les erreurs de contenu sont des erreurs de développement, jamais montrées au joueur.

## Interface

Aucune.

### Raccourcis clavier

Aucun.

## Migration du contenu existant

1. Après la fusion des PR 022 et 023 (la 022 ajoute l'effet renforcé aux définitions de cartes : migrer avant créerait des conflits).
2. Une PR : tests de la validation (L1 à L5, rouges), puis lecture et validation, puis bascule des trois familles une par une, en gardant
   les fonctions publiques actuelles (`BasicAction::definition`, `Hero::starting_deck`, `tile`, `base_game_tiles`) comme accès au contenu.
   Le moteur et l'UI ne changent pas.
3. Suppression des constantes Rust remplacées.
4. Docs : `tuiles.md` et `decks-de-depart.md` gardent les sources et les notes de lecture, leurs tableaux sont remplacés par un lien vers
   le fichier de contenu (Q4).
5. Mise à jour de `CLAUDE.md` (phrase citée plus haut) et de `.claude/rules/arcanum-core.md` : nouveau contenu = fichier de contenu (Q7).

## Hors périmètre

- Charger des fichiers depuis le disque pendant la partie (mods, rechargement à chaud, contenu des extensions sans recompiler).
- Décrire des mécaniques en données (script, mini-langage d'effets).
- Les scénarios, qui restent en Rust (spec 012).
- Le contenu qui n'existe pas encore : ennemis (030), sites (041 à 044), unités (042), cartes du J5. Leurs specs l'écriront dans des
  fichiers de contenu.
- Un outil d'édition du contenu.

## Décisions

- **Q1 — RON** (Mathieu, 2026-10-25) : plus adapté à un projet entièrement en Rust. Nouvelle dépendance `ron`, avec `serde` (derive) dans arcanum-core.
- **Q2 — `crates/arcanum-core/content/`** (Mathieu, 2026-10-25) : fichiers embarqués à la compilation, arcanum-core reste sans I/O.
- **Q4 — Docs allégées** (Mathieu, 2026-10-25) : `tuiles.md` et `decks-de-depart.md` gardent sources et notes de lecture ; leurs tableaux
  sont remplacés par un lien vers le fichier de contenu.
- **Q3 — Validation** (Mathieu, 2026-10-25) : `parse` renvoie une erreur typée, testée (L1 à L5) ; C1 vérifie les fichiers embarqués
  à chaque CI ; en jeu, la lecture dans `LazyLock` panique si elle échoue, invariant justifié par C1. Pas de test de dossier courant pour
  C6 : `include_str!` le garantit par construction.
- **Q5 — Frontière données / code** (Mathieu, 2026-10-25) : en données, seulement des valeurs (chiffres, couleurs, compositions, choix
  parmi des effets connus) ; toute mécanique et les scénarios restent en Rust.
- **Q6 — Calendrier** (Mathieu, 2026-10-25) : implémentation après la fusion des PR 022 et 023, juste avant la spec 030.
- **Q7 — ADR** (Mathieu, 2026-10-25) : ADR `0005-contenu-en-donnees.md` ; `CLAUDE.md` et `.claude/rules/arcanum-core.md` mis à jour à
  l'implémentation (étape 5 de la migration), quand le contenu passe réellement en données.

## Questions ouvertes

Aucune.
