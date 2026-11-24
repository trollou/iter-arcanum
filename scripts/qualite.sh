#!/usr/bin/env bash
# Rapport de qualité du code : taille des fichiers, longueur et complexité des
# fonctions, duplication, dépendances inutilisées, et couverture en option.
# Rapport seulement, jamais bloquant : il alimente le bilan de fin de spec
# (docs/qualite/README.md).
#
# Usage : scripts/qualite.sh                rapport, comparé à la référence
#         scripts/qualite.sh --couverture   ajoute la couverture (cargo llvm-cov,
#                                           4 à 5 min : fin de milestone)
#         scripts/qualite.sh --enregistrer  écrit aussi les mesures comme nouvelle
#                                           référence (docs/qualite/reference.txt)
# Les options se combinent. Un outil absent est sauté, avec sa commande d'installation.
# Code de sortie : 0, sauf erreur de compilation de clippy.

set -u
cd "$(dirname "$0")/.." || exit 1

couverture=0
enregistrer=0
for option in "$@"; do
    case "$option" in
        --couverture) couverture=1 ;;
        --enregistrer) enregistrer=1 ;;
        *) echo "option inconnue : $option" >&2; exit 2 ;;
    esac
done

REFERENCE=docs/qualite/reference.txt
# Seuils : ceux de scripts/qualite/clippy.toml pour les fonctions, celui-ci pour les fichiers.
SEUIL_FICHIER=2000
dossier=target/qualite
mkdir -p "$dossier"
mesures="$dossier/mesures.txt"
: >"$mesures"
details="$dossier/details.txt"
: >"$details"

mesure() { echo "$1 $2" >>"$mesures"; }
absent() { echo "  $1 absent, mesure sautée (installer : $2)"; }

# Code de production : les src/ des crates ; les tests/ sont mesurés à part.
fichiers_src=$(find crates/*/src -name '*.rs' | sort)
fichiers_tests=$(find crates/*/tests -name '*.rs' | sort)

# --- Fichiers -----------------------------------------------------------------
tailles=$(printf '%s\n' $fichiers_src | xargs wc -l | grep -v ' total$' | sort -rn)
mesure lignes_src "$(printf '%s\n' $fichiers_src | xargs cat | wc -l)"
mesure lignes_tests "$(printf '%s\n' $fichiers_tests | xargs cat | wc -l)"
mesure plus_gros_fichier "$(printf '%s\n' "$tailles" | head -n 1 | awk '{print $1}')"
mesure fichiers_trop_longs "$(printf '%s\n' "$tailles" | awk -v s=$SEUIL_FICHIER '$1 > s' | wc -l)"
{
    echo "## Fichiers de plus de $SEUIL_FICHIER lignes"
    printf '%s\n' "$tailles" | awk -v s=$SEUIL_FICHIER '$1 > s {print "  " $1 "  " $2}'
} >>"$details"

# --- Fonctions (lints clippy hors défaut, seuils dans scripts/qualite/clippy.toml) --
# CLIPPY_CONF_DIR à part : le clippy habituel (verifier.sh, CI) n'est pas touché.
sortie_clippy="$dossier/clippy.txt"
CLIPPY_CONF_DIR=scripts/qualite cargo clippy --workspace --message-format=short -- \
    -A clippy::all -W clippy::too_many_lines -W clippy::cognitive_complexity \
    >"$sortie_clippy" 2>&1
if grep -q '^error' "$sortie_clippy"; then
    echo "qualité : clippy ne compile pas, voir $sortie_clippy"
    exit 1
fi
# Un même avertissement peut sortir deux fois (lib et cible de test) : dédoublonner.
longues=$(grep 'too many lines' "$sortie_clippy" | sort -u |
    sed -E 's/^([^:]+:[0-9]+):[0-9]+: .*\(([0-9]+)\/[0-9]+\)$/\2 \1/' | sort -rn)
complexes=$(grep 'cognitive complexity' "$sortie_clippy" | sort -u |
    sed -E 's/^([^:]+:[0-9]+):[0-9]+: .*\(([0-9]+)\/[0-9]+\)$/\2 \1/' | sort -rn)
compter() { if [ -z "$1" ]; then echo 0; else printf '%s\n' "$1" | wc -l; fi; }
mesure fonctions_trop_longues "$(compter "$longues")"
mesure fonctions_trop_complexes "$(compter "$complexes")"
mesure plus_longue_fonction "$(printf '%s\n' "$longues" | head -n 1 | awk '{print $1+0}')"
mesure plus_complexe_fonction "$(printf '%s\n' "$complexes" | head -n 1 | awk '{print $1+0}')"
{
    echo "## Fonctions trop longues (lignes, emplacement)"
    printf '%s\n' "$longues" | sed 's/^/  /'
    echo "## Fonctions trop complexes (complexité cognitive, emplacement)"
    printf '%s\n' "$complexes" | sed 's/^/  /'
} >>"$details"

# --- Duplication (jscpd) --------------------------------------------------------
# jscpd liste chaque clone, puis un tableau dont la ligne « Total: » donne « lignes (pct%) » en premier.
dupliquer() {
    npx --no-install jscpd "$@" --format rust --min-tokens 70 --reporters console 2>/dev/null |
        sed 's/\x1b\[[0-9;]*m//g'
}
resume() { grep 'Total:' | grep -oE '[0-9]+ \([0-9.]+%\)' | head -n 1 | tr -d '(%)'; }
if npx --no-install jscpd --version >/dev/null 2>&1; then
    clones_src=$(dupliquer crates --ignore '**/tests/**')
    read -r lignes pct <<<"$(printf '%s\n' "$clones_src" | resume)"
    mesure duplication_src_lignes "${lignes:-0}"
    mesure duplication_src_pct "${pct:-0}"
    read -r lignes pct <<<"$(dupliquer crates/*/tests | resume)"
    mesure duplication_tests_lignes "${lignes:-0}"
    mesure duplication_tests_pct "${pct:-0}"
    {
        echo "## Clones du code de production (jscpd ; chemins relatifs à crates/)"
        printf '%s\n' "$clones_src" | grep -E '^ - |^   [^ ]'
    } >>"$details"
else
    absent jscpd "npm install -g jscpd"
fi

# --- Dépendances inutilisées (cargo machete) ------------------------------------
if command -v cargo-machete >/dev/null 2>&1; then
    inutiles=$(cargo machete 2>/dev/null | grep -E '^\s+[a-z0-9_-]+$' | wc -l)
    mesure dependances_inutilisees "$inutiles"
else
    absent "cargo machete" "cargo install --locked cargo-machete"
fi

# --- Couverture (option, cargo llvm-cov) ----------------------------------------
if [ $couverture = 1 ]; then
    if command -v cargo-llvm-cov >/dev/null 2>&1; then
        table="$dossier/couverture.txt"
        # Sans symboles de débogage (inutiles à la couverture) : compilation bien plus légère.
        if ! CARGO_PROFILE_DEV_DEBUG=0 cargo llvm-cov --workspace --summary-only \
            >"$table" 2>"$dossier/couverture.log"; then
            echo "  couverture en échec (disque plein ? test rouge ?), voir $dossier/couverture.log"
        fi
        # Colonnes : fichier, régions (total, manquées, %), fonctions (idem), lignes (idem)…
        awk '$1 ~ /^crates\// || $1 ~ /^arcanum-/ {
                 n = split($1, p, "/"); c = (p[1] == "crates") ? p[2] : p[1]
                 total[c] += $8; manquees[c] += $9
             }
             $1 == "TOTAL" { printf "couverture_totale %.1f\n", $10 + 0 }
             END { for (c in total) if (total[c] > 0)
                       printf "couverture_%s %.1f\n", c, 100 * (total[c] - manquees[c]) / total[c] }' \
            "$table" | sed 's/couverture_arcanum-/couverture_arcanum_/' | sort >>"$mesures"
        # La compilation instrumentée pèse plus d'1 Go : la libérer.
        cargo llvm-cov clean --workspace >/dev/null 2>&1
        rm -rf target/llvm-cov-target
    else
        absent "cargo llvm-cov" "cargo install --locked cargo-llvm-cov ; rustup component add llvm-tools-preview"
    fi
fi

# --- Rapport ----------------------------------------------------------------------
echo "qualité : mesures (référence : $REFERENCE)"
while read -r cle valeur; do
    avant=$(awk -v k="$cle" '$1 == k {print $2}' "$REFERENCE" 2>/dev/null)
    if [ -z "$avant" ]; then
        printf '  %-28s %10s\n' "$cle" "$valeur"
    else
        ecart=$(awk -v a="$avant" -v b="$valeur" 'BEGIN {
            d = b - a; if (d == 0) print "="; else if (d == int(d)) printf "%+d", d; else printf "%+.1f", d }')
        printf '  %-28s %10s   (%s, avant %s)\n' "$cle" "$valeur" "$ecart" "$avant"
    fi
done <"$mesures"
echo "détail (fichiers, fonctions, clones) : $details"

if [ $enregistrer = 1 ]; then
    {
        echo "# Référence de scripts/qualite.sh : $(git log -1 --format='%h %cs') ; une mesure par ligne."
        # Garder la couverture de la référence si elle n'a pas été remesurée.
        cat "$mesures"
        if [ $couverture = 0 ] && [ -f "$REFERENCE" ]; then grep '^couverture_' "$REFERENCE"; fi
    } >"$REFERENCE.tmp" && mv "$REFERENCE.tmp" "$REFERENCE"
    echo "référence mise à jour : $REFERENCE"
fi
exit 0
