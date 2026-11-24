# Génère les mages de glace et de feu (pile violette, spec 075, fil D4) : un même mage en robe à capuche, bâton à gauche,
# sort levé à droite ; ne changent que la palette, l'ornement du bâton et le sort.
# Cadre 120 x 160 du gabarit (assets/images/figure/README.md).
# Usage : python3 scripts/figurines/mages.py
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "images", "figure")
SW = 'stroke="#17120c" stroke-width="2.2" stroke-linejoin="round"'


def grad(id_, c1, c2, c3):
    return (f'<linearGradient id="{id_}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{c1}"/>'
            f'<stop offset=".55" stop-color="{c2}"/><stop offset="1" stop-color="{c3}"/></linearGradient>')


def g(id_, *items):
    return f'  <g id="{id_}" {SW}>\n' + "".join(f"    {i}\n" for i in items if i) + "  </g>\n"


def mage(name, robe, trim, spell, staff_top, spell_shape, eye, beard):
    defs = [grad("sk", "#f6d6b0", "#d9a77c", "#b9805a"), grad("ro", *robe), grad("sp", *spell),
            '<linearGradient id="wd" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#9a7448"/>'
            '<stop offset="1" stop-color="#4f3820"/></linearGradient>',
            '<radialGradient id="sh"><stop offset="0" stop-color="#000" stop-opacity=".55"/>'
            '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>']
    body = ['  <ellipse id="ombre" cx="60" cy="150" rx="40" ry="8" fill="url(#sh)"/>\n']
    body.append(g("cape",
                  '<path d="M38,52 C24,76 22,110 24,140 L96,140 C98,110 96,76 82,52 Z" fill="url(#ro)"/>',
                  '<path d="M30,90 C28,110 28,124 30,136 M90,90 C92,110 92,124 90,136" stroke-opacity=".4" '
                  'stroke-width="1.4" fill="none"/>'))
    body.append(g("jambes",
                  '<path d="M38,138 L56,138 L56,146 L30,146 C30,142 34,138 38,138 Z" fill="#2b1a0c"/>',
                  '<path d="M64,138 L82,138 C86,138 90,142 90,146 L64,146 Z" fill="#2b1a0c"/>',
                  '<path d="M42,100 C38,114 34,128 32,140 L88,140 C86,128 82,114 78,100 Z" fill="url(#ro)"/>',
                  f'<path d="M33,135 L87,135" stroke="{trim}" stroke-width="3.2" fill="none"/>',
                  '<path d="M60,104 L60,132" stroke-opacity=".45" stroke-width="1.4" fill="none"/>'))
    body.append(g("torse",
                  '<path d="M40,62 C38,80 40,94 42,106 L78,106 C80,94 82,80 80,62 C70,56 50,56 40,62 Z" fill="url(#ro)"/>',
                  f'<path d="M60,62 L60,106" stroke="{trim}" stroke-width="3.2" fill="none"/>',
                  '<path d="M41,96 L79,96 L78,104 L42,104 Z" fill="#2b1a0c"/>',
                  '<circle cx="60" cy="100" r="4.5" fill="url(#sp)" stroke-width="1.6"/>'))
    body.append(g("bras-gauche",
                  '<path d="M20,146 L22,32 L28,32 L27,146 Z" fill="url(#wd)"/>',
                  staff_top,
                  '<path d="M40,64 C30,70 24,80 22,90 L36,94 C36,86 40,78 46,74 Z" fill="url(#ro)"/>',
                  f'<path d="M22,90 L36,94" stroke="{trim}" stroke-width="3" fill="none"/>',
                  '<circle cx="25" cy="94" r="7" fill="url(#sk)"/>'))
    body.append(g("bras-droit",
                  '<path d="M80,64 C90,62 96,56 98,46 L86,40 C84,50 80,54 74,58 Z" fill="url(#ro)"/>',
                  f'<path d="M86,40 L98,46" stroke="{trim}" stroke-width="3" fill="none"/>',
                  '<circle cx="93" cy="38" r="7" fill="url(#sk)"/>',
                  spell_shape))
    body.append(g("tete",
                  '<path d="M38,58 C34,40 40,22 52,12 C56,8 58,4 58,0 C66,8 82,24 82,58 C76,62 44,62 38,58 Z" fill="url(#ro)"/>',
                  '<path d="M44,50 C44,38 50,32 60,32 C70,32 76,38 76,50 C76,58 70,64 60,64 C50,64 44,58 44,50 Z" fill="url(#sk)"/>',
                  f'<path d="M46,58 C48,70 54,82 60,86 C66,82 72,70 74,58 C68,62 52,62 46,58 Z" fill="{beard}"/>',
                  '<path d="M52,64 C56,62 64,62 68,64" stroke-width="1.6" fill="none"/>',
                  '<path d="M47,42 L56,45 M73,42 L64,45" stroke-width="3.2" stroke-linecap="round" fill="none"/>',
                  f'<ellipse cx="52" cy="49" rx="3" ry="2.4" fill="{eye}" stroke-width="1"/>'
                  f'<ellipse cx="68" cy="49" rx="3" ry="2.4" fill="{eye}" stroke-width="1"/>',
                  f'<path d="M40,56 C38,40 44,28 56,22 M80,56 C82,40 76,28 64,22" stroke="{trim}" stroke-width="2.4" fill="none"/>'))
    svg = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 160" width="120" height="160">\n'
           '  <defs>\n' + "".join(f"    {d}\n" for d in defs) + '  </defs>\n' + "".join(body) + '</svg>\n')
    with open(os.path.join(OUT, f"{name}.svg"), "w", encoding="utf-8", newline="\n") as f:
        f.write(svg)


# Mages de glace : robe bleu pâle, barbe blanche, bâton coiffé d'un cristal, éclats de glace au poing.
mage("ice_mages", ("#bfe6f7", "#5fa6cf", "#24577e"), "#f4fbff", ("#ffffff", "#9ff0ff", "#2fb6d6"),
     '<path d="M25,4 L32,18 L25,34 L18,18 Z" fill="url(#sp)"/><path d="M25,4 L25,34" stroke-width="1.2" '
     'stroke-opacity=".5" fill="none"/>',
     '<path d="M93,30 L88,10 L96,22 L100,2 L102,22 L112,12 L104,30 Z" fill="url(#sp)"/>'
     '<circle cx="84" cy="22" r="1.8" fill="#e8fbff" stroke="none"/><circle cx="110" cy="30" r="1.8" fill="#e8fbff" stroke="none"/>',
     "#9ff0ff", "#f4f7fa")

# Mages de feu : robe rouge sombre, barbe rousse, bâton à braise, boule de feu dans la main.
mage("fire_mages", ("#e0503a", "#a32418", "#4e0f0a"), "#ffc23a", ("#fff2a0", "#ffb02e", "#e0401a"),
     '<path d="M25,32 C16,26 16,16 22,8 C22,14 26,16 26,10 C30,14 32,8 30,2 C38,10 36,26 25,32 Z" fill="url(#sp)" '
     'stroke-width="1.6"/>',
     '<path d="M93,30 C82,26 82,12 90,6 C90,12 94,14 94,8 C98,12 102,6 100,0 C112,8 110,26 93,30 Z" fill="url(#sp)" '
     'stroke-width="1.6"/><circle cx="94" cy="20" r="4" fill="#fff8d0" stroke="none"/>',
     "#ffd94a", "#b8542a")
