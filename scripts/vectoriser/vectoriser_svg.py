#!/usr/bin/env python3
"""Vectorisation SVG couleur avec prétraitement pour limiter les arrêts mémoire.

Installation :
    python3 -m pip install --upgrade pillow vtracer

Exemple :
    python3 scripts/vectoriser/vectoriser_svg.py image.png -o image.svg

Réglages qualité :
    python3 scripts/vectoriser/vectoriser_svg.py image.png -o image.svg --colors 96 --max-side 1200
"""
from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path

from PIL import Image


def prepare_image(
    source_path: Path,
    work_path: Path,
    *,
    colors: int,
    max_side: int,
    alpha_threshold: int,
) -> tuple[tuple[int, int], tuple[int, int], int]:
    """Nettoie l'alpha, recadre, redimensionne puis réduit la palette."""
    with Image.open(source_path) as im:
        rgba = im.convert("RGBA")

    original_size = rgba.size
    alpha = rgba.getchannel("A")
    mask = alpha.point(lambda value: 255 if value >= alpha_threshold else 0)

    bbox = mask.getbbox()
    if bbox is None:
        raise ValueError("L'image ne contient aucun pixel visible après nettoyage alpha.")

    rgba = rgba.crop(bbox)
    mask = mask.crop(bbox)

    # Redimensionner AVANT la quantification : un redimensionnement LANCZOS
    # après quantification recréerait des milliers de couleurs intermédiaires.
    if max_side > 0 and max(rgba.size) > max_side:
        factor = max_side / max(rgba.size)
        new_size = (
            max(1, round(rgba.width * factor)),
            max(1, round(rgba.height * factor)),
        )
        rgb = rgba.convert("RGB").resize(new_size, Image.Resampling.LANCZOS)
        mask = mask.resize(new_size, Image.Resampling.LANCZOS)
        mask = mask.point(lambda value: 255 if value >= 128 else 0)
    else:
        rgb = rgba.convert("RGB")

    # Les couleurs RGB des pixels transparents ne sont pas visibles ; on les
    # uniformise sans construire de liste Python de pixels.
    inverse_mask = Image.eval(mask, lambda value: 255 - value)
    rgb.paste((0, 0, 0), mask=inverse_mask)

    # Réduction de palette après redimensionnement, sans tramage (pas de bruit).
    if colors > 0:
        indexed = rgb.quantize(
            colors=colors,
            method=Image.Quantize.FASTOCTREE,
            dither=Image.Dither.NONE,
        )
        rgb = indexed.convert("RGB")

    rgba = Image.merge("RGBA", (*rgb.split(), mask))
    rgba.save(work_path, format="PNG", optimize=False)

    color_list = rgba.convert("RGB").getcolors(maxcolors=2_000_000)
    actual_colors = len(color_list) if color_list is not None else -1
    return original_size, rgba.size, actual_colors


def set_svg_size(svg_text: str, width: int, height: int) -> str:
    match = re.search(r"<svg\b[^>]*>", svg_text, flags=re.IGNORECASE | re.DOTALL)
    if not match:
        raise ValueError("Le SVG généré ne contient pas de balise <svg> valide.")
    tag = match.group(0)
    for attr, value in (("width", f'{width}px'), ("height", f'{height}px')):
        pattern = rf'\b{attr}\s*=\s*(["\']).*?\1'
        if re.search(pattern, tag, flags=re.IGNORECASE | re.DOTALL):
            tag = re.sub(pattern, f'{attr}="{value}"', tag, count=1, flags=re.IGNORECASE)
        else:
            tag = tag[:-1].rstrip() + f' {attr}="{value}">'
    return svg_text[:match.start()] + tag + svg_text[match.end():]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Vectorise une illustration en SVG en nettoyant l'alpha et en limitant les couleurs."
    )
    parser.add_argument("image", type=Path, help="Image source PNG/JPEG")
    parser.add_argument("-o", "--output", type=Path, help="Fichier SVG de sortie")
    parser.add_argument("--colors", type=int, default=64,
                        help="Nombre de couleurs de prétraitement (défaut : 64 ; 0 désactive)")
    parser.add_argument("--max-side", type=int, default=1000,
                        help="Taille maximale d'un côté en pixels (défaut : 1000 ; 0 désactive)")
    parser.add_argument("--alpha-threshold", type=int, default=128,
                        help="Seuil alpha 0-255 pour supprimer le fond transparent (défaut : 128)")
    parser.add_argument("--filter-speckle", type=int, default=4,
                        help="Supprime les petites régions (défaut : 4)")
    parser.add_argument("--color-precision", type=int, default=5, choices=range(1, 9))
    parser.add_argument("--layer-difference", type=int, default=16)
    parser.add_argument("--corner-threshold", type=int, default=60)
    parser.add_argument("--length-threshold", type=float, default=4.0)
    parser.add_argument("--max-iterations", type=int, default=10)
    parser.add_argument("--splice-threshold", type=int, default=45)
    parser.add_argument("--path-precision", type=int, default=3)
    parser.add_argument("--mode", choices=("spline", "polygon", "none"), default="spline")
    parser.add_argument("--hierarchical", choices=("stacked", "cutout"), default="stacked")
    args = parser.parse_args()

    if not args.image.is_file():
        print(f"Image introuvable : {args.image}", file=sys.stderr)
        return 2
    if not 0 <= args.alpha_threshold <= 255:
        parser.error("--alpha-threshold doit être compris entre 0 et 255")
    if args.colors != 0 and not 2 <= args.colors <= 256:
        parser.error("--colors doit être compris entre 2 et 256, ou égal à 0")

    try:
        import vtracer
    except ImportError:
        print("Dépendance manquante. Installe avec : python3 -m pip install --upgrade pillow vtracer", file=sys.stderr)
        return 2

    output = args.output or args.image.with_suffix(".svg")
    output.parent.mkdir(parents=True, exist_ok=True)

    try:
        with tempfile.TemporaryDirectory(prefix="vtracer_clean_") as tmp:
            tmpdir = Path(tmp)
            prepared = tmpdir / "image_pretraitee.png"
            temp_svg = tmpdir / "resultat.svg"

            print("Étape 1/2 : nettoyage alpha, recadrage et réduction de palette...", flush=True)
            original_size, processed_size, actual_colors = prepare_image(
                args.image, prepared,
                colors=args.colors,
                max_side=args.max_side,
                alpha_threshold=args.alpha_threshold,
            )
            print(
                f"Image : {original_size[0]}x{original_size[1]} -> "
                f"{processed_size[0]}x{processed_size[1]} px ; "
                f"environ {actual_colors} couleurs RGB.", flush=True
            )

            print("Étape 2/2 : vectorisation VTracer...", flush=True)
            vtracer.convert_image_to_svg_py(
                str(prepared), str(temp_svg),
                colormode="color",
                hierarchical=args.hierarchical,
                mode=args.mode,
                filter_speckle=args.filter_speckle,
                color_precision=args.color_precision,
                layer_difference=args.layer_difference,
                corner_threshold=args.corner_threshold,
                length_threshold=args.length_threshold,
                max_iterations=args.max_iterations,
                splice_threshold=args.splice_threshold,
                path_precision=args.path_precision,
            )

            if not temp_svg.exists() or temp_svg.stat().st_size == 0:
                raise RuntimeError("VTracer n'a pas produit de SVG.")

            svg = temp_svg.read_text(encoding="utf-8")
            # Le viewBox conserve les coordonnées réelles du fichier vectorisé.
            svg = set_svg_size(svg, processed_size[0], processed_size[1])
            output.write_text(svg, encoding="utf-8")

        print(f"Terminé : {output}")
        print(f"SVG : {output.stat().st_size / 1024:.1f} Ko")
        print(
            f"Paramètres : colors={args.colors}, max-side={args.max_side}, "
            f"alpha={args.alpha_threshold}, speckle={args.filter_speckle}, "
            f"mode={args.mode}, hierarchy={args.hierarchical}"
        )
        return 0
    except Exception as exc:
        print(f"Échec de la vectorisation : {type(exc).__name__}: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
