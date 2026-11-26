#!/usr/bin/env bash
# Sauvegardes témoins d'une version publiée (spec #52, issue #84 ; docs/release.md, « Garder les
# sauvegardes d'une version publiée »).
#
# Usage : scripts/temoins-release.sh vX.Y.Z [<révision>]
#   Dans une copie de travail du tag (git worktree, supprimée à la fin), joue les parcours de fin de
#   jalon et les sauvegarde, puis écrit dans crates/arcanum-persistance/tests/ :
#     temoins/vX.Y.Z/  les parties préparées du tag (scripts/parties/*.json) et ces sauvegardes ;
#     format/vX.Y.Z.yaml  l'empreinte du format au tag (copie de format/sauvegarde.yaml).
#   <révision> remplace le tag (essai avant le tag : un commit, pas des fichiers modifiés).
#   Refuse d'écraser des témoins existants : ils ne sont jamais régénérés.
#   Ne vaut que pour les tags où `episode` rend sa partie (v0.2.0 et suivants) ; ceux de v0.1.0 sont
#   faits.
#
# Parcours de fin de jalon joués : ceux de PARCOURS ci-dessous (fichier de crates/arcanum-core/tests/)
# et le test TEST_TEMOINS ajouté à sa copie. Au jalon suivant, changer les deux.

set -euo pipefail
racine="$(cd "$(dirname "$0")/.." && pwd)"
cd "$racine"

PARCOURS="parcours_j6.rs"
# Ajouté à la fin de la copie de $PARCOURS dans les tests d'arcanum-persistance : une sauvegarde par
# parcours, en fin de partie, datée comme les parties préparées.
TEST_TEMOINS='
#[test]
fn temoins_release() {
    let dir = std::path::PathBuf::from(std::env::var_os("ARCANUM_TEMOINS").expect("ARCANUM_TEMOINS"));
    let save = |stem: &str, runner: &arcanum_core::GameRunner| {
        let game = arcanum_persistance::SaveGame::from_runner(runner, "2026-01-01T00:00:00+00:00")
            .expect("partie commencée");
        arcanum_persistance::save(&dir, stem, &game).expect("sauvegarde écrite");
    };
    save("j6-conquete-en-solitaire", &parcours(SEED, true).p.runner);
    save("j6-conquete-des-cites", &episode(SEED).runner);
}
'

tag="${1:-}"
if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "usage : scripts/temoins-release.sh vX.Y.Z [<révision>]" >&2
    exit 1
fi
revision="${2:-$tag}"
persistance="crates/arcanum-persistance/tests"
temoins="$persistance/temoins/$tag"
empreinte="$persistance/format/$tag.yaml"
if [ -e "$temoins" ] || [ -e "$empreinte" ]; then
    echo "$temoins ou $empreinte existe déjà : les témoins ne sont jamais régénérés" >&2
    exit 1
fi
git rev-parse --verify --quiet "$revision^{commit}" >/dev/null || {
    echo "révision $revision introuvable (git fetch --tags ?)" >&2
    exit 1
}

copie="$(mktemp -d)"
sorties="$(mktemp -d)"
nettoyer() {
    git worktree remove --force "$copie" 2>/dev/null || rm -rf "$copie"
    rm -rf "$sorties"
}
trap nettoyer EXIT
git worktree add --quiet --detach "$copie" "$revision"

cp "$copie/crates/arcanum-core/tests/$PARCOURS" "$copie/$persistance/"
cp -r "$copie/crates/arcanum-core/tests/support" "$copie/$persistance/"
printf '%s' "$TEST_TEMOINS" >>"$copie/$persistance/$PARCOURS"
if [ ! -f "$copie/$persistance/format/sauvegarde.yaml" ]; then
    echo "$revision n'a pas d'empreinte $persistance/format/sauvegarde.yaml" >&2
    exit 1
fi

echo "Parcours de fin de jalon ($PARCOURS) joués sur $revision…"
# Le dossier target du dépôt : les dépendances externes déjà compilées servent.
(cd "$copie" && ARCANUM_TEMOINS="$sorties" CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$racine/target}" \
    cargo test -q -p arcanum-persistance --test "${PARCOURS%.rs}" temoins_release -- --exact)

mkdir -p "$temoins"
cp "$copie"/scripts/parties/*.json "$sorties"/*.json "$temoins/"
cp "$copie/$persistance/format/sauvegarde.yaml" "$empreinte"
echo "Écrits : $temoins/ ($(ls "$temoins" | wc -l) sauvegardes) et $empreinte"
echo "Reste à faire : docs/release.md, « Garder les sauvegardes d'une version publiée », étapes 2 et 3."
