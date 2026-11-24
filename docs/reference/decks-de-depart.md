# Composition des decks de départ

Chaque joueur mélange « les 16 cartes d'action de base de son héros (voir le symbole en haut à droite) » (livret, mise en place).
Ce deck part d'un deck standard dont **deux cartes** sont remplacées par les deux cartes propres au héros.

Sources : numéros « n/16 » imprimés sur les cartes du jeu de base (scans de `../mage-knight/docs/regles/scans/deed/`), recoupés avec
[victoryconditions.com](https://victoryconditions.com/mage-knight-heroes-and-their-abilities/) et
[taogaming.wordpress.com](https://taogaming.wordpress.com/2017/07/29/too-many-words-about-mage-knight-part-vi-characters/).

## Composition

Le deck standard (16 cartes, dans l'ordre des numéros n/16) et les deux remplacements de chaque héros sont dans
[`crates/arcanum-core/content/heroes.ron`](../../crates/arcanum-core/content/heroes.ron), seule source pour le code et pour la doc (spec 029).
Les effets des cartes sont dans `crates/arcanum-core/content/basic_actions.ron`.

## Notes de lecture

Un seul exemplaire de chaque carte est scanné : les numéros 4, 6 et 8 (seconds exemplaires) n'ont pas de scan propre.

Pour la 1re carte propre au héros, le remplacement est confirmé par le numéro n/16 qu'elle partage avec la carte remplacée.
Pour la 2e carte (numérotée n/377 dans l'édition Ultimate), il vient des deux sources web, qui concordent.
Chaque carte propre est d'ailleurs une version améliorée de la carte qu'elle remplace.

## Hors périmètre pour l'instant

Héros des extensions, d'après la même source : Krang (March → Savage Harvesting, Threaten → Ruthless Coercion),
Wolfhawk (Stamina → Tirelessness, Swiftness → Swift Reflexes), Braevalar (March → One With the Land, Stamina → Druidic Paths).
