# Génère les quatre dragons (pile rouge, spec 075, fil D3) : un même corps, une palette et des détails par dragon.
# Cadre 120 x 160 du gabarit (assets/images/figure/README.md) ; un dragon remplit le cadre en largeur.
# Usage : python3 scripts/figurines/dragons.py
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images", "figure")
SW = 'stroke="#17120c" stroke-width="2.2" stroke-linejoin="round"'
IVORY = "#fbf6e6"


def grad(id_, c1, c2, c3):
    return (f'<linearGradient id="{id_}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{c1}"/>'
            f'<stop offset=".5" stop-color="{c2}"/><stop offset="1" stop-color="{c3}"/></linearGradient>')


def g(id_, *items):
    return f'  <g id="{id_}" {SW}>\n' + "".join(f"    {i}\n" for i in items if i) + "  </g>\n"


def mirror(d):
    """Symétrique d'un chemin absolu par rapport à x = 60 (nombres par paires x,y)."""
    # Les coordonnées sont écrites « x,y » : on retourne chaque x.
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


WING = "M44,72 C34,50 18,32 4,14 C10,38 8,56 2,74 C10,72 16,76 18,84 C24,80 30,82 32,90 C36,86 42,88 44,94 Z"
RIBS = "M42,74 L6,18 M42,76 L4,72 M42,78 L18,84 M42,80 L32,90"
THIGH = "M30,106 C20,114 22,134 34,138 L50,138 C54,126 50,112 42,104 Z"
FOOT = "M28,134 L54,134 L56,146 L22,146 C22,140 24,136 28,134 Z"
CLAWS = "M22,146 L18,146 L24,141 Z M32,146 L28,146 L34,140 Z M42,146 L38,146 L44,140 Z"
ARM = "M40,82 C32,88 30,98 34,106 L42,104 C40,98 42,92 46,88 Z"
HAND = "M30,105 C30,100 44,100 44,105 L44,108 C44,112 30,112 30,108 Z"
FCLAWS = "M31,110 L30,115 L34,110 Z M36,110 L36,115 L39,110 Z M41,110 L42,115 L44,110 Z"
HORN = "M44,20 C38,12 32,6 26,1 C30,8 34,14 40,26 Z"
FRILL = "M39,30 L26,30 L37,36 L26,42 L40,42 Z"
BROW = "M44,28 L56,34"


def dragon(name, scales, belly, wing, horn, eye, mouth, extra_defs="", head_extra="", wing_extra="",
           tail_tip="", crown=""):
    defs = [grad("sc", *scales), grad("be", *belly), grad("wi", *wing), extra_defs,
            '<radialGradient id="sh"><stop offset="0" stop-color="#000" stop-opacity=".55"/>'
            '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>']
    body = [f'<ellipse id="ombre" cx="60" cy="150" rx="54" ry="8" fill="url(#sh)"/>\n']
    body.append(g("ailes",
                  f'<path d="{WING}" fill="url(#wi)"/>', f'<path d="{mirror(WING)}" fill="url(#wi)"/>',
                  f'<path d="{RIBS} {mirror(RIBS)}" stroke-width="1.6" stroke-opacity=".7" fill="none"/>',
                  f'<path d="M4,14 L42,72 M116,14 L78,72" stroke="url(#sc)" stroke-width="3.4" fill="none"/>',
                  wing_extra))
    body.append(g("queue",
                  '<path d="M80,118 C96,130 106,138 110,128 C112,122 110,116 114,110 C118,116 120,124 118,132 '
                  'C114,146 98,148 84,138 Z" fill="url(#sc)"/>',
                  tail_tip or f'<path d="M112,112 L118,98 L118,116 Z" fill="{horn}"/>'))
    body.append(g("jambes",
                  f'<path d="{THIGH}" fill="url(#sc)"/>', f'<path d="{mirror(THIGH)}" fill="url(#sc)"/>',
                  f'<path d="{FOOT}" fill="url(#sc)"/>', f'<path d="{mirror(FOOT)}" fill="url(#sc)"/>',
                  f'<path d="{CLAWS} {mirror(CLAWS)}" fill="{IVORY}" stroke-width="1.2"/>'))
    body.append(g("torse",
                  '<path d="M38,74 C28,90 30,124 40,136 L80,136 C90,124 92,90 82,74 C74,64 46,64 38,74 Z" '
                  'fill="url(#sc)"/>',
                  '<path d="M48,78 C42,96 44,120 50,134 L70,134 C76,120 78,96 72,78 C66,74 54,74 48,78 Z" '
                  'fill="url(#be)"/>',
                  '<path d="M47,88 L73,88 M46,98 L74,98 M46,108 L74,108 M47,118 L73,118 M49,127 L71,127" '
                  'stroke-width="1.4" stroke-opacity=".55" fill="none"/>'))
    body.append(g("bras-gauche", f'<path d="{ARM}" fill="url(#sc)"/>', f'<path d="{HAND}" fill="url(#sc)" />',
                  f'<path d="{FCLAWS}" fill="{IVORY}" stroke-width="1.1"/>'))
    body.append(g("bras-droit", f'<path d="{mirror(ARM)}" fill="url(#sc)"/>',
                  f'<path d="{mirror(HAND)}" fill="url(#sc)"/>',
                  f'<path d="{mirror(FCLAWS)}" fill="{IVORY}" stroke-width="1.1"/>'))
    body.append(g("tete",
                  '<path d="M48,76 C46,62 48,52 52,44 L68,44 C72,52 74,62 72,76 Z" fill="url(#sc)"/>',
                  '<path d="M54,50 L66,50 M53,58 L67,58 M53,66 L67,66" stroke-width="1.4" stroke-opacity=".5" fill="none"/>',
                  crown or (f'<path d="{HORN}" fill="{horn}"/><path d="{mirror(HORN)}" fill="{horn}"/>'),
                  f'<path d="{FRILL}" fill="url(#wi)"/><path d="{mirror(FRILL)}" fill="url(#wi)"/>',
                  '<path d="M38,30 C38,16 48,10 60,10 C72,10 82,16 82,30 C82,38 78,42 74,46 L72,58 '
                  'C70,64 50,64 48,58 L46,46 C42,42 38,38 38,30 Z" fill="url(#sc)"/>',
                  f'<path d="M49,52 C54,55 66,55 71,52 L70,60 C64,64 56,64 50,60 Z" fill="{mouth}"/>',
                  f'<path d="M51,53 L53,58 L55,54 Z M65,54 L67,58 L69,53 Z M57,62 L58.5,58.5 L60,62.5 Z" '
                  f'fill="{IVORY}" stroke-width="1"/>',
                  '<path d="M55,47 L56.5,49 M65,47 L63.5,49" stroke-width="1.8" stroke-linecap="round" fill="none"/>',
                  f'<path d="{BROW} {mirror(BROW)}" stroke-width="3.4" stroke-linecap="round" fill="none"/>',
                  f'<path d="M45,34 C47,31 53,31 55,35 C52,38 48,38 45,34 Z" fill="{eye}" stroke-width="1"/>',
                  f'<path d="{mirror("M45,34 C47,31 53,31 55,35 C52,38 48,38 45,34 Z")}" fill="{eye}" stroke-width="1"/>',
                  '<path d="M50,32 L50,37 M70,32 L70,37" stroke-width="1.6" fill="none"/>',
                  '<path d="M46,16 C50,13 54,12 58,12" stroke="#fff" stroke-opacity=".45" stroke-width="1.4" fill="none"/>',
                  head_extra))
    svg = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 160" width="120" height="160">\n  <defs>\n    '
           + "\n    ".join(d for d in defs if d) + "\n  </defs>\n" + "".join(body) + "</svg>\n")
    with open(os.path.join(OUT, f"{name}.svg"), "w", encoding="utf-8", newline="\n") as f:
        f.write(svg)


def flames(grad_left, grad_right):
    """Deux flammes qui jaillissent des coins de la gueule et montent de part et d'autre de la tête."""
    left = "M50,56 C40,62 26,60 18,48 C24,50 28,48 28,40 C32,46 36,44 38,38 C40,46 44,50 50,52 Z"
    return (f'<path d="{left}" fill="url(#{grad_left})" stroke-width="1.6"/>'
            f'<path d="{mirror(left)}" fill="url(#{grad_right})" stroke-width="1.6"/>')


FIRE = ('<linearGradient id="fl" x1="1" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#fff3a0"/>'
        '<stop offset=".5" stop-color="#ffa630"/><stop offset="1" stop-color="#e0401a"/></linearGradient>')
COLD = ('<linearGradient id="co" x1="1" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#f2fdff"/>'
        '<stop offset=".5" stop-color="#7fd8ff"/><stop offset="1" stop-color="#2a78d8"/></linearGradient>')

# Dragon du marais : vert sombre, crête palmée, bave empoisonnée (Venimeux).
dragon("swamp_dragon", ("#7fa04a", "#4c6a2e", "#26391a"), ("#d8d68a", "#a9a85a", "#6f6e34"),
       ("#6f7a4a", "#4a5232", "#2a301c"), "#c9c08a", "#c6ff4a", "#3a1a12",
       head_extra=('<path d="M52,61 C53,68 54,74 51,78 C47,76 48,68 52,61 Z M68,61 C70,70 71,76 69,82 '
                   'C65,80 65,70 68,61 Z" fill="#9cff3a" stroke-width="1.4"/>'
                   '<path d="M52,12 L48,2 L56,10 L60,0 L64,10 L72,2 L68,12" fill="#6f7a4a"/>'))

# Dragon de feu : rouge, ventre doré, flammes aux naseaux et au bout de la queue.
dragon("fire_dragon", ("#e2553a", "#b02a1c", "#5e120c"), ("#ffd27a", "#e8a23a", "#a8621c"),
       ("#d0663a", "#9a3a1e", "#5a1a0e"), "#2b1a0c", "#ffd94a", "#ff8a2a", extra_defs=FIRE,
       head_extra=flames("fl", "fl"),
       tail_tip='<path d="M112,112 C108,104 112,98 116,92 C116,98 120,100 120,106 C120,112 116,116 112,112 Z" '
                'fill="url(#fl)" stroke-width="1.6"/>')

# Dragon de glace : bleu pâle, couronne de cristaux, glaçons aux ailes, souffle de givre.
ICE_CROWN = ('<path d="M44,22 L34,2 L48,16 Z M52,14 L50,0 L58,12 Z M76,22 L86,2 L72,16 Z M68,14 L70,0 L62,12 Z" '
             'fill="#e8faff"/>')
dragon("ice_dragon", ("#d8f0fb", "#7fb8d8", "#3a6a92"), ("#ffffff", "#d6eef8", "#9cc6dc"),
       ("#bfe6f6", "#7fb0cc", "#3e6c8c"), "#e8faff", "#7fe8ff", "#24466a", extra_defs=COLD,
       crown=ICE_CROWN, head_extra=flames("co", "co"),
       wing_extra=('<path d="M2,74 L4,82 L6,74 Z M18,84 L19,92 L22,84 Z M32,90 L33,98 L36,90 Z '
                   'M118,74 L116,82 L114,74 Z M102,84 L101,92 L98,84 Z M88,90 L87,98 L84,90 Z" fill="#e8faff" '
                   'stroke-width="1.2"/>'))

# Haut dragon : écailles d'or, ailes lie-de-vin, couronne d'ivoire, flamme froide (bleue) et brûlante (orange) : Feu glacial.
GOLD_CROWN = ('<path d="M44,22 C34,14 30,6 30,0 C36,6 42,10 50,14 Z M76,22 C86,14 90,6 90,0 C84,6 78,10 70,14 Z" '
              'fill="#fbf6e6"/><path d="M50,14 L46,2 L56,11 Z M70,14 L74,2 L64,11 Z" fill="#fbf6e6"/>')
dragon("high_dragon", ("#f2d27a", "#c4932e", "#6e4a14"), ("#fff6d8", "#efdca0", "#b8a060"),
       ("#9a3040", "#5e1828", "#2e0a14"), "#fbf6e6", "#ff5a3a", "#2b1a0c", extra_defs=FIRE + "\n    " + COLD,
       crown=GOLD_CROWN, head_extra=flames("co", "fl"),
       tail_tip='<path d="M112,112 L118,98 L118,116 Z" fill="#fbf6e6"/>')
