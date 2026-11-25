//! Spec #5 : la disparition des ennemis vaincus, relevée dans le journal par la session comme le
//! glissement de la figurine (spec 075, C35). Rien de caché : seulement des ennemis que le plateau
//! montrait avant le combat et que le joueur a vaincus.

use std::collections::{BTreeMap, BTreeSet};

use arcanum_core::{EnemyToken, Event, Hex, LoggedEvent, PlayerId, PlayerView};

use crate::DepartingEnemy;

/// Les ennemis que le plateau montre, d'après la vue : jetons face visible et défenseurs face
/// cachée.
#[derive(Clone, Debug, Default)]
struct BoardEnemies {
    tokens: BTreeMap<Hex, Vec<EnemyToken>>,
    face_down: BTreeSet<Hex>,
}

impl BoardEnemies {
    fn of(view: &PlayerView) -> Self {
        Self {
            tokens: view.enemy_tokens.clone(),
            face_down: view.face_down_defenders.keys().copied().collect(),
        }
    }

    /// La place du jeton `token` de `hex` sur le plateau d'avant le combat, s'il y était.
    fn departing(&self, hex: Hex, token: EnemyToken) -> Option<DepartingEnemy> {
        let tokens = self.tokens.get(&hex).map_or(&[][..], Vec::as_slice);
        let (place, of) = match tokens.iter().position(|shown| *shown == token) {
            Some(place) => (place, tokens.len()),
            // Q : un défenseur face cachée s'efface sous sa figure retournée (Règles, « Où »).
            None if self.face_down.contains(&hex) => (0, 1),
            None => return None,
        };
        Some(DepartingEnemy {
            hex,
            token,
            place,
            of,
        })
    }
}

/// Suit le journal de la partie pour en tirer la dernière disparition.
#[derive(Debug, Default)]
pub(crate) struct DepartureTracker {
    /// Le plateau hors combat, et la taille du journal quand il a été relevé.
    board: Option<(usize, BoardEnemies)>,
    /// Le combat en cours : l'indice de son `CombatStarted` et le plateau d'avant.
    combat: Option<(usize, BoardEnemies)>,
    /// Entrées du journal déjà lues.
    scanned: usize,
    count: u64,
    last: Option<(u64, Vec<DepartingEnemy>)>,
}

impl DepartureTracker {
    /// Un suivi qui commence à la fin d'un journal de `len` entrées (partie neuve ou chargée).
    pub(crate) fn from_log_len(len: usize) -> Self {
        Self {
            scanned: len,
            ..Self::default()
        }
    }

    pub(crate) fn last(&self) -> Option<(u64, &[DepartingEnemy])> {
        self.last
            .as_ref()
            .map(|(id, enemies)| (*id, enemies.as_slice()))
    }

    /// Relève le plateau avant une image, hors combat et seulement si le journal a changé.
    pub(crate) fn note_board(&mut self, len: usize, view: impl FnOnce() -> PlayerView) {
        if self.board.as_ref().is_some_and(|(known, _)| *known == len) {
            return;
        }
        let view = view();
        if view.combat.is_none() {
            self.board = Some((len, BoardEnemies::of(&view)));
        }
    }

    /// Lit les entrées nouvelles du journal : un combat qui commence garde le plateau d'avant ; un
    /// combat qui finit donne la disparition (C1 à C6).
    pub(crate) fn scan(&mut self, entries: &[LoggedEvent]) {
        for index in self.scanned..entries.len() {
            match &entries[index].event {
                Event::CombatStarted { player, .. } if *player == PlayerId::SOLO => {
                    self.combat = self.board.as_ref().map(|(_, board)| (index, board.clone()));
                }
                Event::CombatEnded { player } if *player == PlayerId::SOLO => {
                    self.finish(&entries[..index]);
                }
                _ => {}
            }
        }
        self.scanned = entries.len();
    }

    fn finish(&mut self, entries: &[LoggedEvent]) {
        self.last = None;
        let Some((start, board)) = self.combat.take() else {
            return;
        };
        let Event::CombatStarted {
            enemies: fought, ..
        } = &entries[start].event
        else {
            return;
        };
        let defeated = entries[start..]
            .iter()
            .flat_map(|entry| match &entry.event {
                Event::EnemiesDefeated {
                    player, enemies, ..
                } if *player == PlayerId::SOLO => enemies.clone(),
                Event::EnemyDestroyed { player, enemy } if *player == PlayerId::SOLO => {
                    vec![*enemy]
                }
                _ => Vec::new(),
            });
        let departing: Vec<DepartingEnemy> = defeated
            .filter_map(|token| {
                let (hex, _) = fought.iter().find(|(_, fought)| *fought == token)?;
                board.departing(*hex, token)
            })
            .collect();
        if !departing.is_empty() {
            self.count += 1;
            self.last = Some((self.count, departing));
        }
    }

    /// Après un retour en arrière (C7) : plus de disparition ; le combat en cours reste suivi s'il
    /// a commencé avant le point de retour.
    pub(crate) fn undone(&mut self, len: usize) {
        self.last = None;
        self.board = None;
        self.scanned = self.scanned.min(len);
        if self.combat.as_ref().is_some_and(|(start, _)| *start >= len) {
            self.combat = None;
        }
    }
}
