# Plan de développement — spec #N <Titre>

Gabarit d'un plan de développement : à copier en `plans/issue-<N>-<sujet>.md` quand une spec demande plus que quelques fichiers
(`CLAUDE.md`, « proposer un plan »). Exemple rempli : [`031-deroule-combat.md`](031-deroule-combat.md).

- Statut : brouillon | accepté par Mathieu (date).
- Spec : [`issue-<N>-<sujet>.md`](../issue-<N>-<sujet>.md). Branche : `feature/<N>-<sujet>`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR si possible. Sinon, des PR fusionnables l'une après l'autre, chacune verte et utilisable : contenu, cas couverts,
état du jeu après fusion.

## Découpage en fils

Un fil d'implémentation par défaut. **Deux fils successifs sur la même branche** quand le plan dépasse 8 étapes, ou touche
à la fois le moteur et l'écran (issue privée 118, bilan de la spec 042 : un seul fil a dû être résumé en cours de route) :

1. moteur, contenu, persistance ;
2. libellés, écran, partie préparée, vérification à l'œil, docs.

**Contrôle avant d'accepter le plan** (issue privée 130) : pour chaque cas d'écran, le champ de la vue qui le nourrit (section Interface
de la spec) est soit déjà dans `PlayerView`, soit produit par une étape du fil 1. Un champ manquant devient une étape moteur,
avec son test arcanum-core, et non une découverte du fil 2.

Le plan dit où passe la coupure (« fil 1 : étapes 1 à 6 »). Le premier fil pousse sa dernière étape verte et s'arrête ; le
second repart de la spec, du plan et de la branche, sans relire l'historique du premier.

## Étapes

Chaque étape qui code un comportement = un commit `Tests rouges : <étape>` (local) puis un commit de code (vert). Sans
comportement (libellés, docs) : pas de rouge, la colonne le dit (« — »). Règles, issue privée 106 (bilan de la spec 050) :

- **Une étape d'écran a son rouge** (issue privée 218, bilan de la spec 052) : `game_screen`, `board_screen` et `combat_screen` se
  testent sans fenêtre (ADR 0006), sur les écrans calculés (le champ vide du squelette fait échouer l'assertion). La colonne
  « Rouge attendu » ne met « — (vérification à l'œil) » que pour le pur dessin egui (couleurs, placement), jamais pour ce qu'un
  écran calcule (cases cliquables, libellés choisis, boutons proposés).

- **Squelette d'abord** : une étape qui crée des types commence par leurs signatures seules (`todo!()`, données vides), pour
  que les tests **compilent et échouent sur une assertion**. Un échec de compilation ne compte pas comme rouge.
- Le message du commit rouge nomme les tests en échec et leur nombre (« 4 failed »), donnés par `scripts/verifier.sh --rouge`
  (tous les fichiers de test, `--no-fail-fast`, issue privée 120). Il refuse un échec de compilation, qui n'est pas un rouge.
  Limité aux tests de l'étape (`--rouge -p <crate>`, avec `--test <fichier>` au besoin, issue privée 359), il ne compile et ne lance
  que ces tests au lieu des quatre minutes du passage complet ; celui-ci reste celui du hook avant le push.
- **Pas d'étape « tests seulement »** : les tests d'un comportement vont dans l'étape qui écrit son code. Un test qui passe du
  premier coup est signalé dans la PR.
- Le corps de la PR a une section **« Rouge vu »** : une ligne par étape, empreinte du commit rouge et nombre d'échecs.
- Dans le fil, la checklist de progression sépare chaque étape en « tests rouges » puis « code ».
- **Rouge d'un parcours** (spec de parcours, règles déjà implémentées) : chaque point d'étape est d'abord écrit avec l'attente
  du moteur *avant* la règle qu'il vérifie, vu en échec, puis rendu juste (méthode du plan de la 049). Un test s'arrête au premier
  `assert` rouge : si la colonne « Rouge attendu » d'une étape cite plusieurs points, ils sont vus rouges **un par un** (remettre
  juste le précédent avant de lancer le suivant), et la section « Rouge vu » de la PR nomme chacun (issue privée 145).

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | … | C1 à C3 | C2 : … (assertion qui échoue avant le code) | … | non | fil principal |
| 2 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |

## Choix du modèle par étape

Économiser les modèles les plus chers là où il y a peu à réfléchir (demande de Mathieu, 2026-10-28), **sans faire relire au
sous-agent tout le contexte que le fil a déjà** : déléguer une étape liée au reste coûte plus cher que de l'écrire (bilan de la
spec 031, issue privée 66).

- **Ne déléguer que les étapes isolées**, celles qu'un sous-agent fait à partir d'un brief court sans relire la spec entière ni
  le code voisin : recopie de libellés, docs (architecture, index), lecture du livret ou du compendium pour une question
  précise, captures sous Xvfb, tests sur un patron déjà posé (ex. manques de libellés, sur le patron de la spec 030, C28).
  Modèle : Haiku pour le mécanique, Sonnet pour un patron à suivre.
- **Une étape liée au reste** (nouveaux types, dessin qui dépend des tests et des corrections vues à l'œil, étapes qui se
  suivent sur les mêmes fichiers) reste au fil principal. La colonne Modèle dit « fil principal » ; le plan ne fixe pas d'autre
  modèle, le choix se fait à l'implémentation.
- Le fil principal écrit le brief, relit le diff et lance `scripts/verifier.sh`. Si une étape déléguée revient deux fois fausse,
  il la reprend.

Le brief d'un sous-agent (outil `Agent`, paramètre `model`) donne : les cas de la spec à couvrir, les fichiers à toucher, le
patron à suivre, ce qu'il ne faut pas faire (pas de nouvelle dépendance, pas de test modifié sans le dire, pas de push). Il
exige **un compte rendu final** :

- fichiers touchés ;
- tests lancés et leur résultat ;
- commit créé (empreinte et message), ou « aucun commit » et pourquoi.

Un sous-agent qui finit sans ce compte rendu a son travail vérifié dans `git log` et `git diff` avant toute suite.

## Risques repérés

Ce qui peut faire dérailler le plan : choix qui se propage, nombre d'actions légales, tests des specs déjà implémentées qui
changent de sens, CI indisponible.
