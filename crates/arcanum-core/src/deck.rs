use crate::CardId;

/// Deck d'un joueur ; la carte du dessus est la première.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeedDeck {
    cards: Vec<CardId>,
}

impl DeedDeck {
    pub(crate) fn new(cards: Vec<CardId>) -> Self {
        Self { cards }
    }

    pub fn cards(&self) -> &[CardId] {
        &self.cards
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn top(&self) -> Option<CardId> {
        self.cards.first().copied()
    }

    pub(crate) fn remove_top(&mut self) {
        if !self.cards.is_empty() {
            self.cards.remove(0);
        }
    }

    /// Une carte gagnée va sur le dessus du deck (MKUE p. 5, *"Gain" Effects* 3 ; spec 050).
    pub(crate) fn put_on_top(&mut self, card: CardId) {
        self.cards.insert(0, card);
    }

    /// Retire cette carte du deck, où qu'elle soit (Préparation, spec 060).
    pub(crate) fn remove(&mut self, card: CardId) -> bool {
        match self.cards.iter().position(|&held| held == card) {
            Some(index) => {
                self.cards.remove(index);
                true
            }
            None => false,
        }
    }

    /// Sous le deck (Temporisation, spec 052).
    pub(crate) fn put_under(&mut self, card: CardId) {
        self.cards.push(card);
    }
}
