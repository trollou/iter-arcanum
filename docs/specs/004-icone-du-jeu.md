# 004 — Icône du jeu

- Statut : implémentée (2026-10-25)
- Jalon : J0 (transverse, hors des jalons de règles)
- Dépend de : 002 (fenêtre, `main.rs`), 003 (logs)
- Livret : aucune règle de jeu.
- Crates touchées : arcanum-ui (code, `build.rs`), `assets/images/icon/` (nouveau)

## Objectif

Le jeu a sa propre icône, créée par nous. Elle s'affiche :

- sur le bouton de la fenêtre dans la barre des tâches, sous Windows comme sous Linux, quand le jeu est lancé ;
- sur le fichier `iter-arcanum.exe` dans l'explorateur Windows (et ses raccourcis).

Un exécutable Linux (ELF) n'a pas d'icône de fichier : sous Linux, seule l'icône de fenêtre est concernée (voir Hors périmètre).

## Règles

Aucune règle de jeu. Contraintes du projet :

- **Droits** (`CLAUDE.md`, « Assets et droits ») : visuel de notre cru, aucun logo ni élément repris de Mage Knight.
- **Portabilité** (ADR 0004) : le code propre à Windows est isolé derrière `#[cfg]` (ici, dans `build.rs`, derrière la cible de compilation).
- **Pas d'échec silencieux** (spec 003) : si l'icône ne peut pas être préparée, le jeu se lance quand même et l'écrit dans les logs.

## Modèle

### Le visuel

`assets/images/icon/iter-arcanum.svg`, carré 256 × 256, choisi par Mathieu le 2026-11-13 avec le nouveau nom du jeu (Iter Arcanum) : un
**bouclier** bleu nuit à liseré or, un **cristal violet** lumineux au centre et deux étincelles. Mêmes contours brun foncé que la figurine
du héros (spec 011). Aucun texte ni initiale. De notre cru, sans reprendre le logo du jeu. Il remplace le premier visuel (lettres MK et
épée dans un hexagone) ; le changer n'a demandé aucun changement de code. Aperçu : `images/004-icone-apercu.png` (256, 48, 32 et 16 pixels).

Le SVG est la **source unique**. Deux fichiers en sont dérivés :

| Fichier | Usage | Produit par |
|---|---|---|
| Image RGBA 256 × 256 en mémoire | Icône de fenêtre (Windows et Linux) | rendu du SVG au lancement, par le moteur SVG déjà utilisé pour le plateau de jeu (resvg via `egui_extras`) |
| `assets/images/icon/iter-arcanum.ico` (versionné) | Icône de l'exécutable Windows | un outil du dépôt (`cargo run -p arcanum-ui --example icone`), à relancer quand le SVG change ; un test vérifie qu'il est à jour |

L'`.ico` contient les tailles 16, 24, 32, 48 et 256 pixels, celles qu'utilise Windows selon l'affichage et l'échelle de l'écran.

### Icône embarquée dans le binaire

L'icône de fenêtre est **compilée dans le binaire** (`include_bytes!` du SVG), et non lue dans le dossier des ressources comme les
autres images (spec 011). Raisons : elle sert avant l'ouverture de la fenêtre, elle doit exister même quand le dossier des ressources
est introuvable, et l'icône de l'exécutable Windows est de toute façon embarquée. Changer le SVG demande donc de recompiler.

### Identifiant d'application

La fenêtre déclare l'identifiant d'application `iter-arcanum` (`ViewportBuilder::with_app_id`). Sous Wayland, c'est lui qui relie la
fenêtre à un fichier `.desktop` et à son icône ; sous X11 et Windows il est sans effet. Le déclarer dès maintenant prépare la spec
d'installation sans rien coûter.

### Fichier `.desktop` sous Linux (issue privée 315)

Ajout du 2026-11-17 (choix de Mathieu sur l'issue privée 315). winit ne pose qu'**une** image dans l'icône de fenêtre X11, en 256 px.
Faute de fichier `.desktop` qui corresponde à la fenêtre, Cinnamon (Linux Mint) la réduit à la taille de la barre des tâches
sans lissage : liseré et contours deviennent des pointillés. Avec un `.desktop`, Cinnamon prend l'icône du thème, rendue à la bonne taille.

Au lancement, **sous Linux seulement** (`#[cfg(target_os = "linux")]`), le jeu écrit dans le dossier des données de l'utilisateur
(`$XDG_DATA_HOME`, sinon `~/.local/share`) :

| Fichier | Contenu |
|---|---|
| `applications/iter-arcanum.desktop` | `Type=Application`, `Name=Iter Arcanum`, `Exec=` chemin de l'exécutable lancé (entre guillemets, échappé selon la norme Desktop Entry), `Icon=iter-arcanum`, `StartupWMClass=iter-arcanum` (la classe `WM_CLASS` que pose l'identifiant d'application), `Categories=Game;`, `Terminal=false`, et la marque `X-Iter-Arcanum-Generated=true` |
| `icons/hicolor/scalable/apps/iter-arcanum.svg` | le SVG embarqué de l'icône, tel quel |

Le nom affiché reste en dur (`Iter Arcanum`) : c'est le nom du jeu, pas un libellé à traduire. Un `.desktop` **sans la marque**
(écrit ou modifié par l'utilisateur) n'est jamais touché ; un `.desktop` à nous est réécrit s'il a changé (exécutable déplacé).
Un échec d'écriture ne bloque pas le lancement : il est écrit dans les logs (spec 003), comme un échec de l'icône de fenêtre.

## Actions et événements

Aucun : la spec ne touche pas au moteur.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Les tests sont dans arcanum-ui, sans fenêtre.

- **C1** — Étant donné le SVG embarqué, quand on prépare l'icône de fenêtre, alors on obtient une image RGBA de 256 × 256 pixels
  (`window_icon_is_256_pixels_square`).
- **C2** — L'image n'est pas vide : le centre de l'hexagone est opaque et les coins du carré, hors de l'hexagone, sont transparents
  (`window_icon_has_transparent_corners_and_opaque_center`).
- **C3** — L'alpha de l'image est non prémultiplié, comme l'attend la fenêtre : un pixel opaque garde sa couleur, un pixel transparent
  vaut `(0, 0, 0, 0)` (`window_icon_alpha_is_not_premultiplied`).
- **C4** — Les réglages de la fenêtre (titre, taille de la spec 002) portent l'icône et l'identifiant d'application `iter-arcanum`
  (`viewport_carries_icon_and_app_id`).
- **C5** — Étant donné une icône impossible à préparer, quand on construit les réglages de la fenêtre, alors ils n'ont pas d'icône,
  le reste est inchangé, et une erreur est écrite dans les logs (`viewport_without_icon_when_icon_fails`, `icon_failure_is_logged`).
- **C6** — `iter-arcanum.ico` contient exactement les tailles 16, 24, 32, 48 et 256 (`ico_contains_expected_sizes`).
- **C7** — Chaque image de l'`.ico` correspond au rendu du SVG à la même taille, à une petite tolérance près (arrondis de compression) :
  si quelqu'un change le SVG sans relancer l'outil, le test échoue et dit quelle commande lancer (`ico_matches_svg`).
- **C8** — Sous Windows, `iter-arcanum.exe` contient une ressource icône (groupe d'icônes) : test d'intégration qui lit l'exécutable compilé,
  compilé et lancé seulement sous Windows (`windows_exe_has_icon_resource`). La CI Windows le fait tourner.

Vérification à l'œil, en plus des tests : capture de la barre des tâches sous Linux (conteneur cloud, Xvfb) jointe à la PR ; sous Windows,
Mathieu regarde l'icône de `iter-arcanum.exe` dans l'explorateur et dans la barre des tâches.

- **C9** — Le `.desktop` nomme l'exécutable, l'icône `iter-arcanum` et la classe de fenêtre `iter-arcanum`, et porte la marque
  (`desktop_entry_names_exe_icon_and_window_class`).
- **C10** — Un chemin d'exécutable avec espace, guillemet, `$` ou antislash est mis entre guillemets et échappé
  (`desktop_entry_quotes_and_escapes_exec_path`).
- **C11** — Étant donné un dossier des données vide, l'installation écrit le `.desktop` et le SVG de l'icône
  (`install_writes_desktop_file_and_svg_icon`).
- **C12** — Un `.desktop` sans la marque est laissé tel quel ; l'icône est quand même écrite (`install_keeps_user_desktop_file`).
- **C13** — Un `.desktop` à nous, écrit pour un autre exécutable, est réécrit avec le nouveau chemin
  (`install_rewrites_own_desktop_file_when_exe_moves`).

Vérification à l'œil (issue privée 315) : Mathieu relance le jeu sous Linux Mint et regarde l'icône de la barre des tâches.

## Cas limites et refus

- **L1** — Compilation Windows sans compilateur de ressources (`rc.exe` du SDK Windows, ou `windres` de MinGW) : la compilation **réussit**,
  avec un avertissement Cargo qui l'explique ; l'exécutable n'a pas d'icône de fichier mais garde l'icône de fenêtre. C'est le cas probable
  du poste de Mathieu (toolchain GNU sans droits admin). La CI Windows (toolchain MSVC, SDK présent) produit toujours l'icône, et C8 l'y vérifie.
- **L2** — Compilation sous Linux : `build.rs` ne fait rien (pas de dépendance Windows compilée, cible différente).
- **L3** — Dossier des ressources introuvable au lancement : l'icône de fenêtre est là quand même (embarquée).

## Information cachée

Sans objet.

## Libellés (i18n)

Aucune clé : l'icône ne contient pas de texte.

## Interface

L'icône apparaît sur le bouton de la fenêtre dans la barre des tâches et dans la barre de titre quand le système l'y montre.
Rien ne change dans la fenêtre elle-même.

### Raccourcis clavier

Aucun.

## Nouvelles dépendances (à valider)

| Crate | Type | Pourquoi | Alternative écartée |
|---|---|---|---|
| `winresource` | dépendance de build, **cible Windows seulement** | Écrit le fichier de ressources et appelle le compilateur de ressources (`rc.exe` ou `windres`) pour embarquer l'`.ico` dans l'exe. Fork maintenu de `winres`. | `embed-resource` : équivalent, mais demande d'écrire et de tenir un fichier `.rc` à la main. |
| `ico` | dépendance de développement (tests et outil) | Lire et écrire un `.ico`, en Rust pur, petite crate. Sert à C6, C7 et à l'outil qui produit l'`.ico`. | ImageMagick ou un site web : pas reproductible, pas installable sur le poste de Mathieu. |
| `object` | dépendance de développement, **cible Windows seulement** | Lire les ressources de `iter-arcanum.exe` pour C8. Crate de référence (utilisée par la bibliothèque standard de Rust). | Pas de test, vérification à l'œil seulement. |

Aucune nouvelle dépendance dans le code livré au joueur : le rendu SVG passe par `egui_extras`, déjà présent.

## Hors périmètre

- **Installation sous Linux** au-delà du `.desktop` écrit au lancement (issue privée 315) : paquet, désinstallation (retirer les deux
  fichiers), icône hors du thème `hicolor`. À traiter avec une future spec d'installation et de distribution.
- **Installeur Windows**, raccourci du menu Démarrer : même future spec.
- **Informations de version** dans l'exe Windows (éditeur, description) : possibles avec `winresource`, non demandées.
- macOS : hors cible (`CLAUDE.md`).

## Questions ouvertes

Aucune.

## Décisions (2026-10-25, Mathieu)

1. **Visuel** : MK devant l'épée, dans l'hexagone. Définitif, pas provisoire.
2. **Dépendances** : `winresource` (build, Windows), `ico` (dev) et `object` (dev, Windows) acceptées.
3. **L1** : une compilation Windows sans compilateur de ressources avertit et réussit.
4. **Linux** : le fichier `.desktop` attend une future spec d'installation.
