# Génère les trois golems (pile grise : Golems ; pile violette : Golems de glace et de feu ; spec 075, fil D4) :
# un même corps de roche, une matière et quelques détails par golem.
# Cadre 120 x 160 du gabarit (assets/images/figure/README.md) ; un golem remplit le cadre en largeur.
# Usage : python3 scripts/figurines/golems.py
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images", "figure")
SW = 'stroke="#17120c" stroke-width="2.2" stroke-linejoin="round"'


def grad(id_, c1, c2, c3):
    return (f'<linearGradient id="{id_}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{c1}"/>'
            f'<stop offset=".5" stop-color="{c2}"/><stop offset="1" stop-color="{c3}"/></linearGradient>')


def g(id_, *items):
    return f'  <g id="{id_}" {SW}>\n' + "".join(f"    {i}\n" for i in items if i) + "  </g>\n"


def mirror(d):
    """Symétrique d'un chemin absolu par rapport à x = 60 (coordonnées écrites « x,y »)."""
    res = []
    for t in d.split():
        cmd = ""
        if t[0].isalpha():
            cmd, t = t[0], t[1:]
        if "," in t:
            x, y = t.split(",")
            t = f"{120 - float(x):g},{y}"
        res.append(cmd + t)
    return " ".join(res)


def both(d):
    return f"{d} {mirror(d)}"


LEG = "M36,106 L32,128 L28,146 L56,146 L56,126 L54,106 Z"
TOE = "M36,140 L36,146 M46,140 L46,146"
TORSO = "M30,58 L44,48 L76,48 L90,58 L94,84 L86,112 L34,112 L26,84 Z"
TORSO_CRACKS = "M44,48 L50,62 L44,74 M76,48 L72,58 M34,112 L42,98 L40,90 M86,112 L80,100 M90,58 L82,70"
SHOULDER = "M32,56 L16,58 L8,74 L12,90 L28,92 L34,74 Z"
FIST = "M4,96 L12,88 L28,90 L32,104 L24,116 L8,116 L2,106 Z"
KNUCKLES = "M10,104 L20,104 M14,110 L24,110"
HEAD = "M46,34 L52,24 L68,24 L74,34 L74,50 L66,56 L54,56 L46,50 Z"
BROW = "M48,38 L58,42 L48,44 Z"


def golem(name, rock, dark, glow, eye, extra_defs="", back="", torso_extra="", arm_extra="", head_extra=""):
    defs = [grad("rk", *rock), grad("dk", *dark), extra_defs,
            '<radialGradient id="sh"><stop offset="0" stop-color="#000" stop-opacity=".55"/>'
            '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>']
    body = ['  <ellipse id="ombre" cx="60" cy="150" rx="52" ry="8" fill="url(#sh)"/>\n']
    if back:
        body.append(g("cape", back))
    body.append(g("jambes",
                  f'<path d="{LEG}" fill="url(#dk)"/>', f'<path d="{mirror(LEG)}" fill="url(#dk)"/>',
                  f'<path d="{both(TOE)}" stroke-width="1.4" stroke-opacity=".6" fill="none"/>'))
    body.append(g("torse",
                  f'<path d="{TORSO}" fill="url(#rk)"/>',
                  f'<path d="{TORSO_CRACKS}" stroke="{glow}" stroke-width="2" fill="none"/>',
                  f'<path d="M40,54 C46,51 54,50 60,50" stroke="#fff" stroke-opacity=".45" stroke-width="1.4" fill="none"/>',
                  torso_extra))
    for side, d in (("bras-gauche", lambda x: x), ("bras-droit", mirror)):
        body.append(g(side,
                      f'<path d="{d(SHOULDER)}" fill="url(#rk)"/>',
                      f'<path d="{d(FIST)}" fill="url(#dk)"/>',
                      f'<path d="{d(KNUCKLES)}" stroke-width="1.4" stroke-opacity=".6" fill="none"/>',
                      arm_extra if side == "bras-gauche" else mirror_items(arm_extra)))
    body.append(g("tete",
                  f'<path d="{HEAD}" fill="url(#rk)"/>',
                  f'<path d="{both(BROW)}" fill="#17120c" stroke-width="1.4"/>',
                  f'<path d="M50,44 L57,46 L50,47 Z {mirror("M50,44 L57,46 L50,47 Z")}" fill="{eye}" stroke="none"/>',
                  '<path d="M54,52 L66,52" stroke-width="2" fill="none"/>',
                  head_extra))
    svg = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 160" width="120" height="160">\n'
           '  <defs>\n' + "".join(f"    {d}\n" for d in defs if d) + '  </defs>\n' + "".join(body) + '</svg>\n')
    with open(os.path.join(OUT, f"{name}.svg"), "w", encoding="utf-8", newline="\n") as f:
        f.write(svg)


def mirror_items(item):
    """Reflète les attributs d="…" d'un fragment (détails du bras gauche reportés à droite)."""
    if not item:
        return ""
    out, rest = [], item
    while 'd="' in rest:
        before, after = rest.split('d="', 1)
        path, rest = after.split('"', 1)
        out.append(before + 'd="' + mirror(path) + '"')
    return "".join(out) + rest


# Golems : pierre grise, mousse, rune d'ambre gravée sur le poitrail.
golem("golems", ("#c4beb2", "#8f897d", "#5a554c"), ("#9a948a", "#6b665d", "#3e3a34"), "#3e3a34", "#ffd94a",
      torso_extra='<circle cx="60" cy="78" r="9" fill="none" stroke="#ffb02e" stroke-width="2.4"/>'
                  '<path d="M60,70 L60,86 M53,74 L67,82" stroke="#ffb02e" stroke-width="2" fill="none"/>',
      arm_extra='<path d="M16,58 C20,54 28,54 32,57 L26,60 Z" fill="#6f8f3a" stroke-width="1.4"/>',
      head_extra='<path d="M52,24 C56,20 64,20 68,24 L62,27 Z" fill="#6f8f3a" stroke-width="1.4"/>')

# Golems de glace : bloc de glace, cristaux sur les épaules et le crâne.
CRYSTAL = "M14,60 L10,40 L20,54 L22,36 L28,56 Z"
golem("ice_golems", ("#f2fbff", "#a9dcf2", "#5d9ec4"), ("#cdeefc", "#7fbadb", "#3f7aa3"), "#ffffff", "#2fb6d6",
      torso_extra='<path d="M48,66 L56,62 L52,76 Z M68,78 L74,72 L74,88 Z" fill="#ffffff" fill-opacity=".7" stroke-width="1.2"/>',
      arm_extra=f'<path d="{CRYSTAL}" fill="#e8fbff"/>',
      head_extra='<path d="M52,26 L50,10 L58,22 L60,4 L64,22 L70,10 L68,26 Z" fill="#e8fbff"/>')

# Golems de feu : basalte noir fendu de lave, flammes au crâne et aux épaules.
FLAME = "M12,60 C6,52 10,44 14,38 C14,46 18,48 20,42 C22,48 26,50 26,42 C32,50 30,58 26,60 Z"
golem("fire_golems", ("#6e5a50", "#463830", "#241c18"), ("#5a4a42", "#382c26", "#1a1412"), "#ff8a1e", "#ffd94a",
      extra_defs=grad("fl", "#fff2a0", "#ffb02e", "#e0401a"),
      torso_extra='<path d="M38,70 L50,78 L46,92 L58,100 M80,64 L70,80 L78,94" stroke="#ffd04a" stroke-width="2.4" fill="none"/>',
      arm_extra=f'<path d="{FLAME}" fill="url(#fl)" stroke-width="1.6"/>',
      head_extra='<path d="M48,26 C44,16 48,8 54,2 C54,10 58,12 60,6 C62,12 66,12 66,4 C74,12 76,20 72,26 Z" '
                 'fill="url(#fl)" stroke-width="1.6"/>')
