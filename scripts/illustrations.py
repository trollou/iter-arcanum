#!/usr/bin/env python3
"""Illustrations générées des cartes et des unités (spec 072, ADR 0009).

Outil de développement, lancé depuis un fil cloud : jamais par le jeu. Il lit la clé de l'API
dans le secret d'environnement GEMINI_API_KEY, qui n'est jamais versionné.

  essai <famille> <id>                                   prompt complet, sans appel ni coût
  generer <famille> [<id>...] [--candidats N] [--max M]  candidats dans le dossier de sortie
                                                         (--max : plafond d'appels, reprises comprises)
  planche <famille>                                      planche des candidats, à faire choisir
  retenir <famille> <id> <n>                             copie le candidat n dans assets/images/

Dossier de sortie des candidats et des planches : --sortie DOSSIER (avant la commande), sinon la
variable ILLUSTRATIONS_SORTIE, sinon target/illustrations/, qui disparaît avec un conteneur cloud.
Depuis un fil cloud, viser un dossier qui lui survit (issue privée 171) :
  scripts/illustrations.py --sortie /mnt/project-files/illustrations generer unit-art

Familles : unit-art (units.json), card-art (cards.json), et heroes (heroes.json) : portraits des
héros, dans assets/images/hero-art/ (affichés sur l'écran de nouvelle partie, spec 078) et donnés au
générateur avec chaque carte qui porte un « hero », pour qu'il le dessine pareil partout.
"""

import argparse
import base64
import datetime
import io
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "scripts" / "illustrations"
ASSETS = ROOT / "assets" / "images"
CANDIDATES = ROOT / "target" / "illustrations"  # remplacé par --sortie (main)
FAMILIES = {"unit-art": "units.json", "card-art": "cards.json", "heroes": "heroes.json"}
SIZE = 256  # quatre fois la vignette de 64 px (issue privée 167)
HERO_SIZE = 512  # portrait affiché en grand sur la fiche du héros (spec 078)
COLORS = 256  # PNG à palette : dix fois plus léger, sans écart visible en vignette (issue privée 167)
API = "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
ATTEMPTS = 3  # essais par image, sur une erreur passagère seulement (issue privée 170)
WAIT = 10  # secondes avant la 1re reprise, doublées ensuite


def shown(path):
    """Chemin relatif au dépôt s'il y est, absolu sinon (dossier de sortie hors du dépôt)."""
    try:
        return path.relative_to(ROOT)
    except ValueError:
        return path


def fail(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def style():
    return json.loads((DATA / "style.json").read_text(encoding="utf-8"))


def descriptions_path(family):
    return DATA / FAMILIES[family]


def descriptions(family):
    return json.loads(descriptions_path(family).read_text(encoding="utf-8"))


def save_descriptions(family, entries):
    text = json.dumps(dict(sorted(entries.items())), ensure_ascii=False, indent=2)
    descriptions_path(family).write_text(text + "\n", encoding="utf-8")


def entry(family, id):
    entries = descriptions(family)
    if id not in entries or not entries[id].get("description"):
        fail(f"{FAMILIES[family]} : pas de description pour {id}")
    return entries[id]


def prompt(family, id):
    """Préambule, variante de la famille, puis « Subject: » et la description (spec 072) ; pour une
    carte de héros, sa description et la consigne de suivre le portrait de référence."""
    s = style()
    value = entry(family, id)
    text = f"{s['preamble']}\n{s['variants'][family]}\nSubject: {value['description']}"
    if "hero" in value:
        hero = entry("heroes", value["hero"])["description"]
        text += f"\n{s['hero_reference']}\nMain character: {hero}"
    return text


def model_for(family, id):
    """Modèle de la famille ; les portraits des héros et les cartes qui les montrent passent par le
    modèle plus puissant « hero_model », pour que le héros reste reconnaissable (Mathieu, 2026-11-06)."""
    s = style()
    if family == "heroes" or descriptions(family).get(id, {}).get("hero"):
        return s["hero_model"]
    return s["model"]


def retained_path(family, id):
    folder = ASSETS / ("hero-art" if family == "heroes" else family)
    return folder / f"{id}.png"


def retained(family, id):
    return retained_path(family, id).exists()


def reference(family, id):
    """Le portrait retenu du héros de la carte, s'il y en a un."""
    hero = descriptions(family).get(id, {}).get("hero")
    if not hero:
        return None
    path = retained_path("heroes", hero)
    if not path.exists():
        fail(f"portrait de référence absent : {shown(path)}")
    return path


def next_candidate(family, id):
    folder = CANDIDATES / family
    n = 1
    while (folder / f"{id}-{n}.png").exists():
        n += 1
    return folder / f"{id}-{n}.png"


def trim_border(image):
    """Retire le cadre uni, clair ou sombre, que le générateur ajoute parfois autour de la scène, et le
    filet qui le borde : la spec 072 veut une scène bord à bord."""
    gray = image.convert("L")
    width, height = gray.size

    def plain(values):
        values = list(values)
        return sum(v < 25 or v > 225 for v in values) >= 0.9 * len(values)

    def row(y):
        return (gray.getpixel((x, y)) for x in range(0, width, 4))

    def column(x):
        return (gray.getpixel((x, y)) for y in range(0, height, 4))

    limit = min(width, height) // 6
    top = next((y for y in range(limit) if not plain(row(y))), 0)
    bottom = next((y for y in range(limit) if not plain(row(height - 1 - y))), 0)
    left = next((x for x in range(limit) if not plain(column(x))), 0)
    right = next((x for x in range(limit) if not plain(column(width - 1 - x))), 0)
    if top == bottom == left == right == 0:
        return image
    inset = min(width, height) // 50
    return image.crop((left + inset, top + inset, width - right - inset, height - bottom - inset))


def square(image, size=SIZE):
    """Marge claire retirée, recadré au centre si besoin, puis réduit à size × size, opaque."""
    image = trim_border(image.convert("RGB"))
    side = min(image.size)
    left = (image.width - side) // 2
    top = (image.height - side) // 2
    image = image.crop((left, top, left + side, top + side))
    return image.resize((size, size), Image.LANCZOS)


def retained_format(image, family):
    """Format d'une image retenue : carrée comme un candidat, puis réduite à une palette de COLORS
    couleurs ; un portrait de héros garde HERO_SIZE et toutes ses couleurs, vu en grand."""
    if family == "heroes":
        return square(image, HERO_SIZE)
    return square(image).quantize(COLORS)


def call_api(key, model, s, text, image=None):
    parts = [{"text": text}]
    if image:
        data = base64.b64encode(image.read_bytes()).decode()
        parts.insert(0, {"inlineData": {"mimeType": "image/png", "data": data}})
    body = {
        "contents": [{"parts": parts}],
        "generationConfig": {
            "responseModalities": ["IMAGE"],
            "imageConfig": {"aspectRatio": s["aspect_ratio"]},
        },
    }
    request = urllib.request.Request(
        API.format(model=model),
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json", "x-goog-api-key": key},
    )
    with urllib.request.urlopen(request, timeout=180) as response:
        answer = json.load(response)
    for part in answer["candidates"][0]["content"]["parts"]:
        if "inlineData" in part:
            return base64.b64decode(part["inlineData"]["data"])
    raise RuntimeError("réponse sans image")


def transient(error):
    """Erreur réseau ou 5xx : un nouvel essai a des chances de passer. Une 4xx (clé, requête refusée)
    échouerait pareil, et une réponse sans image a déjà été facturée : pas de reprise."""
    if isinstance(error, urllib.error.HTTPError):
        return error.code >= 500
    return isinstance(error, (urllib.error.URLError, TimeoutError, ConnectionError))


def cmd_essai(args):
    print(prompt(args.famille, args.id))


def cmd_generer(args):
    key = os.environ.get("GEMINI_API_KEY")
    if not key:
        fail("GEMINI_API_KEY absente : aucun appel.")
    s = style()
    ids = args.ids or [
        id
        for id, value in sorted(descriptions(args.famille).items())
        if value.get("description") and not retained(args.famille, id)
    ]
    for id in ids:
        entry(args.famille, id)
    jobs = [id for id in ids for _ in range(args.candidats)]
    if len(jobs) > args.max:
        print(f"{len(jobs)} appels demandés, limités à {args.max} (--max).")
        jobs = jobs[: args.max]
    premium = sum(model_for(args.famille, id) != s["model"] for id in jobs)
    cost = (len(jobs) - premium) * s["usd_per_image"] + premium * s["hero_usd_per_image"]
    print(f"{len(jobs)} appels dont {premium} au modèle {s['hero_model']}, environ {cost:.2f} $.")
    (CANDIDATES / args.famille).mkdir(parents=True, exist_ok=True)
    calls = 0
    for id in jobs:
        data = None
        for attempt in range(1, ATTEMPTS + 1):
            if calls >= args.max:
                break
            if attempt > 1:
                wait = WAIT * 2 ** (attempt - 2)
                print(f"reprise {id} dans {wait} s (essai {attempt}/{ATTEMPTS})")
                time.sleep(wait)
            calls += 1
            try:
                data = call_api(
                    key, model_for(args.famille, id), s, prompt(args.famille, id), reference(args.famille, id)
                )
                break
            except Exception as error:  # Un échec n'arrête pas le lot.
                print(f"échec {id} : {error}")
                if not transient(error):
                    break
        if data is None:
            if calls >= args.max:
                print(f"plafond de {args.max} appels atteint (--max), reprises comprises : arrêt.")
                break
            continue
        path = next_candidate(args.famille, id)
        square(Image.open(io.BytesIO(data))).save(path)
        print(f"ok {shown(path)}")


def cmd_planche(args):
    folder = CANDIDATES / args.famille
    files = sorted(folder.glob("*.png")) if folder.exists() else []
    if not files:
        fail(f"aucun candidat dans {shown(folder)}")
    thumb, label, columns = 256, 24, 5
    rows = (len(files) + columns - 1) // columns
    sheet = Image.new("RGB", (columns * thumb, rows * (thumb + label)), "white")
    draw = ImageDraw.Draw(sheet)
    for index, path in enumerate(files):
        x = (index % columns) * thumb
        y = (index // columns) * (thumb + label)
        sheet.paste(Image.open(path).convert("RGB").resize((thumb, thumb)), (x, y))
        draw.text((x + 6, y + thumb + 5), path.stem, fill="black")
    out = CANDIDATES / f"planche-{args.famille}.png"
    sheet.save(out)
    print(out)


def cmd_retenir(args):
    source = CANDIDATES / args.famille / f"{args.id}-{args.n}.png"
    if not source.exists():
        fail(f"{shown(source)} n'existe pas")
    entries = descriptions(args.famille)
    entry(args.famille, args.id)
    target = retained_path(args.famille, args.id)
    target.parent.mkdir(parents=True, exist_ok=True)
    retained_format(Image.open(source), args.famille).save(target, optimize=True)
    entries[args.id]["model"] = model_for(args.famille, args.id)
    entries[args.id]["date"] = datetime.date.today().isoformat()
    save_descriptions(args.famille, entries)
    print(f"{target.relative_to(ROOT)} retenue")


def main():
    global CANDIDATES
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--sortie",
        type=Path,
        default=os.environ.get("ILLUSTRATIONS_SORTIE") or CANDIDATES,
        help="dossier des candidats et des planches (défaut : $ILLUSTRATIONS_SORTIE, sinon target/illustrations)",
    )
    commands = parser.add_subparsers(dest="commande", required=True)
    families = list(FAMILIES)

    essai = commands.add_parser("essai", help="affiche le prompt, sans appel")
    essai.add_argument("famille", choices=families)
    essai.add_argument("id")
    essai.set_defaults(run=cmd_essai)

    generer = commands.add_parser("generer", help="appelle le générateur")
    generer.add_argument("famille", choices=families)
    generer.add_argument("ids", nargs="*")
    generer.add_argument("--candidats", type=int, default=1)
    generer.add_argument("--max", type=int, default=20)
    generer.set_defaults(run=cmd_generer)

    planche = commands.add_parser("planche", help="monte les candidats en une planche")
    planche.add_argument("famille", choices=families)
    planche.set_defaults(run=cmd_planche)

    retenir = commands.add_parser("retenir", help="versionne un candidat")
    retenir.add_argument("famille", choices=families)
    retenir.add_argument("id")
    retenir.add_argument("n", type=int)
    retenir.set_defaults(run=cmd_retenir)

    args = parser.parse_args()
    CANDIDATES = Path(args.sortie).resolve()
    args.run(args)


if __name__ == "__main__":
    main()
