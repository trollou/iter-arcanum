# NNN — Titre de la spec

- Statut : brouillon | à valider | validée | implémentée (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J?
- Dépend de : NNN, NNN
- Livret : MKUE p. ?
- Crates touchées : arcanum-core, …

## Objectif

Deux ou trois phrases : ce que le joueur peut faire une fois cette spec implémentée.

## Règles

**Renvoyer au livret, ne pas le recopier.** Une puce par règle : la notion, le renvoi précis
(`MKUE p. 7, 5.b`, soit `../mage-knight/docs/regles/livret/p07.md` §5.b, dans le dépôt privé voisin ; voir `docs/reference/index-par-notion.md`), puis seulement
ce que la spec en fait : interprétation, simplification, report à une autre spec, question ouverte.
Un chiffre court peut rester s'il évite d'ouvrir la page (« 2 points de mouvement »). Pas de traduction ni de paraphrase du livret ;
citer le texte anglais seulement quand l'interprétation est délicate. Ce qui n'est pas dans le livret (plateau Jour/Nuit, texte des
cartes lu sur les scans) peut être recopié, avec sa source.

Exemple : `- **Coût** (MKUE p. 7, 5.b) : 2 points de mouvement, de jour comme de nuit. La figurine ne bouge pas.`

Quand le livret ne tranche pas : renvoyer à la FAQ (`FAQ p2, Cartes`) ou au compendium (`Compendium, Combat S34`), trouvés par
`docs/reference/index-par-notion.md`. Une contradiction avec le livret devient une question ouverte pour Mathieu.

## Modèle

Termes du glossaire utilisés. Nouveaux termes à ajouter au glossaire **avant** validation.
Données nouvelles dans l'état (en mots, pas en code).

Spec de parcours (ADR 0006) : un schéma du parcours fixe les **points d'étape**, leurs **préconditions** et l'**ordre** qui compte pour
les règles. Son découpage en tours est **indicatif** : l'implémentation peut l'ajuster sans nouvelle validation, en le signalant dans
la PR. Les cas évitent donc « au tour 2 » et disent plutôt « après C3 » ou « depuis le village ».
Elle dit aussi d'où part chaque parcours ou épisode (ADR 0006, issue privée 318) : **mise en place arrangée** (ce qui est choisi dans les
piles, la Source, la case de départ) ou **état posé** (les événements écrits par le test, et pourquoi le hasard ne suffit pas).
Rien de ce qu'un point d'étape vérifie n'est posé.
**Essai de faisabilité avant la validation** (issue privée 352, leçon de la spec 069) : le fil de rédaction fait jouer, dans un test
jetable et non commité (`#[ignore]`, comme `balayer_les_graines`), le script prévu, ou à défaut l'explorateur, sur une vingtaine
de graines depuis le départ choisi. Il note dans la spec, sous « Départ et préconditions », combien de graines remplissent
chaque précondition et l'ordre réel des événements que citent les points d'étape. Mathieu valide le départ et le seuil de
robustesse sur ces mesures ; une précondition trop rare devient un état posé ou change de départ avant la validation, pas au codage.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|

« Révèle une information ? » : ce qu'un autre joueur en voit (filtrage). « Point de non-retour ? » : l'événement révèle une
information, lance un dé ou ferme le tour, et l'on ne peut plus revenir en arrière avant lui (spec 024, `Event::blocks_undo`).

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses).

**Un cas qui dit « comme aujourd'hui », « inchangé » ou « déjà le cas » se vérifie contre le code ou les tests existants
pendant la rédaction**, et cite le test ou la spec qui le prouve (issue privée 154, bilan de la spec 070 : un ennemi seul et fortifié,
dit « comme aujourd'hui » sans cible à distance, en restait une ; seul un ennemi doublement fortifié n'en est pas une).

Exemple : `- **C4** — Comme aujourd'hui (spec 043 C39, test nom_du_test_existant), un ennemi doublement fortifié …`

**Un exemple chiffré** (ennemi, carte, unité, site, case) **est repris du contenu du jeu pendant la rédaction** : de
`crates/arcanum-core/content/*.ron` avec son identifiant, sinon de `../mage-knight/docs/regles/cartes/` (case : `docs/reference/bac-a-sable.md`).
Un exemple inventé oblige le fil d'implémentation à en choisir un autre (issue privée 158, bilan de la spec 071 : « Gardes du feu »
n'existe pas, remplacé par les Mages de feu).

Exemple : `- **C8** — L'infobulle des Mages de feu (enemies.ron, FireMages : armure 5, attaque de feu 6, résistance au feu, gloire 5) …`

**Chaque carte, unité ou effet cité dans un cas est jouable dans la situation décrite** (phase, combat ou non, Jour ou
Nuit, mana disponible) : le fil de rédaction le vérifie avant de demander la validation (issue privée 175, bilan de la spec 073 :
C6 payait hors combat avec Détermination renforcée, une Parade, injouable hors combat ; le test a pris Endurance).

**Avant d'écrire la liste des tests existants modifiés**, la rédaction cherche ce que la spec touche **là où c'est utilisé**,
pas là où c'est défini (`grep -rn 'fn nom'` ne trouve que la définition), dans `crates/*/tests` et `crates/*/src`, et signale
tous les tests trouvés :
- **champ retiré ou renommé** (de l'écran, de la vue ou de l'état) : ses lecteurs, `grep -rn '\.<champ>\b'` (issue privée 190, bilan
  de la spec 074 : quatre tests qui lisaient `time_of_day` et `scenario`, retirés de `GameScreen`, ne sont apparus qu'à la
  compilation) ;
- **fonction dont la signature change** : ses appels, `grep -rn '\b<nom>('` (issue privée 357, bilan de la spec 080 : quatre appels
  de `viewport(title, icon)` dans `icone.rs` et `icone_logs.rs`, trouvés à l'implémentation) ;
- **liste ou compte qui change** (éléments du Menu, `KEY_BINDINGS`, actions légales…) : les tests qui les comptent ou les
  énumèrent (issue privée 357 : `bar_stays_during_combat` comptait les lignes du Menu, 10 devenues 12).

- **C1** — Étant donné …, quand …, alors … (`nom_du_test`) — MKUE p. ?
- **C2** — …

## Cas limites et refus

- **L1** — …

## Information cachée

Ce que la vue du joueur contient ou non.

**Tout ce que l'écran affiche se lit dans la vue** (`PlayerView`) : l'écran ne lit ni l'état ni les événements (issue privée 130, bilan
de la spec 044 : le dé du donjon, dit « dans les événements », a dû être ajouté à la vue pendant le fil écran). Lister ici les champs
de la vue ajoutés par la spec ; la section Interface renvoie à eux.

## Libellés (i18n)

Nouvelles clés et leurs paramètres.

Avant de passer la spec « à valider », vérifier que chaque libellé proposé est possible (issue privée 129) :

- **Pas de clé fille sous un libellé.** Dans `assets/i18n/*.json`, une clé est soit un libellé (texte), soit un groupe (objet) :
  si `reward.die` est un libellé, `reward.die.spell` est impossible. Écrire la variante à côté, `reward.die_spell`.
- **Pas de caractère hors des polices du jeu** : celles d'egui par défaut, emojis monochromes compris (spec 043). Un symbole ou
  un emoji n'est proposé qu'après l'avoir vu s'afficher, et un test garde la police qui le dessine
  (`default_fonts_draw_crossed_swords`) ; sinon des mots ou la ponctuation courante (« → » s'affiche comme un carré, spec 044).
- **Un libellé réutilisé est cité par sa clé exacte et sa valeur dans `assets/i18n/fr.json`**, relues à la rédaction (issue privée 127),
  pour vérifier que la valeur convient là où on la place. **Si elle a un paramètre**, écrire aussi le texte obtenu en fr et en en
  avec la valeur prévue du paramètre (`site.conquered.adventure` avec `{site}` = « Ruines » donne « Ruines conquis ») : l'accord en
  genre et en nombre ne se voit pas dans la seule clé (issue privée 142). **Son existence se vérifie dans le fichier de `main`**, pas
  de mémoire ni d'après une spec plus ancienne (une spec suivante a pu la retirer, issue privée 284) : pour une clé `a.b.c`, chercher
  `"c":` dans le groupe `a.b` de `assets/i18n/fr.json` (ex. `grep -n '"scenario_and_hero"' assets/i18n/fr.json`) ; aucune ligne
  = clé disparue, à ajouter dans le tableau des clés nouvelles.
- **Chaque texte de la section Interface a sa clé** dans le tableau des nouvelles clés ou parmi les clés réutilisées citées
  (issue privée 332, bilan de la spec 060 : le nombre de cartes sous Provisions n'avait pas de clé, `tactic.stored` est née au fil
  écran). À la rédaction, relire la section Interface point par point : un bouton, un titre, un compteur, une infobulle ou une
  ligne de bandeau sans clé est à ajouter ici.

## Interface

Ce que l'UI affiche et permet (minimal, visuels provisoires). « Aucune » si la spec est purement moteur.

**Chaque point de cette section renvoie à un cas de test** (`C12`) qui le vérifie, écrit rouge avant le code comme les autres
(issue privée 119). À la rédaction, vérifier la correspondance point par point. Ce qui est purement visuel (couleur, mise en page) n'a pas
de test (ADR 0006) : le point le dit explicitement, « vérification à l'œil ».

Chaque point qui montre une donnée du jeu **nomme aussi le champ de la vue qui le nourrit** (`PlayerView::reward_die`), existant ou
ajouté par la spec (section Information cachée). À la rédaction, vérifier qu'aucun point n'en manque : un champ absent de la vue
devient un cas moteur (« la vue montre … »), implémenté avec le moteur et non découvert par le fil écran.

Exemple : `- **Repères d'efficacité** sur les capacités d'unités, comme sur les cartes (C41).` ;
`- **Dé de récompense** du donjon jusqu'à la fin du tour (C45, PlayerView::reward_die).` ;
`- **Placement** : sous la main de l'écran de combat (vérification à l'œil).`

**Parties préparées** pour la vérification à l'œil (`docs/verification-manuelle.md`) : en lister les noms et, pour chacune, la
situation de départ et ce qu'on y vérifie. **Chaque situation à montrer s'atteint depuis le départ de sa partie par des actions
légales, en avançant** : un retour en arrière ne remonte pas avant le début du tour, donc une partie ne sert pas deux moments
séparés par une fin de tour ou par une étape irréversible (révélation, dé lancé, tactique prise). Sinon, prévoir une partie par
moment (issue privée 332, bilan de la spec 060 : « manche 3 avec Vol de mana pris » supposait de revenir sur la prise de la tactique,
coupée en `tactiques-manche-3` et `tactiques-de-vole`).

### Raccourcis clavier

Touches ajoutées ou modifiées par la spec (touche, effet, quand elle agit), reportées dans `docs/raccourcis.md`.
« Aucun » si la spec n'en ajoute pas : chaque action proposée à la souris doit se demander si elle mérite une touche.

## Hors périmètre

Ce qui est volontairement laissé pour plus tard, et dans quelle spec.

## Questions ouvertes

À trancher par Mathieu pendant la relecture. Doit être vide pour passer en « validée » : une question qu'on choisit de reporter
devient une issue `question ouverte` (`docs/issues.md`), et la spec n'en garde que le renvoi (« reporté, issue #N »).
