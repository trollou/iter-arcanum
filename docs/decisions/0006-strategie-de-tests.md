# 0006 — Stratégie de tests

- Statut : proposée (2026-10-26), direction choisie par Mathieu dans le fil « Définir la stratégie de tests UI ».
- Complète la section « Tests » de `CLAUDE.md` et la méthode TDD, sans les remplacer.

## Contexte

Au 2026-10-26, on compte environ 450 tests d'intégration. On y trouve des scénarios de règles par spec dans `crates/arcanum-core/tests/`, et
des tests du modèle d'écran dans `crates/arcanum-ui/tests/` (`game_screen`, `board_screen`, `GameSession`), sans fenêtre. Restent deux trous :

1. **Aucun parcours de bout en bout.** Chaque test construit son état à la main, événement par événement. Rien ne vérifie que les specs
   s'enchaînent dans une vraie partie, de la mise en place à la fin d'une manche.
2. **`app.rs` n'est exécuté par aucun test.** Il ne fait que du câblage : lire les entrées, appeler les fonctions pures, dessiner. Il
   n'est vérifié qu'à l'œil.

On a étudié quatre options : (A) les tests d'interaction egui avec `egui_kittest`, (B) les captures de référence comparées en PNG,
(C) les parcours de bout en bout, et (D) la vérification à l'œil sous Xvfb.

## Décision

- **C — Parcours de bout en bout dans `crates/arcanum-core/tests/parcours_<sujet>.rs`.** Un `GameRunner` part de `new_game`, avec une graine
  fixe et un joueur scripté qui choisit ses actions **par leur valeur** (`Action`), jamais par leur index dans les actions légales. Le
  parcours enchaîne plusieurs specs, par exemple un tour complet dans le bac à sable : jouer des cartes, se déplacer, explorer, finir le
  tour, jusqu'à la fin de manche. Il vérifie les points d'étape qui comptent pour les règles (vue du joueur, événements clés dans
  l'ordre, avec la page du livret en commentaire), pas le journal complet mot pour mot. **Chaque jalon de règles se termine par une
  spec de parcours** (numérotée en 9 dans `PLAN.md`, ex. 019 pour J1), qui écrit au moins un parcours couvrant les grandes
  fonctionnalités du jalon, ce que promet sa ligne « Jouable » (Mathieu, 2026-10-26). La première spec de parcours apporte aussi le joueur scripté
  (la 028, rédigée avant la 019 : décision de Mathieu, 2026-10-27).
  **Une spec de parcours fixe les points d'étape, leurs préconditions et l'ordre qui compte pour les règles ; le découpage en tours de
  son schéma est indicatif** (issue privée 44, Mathieu, 2026-10-27). Le budget de points et de cartes de chaque tour dépend de la main tirée :
  l'implémentation peut déplacer un point d'étape d'un tour à l'autre sans repasser par une validation, à condition de le signaler dans
  la PR. Changer un point d'étape, une précondition ou un ordre qui compte pour les règles reste un changement de spec.
- **Départ d'un parcours : ne rien attendre du hasard qui n'est pas son objet** (issue privée 318, Mathieu, 2026-11-17). Une graine
  qui fait tenir toutes les préconditions d'un long parcours devient rare (J5 : une sur 3 000) et le moindre changement de
  l'usage du hasard la casse. Deux départs, choisis par la spec du parcours :
  - **Mise en place arrangée** (par défaut) : le parcours part de la vraie mise en place de `new_game`, dont le test choisit
    l'ordre des piles mélangées (deck du héros, piles d'ennemis, pioches de sorts, d'actions avancées et d'unités, compétences),
    les dés de la Source et la case de départ de la figurine (`tests/support/arrangement.rs`, `Setup`). Le moteur joue tout le
    reste. Convient quand ce que le parcours vérifie tient dans la première manche, la seule que l'arrangement contrôle.
  - **Épisode posé** : quand un point d'étape dépend d'un hasard de cours de partie (dés relancés, deck remélangé à la nouvelle
    manche, longue marche), il devient un épisode court qui part d'un **état posé** : la mise en place arrangée, puis les
    événements que le test écrit pour poser la situation (comme les parties préparées de `arcanum-persistance` ou le `Sandbox` des
    tests de règles). La spec dit ce qui est posé et pourquoi.
  - **Ce qui ne se pose jamais** : ce que le parcours vérifie. Chaque point d'étape vient d'une action jouée par le moteur ;
    seul ce qui le précède et n'est pas son objet est posé (offres, défenseur, main, Source, case de départ).
  - **La graine n'est plus choisie** : le parcours doit passer avec la plupart des graines, ce que mesure à la rédaction un
    balayage (`precondition` signale encore un départ qui ne tient plus). Plus de `chercher_une_graine` par parcours.
  - **Essai de faisabilité à la rédaction** (issue privée 352) : avant de proposer la spec à la validation, un test jetable fait
    jouer le script prévu (ou l'explorateur) sur une vingtaine de graines depuis le départ choisi, mesure combien tiennent
    chaque précondition et relève l'ordre réel des événements cités par les points d'étape. Le résultat va dans la spec
    (« Départ et préconditions ») : les seuils se valident sur une mesure, pas sur une supposition (spec 069 : l'explorateur
    ne révélait une cité que sur 16 graines sur 40, découvert au codage).
- **D — Vérification à l'œil, inchangée.** Chaque spec qui touche l'UI se termine par une vérification avec `scripts/verif-manuelle.sh`
  (voir `docs/verification-manuelle.md`), avec des captures jointes à la PR ou déposées dans les fichiers du projet.
- **Pas de tests d'interaction egui (A).** `app.rs` reste un câblage sans décision. Toute logique qui choisit quelque chose (quel
  bouton, quelle commande, quelle case) va dans une fonction pure testée (`game_screen`, `board_screen`, `BoardCamera`, `GameSession`).
  Le point reste ouvert (issue #2).
- **Pas de captures de référence (B).** Le rendu diffère entre Windows et Linux, et il faudrait régénérer les images à chaque retouche
  de l'interface.
- **Pas de tests qui bétonnent.** Un test vérifie un comportement décrit par une spec, pas une mise en page ni un détail
  d'implémentation. Le but est que les refontes prévues de l'interface restent bon marché.
  Ainsi, un test de règle vérifie les événements qui portent son comportement, dans l'ordre
  (`assert_events_in_order` de l'arène), et ne compare la liste exacte que si c'est son objet : un
  ordre complet qui compte pour les règles, ou l'absence de tout autre effet (issue privée 110 : un nouvel
  effet de victoire ne doit pas casser les tests de combat qui ne le concernent pas).

## Conséquences

- Aucune nouvelle dépendance. La CI (`fmt`, `clippy`, `test` sous Windows et Linux) ne change pas.
- Le câblage de `app.rs` (un bouton qui n'enverrait pas sa commande, une touche mal lue) n'est attrapé que par la vérification à
  l'œil. C'est un risque accepté. Si une spec complique la lecture des entrées (glisser-déposer, raccourcis plus riches…), on extrait
  cette lecture dans une fonction pure testée, plutôt que d'ajouter A.
- On revoit A si un bug de câblage échappe à la vérification à l'œil, ou si l'interface grossit au point que cette vérification ne
  suffise plus.
