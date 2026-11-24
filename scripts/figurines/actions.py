# Icônes des boutons d'action du plateau (spec 075, D2). Cadre 64 x 64 comme ui/*.svg.
import os
IMAGES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images")
OUT = os.path.join(IMAGES, "action", "")
import os
os.makedirs(OUT, exist_ok=True)
SW = 'stroke="#20242A" stroke-width="2.2" stroke-linejoin="round"'

def svg(defs, body):
    d = f"<defs>{''.join(defs)}</defs>" if defs else ""
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">{d}<g {SW}>\n{body}</g></svg>\n'

def lg(id_, a, b):
    return f'<linearGradient id="{id_}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>'

STEEL = lg("s", "#f4f7fa", "#8a96a3")
GOLD = lg("g", "#f6dc8c", "#a87a2a")

def sword(angle):
    return (f'<g transform="rotate({angle} 32 32)">'
            '<path d="M32,4 L36,10 L36,38 L28,38 L28,10 Z" fill="url(#s)"/>'
            '<path d="M32,9 L32,36" stroke="#FFFFFF" stroke-opacity=".5" stroke-width="1.4"/>'
            '<rect x="21" y="38" width="22" height="5" rx="2" fill="url(#g)"/>'
            '<rect x="29.5" y="43" width="5" height="10" fill="#6B4A2E"/>'
            '<circle cx="32" cy="56" r="3.6" fill="url(#g)"/></g>\n')

ICONS = {
    # Défier et Engager le combat : deux épées croisées (le fond rouge du second est dessiné par le code).
    "swords": svg([STEEL, GOLD], sword(-40) + sword(40)),
    # Interagir : bulle de dialogue, trois points.
    "speech": svg([lg("b", "#ffffff", "#c9d2db")],
        '<path d="M8,12 C8,8 11,6 15,6 L49,6 C53,6 56,8 56,12 L56,38 C56,42 53,44 49,44 L28,44 L16,56 L18,44 L15,44 C11,44 8,42 8,38 Z" fill="url(#b)"/>\n'
        '<circle cx="21" cy="25" r="4" fill="#20242A" stroke="none"/><circle cx="32" cy="25" r="4" fill="#20242A" stroke="none"/><circle cx="43" cy="25" r="4" fill="#20242A" stroke="none"/>\n'),
    # Piller : bourse nouée, pièces.
    "purse": svg([lg("p", "#c08a54", "#6b4220"), GOLD],
        '<path d="M22,22 C10,30 8,46 14,54 C18,58 46,58 50,54 C56,46 54,30 42,22 Z" fill="url(#p)"/>\n'
        '<path d="M22,22 L18,10 L26,14 L32,8 L38,14 L46,10 L42,22 Z" fill="url(#p)"/>\n'
        '<path d="M21,22 C28,25 36,25 43,22" stroke="#d9b24a" stroke-width="3.2" fill="none"/>\n'
        '<ellipse cx="46" cy="56" rx="10" ry="4" fill="url(#g)"/><ellipse cx="46" cy="51" rx="10" ry="4" fill="url(#g)"/>\n'
        '<path d="M22,38 C24,32 28,30 30,30" stroke="#ffffff" stroke-opacity=".5" stroke-width="2" fill="none"/>\n'),
    # Incendier : flamme.
    "flame": svg([lg("f", "#ffd94a", "#e0402a")],
        '<path d="M32,4 C36,16 50,22 50,40 C50,52 42,60 32,60 C22,60 14,52 14,40 C14,30 20,24 22,16 C26,24 26,28 28,30 C30,22 30,12 32,4 Z" fill="url(#f)"/>\n'
        '<path d="M32,30 C34,38 42,42 42,50 C42,56 37,59 32,59 C27,59 22,56 22,50 C22,44 28,40 32,30 Z" fill="#fff3b0"/>\n'),
    # Entrer : porte voûtée entrouverte.
    "door": svg([lg("w", "#a8703c", "#5a3416"), lg("st", "#d8d2c6", "#7a7064")],
        '<path d="M10,60 L10,26 C10,8 54,8 54,26 L54,60 Z" fill="url(#st)"/>\n'
        '<path d="M18,60 L18,28 C18,16 46,16 46,28 L46,60 Z" fill="#17120c"/>\n'
        '<path d="M18,60 L18,28 C18,20 26,16 32,16 L34,18 L34,62 Z" fill="url(#w)"/>\n'
        '<path d="M22,30 L30,30 M22,44 L30,44" stroke-width="1.6" stroke-opacity=".7" fill="none"/>\n'
        '<circle cx="30" cy="38" r="2" fill="#d9b24a" stroke-width="1.2"/>\n'),
    # Offrande : autel de pierre, flamme de mana au-dessus.
    "altar": svg([lg("st", "#e6e0d4", "#8a8074"), lg("m", "#ffffff", "#b494ff")],
        '<path d="M10,60 L14,52 L50,52 L54,60 Z" fill="url(#st)"/>\n'
        '<path d="M18,52 L18,36 L46,36 L46,52 Z" fill="url(#st)"/>\n'
        '<path d="M12,36 L12,30 L52,30 L52,36 Z" fill="url(#st)"/>\n'
        '<path d="M32,4 C36,12 42,16 42,22 C42,27 37,30 32,30 C27,30 22,27 22,22 C22,16 28,12 32,4 Z" fill="url(#m)"/>\n'
        '<path d="M26,44 L38,44" stroke-width="1.6" stroke-opacity=".6"/>\n'),
}
for name, body in ICONS.items():
    open(OUT + name + ".svg", "w").write(body)
print("ok", len(ICONS))
