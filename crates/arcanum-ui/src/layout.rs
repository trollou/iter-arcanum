//! Disposition des cartes du panneau du bas (spec 073) : largeur des sections et cartes posées en
//! rangée, qui se chevauchent quand la place manque. Sans egui : des points, de gauche à droite.

/// Largeur d'une carte de la main ou d'une unité, en points (spec 077).
pub const CARD_WIDTH: f32 = 216.0;
/// Hauteur d'une carte de la main ou d'une unité, en points (spec 077).
pub const CARD_HEIGHT: f32 = 240.0;
/// Écart entre deux cartes posées côte à côte, sans chevauchement.
pub const CARD_GAP: f32 = 8.0;
/// Largeur gardée par une section quand la place manque : deux cartes.
pub const MIN_SECTION_WIDTH: f32 = 400.0;
/// Écart ajouté entre deux sections du panneau du bas.
pub const SECTION_GAP: f32 = 16.0;
/// Largeur de la section Compétences, à droite de la main (spec 051).
pub const SKILLS_WIDTH: f32 = 240.0;

/// Largeur naturelle d'une rangée de `n` cartes : cartes et écarts, sans chevauchement.
pub fn row_width(n: usize) -> f32 {
    if n == 0 {
        return 0.0;
    }
    let n = n as f32;
    n * CARD_WIDTH + (n - 1.0) * CARD_GAP
}

/// Largeur de chaque section, à partir de leurs largeurs naturelles et de la largeur disponible :
/// naturelles si tout tient, sinon partagées en proportion, chaque section gardant au moins la plus
/// petite de sa largeur naturelle et de `MIN_SECTION_WIDTH`.
pub fn section_widths(natural: &[f32], available: f32) -> Vec<f32> {
    let total: f32 = natural.iter().sum();
    if total <= available {
        return natural.to_vec();
    }
    // Les sections sous leur minimum le gardent ; les autres se partagent le reste en proportion.
    // Chaque tour fixe au moins une section de plus, ou s'arrête.
    let minimum: Vec<f32> = natural
        .iter()
        .map(|&width| width.min(MIN_SECTION_WIDTH))
        .collect();
    let mut fixed = vec![false; natural.len()];
    loop {
        let fixed_width: f32 = (0..natural.len())
            .filter(|&i| fixed[i])
            .map(|i| minimum[i])
            .sum();
        let shared: f32 = (0..natural.len())
            .filter(|&i| !fixed[i])
            .map(|i| natural[i])
            .sum();
        let rest = (available - fixed_width).max(0.0);
        let share = |i: usize| {
            if shared > 0.0 {
                rest * natural[i] / shared
            } else {
                0.0
            }
        };
        let below: Vec<usize> = (0..natural.len())
            .filter(|&i| !fixed[i] && share(i) < minimum[i])
            .collect();
        if below.is_empty() {
            return (0..natural.len())
                .map(|i| if fixed[i] { minimum[i] } else { share(i) })
                .collect();
        }
        for i in below {
            fixed[i] = true;
        }
    }
}

/// Place prise entre deux sections : l'écart, et l'espacement d'egui entre éléments (`spacing`)
/// de part et d'autre (issue privée 295 : sans eux, la section Compétences débordait à droite).
pub fn section_gap(spacing: f32) -> f32 {
    SECTION_GAP + 2.0 * spacing
}

/// Largeur des sections du panneau du bas, de gauche à droite, dans un panneau de `panel` points :
/// Unités et Main se partagent (`section_widths`) ce que laissent les écarts et, s'il y en a une,
/// la section Compétences, qui garde `SKILLS_WIDTH` (spec 051, 073).
pub fn bottom_section_widths(
    units: f32,
    hand: f32,
    skills: bool,
    panel: f32,
    spacing: f32,
) -> Vec<f32> {
    let gaps = if skills { 2.0 } else { 1.0 };
    let reserved = gaps * section_gap(spacing) + if skills { SKILLS_WIDTH } else { 0.0 };
    let mut widths = section_widths(&[units, hand], (panel - reserved).max(0.0));
    if skills {
        widths.push(SKILLS_WIDTH);
    }
    widths
}

/// Spec 077, C2 : la part d'une image carrée gardée dans une bande de `width` × `height` points,
/// centrée verticalement, en coordonnées de texture (0 en haut, 1 en bas) : `(haut, bas)`.
pub fn band_uv(width: f32, height: f32) -> (f32, f32) {
    if width <= 0.0 || height >= width {
        return (0.0, 1.0);
    }
    let kept = height / width;
    let top = (1.0 - kept) / 2.0;
    (top, top + kept)
}

/// Hauteur d'une carte survolée (spec 073, C12 bis) : celle de son contenu, cadre compris, au
/// moins `CARD_HEIGHT` et au plus `ceiling`, la place au-dessus du bas de la carte.
pub fn card_height(content: f32, ceiling: f32) -> f32 {
    content.min(ceiling).max(CARD_HEIGHT)
}

/// Les cartes d'une section posées en rangée : abscisse de chaque carte depuis le bord gauche de
/// la section, et ordre de dessin.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FanLayout {
    /// Abscisse du bord gauche de chaque carte, dans l'ordre de la section.
    pub positions: Vec<f32>,
    /// Ordre de dessin : la suivante par-dessus la précédente, la carte survolée en dernier.
    pub draw_order: Vec<usize>,
}

impl FanLayout {
    /// La carte du dessus sous le pointeur, à l'abscisse `x` dans la section.
    pub fn card_at(&self, x: f32) -> Option<usize> {
        self.draw_order.iter().rev().copied().find(|&card| {
            self.positions
                .get(card)
                .is_some_and(|&left| left <= x && x < left + CARD_WIDTH)
        })
    }
}

/// Pose `n` cartes dans une section de largeur `width`, la carte `hovered` survolée.
pub fn fan_layout(n: usize, width: f32, hovered: Option<usize>) -> FanLayout {
    let mut draw_order: Vec<usize> = (0..n).collect();
    if n == 0 || row_width(n) <= width {
        let positions = (0..n).map(|i| i as f32 * (CARD_WIDTH + CARD_GAP)).collect();
        return FanLayout {
            positions,
            draw_order,
        };
    }
    // Aucune carte ne dépasse le bord droit ; dans une section trop étroite, elles s'empilent.
    let right = (width - CARD_WIDTH).max(0.0);
    let step = |covered: f32, gaps: usize| {
        if gaps == 0 {
            0.0
        } else {
            ((width - covered) / gaps as f32).max(0.0)
        }
    };
    // La dernière carte est toujours entière : la survoler ne change rien.
    let hovered = hovered.filter(|&k| k + 1 < n);
    let positions = match hovered {
        None => {
            let s = step(CARD_WIDTH, n - 1);
            (0..n).map(|i| (i as f32 * s).min(right)).collect()
        }
        Some(k) => {
            let s = step(2.0 * CARD_WIDTH, n - 2);
            draw_order.retain(|&card| card != k);
            draw_order.push(k);
            (0..n)
                .map(|i| {
                    let x = if i <= k {
                        i as f32 * s
                    } else {
                        CARD_WIDTH + (i - 1) as f32 * s
                    };
                    x.min(right)
                })
                .collect()
        }
    };
    FanLayout {
        positions,
        draw_order,
    }
}
