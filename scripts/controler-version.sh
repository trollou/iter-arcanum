#!/usr/bin/env bash
# Contrôle de version d'une release (spec #33, E1) : le tag doit porter la version de Cargo.toml.
#
# Usage : scripts/controler-version.sh [<tag>]
#   v0.1.0 et v0.1.0-rc.2 exigent version = "0.1.0" (le suffixe -rc.N ne se reporte pas dans
#   Cargo.toml) ; un tag sur une version -dev est refusé. Sans tag (essai à blanc), rien n'est refusé.
#   Écrit la version à empaqueter sur la sortie standard (celle du tag, -rc.N compris, ou celle de
#   Cargo.toml) ; code de sortie 1 et message sur la sortie d'erreur en cas de refus.

set -euo pipefail
racine="$(cd "$(dirname "$0")/.." && pwd)"

# Première ligne `version = "…"` : celle de [workspace.package].
cargo="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$racine/Cargo.toml" | head -n 1)"
if [ -z "$cargo" ]; then
    echo "version introuvable dans Cargo.toml" >&2
    exit 1
fi

tag="${1:-}"
if [ -z "$tag" ]; then
    echo "$cargo"
    exit 0
fi

version="${tag#v}"
if [ "$version" = "$tag" ] || ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
    echo "tag $tag invalide : attendu vX.Y.Z ou vX.Y.Z-rc.N" >&2
    exit 1
fi
if [[ "$cargo" == *-dev ]]; then
    echo "tag $tag refusé : Cargo.toml porte la version de développement $cargo" >&2
    exit 1
fi
if [ "${version%-rc.*}" != "$cargo" ]; then
    echo "tag $tag refusé : sa version ${version%-rc.*} diffère de celle de Cargo.toml, $cargo" >&2
    exit 1
fi
echo "$version"
