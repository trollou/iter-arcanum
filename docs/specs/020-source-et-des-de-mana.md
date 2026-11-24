# 020 — Source et dés de mana

- Statut : implémentée (2026-10-25)
- Jalon : J2
- Dépend de : 012 (mise en place), 014 (Jour/Nuit), 016 (fin du tour)
- Livret : MKUE p. 3 (« Day/Night board » : la Source à la mise en place), p. 4 (2.b : réinitialiser la Source), p. 5 (« Using
  mana » 1 à 3), p. 6 (8.a) et p. 11 (« End of the Turn » 1 : rendre et relancer les dés utilisés)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

La partie commence avec une **Source** : trois **dés de mana** lancés à la mise en place, en respectant la règle « au moins la moitié
de couleurs de base ». Le joueur voit la Source dans le panneau latéral : la couleur de chaque dé, et les dés **épuisés** (noir de
jour, or de nuit) mis à part. Les dés ne servent encore à rien : on les prendra pour payer un effet renforcé à la spec 022 (Q1).

## Règles

- **Couleurs de mana** (MKUE p. 5, « Using mana » 1 et 2) : quatre couleurs de base, plus l'**or** et le **noir**, qui n'existent qu'en
  mana fondamental (dés et jetons), jamais en cristal.
- **Le dé** : six faces, une par couleur (scans `docs/regles/scans/misc/die_*.png`, confirmé sur le dé physique, Q4) : une chance sur six
  chacune.
- **Nombre de dés** (MKUE p. 3) : joueurs réels plus 2, donc **3 en solo** ; le joueur fantôme (spec 061) n'est pas un joueur réel. Les
  autres dés restent dans la réserve (« Bank ») : rien à modéliser tant qu'aucun effet ne fait lancer un dé.
- **Au moins la moitié de couleurs de base** (MKUE p. 3) : avec 3 dés, au moins **2** (1,5 arrondi au-dessus). Sinon on relance
  **ensemble tous les dés or et noir**, jusqu'à respecter la règle ; les dés de couleur de base ne sont jamais relancés.
- **Dés épuisés** (MKUE p. 5, 3.c) : le noir de jour, l'or de nuit. C'est une conséquence de la couleur et du moment de la journée, pas un
  choix : le moteur la calcule, il ne la stocke pas.
- **Or et noir** (MKUE p. 5, 2.a et 2.b) : cette spec ne fait que l'**afficher** (info-bulle) ; l'usage est la spec 022.
- **Prendre un dé** (MKUE p. 5, 3 à 3.b ; p. 6, 8.a ; p. 11, 1). Prendre un dé sans utiliser son mana est interdit (3.b) : tant que rien
  ne consomme de mana, la prise et la relance de fin de tour n'ont pas de sens et arrivent avec le premier usage, la spec 022 (Q1).
- **Réinitialiser la Source** (MKUE p. 4, 2.b) : spec 023, qui réutilisera telle quelle la fonction de lancer de cette spec.

### Exemple : un lancer de mise en place, en solo, de jour

```
Lancer          ⬛ noir    🟨 or      🟦 bleu      couleurs de base : 1 sur 3 < 2  → relancer noir et or, ensemble
Relance 1       🟨 or      ⬛ noir    🟦 bleu      couleurs de base : 1 sur 3 < 2  → relancer encore les deux
Relance 2       🟩 vert    ⬛ noir    🟦 bleu      couleurs de base : 2 sur 3      → la Source est prête
                                                 de jour, le noir est épuisé

Source affichée :   🟩 vert   🟦 bleu   │ épuisé : ⬛ noir
```

Le bleu n'est jamais relancé. Seul le résultat final entre dans le journal (l'événement porte le hasard résolu, ADR 0001).

## Modèle

### Termes

Termes du glossaire déjà présents : Source (`Source`), Dé de mana (`ManaDie`), Rouge / Bleu / Blanc / Vert (`BasicColor`), Jour / Nuit
(`TimeOfDay`), Cristal (`Crystal`), Mise en place (`setup`).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider, Q2) :

| Livret (EN) | Français | Code |
|---|---|---|
| Mana color | Couleur de mana | `ManaColor` (`Basic(BasicColor)`, `Gold`, `Black`) |
| Gold (mana) | Or | `ManaColor::Gold` |
| Black (mana) | Noir | `ManaColor::Black` |
| Pure mana | Mana fondamental | — (spec 021 et 022) |
| Roll (the dice) | Lancer | `SourceRolled` (événement) |
| Reroll | Relancer | — (même événement : seul le résultat compte) |
| Depleted (die) | Épuisé | `ManaColor::is_depleted(TimeOfDay)` |
| Reset the Source | Réinitialiser la Source | — (spec 023) |

### Nouvelles données dans l'état

- la **Source** : la couleur de chaque dé, dans l'ordre du lancer (dé 1, dé 2, dé 3). Publique. Vide avant la mise en place.

Le nombre de dés se calcule (joueurs réels + 2) ; l'épuisement se calcule (couleur + moment de la journée). Rien d'autre n'est stocké.

## Actions et événements

Aucune action nouvelle.

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `SourceRolled { dice }`, `dice` = couleur de chaque dé après les relances | remplace la Source par `dice` | oui, publique |

**Mise en place.** `new_game` lance la Source et ajoute `SourceRolled` **à la fin** des événements de démarrage, après les `CardDrawn`
de la main de départ. Le générateur est utilisé **après** le mélange du deck : pour une graine donnée, la pioche de tuiles et le deck
restent ceux d'aujourd'hui (l'ordre d'usage du générateur fait partie du contrat de la spec 012).

**Lancer.** Fonction pure d'arcanum-core : lancer `n` dés (une face au hasard parmi les six, dé après dé, dans l'ordre), puis, tant que
moins de la moitié (arrondie au-dessus) montre une couleur de base, relancer chaque dé or ou noir, dans l'ordre des dés. Cet ordre
d'usage du générateur fait partie du contrat (tests déterministes).

**Actions légales et décision en attente** : inchangées.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable avec Tovak.

Lancer (arcanum-core, fonction pure) :

- **C1** — Sur 10 000 lancers d'un dé seul (sans la règle de la moitié), chacune des six couleurs sort, chacune entre 15 % et 18,5 %
  des fois (`a_die_has_six_equally_likely_faces`) — MKUE p. 5, 1 et 2.
- **C2** — Pour 1 000 graines, un lancer de 3 dés montre au moins 2 couleurs de base (`at_least_half_of_the_dice_show_basic_colors`)
  — MKUE p. 3.
- **C3** — Avec un générateur dont on connaît les tirages (graine trouvée par le test et figée), un premier lancer avec un seul dé de
  couleur de base : ce dé garde sa couleur, seuls les dés or et noir changent (`only_gold_and_black_dice_are_rerolled`) — MKUE p. 3.
- **C4** — Même graine, même lancer (`rolling_is_deterministic`).
- **C5** — Règle de la moitié pour 1 à 5 dés : il faut 1, 1, 2, 2, 3 dés de couleur de base (`half_is_rounded_up`) — MKUE p. 3.

Mise en place (arcanum-core) :

- **C6** — Après `new_game` en solo, la Source a 3 dés et le dernier événement de démarrage est `SourceRolled`
  (`setup_rolls_three_dice_in_solo`) — MKUE p. 3.
- **C7** — Pour une graine donnée, la pioche de tuiles, l'ordre du deck et la main de départ sont les mêmes qu'avant cette spec
  (valeurs relevées sur `main` avant l'implémentation) (`rolling_the_source_does_not_change_earlier_setup`) — spec 012.
- **C8** — De jour, un dé noir est épuisé, un dé or ne l'est pas ; de nuit, l'inverse ; un dé de couleur de base ne l'est jamais
  (`black_is_depleted_by_day_and_gold_by_night`) — MKUE p. 5, 3.c.

Information cachée et journal (arcanum-core) :

- **C9** — La vue joueur contient la Source, dans l'ordre du lancer ; le journal filtré montre `SourceRolled` en entier
  (`source_is_public`).
- **C10** — Rejouer le journal redonne la même Source (`replaying_log_rebuilds_source`).
- **C11** — La Source ne change pas pendant un tour, ni à la fin du tour : jouer des cartes, se déplacer, explorer, finir le tour
  (`source_is_unchanged_by_turns`) — rien ne prend de dé avant la spec 022.

Affichage (arcanum-ui) :

- **C12** — Le panneau latéral affiche une ligne « Source », sous « Jour · Tour 1 » : un dé par pastille de sa couleur, les dés
  utilisables d'abord dans l'ordre du lancer, puis « épuisé : » et les dés épuisés (`panel_shows_source_dice`).
- **C13** — Survoler un dé affiche son info-bulle : couleur de base « Dé de mana bleu » ; or de jour « Dé de mana or : peut servir de
  mana de n'importe quelle couleur de base, de jour. » ; noir de jour « Dé de mana noir : épuisé, inutilisable de jour. »
  (`source_die_tooltips`).
- **C14** — Avec le vrai `fr.json`, calculer l'écran avec une Source de chacune des six couleurs, de jour et de nuit (état construit),
  ne relève aucun manque (`every_source_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux montre trois dés dans le panneau ; d'une graine à l'autre, jamais plus d'un dé
or ou noir ; un dé noir est rangé à part, marqué épuisé.

## Cas limites et refus

- **L1** — Trois dés or ou noir au premier lancer : on relance les trois ensemble, autant de fois qu'il faut (couvert par C2 et C3).
- **L2** — La boucle de relance s'arrête : avec 3 dés, une relance réussit au moins 74 fois sur 100 (elle échoue avec probabilité
  1/9 s'il reste deux dés or ou noir, 7/27 s'il en reste trois) ; dix échecs de suite arrivent moins d'une fois sur 700 000. Pas de
  borne artificielle : une boucle sans limite, justifiée en commentaire.
- **L3** — Nuit : impossible avant la spec 023 (la partie commence de jour). C8 et C14 l'exercent sur un état construit.
- **L4** — `apply` d'un `SourceRolled` sur une Source déjà lancée la remplace entièrement (réinitialisation de la spec 023)
  (`source_rolled_replaces_the_source`).

## Information cachée

- Publique : la Source (couleur de chaque dé). Il n'y a rien de caché : un dé de la Source se voit sur la table.
- Le générateur et les tirages intermédiaires (relances) ne sont dans aucune vue ni aucun événement.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `source.title` | — | Source | Source |
| `source.depleted` | — | épuisé : | depleted: |
| `mana.red` | — | rouge | red |
| `mana.blue` | — | bleu | blue |
| `mana.white` | — | blanc | white |
| `mana.green` | — | vert | green |
| `mana.gold` | — | or | gold |
| `mana.black` | — | noir | black |
| `source.die.basic` | `color` | Dé de mana {color} | {color} mana die |
| `source.die.gold_day` | — | Dé de mana or : peut servir de mana de n'importe quelle couleur de base, de jour. | Gold mana die: can be used as mana of any basic color, during the Day. |
| `source.die.black_night` | — | Dé de mana noir : alimente certains effets, de nuit. | Black mana die: powers some effects, at Night. |
| `source.die.depleted` | `color`, `time_of_day_lower` | Dé de mana {color} : épuisé, inutilisable {time_of_day_lower}. | {color} mana die: depleted, unusable {time_of_day_lower}. |

| `source.by_day` | — | de jour | during the Day |
| `source.by_night` | — | de nuit | at Night |

Pour `source.die.depleted`, le paramètre `time_of_day_lower` vaut `source.by_day` ou `source.by_night`. Les couleurs sont au masculin (« dé ») ; si un autre nom en a besoin plus tard
(« mana », « cristal »), on ajoutera des clés, pas d'accord calculé.

## Interface

- **Panneau latéral** : sous « Jour · Tour n », une ligne **Source** avec un petit carré arrondi par dé, rempli de la couleur du dé
  (mêmes teintes que les pastilles des cartes pour les quatre couleurs de base ; or et noir ajoutés). Les dés épuisés viennent après
  « épuisé : », grisés. Visuels provisoires, dessinés par nous.
- **Info-bulle** au survol d'un dé (C13).
- Pas de clic sur un dé : prendre un dé arrive avec la spec 022.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Bac à sable                                       │
│ Jour · Tour 1                                     │
│ Source :  ▣ vert  ▣ bleu   épuisé : ▢ noir        │
│ Mouvement : 0 · Influence : 0                     │
│ Main                                              │
│ ● Marche          Jouer : Mouvement 2   …         │
│ …                                                 │
└───────────────────────────────────────────────────┘
```

### Raccourcis clavier

Aucun : la Source se regarde, elle ne se manipule pas encore (Q5).

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **012** : `setup_events_come_in_order` gagne `SourceRolled` à la fin de la liste des événements de démarrage, et sa
  correspondance exhaustive sur les événements gagne une branche.
- **002, 011, 012, 014, 016** : `PlayerView`, construit champ par champ dans des tests, gagne la Source.
- Les tests qui vérifient le nombre exact d'événements de démarrage (s'il y en a) gagnent 1.

## Hors périmètre

- Prendre un dé, un par tour, l'utiliser tout de suite, relancer et rendre les dés utilisés en fin de tour : spec 022, avec le premier
  usage du mana (Q1).
- Or comme couleur de base de jour, noir de nuit : spec 022.
- Cristaux et jetons de mana (mana fondamental hors des dés) : spec 021. Le terme français de la « Bank » du livret (réserve commune des cristaux et des dés inutilisés, p. 3) y sera choisi.
- Réinitialiser la Source au début d'une manche : spec 023.
- Effets qui prennent un dé de plus ou changent sa face (Source de mana, Attraction de mana) : spec 022 ou la spec de leur effet
  renforcé.
- Tactiques qui touchent la Source (« Mana Steal », « Mana Search ») : spec 060.
- Lancer un dé hors de la Source (récompense de cristal au hasard, p. 11, 5.a) : spec 031.
- Plusieurs joueurs (Source de 4 dés et plus) : la fonction de lancer prend le nombre de dés, testée de 1 à 5 (C5), mais la partie
  reste solo.

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25) :

1. **Q1** — Accepté : la Source seule ; prendre un dé, la limite d'un par tour et la relance en fin de tour passent dans la 022
   (`PLAN.md` mis à jour).
2. **Q2** — « Or », « Noir », « épuisé » acceptés. *Pure mana* se dit **mana fondamental**, pas « mana pur ». La « Bank » du livret
   (réserve commune des cristaux et des dés inutilisés, p. 3) ne sert pas dans cette spec : son terme est retiré du glossaire et sera
   choisi avec la spec 021, qui l'utilise pour les cristaux.
3. **Q3** — Accepté : dés utilisables d'abord, dans l'ordre du lancer, puis les épuisés à part.
4. **Q4** — Confirmé sur le dé physique : six faces, or, noir, bleu, rouge, blanc, vert.
5. **Q5** — Accepté : aucun raccourci.

## Corrections pendant l'implémentation

- **Libellés** : `source.die` devient `source.die.basic`, une clé ne pouvant pas être à la fois un texte et le parent d'autres clés
  dans le JSON imbriqué (spec 001). `source.by_day` et `source.by_night` ajoutées au tableau.
- **Spec 013, `setup_events_end_with_five_draws`** : après le mélange, il n'y a plus que des pioches ; le test vérifie maintenant que
  le dernier événement est `SourceRolled` et que les pioches qui le précèdent sont les 5 premières cartes du deck. Même sens.
- **Spec 012, `setup_events_come_in_order`** et **`PlayerView`** construit champ par champ (spec 002) : comme prévu dans « Effets sur
  les specs déjà implémentées ». Aucun autre test ne comptait les événements de démarrage.
- **C7** : graine 20, valeurs relevées sur `main` avant l'implémentation.

Vérification manuelle sous Linux (Xvfb) : captures `spec-020-source.png` (panneau, un dé noir épuisé) et `spec-020-fenetre.png`
dans les fichiers du projet. Sur quatre lancements, jamais plus d'un dé or ou noir.

## Questions ouvertes

Aucune.
