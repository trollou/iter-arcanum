// Planche de contrôle des figurines (spec 075, assets/images/figure/README.md).
// Usage : node scripts/figurines/planche.js <sortie.png> [nom ou chemin.svg…]
// Sans nom : toutes les figurines de assets/images/figure/. Un argument qui finit par .svg est lu tel quel
// (essais hors du dépôt) ; `--sans-nom` retire les étiquettes (essai comparatif à l'aveugle).
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright');

const dir = path.join(__dirname, '..', '..', 'assets', 'images', 'figure');
const args = process.argv.slice(2);
const blind = args.includes('--sans-nom');
const [out, ...names] = args.filter((a) => a !== '--sans-nom');
if (!out) {
  console.error('Usage : node scripts/figurines/planche.js <sortie.png> [--sans-nom] [nom ou chemin.svg…]');
  process.exit(1);
}
// Un nom sans chemin est cherché dans figure/, puis site/ (sites debout, même échelle).
const siteDir = path.join(dir, '..', 'site');
const find = (n) => {
  if (n.endsWith('.svg')) return path.resolve(n);
  const figure = path.join(dir, `${n}.svg`);
  return fs.existsSync(figure) ? figure : path.join(siteDir, `${n}.svg`);
};
const files = names.length
  ? names.map(find)
  : fs.readdirSync(dir).filter((f) => f.endsWith('.svg')).sort().map((f) => path.join(dir, f));

// Couleurs de `terrain_color` et de `night_veil` (crates/arcanum-ui/src/board/images.rs).
const grounds = [['plaine', '#A5D36A'], ['colline', '#C9A15E']];
const veil = 'rgba(11,22,58,0.47)';
// Hauteur d'une figurine : 1,4 × le zoom ; 22 au dézoom maximal.
const heights = [22, 56, 84, 126];
// Le brouillard est teinté par le code : la planche le montre aux couleurs des trois piles face cachée.
const tints = { fog: ['#9aa0a8', '#8e5bb5', '#f2f2f2'] };

const uri = (file) => `data:image/svg+xml;base64,${fs.readFileSync(file).toString('base64')}`;
// Hauteur du cadre en unités du dessin : les figurines font 160, les sites 200, à la même échelle.
const frameHeight = (file) => Number((fs.readFileSync(file, 'utf8').match(/viewBox="[\d.]+ [\d.]+ [\d.]+ ([\d.]+)"/) || [0, 160])[1]);
// Un site, carré, ne tient pas à deux bandes de front : une par ligne.
const wide = (file) => frameHeight(file) > 160;
const img = (file, figureHeight, tint) => {
  const h = (figureHeight * frameHeight(file)) / 160;
  const src = uri(file);
  if (!tint) return `<img src="${src}" height="${h}">`;
  const w = (h * 120) / 160;
  // Multiplication par la teinte, limitée à la forme de l'image.
  return `<div style="width:${w}px;height:${h}px;background:${tint};-webkit-mask:url(${src}) center/contain no-repeat"><img src="${src}" height="${h}" style="mix-blend-mode:multiply"></div>`;
};

const strip = (file, tint) =>
  grounds
    .map(([, color]) =>
      [false, true]
        .map(
          (night) =>
            `<div class="sol" style="background:${color}">${heights.map((h) => img(file, h, tint)).join('')}${
              night ? `<div class="nuit" style="background:${veil}"></div>` : ''
            }</div>`
        )
        .join('')
    )
    .join('');

const cards = files
  .flatMap((file, i) => {
    const name = path.basename(file, '.svg');
    const label = blind ? String.fromCharCode(65 + i) : name;
    return (tints[name] || [null]).map(
      (tint) =>
        `<section><div class="grand">${img(file, (400 * 160) / frameHeight(file), tint)}</div><div class="bandes${wide(file) ? ' large' : ''}">${strip(file, tint)}</div><p>${label}${
          blind ? '' : ` (${(fs.statSync(file).size / 1024).toFixed(1)} Ko)`
        }</p></section>`
    );
  })
  .join('');

const html = `<html><body><style>
body{margin:0;background:#22242a;font-family:sans-serif;color:#ddd;display:flex;flex-wrap:wrap;gap:28px;padding:28px;width:1620px}
section{width:520px;text-align:center}
.grand{height:400px;display:flex;justify-content:center}
.bandes{display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-top:10px}
.large{grid-template-columns:1fr}
.large .sol{height:170px}
.sol{position:relative;display:flex;gap:8px;align-items:flex-end;justify-content:center;padding:10px 4px 6px;border-radius:6px;overflow:hidden;height:130px}
.nuit{position:absolute;inset:0}
p{margin:8px 0 0;font-size:18px}
</style>${cards}</body></html>`;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1680, height: 600 }, deviceScaleFactor: 2 });
  await page.setContent(html);
  await page.waitForTimeout(300);
  await page.screenshot({ path: out, fullPage: true });
  await browser.close();
})();
