# 0008 — Moteur d'effets : les modificateurs

- Statut : acceptée (2026-11-11), avec la spec `docs/specs/052-moteur-d-effets.md`. Revue après une
  relecture critique (sorte paramétrée, effet « points + modificateurs », durée et déclenchement séparés, modificateurs d'ennemis).
- Complète l'ADR 0001 (l'état ne change que par des événements) et l'ADR 0005 (contenu en données).

## Contexte

Jusqu'à la spec 050, un effet de carte ou d'unité produit **tout de suite** quelque chose de mesurable : des points (Mouvement,
Influence, Attaque, Parade, Soin), du mana, une carte piochée, une unité rendue disponible. `apply` lit l'effet de la carte jouée
(`CardId::effect(play)`) et l'ajoute à la réserve qui convient. Les cartes qui font autre chose sont `Unsupported` et se jouent de
côté.

Le jalon J5 apporte des effets d'une autre nature (MKUE p. 5, *Effects* 1.b) : ils **changent une règle** pour la suite du tour
(ou du combat, ou jusqu'au prochain tour), et d'autres règles les consultent plus tard :

- le coût de déplacement des terrains, les terrains franchissables (MKUE p. 7, 7.a et 7.b) ;
- ce que valent des points dans une autre réserve (Mouvement en Attaque, Influence en Parade) ;
- la prochaine carte d'un genre (Embuscade) ;
- la fin du tour (réputation, place d'une carte) ;
- puis, avec les specs 053 à 055 et 051 : les unités (valeurs augmentées, dégâts interdits), les ennemis (armure réduite,
  résistances perdues, « n'attaque pas »), des effets « jusqu'au début de votre prochain tour » (compétences) ou de la manche.

Il faut une seule façon de les garder, de les consulter et de les faire disparaître, que la sauvegarde et le retour en arrière
suivent sans travail de plus, et qui n'oblige pas à une variante de code par carte : 20 sorts, 16 artefacts et 40 compétences
viendront.

Trois pistes ont été étudiées :

- **(A) Une liste de modificateurs dans l'état**, remplie par l'`apply` de l'événement qui joue l'effet (`CardPlayed`,
  `UnitActivated`), vidée par l'`apply` de l'événement qui clôt leur durée.
- **(B) Des événements propres** (`ModifierAdded`, `ModifierExpired`) produits par `decide` à côté de l'événement qui joue l'effet.
- **(C) Rien dans l'état** : les modificateurs se recalculent à la demande depuis la zone de jeu.

(C) n'ajoute aucune donnée, mais les unités activées, les compétences et les cartes jouées depuis une offre ne sont pas dans la zone de
jeu, et un effet consommé (Embuscade) obligerait à rejouer l'ordre des cartes à chaque lecture. (B) double l'information : l'événement
qui joue la carte dit déjà tout, et `decide` recopierait une règle qu'`apply` connaît. (A) suit ce que fait déjà le moteur pour les
points.

## Décision

**Piste (A) : les effets qui durent sont des modificateurs (`Modifier`) gardés dans l'état.**

### Un effet de carte = des points et des modificateurs

- Une option d'une carte (ou d'une capacité d'unité) est **un effet de points** (les variantes actuelles de `Effect`, inchangées)
  **et zéro, un ou plusieurs modificateurs posés**. Chemins de traverse de base : `Move(2)` et une réduction de coût. Pas de variante
  d'`Effect` propre à une carte pour « points + règle ».
- Le bonus de Concentration (`with_bonus`) ne touche que les points (MKUE p. 5, *"Composite" Effects* 1.b ; Compendium,
  *Concentration* S5, *Will Focus* S1).
- En données (ADR 0005), le modificateur s'écrit comme une valeur : `(Move(2), [TerrainCost(terrains: All, change: Reduce(by: 1,
  min: 2))])`. Le fichier ne contient toujours que des choix parmi ce que le moteur connaît.

### Sorte : paramétrée, pas une par carte

La **sorte** d'un modificateur est une énumération fermée de règles **paramétrées** ; une carte nouvelle qui change la même règle
n'ajoute que des données. Sortes posées par la spec 052 :

| Sorte (nom indicatif) | Paramètres | Cartes de la 052 (puis exemples à venir) |
|---|---|---|
| `TerrainCost` | terrains visés ; `Reduce { by, min }` ou `SetTo(n)` | Chemins de traverse, Pont de glace, Bûcherons (Mélodie du vent, 053) |
| `Passable` | un terrain (lac) | Pont de glace renforcé (Mélodie du vent, 053) |
| `Convert` | réserve de départ, points obtenus (Attaque d'un genre, Parade d'un élément), taux | Agilité, Diplomatie |
| `NextCardBonus` | bonus d'Attaque, bonus de Parade | Embuscade |
| `EndOfTurnReputation` | variation | Intimidation (Récit héroïque, 053) |
| `ReturnToDeck` | sur ou sous le deck | Temporisation |

Les sortes des specs suivantes s'ajoutent à la même énumération (bonus des unités, dégâts interdits aux unités, mana des dés…).

### Durée et déclenchement

Deux notions séparées :

- **Durée** : quand le modificateur disparaît. `Turn` (par défaut, MKUE p. 5, *Effects* 1.b), `Combat` ; réservées pour la suite :
  `UntilNextTurn` (compétences « jusqu'au début de votre prochain tour », MKUE p. 5, *Using Skills* 4.b) et `Round`. Chaque événement
  de fin retire **les modificateurs de sa durée** : `TurnEnded` les `Turn`, `CombatEnded` les `Combat`, `TurnStarted` les
  `UntilNextTurn`, la nouvelle manche les `Round` ; jamais « tout ».
- **Déclenchement** : comment il agit. **Lu** tant qu'il existe (coût des terrains, conversion) ; **consommé** par l'événement qui
  l'utilise (Embuscade, retiré par la carte qui reçoit le bonus) ; **à la fin du tour** (réputation, place de la carte : la fin du
  tour produit son propre événement, avant `TurnEnded`).

### Source

`ModifierSource` dit d'où vient le modificateur : une carte (`CardId`) ou une unité (`UnitCard`), puis une compétence, une tactique.
La source porte la **couleur** de l'effet, pour la règle générale des résistances (FAQ p2 : un ennemi résistant ignore l'effet
spécial d'une carte de sa couleur) que la 053 écrira une fois pour toutes. Plusieurs modificateurs peuvent avoir la même source
(Effet maximum, 053).

### Ajout, retrait, lecture

- **Ajout** par l'`apply` de l'événement qui joue l'effet, comme les points : aucun événement nouveau, aucun hasard.
- **Lecture** par des fonctions de règles qui reçoivent les modificateurs en paramètre (coût de déplacement, actions légales du
  combat, fin du tour). L'écran ne les lit que par la vue (`PlayerView`), où ils sont publics.
- **Ennemis** : ce qui change un ennemi pour le reste du combat est une liste de modificateurs **de l'ennemi du combat**
  (`CombatEnemy::modifiers`, même forme, sortes propres), qui disparaît avec le combat. Aucun n'est posé par la 052 ; la 053 en pose
  les premiers (Bouclier de glace, unités qui empêchent une attaque).
- **Bannières** (spec 055) : un artefact attaché à une unité est un état de l'unité, pas un modificateur.

### Sauvegarde et retour en arrière

Rien à faire : la sauvegarde rejoue le journal (spec 024), le retour en arrière restaure l'état d'avant la décision. Le format des
événements change seulement par les variantes ajoutées, sans migration avant la fin du J6 (spec 024, Q1).

## Conséquences

- Une spec qui ajoute un effet durable ajoute, au plus, une sorte de modificateur et la règle qui la lit ; souvent seulement des
  données. Elle ne touche ni au journal ni à la sauvegarde.
- Chaque règle modifiable prend les modificateurs en paramètre : `move_cost` et les trajets (`routes…`) reçoivent des règles de
  déplacement (`MoveRules`, construites depuis `TimeOfDay` pour les appels sans modificateur) ; l'écran les obtient de la vue.
- Un modificateur qui demande un choix (élément de Diplomatie renforcée) est choisi en jouant la carte, par une option de la carte.
- Les effets qui ne font que donner des points ou des choses tout de suite (Coup de sang, Marche revigorante…) restent des effets
  de points, au besoin des variantes de `Effect` nouvelles ; cet ADR ne crée pas de langage d'effets composés.
