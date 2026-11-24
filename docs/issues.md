# Travailler avec les issues

Comment on ouvre, classe, traite et ferme une issue de `trollou/iter-arcanum`. À lire au début d'un fil
d'issue. Une nouvelle fonctionnalité de jeu n'est pas une issue : elle devient une ligne de
`docs/specs/PLAN.md`, puis une spec.

## Quand ouvrir une issue

- **Bilan de fin de spec** : une issue par point réel, confirmé avec Mathieu (CLAUDE.md), label `bilan`.
- **Bug** trouvé hors d'une spec : comportement contraire à la spec, au livret ou à une carte.
- **Règle à trancher** : livret ambigu, ou en contradiction avec la FAQ, une carte, une spec ou le code.
- **Point ouvert** : voir [Points ouverts](#points-ouverts).
- **Idée de Mathieu**, même en une ligne : le fil qui la traite la reformule dans un commentaire avant
  de commencer.

## Corps d'une issue

Le modèle `.github/ISSUE_TEMPLATE/issue.md` le préremplit :

- **Origine** : bilan de la spec NNN (PR #…), trouvé en faisant…, ou idée de Mathieu.
- **Constat** : ce qui se passe, avec le fichier, la fonction ou la page du livret (`MKUE p. 9, 5.c`).
- **Proposition** : la correction envisagée, et ce qu'elle touche (spec, code, doc).
- **À trancher**, si besoin : la question pour Mathieu, avec les options.

## Labels

Exactement un label de **type**, au plus un label d'**état**, et les deux labels facultatifs.

| Type | Pour |
|---|---|
| `bug` | comportement contraire à la spec, au livret ou à une carte |
| `règle` | règle du jeu à trancher (livret ambigu, contradiction) |
| `méthode` | façon de travailler : CLAUDE.md, gabarits, fils, tokens |
| `tests` | aides, arène, parcours, organisation des tests |
| `refactor` | organisation du code, sans changer le comportement |
| `outillage` | scripts, CI, hooks, parties préparées |
| `contenu` | données du jeu, libellés, noms français, transcriptions |
| `documentation` | docs du dépôt |

| État | Pour |
|---|---|
| `à trancher` | une question attend Mathieu : règle, choix de solution, ou intérêt contesté par le fil |
| `en attente` | bloquée par autre chose (photos, une spec, une autre issue) ; un commentaire dit quoi |

Un label d'état se retire dès qu'il ne s'applique plus.

| Facultatif | Pour |
|---|---|
| `bilan` | née d'un bilan de fin de spec |
| `réservé` | **Mathieu s'en occupe : Claude ne la traite pas**, ni correction, ni branche, ni PR (lire et citer reste permis) |
| `question ouverte` | point ouvert sorti d'un fichier Markdown (voir [Points ouverts](#points-ouverts)) |
| `planifié` | intégrée à `docs/specs/PLAN.md` (une ligne de spec la cite), pas encore développée (voir [Issues planifiées](#issues-planifiées)) |

Les labels de Dependabot (`dependencies`, `rust`, `github_actions`) ne servent qu'à ses PR. Une issue en
double ou abandonnée se ferme avec le motif de GitHub (`duplicate`, `not planned`), pas par un label.

## Points ouverts

Un point ouvert est une question qui reste sans réponse au-delà du fil qui l'a soulevée : sujet reporté (« à revoir quand… »),
décision provisoire, lecture incertaine d'une transcription, idée pas encore placée dans le plan. **Il vit dans une issue, pas dans
un fichier Markdown** (Mathieu, 2026-11-11) :

- l'issue porte un label de type, `question ouverte` et `réservé` (Claude ne la traite pas d'elle-même) ;
- le Markdown (spec, doc, plan, transcription) n'en garde qu'un renvoi court : « à revoir avec le premier joueur automatique (issue #3) » ;
- quand le point est tranché, la décision va dans la doc concernée, avec la date, et l'issue est fermée.

Les questions d'une spec **en cours de rédaction** restent dans sa section « Questions ouvertes », tranchées à la relecture de
Mathieu : elles font partie du travail du fil, sans issue (Mathieu, 2026-11-11). Seule une question qu'on décide de reporter
devient une issue, avant que la spec passe en « validée ».

## Issues planifiées

Une issue de fonctionnalité (ou un point ouvert) qu'on décide de développer devient une ligne de `docs/specs/PLAN.md`, qui la
cite (« issue #N »), puis une spec. Dès que la ligne existe, l'issue prend le label `planifié`, pour que Mathieu voie d'un coup
d'œil ce qui est prévu mais pas encore fait. Les autres labels restent : une issue `réservé` ou `question ouverte` le reste tant
que ses questions ne sont pas tranchées, à la rédaction de la spec au plus tard.

- La PR qui ajoute la ligne au plan pose le label (et le crée s'il manque).
- La PR qui implémente la spec ferme l'issue (`Closes #N`) ; si l'issue reste ouverte pour un reste hors de la spec, retirer
  `planifié` et dire dans un commentaire ce qui reste.
- Une ligne retirée du plan retire le label.

## Quand les traiter

**Entre deux specs** : quand le bilan d'une spec a ouvert des issues, on les traite après la fusion de
la spec et avant de démarrer la suivante. Les autres issues attendent que Mathieu les lance.

**Une issue à la fois, un fil par issue** : la suivante ne démarre qu'une fois la précédente finie
(PR fusionnée, ou issue laissée `à trancher`). Les issues `réservé` sont sautées.

## Traiter une issue

1. **Challenger l'intérêt** en une phrase (coût, bénéfice). Si c'est discutable : commentaire argumenté
   sur l'issue, label `à trancher`, ni branche ni PR ; le fil s'arrête là.
2. Sinon, branche `fix/<n°>-<sujet>` et même cycle que pour une spec : test rouge d'abord si du code de
   production change, `scripts/verifier.sh` vert, PR dont le corps contient `Closes #<n°>`.
3. Si la correction change le comportement du jeu au-delà d'une spec : mettre la spec à jour dans la
   même PR, ou passer l'issue `en attente` d'une spec.
4. **Fusion** : sans question ouverte (solution donnée par l'issue, aucun choix de règle), le fil
   fusionne lui-même, en squash, une fois la CI verte (une PR de doc seule n'a pas de CI). Sinon Mathieu
   fusionne.
5. **Fermeture** : par la PR (`Closes`), ou à la main avec le motif (`completed`, `not planned`,
   `duplicate`) et une ligne qui dit pourquoi.
