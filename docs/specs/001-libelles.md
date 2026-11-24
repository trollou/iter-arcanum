# 001 — Libellés

- Statut : implémentée (2026-10-23)
- Jalon : J0
- Dépend de : — (n'utilise pas arcanum-core)
- Livret : aucune règle de jeu (traduit l'ADR 0003)
- Crates touchées : arcanum-i18n (création de la crate), `assets/i18n/` (création des fichiers)

## Objectif

Permettre à l'UI d'afficher n'importe quel texte dans la langue choisie, à partir d'une clé et de paramètres nommés,
sans qu'aucun texte affiché ne soit écrit dans le code. Une clé absente ne fait jamais planter le jeu : elle se voit à l'écran
et elle est relevée. Des tests garantissent que toutes les langues restent cohérentes avec le français, langue de référence.

## Règles

Pas de règle du livret. Comportement fixé par l'ADR 0003 et `.claude/rules/i18n.md` :

- un fichier JSON par langue, `assets/i18n/<langue>.json`, en UTF-8 ; `fr.json` est la référence ;
- clés hiérarchiques en anglais (`card.march.name`), paramètres nommés entre accolades (`{amount}`) ;
- une clé manquante s'affiche `⟦clé⟧` et est relevée, sans planter ;
- toutes les langues ont les mêmes clés et les mêmes paramètres ; toute clé utilisée par le code existe dans `fr.json`.

## Modèle

Nouveaux termes à ajouter au glossaire (vocabulaire technique, ajoutés dans la même PR que cette spec) :

| Français | Code | Sens |
|---|---|---|
| Libellé | `Label` | Texte affiché à l'écran, dans une langue donnée. |
| Clé de libellé | `LabelKey` | Identifiant hiérarchique d'un libellé, ex. `card.march.name`. |
| Langue | `Language` | Code de langue, tiré du nom du fichier : `fr`, `en`… |
| Catalogue | `Catalog` | Ensemble des libellés d'une langue, chargé depuis son fichier. |
| Paramètre | — | Valeur nommée insérée dans un libellé, écrite `{amount}`. |

### Format d'un fichier de langue

Objets JSON imbriqués ; la clé d'un libellé est le chemin des noms joints par des points.

```json
{
  "language": { "name": "Français" },
  "card": {
    "march": {
      "name": "Marche",
      "basic": "Mouvement {amount}"
    }
  }
}
```

Donne les clés `language.name`, `card.march.name` et `card.march.basic`.

- Une **feuille** est forcément une chaîne non vide. Tout autre type (nombre, booléen, tableau, null) est une erreur de chargement.
- Un **segment** de clé et un **nom de paramètre** s'écrivent en minuscules ASCII, chiffres et `_`, et commencent par une lettre.
- Dans une chaîne, `{nom}` est un paramètre ; `{{` et `}}` écrivent une accolade littérale. Toute autre accolade (non fermée, vide, nom invalide) est une erreur de chargement.
- Chaque fichier contient `language.name` : le nom de la langue dans cette langue (« Français », « English »), pour le futur choix de langue.

Le format imbriqué empêche par construction qu'une clé soit à la fois un libellé et un préfixe (`card.march` et `card.march.name`).

### Ce que fournit arcanum-i18n (en mots, le code est libre tant que ce contrat tient)

- Charger un catalogue depuis une chaîne JSON (pur, utilisé par les tests) ou depuis un dossier et une langue (lit `<dossier>/<langue>.json`, chemin construit avec `PathBuf`). Erreur typée en cas d'échec : fichier illisible, JSON invalide, feuille qui n'est pas une chaîne non vide, nom de clé invalide, accolade mal formée, clé en double.
- Lister les langues disponibles dans un dossier : les noms des fichiers `*.json`, triés.
- Résoudre une clé sans paramètre → le libellé.
- Résoudre une clé avec des paramètres (couples nom → texte déjà formaté par l'appelant) → le libellé avec les paramètres remplacés.
- Donner l'ensemble des clés d'un catalogue et, pour chaque clé, l'ensemble de ses paramètres (sert aux tests de cohérence).
- Donner la liste des manques relevés depuis le chargement, sans doublon : clés manquantes, et paramètres non fournis (clé + nom du paramètre).

La résolution ne renvoie jamais d'erreur : elle renvoie toujours un texte affichable.

## Actions et événements

Aucun : arcanum-i18n n'est pas un moteur de règles.

## Comportements attendus

Les tests de chargement et de résolution utilisent des JSON écrits dans le test, pas les vrais fichiers de `assets/i18n/`.

- **C1** — Étant donné un JSON imbriqué `{"card": {"march": {"name": "Marche"}}}`, quand on résout `card.march.name`, alors on obtient « Marche » (`nested_keys_resolve_by_dotted_path`).
- **C2** — Étant donné le libellé `"Mouvement {amount}"`, quand on le résout avec `amount = "2"`, alors on obtient « Mouvement 2 » (`named_parameter_is_substituted`).
- **C3** — Un libellé avec deux paramètres, fournis dans l'ordre inverse de leur apparition, est correctement rempli ; un même paramètre présent deux fois est remplacé aux deux endroits (`parameters_are_matched_by_name`).
- **C4** — `{{` et `}}` s'affichent `{` et `}` et ne sont pas des paramètres (`double_braces_are_literal`).
- **C5** — Quand on résout une clé absente du catalogue, on obtient `⟦la.cle⟧`, et la clé apparaît dans les clés manquantes (`missing_key_is_visible_and_recorded`).
- **C6** — Une même clé manquante résolue plusieurs fois n'apparaît qu'une fois dans les clés manquantes (`missing_key_is_recorded_once`).
- **C7** — Résoudre une clé qui existe mais dont un paramètre n'est pas fourni laisse `{nom}` visible dans le texte et relève le couple clé + paramètre (`missing_parameter_stays_visible`).
- **C8** — Un paramètre fourni mais absent du libellé est ignoré (`extra_parameter_is_ignored`).
- **C9** — Le catalogue donne, pour chaque clé, l'ensemble de ses paramètres (`catalog_lists_keys_and_parameters`).
- **C10** — Charger depuis un dossier et une langue lit `<dossier>/<langue>.json` ; lister les langues d'un dossier de test contenant `en.json`, `fr.json` et `notes.txt` donne `[en, fr]` (`load_from_directory_and_list_languages`).

Tests sur les vrais fichiers `assets/i18n/` (le dossier est trouvé à partir de `CARGO_MANIFEST_DIR`, sans séparateur en dur) :

- **C11** — `fr.json` existe et se charge sans erreur, ainsi que chaque autre fichier de langue (`all_language_files_load`).
- **C12** — Chaque langue a exactement les mêmes clés que `fr.json` ; en cas d'échec, le message liste les clés en trop et en moins, par langue (`all_languages_have_the_same_keys`).
- **C13** — Pour chaque clé, chaque langue a exactement les mêmes paramètres que `fr.json` (`all_languages_have_the_same_parameters`).

Contenu initial : `fr.json` et `en.json` avec seulement `language.name` (« Français », « English »). Les clés de l'UI arrivent avec la spec 002.

## Cas limites et refus

- **L1** — Une feuille qui n'est pas une chaîne (nombre, booléen, tableau, null) : erreur de chargement qui nomme la clé (`non_string_leaf_is_refused`).
- **L2** — Une chaîne vide : erreur de chargement qui nomme la clé, car un libellé vide est presque toujours un oubli (`empty_label_is_refused`).
- **L3** — Un segment de clé ou un nom de paramètre invalide (majuscule, espace, tiret, commence par un chiffre) : erreur de chargement (`invalid_key_or_parameter_name_is_refused`).
- **L4** — Une accolade mal formée (`{amount`, `{}`, `}` seul) : erreur de chargement qui nomme la clé (`malformed_braces_are_refused`).
- **L5** — Un JSON invalide ou une racine qui n'est pas un objet : erreur de chargement (`invalid_json_is_refused`).
- **L6** — Une clé présente deux fois dans le même objet JSON : erreur de chargement plutôt que garder silencieusement la dernière (`duplicate_key_is_refused`).
- **L7** — Un fichier de langue absent du dossier : erreur typée « fichier introuvable », pas de panique (`missing_language_file_is_a_typed_error`).
- **L8** — Un fichier qui n'est pas de l'UTF-8 valide : erreur de chargement (`non_utf8_file_is_refused`).

## Information cachée

Sans objet.

## Libellés (i18n)

`language.name` dans `fr.json` et `en.json`. Aucune autre clé.

## Interface

Aucune. L'UI utilisera arcanum-i18n à partir de la spec 002 (qui décidera où l'exécutable trouve `assets/i18n/`).

## Hors périmètre

- **Pluriels et accords** (« 1 point », « 2 points ») : pas de mécanisme pour l'instant. On rédige les libellés pour les éviter, à la manière des cartes (« Mouvement {amount} »). Une spec dédiée si le besoin devient réel.
- **Repli sur le français** quand une clé manque dans une autre langue : inutile tant que le test C12 garantit les mêmes clés ; une clé manquante reste visible `⟦clé⟧`.
- **Choix de la langue**, préférence enregistrée, écran de sélection : plus tard (après 002), avec sa spec.
- **Format des nombres** selon la langue : l'appelant fournit des textes déjà formatés ; Mage Knight n'utilise que des entiers.
- **Test « toute clé utilisée par le code existe dans `fr.json` »** : arcanum-i18n fournit de quoi le faire (ensemble des clés), mais le test vit dans chaque crate qui utilise des clés, dès qu'elle en a (spec 002 pour arcanum-ui). Voir la décision 3.
- **Police** : vérifier que la police de egui affiche bien `⟦` et `⟧` relève de la spec 002.
- **Langues de droite à gauche**, rechargement à chaud des fichiers.

## Décisions prises à la validation

1. Dépendances acceptées : `serde` et `serde_json` (`serde` en direct pour refuser les clés en double, L6).
2. Pas de bibliothèque de logs dans cette spec : arcanum-i18n relève les manques dans le catalogue (C5 à C7). Leur écriture dans les logs viendra avec la spec 003 Logs techniques.
3. Clés utilisées par le code : chaque crate qui emploie des clés expose la liste de celles qu'elle peut demander, et un test de cette crate vérifie qu'elles existent dans `fr.json`.
4. Fichiers de langue en objets JSON imbriqués.

## Questions ouvertes

Aucune.
