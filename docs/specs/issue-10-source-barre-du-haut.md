# #10 — Source dans la barre du haut

- Statut : implémentée (validée le 2026-11-24 ; Q2 à Q5 retenues telles que proposées, Q1 changée à l'implémentation)
- Issue : #10 (Epic #31 Finitions v0.1.0, milestone v0.1.0)
- Dépend de : spec 020 (Source), 022 et 025 (dés pris, dés mis sur une couleur), 060 (Vol de mana, Appel du mana),
  058 (Embrasure de la Source, Magie inutile), 073 (panneau de droite), 074 (barre de menu) ; toutes implémentées
- Livret : aucune règle nouvelle. Rappel : MKUE p. 3 (*Game Setup*, la Source : autant de dés que de joueurs plus 2, soit 3
  en solo)
- Crates touchées : arcanum-ui (dessin seulement, `app/`). Rien dans arcanum-core, ni dans la vue, ni dans les libellés.

## Objectif

La Source de mana quitte le haut du panneau de droite pour la **barre de menu**, juste après les pioches de tuiles : ses
dés se voient d'un coup d'œil, sur le plateau comme en combat. Le panneau de droite commence directement par le joueur, qui gagne la place de la Source. Ce qu'on fait avec les dés
ne change pas.

## Règles

Aucune règle n'est ajoutée ni changée : la Source est publique et l'écran la montrait déjà (spec 020).

- **Dés épuisés** (MKUE p. 5, 3.c ; spec 020) : l'or de nuit, le noir de jour, toujours grisés et à part.
- **Dés pris ce tour** (MKUE p. 5, 3.a et 3.b ; spec 022) : à part, relancés à la fin du tour, comme aujourd'hui.

## Modèle

Aucun terme nouveau : **Source**, **barre de menu**, **dé pris**, **dé épuisé** sont au glossaire.

**Aucune donnée nouvelle.** La ligne de la Source reste calculée comme aujourd'hui dans `GameScreen::source` (`SourceLine`,
`screen/source.rs`) : elle dépend du mode en cours (dés cliquables en mode « mettre un dé sur une couleur », dés cochés pour
Appel du mana), que l'écran connaît et la barre non. Seul le dessin change de place : `menu_bar_panel` reçoit la
`SourceLine` de l'écran de la partie et la dessine dans la barre ; `right_panel_body` ne la dessine plus.

### Écran

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ [Menu ▾] Première reconnaissance  ☀ Manche 1/4 (tour 3)  ⬡ 6 ⬡ 3 │ Source ■ rouge ■ vert pris : ● bleu épuisé : ■ noir   Partie sauvegardée │
├─────────────────────────────────────────────────────────────────────────────┬────────────────────────────┤
│                                                                             │ (bandeaux)                 │
│ Plateau de jeu                                                              │ Joueur                     │
│ ou écran de combat                                                          │ Zone de jeu, ce tour …     │
│                                                                             │ [Offres] [Fin du tour]     │
├─────────────────────────────────────────────────────────────────────────────┴────────────────────────────┤
│ Panneau du bas (spec 073, 077) : inchangé                                                                │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Ordre de la barre, de gauche à droite : Menu | scénario | soleil ou lune, manche | pioches de tuiles | **Source** | à droite,
le message de sauvegarde (Q1).

## Actions et événements

Aucun : ni action, ni événement, ni validation ne change.

## Comportements attendus

Déplacement d'affichage seul : rien de ce que l'écran calcule ne change, donc **aucun test nouveau** (ADR 0006 : un test
vérifie un comportement, pas une mise en page). Tout ce qui suit se vérifie à l'œil.

**Ce qui reste vrai, vérifié contre le code :**

- **C1** — Les dés, leurs groupes (« pris : », « épuisé : »), leurs infobulles et leur ordre sont ceux de
  `GameScreen::source`, inchangée : tests existants de `crates/arcanum-ui/tests/source.rs`, `effets_renforces.rs`,
  `cartes_de_mana.rs`, `tactiques.rs`, `actions_avancees_mana.rs`, `sorts_deplacement.rs`, `competences_second_lot.rs`
  (qui lisent tous `screen.source` et ne bougent pas).
- **C2** — Les dés restent cliquables là où ils le sont aujourd'hui (`SourceDie::pick`) : dé mis sur une couleur (spec 025,
  test de `cartes_de_mana.rs` qui lit `screen.source.dice[1].pick`), Vol de mana (spec 060, `tactiques.rs` ligne 400), Tempête
  de mana (spec 056, `actions_avancees_mana.rs` ligne 452). Le clic rend la même commande, renvoyée par la barre au client
  comme les éléments du Menu.
- **C3** — Appel du mana : les dés cochés restent en bouton sélectionné (`SourceDie::checked`, spec 060).
- **C4** — Embrasure de la Source : le bouton « Garder ce dé tel quel » (`SourceLine::keep`, spec 058, C39) suit les dés pris,
  dans la barre.
- **C5** — Magie inutile à +3 : tous les dés grisés (`SourceDie::forbidden`, spec 058, C6 ;
  `competences_second_lot.rs` ligne 260).

**Ce qui change (vérification à l'œil) :**

- **C6** — Sur le plateau, la Source est dans la barre, après les pioches de tuiles, séparée d'elles par un trait ; elle n'est plus dans le panneau de droite.
- **C7** — En combat (écran de combat ou « Voir le plateau »), même chose : la barre est la même (spec 074, test
  `bar_stays_during_combat`).
- **C8** — Le panneau de droite commence par les bandeaux et le message, puis le joueur (portrait, inventaire…) ; un seul
  trait les sépare, au lieu de deux autour de la Source.
- **C9** — Les dés gardent leur taille (carré de 14 px et nom de la couleur, rond cerclé pour un dé pris) : la barre fait la
  hauteur d'un bouton, qui les contient déjà. Les boutons des dés cliquables prennent le style des boutons de la barre (sans
  cadre au repos, surlignés au survol) ; un dé coché reste surligné.

## Cas limites et refus

- **L1** — Fenêtre étroite (1280 px) ou nom de scénario long : la barre ne passe pas à la ligne ; la Source ne se replie ni
  ne se tronque ; le nom du scénario se tronque toujours en premier (spec 074, L7), la place réservée à droite comptant
  désormais la Source.
- **L2** — Beaucoup de dés (bac à sable à 4 dés, dé de plus d'Embrasure de la Source, tous les groupes présents) : même
  règle que L1. Mesure attendue au pire, 5 dés et les deux titres : environ 450 px, que la barre tient à 1280 px avec le
  message de sauvegarde (à confirmer à l'œil).
- **L3** — Écran de nouvelle partie ouvert en cours de partie (spec 078, Q4) : la barre s'y dessine sans Source, comme le
  plateau et les panneaux qui ne sont pas dessinés.
- **L4** — Partie finie ou arrêtée : la Source reste dans la barre, non cliquable, comme aujourd'hui dans le panneau.

## Information cachée

Rien de nouveau : la Source est publique (`PlayerView::source`, `dice_taken`, `dice_set`, `kept_die`, spec 020). Aucun champ
de la vue n'est ajouté.

## Libellés (i18n)

Aucune clé nouvelle. Clés réutilisées, telles quelles (relues dans `assets/i18n/fr.json` de `main`) :

| Clé | fr | en |
|---|---|---|
| `source.title` | « Source » | « Source » |
| `source.taken` | « pris : » | « taken: » |
| `source.depleted` | « épuisé : » | « depleted: » |

## Interface

- **Source dans la barre de menu**, après les pioches de tuiles (C6, C7 ; `GameScreen::source` ; vérification
  à l'œil).
- **Dés cliquables et cochés** dans la barre (C2, C3, C4, C5 ; `SourceDie::pick`, `checked`, `forbidden`, `SourceLine::keep`).
- **Panneau de droite** sans Source (C8, vérification à l'œil).
- **Fenêtre étroite** : nom du scénario tronqué d'abord (L1, L2, vérification à l'œil à 1280 px).

Parties préparées pour la vérification à l'œil (`docs/verification-manuelle.md`), toutes existantes :

| Partie | Ce qu'on y vérifie |
|---|---|
| `combat-deux-ennemis` | 4 dés dans la barre, sur le plateau puis en combat (C6, C7, C8) ; fenêtre à 1280 px (L1) |
| `tactiques-jour` | Vol de mana : les dés cliquables dans la barre, un clic prend le dé (C2) |
| `actions-avancees-mana` | Tempête de mana sur un dé de la barre (C2) |
| `competences-goldyx` | Embrasure de la Source : au tour suivant, le dé en plus et « Garder ce dé tel quel » dans la barre (C4, L2) |
| `sorts-cibles` | Nuit : la Source dans la barre, le dé noir utilisable (C6) ; jouer un sort renforcé avec le dé noir le range dans « pris : » (C1) |

Aucune partie préparée ne part avec un dé épuisé (or de nuit, noir de jour) : le groupe « épuisé : », inchangé, n'est vu
dans la barre que si une partie de la liste en tire un. Les lignes de `docs/verification-manuelle.md` disent « dans la
Source » ou « près de la Source », jamais « dans le panneau » : elles restent justes.

### Raccourcis clavier

Aucun (voir Q4).

## Hors périmètre

- Le panneau du bas (spec 077) : inchangé.
- Le reste du panneau de droite (joueur, zone de jeu, boutons) : il remonte, rien d'autre ne bouge.
- Une icône de dé dessinée (face de dé, symbole de couleur) à la place du carré et du nom : non prévue ici.

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-11-24) : les propositions de Q2 à Q5 sont retenues telles quelles ; pour Q1, à la
vérification à l'œil, Mathieu a préféré les pioches de tuiles à gauche, la Source à leur droite.

- **Q1. Place dans la barre** (issue, question 1) : entre la manche et les pioches de tuiles, à côté du soleil ou de la lune
  qui explique les dés épuisés (proposition). Autres : après les pioches ; à droite, le message de sauvegarde passant alors
  avant elle. **Décision : après les pioches** (Mathieu, vu à l'œil, 2026-11-24).
- **Q2. Taille et forme des dés** (questions 2 et 3) : taille actuelle, nom de la couleur écrit, groupes « pris : » et
  « épuisé : » sur la même ligne comme dans le panneau (proposition). Autre : carrés seuls, sans le nom (le nom en infobulle),
  pour une barre plus courte ; le daltonien perd alors le nom écrit.
- **Q3. Fenêtre étroite** (question 6) : la Source ne se replie jamais, seul le nom du scénario se tronque (proposition, L1).
  Autre : sous une largeur donnée, la Source se réduit aux carrés sans noms.
- **Q4. Raccourci clavier** (question 8) : aucun (proposition). Prendre un dé ne se fait jamais seul : le dé se choisit dans
  un mode (couleur à payer, Vol de mana, Tempête de mana) dont les fenêtres ont déjà leurs touches, et les chiffres 1 à 6
  servent déjà au choix de la tactique.
- **Q5. En combat** (question 5) et **panneau libéré** (question 7) : la barre étant déjà la même en combat, la Source y est
  pareille ; le panneau de droite ne gagne que la place rendue (proposition). Rien d'autre n'y monte.
