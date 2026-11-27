# #68 — Texte des cartes et des compétences

- Statut : à valider
- Issue : #68 (Bug, milestone v0.2.0, Effort Medium : spec codée en un fil)
- Dépend de : spec #11 (bouton « i » et texte complet), spec #32 (cartes des fenêtres), spec #27 (repères `[[…]]` à
  icône dans un texte), spec 071 (icônes), spec 050 (textes écrits des cartes), spec 055 (artefacts) ; toutes implémentées
- Livret : aucune règle nouvelle. Source du sens de chaque texte : transcriptions des cartes
  (`../mage-knight/docs/regles/cartes/`, version *Lost Legion* quand il y a un errata)
- Crates touchées : `assets/i18n/` (fr.json et en.json, l'essentiel du travail), arcanum-ui (choix du texte affiché,
  repères à icône). Rien dans arcanum-core.

## Objectif

Le texte d'une carte, d'une compétence ou d'une tactique que le joueur lit pour comprendre ce qu'elle fait (bouton « i »,
fenêtres de cartes, infobulles) devient une suite de phrases complètes, dans l'ordre et avec toutes les conditions de la
carte du jeu de base, au lieu des formules compactées d'aujourd'hui. Les mots-clés de points (Mouvement, Influence,
Attaque, Parade, Armure, Gloire) y portent leur icône, pour que le texte se lise d'un coup d'œil (retour de joueur de la
v0.1.0, commentaire de l'issue).

Les boutons, eux, restent courts : ils disent ce qu'on obtient en cliquant, pas la carte entière.

## Constat (relevé à la rédaction)

Trois défauts, avec des exemples tirés de `assets/i18n/fr.json` de `main` :

| Défaut | Exemple aujourd'hui | Carte |
|---|---|---|
| Formule au lieu d'une phrase | « Mouvement 4 ; en combat, 1 mouvement = Attaque 1, 2 mouvements = Attaque à distance 1 » (`card.agility.strong_text`) | Agilité (Mouvement 2 ; pendant le combat de ce tour, les points de Mouvement restants servent d'Attaque, un pour un) |
| Texte calculé illisible (actions de base, qui n'ont pas de texte écrit : le « i » montre les effets mis bout à bout) | « Mettre 1 dé(s) sur une couleur, gagner des jetons » (`effect.set_dice`) ; « Renforcer une autre carte gratuitement, +2 » (`effect.boost`) | Source de mana renforcée (prendre un dé de la Source, le mettre sur une couleur sauf or, gagner deux jetons de cette couleur, le dé n'est pas relancé) ; Concentration renforcée (jouer une autre carte action avec elle, obtenir son effet renforcé gratuitement, +2 aux points qu'il donne) |
| Abréviations et raccourcis | « (min. 1) », « Idem », « etc. », « −2 » collé à un terrain, chaînes de « ; » | Secousse, Transe, Ailes de la nuit, Chemins de traverse |

Le même texte écrit (`basic_text`, `strong_text`) sert aujourd'hui à trois places : le « i » et les fenêtres de cartes, le
recto d'un artefact (spec #11, Q1), et le libellé du bouton d'une option sans effet propre (Au cœur du combat, Tourbillon :
`card_option_text`, spec 053). C'est pourquoi il a été compacté : il devait tenir sur un bouton.

## Règles

- **Sens d'un texte** : celui de la carte transcrite dans `../mage-knight/docs/regles/cartes/` (version *Lost Legion* si
  errata), sans rien omettre ni ajouter. Aucun renvoi au livret : la spec ne change aucune règle.
- **Écart entre la carte et le code** : si, en relisant une carte, le fil voit que le code ne fait pas ce qu'elle dit, il
  n'écrit pas un texte qui décrit le code : il écrit le texte de la carte et ouvre une issue `Bug` (renvoi dans la PR).
- **Droits** (CLAUDE.md, Assets et droits : aucun texte officiel dans le jeu) : nos mots, jamais un copier-coller ni une
  traduction mot à mot des transcriptions, en français comme en anglais. Voir Q1.
- **Termes** : ceux du glossaire (`docs/glossaire.md`) et de la FAQ française (Vif, Toxique, Paralysant…).

## Charte de rédaction

S'applique aux **textes complets** (voir Modèle) des deux langues.

1. **Des phrases complètes**, chacune terminée par un point. Pas de « ; » pour enchaîner des morceaux, pas de « = », pas
   d'abréviation (« min. », « etc. », « Idem » : écrire « au moins 1 », énumérer, redire l'effet).
2. **L'ordre de la carte** : d'abord ce qu'on gagne, puis les conditions et les limites dans l'ordre où la carte les donne.
3. **Le joueur est « vous »**, à l'indicatif ou à l'impératif (« Gagnez un jeton de mana bleu, blanc ou rouge. »). Fini
   l'infinitif des compétences (« Défausser une carte pour… ») et le mélange actuel entre familles.
4. **Les quantités en chiffres** après le mot-clé (« Mouvement 4 », « Attaque de feu 5 »), comme sur les boutons.
5. **Le moment** est dit en clair : « pendant le combat de ce tour », « à la fin de votre tour », « une fois par manche ».
6. **Repère à icône** devant chaque mot-clé de points (voir Modèle et Q2) : `[[move]]Mouvement 4`.
7. **Longueur** : celle qu'il faut pour être compris. Un texte complet n'est jamais un libellé de bouton.

Les **libellés de bouton** (voir Modèle) suivent les règles 3 et 4, sans phrase ni point final : « Mouvement 4, reste en
Attaque » plutôt que la carte entière. Pas de repère : le bouton porte déjà son icône (spec 073).

**Exemples** (fr ; le fil écrit l'anglais sur le même modèle) :

| Carte | Aujourd'hui (« i ») | Proposé (texte complet) |
|---|---|---|
| Concentration, renforcé | Renforcer une autre carte gratuitement, +2 | Jouez une autre carte action avec celle-ci : vous obtenez son effet renforcé sans payer de mana. S'il vous donne du [[move]]Mouvement, de l'[[influence]]Influence, une [[attack]]Attaque ou une [[block]]Parade, ajoutez 2. |
| Improvisation, base | Défausser une autre carte : Mouvement 3 ou Influence 3 ou Attaque 3 ou Parade 3 | Défaussez une autre carte de votre main pour obtenir [[move]]Mouvement 3, [[influence]]Influence 3, [[attack]]Attaque 3 ou [[block]]Parade 3. |
| Agilité, renforcé | Mouvement 4 ; en combat, 1 mouvement = Attaque 1, 2 mouvements = Attaque à distance 1 | [[move]]Mouvement 4. Pendant le combat de ce tour, chaque point de Mouvement qui vous reste peut servir d'[[attack]]Attaque 1, ou deux d'entre eux d'[[ranged_attack]]Attaque à distance 1. |
| Secousse, base (sort) | Un ennemi Armure -3, ou tous Armure -2 (min. 1) | Un ennemi perd 3 d'[[armor]]Armure, ou tous les ennemis perdent 2 d'[[armor]]Armure. L'Armure d'un ennemi ne descend pas sous 1. |
| Invocation (compétence d'Arythea) | Défausser une carte pour un jeton de mana à dépenser aussitôt : rouge ou noir pour une Blessure, blanc ou vert sinon. | Défaussez une carte de votre main pour gagner un jeton de mana, à dépenser aussitôt : rouge ou noir si c'est une Blessure, blanc ou vert sinon. |

## Modèle

Aucune donnée nouvelle dans l'état ni dans la vue : tout se joue dans les libellés et dans le choix du libellé affiché.

**Deux sortes de textes écrits** par carte :

- **Texte complet** (`basic_text`, `strong_text`, clés existantes dont le sens change) : lu par le « i » (spec #11), les
  fenêtres de cartes (spec #32) et le recto d'un artefact (spec 055). **Les actions de base en reçoivent un** (20 cartes,
  clés nouvelles `card.<carte>.basic_text` et `strong_text`) : leur « i » ne montre plus d'effets calculés mis bout à bout.
- **Libellé de bouton** (`basic_label`, `strong_label`, clés nouvelles) : seulement pour les cartes dont un bouton lit
  aujourd'hui le texte écrit (option sans effet propre, `card_option_text`). Le fil d'implémentation en dresse la liste en
  cherchant les options `Effect::Nothing` des contenus RON et la donne dans la PR.

**Textes à un seul usage**, réécrits sur place selon la charte : compétences (`skill.<compétence>.text`, 40), tactiques
(`tactic.<tactique>.text`, 12). Les capacités d'unité (`unit.<unité>.ability_N`, 20 unités) sont des libellés de bouton
(spec 073) : elles suivent les règles des boutons, voir Q3.

**Libellés calculés de bouton** (`effect.*`) : seuls ceux qui ne se comprennent pas sont repris, liste fermée : `set_dice`,
`boost`, `repeat`, `extra_die_black`, `does_not_attack_per_move`, `gem_pouch_dice`, `horn_of_wrath`. Les autres
(« Attaque 3 », « Jeton bleu ») sont déjà clairs.

**Repères à icône nouveaux** (`ICON_MARKERS`, spec #27), un par icône de points de la planche (spec 071) : `move`,
`influence`, `armor`, `fame`, `attack`, `ranged_attack`, `siege_attack`, `block`, et leurs variantes d'élément
(`fire_attack`, `ice_block`…), nommés comme les clés `effect.*` qui leur correspondent. Le repère se dessine comme
`[[swift]]Vif` aujourd'hui : l'icône à la hauteur du texte, collée au mot qui suit, avec son infobulle.

Termes : rien de nouveau au glossaire (« texte complet » est déjà le terme de la spec #11).

## Comportements attendus

Le contenu des textes ne se teste pas (relu par Mathieu, voir Interface) ; les tests gardent ce qui se vérifie.

- **C1** — Chaque carte qui se joue (20 actions de base, 28 actions avancées, 20 sorts, 16 artefacts) a un texte complet de
  base et un renforcé, non vides, en fr et en en (`every_deed_card_has_full_texts_in_both_languages`). Rouge attendu : les
  actions de base n'en ont pas.
- **C2** — Le « i » d'une action de base montre ses textes complets : Concentration (basic_actions.ron, `Concentration`),
  bloc de base `card.concentration.basic_text`, bloc renforcé `card.concentration.strong_text`
  (`basic_action_info_shows_its_full_text`). Rouge attendu : il montre aujourd'hui les effets calculés.
- **C3** — Une option sans effet propre a pour bouton son libellé de bouton, et son « i » le texte complet : Au cœur du
  combat (advanced_actions.ron, `IntoTheHeat`, jouable au début d'un combat), bouton `card.into_the_heat.basic_label`, « i »
  `card.into_the_heat.basic_text` (`option_button_reads_label_and_info_reads_full_text`). Rouge attendu : la clé
  `basic_label` n'existe pas.
- **C4** — Les fenêtres de cartes (Offres, spec #32) montrent les textes complets, comme le « i » : une action de base n'y
  passe pas, donc le cas prend Agilité dans l'offre d'actions avancées (`offer_window_shows_full_texts`). Comme aujourd'hui
  pour les actions avancées (spec #32, `window_card` lit `card_info`) ; le test garde le lien.
- **C5** — Chaque texte complet, chaque texte de compétence et chaque texte de tactique, en fr et en en, finit par un point
  et ne contient ni « ; », ni « = », ni « min. », ni « Idem », ni « etc. » (`full_texts_follow_the_writing_rules`). Rouge
  attendu : presque tous les textes actuels.
- **C6** — Le repère `[[move]]` se découpe en icône Mouvement suivie du mot (`move_marker_draws_the_move_icon`) ; de même,
  un test par famille : `[[fire_attack]]` donne l'icône Attaque de feu, `[[ice_block]]` la Parade de glace, `[[armor]]`,
  `[[fame]]`, `[[influence]]` (`point_markers_draw_their_icons`). Rouge attendu : repères inconnus, laissés en texte.
- **C7** — Chaque repère `[[…]]` de fr.json et en.json est connu, et une clé a les mêmes repères dans les deux langues
  (`markers_are_known_and_match_across_languages`). Vert aujourd'hui pour les 6 repères existants ; le test garde la
  parité une fois les textes réécrits.
- **C8** — Le recto d'un artefact montre son texte complet, repères dessinés : Anneau de rubis (artifacts.ron, `RubyRing`)
  (`artifact_face_shows_full_text`). Comme aujourd'hui (spec 055) sur la clé ; le test garde le lien.

**Tests existants qui lisent un texte réécrit** (cherchés dans `crates/*/tests` et `crates/*/src`), à mettre à jour sur la
nouvelle valeur ou, mieux, sur la clé du catalogue, en le signalant dans la PR :

- `crates/arcanum-ui/tests/moteur_d_effets.rs:260` (`hand_shows_written_text_of_modifier_cards`, Chemins de traverse) ;
- `crates/arcanum-ui/tests/unites.rs:208` et `:753` (Bûcherons, Gardes d'Utem) et `crates/arcanum-ui/tests/icones.rs:977`
  (Gardiens d'Altem), si Q3 retient les capacités d'unité ;
- `crates/arcanum-ui/tests/interaction.rs:178` (`effect.influence_in_interaction`, si le libellé change) ;
- `cartes_epurees.rs` (`spell_info_holds_written_text_for_other_players`, `advanced_action_info_holds_written_text`),
  `actions_avancees_mana.rs:688` (`blood_ritual_basic_effect_uses_written_text`), `competences.rs:62`
  (`every_skill_has_a_name_and_a_text`) : ils lisent les clés, inchangées ; à relancer seulement.

## Cas limites et refus

- **L1** — Un sort a deux noms (Éclair de feu / Tempête de feu) : le texte complet renforcé ne répète pas le nom, le titre
  du bloc le porte déjà (spec #11).
- **L2** — Effet maximum, Concentration : leur texte dit ce que fait la carte, pas la liste des cartes qu'elle peut
  renforcer (le choix se fait à l'écran, spec 055 et suivantes).
- **L3** — Sort à effets « pour les autres joueurs » (Fuite de mana, Lecture d'esprit) : le texte complet dit tout l'effet
  de la carte, comme aujourd'hui (spec #11, `spell_info_holds_written_text_for_other_players`), même ce qui ne sert pas en
  solo.
- **L4** — Un texte qui ne tient plus dans la largeur du « i » (`INFO_WIDTH`) passe à la ligne, sans troncature
  (vérification à l'œil).

## Information cachée

Rien de nouveau : les textes sont du contenu public, lus dans le catalogue de libellés.

## Libellés (i18n)

Clés nouvelles (fr et en) :

| Clé | Pour |
|---|---|
| `card.<action de base>.basic_text`, `.strong_text` | texte complet des 20 actions de base (`BasicAction::ALL`, dont les cartes propres aux héros) |
| `card.<carte>.basic_label`, `.strong_label` | libellé de bouton des cartes à option sans effet propre (liste dans la PR) |

Clés réécrites : `card.*.basic_text` et `strong_text` (actions avancées, sorts, artefacts), `skill.*.text`,
`tactic.*.text`, les sept `effect.*` listés au Modèle, et `unit.*.ability_N` selon Q3. Aucune clé fille sous un libellé :
`basic_label` est à côté de `basic_text` dans le groupe de la carte. Pas de caractère nouveau : les repères sont des
icônes, pas des glyphes.

## Interface

- **« i » d'une carte** (spec #11) : textes complets, repères dessinés (C2, C6). Mise en page inchangée (vérification à
  l'œil).
- **Fenêtres de cartes** (Offres, choix de l'artefact ou du sort) : textes complets (C4).
- **Recto d'un artefact** : texte complet (C8).
- **Infobulles des compétences et des tactiques** : textes réécrits, repères dessinés (C5, C6 ; vérification à l'œil).
- **Boutons** : libellés courts (C3), inchangés pour les options à effet calculé.

**Relecture par Mathieu** : la PR joint un tableau avant / après de tous les textes réécrits, en fr et en en, déposé dans
les fichiers du projet (pas dans le dépôt). Mathieu le relit avant la fusion ; le fil corrige d'après ses remarques.

**Parties préparées** : aucune nouvelle. La vérification à l'œil prend le bac à sable (« i » d'une action de base, d'une
action avancée de l'offre ; une compétence, une tactique) et `niveau-pair-choix` (fenêtre du niveau 2 : textes des
compétences et des actions avancées proposées).

### Raccourcis clavier

Aucun.

## Hors périmètre

- Textes des ennemis, des sites et des cités (infobulles du plateau), lignes des effets en cours (`modifier.*`), et le
  pluriel des libellés à « (s) » (« carte(s) »), qui demande une règle de pluriel dans arcanum-i18n.
- Remplacer davantage de texte par des icônes dans la zone de jeu, ou agrandir les icônes : Epic #96 (ergonomie).

## Questions ouvertes

- **Q1 — Proximité avec le texte du jeu.** L'issue demande des textes « plus proches de ceux du jeu de base », et le
  CLAUDE.md interdit tout texte officiel dans le jeu public. Proposé : même sens, même ordre, même découpage en phrases que
  la carte, mais nos mots, sans copier ni traduire mot à mot la transcription ; l'anglais aussi (c'est là que le risque de
  recopier est le plus grand). Convient-il ?
- **Q2 — Icônes dans les textes.** Proposé : le repère à icône devant chaque mot-clé de points des textes complets, des
  compétences et des tactiques (`[[move]]Mouvement 4` : icône puis mot), comme `[[swift]]Vif` aujourd'hui. Autres choix :
  l'icône à la place du mot (« [icône] 4 », plus court, mais le joueur doit connaître les icônes) ; ou pas d'icône, laissée à
  l'Epic #96.
- **Q3 — Capacités d'unité.** Ce sont des boutons. Proposé : les reprendre selon les règles des boutons (plus de « ; » ni
  de « (min. 0) »), sans texte complet ni « i » pour une unité. Ou les laisser telles quelles.
- **Q4 — Un fil ou deux.** Environ 150 textes à écrire dans chaque langue. Proposé : un seul fil (Effort Medium), qui écrit
  le français, le fait relire par Mathieu sur le tableau avant / après, puis écrit l'anglais. Ou deux fils : français, puis
  anglais.
