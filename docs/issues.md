# Travailler avec les issues et le projet

Comment on planifie, ouvre, classe, traite et ferme une issue de `trollou/iter-arcanum`. À lire au début d'un fil d'issue
ou de spec. La planification vit dans GitHub (issues, milestones, projet « suivi projet ») depuis le 2026-11-24 ;
`docs/specs/PLAN.md` est archivé et ne bouge plus.

## Planification

- **Milestone = version** du jeu (`v0.1.0`, `v0.2.0`…), sans date : on livre quand c'est prêt. À chaque milestone
  terminé, une release (issue #33). Une version corrective (dernier chiffre) peut sortir hors milestone, sans fonctionnalité
  nouvelle. Pas d'itérations (Mathieu, 2026-11-24).
- **Ordre de travail** : le milestone ouvert le plus ancien, puis le champ **Rank** croissant. Les issues `réservé` sont
  sautées, comme celles bloquées par une issue encore ouverte (« blocked by »).
- **Projet GitHub** « suivi projet » : la vue de Mathieu sur ces informations. Claude ne peut ni le lire ni l'écrire (API
  fermée depuis ses sessions) : tout ce qu'il doit lire ou remplir est porté par l'issue elle-même (type, milestone, parent,
  dépendances, labels, champs d'issue).

## Types d'issue

Chaque issue a exactement un type GitHub :

| Type | Pour |
|---|---|
| `Epic` | une grande fonctionnalité (l'ancien jalon), découpée en sous-issues |
| `Feature` | une fonctionnalité : elle devient une spec (voir [Specs et issues](#specs-et-issues)) |
| `Bug` | comportement contraire à la spec, au livret ou à une carte |
| `Task` | ni l'un ni l'autre : méthode, tests, refactor, outillage, contenu, documentation |

Les Features et les Tasks d'une grande fonctionnalité sont des **sous-issues** de son Epic. Une Feature trop grosse pour une
spec devient elle-même une Epic. Quand une issue ne peut pas commencer avant une autre, on le dit par la dépendance
GitHub « blocked by », pas par un label.

## Champs d'issue

| Champ | Valeurs | Qui le remplit |
|---|---|---|
| Priority | liste de choix | Mathieu |
| Effort | `Low` : un fil d'issue ; `Medium` : une spec codée en un fil ; `High` : une spec en deux fils ou plus | Mathieu, ou Claude quand il est vide |
| Rank | nombre (10, 20, 30… pour pouvoir insérer sans tout renuméroter) | Claude, d'après la priorité, les dépendances et l'ordre des sous-issues de l'Epic |

Claude remplit Effort et Rank quand il crée une issue, et quand il en trouve une sans ces champs dans un milestone. Le Rank
ne compte qu'à l'intérieur d'un milestone.

## Specs et issues

Une spec naît d'une issue `Feature` (ou `Epic`), qui doit exister avant la rédaction.

- Fichier `docs/specs/issue-<N>-<sujet>.md`, cité « spec #N » ; plan éventuel `docs/specs/plans/issue-<N>-<sujet>.md`.
- Branche `feature/<N>-<sujet>` (rédaction et implémentation sur la même branche).
- La PR qui rédige la spec la cite (« spec #N ») ; la PR qui l'implémente ferme l'issue (`Closes #N`).
- Le statut (brouillon, à valider, validée, implémentée) est dans l'en-tête de la spec.

Les specs d'avant le 2026-11-24 gardent leur numéro à trois chiffres (« spec 042 ») : la migration vers ce dépôt a
renuméroté les issues à partir de 1, donc ces numéros ne sont pas ceux des issues.

## Quand ouvrir une issue

- **Fonctionnalité** à développer : une `Feature`, rattachée à son Epic et à un milestone.
- **Bilan de fin de spec** : une issue par point réel, confirmé avec Mathieu (CLAUDE.md), label `bilan`.
- **Bug** trouvé hors d'une spec.
- **Règle à trancher** : livret ambigu, ou en contradiction avec la FAQ, une carte, une spec ou le code (label `règle`).
- **Point ouvert** : voir [Points ouverts](#points-ouverts).
- **Idée de Mathieu**, même en une ligne : le fil qui la traite la reformule dans un commentaire avant de commencer.

## Corps d'une issue

Les modèles de `.github/ISSUE_TEMPLATE/` (un par type) le préremplissent :

- **Origine** : bilan de la spec #N (PR #…), trouvé en faisant…, ou idée de Mathieu.
- **Constat** (ou **Objectif** pour une Feature) : ce qui se passe ou ce qu'on veut, avec le fichier, la fonction ou la page
  du livret (`MKUE p. 9, 5.c`).
- **Proposition** : la correction ou la solution envisagée, et ce qu'elle touche (spec, code, doc).
- **À trancher**, si besoin : la question pour Mathieu, avec les options.

## Labels

Le type, le milestone et les dépendances portent l'essentiel ; il ne reste que cinq labels, tous facultatifs :

| Label | Pour |
|---|---|
| `règle` | règle du jeu en jeu (livret ambigu, contradiction, effet de carte) |
| `bilan` | née d'un bilan de fin de spec |
| `à trancher` | une question attend Mathieu : règle, choix de solution, ou intérêt contesté par le fil ; à retirer une fois tranchée |
| `question ouverte` | point ouvert sorti d'un fichier Markdown (voir [Points ouverts](#points-ouverts)) |
| `réservé` | **Mathieu s'en occupe : Claude ne la traite pas**, ni correction, ni branche, ni PR (lire et citer reste permis) |

Les labels de Dependabot (`dependencies`, `rust`, `github_actions`) ne servent qu'à ses PR. Une issue en double ou abandonnée
se ferme avec le motif de GitHub (`duplicate`, `not planned`), pas par un label.

## Points ouverts

Un point ouvert est une question qui reste sans réponse au-delà du fil qui l'a soulevée : sujet reporté (« à revoir quand… »),
décision provisoire, lecture incertaine d'une transcription, idée pas encore planifiée. **Il vit dans une issue, pas dans
un fichier Markdown** (Mathieu, 2026-11-11) :

- l'issue porte son type, `question ouverte` et `réservé` (Claude ne la traite pas d'elle-même), sans milestone tant qu'elle
  n'est pas planifiée ;
- le Markdown (spec, doc, transcription) n'en garde qu'un renvoi court : « à revoir avec le premier joueur automatique (issue #3) » ;
- quand le point est tranché, la décision va dans la doc concernée, avec la date, et l'issue est fermée.

Les questions d'une spec **en cours de rédaction** restent dans sa section « Questions ouvertes », tranchées à la relecture de
Mathieu : elles font partie du travail du fil, sans issue (Mathieu, 2026-11-11). Seule une question qu'on décide de reporter
devient une issue, avant que la spec passe en « validée ».

## Quand les traiter

**Dans l'ordre de travail** (voir [Planification](#planification)) : la prochaine issue est la première du milestone en cours
par Rank, qui n'est ni `réservé` ni bloquée.

**Bilan d'abord** : les issues ouvertes par le bilan d'une spec se traitent après la fusion de la spec et avant la suivante,
même si leur Rank est plus loin.

**Une issue à la fois, un fil par issue** : la suivante ne démarre qu'une fois la précédente finie (PR fusionnée, ou issue
laissée `à trancher`), sauf si plusieurs se regroupent naturellement (même code).

## Traiter une issue

1. **Challenger l'intérêt** en une phrase (coût, bénéfice). Si c'est discutable : commentaire argumenté sur l'issue, label
   `à trancher`, ni branche ni PR ; le fil s'arrête là.
2. Une `Feature` passe par une spec (voir [Specs et issues](#specs-et-issues)). Sinon, branche `fix/<n°>-<sujet>` et même
   cycle que pour une spec : test rouge d'abord si du code de production change, `scripts/verifier.sh` vert, PR dont le corps
   contient `Closes #<n°>`.
3. Si la correction change le comportement du jeu au-delà d'une spec : mettre la spec à jour dans la même PR, ou marquer
   l'issue « blocked by » l'issue de la spec à venir.
4. **Fusion** : sans question ouverte (solution donnée par l'issue, aucun choix de règle), le fil fusionne lui-même, en squash,
   une fois la CI verte (une PR de doc seule n'a pas de CI). Sinon Mathieu fusionne.
5. **Fermeture** : par la PR (`Closes`), ou à la main avec le motif (`completed`, `not planned`, `duplicate`) et une ligne qui
   dit pourquoi.
