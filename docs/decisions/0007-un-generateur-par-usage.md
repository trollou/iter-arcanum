# 0007 — Un générateur par usage

- Statut : acceptée (2026-10-28), pour l'issue privée 52 ; piste (A) choisie par Mathieu ; traduite en comportements par la
  spec 038.
- Complète l'ADR 0001 (le hasard est résolu dans l'événement) et sa révision par la spec 024 (sauvegarde = journal + position du
  générateur).

## Contexte

Le moteur n'a qu'un générateur (`GameRng`, ChaCha8 initialisé par la graine de `GameConfig`). Tout le hasard y passe, dans l'ordre où
le moteur le tire : pioche de tuiles, deck du héros, Source, piles d'ennemis à la mise en place, puis dés relancés, nouvelles manches,
tuile écartée tirée au hasard.

Conséquence : **tout nouveau tirage décale tous ceux qui le suivent.** La spec 030 a ajouté le mélange des piles d'ennemis à la fin de
la mise en place ; les dés de la Source des manches suivantes et les mélanges du deck ont changé pour une même graine, et les
préconditions du parcours J2 ne tenaient plus avec la graine 10. Il a fallu chercher la graine 73 parmi 400 (issue privée 52). Les specs à
venir ajoutent d'autres tirages (ruines, offres d'unités, de sorts et d'actions avancées, artefacts, compétences, joueur fantôme) :
cette recherche reviendrait à chacune.

Deux pistes ont été étudiées :

- **(A) Un générateur par usage**, dérivé de la graine : la Source, la pioche de tuiles, le deck, chaque pile d'ennemis ont chacun le
  leur. Un nouvel usage ne touche pas aux autres.
- **(B) Un outil de test** qui cherche lui-même une graine satisfaisant les préconditions d'un parcours et l'écrit dans le message
  d'échec.

(B) ne fait que rendre la recherche plus rapide : la graine d'un parcours changerait encore à chaque spec, avec la relecture du
parcours qui va avec, et le même problème toucherait les tests de règles qui fixent une graine. (A) supprime la cause.

## Décision

**Un générateur par usage du hasard, tous dérivés de la seule graine de la partie.**

- **Usage.** Un usage est une famille de tirages qui partagent une même suite : « la Source », « la pioche de tuiles », « le deck du
  joueur 0 », « la pile d'ennemis verte »… Chaque usage a un numéro fixe, écrit en dur dans le code (pas l'ordre d'une énumération) :
  il ne change jamais une fois livré, et un usage retiré laisse son numéro inutilisé.
- **Dérivation.** Le générateur d'un usage est ChaCha8 initialisé par la graine de la partie, sur le **flux** (*stream*, `set_stream`)
  qui porte le numéro de l'usage. ChaCha8 offre 2⁶⁴ flux indépendants par graine : aucune nouvelle dépendance, aucun hachage maison,
  et le résultat est le même sous Windows et Linux.
- **Création à la demande.** Le générateur d'un usage n'est créé qu'à son premier tirage. Un usage jamais tiré n'a pas de position.
- **Un seul point d'entrée.** `decide` et `new_game` reçoivent l'ensemble des générateurs et y prennent celui de l'usage qu'ils
  tirent. Tirer le hasard reste interdit ailleurs (ADR 0001).
- **Sauvegarde.** La position d'**un** générateur est remplacée par la position de **chaque** générateur déjà créé, rangée par usage.
  Un usage absent de la sauvegarde repart du début de son flux : c'est exactement ce qu'aurait fait la partie d'origine, puisqu'il n'y
  avait encore rien tiré.

## Conséquences

- Ajouter un usage (une spec qui introduit un nouveau tirage) ne change **aucun** tirage existant pour une même graine. Les graines
  des parcours et des tests de règles restent valables.
- Un tirage de plus **dans un usage existant** décale encore la suite de cet usage (par exemple une carte qui relance un dé de la
  Source décale les relances suivantes de la Source). C'est voulu : c'est la même suite, et il n'y a qu'une suite par usage. Le
  découpage en usages doit donc suivre les piles et les dés du jeu, pas les specs.
- Le passage à ce modèle change une dernière fois tous les tirages : les graines des parcours J1 et J2 sont rechoisies une fois.
- Le format de sauvegarde change : les sauvegardes d'avant sont refusées avec un message, comme le prévoit la révision de l'ADR 0001
  (pas de migration avant la fin du J6).
- Les simulations (MCTS, joueurs automatiques) copient l'ensemble des générateurs au lieu d'un seul : coût négligeable (quelques
  générateurs de quelques dizaines d'octets).
- En partie à plusieurs, chaque joueur aura ses usages propres (son deck, plus tard ses compétences) : le numéro d'usage contient
  l'indice du joueur, comme celui d'une pile contient sa couleur.
