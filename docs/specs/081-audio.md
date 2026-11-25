# 081 — Audio

- Statut : implémentée le 2026-11-20 ; validée le 2026-11-20 (Q1 à Q12 tranchées par Mathieu : réponses proposées retenues ; Q4, fin de partie,
  NC et échantillons des curseurs précisés à sa demande pendant la relecture)
- Jalon : Finitions
- Dépend de : 080 (préférences, fenêtre et fichier), 075 (animation du trajet, `WALK_STEP_SECONDS`), 024 (Revenir en
  arrière, chargement), 065 (nouvelle partie) ; toutes implémentées
- Livret : aucune règle de jeu
- Crates touchées : **arcanum-audio (nouvelle)**, arcanum-persistance (volumes dans les préférences), arcanum-ui (fenêtre des
  préférences, branchement), `assets/sounds/` (nouveau), `assets/i18n/`. Rien dans arcanum-core. Aussi : ADR 0010 (rédigé
  avec la spec), CLAUDE.md, `.claude/rules/arcanum-audio.md`, `scripts/verifier.sh`, `.github/workflows/ci.yml` et
  `docs/ci.md`, `docs/architecture.md`, `NOTICE`, README, glossaire
- Base : issue privée 187 et commentaire de Mathieu du 2026-11-12 (Kira, nouvelle crate pilotée par les événements, sons
  superposés, fondus, volumes par type, spatialisation éventuelle, sources de sons et licences)

## Objectif

Le jeu a du son. Une **musique** de fond en boucle qui change avec la situation (écran de lancement, exploration de
jour, de nuit, combat) par un **fondu enchaîné** ; une **ambiance** de jour ou de nuit sous la musique ; des **effets
sonores** joués par-dessus, sans couper la musique, quand le jeu produit certains événements (coq au lever du jour,
hibou à la tombée de la nuit, parade réussie, blessure, ennemi vaincu, fin de combat, sort lancé…). Le joueur règle
quatre **volumes** dans la fenêtre des préférences (spec 080) : général, musique, ambiances, effets. Sans carte son,
ou sans fichiers de sons, le jeu tourne muet.

## Règles

Aucune règle du livret. Décisions que la spec applique :

- **Le son ne décide rien** : il écoute la partie, il ne la change jamais. Rien dans l'état, le journal ni les
  sauvegardes ; Revenir en arrière ne joue aucun son et ne rejoue pas ceux des événements annulés.
- **Le son n'entend que ce que voit le joueur** : les événements lui arrivent filtrés pour le joueur humain
  (`EventLog::filtered_for(PlayerId::SOLO)`, ADR 0002), comme à tout joueur. Un son ne trahit donc aucune information
  cachée.
- **Seuls les événements nouveaux sonnent** : une partie chargée, ou reprise par « Continuer », ne rejoue pas son
  journal ; une partie neuve est entendue depuis son premier événement (le coq du premier jour, C9).
- **La musique suit la situation**, lue dans la vue (`PlayerView`) et non dans les événements (C3) : elle se rattrape
  donc d'elle-même après un retour en arrière, un chargement ou une nouvelle partie.
- **Sons superposés** : musique, ambiance et effets jouent en même temps, chacun sur sa piste et son volume.
- **Logs en français**, comme toujours (spec 003).

## Modèle

### Termes

Ajoutés au glossaire (nouvelle section « Audio (spec 081) », dans cette PR) :

| Terme | Code | Sens |
|---|---|---|
| Musique | `Music` | Morceau en boucle choisi selon la situation : `Title`, `Day`, `Night`, `Combat`. |
| Ambiance | `Ambience` | Fond sonore en boucle sous la musique, de jour ou de nuit (`Day`, `Night`). |
| Effet sonore | `Cue` | Son court joué une fois, déclenché par un événement de la partie ou par la fenêtre des préférences. |
| Type de son | `SoundCategory` | Musique, ambiances ou effets : chacun sa piste et son volume, sous le volume général. |
| Volumes | `Volumes` | Les quatre réglages de 0 à 100 : général, musique, ambiances, effets ; dans les préférences. |
| Situation sonore | `SoundSituation` | Ce que l'écran montre, pour choisir musique et ambiance : lancement, exploration (jour ou nuit), combat, fin de partie. |
| Directeur audio | `AudioDirector` | Partie pure d'arcanum-audio : reçoit la situation et les événements nouveaux, rend les ordres de son. |
| Ordre de son | `AudioCommand` | Ce que le directeur demande au lecteur : changer de musique ou d'ambiance (avec fondu), jouer un effet (avec délai), régler un volume. |
| Lecteur audio | `AudioPlayer` | Partie d'arcanum-audio qui exécute les ordres sur la carte son (Kira), ou ne fait rien (`SilentPlayer`). |
| Banque de sons | `SoundBank` | Les fichiers trouvés dans `assets/sounds/` au lancement, rangés par musique, ambiance et effet (avec leurs variantes). |

### La crate arcanum-audio

Nouvelle crate (ADR 0010, à valider avec la spec), entre arcanum-core et arcanum-ui :

```
arcanum-ui ──► arcanum-audio ──► arcanum-core
   │          └──► kira (lecteur seulement)
   └──► arcanum-persistance (Volumes) ──► arcanum-core
```

- **`director`** (pur, sans I/O ni Kira, testé) : `AudioDirector::new(step_interval)` ; à chaque image,
  `update(situation, &[PlayerEvent]) -> Vec<AudioCommand>`. Il garde la musique et l'ambiance en cours, ne rend un
  changement que si la situation change, et traduit les événements en effets (table ci-dessous). `step_interval` est
  la durée d'un pas de l'animation du trajet, donnée par arcanum-ui (`WALK_STEP_SECONDS`, spec 075), pour espacer les pas.
- **`bank`** (I/O de fichiers, testé avec un dossier temporaire) : `SoundBank::scan(dir)` liste
  `music/`, `ambience/` et `effects/` ; un fichier manquant n'est pas une erreur (C24).
- **`player`** : le trait `AudioPlayer` (`execute(&AudioCommand)`), `SilentPlayer` (ne fait rien) et `KiraPlayer`
  (Kira : une piste par type de son sous la piste principale, musique et ambiance lues en flux, effets chargés en
  mémoire). `KiraPlayer` n'a pas de test (comme `app/`, ADR 0006) : vérification à l'oreille.
- **Dépendance** : `kira = { version = "0.12", default-features = false, features = ["cpal", "ogg", "vorbis"] }`
  (Q1). Sans les options par défaut : ni mp3, ni flac, ni wav, ni `cpal-realtime-dbus` (qui demanderait libdbus sous
  Linux). Licences relevées par `cargo metadata` sur un projet d'essai : MIT, Apache-2.0, MPL-2.0 (symphonia,
  triple_buffer), Zlib, BSD, et « Unlicense OR MIT » (passe par MIT) : toutes déjà permises par `deny.toml`, que la PR
  du code vérifie par `cargo deny check licenses`.
- **Linux** : cpal passe par ALSA, et `alsa-sys` demande à la compilation le paquet `libasound2-dev` (et
  `pkg-config`). Vérifié sur l'essai : sans lui la compilation échoue, avec lui elle passe, et le lancement sans carte
  son (conteneur cloud) écrit quelques lignes `ALSA lib …` sur la sortie d'erreur puis continue. Windows (WASAPI) ne
  demande rien. Conséquences (Q9) : `scripts/verifier.sh` installe le paquet s'il manque (comme `verif-manuelle.sh`
  pour Xvfb), le job Linux de la CI de nuit l'installe, le README le dit pour Linux Mint (`sudo apt install
  libasound2-dev pkg-config`).

### Situation sonore, musique et ambiance

`sound_situation` (arcanum-ui, pure, testée) lit l'écran et la vue :

| Ce que montre l'écran | Situation | Musique | Ambiance |
|---|---|---|---|
| Écran de lancement, ou écran de nouvelle partie en cours de partie | `Title` | `Title` | aucune |
| Partie, `PlayerView::combat` vide, `time_of_day` Jour | `Exploration(Day)` | `Day` | `Day` |
| Partie, `PlayerView::combat` vide, `time_of_day` Nuit | `Exploration(Night)` | `Night` | `Night` |
| Partie, `PlayerView::combat` présent (« Voir le plateau » compris) | `Combat(jour ou nuit)` | `Combat` | celle du moment (Q5) |
| Partie finie (`PlayerView::game_end` présent) | `GameOver(jour ou nuit, victoire ou défaite)` | aucune (fondu de sortie) | celle du moment |

- **Fondus selon la transition** (Q4, précisée le 2026-11-20 à la demande de Mathieu) : chaque passage d'une situation
  à une autre a sa propre durée de **sortie** (l'ancien morceau baisse) et d'**entrée** (le nouveau monte), qui peuvent
  différer. Table de départ, à régler à l'oreille au fil des sons :

  | Transition | Sortie de l'ancien | Entrée du nouveau | Pourquoi |
  |---|---|---|---|
  | Silence → titre (lancement) | — | 1 s | Départ doux. |
  | Titre → exploration (partie commencée ou chargée) | 1,5 s | 1,5 s | Changement d'écran. |
  | Exploration → combat | 0,5 s | 0 s (attaque franche) | La musique de combat doit frapper tout de suite ; l'effet `CombatStart` couvre la coupure. |
  | Combat → exploration | 2 s | 3 s | Retour au calme, sans à-coup. |
  | Jour ↔ nuit (musique et ambiance) | 4 s | 4 s | Passage lent, comme la tombée de la nuit ; le coq ou le hibou marque le moment. |
  | Partie finie | 3 s | — | La musique s'efface sous le jingle `Victory` ou `Defeat`. |
  | Toute autre transition (retour au titre, chargement d'une partie en combat…) | 1 s | 1 s | Valeur de repli. |

- La table vit dans le **directeur** (une fonction `fade(depuis, vers)` d'arcanum-audio, testée), pas dans les préférences :
  c'est un réglage de mise en son, pas un choix du joueur. Ce sont des durées, pas une mécanique de jeu : le fil des sons
  les ajuste après écoute sans nouvelle validation, en le signalant dans la PR.
- Musique et ambiance tournent en boucle.
- Le choix de la tactique au début d'une manche, le repos, les fenêtres (Offres, défausse…) ne changent rien :
  exploration.

### Effets sonores

Événements filtrés (`PlayerEvent`) du joueur `PlayerId::SOLO`, sauf mention. Proposition à valider (Q3) :

| Effet (`Cue`) | Événement | Remarque |
|---|---|---|
| `Rooster` (coq) | `RoundStarted` avec `TimeOfDay::Day`, ou `GameStarted` | Premier jour compris (partie neuve) : il n'a pas de `RoundStarted` dans le journal, son coq vient de `GameStarted` (relevé au codage). |
| `Owl` (hibou) | `RoundStarted` avec `TimeOfDay::Night` | |
| `Footstep` (pas) | `HeroMoved` | Un par case, espacés de `step_interval` (0,15 s) : un trajet de trois cases donne trois pas, comme l'animation. |
| `TileRevealed` (parchemin) | `TileRevealed` | Exploration d'une tuile. |
| `CombatStart` (épées tirées) | `CombatStarted` | |
| `Block` (bouclier) | `EnemyBlocked` | Parade réussie. |
| `Miss` (souffle dans le vide) | `AttackFailed`, `BlockFailed` | Attaque ou parade ratée. |
| `Wound` (cri étouffé) | `WoundsTaken`, `WoundFromCard`, `UnitWounded` | Une seule fois par image, même si plusieurs ennemis blessent. |
| `EnemyDefeated` (coup et chute) | `EnemiesDefeated`, `EnemyDestroyed` | |
| `CombatEnd` (fin de combat) | `CombatEnded` | |
| `Spell` (sort lancé) | `CardPlayed` dont la carte est un sort (`CardId::card().is_spell()`) | Effet de base ou renforcé, même son. |
| `Conquest` (fanfare courte) | `SiteConquered`, `CityConquered` | |
| `LevelUp` (fanfare de niveau) | `LevelGained` | |
| `Recruit` (recrutement) | `UnitRecruited` | |
| `KnockedOut` (héros assommé) | `KnockedOut` | |
| `LastTurn` (cloche) | `LastTurnAnnounced` | Dernier tour annoncé. |
| `Victory` (jingle de victoire) | `GameEnded`, partie gagnée | Pour tout joueur ; la musique s'efface (situation `GameOver`). Victoire ou défaite : voir sous la table. |
| `Defeat` (jingle de défaite) | `GameEnded`, partie perdue | Idem. |
| `Preview` (son d'essai) | aucun événement : fenêtre des préférences | Échantillon des effets et du volume général, répété tant qu'on glisse leur curseur (C21). |

**Victoire ou défaite** (demande de Mathieu, 2026-11-20), lue par `sound_situation` dans la vue (`PlayerView::scenario`, son `ScenarioId`
et `game_end`), et portée par la situation `GameOver` ; le directeur choisit le jingle à l'événement `GameEnded` :

| Scénario | Victoire | Défaite |
|---|---|---|
| Conquête en solitaire | `GameEndReason::AllCitiesConquered` (mission accomplie, spec 064 ; MKUE p. 19, *Scoring*) | `LastRound` (mission échouée) |
| Première reconnaissance | `CityRevealed` : la cité est trouvée, ce qui est la mission (*your task is to locate its Capital*, MKUE p. 18) | `LastRound` : la cité n'est pas trouvée à temps |
| Bac à sable (sans objectif) | toujours `Victory` : la partie est menée à son terme, il n'y a rien à perdre (Q11) | — |

Une partie chargée déjà finie ne rejoue pas le jingle (seuls les événements nouveaux sonnent).

Règles de lecture des effets (C13 à C16) :

- **Un même effet une fois par image** (sauf les pas, un par case) : trois `WoundsTaken` dans la même image donnent un
  seul cri.
- **Quatre effets au plus par image**, choisis dans l'ordre de la table (le coq passe avant les pas) ; les autres sont
  laissés, avec une ligne `debug` des logs.
- **Variantes** : un effet peut avoir plusieurs fichiers (`footstep_1.ogg`, `footstep_2.ogg`…) ; le directeur les
  prend **à tour de rôle** (déterministe, sans générateur), pour éviter la répétition mécanique.
- Rien d'autre ne sonne : les cartes piochées, le mana, les tours du fantôme, Revenir en arrière, les clics
  d'interface restent muets (Hors périmètre).

### Les fichiers de sons

```
assets/sounds/
├── music/       title.ogg, day.ogg, night.ogg, combat.ogg
├── ambience/    day.ogg, night.ogg
├── effects/     rooster.ogg, owl.ogg, footstep_1.ogg, footstep_2.ogg, … (nom de l'effet en snake_case, variantes _N)
└── LICENSES.md  un tableau : fichier, titre, auteur, source (URL), licence, modifications
```

- **Format** : Ogg Vorbis (libre, compact, lu par Kira) ; musiques et ambiances en stéréo, qualité Vorbis 4
  (environ 128 kbit/s) ; effets en mono. Conversion par `ffmpeg` (présent dans les fils cloud, encodeur libvorbis).
  Budget du dossier : 20 Mo au plus (Q10).
- **Licences permises** pour un son : CC0, CC BY 3.0 ou 4.0, CC BY-SA 3.0 ou 4.0, OGA-BY 3.0 ; et, **en second
  choix**, CC BY-NC 3.0 ou 4.0, CC BY-NC-SA 3.0 ou 4.0 et Sampling+ 1.0 (ancienne licence Creative Commons de Freesound : copie du
  fichier tel quel pour un usage non commercial seulement ; ajoutée le 2026-11-20 par Mathieu pour le hibou) (Q12 : Mathieu ne prévoit pas d'usage commercial,
  2026-11-20). Un son NC n'est pris que si aucun candidat libre ne convient ; sa ligne de `LICENSES.md` le signale, pour
  le remplacer facilement si le projet change d'avis. Jamais ND (pas de modification : couper une boucle ou régler le
  volume en serait une). Chaque fichier a sa ligne dans `LICENSES.md`, qui fait foi
  pour les crédits ; `NOTICE` dit que `assets/sounds/` n'est pas sous CC BY-SA 4.0 mais sous la licence de chaque
  fichier (C25, C26).
- **Sources** (Q2) : **Kenney** (kenney.nl, packs CC0 : *RPG Audio*, *Impact Sounds*…) et **OpenGameArt**
  (opengameart.org, en filtrant par licence) d'abord ; **Freesound** seulement en CC0 ou CC BY, et seulement si Mathieu
  télécharge (compte exigé). **Écartés par prudence** : Pixabay et le lot *Sonniss GDC*, dont les licences interdisent
  de redistribuer les fichiers eux-mêmes, ce que fait un dépôt public (spec 082).
- **Accès réseau** : depuis un fil cloud, kenney.nl, opengameart.org et freesound.org sont aujourd'hui refusés par le
  réseau de l'environnement (essayé à la rédaction ; seul GitHub passe). Il faut les autoriser dans l'environnement
  cloud avant le fil des sons (Q2).
- **Choix des sons** : le fil des sons prépare une **planche d'écoute** (une page avec un lecteur par candidat, sa
  source et sa licence, deux ou trois candidats par son), Mathieu choisit, puis seulement les fichiers entrent dans le
  dépôt (comme la planche des illustrations, ADR 0009).

### Volumes dans les préférences

Le fichier des préférences (spec 080) gagne un groupe, lu avec ses défauts s'il manque (spec 080 C3, sans migration ni
changement de `version`) :

```json
{
  "version": 1,
  "language": "fr",
  "fullscreen_at_launch": false,
  "volumes": { "master": 80, "music": 50, "ambience": 50, "effects": 80 }
}
```

- `Volumes` (arcanum-persistance, `Default` : 80, 50, 50, 80, Q6) : quatre entiers de 0 à 100 ; une valeur au-delà de 100
  est ramenée à 100 à la lecture, avec un `warn`.
- Volume effectif d'une piste = général × type / 100. 0 = muet. La conversion en décibels (courbe perceptive) est un
  détail de `KiraPlayer`, réglé à l'oreille.
- Le champ s'appelle `volumes` et non `music` : le test de la spec 080 C3 garde `"music": 40` comme exemple de champ
  inconnu.

### Ce que fait le lancement

```
main.rs : dossier des ressources ─► SoundBank::scan(<ressources>/sounds)
          KiraPlayer::new(volumes des préférences) ── échec (pas de carte son) ─► SilentPlayer + warn
          AudioDirector::new(WALK_STEP_SECONDS)
à chaque image (après run_frame) :
          événements nouveaux = ClientSession::take_heard_events()   (filtrés pour SOLO)
          situation          = sound_situation(&client)
          pour chaque ordre de director.update(situation, événements) : player.execute(ordre)
```

`ClientSession::take_heard_events` (arcanum-ui, testée) rend les événements filtrés apparus depuis le dernier appel et
avance son curseur ; quand le journal a raccourci (retour en arrière) ou qu'une autre partie commence, le curseur se
replace sans rien rendre des anciens (C6 à C9).

## Actions et événements

Aucune action ni événement de jeu. Commandes d'écran nouvelles (`Command`, comme celles de la spec 080) :

| Commande | Effet |
|---|---|
| `Command::SetVolume(SoundCategory, u8)` | Change un volume pour la session, tout de suite (pendant qu'on glisse le curseur), sans écrire le fichier ; au premier changement d'un glissé, demande l'échantillon de ce type de son. `SoundCategory` : `Master`, `Music`, `Ambience`, `Effects`. |
| `Command::SaveVolumes` | Écrit le fichier (au lâcher du curseur) et arrête l'échantillon. |

La session tient les volumes (dans `Preferences`) et les transmet par une demande à l'application
(`WindowRequest::Volumes(Volumes)`, nouveau), qui les passe au lecteur ; l'échantillon passe par
`WindowRequest::Preview(Some(catégorie))`, son arrêt par `WindowRequest::Preview(None)`.

**Échantillon pendant le glissé** (demande de Mathieu, 2026-11-20) : tant qu'on tient un curseur, on entend un
échantillon de son type de son, joué sur la piste de ce type, donc au volume qu'on est en train de régler :

| Curseur | Échantillon |
|---|---|
| Musique | La musique en cours si la situation en a une ; sinon (fin de partie) un extrait de la musique `Title`, en boucle. |
| Ambiances | L'ambiance en cours si la situation en a une ; sinon (écran de lancement) l'ambiance `Day`, en boucle. |
| Effets sonores | L'effet `Preview`, rejoué dès qu'il se termine. |
| Volume général | L'effet `Preview` aussi : la musique et l'ambiance en cours varient déjà avec lui. |

Au lâcher, l'échantillon ajouté s'arrête par un fondu de 0,5 s ; la musique ou l'ambiance qui jouaient déjà continuent.
Le curseur réglé au clavier (flèches, sans glissé) fait de même : un changement démarre l'échantillon, et il s'arrête
1 s après le dernier changement. La session reste testable sans carte son, comme pour la langue (spec 080).

Écart assumé avec la spec 080 (« enregistrement immédiat ») : un curseur qu'on glisse change sa valeur à chaque image ;
le fichier n'est écrit qu'au lâcher, pas trente fois par seconde.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Exemples d'événements écrits à la main dans les
tests du directeur (aucune partie à jouer) ; les tests de la session partent d'une partie de Première reconnaissance
(`first_reconnaissance_config`, `tests/support/`).

**Le directeur (arcanum-audio)**

- **C1** — Première image en `Title` : ordre de jouer la musique `Title` avec une entrée de 1 s, aucune ambiance
  (`title_starts_title_music`).
- **C2** — Passage de `Title` à `Exploration(Day)` : musique `Day` et ambiance `Day`, sortie et entrée de 1,5 s ; une
  seconde image dans la même situation ne rend aucun ordre (`exploration_by_day_crossfades_once`).
- **C3** — `Exploration(Day)` puis `Combat(Day)` : musique `Combat`, sortie de 0,5 s et entrée immédiate, ambiance
  inchangée (aucun ordre d'ambiance) ; puis `Exploration(Day)` : retour à `Day`, sortie de 2 s et entrée de 3 s
  (`combat_switches_music_and_back`).
- **C4** — `Exploration(Day)` puis `Exploration(Night)` : musique et ambiance de nuit, sortie et entrée de 4 s
  (`nightfall_switches_music_and_ambience`).
- **C5** — `GameOver(Night)` : la musique s'efface en 3 s (aucune nouvelle), l'ambiance reste
  (`game_over_fades_music_out`).

- **C27** — Une transition hors table (de `Combat(Day)` à `Title`) prend la valeur de repli, 1 s et 1 s
  (`unlisted_transition_uses_fallback_fade`).

**Les événements entendus (arcanum-ui, session)**

- **C6** — Partie neuve : le premier `take_heard_events` rend les événements de la mise en place, à
  commencer par `GameStarted` (le premier jour n'a pas de `RoundStarted`, relevé au codage) ; un second appel
  sans rien jouer ne rend rien (`new_game_events_are_heard_once`).
- **C7** — Après un déplacement puis Revenir en arrière : l'appel qui suit le retour ne rend rien, et un nouveau
  déplacement est rendu (`undo_replays_no_sound`).
- **C8** — Partie chargée (fenêtre de chargement) ou reprise par « Continuer » : le premier appel ne rend rien
  (`loaded_game_is_not_replayed`).
- **C9** — Les événements rendus sont ceux de `filtered_for(PlayerId::SOLO)` : le mélange de la pioche de la mise en
  place y est `PlayerEvent::DeckShuffled { count }`, sans l'ordre des cartes
  (`heard_events_are_filtered_for_the_player`).

**La situation (arcanum-ui)**

- **C10** — `sound_situation` : écran de lancement → `Title` ; écran de nouvelle partie ouvert en cours de partie →
  `Title` ; partie de jour sans combat → `Exploration(Day)` (`situation_follows_the_screen`).
- **C11** — Partie en combat (`PlayerView::combat` présent et pas fini), « Voir le plateau » compris → `Combat(…)` ; combat
  fini (`CombatEnded`, il reste dans la vue jusqu'à la fin du tour) → `Exploration(…)`, issue #18
  (`situation_returns_to_exploration_once_combat_ended`) ; partie finie (`PlayerView::game_end`) → `GameOver(…)`
  (`situation_in_combat_and_after_game_end`).
- **C28** — Victoire ou défaite de la situation `GameOver`, une ligne par case du tableau : Conquête en solitaire finie par
  `AllCitiesConquered` → victoire, par `LastRound` → défaite ; Première reconnaissance par `CityRevealed` → victoire, par
  `LastRound` → défaite ; bac à sable → victoire (`game_over_tells_victory_or_defeat`).
- **C29** — Le directeur, à `GameEnded` : `Victory` si la situation dit victoire, `Defeat` sinon
  (`game_ended_plays_victory_or_defeat`).

**Les effets (arcanum-audio)**

- **C12** — Chaque ligne de la table des effets : l'événement donne l'effet attendu (`RoundStarted` de nuit → `Owl`, `GameStarted` → `Rooster`,
  `EnemyBlocked` → `Block`, `CardPlayed` de Démolition, carte `Spell::Demolish` de `spells.ron`, → `Spell`,
  `CardPlayed` de Détermination (`BasicAction::Determination`) → aucun effet, etc.) (`events_map_to_cues`). Le fil d'implémentation reprend les
  identifiants exacts du contenu (gabarit, issue privée 158).
- **C13** — Trois `WoundsTaken` dans la même image : un seul `Wound` (`same_cue_once_per_frame`).
- **C14** — Trois `HeroMoved` dans la même image : trois `Footstep`, avec des délais 0 ; 0,15 s ; 0,30 s
  (`footsteps_follow_the_walk`).
- **C15** — Six effets différents dans la même image : les quatre premiers dans l'ordre de la table sont joués
  (`at_most_four_cues_per_frame`).
- **C16** — Un effet à trois variantes joué quatre fois : variantes 1, 2, 3, 1 (`variants_rotate`).
- **C17** — Événements d'un autre joueur que `SOLO` (sauf `GameEnded`) : aucun effet (`only_the_human_player_is_heard`).

**Les volumes (arcanum-persistance et arcanum-ui)**

- **C18** — `Volumes::default()` : 80, 50, 50, 80 ; écrire puis relire des préférences avec des volumes rend les mêmes
  (`volumes_default_and_round_trip`).
- **C19** — Un fichier de la spec 080, sans `volumes`, se lit avec les volumes par défaut ; `"music": 140` dans
  `volumes` se lit 100, avec un `warn` (`volumes_tolerate_old_and_out_of_range_files`).
- **C20** — `SetVolume(Music, 30)` : la session demande `WindowRequest::Volumes` avec la musique à 30, la fenêtre montre
  30, le fichier n'est pas écrit ; `SaveVolumes` l'écrit (`volume_applies_live_and_saves_on_release`).
- **C21** — Premier `SetVolume(Effects, …)` d'un glissé : `WindowRequest::Preview(Some(Effects))`, une seule fois pour
  tout le glissé ; `SaveVolumes` : `WindowRequest::Preview(None)`. De même pour `Music`, `Ambience` et `Master`
  (`dragging_a_volume_plays_its_sample`).
- **C30** — Le directeur, à `Preview(Some(Music))` en fin de partie (aucune musique en cours) : extrait de `Title` sur la
  piste de la musique ; dans une situation qui a sa musique : aucun ordre de plus ; à `Preview(Some(Ambience))` sur
  l'écran de lancement : ambiance `Day` ; à `Preview(None)` : arrêt de ce qu'il avait ajouté, fondu de 0,5 s
  (`preview_uses_current_sound_or_a_sample`).
- **C22** — Écriture impossible : comme la spec 080 C21, le volume vaut pour la session et la fenêtre montre
  « Préférences non enregistrées (voir les logs). » (`unsaved_volumes_show_message`).
- **C23** — La fenêtre des préférences a une section « Son » : quatre curseurs (Volume général, Musique, Ambiances,
  Effets sonores) avec leur valeur (`PreferencesWindow::volumes`) ; sans sortie audio, la ligne « Aucune sortie audio :
  le jeu reste muet. » (`PreferencesWindow::no_audio`) (`preferences_window_lists_volumes`).

**La banque de sons et les licences (arcanum-audio)**

- **C24** — `SoundBank::scan` d'un dossier temporaire avec `music/day.ogg`, `effects/footstep_1.ogg`,
  `effects/footstep_2.ogg` : la musique `Day` et deux variantes de `Footstep` ; un effet sans fichier n'est pas une
  erreur, et son premier appel donne un seul `warn` qui le nomme (`bank_finds_files_and_variants`).
- **C25** — Chaque fichier `.ogg` de `assets/sounds/` a sa ligne dans `LICENSES.md`, avec une licence de la liste
  permise, et chaque ligne nomme un fichier présent ; les sons NC sont comptés et listés dans la sortie du test
  (`every_sound_has_a_permitted_license`).
- **C26** — Une fois les sons ajoutés (fil des sons) : chaque musique, ambiance et effet de la table a au moins un
  fichier, et le dossier fait 20 Mo au plus (`every_cue_has_a_sound_within_budget`).

## Cas limites et refus

- **L1** — Pas de carte son (création de `KiraPlayer` en échec) : `SilentPlayer`, un `warn` dans les logs, le jeu
  tourne normalement ; la fenêtre le dit (C23). Vérifié à l'œil dans un fil cloud (pas de carte son sous Xvfb).
- **L2** — Aucun dossier des ressources, ou pas de dossier `sounds/` : banque vide, rien ne sonne, un seul `info` dans
  les logs (`missing_sounds_dir_gives_empty_bank`).
- **L3** — Fichier de son illisible (Vorbis invalide) : l'effet est sauté avec un `warn` qui nomme le fichier, une fois
  (`KiraPlayer`, vérification à l'oreille avec un faux fichier).
- **L4** — Volume général à 0 : rien ne s'entend, mais le directeur continue (la musique reprend au bon endroit de la
  situation quand on remonte le volume) (`zero_master_volume_keeps_directing`).
- **L5** — Les tests d'arcanum-ui et d'arcanum-audio n'ouvrent jamais la carte son : seul `main.rs` crée `KiraPlayer`.
- **L6** — Les messages des logs de C19, C24, L1 à L3 sont en français, en dur (spec 003).

### Tests existants touchés

Relevés par `grep -rn "Preferences {" crates --include=*.rs` (appels du constructeur, issue privée 357) et
`grep -rn "SetFullscreenAtLaunch" crates` (listes des commandes de préférences) :

- `Preferences { language, fullscreen_at_launch }` écrit en entier : trois fois dans
  `crates/arcanum-persistance/tests/preferences.rs` (C2, C3, C4 de la spec 080) et deux fois dans
  `crates/arcanum-ui/tests/preferences.rs` (lignes 164 et 377) : ils gagnent `..Preferences::default()` (même comportement).
- `PreferencesWindow` gagne des champs : `crates/arcanum-ui/tests/preferences.rs` ne le construit pas en entier (lit ses
  champs, fonction `window`) : rien à changer, à revérifier au codage.
- `crates/arcanum-ui/src/session.rs` (ligne 615) énumère les commandes de préférences qui ne vont pas à la partie : il gagne
  `SetVolume` et `SaveVolumes` (code, pas un test).
- `KEY_BINDINGS` et le Menu ne changent pas (aucune touche, Q7) : `menu_lists_commands_with_keys` et
  `bar_stays_during_combat` intacts.

## Information cachée

- Les effets n'entendent que les événements filtrés pour le joueur (`filtered_for(PlayerId::SOLO)`, C9).
- La situation se lit dans la vue (`PlayerView::combat`, `time_of_day`, `game_end`) et dans l'écran (lancement, page de
  nouvelle partie) : aucun champ de vue nouveau.

## Libellés (i18n)

Clés nouvelles dans le groupe `preferences` (existant ; aucune des clés ci-dessous n'y est, vérifié dans `fr.json` de
`main` : `title`, `button`, `language`, `system_language`, `fullscreen_at_launch`, `fullscreen_hint`, `not_saved`,
`close`) :

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `preferences.sound` | — | Son | Sound |
| `preferences.volume_master` | — | Volume général | Master volume |
| `preferences.volume_music` | — | Musique | Music |
| `preferences.volume_ambience` | — | Ambiances | Ambience |
| `preferences.volume_effects` | — | Effets sonores | Sound effects |
| `preferences.volume_value` | `percent` | {percent} % | {percent}% |
| `preferences.no_audio` | — | Aucune sortie audio : le jeu reste muet. | No audio output: the game stays silent. |

Texte obtenu : `preferences.volume_value` avec `{percent}` = 80 donne « 80 % » (espace insécable avant %, usage
français) et « 80% ». Le curseur egui affiche sa valeur par ce libellé (`Slider::custom_formatter`), pas par un « % »
en dur.

Réutilisés : `preferences.title` « Préférences », `preferences.not_saved` « Préférences non enregistrées (voir les
logs). » (C22), `preferences.close` « Fermer ». Aucun caractère nouveau (l'espace insécable s'affiche déjà, à vérifier
à l'œil).

## Interface

```
┌ Préférences ─────────────────────────────── × ┐
│ Langue                                         │
│   [Langue du système (Français)          ▾]    │
│                                                │
│ [✓] Plein écran au lancement                   │
│     F11 bascule la fenêtre sans changer ce     │
│     réglage.                                   │
│                                                │
│ Son                                            │
│   Volume général   [━━━━━━━━━━━━━━●━━━]  80 %  │
│   Musique          [━━━━━━━━━●━━━━━━━━]  50 %  │
│   Ambiances        [━━━━━━━━━●━━━━━━━━]  50 %  │
│   Effets sonores   [━━━━━━━━━━━━━━●━━━]  80 %  │
│   (Aucune sortie audio : le jeu reste muet.)   │
│                                                │
│ (Préférences non enregistrées (voir les logs).)│
│                                     [Fermer]   │
└────────────────────────────────────────────────┘
```

- **Section « Son »** sous le plein écran : titre `preferences.sound`, quatre curseurs de 0 à 100 avec leur libellé et
  leur valeur (C23, `PreferencesWindow::volumes`). Mise en page : vérification à l'œil.
- **Glisser un curseur** change le volume tout de suite et fait entendre un échantillon de son type de son à ce volume
  (C20, C21, C30) ; le **lâcher** écrit le fichier et arrête l'échantillon (C20, C21).
- **Aucune sortie audio** : la ligne `preferences.no_audio` en couleur d'avertissement sous les curseurs, qui restent
  réglables (C23, `PreferencesWindow::no_audio`).
- **Musique, ambiance et effets** : rien à l'écran ; vérification à l'oreille par Mathieu, sur son poste Windows et
  sur Linux Mint (un fil cloud n'a pas de carte son).

**Vérification à l'oreille** (par Mathieu, liste préparée par le fil écran dans la PR) : lancement (musique de titre),
nouvelle partie (coq, passage à la musique de jour), déplacement de trois cases (trois pas), exploration (parchemin),
combat (bascule de la musique, épées, parade, blessure, ennemi vaincu, fin de combat et retour à la musique de jour),
sort lancé, fin de manche vers la nuit (hibou, musique et ambiance de nuit), curseurs (échantillon de chaque type pendant le glissé),
Revenir en arrière (aucun son). Parties préparées réutilisées si elles existent pour le combat et la nuit (le fil écran
les choisit dans `docs/verification-manuelle.md` ; une nouvelle seulement s'il en manque une, nommée dans la PR). Dans
un fil cloud : capture de la fenêtre des préférences et de la ligne « Aucune sortie audio ».

### Raccourcis clavier

Aucun (Q7) : couper le son passe par le volume général à 0.

## Documentation

- **ADR 0010 Audio** (`docs/decisions/0010-audio.md`, rédigé avec la spec) : crate arcanum-audio, Kira, directeur pur et
  lecteur, sons sous licences libres et `LICENSES.md`.
- **CLAUDE.md** : arcanum-audio dans l'arbre et les dépendances (`arcanum-ui → arcanum-audio → arcanum-core`), `assets/sounds/` et ses
  licences dans « Assets et droits », paquet `libasound2-dev` pour compiler sous Linux.
- **`.claude/rules/arcanum-audio.md`** : le son écoute et ne décide rien, événements filtrés, pas de carte son dans les
  tests, un fichier = une ligne de `LICENSES.md`.
- **`docs/architecture.md`** : la crate, ses modules, le branchement dans `app/`, la recette « ajouter un effet
  sonore ».
- **`NOTICE`** : `assets/sounds/` sous la licence de chaque fichier (`assets/sounds/LICENSES.md`).
- **README** : paquet Linux, volumes dans les préférences.
- **`docs/ci.md`** et `.github/workflows/ci.yml` : installation de `libasound2-dev` dans le job Linux.
- **Glossaire** : la section « Audio (spec 081) » (dans cette PR).

## Hors périmètre

- **Spatialisation** (son placé à gauche ou à droite selon la case) : le plateau tient dans l'écran et le gain est
  faible pour le travail ; si Mathieu la veut, une issue (Q8).
- **Musique en couches synchronisées** (une couche de combat qui monte sur la même mesure) : demande des morceaux
  composés pour, introuvables tels quels sous licence libre ; fondu enchaîné à la place (Q4).
- **Sons d'interface** (clics, survol), **cartes piochées**, **mana**, **tours du fantôme** : pas demandés.
- **Plusieurs morceaux par situation** (liste de lecture) : un seul pour commencer ; la banque accepte déjà des
  variantes pour les effets.
- **Couper le son quand la fenêtre perd le focus** : pas demandé.
- **Voix** (héros qui parlent) : non.
- **macOS** : Kira passe par CoreAudio sans rien de plus ; la spec #7 le vérifie.

## Découpage (grosse spec, issue privée 118)

Deux fils successifs sur la même branche :

1. **Moteur** : crate arcanum-audio (directeur, table des effets, banque de sons, `LICENSES.md` vide et son test C25,
   `KiraPlayer` et `SilentPlayer`), volumes dans arcanum-persistance, `take_heard_events` et `sound_situation` dans arcanum-ui,
   `verifier.sh` et CI pour `libasound2-dev`, `cargo deny`. C1 à C22, C24, C25, C27 à C30, L2, L4, L5.
2. **Sons et écran** : planche d'écoute et choix de Mathieu, fichiers et `LICENSES.md`, C26 ; section « Son » de la
   fenêtre (C23), branchement dans `app/` et `main.rs`, L1, L3 ; docs ; liste de vérification à l'oreille.

## Décisions

Validées par Mathieu le 2026-11-20 : les réponses proposées sont retenues.

- **Q1 — Bibliothèque audio.** **Kira 0.12** (demandée dans l'issue), sans ses options par défaut (`cpal`, `ogg`, `vorbis`). Pourquoi elle : pistes et volumes par type, fondus (`Tween`), départ différé (`StartTime::Delayed`, pour les pas), lecture en flux, licences compatibles (relevées plus haut). Écartée : rodio, plus simple, mais sans pistes ni fondus enchaînés : il faudrait les écrire à la main.
- **Q2 — Sources des sons et accès réseau.** Kenney et OpenGameArt d'abord, Freesound seulement en CC0 ou CC BY si Mathieu télécharge, Pixabay et Sonniss écartés (redistribution des fichiers interdite). Il faut **autoriser kenney.nl et opengameart.org** dans le réseau de l'environnement cloud (aujourd'hui refusés) avant le fil des sons ; sinon Mathieu télécharge les candidats et les déposes dans le projet.
- **Q3 — Liste des effets.** Les dix-neuf effets de la table (dont les sept exemples de Mathieu, et victoire et défaite séparées).
- **Q4 — Passage exploration ↔ combat.** Des fondus propres à chaque transition (table de la section Situation sonore : entrée en combat franche, retour au calme lent, jour et nuit très lents), entre deux morceaux en boucle, plutôt que des couches synchronisées (Hors périmètre). Quatre musiques : titre, jour, nuit, combat.
- **Q5 — Ambiance en combat.** L'ambiance de jour ou de nuit continue sous la musique de combat.
- **Q6 — Volumes.** Quatre curseurs (général, musique, ambiances, effets), défauts 80, 50, 50, 80.
- **Q7 — Touche pour couper le son.** Aucune (volume général à 0).
- **Q8 — Spatialisation.** Hors périmètre (gain faible ici).
- **Q9 — Paquet Linux `libasound2-dev`.** `verifier.sh` l'installe s'il manque (comme `verif-manuelle.sh` pour Xvfb), la CI de nuit l'installe, le README le dit pour Linux Mint.
- **Q10 — Poids des sons dans le dépôt.** 20 Mo au plus (Ogg Vorbis qualité 4, effets en mono), vérifié par C26. Quatre musiques de deux à trois minutes font environ 12 Mo.
- **Q11 — Bac à sable et fin de partie.** Jingle de victoire (rien à perdre).
- **Q12 — Sons sous licence NC.** Permis en second choix (Mathieu ne prévoit pas d'usage commercial), signalés dans `LICENSES.md`. Ce qu'ils coûtent : `NOTICE` doit dire que ces fichiers ne sont pas réutilisables commercialement, alors que le reste du dépôt (GPL, CC BY-SA) l'est ; une distribution Linux (Debian, Flathub selon ses règles) refuserait un paquet qui les contient ; et un jour de vente, de financement participatif ou de dons réguliers (zone grise du « non commercial »), il faudrait les remplacer.
- **Hibou en Sampling+ 1.0** (Mathieu, 2026-11-20, au codage) : seul hululement convenable trouvé sur Freesound ;
  Sampling+ rangée avec les licences NC de second choix (section Les fichiers de sons), le fichier est signalé dans
  `LICENSES.md`.
- **Freesound** (Mathieu, à la validation) : si le fil des sons ne trouve pas ce qu'il faut chez Kenney ou OpenGameArt, il
  demande à Mathieu, qui télécharge sur Freesound (compte exigé).

## Questions ouvertes

Aucune.
