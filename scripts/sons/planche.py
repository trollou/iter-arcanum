#!/usr/bin/env python3
"""Planche d'écoute des sons (spec 081) : chercher des candidats sur OpenGameArt, relever leur
licence, construire une page où Mathieu écoute et choisit, puis convertir le son retenu.

Stdlib seulement, plus ffmpeg (encodeurs libmp3lame et libvorbis) ; les archives .7z demandent
py7zr (pip install py7zr) ou la commande 7z. Mode d'emploi : scripts/sons/README.md.

  planche.py chercher effet|musique <mots>...    pages OpenGameArt qui correspondent
  planche.py fiche <page>...                      titre, auteur, licence et fichiers d'une page
  planche.py construire <manifeste.json> <dossier>  data.json + planche.html, à publier en Artifact
  planche.py convertir <source> <cible.ogg> [--stereo] [--debut S] [--duree S]
"""

import argparse
import base64
import glob
import html
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request
import zipfile

OGA = "https://opengameart.org"
# Types d'art d'OpenGameArt : 13 = effet sonore, 12 = musique.
ART_TYPES = {"effet": 13, "musique": 12}
HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = os.environ.get("ARCANUM_SONS_CACHE", os.path.join(HERE, "..", "..", "target", "sons"))
AUDIO = (".ogg", ".wav", ".mp3", ".flac")


def get(url):
    request = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def page_url(page):
    """Accepte une URL complète ou le nom court d'une page OpenGameArt."""
    return page if page.startswith("http") else f"{OGA}/content/{page}"


def chercher(kind, words):
    for word in words:
        query = urllib.parse.urlencode(
            {"keys": word, "field_art_type_tid[]": ART_TYPES[kind], "sort_by": "count",
             "sort_order": "DESC", "items_per_page": 48})
        text = get(f"{OGA}/art-search-advanced?{query}").decode("utf-8", "replace")
        pages = [p for p in dict.fromkeys(re.findall(r'href="/content/([^"]+)"', text)) if p != "faq"]
        print(f"{word} : {' '.join(pages) or '(rien)'}")


def fiche(pages):
    for page in pages:
        text = get(page_url(page)).decode("utf-8", "replace")
        title = re.search(r'property="dc:title"><h2>(.*?)</h2>', text, re.S)
        author = re.search(r'field-name-author-submitter.*?<a href="/users/[^"]*"[^>]*>(.*?)</a>', text, re.S)
        licenses = re.findall(r"<div class=.license-name.>(.*?)</div>", text)
        files = [f for f in dict.fromkeys(re.findall(r'href="(https://opengameart.org/sites/default/files/[^"]+)"', text))
                 if not re.search(r"\.(png|jpe?g|gif)$", f, re.I)]
        print(f"== {page} | {html.unescape(title.group(1).strip()) if title else '?'} | "
              f"{author.group(1) if author else '?'} | {', '.join(licenses) or '?'}")
        for file in files:
            print(f"   {file}")


def extract(archive, target):
    if archive.endswith(".zip"):
        zipfile.ZipFile(archive).extractall(target)
    elif archive.endswith(".7z"):
        try:
            import py7zr
            py7zr.SevenZipFile(archive).extractall(target)
        except ImportError:
            subprocess.run(["7z", "x", "-y", f"-o{target}", archive], check=True, capture_output=True)


def fetch(source):
    """Chemin local d'un fichier de son ou d'une archive ; une URL est téléchargée dans le cache,
    une archive extraite à côté. Les fichiers téléchargés ne sont jamais exécutés."""
    if not source.startswith("http"):
        return source
    name = urllib.parse.unquote(source.rsplit("/", 1)[1])
    folder = os.path.join(CACHE, re.sub(r"[^A-Za-z0-9._-]", "_", source.split("/files/")[-1]))
    path = os.path.join(folder, name)
    if not os.path.exists(path):
        os.makedirs(folder, exist_ok=True)
        with open(path, "wb") as out:
            out.write(get(source))
        if path.endswith((".zip", ".7z")):
            extract(path, os.path.join(folder, "x"))
    return path


def resolve(candidate):
    """Les fichiers de son d'un candidat : `fichier` seul, ou `membres` (motif glob) dans l'archive
    ou le dossier de `fichier`."""
    path = fetch(candidate["fichier"])
    pattern = candidate.get("membres")
    if not pattern:
        return [path]
    base = os.path.join(os.path.dirname(path), "x") if path.endswith((".zip", ".7z")) else path
    found = sorted(f for f in glob.glob(os.path.join(base, "**", pattern), recursive=True)
                   if "/._" not in f and "__MACOSX" not in f)
    return found


def excerpt(path, start, duration, stereo, fade):
    """Extrait en mp3 (lu par tous les navigateurs), en base64."""
    filters = f"afade=t=out:st={max(duration - 1, 0)}:d=1" if fade else "anull"
    result = subprocess.run(
        ["ffmpeg", "-v", "error", "-ss", str(start), "-t", str(duration), "-i", path,
         "-ac", "2" if stereo else "1", "-b:a", "80k" if stereo else "48k", "-af", filters, "-f", "mp3", "-"],
        capture_output=True)
    if not result.stdout:
        sys.exit(f"extrait impossible : {path}\n{result.stderr.decode(errors='replace')}")
    return base64.b64encode(result.stdout).decode()


def length(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path],
                         capture_output=True, text=True).stdout.strip()
    return float(out or 0)


def construire(manifest_path, out_dir):
    manifest = json.load(open(manifest_path, encoding="utf-8"))
    needs = []
    for need in manifest["besoins"]:
        long_sound = need["groupe"] != "Effets"
        items = []
        for candidate in need["candidats"]:
            files = resolve(candidate)
            if not files:
                sys.exit(f"aucun fichier pour « {candidate['titre']} » ({need['cle']})")
            duration = candidate.get("duree", 30 if long_sound else 4)
            clips = [excerpt(f, candidate.get("debut", 0), duration, long_sound, need["groupe"] == "Musiques")
                     for f in files]
            items.append({"letter": chr(65 + len(items)), "title": candidate["titre"],
                          "author": candidate["auteur"], "url": page_url(candidate["page"]),
                          "lic": candidate["licence"], "clips": clips, "full": length(files[0])})
        needs.append({"grp": need["groupe"], "key": need["cle"], "label": need["libelle"], "items": items})
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "data.json"), "w", encoding="utf-8") as out:
        json.dump({"intro": manifest.get("intro", ""), "besoins": needs}, out)
    page = open(os.path.join(HERE, "planche.html"), encoding="utf-8").read()
    with open(os.path.join(out_dir, "planche.html"), "w", encoding="utf-8") as out:
        out.write(page.replace("__TITRE__", html.escape(manifest["titre"])))
    size = os.path.getsize(os.path.join(out_dir, "data.json")) / 1e6
    print(f"{out_dir}/planche.html et data.json ({size:.1f} Mo ; une page d'Artifact tient en 16 Mo)")


def convertir(source, target, stereo, start, duration):
    """Format de assets/sounds/ (spec 081) : Ogg Vorbis qualité 4, 44,1 kHz, sans métadonnées ;
    musiques et ambiances en stéréo, effets en mono."""
    if shutil.which("ffmpeg") is None:
        sys.exit("ffmpeg introuvable")
    command = ["ffmpeg", "-v", "error", "-y"]
    if start:
        command += ["-ss", str(start)]
    if duration:
        command += ["-t", str(duration)]
    command += ["-i", fetch(source), "-ac", "2" if stereo else "1", "-ar", "44100", "-c:a", "libvorbis",
                "-q:a", "4", "-map_metadata", "-1", target]
    subprocess.run(command, check=True)
    print(f"{target} : {os.path.getsize(target) // 1024} Ko ; ajouter sa ligne dans assets/sounds/LICENSES.md")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    search = sub.add_parser("chercher")
    search.add_argument("type", choices=ART_TYPES)
    search.add_argument("mots", nargs="+")
    info = sub.add_parser("fiche")
    info.add_argument("pages", nargs="+")
    build = sub.add_parser("construire")
    build.add_argument("manifeste")
    build.add_argument("dossier")
    convert = sub.add_parser("convertir")
    convert.add_argument("source")
    convert.add_argument("cible")
    convert.add_argument("--stereo", action="store_true")
    convert.add_argument("--debut", type=float)
    convert.add_argument("--duree", type=float)
    args = parser.parse_args()
    if args.command == "chercher":
        chercher(args.type, args.mots)
    elif args.command == "fiche":
        fiche(args.pages)
    elif args.command == "construire":
        construire(args.manifeste, args.dossier)
    else:
        convertir(args.source, args.cible, args.stereo, args.debut, args.duree)


if __name__ == "__main__":
    main()
