# 0009 — Illustrations générées

- Statut : acceptée (2026-11-05), **révisée le 2026-11-06** par la spec 072 (voir « Révision » en fin de page : héros et
  monstres restent en SVG, images carrées), issue de la discussion sur la refonte de l'IHM (synthèse `docs/specs/refonte-ihm.md`) ;
  traduite en comportements par la spec 072.
- Complète la règle « Assets et droits » de `CLAUDE.md` (visuels créés par nous), mise à jour pour dire que les illustrations
  générées en font partie.
- Le numéro 0008 est réservé au moteur d'effets (spec 052).

## Contexte

La refonte de l'IHM veut de vraies illustrations là où le jeu montre aujourd'hui du texte ou une pastille : cartes action, unités,
héros, monstres. Les SVG dessinés par le code restent nets à tout zoom et conviennent au plateau, mais ils restent simples. Un essai
avec l'API d'images de Google Gemini (2026-11-05, une comparaison dans `docs/specs/images/refonte-ihm/`) donne un rendu bien plus riche
pour une figurine, un site, une carte et une compétence (synthèse `docs/specs/refonte-ihm.md`). Mathieu a choisi le partage
ci-dessous au vu de cet essai.

## Décision

- **Deux familles de visuels.**
  - **SVG dessinés par le code** : le plateau isométrique (terrains, sites, figurines posées sur les cases) et les icônes simples
    (Mouvement, Attaque par élément, Attaque à distance, Attaque de siège, Influence, cristaux, Armure, Gloire…).
  - **Images générées** : les illustrations des cartes action, des unités, des héros et des monstres (sorts et artefacts aussi, Q1).
  - Une figurine SVG du plateau affiche l'image générée de son héros ou de son monstre dans son infobulle, pour faire le lien.
- **Rien d'officiel, mais reconnaissable.** Les descriptions données au générateur ne nomment ni Mage Knight, ni son éditeur, ni
  ses héros ou ennemis (Q5). Elles décrivent précisément, avec nos mots, l'image voulue d'après le visuel du
  jeu (scans de `../mage-knight/docs/regles/scans/`) : type de personnage ou de créature, silhouette, couleurs dominantes, équipement, ambiance,
  pour que le joueur reconnaisse ce qu'il connaît. Elles ne décrivent pas la composition d'une illustration précise (pose,
  cadrage, décor). Aucun scan, photo ou illustration du jeu n'est donné au générateur comme image de départ. Une image qui
  reproduit une illustration officielle est rejetée.
- **Aucun texte dans les images** (ADR 0003) : la description l'interdit ; une image qui en contient est rejetée.
- **Un style commun.** Un préambule de style unique (trait, palette, lumière, cadrage) est placé avant chaque description, avec une
  variante par famille (figurine détourée, illustration de carte en portrait, icône carrée). Il est versionné avec les images.
- **L'image versionnée fait foi.** Le générateur ne rejoue pas une image à l'identique : on versionne le PNG retenu, et à côté sa
  description, le modèle et la date, pour pouvoir en refaire une proche ou toute la série dans le même style.
- **Format et rangement.** PNG, fond transparent pour ce qui est détouré (figurines, unités, monstres), redimensionné à deux fois sa
  plus grande taille d'affichage : 512 px pour une carte ou une unité, 256 px pour une figurine (Q3). Rangé dans
  `assets/images/` par famille et nommé par l'identifiant du code, comme les SVG (ex. `assets/images/enemy-art/orc_marauders.png`), chargé depuis le dossier des ressources comme eux (spec 011).
- **Une image manquante n'empêche pas de jouer** : l'écran garde son affichage actuel (SVG, texte). Les illustrations peuvent donc
  arriver par lots, indépendamment des specs de règles.
- **Génération hors du jeu.** Un script de `scripts/` appelle l'API, depuis un fil cloud ; la clé est le secret d'environnement
  `GEMINI_API_KEY`, jamais versionnée. Mathieu valide chaque lot à l'œil avant fusion. Le jeu n'appelle jamais l'API.

## Conséquences

- Afficher un PNG demande la fonction d'images matricielles d'`egui_extras` et la crate `image` réduite au PNG (Q2).
- Le dépôt grossit : de l'ordre de 100 à 200 images, soit quelques dizaines de Mo. Il reste privé (`CLAUDE.md`).
- La génération coûte : la facturation est activée sur le projet Google AI Studio (l'offre gratuite ne donne pas accès aux
  modèles d'image, constaté le 2026-11-05).
- Un test d'arcanum-ui vérifie que chaque image présente porte un identifiant connu du contenu et que chaque PNG se décode ; il ne
  vérifie pas le rendu (ADR 0006).
- La spec 072 décrit l'infobulle, les tailles d'affichage et le repli sans image.

## Questions tranchées

Toutes tranchées par Mathieu le 2026-11-05 (Q5 précisée, les autres selon la proposition).

- **Q1. Autres familles** : sorts et artefacts en images générées, comme les cartes action ; compétences et tactiques en icônes
  SVG avec infobulle ; sites en SVG sur le plateau.
- **Q2. Dépendance PNG** : la fonction d'images matricielles d'`egui_extras` et la crate `image` réduite au PNG sont acceptées ;
  ajoutées par la spec 072, qui en a besoin la première.
- **Q3. Taille** : 512 px pour le plus grand côté d'une carte ou d'une unité, 256 px pour une figurine d'infobulle.
- **Q4. Détourage** : le générateur ne rend pas de fond transparent ; on demande un fond uni de couleur franche, retiré par le
  script, puis vérifié à l'œil.
- **Q5. Noms propres** : jamais de nom du jeu dans les descriptions, y compris héros et ennemis ; la description s'appuie
  précisément sur le visuel du jeu, sans le citer.
- **Q6. Droits** : les [conditions de l'API Gemini](https://ai.google.dev/gemini-api/terms) (version du 2026-04-28, « Use of
  Generated Content ») disent que Google ne revendique pas la propriété des contenus générés, mais peut produire des contenus
  semblables pour d'autres ; l'utilisateur répond de l'usage qu'il en fait. Avec l'offre payante, les descriptions et les
  images ne servent pas à entraîner les modèles de Google. À relire si le dépôt devait devenir public.

## Révision (2026-11-06, spec 072)

Décidée par Mathieu en validant la spec 072 ; elle l'emporte sur ce qui précède.

- **Héros et monstres restent en SVG.** Une image générée n'apporte pas assez s'il ne sert qu'en infobulle d'une figurine ou
  dans la carte d'ennemi du combat ; on rendra plutôt leurs SVG plus soignés (spec 075). Les images générées sont réservées
  aux cartes : unités, actions (de base et avancées), sorts, artefacts. Plus d'infobulle d'image sur les figurines du plateau.
- **Format carré**, 512 × 512, pour les cartes comme pour les unités : la vignette occupe l'espace libre sous les boutons
  « jouer de côté » (spec 071). La taille de 256 px des figurines (Q3) ne sert plus.
- **Pas de détourage** : les images sont des scènes opaques, la Q4 ne s'applique plus.
- **Bande et vignette** : bande recadrée dans le panneau du bas (spec 077), bande dans les fenêtres aussi (spec [#32](../specs/issue-32-cartes-des-fenetres.md)) ; sans image
  en grand au survol.

## Révision (2026-11-11, issue privée 167)

Demandée par Mathieu dans l'issue privée 167 ; elle l'emporte sur la taille de 512 px ci-dessus.

- **256 × 256, palette de 256 couleurs** (PNG à palette) pour les cartes et les unités : quatre fois la vignette de 64 px,
  assez même sur un écran à haute densité, et chaque image passe d'environ 480 Ko à 50 Ko (18 Mo à 2 Mo pour le premier
  lot). Une carte affichée en plus grand demanderait de régénérer ou de reprendre le candidat d'origine.
- `scripts/illustrations.py retenir` produit directement ce format ; le test de la spec 072 (C2) le vérifie.
