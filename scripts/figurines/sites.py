# Génère les sites debout (spec 075, D2). Cadre 200 x 200, sol y = 186, pied (100, 190).
import os
IMAGES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images")
import os, sys
OUT = os.path.join(IMAGES, "site", "")
INK = "#17120c"
SW = 'stroke="#17120c" stroke-width="2.2" stroke-linejoin="round"'

def grad(id_, c1, c2, c3=None, x2=1, y2=1):
    stops = f'<stop offset="0" stop-color="{c1}"/>'
    if c3:
        stops += f'<stop offset=".5" stop-color="{c2}"/><stop offset="1" stop-color="{c3}"/>'
    else:
        stops += f'<stop offset="1" stop-color="{c2}"/>'
    return f'<linearGradient id="{id_}" x1="0" y1="0" x2="{x2}" y2="{y2}">{stops}</linearGradient>'

SHADOW = '<radialGradient id="sh"><stop offset="0" stop-color="#000" stop-opacity=".55"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>'

def svg(defs, body, rx=80, shadow=True):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">\n'
            f'  <defs>\n    ' + "\n    ".join(defs + ([SHADOW] if shadow else [])) + '\n  </defs>\n'
            # Ombre plate, sous la base : plus bas, elle ferait léviter le bâti.
            + (f'  <ellipse id="ombre" cx="100" cy="184" rx="{rx}" ry="8" fill="url(#sh)"/>\n' if shadow else '')
            + body + '</svg>\n')

def g(id_, *items):
    return f'  <g id="{id_}" {SW}>\n' + "".join(f"    {i}\n" for i in items) + '  </g>\n'

# ---------- briques ----------
def house(x, y, w, h, dx, dy, wall, wall_side, roof, roof_side, door=True, win=True, timber=False):
    """Maison en trois quarts : façade à pignon, flanc droit qui fuit vers l'arrière."""
    rh = w * 0.95
    it = []
    it.append(f'<path d="M{x+w},{y} L{x+w+dx},{y-dy} L{x+w+dx},{y-dy-h} L{x+w},{y-h} Z" fill="{wall_side}"/>')
    it.append(f'<path d="M{x-w},{y} L{x+w},{y} L{x+w},{y-h} L{x},{y-h-rh} L{x-w},{y-h} Z" fill="{wall}"/>')
    # toit : pan droit qui part vers l'arrière, et bord du pignon
    it.append(f'<path d="M{x-2},{y-h-rh-3} L{x+w+4},{y-h+2} L{x+w+4+dx},{y-h+2-dy} L{x-2+dx},{y-h-rh-3-dy} Z" fill="{roof_side}"/>')
    it.append(f'<path d="M{x-w-4},{y-h+2} L{x-2},{y-h-rh-3} L{x+3},{y-h-rh-3} L{x-w+3},{y-h+3} Z" fill="{roof}"/>')
    if timber:
        it.append(f'<path d="M{x-w+1},{y-h} L{x+w-1},{y-h} M{x-w*0.5},{y-h} L{x-w*0.5},{y} M{x+w*0.5},{y-h} L{x+w*0.5},{y}" stroke-width="1.6" stroke-opacity=".7" fill="none"/>')
    if door:
        dw = w * 0.32
        it.append(f'<path d="M{x-dw},{y} L{x-dw},{y-h*0.5} C{x-dw},{y-h*0.66} {x+dw},{y-h*0.66} {x+dw},{y-h*0.5} L{x+dw},{y} Z" fill="#553419"/>')
    if win:
        it.append(f'<rect x="{x+w+dx*0.3}" y="{y-h*0.72-dy*0.3}" width="{dx*0.4}" height="{h*0.32}" fill="#ffd94a"/>')
        it.append(f'<rect x="{x-w*0.45-4}" y="{y-h-rh*0.55}" width="8" height="8" fill="#ffd94a" stroke-width="1.6"/>')
    return it

def tower_round(x, y, r, h, fill, roof, roof_h, cren=False, window=True):
    it = [f'<path d="M{x-r},{y} L{x-r},{y-h} L{x+r},{y-h} L{x+r},{y} C{x+r},{y+r*0.35} {x-r},{y+r*0.35} {x-r},{y} Z" fill="{fill}"/>']
    if roof:
        it.append(f'<path d="M{x-r-4},{y-h} L{x},{y-h-roof_h} L{x+r+4},{y-h} C{x+r+4},{y-h+r*0.4} {x-r-4},{y-h+r*0.4} {x-r-4},{y-h} Z" fill="{roof}"/>')
    if cren:
        n = 4; step = 2 * r / (2 * n - 1)
        d = "".join(f"M{x-r+i*2*step:.1f},{y-h} v-7 h{step:.1f} v7 " for i in range(n))
        it.append(f'<path d="{d}" fill="{fill}"/>')
    if window:
        it.append(f'<path d="M{x-3},{y-h*0.45} v-8 a3,3 0 0 1 6,0 v8 z" fill="#2b1a0c" stroke-width="1.4"/>')
    return it

def block(x, y, w, h, dx, dy, front, side, top, cren=True):
    """Bloc de pierre en trois quarts, créneaux en façade."""
    it = [f'<path d="M{x+w},{y} L{x+w+dx},{y-dy} L{x+w+dx},{y-dy-h} L{x+w},{y-h} Z" fill="{side}"/>',
          f'<path d="M{x-w},{y-h} L{x-w+dx},{y-h-dy} L{x+w+dx},{y-h-dy} L{x+w},{y-h} Z" fill="{top}"/>',
          f'<rect x="{x-w}" y="{y-h}" width="{2*w}" height="{h}" fill="{front}"/>']
    if cren:
        n = max(2, int(w / 6)); step = 2 * w / (2 * n - 1)
        d = "".join(f"M{x-w+i*2*step:.1f},{y-h} v-7 h{step:.1f} v7 Z " for i in range(n))
        it.append(f'<path d="{d}" fill="{front}"/>')
        it.append(f'<path d="M{x+w},{y-h} L{x+w+dx},{y-h-dy} L{x+w+dx},{y-h-dy-7} L{x+w},{y-h-7} Z" fill="{side}"/>')
    return it

def pennant(x, y, h, color):
    return [f'<path d="M{x},{y} L{x},{y-h}" stroke-width="2.6"/>',
            f'<path d="M{x},{y-h} L{x+20},{y-h+5} L{x+12},{y-h+8} L{x+20},{y-h+12} L{x},{y-h+14} Z" fill="{color}"/>']

def crystal(x, y, s, light, dark):
    return [f'<path d="M{x-7*s},{y} L{x-8*s},{y-18*s} L{x},{y-30*s} L{x},{y+2*s} Z" fill="{light}"/>',
            f'<path d="M{x},{y-30*s} L{x+8*s},{y-18*s} L{x+7*s},{y} L{x},{y+2*s} Z" fill="{dark}"/>',
            f'<path d="M{x-4*s},{y-17*s} L{x-2*s},{y-24*s}" stroke="#fff" stroke-opacity=".8" stroke-width="1.6" fill="none"/>']

def rocks(pts, fill):
    return [f'<path d="M{x-r},{y} C{x-r},{y-r*1.1} {x+r},{y-r*1.1} {x+r},{y} Z" fill="{fill}"/>' for x, y, r in pts]

STONE = grad("st", "#d8d2c6", "#a39a8c", "#6f665b")
STONE_SIDE = grad("sts", "#8c8476", "#5a534a", x2=1, y2=0)
WOOD = grad("wd", "#8a5a34", "#553419")

# ---------- sites ----------
def village():
    defs = [grad("wl", "#fbf1d8", "#d9c49a"), grad("wls", "#bfa77c", "#8f7650", x2=1, y2=0),
            grad("rf", "#e07a45", "#a5442a"), grad("rfs", "#9a3c24", "#6a2414", x2=1, y2=0),
            grad("th", "#e8c46a", "#b58a3a"), grad("ths", "#a77c32", "#6f5020", x2=1, y2=0), WOOD]
    back = house(64, 150, 22, 26, 22, 14, "url(#wl)", "url(#wls)", "url(#th)", "url(#ths)", timber=True)
    back2 = house(136, 154, 20, 24, 20, 12, "url(#wl)", "url(#wls)", "url(#rf)", "url(#rfs)")
    front = house(100, 176, 28, 34, 22, 14, "url(#wl)", "url(#wls)", "url(#rf)", "url(#rfs)", timber=True)
    # puits et clôture
    well = [f'<path d="M34,182 L34,166 L52,166 L52,182 C52,186 34,186 34,182 Z" fill="url(#st)"/>',
            '<ellipse cx="43" cy="166" rx="9" ry="3" fill="#2b1a0c"/>',
            '<path d="M35,166 L35,148 M51,166 L51,148" stroke-width="2.6"/>',
            '<path d="M31,150 L43,141 L55,150 Z" fill="url(#rf)"/>',
            '<path d="M150,184 L150,170 M160,180 L160,166 M170,176 L170,162 M146,176 L174,166" stroke-width="2.6" stroke="#553419"/>']
    # la cour : un sol de terre battue sur lequel tout repose
    yard = ['<path d="M14,180 C14,150 52,138 100,138 C150,138 188,150 188,180 C188,194 14,194 14,180 Z" fill="url(#yd)"/>',
            '<path d="M84,190 C90,184 104,184 112,190" stroke="#8a6a3a" stroke-width="5" stroke-linecap="round" fill="none" stroke-opacity=".6"/>',
            '<path d="M30,160 l-3,-6 M30,160 l1,-7 M30,160 l4,-6 M172,162 l-3,-6 M172,162 l1,-7 M172,162 l4,-6" stroke="#5a7a2a" stroke-width="2" fill="none" stroke-linecap="round"/>']
    smoke = ['<path d="M56,114 C50,106 62,102 56,94 C52,88 60,82 58,76" stroke="#fff" stroke-opacity=".55" stroke-width="3.2" fill="none" stroke-linecap="round"/>']
    body = g("cour", *yard) + g("arriere", *back, *back2) + g("devant", *front) + g("abords", *well) + g("fumee", *smoke)
    return svg(defs + [STONE, grad("yd", "#c8b078", "#a08850", "#7a6438")], body, rx=92)

def monastery():
    defs = [grad("wl", "#fbf6e6", "#ddd2b8"), grad("wls", "#c4b898", "#94876a", x2=1, y2=0),
            grad("rf", "#e09a4a", "#a8602a"), grad("rfs", "#9a5524", "#663414", x2=1, y2=0),
            grad("bell", "#f6dc8c", "#8a6524")]
    nave = house(84, 180, 36, 40, 34, 20, "url(#wl)", "url(#wls)", "url(#rf)", "url(#rfs)", door=True, win=False)
    rose = ['<circle cx="84" cy="122" r="7" fill="#7fb8e0" stroke-width="1.8"/>',
            '<path d="M84,115 V129 M77,122 H91" stroke-width="1.2" fill="none"/>',
            '<path d="M129,152 v-10 a4,4 0 0 1 8,0 v10 z M143,144 v-10 a4,4 0 0 1 8,0 v10 z" fill="#7fb8e0" stroke-width="1.6"/>']
    # clocher carré à gauche
    tower = block(40, 182, 14, 92, 10, 6, "url(#wl)", "url(#wls)", "#ddd2b8", cren=False)
    tower += [f'<path d="M24,90 L44,48 L64,90 Z" fill="url(#rf)"/>',
              f'<path d="M44,48 L64,90 L66,84 Z" fill="url(#rfs)"/>',
              '<path d="M34,120 v-14 a6,6 0 0 1 12,0 v14 z" fill="#2b1a0c"/>',
              '<path d="M35,118 C35,108 45,108 45,118 Z" fill="url(#bell)" stroke-width="1.6"/>',
              '<path d="M44,48 L44,38" stroke-width="2.6"/>',
              '<circle cx="44" cy="35" r="4.4" fill="url(#bell)" stroke-width="1.6"/>']
    return svg(defs, g("eglise", *nave, *rose) + g("clocher", *tower), shadow=False)

def keep():
    defs = [STONE, STONE_SIDE, grad("tp", "#e6e0d4", "#b9b1a3"), WOOD]
    walls = block(100, 184, 58, 40, 18, 12, "url(#st)", "url(#sts)", "url(#tp)")
    donjon = block(104, 150, 26, 76, 14, 9, "url(#st)", "url(#sts)", "url(#tp)")
    left = block(42, 186, 15, 62, 9, 6, "url(#st)", "url(#sts)", "url(#tp)")
    right = block(158, 186, 15, 62, 9, 6, "url(#st)", "url(#sts)", "url(#tp)")
    gate = ['<path d="M88,184 V164 C88,150 112,150 112,164 V184 Z" fill="#2b1a0c"/>',
            '<path d="M92,184 V166 M100,184 V158 M108,184 V166 M89,170 H111 M89,178 H111" stroke="#8a5a34" stroke-width="2" fill="none"/>',
            '<path d="M98,104 v-10 a3,3 0 0 1 6,0 v10 z M112,104 v-10 a3,3 0 0 1 6,0 v10 z" fill="#2b1a0c" stroke-width="1.4"/>',
            '<path d="M38,150 v-9 a4,4 0 0 1 8,0 v9 z M154,150 v-9 a4,4 0 0 1 8,0 v9 z" fill="#2b1a0c" stroke-width="1.4"/>',
            '<path d="M56,166 L70,164 M126,170 L142,168 M64,178 L76,177" stroke-width="1.4" stroke-opacity=".5" fill="none"/>']
    flag = pennant(110, 67, 34, "#5f6b78")
    return svg(defs, g("donjon", *donjon, *flag) + g("remparts", *walls, *gate) + g("tours", *left, *right), rx=88)

def mage_tower():
    defs = [grad("tw", "#e4dcf2", "#a093bd", "#5e5180", y2=0), grad("rf", "#9b6be0", "#3d2370"),
            '<radialGradient id="orb"><stop offset="0" stop-color="#fff"/><stop offset=".35" stop-color="#ffe37a"/><stop offset="1" stop-color="#ffe37a" stop-opacity="0"/></radialGradient>',
            STONE, STONE_SIDE]
    base = block(100, 186, 34, 16, 12, 8, "url(#st)", "url(#sts)", "#cfc6b6", cren=False)
    tower = [f'<path d="M80,174 L84,64 L116,64 L120,174 C120,182 80,182 80,174 Z" fill="url(#tw)"/>',
             '<path d="M78,66 L76,58 L124,58 L122,66 Z" fill="url(#tw)"/>',
             '<path d="M72,60 L100,4 L128,60 C128,68 72,68 72,60 Z" fill="url(#rf)"/>',
             '<path d="M100,4 L112,58" stroke-width="1.4" stroke-opacity=".5" fill="none"/>',
             '<path d="M94,104 v-12 a6,6 0 0 1 12,0 v12 z M94,146 v-12 a6,6 0 0 1 12,0 v12 z" fill="#ffd94a" stroke-width="1.8"/>',
             '<path d="M84,120 C95,116 105,116 116,120" stroke-width="1.4" stroke-opacity=".5" fill="none"/>',
             '<path d="M92,174 V160 a8,8 0 0 1 16,0 V174 Z" fill="#2b1a0c"/>']
    orb = ['<circle cx="100" cy="6" r="16" fill="url(#orb)" stroke="none"/>',
           '<path d="M70,40 l3,-6 l3,6 l-3,6 z M128,30 l3,-6 l3,6 l-3,6 z" fill="#ffe37a" stroke-width="1.2"/>']
    return svg(defs, g("socle", *base) + g("tour", *tower) + g("eclat", *orb), rx=60)

CITY = {
    "blue": ("#6fa8f0", "#2f5fae", "#203f78"),
    "green": ("#7fd08a", "#2f8a44", "#1d5a2c"),
    "red": ("#f08070", "#b8322e", "#7a1e1b"),
    "white": ("#ffffff", "#c9d2db", "#8a96a3"),
}

def city(color):
    light, mid, dark = CITY[color]
    defs = [STONE, STONE_SIDE, grad("tp", "#e6e0d4", "#b9b1a3"), grad("rf", light, mid, dark), grad("rfs", mid, dark, x2=1, y2=0),
            grad("gd", "#f6dc8c", "#8a6524")]
    # palais au fond
    palace = block(100, 160, 30, 90, 14, 9, "url(#st)", "url(#sts)", "url(#tp)", cren=False)
    palace += [f'<path d="M66,70 C66,40 134,40 134,70 Z" fill="url(#rf)"/>',
               '<path d="M100,42 L100,24" stroke-width="2.6"/><circle cx="100" cy="22" r="4" fill="url(#gd)"/>',
               '<path d="M88,104 v-14 a6,6 0 0 1 12,0 v14 z M106,104 v-14 a6,6 0 0 1 12,0 v14 z" fill="#ffd94a" stroke-width="1.6"/>']
    spires = tower_round(58, 158, 10, 94, "url(#st)", "url(#rf)", 30) + tower_round(146, 156, 10, 88, "url(#st)", "url(#rf)", 28)
    spires += pennant(58, 34, 22, mid) + pennant(146, 40, 22, mid)
    # ville basse : maisons serrées entre le palais et le rempart
    town = []
    for x, y, w in ((40, 156, 13), (80, 150, 12), (124, 150, 12), (164, 156, 13), (60, 158, 12), (142, 158, 12)):
        town += [f'<rect x="{x-w}" y="{y-14}" width="{2*w}" height="16" fill="#efe6d0"/>',
                 f'<path d="M{x-w-3},{y-14} L{x},{y-30} L{x+w+3},{y-14} Z" fill="url(#rf)"/>',
                 f'<rect x="{x-3}" y="{y-10}" width="6" height="7" fill="#ffd94a" stroke-width="1.4"/>']
    wall = block(100, 186, 66, 34, 16, 10, "url(#st)", "url(#sts)", "url(#tp)")
    gate = tower_round(76, 188, 12, 50, "url(#st)", None, 0, cren=True) + tower_round(124, 188, 12, 50, "url(#st)", None, 0, cren=True)
    gate += ['<path d="M88,186 V168 C88,154 112,154 112,168 V186 Z" fill="#2b1a0c"/>',
             f'<path d="M84,146 L116,146 L116,164 L100,172 L84,164 Z" fill="url(#rf)"/>',
             '<circle cx="100" cy="157" r="5" fill="url(#gd)" stroke-width="1.6"/>',
             '<path d="M44,170 L56,168 M140,172 L156,170" stroke-width="1.4" stroke-opacity=".5" fill="none"/>']
    corners = tower_round(32, 190, 11, 46, "url(#st)", "url(#rf)", 24) + tower_round(170, 186, 11, 46, "url(#st)", "url(#rf)", 24)
    return svg(defs, g("palais", *palace, *spires) + g("ville", *town) + g("remparts", *wall) + g("porte", *gate) + g("tours", *corners), rx=92)

MINE = {
    "blue": ("#d6e6ff", "#3f7fd6"),
    "green": ("#c8f5c0", "#2f9a48"),
    "red": ("#ffc0b0", "#c8322a"),
    "white": ("#ffffff", "#a8b4c0"),
}

def mine(color):
    light, dark = MINE[color]
    defs = [grad("rk", "#b0a08a", "#7a6a58", "#4f4338"), grad("cl", light, dark), grad("cd", dark, "#17120c"), WOOD,
            f'<radialGradient id="gl"><stop offset="0" stop-color="{light}" stop-opacity=".7"/><stop offset="1" stop-color="{light}" stop-opacity="0"/></radialGradient>']
    hill = ['<path d="M12,186 C14,156 30,130 54,124 C74,120 90,132 96,150 L100,186 Z" fill="url(#rk)"/>',
            '<path d="M120,186 C124,170 140,160 160,160 C176,160 188,172 190,186 Z" fill="url(#rk)"/>',
            '<path d="M28,152 C36,140 44,134 54,132" stroke-width="1.6" stroke-opacity=".5" fill="none"/>']
    entrance = ['<path d="M30,186 V162 C30,148 58,148 58,162 V186 Z" fill="#17120c"/>',
                '<path d="M30,186 V158 M58,186 V158 M25,158 H63" stroke="#553419" stroke-width="7" stroke-linecap="square"/>',
                '<path d="M30,186 V158 M58,186 V158 M25,158 H63" stroke="#8a5a34" stroke-width="3" fill="none"/>']
    glow = ['<ellipse cx="112" cy="110" rx="70" ry="78" fill="url(#gl)" stroke="none"/>']
    gems = (crystal(96, 172, 2.0, "url(#cl)", "url(#cd)") + crystal(140, 178, 2.0, "url(#cl)", "url(#cd)")
            + crystal(118, 168, 3.6, "url(#cl)", "url(#cd)") + crystal(80, 182, 1.2, "url(#cl)", "url(#cd)")
            + crystal(160, 186, 1.2, "url(#cl)", "url(#cd)"))
    cart = ['<path d="M38,170 L76,170 L72,186 L42,186 Z" fill="url(#wd)"/>',
            '<path d="M44,170 L50,160 L56,168 L62,158 L70,170 Z" fill="url(#cl)"/>',
            '<circle cx="48" cy="188" r="5" fill="#5f6b78"/><circle cx="68" cy="188" r="5" fill="#5f6b78"/>']
    pebbles = rocks([(186, 190, 5)], "url(#rk)")
    return svg(defs, g("colline", *hill) + g("entree", *entrance) + g("lueur", *glow) + g("cristaux", *gems) + g("wagonnet", *cart) + g("cailloux", *pebbles), rx=90)

def magical_glade():
    defs = ['<radialGradient id="pool"><stop offset="0" stop-color="#e8fff4"/><stop offset=".6" stop-color="#7fe0d0"/><stop offset="1" stop-color="#2f8a8a"/></radialGradient>',
            '<radialGradient id="halo"><stop offset="0" stop-color="#f4ffd0" stop-opacity=".75"/><stop offset="1" stop-color="#f4ffd0" stop-opacity="0"/></radialGradient>',
            grad("stn", "#d8d2e6", "#8f86a8"), grad("cap", "#ff9ad0", "#b8327a"), grad("lf", "#7fd08a", "#2f7a36")]
    halo = ['<ellipse cx="100" cy="150" rx="86" ry="54" fill="url(#halo)" stroke="none"/>']
    stone = ['<path d="M84,160 L86,88 C88,76 112,76 114,88 L116,160 Z" fill="url(#stn)"/>',
             '<path d="M100,98 L100,112 M93,105 L107,105 M95,128 C100,122 105,122 105,130 C105,138 95,138 95,130" stroke="#8be8ff" stroke-width="2.4" fill="none" stroke-linecap="round"/>']
    pool = ['<ellipse cx="100" cy="172" rx="44" ry="13" fill="url(#pool)"/>',
            '<path d="M76,172 C86,168 94,176 104,172 C112,168 118,174 124,172" stroke="#fff" stroke-opacity=".8" stroke-width="1.6" fill="none"/>']
    def shroom(x, y, s):
        return [f'<path d="M{x-3*s},{y} L{x-2*s},{y-10*s} L{x+2*s},{y-10*s} L{x+3*s},{y} Z" fill="#fbf6e6"/>',
                f'<path d="M{x-10*s},{y-9*s} C{x-10*s},{y-22*s} {x+10*s},{y-22*s} {x+10*s},{y-9*s} Z" fill="url(#cap)"/>',
                f'<circle cx="{x-3*s}" cy="{y-15*s}" r="{1.8*s}" fill="#fff" stroke="none"/>']
    def flower(x, y, c):
        return [f'<path d="M{x},{y} L{x},{y-12}" stroke="#2f7a36" stroke-width="2.6"/>',
                f'<circle cx="{x}" cy="{y-14}" r="5" fill="{c}" stroke-width="1.6"/>',
                f'<circle cx="{x}" cy="{y-14}" r="1.8" fill="#ffd94a" stroke="none"/>']
    plants = shroom(40, 172, 1.4) + shroom(56, 182, 1.0) + shroom(158, 176, 1.3) + flower(144, 186, "#a6e3f7") + flower(70, 190, "#f6e36b") + flower(132, 158, "#fff")
    plants += ['<path d="M24,186 C30,170 34,160 30,146 C40,158 40,172 36,186 Z M172,184 C170,168 174,156 182,146 C182,162 180,174 180,184 Z" fill="url(#lf)"/>']
    sparks = []
    for x, y, r in [(60, 104, 3), (140, 92, 3.4), (128, 128, 2.6), (70, 136, 2.4), (152, 120, 2.2), (48, 128, 2)]:
        sparks.append(f'<circle cx="{x}" cy="{y}" r="{r*3}" fill="url(#halo)" stroke="none"/><circle cx="{x}" cy="{y}" r="{r}" fill="#fffbd0" stroke="none"/>')
    return svg(defs, g("lueur", *halo) + g("pierre", *stone) + g("source", *pool) + g("plantes", *plants) + g("lucioles", *sparks), rx=84)

def dungeon():
    defs = [grad("rk", "#9a9088", "#625a54", "#3e3834"), STONE, STONE_SIDE, grad("bn", "#fbf6e6", "#c8bc9c")]
    rock = ['<path d="M22,186 C18,150 34,112 66,96 C96,82 132,86 156,108 C178,128 184,160 180,186 Z" fill="url(#rk)"/>',
            '<path d="M40,130 C48,118 58,110 70,106 M150,122 C158,132 164,144 166,156" stroke-width="1.6" stroke-opacity=".5" fill="none"/>']
    arch = ['<path d="M62,186 V138 C62,104 138,104 138,138 V186 Z" fill="url(#st)"/>',
            '<path d="M74,186 V142 C74,116 126,116 126,142 V186 Z" fill="#17120c"/>',
            '<path d="M62,138 L74,142 M138,138 L126,142 M70,118 L80,126 M130,118 L120,126 M100,108 L100,118" stroke-width="1.8" fill="none"/>',
            '<path d="M80,186 L84,176 L116,176 L120,186 Z M86,176 L89,168 L111,168 L114,176 Z M91,168 L93,162 L107,162 L109,168 Z" fill="url(#sts)"/>']
    skull = ['<path d="M88,104 C88,88 112,88 112,104 C112,110 108,112 106,114 L94,114 C92,112 88,110 88,104 Z" fill="url(#bn)"/>',
             '<circle cx="95" cy="102" r="3.4" fill="#17120c" stroke="none"/><circle cx="105" cy="102" r="3.4" fill="#17120c" stroke="none"/>',
             '<path d="M97,114 V110 M100,114 V110 M103,114 V110" stroke-width="1.4"/>',
             '<circle cx="92" cy="152" r="2.4" fill="#ffd94a" stroke="none"/><circle cx="104" cy="150" r="2.4" fill="#ffd94a" stroke="none"/>']
    torches = []
    for x in (50, 150):
        torches += [f'<path d="M{x},178 L{x},148" stroke="#553419" stroke-width="4"/>',
                    f'<path d="M{x-6},146 C{x-8},134 {x-2},128 {x},120 C{x+4},130 {x+8},134 {x+6},146 Z" fill="#ff9a2a"/>',
                    f'<path d="M{x-2},144 C{x-3},138 {x},134 {x},130 C{x+2},136 {x+3},140 {x+2},144 Z" fill="#ffe37a" stroke="none"/>']
    return svg(defs, g("rocher", *rock) + g("entree", *arch) + g("crane", *skull) + g("torches", *torches), rx=88)

def tomb():
    defs = [grad("mb", "#e4dcf0", "#a89cc0", "#6a5e88"), grad("mbs", "#857aa0", "#4f4568", x2=1, y2=0),
            grad("rf", "#6a5a90", "#2f2448"), grad("gs", "#b8b0a4", "#6f665b")]
    body = block(100, 172, 44, 54, 18, 11, "url(#mb)", "url(#mbs)", "#c8bedc", cren=False)
    body += ['<path d="M50,118 L100,84 L150,118 Z" fill="url(#rf)"/>',
             '<path d="M100,84 L150,118 L168,107 L118,73 Z" fill="url(#mbs)"/>',
             '<circle cx="100" cy="106" r="6" fill="#8be8a0" stroke-width="1.6"/>']
    cols = []
    for x in (64, 82, 118, 136):
        cols.append(f'<rect x="{x-5}" y="120" width="10" height="50" fill="url(#mb)"/>')
    cols += ['<path d="M88,172 V136 C88,126 112,126 112,136 V172 Z" fill="#17120c"/>',
             '<path d="M50,172 L150,172 L156,186 L44,186 Z" fill="url(#gs)"/>']
    stones = ['<path d="M18,186 V164 C18,154 34,154 34,164 V186 Z" fill="url(#gs)"/>',
              '<path d="M170,188 L168,168 C168,160 182,158 184,166 L186,186 Z" fill="url(#gs)"/>',
              '<path d="M22,168 H30 M22,174 H30" stroke-width="1.6" stroke-opacity=".7" fill="none"/>',
              '<path d="M36,146 C40,140 44,140 46,146 M156,140 C160,134 164,134 166,140" stroke="#8be8a0" stroke-width="2" fill="none" stroke-opacity=".8"/>']
    return svg(defs, g("mausolee", *body) + g("colonnes", *cols) + g("tombes", *stones), rx=92)

def monster_den():
    defs = [grad("rk", "#a8987e", "#6f604c", "#463a2c"), grad("bn", "#fbf6e6", "#c8bc9c"),
            grad("lf", "#6a8a3a", "#34501c")]
    hill = ['<path d="M16,186 C16,140 44,104 90,96 C140,90 182,124 186,186 Z" fill="url(#rk)"/>',
            '<path d="M36,186 C30,170 34,152 40,140 C46,160 50,174 52,186 Z M160,186 C166,170 166,152 160,138 C152,158 150,174 150,186 Z" fill="url(#lf)"/>',
            '<path d="M124,112 L132,128 M132,110 L140,126 M140,112 L146,124" stroke-width="2.4" stroke-opacity=".7" fill="none"/>']
    cave = ['<path d="M56,186 C52,152 70,124 100,124 C130,124 148,152 144,186 Z" fill="#17120c"/>',
            '<path d="M60,140 L66,152 L72,138 L78,148 L84,134 M116,134 L122,148 L128,138 L134,152 L140,140" fill="url(#bn)" stroke-width="1.6"/>',
            '<path d="M82,158 L90,160 L82,164 Z M118,158 L110,160 L118,164 Z" fill="#ffd94a" stroke="none"/>']
    bones = ['<path d="M24,184 L54,176" stroke="#fbf6e6" stroke-width="4.4" stroke-linecap="round"/>',
             '<circle cx="24" cy="184" r="4" fill="url(#bn)" stroke-width="1.6"/><circle cx="54" cy="176" r="4" fill="url(#bn)" stroke-width="1.6"/>',
             '<path d="M146,186 C146,174 168,174 168,186 Z" fill="url(#bn)"/>',
             '<circle cx="153" cy="181" r="2.2" fill="#17120c" stroke="none"/><circle cx="161" cy="181" r="2.2" fill="#17120c" stroke="none"/>']
    return svg(defs, g("colline", *hill) + g("antre", *cave) + g("os", *bones), rx=90)

def spawning_grounds():
    defs = [grad("mud", "#7a5a34", "#4a3018", "#2b1a0c"), grad("egg", "#f4f0c8", "#a8b060", "#5a6a2a"),
            grad("slm", "#c8e070", "#5a8a2a"),
            '<radialGradient id="gl"><stop offset="0" stop-color="#c8e070" stop-opacity=".6"/><stop offset="1" stop-color="#c8e070" stop-opacity="0"/></radialGradient>']
    nest = ['<ellipse cx="100" cy="150" rx="88" ry="50" fill="url(#gl)" stroke="none"/>',
            '<path d="M14,186 C14,160 40,148 100,146 C160,148 186,160 186,186 Z" fill="url(#mud)"/>',
            '<path d="M24,170 C40,162 52,166 64,160 M136,160 C150,164 164,162 176,170" stroke-width="1.6" stroke-opacity=".5" fill="none"/>']
    def egg(x, y, rx, ry, cracked=False):
        it = [f'<path d="M{x-rx},{y} C{x-rx},{y-ry*1.6} {x-rx*0.5},{y-ry*2} {x},{y-ry*2} C{x+rx*0.5},{y-ry*2} {x+rx},{y-ry*1.6} {x+rx},{y} C{x+rx},{y+ry*0.3} {x-rx},{y+ry*0.3} {x-rx},{y} Z" fill="url(#egg)"/>',
              f'<path d="M{x-rx*0.5},{y-ry*1.4} C{x-rx*0.4},{y-ry*1.7} {x-rx*0.1},{y-ry*1.85} {x+rx*0.1},{y-ry*1.85}" stroke="#fff" stroke-opacity=".8" stroke-width="2" fill="none"/>']
        if cracked:
            it.append(f'<path d="M{x-rx*0.8},{y-ry*1.1} L{x-rx*0.4},{y-ry*0.8} L{x-rx*0.1},{y-ry*1.2} L{x+rx*0.3},{y-ry*0.8} L{x+rx*0.8},{y-ry*1.1}" stroke-width="2" fill="none"/>')
            it.append(f'<circle cx="{x-rx*0.15}" cy="{y-ry*0.95}" r="2.4" fill="#ffd94a" stroke="none"/><circle cx="{x+rx*0.2}" cy="{y-ry*0.95}" r="2.4" fill="#ffd94a" stroke="none"/>')
        return it
    eggs = egg(70, 166, 20, 24) + egg(130, 164, 18, 22, cracked=True) + egg(100, 182, 26, 34) + egg(46, 184, 12, 14) + egg(158, 186, 13, 15)
    slime = ['<path d="M78,186 C80,178 86,180 88,186 Z M112,186 C114,176 122,178 122,186 Z" fill="url(#slm)"/>',
             '<path d="M30,150 C26,132 36,120 32,104 M170,152 C176,134 166,124 172,108" stroke="#4a3018" stroke-width="4.4" fill="none" stroke-linecap="round"/>',
             '<path d="M30,150 C26,132 36,120 32,104 M170,152 C176,134 166,124 172,108" stroke="#8a6a3a" stroke-width="2" fill="none" stroke-linecap="round"/>']
    return svg(defs, g("nid", *nest) + g("oeufs", *eggs) + g("bave", *slime), rx=92)

def ruins():
    defs = [grad("st", "#ece4d0", "#b8ab8c", "#7a6e58"), grad("sts", "#8f846c", "#5a5040", x2=1, y2=0),
            grad("mo", "#7fb050", "#3f6a28")]
    base = ['<path d="M24,186 L30,172 L170,172 L178,186 Z" fill="url(#sts)"/>',
            '<path d="M30,172 L40,164 L164,164 L170,172 Z" fill="url(#st)"/>']
    arch = ['<path d="M56,164 V96 L72,96 V164 Z M128,164 V96 L144,96 V164 Z" fill="url(#st)"/>',
            '<path d="M52,98 C52,62 148,62 148,98 L132,98 C132,78 68,78 68,98 Z" fill="url(#st)"/>',
            '<path d="M118,72 L126,62 L134,70" fill="#22242a" stroke="none"/>',
            '<path d="M60,120 L68,122 M132,140 L140,142 M62,146 L70,144" stroke-width="1.4" stroke-opacity=".5" fill="none"/>']
    broken = ['<path d="M150,164 V124 L156,118 L162,126 L166,120 V164 Z" fill="url(#st)"/>',
              '<path d="M30,164 V136 L36,130 L42,138 V164 Z" fill="url(#st)"/>',
              '<path d="M84,186 C84,176 108,174 116,182 L116,188 Z" fill="url(#st)"/>',
              '<path d="M146,192 L170,182 L176,188 L152,196 Z" fill="url(#st)"/>']
    moss = ['<path d="M56,112 C60,106 66,108 72,104 L72,114 C66,116 60,116 56,118 Z M128,152 C134,148 140,150 144,146 L144,158 L128,158 Z" fill="url(#mo)" stroke-width="1.4"/>',
            '<path d="M86,164 C86,156 104,154 108,160 L108,166 Z" fill="url(#st)"/>']
    return svg(defs, g("socle", *base) + g("arche", *arch) + g("debris", *broken) + g("mousse", *moss), rx=88)

def portal():
    defs = [grad("st", "#d8d2c6", "#a39a8c", "#6f665b"),
            '<radialGradient id="vx"><stop offset="0" stop-color="#ffffff"/><stop offset=".3" stop-color="#b494ff"/><stop offset=".75" stop-color="#6b3fd6"/><stop offset="1" stop-color="#2f1a70"/></radialGradient>',
            '<radialGradient id="gl"><stop offset="0" stop-color="#b494ff" stop-opacity=".6"/><stop offset="1" stop-color="#b494ff" stop-opacity="0"/></radialGradient>']
    glow = ['<ellipse cx="100" cy="110" rx="80" ry="84" fill="url(#gl)" stroke="none"/>']
    base = ['<path d="M30,186 L38,172 L162,172 L170,186 Z" fill="url(#st)"/>',
            '<path d="M38,172 L46,164 L154,164 L162,172 Z" fill="#e6e0d4"/>']
    vortex = ['<path d="M62,164 V100 C62,52 138,52 138,100 V164 Z" fill="url(#vx)"/>',
              '<path d="M100,112 C112,112 114,128 100,130 C84,132 80,110 96,102 C116,92 128,116 120,134 C110,154 76,146 74,118 C72,90 104,76 124,92" stroke="#fff" stroke-opacity=".75" stroke-width="2.4" fill="none" stroke-linecap="round"/>']
    arch = ['<path d="M44,164 V96 C44,30 156,30 156,96 V164 L138,164 V100 C138,52 62,52 62,100 V164 Z" fill="url(#st)"/>',
            '<path d="M92,34 L100,22 L108,34 Z" fill="#b494ff"/>',
            '<path d="M48,120 h10 M48,140 h10 M142,120 h10 M142,140 h10 M52,76 l8,4 M148,76 l-8,4" stroke="#b494ff" stroke-width="2.6" fill="none"/>']
    return svg(defs, g("lueur", *glow) + g("socle", *base) + g("vortex", *vortex) + g("arche", *arch), rx=80)

SITES = {
    "village": village, "monastery": monastery, "keep": keep, "mage_tower": mage_tower,
    "magical_glade": magical_glade, "dungeon": dungeon, "tomb": tomb, "monster_den": monster_den,
    "spawning_grounds": spawning_grounds, "ruins": ruins, "portal": portal,
}
for c in CITY:
    SITES[f"city_{c}"] = (lambda c=c: city(c))
    SITES[f"mine_{c}"] = (lambda c=c: mine(c))

only = sys.argv[1:]
for name, fn in SITES.items():
    if only and name not in only:
        continue
    open(os.path.join(OUT, name + ".svg"), "w").write(fn())
print("ok", len(SITES))
