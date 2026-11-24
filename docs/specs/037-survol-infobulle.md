# 037 — Survol du plateau près du curseur

- Statut : implémentée (2026-10-28)
- Jalon : J3 (confort d'interface, issue privée 50)
- Dépend de : 011 (affichage, survol), 014, 015, 017, 021, 030 (lignes de survol)
- Livret : aucun (interface seulement)
- Crates touchées : arcanum-ui, `assets/i18n/` (fr.json et en.json)
- Origine : issue privée 50, constatée à la vérification à l'œil de la spec 030 ; choix de Mathieu (2026-10-28) : une infobulle près du
  curseur plutôt qu'une zone fixe en haut du panneau.

## Objectif

Aujourd'hui, les lignes de survol d'une case (tuile, terrain, coût, jeton d'ennemi, « Provoque : … ») sont écrites **tout en bas du
panneau latéral**, après la main, la zone de jeu et les boutons. Sur un écran de 1600 × 900, dès que la main compte quelques cartes,
elles sont hors de vue, et faire défiler le panneau déplace la souris hors de la case. Or, depuis la spec 030, le survol porte une
information importante : les valeurs d'un ennemi, ou le fait qu'un pas le provoque.

Avec cette spec, ces lignes s'affichent dans une **infobulle à côté du curseur**, sur le plateau de jeu, et suivent la case survolée.
Le panneau latéral ne les montre plus.

## Règles

Aucune règle du jeu. Le **contenu** du survol ne change pas : ce sont les mêmes lignes, dans le même ordre, que celles fixées par les
specs 011 (tuile, terrain, site, héros, coordonnées), 014 (coût, clic), 015 (emplacement libre, exploration), 017 (trajet), 021
(mine) et 030 (jeton, provocation). Seul l'**endroit** où elles s'affichent change, plus le cas « rien sous le curseur » (C3).

## Modèle

Termes du glossaire déjà présents : Case, Plateau de jeu, Emplacement libre, Zone interdite, Jeton d'ennemi.

Nouveau terme d'interface, ajouté au glossaire dans la PR de cette spec (à valider) :

| Français | Code | Sens |
|---|---|---|
| Infobulle de survol | `hover_lines` (inchangé) | Les lignes qui décrivent ce qu'il y a sous le curseur sur le plateau de jeu, affichées à côté de lui. Remplace le « panneau de survol » de la spec 011. Pas un terme du livret. |

Données : aucune dans l'état de la partie. L'écran du plateau (`BoardScreen`, calculé sans egui) garde ses lignes de survol ; elles
sont **vides** quand il n'y a rien à décrire (C3), et l'application n'affiche alors pas d'infobulle. Le champ `hover_lines` de
l'application, qui gardait les lignes de l'image précédente parce que le panneau était dessiné avant le plateau, disparaît :
l'infobulle est dessinée **après** le plateau, avec les lignes de l'image en cours.

## Actions et événements

Aucun. Le survol est de l'état de l'UI (spec 011) : il ne passe ni par le moteur ni par le journal.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses), sur l'écran du plateau calculé sans egui, comme les tests de
survol existants (ADR 0006 : pas de test d'interaction egui).

- **C1** — Curseur sur une case posée : les lignes de survol sont celles d'aujourd'hui, inchangées (les tests de survol des specs 011,
  014, 015, 017, 021 et 030 restent vrais tels quels) (`hover_lines_are_unchanged_on_a_placed_space`, couvert par les tests existants).
- **C2** — Curseur sur une case de la zone interdite ou d'un emplacement libre : les lignes d'aujourd'hui (« Au-delà de la côte… »,
  « Emplacement libre », exploration) (`hover_lines_describe_forbidden_and_free_positions`, couvert par les tests existants).
- **C3** — Curseur hors de toute case décrite (case sans tuile qui n'est ni zone interdite ni emplacement libre, curseur hors de la zone
  du plateau, pas de curseur) : **aucune** ligne, donc pas d'infobulle. Remplace « Survolez une case » (`hovering_nothing_shows_no_lines`,
  remplace `hovering_nothing_shows_placeholder` de la spec 011, C26 ; voir « Effets sur les specs déjà implémentées »).
- **C4** — La clé `board.hover.none` n'existe plus dans `fr.json` ni dans `en.json`, et aucune ligne de survol calculée pour chaque
  case du bac à sable ne contient de clé manquante (le test de cohérence entre langues de la spec 001 et les balayages 011 C27, 014
  C21, 015 C22 le vérifient déjà).

Vérification à l'œil (`scripts/verif-manuelle.sh`, Linux sous Xvfb, écran de **1600 × 900**, main de départ complète) :

- **V1** — Survol d'une case voisine : l'infobulle apparaît **tout de suite** à côté du curseur (en bas à droite par défaut), avec le coût
  et « Cliquez pour vous y déplacer » ; le panneau latéral n'a plus de lignes de survol.
- **V2** — Survol d'un jeton d'ennemi et d'une case qui le provoque : nom, valeurs et « Provoque : … » lisibles sans faire défiler.
- **V3** — Déplacer la souris de case en case : l'infobulle suit le curseur et change de contenu à chaque case, sans retard d'une image.
- **V4** — Curseur près du bord droit, puis du bord bas de la fenêtre : l'infobulle reste entière dans la fenêtre et ne cache pas le
  curseur.
- **V5** — Curseur hors des tuiles (fond gris) : pas d'infobulle.

Captures : `spec-037-voisine.png`, `spec-037-jeton.png`, `spec-037-bord.png`.

Faite le 2026-10-28 (1600 × 900, main de départ complète) : `spec-037-debut.png` (colline lointaine : coût et points manquants,
panneau sans lignes de survol), `spec-037-jeton.png` (Chevaucheurs de loups : valeurs, capacités, une ligne longue passée à la ligne),
`spec-037-voisine.png` (après Marche : « Cliquez pour vous y déplacer » et tracé du trajet visible), `spec-037-bord.png` (curseur en
bas à droite du plateau : l'infobulle remonte au-dessus du curseur, reste dans la fenêtre et déborde sur le panneau),
`spec-037-fond.png` (emplacement libre près du bord droit). En partie, les cases autour des tuiles sont toutes zone interdite ou
emplacement libre : V5 (fond sans rien) n'est couvert que par le test C3. L1 (glissement) n'a pas été rejoué sous Xvfb.

## Cas limites et refus

- **L1** — **Glisser** le plateau (bouton gauche ou milieu enfoncé) : pas d'infobulle tant que le glissement dure ; elle revient au
  relâchement (vérification à l'œil).
- **L2** — Défilement au clavier (flèches) ou zoom (molette, `+`/`-`) sans bouger la souris : la case sous le curseur change, et
  l'infobulle avec elle (vérification à l'œil).
- **L3** — Fenêtre de chargement (spec 024) ouverte au-dessus du plateau : pas d'infobulle quand le curseur est sur la fenêtre.
- **L4** — Curseur sur le panneau latéral : pas d'infobulle de plateau (les infobulles des boutons et des dés, déjà là, ne changent
  pas).
- **L5** — Mode fin du tour, mode choix (défausse, mana, dés) : l'infobulle s'affiche comme aujourd'hui le panneau, avec les lignes que
  ces modes donnent déjà (sans coût de déplacement en mode fin du tour, spec 016).
- **L6** — Infobulle longue (jeton avec capacités et résistances, coordonnées en développement) : les lignes ne sont pas coupées ; une
  ligne trop large passe à la ligne, à une largeur maximale d'environ 320 points.

## Information cachée

Aucun changement : l'infobulle montre ce que montrait le panneau, calculé sur la vue du joueur (spec 012).

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `board.hover.none` | — | *supprimée* | *supprimée* |

Aucune nouvelle clé.

## Interface

```
 Avant (1600 × 900) :                         Après :

 ┌──────────────────────────┬──────────┐      ┌──────────────────────────┬──────────┐
 │                          │ Main     │      │                          │ Main     │
 │      plateau de jeu      │ …        │      │      ⬡ ↖ curseur         │ …        │
 │                          │ Zone de  │      │        ┌──────────────┐  │ Zone de  │
 │         ⬡ ↖ curseur      │ jeu …    │      │        │ Tuile de     │  │ jeu …    │
 │                          │ Boutons  │      │        │ campagne 2   │  │ Boutons  │
 │                          │ Recentrer│      │        │ Plaine       │  │ Recentrer│
 │                          ├──────────┤      │        │ Coût : 2     │  │          │
 │                          │ (survol, │      │        │ Provoque :   │  │          │
 │                          │ hors vue)│      │        │ Rôdeurs      │  │          │
 └──────────────────────────┴──────────┘      └────────┴──────────────┴──┴──────────┘
```

- **Infobulle** : une ligne par ligne de survol, dans le style des infobulles d'egui déjà utilisées pour les boutons et les dés (fond
  sombre, bord fin), sans titre ni mise en forme nouvelle. Elle s'affiche **sans délai** et reste affichée **pendant que la souris
  bouge** (les infobulles d'egui attendent par défaut que la souris s'arrête : on ne veut pas de ce délai sur le plateau, qu'on parcourt
  des yeux en bougeant).
- **Position** : à côté du curseur, en bas à droite ; près d'un bord de la fenêtre, elle est ramenée dedans sans recouvrir le curseur
  (placement d'egui pour les infobulles au pointeur). Elle cache en partie les cases voisines du curseur : c'est accepté, la case
  survolée elle-même reste visible avec son contour clair (spec 011) et le tracé du trajet (spec 017).
- **Panneau latéral** : les lignes de survol disparaissent ; le bouton « Recentrer » reste à sa place.

### Raccourcis clavier

Aucun. L'infobulle suit la souris ; pas de touche pour la masquer (Q1).

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens d'un test existant change (règle « jamais modifier un test sans le signaler ») :

- **011**, `hovering_nothing_shows_placeholder` (C26) : attendait « Survolez une case » ; devient `hovering_nothing_shows_no_lines`
  (C3), qui attend une liste vide. La section « Survol » de la spec 011 (panneau de survol, `board.hover.none`) et sa disposition de
  la fenêtre sont mises à jour dans la PR d'implémentation, avec un renvoi à cette spec.
- **012**, `hovering_the_forbidden_zone_says_so` (C20) : sa dernière vérification, un emplacement permis hors partie, attendait
  « Survolez une case » ; elle attend maintenant une liste vide (C3). La partie sur la zone interdite ne change pas.
- **015** : la note « une case vide reste « Survolez une case » » (Hors périmètre) devient « une case vide n'a pas de ligne de survol ».
- Les autres tests de survol (011, 014, 015, 017, 021, 030) ne changent pas : ils vérifient les lignes, pas l'endroit où elles
  s'affichent.
- `docs/architecture.md` : la ligne `board/` parle déjà du survol ; préciser qu'il s'affiche en infobulle, dessinée par `app.rs` après
  le plateau.

## Hors périmètre

- Une **mise en forme** de l'infobulle (titre en gras, icônes de mana, couleurs d'élément, valeurs d'ennemi en tableau) : à proposer
  avec la spec du combat (031) ou des capacités (033), quand l'infobulle d'un ennemi s'allongera.
- L'infobulle des **cartes** en main ou dans la zone de jeu : le texte des cartes est déjà affiché dans le panneau.
- Une option pour revenir à l'ancien affichage dans le panneau.

## Questions ouvertes

Aucune. Réponses de Mathieu (2026-10-28) :

1. **Q1** — Raccourci : aucune touche pour masquer l'infobulle.
2. **Q2** — Délai : affichage immédiat et pendant que la souris bouge, comme proposé ; à revoir à l'usage.
