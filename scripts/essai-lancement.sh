#!/usr/bin/env bash
# Essai de lancement d'une archive de release (spec #33, E7 et E8).
#
# Usage : scripts/essai-lancement.sh <windows|linux|macos> <archive>
#   Dépaquette l'archive hors du dépôt (le .dmg est monté), lance le jeu sans cargo ni variable de
#   dossier des ressources, l'arrête au bout de 15 s, puis exige que son log dise « dossier des
#   ressources : » suivi d'un chemin dans l'archive dépaquetée, pas celui du dépôt (assets.rs).
#   Linux : il faut un écran (DISPLAY, Xvfb sur le runner). macOS : vérifie aussi les deux
#   architectures (lipo) et la signature (codesign) du paquet (E7).

set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage : $0 <windows|linux|macos> <archive>" >&2
    exit 2
fi
os="$1"
archive="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
essai="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/essai-lancement.XXXXXX")"
logs="$essai/logs"
mkdir -p "$logs"
volume=""
nettoyer() {
    [ -n "$volume" ] && hdiutil detach -quiet "$volume" || true
}
trap nettoyer EXIT

case "$os" in
    windows)
        (cd "$essai" && 7z x -bso0 "$archive")
        exe="$(echo "$essai"/iter-arcanum-*/iter-arcanum.exe)"
        attendu='iter-arcanum-[^/\\]*[/\\]assets$'
        ;;
    linux)
        tar -C "$essai" -xzf "$archive"
        exe="$(echo "$essai"/iter-arcanum-*/iter-arcanum)"
        attendu='/iter-arcanum-[^/]*/assets$'
        ;;
    macos)
        volume="$essai/volume"
        hdiutil attach -quiet -nobrowse -readonly -mountpoint "$volume" "$archive"
        app="$volume/Iter Arcanum.app"
        exe="$app/Contents/MacOS/iter-arcanum"
        archs="$(lipo -archs "$exe")"
        echo "architectures : $archs"
        for arch in x86_64 arm64; do
            [[ " $archs " == *" $arch "* ]] || { echo "architecture $arch absente" >&2; exit 1; }
        done
        codesign --verify --deep --strict "$app"
        echo "signature ad hoc vérifiée"
        attendu='/Iter Arcanum\.app/Contents/MacOS/\.\./Resources/assets$'
        ;;
    *)
        echo "système inconnu : $os (windows, linux ou macos)" >&2
        exit 2
        ;;
esac

[ -f "$exe" ] || { echo "exécutable introuvable : $exe" >&2; exit 1; }

# Comme un jeu déployé : ni cargo ni dossier des ressources imposé. Logs, configuration et
# sauvegardes dans le dossier d'essai, pour lire le log et ne rien laisser sur le runner.
# Le dossier des ressources est écrit dans le log avant l'ouverture de la fenêtre : un échec
# d'OpenGL sur le runner n'empêche pas l'essai.
(
    cd "$essai"
    env -u CARGO_MANIFEST_DIR -u ARCANUM_ASSETS_DIR \
        ARCANUM_LOG_DIR="$logs" ARCANUM_CONFIG_DIR="$essai/config" ARCANUM_SAVE_DIR="$essai/saves" \
        "$exe" >"$essai/sortie.txt" 2>&1 &
    pid=$!
    for _ in $(seq 15); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
)
[ "$os" = windows ] && { taskkill //F //IM iter-arcanum.exe >/dev/null 2>&1 || true; }

ligne="$(cat "$logs"/* 2>/dev/null | grep -m 1 'dossier des ressources : ' || true)"
if [ -z "$ligne" ]; then
    echo "aucune ligne « dossier des ressources » dans les logs ; sortie du jeu :" >&2
    cat "$essai/sortie.txt" >&2 || true
    ls -la "$logs" >&2 || true
    exit 1
fi
chemin="${ligne#*dossier des ressources : }"
chemin="${chemin%$'\r'}"
echo "dossier des ressources retenu : $chemin"
if ! grep -qE "$attendu" <<<"$chemin"; then
    echo "le jeu n'a pas pris les ressources de l'archive (attendu : $attendu)" >&2
    exit 1
fi
echo "essai de lancement réussi"
