# Planche d'écoute des sons

Outil de la spec 081 (Audio) pour choisir un son avec Mathieu : chercher des candidats, relever leur licence, les mettre
sur une page où il les écoute et coche son choix, puis convertir le son retenu au format de `assets/sounds/`. Un fil
cloud n'a pas de carte son : c'est Mathieu qui écoute.

Prérequis : Python 3, `ffmpeg` (encodeurs libmp3lame et libvorbis, présents dans les fils cloud) ; pour une archive
`.7z`, `pip install py7zr` ou la commande `7z`. Téléchargements en cache dans `target/sons/` (`ARCANUM_SONS_CACHE` pour un
autre dossier), jamais versionnés.

## Étapes

```
S=scripts/sons/planche.py
$S chercher effet sword clash          # pages OpenGameArt (effet ou musique), les plus téléchargées d'abord
$S fiche 20-sword-sound-effects-attacks-and-clashes   # titre, auteur, licence, fichiers à télécharger
$S construire scripts/sons/exemple.json target/planche   # data.json + planche.html
```

1. **Chercher** sur OpenGameArt (`chercher`, puis `fiche` pour la licence). kenney.nl et freesound.org refusent les
   requêtes d'un fil cloud : demander à Mathieu les packs Kenney (CC0) ou les sons Freesound (compte exigé), et les
   citer par leur chemin local dans le manifeste.
2. **Ne garder que les licences permises** par la spec 081 (`FREE_LICENSES` et `NON_COMMERCIAL_LICENSES` de
   `crates/arcanum-audio/src/licenses.rs`). Pixabay est écarté : sa licence interdit de redistribuer le fichier seul.
3. **Manifeste** (`exemple.json`) : un besoin par son du jeu (`groupe` : Musiques, Ambiances ou Effets ; `cle` : son
   chemin dans `assets/sounds/` sans extension), deux à quatre candidats chacun. `fichier` est une URL ou un chemin
   local, `membres` un motif dans l'archive ou le dossier (plusieurs fichiers = variantes), `debut` et `duree` en
   secondes (défauts : 30 s pour une musique ou une ambiance, 4 s pour un effet).
4. **Construire**, puis publier `planche.html` en Artifact avec `data.json` à côté (16 Mo au plus : extraits en mp3,
   80 kbit/s en stéréo, 48 en mono). Mathieu coche et colle le récapitulatif dans le fil (« effects/block : C »).
5. **Convertir** le son retenu, et ajouter sa ligne dans `assets/sounds/LICENSES.md` :

   ```
   $S convertir <url ou chemin> assets/sounds/effects/block.ogg            # effet : mono
   $S convertir <url ou chemin> assets/sounds/music/day.ogg --stereo       # musique ou ambiance : stéréo
   ```

   Les tests `every_sound_has_a_permitted_license` et `every_cue_has_a_sound_within_budget` (arcanum-audio) vérifient la
   ligne, la licence, et le budget de 20 Mo.

Leçons de la spec 081 : choisir à l'aveugle a demandé trois tours ; demander d'abord à Mathieu l'ambiance voulue
(« lames qui s'entrechoquent », « hululement »), et proposer quatre ou cinq candidats contrastés par son.
