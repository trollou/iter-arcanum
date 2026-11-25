#!/usr/bin/env python3
"""Illustrations générées des cartes et des unités (spec 072, spec #9, ADR 0009).

Outil de développement, lancé depuis un fil cloud : jamais par le jeu. Il lit la clé de l'API
dans le secret d'environnement GEMINI_API_KEY, qui n'est jamais versionné.

  essai <famille> <id>                                   prompt complet, sans appel ni coût
  generer <famille> [<id>...] [--candidats N] [--max M] [--differe]
                                                         candidats dans le dossier de sortie
                                                         (--max : plafond d'appels, reprises comprises ;
                                                         --differe : génération différée, à moitié prix)
  recuperer                                              écrit les candidats des générations différées finies
  planche <famille>                                      planche des candidats, à faire choisir
  retenir <famille> <id> <n>                             copie le candidat n dans assets/images/

Sans identifiant, generer prend chaque identifiant décrit dont l'image retenue manque ou n'est pas
au format de la famille (spec #9) : 404 × 168 en bande, 256 × 256 pour la Blessure.

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
BANNER = (404, 168)  # deux fois la bande de 202 × 84 points (spec #9)
SQUARE = (256, 256)  # la Blessure, affichée en grand (spec 077), garde son format carré (spec #9)
SQUARE_IDS = {"card-art": {"wound"}}
HERO_SIZE = (512, 512)  # portrait affiché en grand sur la fiche du héros (spec 078)
COLORS = 256  # PNG à palette : dix fois plus léger, sans écart visible en vignette (issue privée 167)
API_ROOT = "https://generativelanguage.googleapis.com/v1beta/"
API = API_ROOT + "models/{model}:generateContent"
BATCH_API = API_ROOT + "models/{model}:batchGenerateContent"
DEFERRED = "differes.json"  # générations différées en attente, dans le dossier de sortie
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


def size_for(family, id):
    """Format retenu d'une image : en bande, sauf la Blessure et les portraits (spec #9)."""
    if family == "heroes":
        return HERO_SIZE
    if id in SQUARE_IDS.get(family, ()):
        return SQUARE
    return BANNER


def is_square(family, id):
    width, height = size_for(family, id)
    return width == height


def prompt(family, id):
    """Préambule, variante de la famille (carrée pour la Blessure), puis « Subject: » et la
    description (spec 072) ; pour une carte de héros, sa description et la consigne de suivre le
    portrait de référence."""
    s = style()
    value = entry(family, id)
    variant = s["variants"]["square"] if family != "heroes" and is_square(family, id) else s["variants"][family]
    text = f"{s['preamble']}\n{variant}\nSubject: {value['description']}"
    if "hero" in value:
        hero = entry("heroes", value["hero"])["description"]
        text += f"\n{s['hero_reference']}\nMain character: {hero}"
    return text


def model_for(family, id):
    """Un seul modèle pour tout, cartes de héros comprises, avec leur portrait de référence (spec #9, Q1)."""
    return style()["model"]


def aspect_ratio(family, id):
    """Le générateur ne connaît pas 2,4:1 : 21:9 pour une bande, recadrée ensuite (spec #9)."""
    return "1:1" if is_square(family, id) else style()["aspect_ratio"]


def retained_path(family, id):
    folder = ASSETS / ("hero-art" if family == "heroes" else family)
    return folder / f"{id}.png"


def retained(family, id):
    """L'image retenue existe et a le format de la famille (spec #9)."""
    path = retained_path(family, id)
    if not path.exists():
        return False
    with Image.open(path) as image:
        return image.size == size_for(family, id)


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


def fit(image, size):
    """Cadre uni retiré, plus grande part centrée au rapport de size, puis réduit à size, opaque."""
    image = trim_border(image.convert("RGB"))
    width, height = size
    if image.width * height > image.height * width:
        kept = image.height * width // height
        left = (image.width - kept) // 2
        image = image.crop((left, 0, left + kept, image.height))
    else:
        kept = image.width * height // width
        top = (image.height - kept) // 2
        image = image.crop((0, top, image.width, top + kept))
    return image.resize(size, Image.LANCZOS)


def candidate_format(image, family, id):
    """Format d'un candidat : celui de l'image retenue, sans palette (spec #9)."""
    return fit(image, size_for(family, id))


def retained_format(image, family, id):
    """Format d'une image retenue, réduite à une palette de COLORS couleurs ; un portrait de héros
    garde toutes ses couleurs, vu en grand."""
    image = fit(image, size_for(family, id))
    return image if family == "heroes" else image.quantize(COLORS)


def write_candidate(family, id, data):
    path = next_candidate(family, id)
    path.parent.mkdir(parents=True, exist_ok=True)
    candidate_format(Image.open(io.BytesIO(data)), family, id).save(path)
    print(f"ok {shown(path)}")


def request_body(family, id):
    """Requête d'une image : portrait de référence éventuel, prompt, format demandé."""
    parts = [{"text": prompt(family, id)}]
    image = reference(family, id)
    if image:
        data = base64.b64encode(image.read_bytes()).decode()
        parts.insert(0, {"inlineData": {"mimeType": "image/png", "data": data}})
    return {
        "contents": [{"parts": parts}],
        "generationConfig": {
            "responseModalities": ["IMAGE"],
            "imageConfig": {"aspectRatio": aspect_ratio(family, id)},
        },
    }


def http(key, url, body=None, timeout=180):
    request = urllib.request.Request(
        url,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Content-Type": "application/json", "x-goog-api-key": key},
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.load(response)


def image_of(answer):
    """L'image d'une réponse generateContent, ou None."""
    for candidate in answer.get("candidates", []):
        for part in candidate.get("content", {}).get("parts", []):
            if "inlineData" in part:
                return base64.b64decode(part["inlineData"]["data"])
    return None


def call_api(key, family, id):
    data = image_of(http(key, API.format(model=model_for(family, id)), request_body(family, id)))
    if data is None:
        raise RuntimeError("réponse sans image")
    return data


def transient(error):
    """Erreur réseau ou 5xx : un nouvel essai a des chances de passer. Une 4xx (clé, requête refusée)
    échouerait pareil, et une réponse sans image a déjà été facturée : pas de reprise."""
    if isinstance(error, urllib.error.HTTPError):
        return error.code >= 500
    return isinstance(error, (urllib.error.URLError, TimeoutError, ConnectionError))


def cmd_essai(args):
    print(prompt(args.famille, args.id))


def deferred_path():
    return CANDIDATES / DEFERRED


def deferred():
    path = deferred_path()
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else []


def save_deferred(batches):
    deferred_path().parent.mkdir(parents=True, exist_ok=True)
    deferred_path().write_text(json.dumps(batches, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def api_key():
    key = os.environ.get("GEMINI_API_KEY")
    if not key:
        fail("GEMINI_API_KEY absente : aucun appel.")
    return key


def cmd_generer(args):
    key = api_key()
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
    if not jobs:
        print("rien à générer.")
        return
    price = s["usd_per_image_deferred"] if args.differe else s["usd_per_image"]
    mode = "en génération différée" if args.differe else "immédiats"
    print(f"{len(jobs)} appels {mode} au modèle {s['model']}, environ {len(jobs) * price:.2f} $.")
    if args.differe:
        send_deferred(key, args.famille, jobs, len(jobs) * price)
        return
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
                data = call_api(key, args.famille, id)
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
        write_candidate(args.famille, id, data)


def send_deferred(key, family, jobs, cost):
    """Envoie les appels en une génération différée (Batch API, spec #9) et la note en attente ;
    chaque requête porte la clé « <id>#<k> » qui la retrouve dans la réponse."""
    model = style()["model"]
    requests = [
        {"request": request_body(family, id), "metadata": {"key": f"{id}#{k}"}}
        for k, id in enumerate(jobs)
    ]
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M")
    body = {
        "batch": {
            "display_name": f"{family}-{stamp}",
            "input_config": {"requests": {"requests": requests}},
        }
    }
    answer = http(key, BATCH_API.format(model=model), body, timeout=600)
    name = answer["name"]
    batches = deferred()
    batches.append(
        {
            "name": name,
            "family": family,
            "model": model,
            "requests": len(jobs),
            "usd": round(cost, 2),
            "sent": datetime.datetime.now().isoformat(timespec="minutes"),
        }
    )
    save_deferred(batches)
    print(f"génération différée {name} envoyée, notée dans {shown(deferred_path())} : « recuperer » d'ici 24 h.")


FINISHED = {"BATCH_STATE_SUCCEEDED", "BATCH_STATE_FAILED", "BATCH_STATE_CANCELLED", "BATCH_STATE_EXPIRED"}


def batch_state(answer):
    return answer.get("metadata", {}).get("state") or answer.get("state", "inconnu")


def inlined_responses(answer):
    """Les réponses d'un lot terminé, là où l'API les met (« response » ou « metadata.output »)."""
    for holder in (answer.get("response", {}), answer.get("metadata", {}).get("output", {}), answer.get("output", {})):
        inlined = holder.get("inlinedResponses")
        if inlined:
            return inlined.get("inlinedResponses", inlined if isinstance(inlined, list) else [])
    return []


def collect(batch, answer):
    """Écrit les candidats d'un lot terminé ; une requête sans image est signalée, pas reprise (L4)."""
    family = batch["family"]
    written = 0
    for item in inlined_responses(answer):
        key = item.get("metadata", {}).get("key", "?")
        id = key.split("#")[0]
        data = image_of(item.get("response", {})) if "response" in item else None
        if data is None:
            print(f"échec {key} : {item.get('error', 'réponse sans image')}")
            continue
        write_candidate(family, id, data)
        written += 1
    print(f"{batch['name']} : {written} candidats sur {batch['requests']}.")


def cmd_recuperer(args):
    key = api_key()
    pending = []
    for batch in deferred():
        answer = http(key, API_ROOT + batch["name"])
        state = batch_state(answer)
        if state not in FINISHED:
            print(f"{batch['name']} ({batch['family']}, envoyé {batch['sent']}) : {state}")
            pending.append(batch)
            continue
        if state != "BATCH_STATE_SUCCEEDED":
            print(f"{batch['name']} ({batch['family']}) : {state}, relancer en appels immédiats (L3).")
            continue
        collect(batch, answer)
    save_deferred(pending)


def cmd_planche(args):
    """Candidats à la taille de la bande ×2 (404 × 168), identifiant et numéro au-dessus (spec #9) ;
    un candidat carré tient dans la même case."""
    folder = CANDIDATES / args.famille
    files = sorted(folder.glob("*.png")) if folder.exists() else []
    if not files:
        fail(f"aucun candidat dans {shown(folder)}")
    (cell_w, cell_h), label, gap, columns = BANNER, 22, 8, 4
    rows = (len(files) + columns - 1) // columns
    sheet = Image.new("RGB", (columns * (cell_w + gap) + gap, rows * (cell_h + label + gap) + gap), "white")
    draw = ImageDraw.Draw(sheet)
    for index, path in enumerate(files):
        x = gap + (index % columns) * (cell_w + gap)
        y = gap + (index // columns) * (cell_h + label + gap)
        draw.text((x + 2, y + 5), path.stem, fill="black")
        image = Image.open(path).convert("RGB")
        image.thumbnail((cell_w, cell_h), Image.LANCZOS)
        sheet.paste(image, (x + (cell_w - image.width) // 2, y + label))
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
    retained_format(Image.open(source), args.famille, args.id).save(target, optimize=True)
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
    generer.add_argument("--differe", action="store_true", help="génération différée, à moitié prix")
    generer.set_defaults(run=cmd_generer)

    recuperer = commands.add_parser("recuperer", help="écrit les candidats des générations différées finies")
    recuperer.set_defaults(run=cmd_recuperer)

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
