#!/usr/bin/env bash
# Empaquette le jeu déjà construit en une archive de release (spec #33).
#
# Usage : scripts/empaqueter.sh <windows|linux|macos> <version> <dossier de sortie>
#   Lancé depuis n'importe où, après la construction :
#   - windows : target/x86_64-pc-windows-msvc/release/iter-arcanum.exe (sinon target/release/) ;
#   - linux   : target/release/iter-arcanum ;
#   - macos   : target/aarch64-apple-darwin/release/ et target/x86_64-apple-darwin/release/,
#               assemblés par lipo ; demande aussi iconutil, codesign et hdiutil (runner macOS).
#   Écrit iter-arcanum-<version>-<os>-<arch>.<zip|tar.gz|dmg> dans le dossier de sortie, puis vérifie
#   que l'archive contient les fichiers attendus (E4). Code de sortie : 0 si l'archive est complète.
#
# assets/ est pris dans `git ls-files` : seuls les fichiers suivis, déjà passés par le garde-fou
# du dépôt public (depot_public.rs). Rien de ../mage-knight/.

set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "usage : $0 <windows|linux|macos> <version> <dossier de sortie>" >&2
    exit 2
fi
os="$1"
version="$2"
racine="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$3"
sortie="$(cd "$3" && pwd)"
nom="iter-arcanum-$version"
travail="$(mktemp -d)"
trap 'rm -rf "$travail"' EXIT

# Fichiers que toute archive doit contenir (E4), relatifs au dossier du jeu ; l'exécutable est
# vérifié à part, son chemin change selon l'OS.
ATTENDUS=(assets/i18n/fr.json assets/LICENSE assets/sounds/LICENSES.md)
TEXTES=(LICENSE NOTICE README.md)

copier_assets() { # <dossier de destination des assets>
    local destination="$1"
    (cd "$racine" && git ls-files -z assets) | while IFS= read -r -d '' fichier; do
        mkdir -p "$destination/$(dirname "${fichier#assets/}")"
        cp "$racine/$fichier" "$destination/${fichier#assets/}"
    done
}

copier_textes() { # <dossier>
    for texte in "${TEXTES[@]}"; do cp "$racine/$texte" "$1/"; done
}

# Vérifie qu'une liste de chemins (stdin) contient chaque fichier attendu (préfixe des textes vide :
# à la racine).
verifier_liste() { # <préfixe des assets> <préfixe des textes> <exécutable>
    local liste manque=0
    liste="$(cat)"
    for attendu in "${ATTENDUS[@]}"; do
        grep -qxF "$1/${attendu#assets/}" <<<"$liste" || { echo "manque : $1/${attendu#assets/}" >&2; manque=1; }
    done
    for texte in "${TEXTES[@]}"; do
        grep -qxF "${2:+$2/}$texte" <<<"$liste" || { echo "manque : ${2:+$2/}$texte" >&2; manque=1; }
    done
    grep -qxF "$3" <<<"$liste" || { echo "manque : $3" >&2; manque=1; }
    return "$manque"
}

empaqueter_dossier() { # <exécutable construit> <nom de l'exécutable>
    local dossier="$travail/$nom"
    mkdir -p "$dossier"
    cp "$1" "$dossier/$2"
    copier_assets "$dossier/assets"
    copier_textes "$dossier"
}

case "$os" in
    windows)
        exe="$racine/target/x86_64-pc-windows-msvc/release/iter-arcanum.exe"
        [ -f "$exe" ] || exe="$racine/target/release/iter-arcanum.exe"
        archive="$sortie/$nom-windows-x86_64.zip"
        empaqueter_dossier "$exe" iter-arcanum.exe
        rm -f "$archive"
        (cd "$travail" && 7z a -tzip -bso0 "$archive" "$nom")
        # `7z l -slt` : une ligne « Path = … » par entrée, séparateur `\` sous Windows.
        7z l -slt "$archive" | sed -n 's/^Path = //p' | tr '\\' '/' |
            verifier_liste "$nom/assets" "$nom" "$nom/iter-arcanum.exe"
        ;;
    linux)
        archive="$sortie/$nom-linux-x86_64.tar.gz"
        empaqueter_dossier "$racine/target/release/iter-arcanum" iter-arcanum
        chmod +x "$travail/$nom/iter-arcanum"
        tar -C "$travail" -czf "$archive" "$nom"
        tar -tzf "$archive" | verifier_liste "$nom/assets" "$nom" "$nom/iter-arcanum"
        [ "$(tar -tvzf "$archive" "$nom/iter-arcanum" | cut -c4)" = x ] || {
            echo "iter-arcanum n'est pas exécutable dans l'archive" >&2
            exit 1
        }
        ;;
    macos)
        archive="$sortie/$nom-macos-universal.dmg"
        volume="$travail/volume"
        app="$volume/Iter Arcanum.app"
        mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
        # D7 : un seul exécutable pour Apple Silicon et Intel.
        lipo -create -output "$app/Contents/MacOS/iter-arcanum" \
            "$racine/target/aarch64-apple-darwin/release/iter-arcanum" \
            "$racine/target/x86_64-apple-darwin/release/iter-arcanum"
        copier_assets "$app/Contents/Resources/assets"
        # Info.plist n'admet que X.Y.Z : le suffixe -rc.N d'un tag de test ne s'y reporte pas.
        sed "s/@VERSION@/${version%%-*}/g" "$racine/packaging/macos/Info.plist" >"$app/Contents/Info.plist"
        # Icône produite ici, jamais commitée : PNG de l'iconset depuis le SVG, puis .icns.
        # Cible arm64 explicite : réutilise les dépendances déjà construites pour elle.
        (cd "$racine" && cargo run --quiet --release --target aarch64-apple-darwin -p arcanum-ui --example icone -- \
            --iconset "$travail/iter-arcanum.iconset")
        iconutil -c icns -o "$app/Contents/Resources/iter-arcanum.icns" "$travail/iter-arcanum.iconset"
        # D6 : signature ad hoc, sans quoi un binaire universel ne se lance pas sur Apple Silicon.
        codesign --force --deep -s - "$app"
        copier_textes "$volume"
        ln -s /Applications "$volume/Applications"
        (cd "$volume" && find . -type f -o -type l | sed 's|^\./||') |
            verifier_liste "Iter Arcanum.app/Contents/Resources/assets" "" "Iter Arcanum.app/Contents/MacOS/iter-arcanum" ||
            exit 1
        rm -f "$archive"
        # hdiutil échoue parfois sur les runners (« Resource busy ») : trois essais.
        for essai in 1 2 3; do
            hdiutil create -quiet -volname "Iter Arcanum $version" -srcfolder "$volume" -format UDZO "$archive" && break
            [ "$essai" -eq 3 ] && exit 1
            sleep 5
        done
        ;;
    *)
        echo "système inconnu : $os (windows, linux ou macos)" >&2
        exit 2
        ;;
esac

echo "archive : $archive"
