# Décor de relief (forêt, collines, montagne) et sols nus. Spec 075, D2.
import os
IMAGES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images")
import math, random, os
REL = os.path.join(IMAGES, "relief", "")
TER = os.path.join(IMAGES, "terrain", "")
os.makedirs(REL, exist_ok=True)
R, TILT, CX, CY = 200, 0.8, 180, 300
SW = 'stroke="#17120c" stroke-width="2.2" stroke-linejoin="round"'
SHADOW = '<radialGradient id="sh"><stop offset="0" stop-color="#000" stop-opacity=".45"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>'

def grad(id_, c1, c2, c3=None, x2=1, y2=1):
    s = f'<stop offset="0" stop-color="{c1}"/>'
    s += (f'<stop offset=".5" stop-color="{c2}"/><stop offset="1" stop-color="{c3}"/>' if c3 else f'<stop offset="1" stop-color="{c2}"/>')
    return f'<linearGradient id="{id_}" x1="0" y1="0" x2="{x2}" y2="{y2}">{s}</linearGradient>'

def at(u, v):
    return CX + u * R, CY + v * R * TILT

def f(x):
    return f"{x:.1f}".rstrip("0").rstrip(".")

# ---------- éléments ----------
def conifer(x, y, s):
    h, w = 110 * s, 30 * s
    it = [f'<ellipse cx="{f(x+6*s)}" cy="{f(y)}" rx="{f(w*1.2)}" ry="{f(w*0.36)}" fill="url(#sh)" stroke="none"/>',
          f'<rect x="{f(x-4*s)}" y="{f(y-16*s)}" width="{f(8*s)}" height="{f(16*s)}" fill="url(#bk)"/>']
    for k, (top, base, ww) in enumerate([(0.42, 0.12, 1.0), (0.66, 0.36, 0.8), (1.0, 0.6, 0.58)]):
        by = y - 12 * s - base * h * 0.85
        ty = y - top * h
        tw = w * ww
        it.append(f'<path d="M{f(x-tw)},{f(by)} L{f(x)},{f(ty)} L{f(x+tw)},{f(by)} C{f(x+tw*0.4)},{f(by+5*s)} {f(x-tw*0.4)},{f(by+5*s)} {f(x-tw)},{f(by)} Z" fill="url(#pn)"/>')
    it.append(f'<path d="M{f(x-w*0.25)},{f(y-h*0.5)} L{f(x-w*0.05)},{f(y-h*0.62)}" stroke="#9be08a" stroke-opacity=".7" stroke-width="2" fill="none"/>')
    return it

def leafy(x, y, s):
    r = 26 * s
    it = [f'<ellipse cx="{f(x+6*s)}" cy="{f(y)}" rx="{f(r*1.25)}" ry="{f(r*0.4)}" fill="url(#sh)" stroke="none"/>',
          f'<path d="M{f(x-5*s)},{f(y)} L{f(x-4*s)},{f(y-34*s)} L{f(x+4*s)},{f(y-34*s)} L{f(x+5*s)},{f(y)} Z" fill="url(#bk)"/>',
          f'<path d="M{f(x-r)},{f(y-40*s)} C{f(x-r*1.35)},{f(y-70*s)} {f(x-r*0.6)},{f(y-96*s)} {f(x)},{f(y-92*s)} C{f(x+r*0.7)},{f(y-98*s)} {f(x+r*1.4)},{f(y-70*s)} {f(x+r)},{f(y-40*s)} C{f(x+r*0.5)},{f(y-28*s)} {f(x-r*0.5)},{f(y-28*s)} {f(x-r)},{f(y-40*s)} Z" fill="url(#lf)"/>',
          f'<path d="M{f(x-r*0.6)},{f(y-66*s)} C{f(x-r*0.5)},{f(y-78*s)} {f(x-r*0.2)},{f(y-84*s)} {f(x+r*0.1)},{f(y-84*s)}" stroke="#c8f0a0" stroke-opacity=".7" stroke-width="2.4" fill="none"/>']
    return it

def mound(x, y, w, h):
    it = [f'<ellipse cx="{f(x+w*0.12)}" cy="{f(y+2)}" rx="{f(w*1.1)}" ry="{f(w*0.22)}" fill="url(#sh)" stroke="none"/>',
          f'<path d="M{f(x-w)},{f(y)} C{f(x-w*0.85)},{f(y-h*0.75)} {f(x-w*0.45)},{f(y-h)} {f(x)},{f(y-h)} C{f(x+w*0.45)},{f(y-h)} {f(x+w*0.85)},{f(y-h*0.75)} {f(x+w)},{f(y)} Z" fill="url(#hl)"/>',
          f'<path d="M{f(x+w*0.25)},{f(y-h*0.92)} C{f(x+w*0.6)},{f(y-h*0.7)} {f(x+w*0.75)},{f(y-h*0.4)} {f(x+w*0.7)},{f(y)} L{f(x+w)},{f(y)} C{f(x+w*0.85)},{f(y-h*0.75)} {f(x+w*0.6)},{f(y-h*0.95)} {f(x+w*0.25)},{f(y-h*0.92)} Z" fill="#8a6a3a" fill-opacity=".45" stroke="none"/>',
          f'<path d="M{f(x-w*0.7)},{f(y-h*0.5)} C{f(x-w*0.55)},{f(y-h*0.8)} {f(x-w*0.3)},{f(y-h*0.92)} {f(x-w*0.05)},{f(y-h*0.93)}" stroke="#f6e0a8" stroke-opacity=".9" stroke-width="2.6" fill="none" stroke-linecap="round"/>',
          f'<path d="M{f(x-w*0.2)},{f(y-h*0.98)} l-3,-7 M{f(x-w*0.2)},{f(y-h*0.98)} l1,-8 M{f(x-w*0.2)},{f(y-h*0.98)} l5,-6 M{f(x+w*0.45)},{f(y-h*0.35)} l-2,-6 M{f(x+w*0.45)},{f(y-h*0.35)} l3,-6" stroke="#6a8a3a" stroke-width="2.2" fill="none" stroke-linecap="round"/>']
    return it

def peak(x, y, w, h, snow=True):
    it = [f'<ellipse cx="{f(x+w*0.15)}" cy="{f(y)}" rx="{f(w*1.15)}" ry="{f(w*0.3)}" fill="url(#sh)" stroke="none"/>',
          f'<path d="M{f(x-w)},{f(y)} L{f(x-w*0.35)},{f(y-h*0.6)} L{f(x)},{f(y-h)} L{f(x+w*0.45)},{f(y-h*0.5)} L{f(x+w)},{f(y)} C{f(x+w*0.4)},{f(y+w*0.12)} {f(x-w*0.4)},{f(y+w*0.12)} {f(x-w)},{f(y)} Z" fill="url(#rk)"/>',
          f'<path d="M{f(x)},{f(y-h)} L{f(x+w*0.12)},{f(y-h*0.45)} L{f(x+w*0.05)},{f(y+w*0.08)} L{f(x+w)},{f(y)} L{f(x+w*0.45)},{f(y-h*0.5)} Z" fill="url(#rks)"/>']
    if snow:
        it.append(f'<path d="M{f(x-w*0.35)},{f(y-h*0.6)} L{f(x)},{f(y-h)} L{f(x+w*0.3)},{f(y-h*0.67)} L{f(x+w*0.12)},{f(y-h*0.62)} L{f(x+w*0.02)},{f(y-h*0.7)} L{f(x-w*0.12)},{f(y-h*0.56)} Z" fill="url(#sn)"/>')
    it.append(f'<path d="M{f(x-w*0.6)},{f(y-h*0.22)} L{f(x-w*0.3)},{f(y-h*0.36)} M{f(x+w*0.55)},{f(y-h*0.2)} L{f(x+w*0.7)},{f(y-h*0.1)}" stroke-width="1.6" stroke-opacity=".5" fill="none"/>')
    return it

# ---------- compositions ----------
# (u, v) en rayons de case, sur le plateau ; v < 0 vers l'arrière. Les places « au centre » (|u| < 0,4 et
# v > -0,2) sont celles du site, du héros et du défenseur : vides dans la variante dégagée.
def keep_for_clear(u, v):
    return v < -0.18 or (abs(u) > 0.6 and v < 0.12)

def compose(spots, draw, clear):
    items = []
    for spot in spots:
        u, v = spot[0], spot[1]
        if clear and not keep_for_clear(u, v):
            continue
        x, y = at(u, v)
        items.append((y, x, draw(x, y, *spot[2:])))
    items.sort()
    return [i for _, _, it in items for i in it]

def doc(defs, id_, items):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 480" width="360" height="480">\n'
            f'  <defs>\n    ' + "\n    ".join(defs + [SHADOW]) + '\n  </defs>\n'
            f'  <g id="{id_}" {SW}>\n' + "".join(f"    {i}\n" for i in items) + '  </g>\n</svg>\n')

FOREST_DEFS = [grad("pn", "#4caa52", "#2a7330", "#174a1c"), grad("lf", "#7fc85a", "#3f8a36", "#235a22"), grad("bk", "#8a5a34", "#553419", x2=1, y2=0)]
FOREST = [(-0.5, -0.55, "c", 0.95), (0.0, -0.68, "l", 1.0), (0.48, -0.52, "c", 1.05), (-0.68, -0.12, "l", 0.9), (0.68, -0.08, "c", 0.95),
          (-0.3, -0.25, "c", 1.0), (0.26, -0.28, "l", 0.95), (-0.55, 0.3, "c", 1.05), (0.55, 0.32, "l", 1.0), (0.0, 0.08, "c", 1.1),
          (-0.22, 0.6, "l", 0.9), (0.28, 0.62, "c", 0.95)]

def forest_item(x, y, kind, s):
    return conifer(x, y, s) if kind == "c" else leafy(x, y, s)

HILL_DEFS = [grad("hl", "#e8c47c", "#c49a58", "#8a6a3a", x2=1, y2=0)]
HILLS = [(-0.4, -0.5, 66, 50), (0.4, -0.52, 60, 44), (-0.64, -0.02, 42, 32), (0.64, 0.02, 44, 34),
         (0.0, -0.05, 78, 60), (-0.38, 0.45, 56, 40), (0.4, 0.46, 58, 42)]

MOUNT_DEFS = [grad("rk", "#b8aca4", "#857470", "#5a4c46"), grad("rks", "#5f524c", "#3e3430", x2=1, y2=0), grad("sn", "#ffffff", "#dfe6ee")]
MOUNTS = [(-0.42, -0.5, 78, 150), (0.38, -0.58, 70, 130), (-0.66, -0.05, 44, 84), (0.66, -0.02, 46, 90),
          (0.0, 0.0, 90, 170), (-0.45, 0.45, 60, 100), (0.48, 0.48, 58, 96)]

def write(name, defs, id_, spots, draw):
    for clear in (False, True):
        fn = f"{name}{'_clear' if clear else ''}.svg"
        open(REL + fn, "w").write(doc(defs, id_, compose(spots, draw, clear)))

write("forest", FOREST_DEFS, "arbres", FOREST, forest_item)
write("hills", HILL_DEFS, "buttes", HILLS, mound)
write("mountain", MOUNT_DEFS, "pics", MOUNTS, peak)

# ---------- sols nus (même cadre que terrain/*.svg) ----------
HEX = "86.603,0 173.205,50 173.205,150 86.603,200 0,150 0,50"
def ground(fill, marks):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 173.205 200.0" width="173.205" height="200.0">\n'
            f'  <defs><clipPath id="c"><polygon points="{HEX}"/></clipPath></defs>\n'
            f'  <polygon points="{HEX}" fill="{fill}"/>\n  <g clip-path="url(#c)">\n'
            + "".join(f"    {m}\n" for m in marks) + '  </g>\n</svg>\n')

rnd = random.Random(75)
def scatter(n, make):
    out = []
    for _ in range(n):
        out.append(make(rnd.uniform(6, 167), rnd.uniform(6, 194)))
    return out

forest_ground = ground("#2F6A30",
    scatter(14, lambda x, y: f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="{f(rnd.uniform(8,16))}" ry="{f(rnd.uniform(5,9))}" fill="#3E8040"/>')
    + scatter(18, lambda x, y: f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="3" ry="2" fill="{rnd.choice(["#7A9A3A","#A8792E","#5BAE5F"])}"/>')
    + scatter(8, lambda x, y: f'<path d="M{f(x)},{f(y)} l-4,-7 M{f(x)},{f(y)} l0,-8 M{f(x)},{f(y)} l4,-7" stroke="#5BAE5F" stroke-width="2" stroke-linecap="round"/>'))
hills_ground = ground("#C9A15E",
    scatter(12, lambda x, y: f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="{f(rnd.uniform(10,20))}" ry="{f(rnd.uniform(6,10))}" fill="#BE9454"/>')
    + scatter(14, lambda x, y: f'<path d="M{f(x)},{f(y)} l-3,-6 M{f(x)},{f(y)} l0,-7 M{f(x)},{f(y)} l3,-6" stroke="#7E8F3E" stroke-width="2" stroke-linecap="round"/>')
    + scatter(6, lambda x, y: f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="3.5" ry="2.5" fill="#8C6B3E"/>'))
mountain_ground = ground("#6E5E58",
    scatter(12, lambda x, y: f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="{f(rnd.uniform(10,18))}" ry="{f(rnd.uniform(6,10))}" fill="#62534D"/>')
    + scatter(16, lambda x, y: f'<path d="M{f(x-5)},{f(y)} L{f(x-2)},{f(y-5)} L{f(x+4)},{f(y-4)} L{f(x+6)},{f(y)} Z" fill="#857470"/>')
    + scatter(8, lambda x, y: f'<path d="M{f(x)},{f(y)} l8,4 l6,-2" stroke="#4F423E" stroke-width="1.5" fill="none"/>'))
open(TER + "forest.svg", "w").write(forest_ground)
open(TER + "hills.svg", "w").write(hills_ground)
open(TER + "mountain.svg", "w").write(mountain_ground)
print("ok")
