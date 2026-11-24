# 0005 — Contenu du jeu en données

- Statut : acceptée (2026-10-25), spec `docs/specs/029-contenu-en-donnees.md`
- Remplace la règle de `CLAUDE.md` « Contenu du jeu écrit en Rust dans arcanum-core » (mise à jour à l'implémentation de la spec 029).

## Contexte

Le contenu du jeu (cartes, decks de départ, tuiles, puis ennemis, sites, unités…) était écrit en constantes Rust, et souvent recopié en
tableaux dans `../mage-knight/docs/regles/`. Il va grossir bien plus vite que la logique, et chaque fil de travail doit le relire. On veut une seule
source, lisible sans connaître le code, sans perdre les garanties du compilateur ni la pureté d'arcanum-core.

## Décision

- **Format RON**, lu par serde (dépendances `serde` et `ron` dans arcanum-core). RON écrit les enums comme Rust et accepte les commentaires.
- **Fichiers dans `crates/arcanum-core/content/`**, embarqués à la compilation (`include_str!`). arcanum-core reste sans I/O ; l'exécutable n'a
  besoin d'aucun dossier à côté de lui. Pas de chargement depuis le disque pendant la partie (mods hors périmètre).
- **Identifiants en Rust.** Les enums (`BasicAction`, `Hero`, `TileId`…) restent dans le code ; les fichiers décrivent ce qu'il y a
  derrière chaque identifiant. Les événements et les sauvegardes ne portent que des identifiants.
- **Valeurs en données, mécaniques en Rust.** Un fichier choisit parmi les effets que le code sait jouer ; une mécanique nouvelle est
  une variante d'enum et son code, apportés par une spec. Les scénarios restent en Rust (points d'extension, spec 012).
- **Validation.** `GameContent::parse` renvoie une erreur typée (fichier, ligne, identifiant manquant ou en double…) ; un test vérifie
  les fichiers embarqués à chaque CI. En jeu, une lecture en échec est une panique, invariant justifié par ce test.
- **Libellés** toujours dans `assets/i18n/` (ADR 0003) : aucun texte pour le joueur dans les fichiers de contenu.

## Conséquences

- Une faute dans un fichier de contenu est attrapée par la CI, plus par le compilateur : la validation doit couvrir l'exhaustivité
  (chaque identifiant défini une seule fois) et les références entre fichiers.
- Tout nouveau contenu (ennemis, sites, unités, cartes du J5) est écrit dans un fichier de contenu par la spec qui l'apporte.
- `docs/reference/tuiles.md` et `decks-de-depart.md` gardent leurs sources et notes de lecture, et renvoient aux fichiers de contenu.
