// Planche de contrôle des sites, du décor de relief et des icônes d'action (spec 075, fil D2) : un
// petit plateau incliné, posé comme le fera le jeu (assets/images/site/README.md, relief/README.md).
// Usage : node scripts/figurines/planche-plateau.js <sortie.png>
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright');

const out = process.argv[2];
if (!out) {
  console.error('Usage : node scripts/figurines/planche-plateau.js <sortie.png>');
  process.exit(1);
}
const images = path.join(__dirname, '..', '..', 'assets', 'images');
const uri = (rel) => `data:image/svg+xml;base64,${fs.readFileSync(path.join(images, `${rel}.svg`)).toString('base64')}`;

const TILT = 0.8; // BOARD_TILT
const SQRT3 = Math.sqrt(3);
const veil = 'rgba(11,22,58,0.47)'; // night_veil

// Tailles et pieds, en rayons de case (crates/arcanum-ui/src/board/screen.rs) ; décor : cadre 1,8 × 2,4,
// centre de la case au point (0,5 ; 0,625) du cadre.
const SITE = { foot: [0, 0.1], size: [1, 1] };
const HERO = { foot: [0.4, 0.55], size: [0.6, 0.8] };
const DECOR = { size: [1.8, 2.4], anchor: [0.5, 0.625] };
const RELIEF = { forest: 'forest', hills: 'hills', mountain: 'mountain' };

const center = (q, r, zoom) => [zoom * SQRT3 * (q + r / 2), zoom * 1.5 * r * TILT];

function board(cases, zoom, night) {
  const placed = cases.map((c) => ({ ...c, at: center(c.q, c.r, zoom) }));
  const xs = placed.map((c) => c.at[0]);
  const ys = placed.map((c) => c.at[1]);
  const minX = Math.min(...xs) - zoom * 1.1;
  const minY = Math.min(...ys) - zoom * 1.6;
  const w = Math.max(...xs) + zoom * 1.1 - minX;
  const h = Math.max(...ys) + zoom * 1.1 - minY;
  // Le sol est écrasé : image dessinée à sa hauteur, puis tassée par la transformation.
  const el = (rel, x, y, width, height, squash = 1) =>
    `<img src="${uri(rel)}" style="position:absolute;left:${x - minX}px;top:${y - minY}px;width:${width}px;height:${height / squash}px;transform:scaleY(${squash});transform-origin:top">`;
  let ground = '';
  let standing = '';
  for (const c of placed) {
    const [x, y] = c.at;
    ground += el(`terrain/${c.terrain}`, x - (zoom * SQRT3) / 2, y - zoom * TILT, zoom * SQRT3, 2 * zoom * TILT, TILT);
  }
  // De l'arrière vers l'avant ; sur une case : décor, site, héros.
  for (const c of [...placed].sort((a, b) => a.at[1] - b.at[1] || a.at[0] - b.at[0])) {
    const [x, y] = c.at;
    if (RELIEF[c.terrain]) {
      const clear = c.site || c.hero ? '_clear' : '';
      const [dw, dh] = DECOR.size.map((s) => s * zoom);
      standing += el(`relief/${RELIEF[c.terrain]}${clear}`, x - dw * DECOR.anchor[0], y - dh * DECOR.anchor[1], dw, dh);
    }
    const stand = (rel, { foot, size }) => {
      const [fx, fy] = [x + foot[0] * zoom, y + foot[1] * zoom * TILT];
      const [sw, sh] = size.map((s) => s * zoom);
      standing += el(rel, fx - sw / 2, fy - sh, sw, sh);
    };
    if (c.site) stand(`site/${c.site}`, SITE);
    if (c.hero) stand(`figure/${c.hero}`, HERO);
  }
  const nightVeil = night ? `<div style="position:absolute;inset:0;background:${veil}"></div>` : '';
  return `<div class="plateau" style="width:${w}px;height:${h}px">${ground}${standing}${nightVeil}</div>`;
}

// Un coin de campagne : chaque relief plein, puis dégagé sous un site ou le héros.
const corner = [
  { q: 0, r: -1, terrain: 'mountain' },
  { q: 1, r: -1, terrain: 'hills', site: 'mine_green' },
  { q: 2, r: -1, terrain: 'forest' },
  { q: -1, r: 0, terrain: 'plains', site: 'keep' },
  { q: 0, r: 0, terrain: 'plains', site: 'village', hero: 'tovak' },
  { q: 1, r: 0, terrain: 'forest', site: 'magical_glade' },
  { q: 2, r: 0, terrain: 'hills' },
  { q: -1, r: 1, terrain: 'hills' },
  { q: 0, r: 1, terrain: 'forest', hero: 'arythea' },
  { q: 1, r: 1, terrain: 'mountain' },
  { q: 2, r: 1, terrain: 'desert', site: 'mage_tower' },
];

// Les 19 sites, chacun sur une plaine, en rangées.
const sites = fs
  .readdirSync(path.join(images, 'site'))
  .filter((f) => f.endsWith('.svg'))
  .map((f) => path.basename(f, '.svg'));
const siteRows = (zoom, night) => {
  const perRow = zoom > 50 ? 7 : 10;
  const rows = [];
  for (let i = 0; i < sites.length; i += perRow) {
    rows.push(board(sites.slice(i, i + perRow).map((site, k) => ({ q: k, r: 0, terrain: 'plains', site })), zoom, night));
  }
  return rows.join('');
};

// Boutons d'action : 36 points, fond sombre (rouge pour Engager le combat), icône au centre.
const actions = [
  ['swords', '#2b2118'],
  ['swords', '#8f1d1d'],
  ['speech', '#2b2118'],
  ['purse', '#2b2118'],
  ['flame', '#2b2118'],
  ['door', '#2b2118'],
  ['altar', '#2b2118'],
];
const buttons = (size) =>
  `<div class="boutons">${actions
    .map(
      ([icon, bg]) =>
        `<div style="width:${size}px;height:${size}px;border-radius:50%;background:${bg};border:2px solid #f3d27a;display:flex;align-items:center;justify-content:center"><img src="${uri(`action/${icon}`)}" style="width:${size * 0.68}px"></div>`
    )
    .join('')}</div>`;

const html = `<html><body><style>
body{margin:0;background:#22242a;font-family:sans-serif;color:#ddd;padding:24px;width:1640px}
h2{font-size:18px;font-weight:normal;margin:24px 0 8px}
.ligne{display:flex;gap:40px;align-items:flex-end;flex-wrap:wrap}
.plateau{position:relative;margin:4px 0}
.boutons{display:flex;gap:10px;padding:14px;background:#a5d36a;border-radius:6px;width:fit-content}
</style>
<h2>Coin de campagne, zoom 90, de jour</h2>${board(corner, 90, false)}
<h2>Zooms 60 et 40, de nuit ; dézoom maximal (22)</h2><div class="ligne">${board(corner, 60, true)}${board(corner, 40, false)}${board(corner, 22, false)}</div>
<h2>Les 19 sites, zoom 60</h2>${siteRows(60, false)}
<h2>Zoom 40, de nuit</h2>${siteRows(40, true)}
<h2>Boutons d'action (36 points, puis 72 pour le détail)</h2><div class="ligne">${buttons(36)}${buttons(72)}</div>
</body></html>`;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1690, height: 800 }, deviceScaleFactor: 2 });
  await page.setContent(html);
  await page.waitForTimeout(300);
  await page.screenshot({ path: out, fullPage: true });
  await browser.close();
})();
