use crate::{Action, Event, GameState, PlayerEvent, PlayerId, PlayerView, apply, filter_event};

/// Entrée du journal : événement numéroté, avec le numéro de la décision qui l'a produit
/// (0 pour la mise en place). La version du format est écrite une fois par sauvegarde (spec 024).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggedEvent {
    pub sequence: u64,
    pub decision: u64,
    pub event: Event,
}

/// Suite ordonnée et numérotée des événements de la partie.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventLog {
    entries: Vec<LoggedEvent>,
}

impl EventLog {
    pub fn push(&mut self, decision: u64, event: Event) {
        let sequence = self.entries.len() as u64;
        self.entries.push(LoggedEvent {
            sequence,
            decision,
            event,
        });
    }

    /// Retire les événements des décisions `decision` et suivantes (revenir en arrière, spec 024).
    pub(crate) fn truncate_from(&mut self, decision: u64) {
        self.entries.retain(|entry| entry.decision < decision);
    }

    pub fn entries(&self) -> &[LoggedEvent] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Reconstruit l'état avec `apply` seul, sans hasard.
    pub fn replay(&self) -> GameState {
        self.entries
            .iter()
            .fold(GameState::default(), |state, entry| {
                apply(state, &entry.event)
            })
    }

    /// Le journal tel que `viewer` a le droit de le voir.
    pub fn filtered_for(&self, viewer: PlayerId) -> Vec<PlayerEvent> {
        self.entries
            .iter()
            .map(|entry| filter_event(&entry.event, viewer))
            .collect()
    }
}

/// Une décision prise : données d'entraînement pour les IA.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRecord {
    pub player: PlayerId,
    pub view: PlayerView,
    pub legal_actions: Vec<Action>,
    pub chosen: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecisionLog {
    entries: Vec<DecisionRecord>,
}

impl DecisionLog {
    pub fn push(&mut self, record: DecisionRecord) {
        self.entries.push(record);
    }

    pub(crate) fn pop(&mut self) {
        self.entries.pop();
    }

    pub fn entries(&self) -> &[DecisionRecord] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
