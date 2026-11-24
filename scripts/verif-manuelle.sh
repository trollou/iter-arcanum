#!/usr/bin/env bash
# Vérification manuelle du client sous Linux, sans écran : Xvfb, xdotool et captures.
# Mode d'emploi : docs/verification-manuelle.md. Linux seulement (conteneur cloud ou PC Linux).
set -euo pipefail

RACINE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ETAT="$RACINE/target/verif-manuelle"
ECRAN="${ARCANUM_VERIF_ECRAN:-:99}"
TAILLE="${ARCANUM_VERIF_TAILLE:-1600x900}"
# Langue du jeu (spec 076) : français par défaut, la langue du système sous Xvfb vaut souvent C ou en.
LANGUE="${ARCANUM_VERIF_LANGUE:-fr}"
if [ -z "${ARCANUM_VERIF_CAPTURES:-}" ]; then
  if [ -d /mnt/project-files/ ]; then
    ARCANUM_VERIF_CAPTURES=/mnt/project-files/captures
  else
    ARCANUM_VERIF_CAPTURES="$ETAT/captures"
  fi
fi
export DISPLAY="$ECRAN"

usage() {
  cat <<'AIDE'
Usage : scripts/verif-manuelle.sh <commande> [arguments]

  installer            installe Xvfb, xdotool, libxkbcommon-x11-0, ImageMagick (apt, si absents)
  demarrer             compile arcanum-ui, lance Xvfb puis le jeu, attend sa fenêtre
  charger NOM          relance le jeu et charge la partie préparée scripts/parties/NOM.json
                       (parties : charger sans NOM)
  cliquer X Y          clic gauche au point (X, Y) de l'écran
  survoler X Y         déplace la souris en (X, Y) sans cliquer
  molette N            N crans de molette (positif : zoom avant, négatif : arrière)
  touche TOUCHE...     appuie sur des touches xdotool (ex. End, Return, Escape, plus)
  attendre [S]         laisse S secondes au jeu pour se redessiner (0.5 par défaut)
  capture NOM [ZONE]   enregistre l'écran dans NOM.png ; ZONE = LxH+X+Y pour une partie seulement
  logs                 affiche le fichier de logs du lancement en cours
  arreter              ferme le jeu et Xvfb

Variables : ARCANUM_VERIF_ECRAN (:99), ARCANUM_VERIF_TAILLE (1600x900), ARCANUM_VERIF_CAPTURES (dossier),
ARCANUM_VERIF_LANGUE (fr : langue du jeu, passée en --lang), ARCANUM_CONFIG_DIR (préférences : dossier neuf
à chaque lancement s'il n'est pas posé).
AIDE
}

manquants() {
  local manque=()
  command -v Xvfb >/dev/null || manque+=(xvfb)
  command -v xdotool >/dev/null || manque+=(xdotool)
  command -v import >/dev/null || manque+=(imagemagick)
  # Sans elle, winit ne trouve pas le clavier sous X11 et le jeu s'arrête au lancement.
  ldconfig -p | grep -q libxkbcommon-x11.so || manque+=(libxkbcommon-x11-0)
  echo "${manque[@]}"
}

installer() {
  local manque
  manque="$(manquants)"
  [ -z "$manque" ] && return 0
  local sudo=""
  [ "$(id -u)" -ne 0 ] && sudo="sudo"
  # shellcheck disable=SC2086
  $sudo apt-get install -y $manque >/dev/null 2>&1 || {
    $sudo apt-get update >/dev/null
    # shellcheck disable=SC2086
    $sudo apt-get install -y $manque >/dev/null
  }
}

vivant() { [ -f "$ETAT/$1.pid" ] && kill -0 "$(cat "$ETAT/$1.pid")" 2>/dev/null; }

demarrer() {
  installer
  mkdir -p "$ETAT" "$ARCANUM_VERIF_CAPTURES"
  (cd "$RACINE" && cargo build -p arcanum-ui --quiet)
  if ! vivant xvfb; then
    Xvfb "$ECRAN" -screen 0 "${TAILLE}x24" >"$ETAT/xvfb.log" 2>&1 </dev/null &
    echo $! >"$ETAT/xvfb.pid"
    sleep 1
  fi
  vivant jeu && { echo "Le jeu tourne déjà (scripts/verif-manuelle.sh arreter pour le fermer)."; return 0; }
  rm -rf "$ETAT/logs"
  # Spec 080 : préférences dans un dossier neuf à chaque lancement, pour qu'une préférence de
  # développement (plein écran, langue) ne fausse pas les captures ; ARCANUM_CONFIG_DIR posé par
  # l'appelant est gardé (relancer avec les mêmes préférences).
  local config="${ARCANUM_CONFIG_DIR:-$ETAT/config}"
  [ -z "${ARCANUM_CONFIG_DIR:-}" ] && rm -rf "$config"
  # Logs à part pour ne pas mêler ceux du dépôt ; rendu OpenGL logiciel (pas de GPU sous Xvfb).
  # Toute la sous-commande est redirigée et `exec` remplace le sous-shell par le jeu : rien ne garde
  # la sortie du script ouverte (sinon `demarrer | tail` ne rend jamais la main) et le PID noté est
  # bien celui du jeu (sinon `arreter` ne tue que le sous-shell).
  (cd "$RACINE" && ARCANUM_LOG_DIR="$ETAT/logs" ARCANUM_CONFIG_DIR="$config" LIBGL_ALWAYS_SOFTWARE=1 \
    exec "$RACINE/target/debug/iter-arcanum" --lang "$LANGUE") >"$ETAT/jeu.log" 2>&1 </dev/null &
  echo $! >"$ETAT/jeu.pid"
  local fenetre=""
  for _ in $(seq 1 60); do
    fenetre="$(xdotool search --onlyvisible --name . 2>/dev/null | head -n 1 || true)"
    [ -n "$fenetre" ] && break
    vivant jeu || { echo "Le jeu s'est arrêté au lancement :" >&2; cat "$ETAT/jeu.log" >&2; exit 1; }
    sleep 0.5
  done
  [ -n "$fenetre" ] || { echo "Aucune fenêtre après 30 s." >&2; exit 1; }
  # Sans gestionnaire de fenêtres, on place et on agrandit nous-mêmes la fenêtre.
  xdotool windowmove "$fenetre" 0 0 windowsize "$fenetre" "${TAILLE%x*}" "${TAILLE#*x}"
  # Le focus échoue tant que la fenêtre n'est pas affichée : on réessaie.
  for _ in $(seq 1 20); do
    xdotool windowfocus --sync "$fenetre" 2>/dev/null && break
    sleep 0.5
  done
  sleep 2
  echo "Jeu lancé sur $ECRAN (${TAILLE}). Captures : $ARCANUM_VERIF_CAPTURES"
}

attendre() { sleep "${1:-0.5}"; }

# Partie préparée (issue privée 64) : un dossier de sauvegardes neuf qui ne contient qu'elle, puis Ctrl+O.
# Tab donne le focus clavier d'egui au premier bouton de la fenêtre, le « Charger » de la seule ligne,
# et Entrée le clique : pas de coordonnées, qui bougent avec la mise en page (issue privée 294).
charger() {
  local parties="$RACINE/scripts/parties"
  if [ -z "${1:-}" ] || [ ! -f "$parties/$1.json" ]; then
    [ -n "${1:-}" ] && echo "Partie préparée inconnue : $1" >&2
    echo "Parties préparées :" >&2
    for fichier in "$parties"/*.json; do echo "  $(basename "$fichier" .json)" >&2; done
    exit 2
  fi
  arreter
  local dossier="$ETAT/sauvegardes"
  rm -rf "$dossier"
  mkdir -p "$dossier"
  cp "$parties/$1.json" "$dossier/"
  # Exportée pour le seul lancement du jeu par `demarrer`.
  ARCANUM_SAVE_DIR="$dossier" demarrer
  xdotool key ctrl+o
  attendre 1
  xdotool key --delay 200 Tab Return
  attendre 1
  echo "Partie $1 chargée."
}

capture() {
  [ -n "${1:-}" ] || { echo "capture : nom manquant" >&2; exit 2; }
  mkdir -p "$ARCANUM_VERIF_CAPTURES"
  attendre
  if [ -n "${2:-}" ]; then
    import -window root -crop "$2" +repage "$ARCANUM_VERIF_CAPTURES/$1.png"
  else
    import -window root "$ARCANUM_VERIF_CAPTURES/$1.png"
  fi
  echo "$ARCANUM_VERIF_CAPTURES/$1.png"
}

arreter() {
  for nom in jeu xvfb; do
    if vivant "$nom"; then kill "$(cat "$ETAT/$nom.pid")" 2>/dev/null || true; fi
    rm -f "$ETAT/$nom.pid"
  done
}

commande="${1:-aide}"
shift || true
case "$commande" in
  installer) installer ;;
  demarrer) demarrer ;;
  charger) charger "${1:-}" ;;
  cliquer) xdotool mousemove "$1" "$2" sleep 0.2 click 1; attendre ;;
  survoler) xdotool mousemove "$1" "$2"; attendre ;;
  molette)
    n="$1"; bouton=4
    [ "$n" -lt 0 ] && { n=$(( -n )); bouton=5; }
    xdotool click --repeat "$n" --delay 50 "$bouton"; attendre ;;
  touche) xdotool key --delay 100 "$@"; attendre ;;
  attendre) attendre "${1:-}" ;;
  capture) capture "${1:-}" "${2:-}" ;;
  logs) cat "$ETAT"/logs/*.log ;;
  arreter) arreter ;;
  aide|-h|--help) usage ;;
  *) usage >&2; exit 2 ;;
esac
