#!/usr/bin/env bash
# Lance fmt, clippy et test en n'affichant que l'essentiel (issue privée 57) :
# une ligne par étape réussie, et seulement l'utile pour une étape en échec.
# But : qu'un fil ne relise pas des centaines de lignes de sortie cargo.
#
# Usage : scripts/verifier.sh          fmt, clippy et test
#         scripts/verifier.sh --rouge  liste et compte les tests en échec, pour le
#                                      commit « Tests rouges : … » (issues privées 106, 120)
#         scripts/verifier.sh --rouge -p arcanum-ui [--test preferences]
#                                      idem, limité à une crate ou un fichier de test :
#                                      les arguments suivants vont à cargo test à la
#                                      place de --workspace (issue privée 359)
# Code de sortie : 0 si tout est vert, 1 sinon (avec --rouge : 0 si au moins un
# test échoue et que tout compile, 1 sinon).
# Sous Windows, les tests sont sautés : l'antivirus bloque leurs exécutables.

set -u
cd "$(dirname "$0")/.." || exit 1

# Spec 081 (Q9) : sous Linux, le son (cpal, ALSA) demande libasound2-dev et pkg-config pour
# compiler. Installés s'ils manquent, comme Xvfb par verif-manuelle.sh.
installer_alsa() {
    [ "$(uname -s)" = Linux ] || return 0
    command -v dpkg >/dev/null || return 0
    local manque=()
    dpkg -s libasound2-dev >/dev/null 2>&1 || manque+=(libasound2-dev)
    command -v pkg-config >/dev/null || manque+=(pkg-config)
    [ "${#manque[@]}" -eq 0 ] && return 0
    local sudo=""
    [ "$(id -u)" -ne 0 ] && sudo="sudo"
    echo "installation de ${manque[*]} (son, spec 081)"
    $sudo apt-get install -y "${manque[@]}" >/dev/null 2>&1 || {
        $sudo apt-get update >/dev/null
        $sudo apt-get install -y "${manque[@]}" >/dev/null
    }
}
installer_alsa

# --rouge : tous les tests en échec de tous les binaires (--no-fail-fast, sinon
# cargo s'arrête au premier fichier rouge et le compte est faux). Un échec de
# compilation n'est pas un rouge : il est signalé comme tel. Des arguments après
# --rouge (ex. -p arcanum-ui --test preferences) restreignent le passage, qui prend
# alors quelques secondes au lieu de minutes ; le push refait tout.
if [ "${1:-}" = "--rouge" ]; then
    shift
    if [ "$#" -eq 0 ]; then
        set -- --workspace
    fi
    sortie=$(mktemp)
    trap 'rm -f "$sortie"' EXIT
    RUST_BACKTRACE=0 cargo test "$@" --no-fail-fast --message-format=short >"$sortie" 2>&1
    if grep -q '^error: could not compile' "$sortie"; then
        echo "rouge : la compilation échoue, ce n'est pas un rouge (squelette todo!() d'abord) :"
        grep -E '^[^ ]+:[0-9]+:[0-9]+: error|^error' "$sortie" | head -n 30
        exit 1
    fi
    # Un argument erroné (crate ou fichier de test inconnu) n'est pas un rouge non plus.
    if ! grep -q '^running [0-9]' "$sortie"; then
        echo "rouge : cargo test n'a lancé aucun test ($*) :"
        grep -E '^error' "$sortie" | head -n 10
        exit 1
    fi
    echecs=$(grep -E '^test .* \.\.\. FAILED$' "$sortie" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort)
    if [ -z "$echecs" ]; then
        echo "rouge : aucun test en échec"
        exit 1
    fi
    nombre=$(printf '%s\n' "$echecs" | wc -l)
    echo "rouge : $nombre failed"
    printf '%s\n' "$echecs" | sed 's/^/  /'
    exit 0
fi

# Au-delà, la sortie est coupée : relancer la commande indiquée pour tout voir.
MAX_LIGNES=60
sortie=$(mktemp)
trap 'rm -f "$sortie"' EXIT
echec=0

# Affiche au plus MAX_LIGNES lignes de l'entrée, en signalant la coupe.
afficher() {
    local commande=$1 lignes
    lignes=$(cat)
    printf '%s\n' "$lignes" | head -n "$MAX_LIGNES"
    if [ "$(printf '%s\n' "$lignes" | wc -l)" -gt "$MAX_LIGNES" ]; then
        echo "  … sortie coupée à $MAX_LIGNES lignes, tout voir : $commande"
    fi
}

# fmt : seulement la liste des fichiers à reformater.
if cargo fmt --all --check -- -l >"$sortie" 2>&1; then
    echo "fmt : OK"
else
    echo "fmt : ÉCHEC, fichiers à reformater (cargo fmt --all) :"
    afficher "cargo fmt --all --check" <"$sortie"
    echec=1
fi

# clippy : messages au format court (fichier:ligne:colonne: message).
if cargo clippy --workspace --all-targets -q --message-format=short -- -D warnings >"$sortie" 2>&1; then
    echo "clippy : OK"
else
    echo "clippy : ÉCHEC"
    grep -vE '^error: could not compile|^warning: build failed|^error: aborting' "$sortie" \
        | afficher "cargo clippy --workspace --all-targets -- -D warnings"
    echec=1
fi

# test : le total si tout passe ; sinon les erreurs de compilation et les
# tests en échec de tous les binaires (--no-fail-fast, issue privée 78) (sans pile d'appels), sans les lignes « running N tests »,
# les points et les bilans verts.
if [ "${OS:-}" = "Windows_NT" ]; then
    echo "test : sauté sous Windows (antivirus), la CI fait foi"
elif RUST_BACKTRACE=0 cargo test --workspace --no-fail-fast -q --message-format=short >"$sortie" 2>&1; then
    total=$(awk '/^test result: ok\./ { n += $4 } END { print n + 0 }' "$sortie")
    echo "test : OK ($total tests)"
else
    echo "test : ÉCHEC"
    grep -vE '^running [0-9]+ tests?$|^[.i]+$|^test result: ok\.|^$|^error: test failed, to rerun' "$sortie" \
        | afficher "cargo test --workspace --no-fail-fast"
    echec=1
fi

exit "$echec"
