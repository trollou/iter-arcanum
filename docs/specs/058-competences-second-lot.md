# 058 — Compétences : second lot

- Statut : implémentée (validée le 2026-11-13, questions Q1 à Q8 tranchées : réponses proposées acceptées ; PR privée 298)
- Jalon : J5
- Dépend de : 051 (compétences : pile, gain, `UseSkill`, `SkillUsed`, premier lot), 052 (moteur d'effets, `PlayMoment`,
  modificateurs), 013 (cartes jouées de côté), 020 à 025 (Source, dés en plus, cristaux, jetons, paiement du mana, or et noir),
  034 (Blessures, pioche, Assommé), 040 (interaction, Influence), 042 (unités : recrutement, jetons de commandement, activation),
  053 (modificateurs d'ennemis, `ArmorReduction`), 054 (sorts, Exposition), 056 (Tirage de mana, Tempête de mana, Apprentissage),
  057 (Distorsion spatiale, Appel aux armes), 044 (donjon et tombeau : unités qui restent à l'écart), 024 (retour en arrière) ;
  toutes implémentées
- Livret : MKUE p. 5 (*Using Skills* 1 à 4, dont 4.b, compétences posées au centre) ; p. 4 (*Preparing a new Round*, 2.g) ;
  p. 5 (*Mana*, or et noir) ; p. 7 (*Movement*, cases sûres, ennemis errants provoqués) ; p. 8 (*Recruiting*) ; texte des
  compétences : `docs/regles/cartes/heros-et-competences.md` (coopératives : section « Compétences coopératives »). Compendium,
  *Skill Tokens* S1, S2, S4, S5, et les entrées *Power of Pain*, *Invocation*, *Polarization*, *Ritual of Pain*, *Glittering
  Fortune*, *Flight*, *Universal Power*, *SOURCE Opening*, *Forward March*, *Leadership*, *Bonds of Loyalty*, *Resistance Break*,
  *I Don't Give a Damn*, *Who Needs Magic*, *Mana Overload* ; *Minimum Turn* S2, S3 ; *Miscellaneous* S12
- ADR : [0005 Contenu en données](../decisions/0005-contenu-en-donnees.md) (`content/skills.ron`), [0008 Moteur
  d'effets](../decisions/0008-moteur-d-effets.md) (modificateurs, durée « tour suivant »)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les **17 compétences** que la spec 051 laissait « Pas encore disponible » servent enfin. Elles demandent chacune une règle
nouvelle : une carte jouée de côté qui vaut plus (et une Blessure jouée de côté), un mana payé autrement (défausse, couleur
opposée), un vol au-dessus du plateau, un bonus sur une unité activée, un ennemi affaibli, un jeton de commandement de plus, et
les quatre compétences coopératives posées **au centre**, dont l'avantage différé revient au joueur **à son tour suivant** en
solo (Compendium, *Skill Tokens* S5).

Après cette spec, les quarante compétences du solo sont jouables ; il ne reste que celles du joueur fantôme (spec 061).

### Les dix-sept compétences

Noms français et texte court : déjà dans `assets/i18n/fr.json` (`skill.<id>.name`, `skill.<id>.text`, spec 051) ; texte
anglais complet : `heros-et-competences.md`.

| Groupe | Compétence (fr / en, héros) | Fréquence | Ce qu'il faut de nouveau |
|---|---|---|---|
| A. Côté | Souffrance sublimée (*Power of Pain*, Arythea) | tour | une Blessure jouée de côté, +2 |
| A. Côté | Flexibilité (*I Don't Give a Damn*, Tovak) | tour | une carte de côté +2, ou +3 (action avancée, sort, artefact) |
| A. Côté | Magie inutile (*Who Needs Magic?*, Tovak) | tour | une carte de côté +2, ou +3 sans dé de la Source ce tour |
| A. Côté | Pouvoir universel (*Universal Power*, Goldyx) | tour | un mana sur une carte de côté : +3, ou +4 (même couleur) |
| B. Mana | Invocation (Arythea) | tour | défausser une carte pour un jeton dépensé aussitôt |
| B. Mana | Polarisation (*Polarization*, Arythea) | tour | un mana payé comme sa couleur opposée |
| C. Points | Fortune clinquante (*Glittering Fortune*, Goldyx) | tour, en interaction | Influence 1 par couleur de cristal, pour l'interaction |
| C. Points | Marche en avant (*Forward March*, Norowas) | tour | Mouvement 1 par unité disponible et non blessée, 3 au plus |
| C. Points | Même pas mal (*I Feel No Pain*, Tovak) | tour, hors combat | défausser une Blessure, piocher une carte |
| D. Plateau | Vol (*Flight*, Goldyx) | manche | une case gratuite, ou deux pour 2 Mouvement, sans provoquer |
| E. Unités | Charisme (*Leadership*, Norowas) | tour | bonus à une unité activée |
| E. Unités | Loyauté sans faille (*Bonds of Loyalty*, Norowas) | permanente | jeton de commandement, recrutement à −5 |
| F. Combat | Botte secrète (*Resistance Break*, Tovak) | tour | un ennemi : Armure −1 par résistance |
| G. Centre | Rituel de guérison (*Ritual of Pain*, Arythea) | manche, hors combat | jeter jusqu'à 2 Blessures ; au tour suivant, une Blessure de côté +3 |
| G. Centre | Embrasure de la Source (*Source Opening*, Goldyx) | manche | relancer un dé ; au tour suivant, un dé de plus et son cristal |
| G. Centre | Temps calme (*Calming the Weather*, Norowas) | manche | terrains −2 ce tour ; au tour suivant, −1 |
| G. Centre | Surcharge de mana (*Mana Overload*, Tovak) | manche | un jeton ; au tour suivant, +4 sur une carte alimentée de cette couleur |

## Règles

### Générales (rappel de la spec 051, inchangé)

- **Moment, une seule fois, une option** (MKUE p. 5, *Using Skills* ; Compendium, *Skill Tokens* S1, S2) : comme la spec 051.
  Une compétence qui agit **sur** une carte, un paiement ou une unité ne modifie **qu'une** carte, un mana ou une unité.
- **Pas l'action du tour, pas une carte jouée** (Compendium, *Skill Tokens* S1 ; *Minimum Turn* S3) : Même pas mal, Invocation
  et Rituel de guérison défaussent ou jettent une Blessure **sans** compter comme une carte jouée pour le tour minimum (MKUE
  p. 6, 5.a). Au contraire, une Blessure **jouée de côté** (Souffrance sublimée, avantage du Rituel de guérison) est une carte
  jouée (Compendium, *Minimum Turn* S2).
- **Une compétence qui accompagne une autre action** s'utilise **avec** elle, d'un seul coup, jamais à l'avance (Q6) : la carte
  jouée de côté porte la compétence (groupe A, avantage du Rituel de guérison), le paiement porte Invocation ou Polarisation
  (groupe B), l'activation d'unité porte Charisme. Rien ne reste « en attente » d'une carte qui ne viendrait pas, et la
  compétence ne se propose que s'il existe une carte, un paiement ou une unité qui la reçoit maintenant.

### A. Cartes jouées de côté

- **Valeur de remplacement** (texte ; Compendium, *Who Needs Magic* S2) : ces compétences disent « +2 au lieu de +1 » : elles
  **remplacent** la valeur de la carte de côté, elles ne s'ajoutent pas. Une carte de côté n'en reçoit **qu'une** (Compendium,
  *I Don't Give a Damn* S1, *Universal Power* S2, S3, *Power of Pain* S2). Ce que donne la carte ne change pas de nature :
  Mouvement, Influence, Attaque physique (ni à distance ni de siège) ou Parade physique, selon le moment (spec 013 ; Compendium,
  *Universal Power* S1).
- **Blessures** : une Blessure ne se joue jamais de côté (spec 034, `wound_cannot_be_played`), **sauf** avec Souffrance
  sublimée (+2) ou l'avantage du Rituel de guérison (+3). Flexibilité, Magie inutile et Pouvoir universel ne s'appliquent pas
  à une Blessure (Compendium, *I Don't Give a Damn* S3, *Power of Pain* S2). La Blessure jouée de côté va dans la zone de jeu
  et, à la fin du tour, dans la défausse avec les autres cartes jouées (texte ; Compendium, *Ritual of Pain* S2). Elle reste une
  Blessure reçue pour Assommé si elle a été reçue dans ce combat (Compendium, *Power of Pain* S3 : rien ne change,
  `knock_out_wounds` compte les Blessures reçues).
- **Flexibilité** : +2 ; +3 si la carte est une action avancée, un sort ou un artefact. Les actions de base propres au héros
  (Résistance au froid, etc.) sont des actions de base : +2 (Compendium, *I Don't Give a Damn* S2).
- **Magie inutile** : deux options, **+2**, ou **+3 si aucun dé de la Source n'a servi ce tour** ; après +3, plus aucun dé de
  la Source ne sert ce tour (Compendium, *Who Needs Magic* Q1). « Servir », c'est **payer avec un dé** : le dé du tour, un dé
  en plus (Tirage de mana de base, avantage d'Embrasure de la Source), un dé épuisé changé par Polarisation. Mettre des dés sur
  une couleur (Tirage de mana renforcé, Tempête de mana) n'est pas s'en servir (lecture étroite du compendium, *Who Needs
  Magic* Q1, note ; Q4). Le joueur choisit +2 pour garder ses dés.
- **Pouvoir universel** : le joueur ajoute **un mana** à la carte jouée de côté (payé comme tout mana : dé, cristal, jeton, avec
  Polarisation ou Invocation au besoin) : **+3**, ou **+4** si la carte est une action (de base ou avancée) ou un sort **de la
  couleur de ce mana**. Un mana or vaut la couleur de base choisie (MKUE p. 5) ; un mana noir de nuit ne donne que +3, même
  sur un sort (Compendium, *Universal Power* S5) ; un artefact n'a pas de couleur : +3 (S6) ; jamais un dé épuisé (S4). Ce
  mana n'**alimente** pas la carte : il ne déclenche pas Surcharge de mana (Q5).

### B. Mana payé autrement

- **Invocation** : en payant un mana (carte, sort, capacité d'unité, achat d'un sort, tribut d'un autel, cristal payé… :
  partout où l'on paie un mana aujourd'hui), le joueur **défausse une carte de sa main** au lieu d'un dé, d'un cristal ou d'un
  jeton : une Blessure paie du **rouge ou du noir**, une autre carte du **blanc ou du vert**. Le jeton gagné est dépensé
  aussitôt, sinon la compétence ne sert pas (texte) : il n'existe que dans ce paiement. La carte va dans la défausse (pas la
  zone de jeu). Un jeton noir ne paie que là où le noir est permis (Compendium, *Invocation* S1). Utilisable en combat
  (Compendium, *Invocation* S2). La carte défaussée était en main **avant** l'action qui paie : une Blessure gagnée par la carte
  qu'on est en train de jouer ne sert pas (Compendium, *Invocation* S2, second exemple, avec Rituel de sang dans le jeu de
  base). Invocation puis Polarisation sur le même mana est permis (Compendium, *Invocation* S3).
- **Polarisation** : en payant un mana, le joueur en donne un de la **couleur opposée** (diagramme des jetons : rouge ↔ bleu,
  blanc ↔ vert, or ↔ noir), sous n'importe quelle forme : dé, cristal, jeton (Compendium, *Polarization* S4). Deux cas du
  texte : **de jour**, un mana noir paie n'importe quelle couleur de base, mais jamais le noir d'un sort renforcé ; **de nuit**,
  un mana or paie le noir d'un sort renforcé, et rien d'autre. Un **dé épuisé** (noir de jour, or de nuit) peut être pris ainsi
  pour le changer (Compendium, *Polarization* S1) ; il compte comme le dé du tour et se relance à la fin du tour. On ne change
  qu'un mana **qu'on dépense** (S2). Le mana obtenu suit ensuite les règles de sa nouvelle couleur, et compte pour cette couleur
  partout (Surcharge de mana, Pouvoir universel ; S3). Tirage de mana renforcé et Polarisation ne se combinent pas dans un même
  effet (S5) : Polarisation agit sur un paiement, pas sur un dé posé.

### C. Points

- **Fortune clinquante** (texte ; Compendium, *Glittering Fortune* S1) : seulement **pendant une interaction** (spec 040,
  `InteractionStarted`) : Influence 1 par couleur de cristal de l'inventaire au moment de l'utiliser (0 à 4). Cette Influence
  ne sert **qu'à l'interaction** (recruter, acheter un soin, un sort, une action avancée, payer un autel…) : elle va dans une
  réserve à part, dépensée **avant** l'Influence ordinaire pour un achat d'interaction, perdue à la fin du tour comme l'autre
  (Q3). Elle ne sert pas à Apprentissage (spec 056, effet de carte et non achat d'interaction), ni à une conversion (Diplomatie).
  Les cristaux comptés peuvent ensuite être dépensés (S1).
- **Marche en avant** (texte) : Mouvement 1 par unité **disponible et non blessée**, 3 au plus, au moment de l'utiliser ;
  une unité activée ensuite garde son Mouvement (Compendium, *Forward March* S2). Comme tout Mouvement, en phase de mouvement
  (spec 052, `PlayMoment`). Aucune unité : la compétence ne se propose pas.
- **Même pas mal** (texte) : hors combat, défausser **une** Blessure de la main (dans la défausse, pas dans la pile des
  Blessures) ; **si** une Blessure a été défaussée, piocher une carte (sans remélanger, spec 034). Ne se propose qu'avec une
  Blessure en main. Ce n'est pas un Soin (Compendium, *Miscellaneous* S9) ; utilisable après un combat où l'on a été Assommé
  (Compendium, *Combat* Q22).

### D. Vol

- **Vol** (texte ; Compendium, *Flight*) : une fois par manche, en phase de mouvement, deux options :
  - **une case voisine, gratuite** ;
  - **deux cases pour 2 points de Mouvement** (pris dans la réserve de Mouvement).
- La case d'arrivée est une **case sûre** (MKUE p. 7 ; `is_safe_space`, sans ennemi errant) ; la case survolée peut être
  n'importe laquelle **révélée** : montagne, lac, site fortifié non conquis, ennemi errant (Compendium, *Flight* S3) ; jamais
  une case hors des tuiles posées, et le vol n'explore pas (S4).
- Le vol **ne provoque pas** les ennemis errants (texte), ni au départ, ni à l'arrivée. Le terrain ne coûte rien.
- **Distorsion spatiale** (spec 057) : « voisine » prend son sens du tour, donc une case à distance 2 est voisine ; l'option à
  deux cases va jusqu'à distance 4 (Compendium, *Flight* Q1). Ni Ailes du vent ni un autre effet de mouvement ne se combinent
  au vol (S2) : il déplace le héros aussitôt, d'un seul coup.
- C'est un déplacement : il compte comme s'être déplacé (pas de repos ensuite, spec 034) et se fait avant l'action du tour.
  Arriver sur un site agit comme un déplacement ordinaire (un site non fortifié n'est pas attaqué ; site fortifié non conquis :
  pas une case sûre, refusé).

### E. Unités

- **Charisme** (texte ; Compendium, *Leadership*) : en **activant** une unité, le joueur ajoute à la capacité activée
  **+3 à une Parade**, **+2 à une Attaque**, ou **+1 à une Attaque à distance** (pas de siège), quel que soit son élément, qui
  reste celui de la capacité. La capacité doit déjà donner ce genre de points (S1). En phase d'attaque, toute Attaque d'unité
  (à distance, de siège) est une Attaque : +2 (S2) ; en phase à distance, une Attaque de siège n'a rien (S2). Une Parade qui
  compte double contre Vif garde ce double, bonus compris (S4, Gardes d'Utem). Une unité de l'offre activée par Appel aux armes
  le reçoit aussi (Q3 du compendium ; spec 057). Une seule unité par tour.
- **Loyauté sans faille** (texte ; Compendium, *Bonds of Loyalty*) : compétence **permanente**, sans fréquence, sans bouton.
  - **Quand elle est gagnée** (au choix du niveau pair, spec 051) : **deux unités régulières** sont tirées de la pioche des
    unités régulières et ajoutées à l'offre d'unités, tout de suite, même au dernier tour d'une manche (Q2 du compendium). Elles
    partent avec l'offre à la manche suivante et ne sont pas remplacées (S8 ; l'offre est refaite à chaque manche, MKUE p. 4).
    Pioche régulière vide : autant que possible.
  - Elle devient un **jeton de commandement** de plus : une unité recrutée **sous ce jeton** coûte **5 Influence de moins**
    (minimum 0), régulière ou d'élite (S5), par le recrutement ordinaire (spec 042). Le joueur dit, en recrutant, si l'unité
    va sous ce jeton ; une seule unité y tient.
  - Cette unité **sert même là où les unités ne servent pas** : donjon, tombeau, monastère incendié (Q1 ; spec 044,
    `units_sit_out`), mais pas blessée ni épuisée (S6). Elle ne peut **pas être renvoyée** (pour libérer un jeton de
    commandement). Si elle disparaît (Paralysant, combat), le jeton se libère et une nouvelle unité s'y recrute avec la
    réduction (Q3).

### F. Combat

- **Botte secrète** (texte ; Compendium, *Resistance Break*) : en combat, à toute phase, le joueur choisit un ennemi du combat
  pas encore vaincu : son **Armure baisse de 1 par résistance** (physique, Feu, Glace : S3), jamais sous 1, jusqu'à la fin du
  combat (modificateur d'ennemi `ArmorReduction`, spec 053). Le nombre de résistances est compté **au moment** de l'utiliser :
  après Exposition (spec 054, résistances perdues), elle ne donne rien ; avant, les deux s'additionnent (S2). Ennemi sans
  résistance : pas proposé. L'immunité arcanique (S1) n'existe pas dans le jeu de base.

### G. Compétences posées au centre (coopératives)

- **Mécanisme** (MKUE p. 5, *Using Skills* 4.b ; Compendium, *Skill Tokens* S4, S5) : une fois par manche, la compétence est
  **posée au centre** avec son effet immédiat. En solo, son **avantage différé** revient au joueur **pendant son tour
  suivant** (S5), puis la compétence lui revient **retournée** jusqu'à la manche suivante (MKUE p. 4, 2.g) : elle ne sert donc
  qu'une fois par manche, comme les autres.
- **Durée** (Q1) : l'avantage n'existe que pendant le **tour suivant** du joueur : utilisé ou non, il disparaît à la fin de ce
  tour. Il disparaît aussi avec la fin de la manche (S4) : posée au dernier tour d'une manche, la compétence ne donne pas son
  avantage. Un tour supplémentaire (Distorsion temporelle, spec 057) est un tour suivant.
- **Déclenchement** (Q2) : l'avantage de **Temps calme** s'applique d'office au début du tour suivant, celui de **Surcharge de
  mana** d'office à la première carte qui le remplit ; ceux du **Rituel de guérison** et d'**Embrasure de la Source**
  s'utilisent au choix du joueur, comme une option.
- **Rituel de guérison** (*Ritual of Pain*) : hors combat, jeter **0, 1 ou 2** Blessures de la main dans la pile des Blessures
  (Compendium, *Ritual of Pain* S1 : zéro permis). Avantage : **une** Blessure jouée de côté pour **+3**, en combat aussi (S1),
  comme Souffrance sublimée (groupe A) : pas avec une autre compétence du groupe A sur la même carte.
- **Embrasure de la Source** (*Source Opening*) : le joueur **peut relancer** un dé de la Source (Compendium, *SOURCE Opening*
  S1), un dé qui n'est pas pris ce tour. Avantage : **un dé de plus** de la Source, d'une **couleur de base** (jamais or, S5),
  avant ou sans le dé du tour (S4) ; s'il s'en sert, il gagne **un cristal de cette couleur** ; ce dé est ensuite, au choix du
  joueur à la fin du tour, relancé ou laissé tel quel, avant la relance des autres (S1, S3).
- **Temps calme** (*Calming the Weather*) : le coût de **tous les terrains** baisse de 2 pour ce tour, jamais sous 1
  (modificateur `TerrainCost` existant, spec 052). Avantage : −1, jamais sous 1, pour le tour suivant.
- **Surcharge de mana** (*Mana Overload*) : le joueur choisit une couleur autre que l'or et gagne **un jeton** de cette
  couleur, qu'il n'est pas obligé de dépenser (Compendium, *Mana Overload* S5). Avantage : la **première** carte du tour
  suivant **alimentée** par un mana de cette couleur et qui donne **elle-même** du Mouvement, de l'Influence, une Attaque ou une
  Parade reçoit **+4** (texte ; S6). « Alimentée » (Q5) : le mana payé pour l'effet renforcé d'une action, ou le mana de la
  couleur d'un sort (de base ou renforcé) ; pas le mana d'une capacité d'unité (S1), ni un mana payé à l'intérieur d'un effet
  (Éclair de mana de base, S2), ni celui de Pouvoir universel. Un mana or ou polarisé compte pour la couleur qu'il paie (S3 de
  *Polarization*). Une carte alimentée qui ne donne pas ces points ne déclenche rien et l'avantage attend (S3). Les +4 vont aux
  points de l'option jouée ; si elle en donne de deux sortes, aux premiers de l'option (O4, lecture proposée).

### Exemple : Goldyx, Pouvoir universel et Embrasure de la Source

Goldyx a gagné Pouvoir universel et Embrasure de la Source. Au tour 1, en phase de mouvement, il utilise Embrasure de la Source
et relance le dé noir de la Source (inutile de jour) : il sort vert. Au tour 2, il prend le dé vert en plus de son dé du tour :
il gagne un cristal vert. Il joue Marche (*March*, verte, `basic_actions.ron`) de côté avec Pouvoir universel, en y mettant le
mana du dé vert : Mouvement 4 au lieu de 1. À la fin du tour, il laisse le dé vert tel quel ; Embrasure de la Source lui revient
retournée jusqu'à la manche suivante.

## Modèle

Termes du glossaire : Compétence, Jouer de côté, Source, Dé épuisé, Jeton de commandement, Recruter, Renvoyer, Case sûre,
Provoquer, Résistance, Interaction. **Termes nouveaux** (ajoutés au glossaire par cette PR) :

| Livret (EN) | Français | Code |
|---|---|---|
| Put (a Skill token) in the center | Poser (une compétence) au centre | `SkillState::InCenter` |
| Secondary advantage (of an interactive Skill) | Avantage différé | `CenterAdvantage` |
| Opposite color (mana token diagram) | Couleur opposée | `ManaColor::opposite` |
| Command token of Bonds of Loyalty | Jeton de Loyauté | `PlayerState::bonds_unit` |

Données nouvelles :

- **Contenu** : les 17 entrées de `skills.ron` passent de `Unsupported` à leurs options ou à une forme propre (`Sideways`,
  `ManaPayment`, `Flight`, `Leadership`, `Permanent`, `Center`…), au choix du fil d'implémentation (ADR 0005 : les
  mécaniques restent en Rust, le fichier ne fait que les choisir).
- **Joueur** : l'état « au centre » d'une compétence et son avantage du tour suivant (sorte, couleur de Surcharge de mana) ;
  l'Influence d'interaction (Fortune clinquante) ; « plus de dé ce tour » (Magie inutile à +3) ; le dé d'Embrasure de la
  Source à relancer ou non ; l'unité sous le jeton de Loyauté sans faille.
- **Actions** : la carte jouée de côté porte une compétence (et le mana de Pouvoir universel) ; un paiement de mana peut être
  une Invocation (la carte défaussée) ou une Polarisation (le mana d'origine) ; l'activation d'unité porte Charisme ;
  `UseSkill` reçoit l'ennemi (Botte secrète), la case (Vol), la couleur (Surcharge de mana), le dé (Embrasure de la Source) ;
  `RecruitUnit` dit si l'unité va sous le jeton de Loyauté. Noms exacts au fil d'implémentation.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` de côté avec compétence | compétence non possédée, utilisée, retournée ; Blessure sans Souffrance sublimée ni avantage du Rituel ; carte non-Blessure avec Souffrance sublimée ; Pouvoir universel sans mana, mana épuisé ; Magie inutile +3 après un dé ; deux compétences | `SkillUsed`, paiement éventuel, `CardPlayed` existant avec la valeur remplacée |
| Paiement Invocation / Polarisation (dans toute action qui paie) | compétence non possédée ou utilisée ; carte absente de la main, ou de la mauvaise sorte pour la couleur ; couleur non opposée ; noir ou or hors des cas permis | `SkillUsed`, `CardDiscarded` (Invocation), puis le paiement comme aujourd'hui (`DieTaken`, `CrystalUsed`…) |
| `UseSkill` Fortune clinquante | pas d'interaction en cours | `SkillUsed`, Influence d'interaction gagnée |
| `UseSkill` Marche en avant | aucune unité disponible et non blessée ; hors phase de mouvement | `SkillUsed`, Mouvement |
| `UseSkill` Même pas mal | en combat ; pas de Blessure en main | `SkillUsed`, `CardDiscarded`, `CardDrawn` (si le deck en a) |
| `UseSkill` Vol `{ to }` | hors phase de mouvement ; case non révélée, pas sûre, avec ennemi errant ; trop loin pour l'option ; 2 Mouvement manquants | `SkillUsed`, `HeroMoved` (sans provocation) |
| `ActivateUnit` avec Charisme | compétence utilisée ; capacité sans ce genre de points ; siège en phase à distance | `SkillUsed`, activation existante avec le bonus |
| `RecruitUnit` sous Loyauté | pas de Loyauté sans faille ; jeton déjà pris | recrutement existant, coût −5 |
| `UseSkill` Botte secrète `{ enemy }` | hors combat ; ennemi vaincu ou absent ; sans résistance | `SkillUsed`, modificateur d'ennemi |
| `UseSkill` coopérative | compétence retournée ou au centre ; Rituel en combat, Blessures manquantes ; Surcharge en or | `SkillUsed`, `SkillPlacedInCenter`, effet immédiat (`WoundThrownAway`, `DieRerolled`, modificateur, `ManaTokenGained`) |
| Avantage du centre (Rituel, Embrasure) | pas l'avantage de cette compétence ce tour | `CenterAdvantageUsed`, puis la carte de côté ou le dé (`CrystalGained`) |
| `GainAdvancedAction` / `ChooseSkill` Loyauté | — | `SkillGained`, `OfferCardDealt` ×2 (unités régulières) |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `SkillPlacedInCenter { player, skill, color }` | compétence au centre ; avantage au tour suivant | non | non |
| `CenterAdvantageUsed { player, skill }` | avantage consommé | non | non |
| `CenterSkillReturned { player, skill }` | fin du tour suivant ou de la manche : compétence retournée, avantage perdu | non | non |
| `SourceDieKept { player, die }` (fin du tour, Embrasure) | le dé n'est pas relancé | non | non (la relance des autres l'est) |

`DieRerolled`, `CardDiscarded`, `CardDrawn`, `HeroMoved`, `OfferCardDealt`, `CrystalGained`, `ManaTokenGained`,
`WoundThrownAway` existent ; les relances et les pioches restent des points de non-retour (spec 024).

## Comportements attendus

### Contenu

- **C1** — Les 40 compétences du contenu sont toutes jouables : plus aucune `Unsupported` (`all_skills_are_supported`) —
  spec 051 C1 mis à jour.

### A. Cartes jouées de côté

- **C2** — Arythea avec Souffrance sublimée et une Blessure en main, en phase de mouvement : jouer la Blessure de côté donne
  Mouvement 2 ; elle est dans la zone de jeu, puis dans la défausse à la fin du tour (`power_of_pain_plays_wound_sideways`) —
  texte ; Compendium, *Ritual of Pain* S2.
- **C3** — Sans la compétence, la Blessure ne se joue toujours pas (`wound_cannot_be_played`, deroule_combat.rs, inchangé) ;
  avec Souffrance sublimée, une carte non-Blessure de côté n'est pas proposée avec elle (`power_of_pain_only_for_wounds`).
- **C4** — Une Blessure jouée avec Souffrance sublimée satisfait le tour minimum (`wound_played_sideways_counts_as_played`) —
  Compendium, *Minimum Turn* S2.
- **C5** — Flexibilité : Rage (`basic_actions.ron`, action de base rouge) de côté en phase d'attaque : Attaque 2 ; Coup de sang
  (`BloodRage`, action avancée) : Attaque 3 ; Bannière de gloire (artefact) : 3 ; une Blessure : refusé
  (`i_dont_give_a_damn_values`) — texte ; Compendium, *I Don't Give a Damn* S2, S3.
- **C6** — Magie inutile sans dé pris : options +2 et +3 ; après +3, payer avec le dé du tour est refusé et Tirage de mana de
  base n'est plus proposé ; avec le dé du tour déjà pris, seule +2 est proposée (`who_needs_magic_three_forbids_dice`) —
  Compendium, *Who Needs Magic* Q1.
- **C7** — Pouvoir universel : Marche (verte) de côté avec un jeton vert : Mouvement 4 ; avec un jeton rouge : 3 ; Éclair de feu
  (sort rouge) de côté en phase d'attaque avec un cristal rouge : Attaque 4 ; de nuit avec un mana noir : 3 ; un artefact : 3 ;
  un dé noir de jour : refusé (`universal_power_values`) — texte ; Compendium, *Universal Power* S4 à S6.
- **C8** — Deux compétences du groupe A sur la même carte : refusé ; Pouvoir universel sur une Blessure jouée avec Souffrance
  sublimée : refusé (`sideways_skills_do_not_stack`) — Compendium, *Who Needs Magic* S2, *Power of Pain* S2.
- **C9** — Les points de côté gardent leur nature : Pouvoir universel de côté en phase à distance n'est pas proposé (pas
  d'Attaque à distance de côté) (`sideways_skill_keeps_sideways_kind`) — Compendium, *Universal Power* S1.

### B. Mana payé autrement

- **C10** — Invocation : Arythea joue Rage renforcée en défaussant une Blessure (rouge) ; la Blessure est dans la défausse, pas
  dans la zone de jeu ; une carte non-Blessure ne paie pas du rouge ; une non-Blessure paie le vert du renforcement de Marche
  (`invocation_pays_by_discarding`) — texte.
- **C11** — Invocation de jour : le noir n'est proposé que là où il est permis (sort renforcé de nuit, donjon) ; en combat,
  permise (`invocation_black_only_where_allowed`) — Compendium, *Invocation* S1, S2.
- **C12** — Invocation ne compte pas pour le tour minimum : après une Invocation seule, finir le tour demande encore une
  défausse (`invocation_is_not_a_played_card`) — Compendium, *Minimum Turn* S3.
- **C13** — Polarisation : un cristal bleu paie le rouge de Rage renforcée ; un jeton blanc paie le vert de Marche ; un rouge
  ne paie pas le blanc (`polarization_pays_opposite_color`) — texte.
- **C14** — Polarisation de jour : le dé noir épuisé de la Source se prend et paie du rouge ; il compte comme le dé du tour et
  se relance à la fin du tour ; il ne paie pas le noir d'un sort renforcé (`polarization_black_by_day`) — Compendium,
  *Polarization* S1.
- **C15** — Polarisation de nuit : le dé or épuisé paie le noir d'Éclair de feu renforcé, et rien d'autre
  (`polarization_gold_by_night`) — texte.
- **C16** — Invocation puis Polarisation sur le même mana : une Blessure (rouge) paie le bleu de Détermination renforcée ; les
  deux compétences sont utilisées ce tour (`invocation_then_polarization`) — Compendium, *Invocation* S3.
- **C17** — Une compétence de paiement utilisée ce tour n'est plus proposée dans le choix du mana ; elle l'est au tour suivant
  (`payment_skill_once_a_turn`) — MKUE p. 5, 4.c.

### C. Points

- **C18** — Fortune clinquante pendant l'interaction au village, avec un cristal rouge, deux bleus et un blanc : Influence 3 ;
  hors interaction, pas proposée (`glittering_fortune_counts_crystal_colors`) — texte ; Compendium, *Glittering Fortune* S1.
- **C19** — L'Influence de Fortune clinquante paie un recrutement ; elle ne paie pas Apprentissage
  (`glittering_fortune_influence_only_for_interaction`) — texte.
- **C20** — Marche en avant avec Paysans disponibles, Bûcherons blessés et Arbalétriers d'Utem épuisés : Mouvement 1 ; avec
  quatre unités disponibles et non blessées : 3 (`forward_march_counts_ready_unwounded_units`) — texte.
- **C21** — Même pas mal hors combat avec deux Blessures en main : une Blessure dans la défausse, une carte piochée ; sans
  Blessure en main ou en combat, pas proposée ; deck vide : la Blessure part, rien n'est pioché
  (`i_feel_no_pain_discards_wound_and_draws`) — texte.

### D. Vol

- **C22** — Bac à sable, de jour, héros en `(0, -3)` (forêt, voisine des orques errants en `(1, -4)`) : Vol, option gratuite,
  vers `(1, -3)` (forêt, clairière magique, voisine des orques aussi) : le héros y est, aucun Mouvement dépensé, pas de combat
  provoqué ; Vol est retourné (`flight_moves_without_provoking`) — texte ; `docs/reference/bac-a-sable.md`.
- **C23** — Option deux cases depuis `(1, -3)` par le lac `(2, -4)` jusqu'à `(2, -3)` (village) avec 2 Mouvement : permise ;
  arrivée sur le lac, sur les orques, ou case non révélée : refusée ; avec 1 Mouvement : refusée (`flight_two_spaces`) —
  Compendium, *Flight* S3, S4.
- **C24** — Avec Distorsion spatiale jouée, l'option gratuite atteint une case à distance 2 (`flight_with_space_bending`) —
  Compendium, *Flight* Q1.
- **C25** — Après le vol, le repos n'est plus permis ; en combat ou après l'action du tour, Vol n'est pas proposé
  (`flight_is_a_move`) — MKUE p. 6, 7.

### E. Unités

- **C26** — Charisme : Arbalétriers d'Utem en phase à distance : Attaque à distance 2 + 1 = 3 ; Paysans en phase de parade :
  Parade 2 + 3 = 5 ; Arbalétriers d'Utem en phase d'attaque, Attaque à distance 2 : + 2 = 4 (`leadership_unit_bonus`) — texte ;
  Compendium, *Leadership* S1, S2.
- **C27** — Charisme sur la Parade de Glace 4 des Golems gardiens (`GuardianGolems`, mana bleu) : Parade de Glace 7 ; sur les
  Gardes d'Utem (Parade 4 qui compte double contre Vif) contre les Chevaucheurs de loups (Vif) : 14
  (`leadership_keeps_element_and_swift`) — Compendium, *Leadership* S4.
- **C28** — Charisme une seule fois par tour : la seconde activation du tour n'a plus le bonus proposé
  (`leadership_once_a_turn`) — Compendium, *Skill Tokens* S2.
- **C29** — Loyauté sans faille gagnée au niveau pair : deux unités régulières de plus dans l'offre d'unités ; elles en
  partent à la nouvelle manche sans être remplacées (`bonds_of_loyalty_adds_two_units`) — texte ; Compendium, *Bonds of
  Loyalty* Q2, S8.
- **C30** — Recruter sous le jeton de Loyauté : Arbalétriers d'Utem (6 Influence) pour 1 ; Herboristes (3) pour 0 ; une seconde
  unité sous ce jeton : refusée ; avec tous les jetons ordinaires pris, le jeton de Loyauté reçoit l'unité sans renvoi
  (`bonds_of_loyalty_discounted_recruit`) — texte ; S5.
- **C31** — L'unité sous Loyauté ne peut pas être renvoyée ; elle s'active dans un donjon (où les autres restent à l'écart),
  mais pas blessée (`bonds_unit_fights_where_units_sit_out`) — texte ; Q1, S6.
- **C32** — L'unité sous Loyauté détruite (Paralysant) : le jeton se libère, une nouvelle s'y recrute à −5
  (`bonds_token_freed_when_unit_dies`) — Q3.

### F. Combat

- **C33** — Botte secrète contre les Gardiens d'Altem (Armure 7, trois résistances) : Armure 4 ; contre le Golem de glace
  (Armure 4, deux résistances) : 2 ; contre les Rôdeurs (sans résistance) : pas proposée (`resistance_break_lowers_armor`) —
  texte ; Compendium, *Resistance Break* S3.
- **C34** — Botte secrète après Exposition sur le même ennemi : rien ; avant Exposition : la baisse reste
  (`resistance_break_counts_resistances_when_used`) — Compendium, *Resistance Break* S2.
- **C35** — Armure jamais sous 1 : un ennemi d'Armure 2 avec deux résistances passe à 1 (`resistance_break_minimum_one`) —
  texte. (Exemple pris du contenu au fil moteur : aucun ennemi du jeu de base n'a Armure 2 et deux résistances ; le test peut
  combiner Botte secrète et une autre baisse d'Armure.)

### G. Compétences posées au centre

- **C36** — Rituel de guérison hors combat, deux Blessures en main, option « jeter 2 » : elles vont dans la pile des
  Blessures ; la compétence est au centre ; en combat, pas proposée ; « jeter 0 » permis (`ritual_of_pain_throws_wounds`) —
  texte ; Compendium, *Ritual of Pain* S1.
- **C37** — Au tour suivant, une Blessure jouée de côté avec l'avantage du Rituel : Parade 3 en combat ; une seconde Blessure :
  refusée ; à la fin de ce tour, la compétence est retournée (`ritual_of_pain_advantage_next_turn`) — texte ; S1.
- **C38** — Avantage non utilisé : il disparaît à la fin du tour suivant, la compétence est retournée jusqu'à la manche
  suivante ; posée au dernier tour d'une manche : pas d'avantage, face visible à la manche suivante
  (`center_advantage_lasts_one_turn`) — Compendium, *Skill Tokens* S4, S5.
- **C39** — Embrasure de la Source : relancer un dé est un point de non-retour ; « ne pas relancer » permis ; au tour suivant,
  un dé vert de plus paie un mana et donne un cristal vert ; un dé or n'est pas proposé en plus ; à la fin du tour, le joueur
  garde ce dé ou le relance (`source_opening_extra_die_and_crystal`) — Compendium, *SOURCE Opening* S1, S3 à S5.
- **C40** — Temps calme de jour : une colline coûte 1 ce tour, une plaine 1 ; au tour suivant, une colline 2, une plaine 1
  (`calming_the_weather_reduces_terrain`) — texte.
- **C41** — Surcharge de mana, couleur rouge : un jeton rouge gagné ; au tour suivant, Rage renforcée payée en rouge : Attaque
  4 + 4 = 8 ; une seconde carte rouge renforcée ensuite : pas de bonus (`mana_overload_first_powered_card`) — texte.
- **C42** — Surcharge de mana : Pouvoir universel avec un mana rouge, une capacité d'unité payée en rouge, ou une carte rouge
  renforcée qui ne donne pas elle-même de points (Au cœur du combat, `IntoTheHeat` : bonus d'unités) ne déclenchent rien, et l'avantage reste
  (`mana_overload_needs_a_powered_points_card`) — Compendium, *Mana Overload* S1, S3, S6.
- **C43** — Surcharge de mana au tour même où elle est posée : aucun bonus (`mana_overload_not_on_same_turn`) — Compendium,
  *Skill Tokens* S5 (Q1).

### Vue, retour en arrière, persistance

- **C44** — La vue montre, pour chaque compétence, ses options utilisables maintenant (y compris les cartes de côté, les
  paiements et les unités qui peuvent la recevoir) ; l'état « au centre » et l'avantage du tour ; l'Influence d'interaction ;
  « plus de dé ce tour » ; l'unité sous Loyauté (`view_shows_second_batch_skills`).
- **C45** — Retour en arrière : Vol, une carte de côté avec compétence, Polarisation s'annulent ; Même pas mal (pioche) et
  Embrasure de la Source (relance) non (`second_batch_skill_undo`) — spec 024.
- **C46** — Une partie sauvegardée avec une compétence au centre, l'Influence d'interaction, l'unité sous Loyauté et le dé
  d'Embrasure se recharge à l'identique (`second_batch_skills_survive_save_and_load`) — spec 024.

## Cas limites et refus

- **L1** — Compétence du groupe A ou B, Charisme : jamais proposée seule ; seulement avec une carte, un paiement ou une unité qui
  la reçoit (Q6).
- **L2** — Invocation avec la seule carte de la main qu'on est en train de jouer : refusée (la carte jouée n'est plus en main).
- **L3** — Polarisation sur un mana de Tirage de mana renforcé (dés posés) : seulement au moment où ce mana paie, pas sur le dé
  posé (Compendium, *Polarization* S5).
- **L4** — Fortune clinquante sans cristal : Influence 0, pas proposée.
- **L5** — Vol avec Distorsion spatiale et l'option deux cases : jusqu'à distance 4, chaque pas « voisin » au sens du tour.
- **L6** — Loyauté sans faille avec la pioche des unités régulières vide : la compétence se gagne, sans unité ajoutée.
- **L7** — Embrasure de la Source sans dé relançable (tous pris) : seulement « ne pas relancer ».
- **L8** — Surcharge de mana dont le jeton de la couleur n'est jamais utilisé : l'avantage reste valable (S5).
- **L9** — Magie inutile +3, puis une carte qui donnerait un dé en plus (Tirage de mana de base) : non proposée ; un dé posé par
  Tirage de mana renforcé donne des jetons, permis (Q4).
- **L10** — Charisme sur une capacité qui donne Attaque et Parade au choix (Paysans) : le bonus suit l'option activée.

## Information cachée

Rien de caché : ces compétences agissent sur des informations déjà visibles. Le dé relancé par Embrasure de la Source et la
carte piochée par Même pas mal sont des points de non-retour (spec 024).

Champs ajoutés ou étendus dans `PlayerView` :

- `skills` (spec 051) : pour chaque compétence, son état `InCenter` en plus de `Ready`, `UsedThisTurn`, `Flipped` ; ses options
  utilisables maintenant, qui incluent désormais les cibles (cartes de la main pour le groupe A, ennemis pour Botte secrète,
  cases pour Vol, couleurs pour Surcharge de mana) ; `unsupported` toujours faux ;
- `center_advantages` : les avantages du centre de ce tour (compétence, couleur, utilisé ou non) ;
- `interaction_influence` : l'Influence d'interaction (Fortune clinquante) ;
- `source_dice_forbidden` : plus de dé de la Source ce tour (Magie inutile à +3) ;
- `bonds_unit` : l'unité sous le jeton de Loyauté sans faille (ou jeton libre) ;
- dans le choix du mana (`mana_choice`, spec 025) : les paiements par Invocation (cartes défaussables, couleurs) et par
  Polarisation (mana d'origine) ;
- sur chaque carte de la main, ses jeux de côté avec compétence et leur valeur ; sur chaque capacité d'unité, sa valeur avec
  Charisme.

## Libellés (i18n)

Noms et textes des 17 compétences : déjà là (`skill.<id>.name`, `skill.<id>.text`, spec 051), relus dans `assets/i18n/fr.json`.
Clés nouvelles, en fr et en :

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `skill.in_center` | — | Au centre : avantage au prochain tour | In the center: advantage on your next turn |
| `skill.advantage` | `skill` | Avantage de {skill} | {skill} advantage |
| `skill.choose_card` | `skill` | {skill} : choisir la carte à jouer de côté | {skill}: choose the card to play sideways |
| `skill.choose_enemy` | `skill` | {skill} : choisir l'ennemi | {skill}: choose the enemy |
| `skill.choose_space` | `skill` | {skill} : choisir la case d'arrivée | {skill}: choose the destination space |
| `skill.choose_color` | `skill` | {skill} : choisir la couleur | {skill}: choose the color |
| `skill.throw_wounds` | `count` | Jeter {count} Blessure(s) | Throw away {count} Wound(s) |
| `skill.reroll_die` | — | Relancer ce dé | Reroll this die |
| `skill.no_reroll` | — | Ne rien relancer | Reroll nothing |
| `skill.keep_die` | — | Garder ce dé tel quel | Keep this die as it is |
| `skill.no_dice` | — | Magie inutile : plus de dé de la Source ce tour | Who Needs Magic: no Source die this turn |
| `skill.interaction_influence` | `amount` | Influence d'interaction : {amount} | Interaction Influence: {amount} |
| `skill.bonds_token` | — | Jeton de Loyauté : recrutement −5 | Loyalty token: recruit for 5 less |
| `skill.bonds_token_used` | `name` | Jeton de Loyauté : {name} | Loyalty token: {name} |
| `skill.bonds_recruit` | `name`, `cost` | Recruter sous Loyauté : {name} ({cost}) | Recruit under Loyalty: {name} ({cost}) |
| `skill.choose_die` | `skill` | {skill} : choisir le dé | {skill}: choose the die |
| `skill.sideways` | `value` | De côté : {value} | Sideways: {value} |
| `skill.with_leadership` | `effect` | Avec Charisme : {effect} | With Leadership: {effect} |
| `effect.move_per_unit` | `max` | Mouvement 1 par unité prête et non blessée ({max} au plus) | Move 1 per ready, unwounded Unit (max {max}) |
| `effect.crystal_color_influence` | — | Influence d'interaction 1 par couleur de cristal | Interaction Influence 1 per crystal color |
| `effect.discard_wound_to_draw` | — | Défausser une Blessure, piocher une carte | Discard a Wound, draw a card |
| `effect.flight` | `spaces` | Vol : {spaces} case(s) | Flight: {spaces} space(s) |
| `effect.flight_cost` | `spaces`, `cost` | Vol : {spaces} case(s) pour {cost} Mouvement | Flight: {spaces} space(s) for {cost} Move |
| `effect.resistance_break` | — | Un ennemi : Armure −1 par résistance | One enemy: Armor −1 per resistance |
| `effect.source_opening` | — | Relancer un dé de la Source | Reroll a Source die |
| `effect.calm_weather` | — | Terrains −2 ce tour (au moins 1) | Terrain costs −2 this turn (min 1) |
| `effect.mana_overload` | — | Un jeton de la couleur choisie | A mana token of the chosen color |
| `mana_choice.invocation` | `card`, `color` | Invocation : défausser {card} ({color}) | Invocation: discard {card} ({color}) |
| `mana_choice.polarization` | `from`, `color` | Polarisation : {from} comme {color} | Polarization: {from} as {color} |

Ajoutées au fil écran (2026-11-14) : `skill.choose_die` (bandeau d'Embrasure de la Source), `skill.sideways` (bouton des
compétences de côté, valeurs des jeux légaux, ou à défaut celles que peut donner l'option, issue privée 302), `skill.with_leadership` (infobulle du second bouton de Charisme), les textes
`effect.*` des options (boutons de la section Compétences) ; `skill.bonds_recruit` nomme l'unité (le bouton est seul quand le
recrutement ordinaire n'est pas légal).

Exemples avec paramètres : `mana_choice.invocation` avec `{card}` = « Blessure », `{color}` = « rouge » (`mana.red`) donne
« Invocation : défausser Blessure (rouge) » ; `mana_choice.polarization` avec `{from}` = « Cristal bleu », `{color}` = « rouge »
donne « Polarisation : Cristal bleu comme rouge ». `skill.throw_wounds` avec `{count}` = 2 : « Jeter 2 Blessure(s) » (accord
par la parenthèse, comme pour 0 et 1).

Clés réutilisées, valeurs relues dans `assets/i18n/fr.json` de `main` : `skill.choose_unit` = « {skill} : choisir l'unité »
(Charisme ; avec `{skill}` = « Charisme » : « Charisme : choisir l'unité ») ; `skill.pick_unit` = « Choisir » ;
`play.sideways_effect` = « Jouer de côté : {effect} » (avec `{effect}` = « Mouvement 4 » : « Jouer de côté : Mouvement 4 ») ;
`mana_choice.cancel` = « Annuler » ; `mana.red` … `mana.black` (couleurs). La clé `effect.unsupported` n'est plus utilisée par
les compétences (elle reste pour les cartes). Aucun symbole hors des polices du jeu.

## Interface

- **Compétences du groupe A et avantage du Rituel** : le bouton de la compétence (section Compétences, spec 051) ouvre un choix
  de la carte dans la main, comme le choix d'unité d'Inspiration : bandeau `skill.choose_card` avec « Annuler », et sur chaque
  carte qui peut la recevoir, ses jeux de côté avec la valeur remplacée (`play.sideways_effect`) ; pour Pouvoir universel, le
  choix du mana suit (fenêtre du mana, spec 025) (C2 à C9, `PlayerView::skills`).
- **Section Compétences** : quand les compétences ne tiennent pas dans la hauteur d'une carte, la liste défile, barre visible ;
  le nom d'une compétence se tronque avant sa fréquence (issue privée 302).
- **Invocation et Polarisation** : de nouvelles lignes dans la fenêtre du choix du mana, `mana_choice.invocation` (une par carte
  défaussable et couleur) et `mana_choice.polarization` (une par mana d'origine) ; pas de bouton dans la section Compétences,
  qui les montre comme les autres avec leur état (C10 à C17, `PlayerView::mana_choice`).
- **Charisme** : sur chaque capacité d'unité qui peut le recevoir, un second bouton avec la valeur augmentée (même icône,
  valeur + bonus) (C26 à C28, `PlayerView::skills`).
- **Vol** : le bouton de l'option met le plateau en choix de case (cases possibles en surbrillance, comme l'exploration),
  bandeau `skill.choose_space` avec « Annuler » (C22 à C25, `PlayerView::skills`).
- **Botte secrète** : choix de l'ennemi dans l'écran de combat, bandeau `skill.choose_enemy`, comme la cible d'un sort (C33,
  `PlayerView::skills`).
- **Surcharge de mana** : choix de la couleur, quatre boutons de couleur (bandeau `skill.choose_color`) (C41).
- **Rituel de guérison** : trois boutons d'option `skill.throw_wounds` (0, 1, 2 selon les Blessures en main) (C36).
- **Embrasure de la Source** : choix d'un dé de la Source (`skill.reroll_die`, `skill.no_reroll`) ; au tour suivant, le dé en
  plus apparaît dans la Source comme prenable ; à la fin du tour, `skill.keep_die` avec le choix de fin de tour (C39,
  `PlayerView::center_advantages`).
- **État au centre** : une compétence au centre montre `skill.in_center` ; au tour suivant, l'avantage est rappelé dans les
  effets en cours (`skill.advantage`) (C37, C38, `PlayerView::center_advantages`).
- **Fortune clinquante** : l'Influence d'interaction s'affiche à côté de l'Influence (`skill.interaction_influence`) (C18, C19,
  `PlayerView::interaction_influence`).
- **Magie inutile à +3** : rappel `skill.no_dice` dans les effets en cours, dés de la Source grisés (C6,
  `PlayerView::source_dice_forbidden`).
- **Loyauté sans faille** : dans la section Unités, un emplacement de plus marqué `skill.bonds_token`
  quand le jeton est libre, `skill.bonds_token_used` quand il porte son unité, qui ne compte pas dans l'en-tête `units.header`
  (issue #26) ; dans l'offre d'unités,
  un bouton `skill.bonds_recruit` à côté de « Recruter » quand le jeton est libre (C29 à C32, `PlayerView::bonds_unit`).
- Placement des bandeaux et des boutons : vérification à l'œil.

Parties préparées pour la vérification à l'œil (`docs/verification-manuelle.md`) :

- `competences-cote` : Tovak, Flexibilité, Magie inutile, Botte secrète, Surcharge de mana ; une Blessure et Coup de sang en
  main ; un ennemi errant voisin (Cogneurs, résistants au physique).
- `competences-goldyx` : Goldyx, Pouvoir universel, Vol, Embrasure de la Source, Fortune clinquante, près d'un village et d'un
  lac du bac à sable, cristaux de trois couleurs.
- `competences-norowas` : Norowas, Charisme, Marche en avant, Loyauté sans faille, Temps calme, trois unités ; et
  `competences-arythea` : Arythea (Souffrance sublimée, Invocation, Polarisation, Rituel de guérison), seconde partie choisie
  au fil écran.

### Raccourcis clavier

Aucun : comme au premier lot (spec 051), les compétences s'utilisent par leurs boutons ; « Annuler » des bandeaux répond à Échap
comme les autres choix annulables.

## Effets sur les specs déjà implémentées

- **051** : C1 (`first_batch_skills_are_supported`) et C24 (`unsupported_skill_cannot_be_used`, qui prend Magie inutile) changent :
  plus aucune compétence `Unsupported` ; C24 est retiré ou repris sur une compétence sans cible maintenant. La partie préparée
  `competences` (Magie inutile « pas encore disponible ») est mise à jour. Liste relevée au fil moteur par
  `grep -rn "Unsupported\|unsupported" crates/*/tests crates/*/src | grep -i skill`.
- **025** : le choix du mana accepte deux paiements nouveaux ; `ManaPayment` gagne deux formes (lecteurs relevés par
  `grep -rn "ManaPayment::" crates`).
- **042** : `RecruitUnit` et `ActivateUnit` gagnent un champ (lecteurs : `grep -rn "RecruitUnit {\|ActivateUnit {" crates`) ;
  le renvoi exclut l'unité sous Loyauté.
- **013** : le jeu de côté porte une compétence (`CardPlay::Sideways`, lecteurs : `grep -rn "CardPlay::Sideways" crates`).
- **044** : `units_sit_out` épargne l'unité sous Loyauté.

## Hors périmètre

- Les compétences compétitives (spec 051, Q2) et celles du joueur fantôme (spec 061).
- Les interactions avec les tactiques (Vol de mana, *Mana Steal* : Compendium, *Who Needs Magic* S3) : spec 060.
- Les héros, compétences et cartes des extensions (immunité arcanique, Labyrinthe, Rituel des anciens…).

## Plan de développement

Grosse spec (moteur et écran, dix-sept règles nouvelles) : trois fils successifs sur la même branche (Q7), plan détaillé
(`docs/specs/plans/058-competences-second-lot.md`) au début du premier.

1. **Fil moteur 1** : groupes A, B et C (cartes de côté, paiements, points) et l'avantage du Rituel de guérison : C1 à C21, C36,
   C37.
2. **Fil moteur 2** : groupes D, E, F et G (Vol, unités, Botte secrète, centre) : C22 à C46, persistance, tests existants
   adaptés.
3. **Fil écran** : libellés, bandeaux et choix, fenêtre du mana, parties préparées, vérification à l'œil, docs
   (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`).

## Décisions (2026-11-13)

Mathieu a accepté les réponses proposées (« ok ») :

- **Q1 — Avantage différé** : seulement pendant le tour suivant du joueur, jamais le tour même où la compétence est posée ;
  perdu si la manche finit avant ; la compétence revient retournée.
- **Q2 — Déclenchement** : d'office pour Temps calme (début du tour suivant) et Surcharge de mana (première carte qui remplit
  la condition) ; au choix pour le Rituel de guérison et Embrasure de la Source.
- **Q3 — Fortune clinquante** : Influence d'interaction dans une réserve à part, dépensée en premier pour un achat
  d'interaction ; jamais pour Apprentissage ni une conversion.
- **Q4 — Magie inutile** : « utiliser un dé » = payer avec un dé (dé du tour, dé en plus, dé épuisé polarisé) ; mettre des dés
  sur une couleur reste permis.
- **Q5 — Surcharge de mana** : « alimenter » = mana de l'effet renforcé d'une action ou mana de la couleur d'un sort ; pas
  Pouvoir universel, ni une capacité d'unité, ni un mana payé dans un effet ; sur une option à deux sortes de points, les +4 vont
  aux premiers.
- **Q6 — Compétence portée par l'action** : la carte de côté, le paiement et l'activation d'unité portent la compétence, en une
  seule action ; à l'écran, choix de la carte depuis le bouton de la compétence, lignes de la fenêtre du mana, second bouton sur
  la capacité d'unité.
- **Q7 — Trois fils** sur la même branche : moteur 1 (A, B, C et avantage du Rituel), moteur 2 (D, E, F, G), écran.
- **Q8 — Trois parties préparées**, sans maquette préalable des bandeaux.

## Questions ouvertes

Aucune.

<details><summary>Questions posées à la relecture</summary>

- **Q1 — Durée de l'avantage différé en solo.** Le compendium dit que l'avantage « revient au joueur pendant son tour suivant »
  (*Skill Tokens* S5) et que les compétences au centre expirent au tour du joueur (S4). Proposition : l'avantage ne vaut que
  **pendant le tour suivant**, puis la compétence revient retournée ; jamais le tour même où elle est posée (donc pas de +4 de
  Surcharge de mana sur le jeton qu'elle vient de donner) ; perdu si la manche finit avant. Autre lecture : l'avantage dès le tour
  même. Elle rendrait Surcharge de mana bien plus forte (jeton et +4 tout de suite).
- **Q2 — Avantage d'office ou au choix.** Le texte dit « tout joueur **peut** la rendre ». Proposition : d'office pour Temps calme
  (au début du tour suivant, sans inconvénient) et pour Surcharge de mana (à la première carte qui la remplit, comme dit le
  texte « le premier joueur qui… ») ; au choix pour le Rituel de guérison (quelle Blessure jouer) et Embrasure de la Source
  (prendre ou non un dé en plus, qui interdit Magie inutile à +3). Autre choix : tout au choix, avec un bouton.
- **Q3 — Influence de Fortune clinquante.** « Cette Influence ne sert qu'en interaction. » Proposition : une réserve à part,
  dépensée en premier pour un achat d'interaction (recruter, soin, sort, action avancée, autel), jamais pour Apprentissage (effet
  de carte) ni une conversion. Autre choix plus simple : l'ajouter à l'Influence ordinaire, puisque la compétence ne se joue
  qu'en interaction ; le seul écart serait Apprentissage pendant une interaction.
- **Q4 — « Utiliser un dé » pour Magie inutile.** Le compendium hésite (*Who Needs Magic* Q1, note) et conseille la lecture
  étroite : payer avec un dé. Proposition : la suivre (dé du tour, dé en plus de Tirage de mana de base ou d'Embrasure de la
  Source, dé épuisé de Polarisation) ; mettre des dés sur une couleur (Tirage de mana renforcé, Tempête de mana) reste permis.
- **Q5 — Ce qui « alimente » une carte pour Surcharge de mana.** Proposition : le mana de l'effet renforcé d'une action, ou le
  mana de la couleur d'un sort ; pas le mana de Pouvoir universel sur une carte de côté (elle n'est pas « alimentée », elle
  est jouée de côté), ni une capacité d'unité (S1), ni un mana payé dans un effet (S2). Sur une option à deux sortes de points
  (O4, le compendium laisse le choix), les +4 vont aux premiers points de l'option, pour ne pas ajouter un choix.
- **Q6 — Compétence qui accompagne une action.** Proposition : la carte de côté, le paiement et l'activation d'unité **portent**
  la compétence, en une seule action (rien d'« armé » à l'avance qui se perdrait) ; à l'écran, le bouton de la compétence ouvre
  le choix de la carte, Invocation et Polarisation sont des lignes de la fenêtre du mana, Charisme un second bouton sur la
  capacité d'unité. Autre choix : un modificateur « prochaine carte de côté » posé par la compétence (comme Embuscade), plus
  simple à coder mais qui peut se perdre.
- **Q7 — Trois fils.** La règle des grosses specs prévoit deux fils (moteur puis écran, issue privée 118). Dix-sept règles nouvelles
  ne tiennent pas dans un seul fil moteur raisonnable. Proposition : deux fils moteur (A, B, C ; puis D, E, F, G) et un fil écran,
  sur la même branche. Autre choix : couper la spec en deux (058 groupes A à C, une 089 pour D à G), chacune avec ses deux fils.
- **Q8 — Parties préparées.** Proposition : trois parties (schéma de la section Interface) plutôt qu'une seule, chaque héros
  n'ayant que ses compétences. Faut-il une maquette des bandeaux de choix avant le fil écran ? Proposition : non, ils copient
  ceux d'Inspiration et des sorts.

</details>
