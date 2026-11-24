use thiserror::Error;

use crate::{
    ActionError, DecisionLog, DecisionRecord, Event, EventLog, GameConfig, GameRngs, GameState,
    LoggedEvent, Player, PlayerId, RngPositions, apply, decide, legal_actions, new_game,
    pending_decision, view,
};

/// Résultat d'un pas de la boucle de partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Advanced,
    Waiting { player: PlayerId },
    Finished,
}

/// Peut-on revenir en arrière, et sinon pourquoi (spec 024).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoStatus {
    Possible,
    /// Rien depuis le début du tour (ou de la partie) : MKUE p. 5, Reverting 1.
    NothingSinceTurnStart,
    /// Une information a été révélée ou un dé lancé : MKUE p. 5, Reverting 2.
    InformationRevealed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum UndoError {
    #[error("rien à reprendre depuis le début du tour")]
    NothingSinceTurnStart,
    #[error("une information a été révélée : impossible de revenir avant")]
    InformationRevealed,
}

/// Journal refusé à la reprise d'une partie (spec 024, L4).
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ResumeError {
    #[error("le journal ne commence pas par GameStarted")]
    NoGameStarted,
    #[error("numéro de décision en recul à l'événement n°{sequence}")]
    DecisionsOutOfOrder { sequence: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RunnerError {
    #[error("choix {index} hors des {count} actions légales")]
    ChoiceOutOfRange { index: usize, count: usize },
    #[error("aucun joueur branché pour {player:?}")]
    MissingPlayer { player: PlayerId },
    #[error(transparent)]
    Action(#[from] ActionError),
}

/// Boucle de partie, sans thread ni horloge : elle avance par `step()`.
pub struct GameRunner {
    state: GameState,
    log: EventLog,
    decisions: DecisionLog,
    rngs: GameRngs,
    /// Le joueur d'indice i est `PlayerId(i)`.
    players: Vec<Box<dyn Player>>,
    /// Taille du journal quand la décision courante a été demandée.
    requested_at: Option<usize>,
    /// La fin de partie n'est écrite qu'une fois dans les logs (spec 003 C12).
    end_logged: bool,
    /// Numéro de la prochaine décision (0 = mise en place).
    next_decision: u64,
    /// L'état avant chaque décision reprenable, depuis le dernier point de non-retour (spec 024).
    undo_stack: Vec<GameState>,
    /// Pourquoi on ne peut pas revenir avant le dernier point de non-retour.
    blocked_by: UndoError,
}

impl GameRunner {
    /// Démarre la partie : les événements de démarrage sont appliqués et journalisés.
    pub fn new(config: GameConfig, players: Vec<Box<dyn Player>>) -> Self {
        // `::log` : la crate de logs, pas le module `log` (journal) d'arcanum-core.
        ::log::info!(
            "partie créée : graine {}, {} joueur(s)",
            config.seed,
            players.len()
        );
        let mut rngs = config.rngs();
        let events = new_game(&config, &mut rngs);
        let mut runner = Self {
            state: GameState::default(),
            log: EventLog::default(),
            decisions: DecisionLog::default(),
            rngs,
            players,
            requested_at: None,
            end_logged: false,
            next_decision: 0,
            undo_stack: Vec::new(),
            blocked_by: UndoError::NothingSinceTurnStart,
        };
        runner.record(events);
        runner
    }

    /// Reprend une partie depuis son journal et les positions des générateurs (spec 024, 038) : l'état
    /// est rejoué par `apply`, la suite tire le même hasard que la partie d'origine.
    pub fn resume(
        log: EventLog,
        rng_positions: RngPositions,
        players: Vec<Box<dyn Player>>,
    ) -> Result<Self, ResumeError> {
        let entries = log.entries();
        let Some(LoggedEvent {
            event: Event::GameStarted { config },
            ..
        }) = entries.first()
        else {
            return Err(ResumeError::NoGameStarted);
        };
        if let Some(pair) = entries
            .windows(2)
            .find(|pair| pair[1].decision < pair[0].decision)
        {
            return Err(ResumeError::DecisionsOutOfOrder {
                sequence: pair[1].sequence,
            });
        }
        let rngs = GameRngs::resume(config.seed, &rng_positions);
        let mut runner = Self {
            state: GameState::default(),
            log: EventLog::default(),
            decisions: DecisionLog::default(),
            rngs,
            players,
            requested_at: None,
            end_logged: false,
            next_decision: 0,
            undo_stack: Vec::new(),
            blocked_by: UndoError::NothingSinceTurnStart,
        };
        let mut group: Vec<Event> = Vec::new();
        let mut current = entries[0].decision;
        for entry in entries {
            if entry.decision != current {
                runner.next_decision = current;
                runner.record(std::mem::take(&mut group));
                current = entry.decision;
            }
            group.push(entry.event.clone());
        }
        runner.next_decision = current;
        runner.record(group);
        ::log::info!(
            "partie reprise : {} événement(s) rejoué(s)",
            runner.log.len()
        );
        Ok(runner)
    }

    /// Avance d'au plus une décision, sans jamais attendre le joueur.
    pub fn step(&mut self) -> Result<StepOutcome, RunnerError> {
        let result = self.advance();
        if let Err(error) = &result {
            ::log::error!("erreur de la boucle de partie : {error}");
        }
        result
    }

    fn advance(&mut self) -> Result<StepOutcome, RunnerError> {
        let Some(decision) = pending_decision(&self.state) else {
            if !self.end_logged {
                self.end_logged = true;
                ::log::info!(
                    "partie terminée : {} événement(s) au journal",
                    self.log.len()
                );
            }
            return Ok(StepOutcome::Finished);
        };
        let player = decision.player;
        let view = view(&self.state, player);
        let legal = legal_actions(&self.state, player);
        let seat = self
            .players
            .get_mut(player.index())
            .ok_or(RunnerError::MissingPlayer { player })?;

        // Une seule demande par décision, même si le joueur met plusieurs pas à répondre.
        if self.requested_at != Some(self.log.len()) {
            ::log::debug!(
                "décision demandée au joueur {player:?} : {} action(s) légale(s)",
                legal.len()
            );
            seat.request(&view, &legal);
            self.requested_at = Some(self.log.len());
        }
        let Some(index) = seat.poll() else {
            return Ok(StepOutcome::Waiting { player });
        };
        let action = legal.get(index).ok_or(RunnerError::ChoiceOutOfRange {
            index,
            count: legal.len(),
        })?;

        ::log::debug!("le joueur {player:?} choisit {action:?}");
        let events = decide(&self.state, player, action, &mut self.rngs)?;
        self.record(events);
        self.decisions.push(DecisionRecord {
            player,
            view,
            legal_actions: legal,
            chosen: index,
        });
        Ok(StepOutcome::Advanced)
    }

    /// Applique et journalise les événements d'une décision, sous le numéro `next_decision`.
    fn record(&mut self, events: Vec<Event>) {
        let before = self.state.clone();
        let blocks = events.iter().any(Event::blocks_undo);
        let closes_turn = events.iter().any(Event::closes_turn);
        for event in events {
            // Peut révéler de l'information cachée : jamais au-dessus de trace (spec 003).
            ::log::trace!("événement n°{} : {event:?}", self.log.len());
            self.state = apply(std::mem::take(&mut self.state), &event);
            self.log.push(self.next_decision, event);
        }
        self.next_decision += 1;
        if blocks {
            self.undo_stack.clear();
            self.blocked_by = if closes_turn {
                UndoError::NothingSinceTurnStart
            } else {
                UndoError::InformationRevealed
            };
        } else {
            self.undo_stack.push(before);
        }
    }

    /// Peut-on revenir en arrière maintenant (spec 024) ?
    pub fn undo_status(&self) -> UndoStatus {
        match (self.undo_stack.is_empty(), self.blocked_by) {
            (false, _) => UndoStatus::Possible,
            (true, UndoError::NothingSinceTurnStart) => UndoStatus::NothingSinceTurnStart,
            (true, UndoError::InformationRevealed) => UndoStatus::InformationRevealed,
        }
    }

    /// Reprend la dernière décision entière (MKUE p. 5, Reverting) : état, journal et journal des
    /// décisions reviennent à ce qu'ils étaient avant elle. Ce n'est pas une action du jeu.
    pub fn undo(&mut self) -> Result<(), UndoError> {
        let Some(before) = self.undo_stack.pop() else {
            return Err(self.blocked_by);
        };
        self.next_decision -= 1;
        self.log.truncate_from(self.next_decision);
        // Après une reprise de partie, les décisions d'avant ne sont pas dans ce journal-là.
        self.decisions.pop();
        self.state = before;
        self.requested_at = None;
        ::log::debug!(
            "retour en arrière : décision n°{} reprise",
            self.next_decision
        );
        Ok(())
    }

    /// Issue privée 307 : reprend l'interaction qui vient de commencer, si rien n'a été fait depuis, pour
    /// pouvoir encore se déplacer (MKUE p. 7, 1 : le mouvement précède l'action). Comme `undo`, ce
    /// n'est pas une action du jeu. Renvoie `true` si l'interaction a été reprise.
    pub fn cancel_untouched_interaction(&mut self) -> bool {
        // La dernière décision n'a produit que le début de l'interaction.
        let last = self.next_decision.checked_sub(1);
        let mut events = self
            .log
            .entries()
            .iter()
            .rev()
            .take_while(|entry| Some(entry.decision) == last)
            .peekable();
        let untouched = events.peek().is_some()
            && events.all(|entry| matches!(entry.event, Event::InteractionStarted { .. }));
        untouched && self.undo().is_ok()
    }

    /// Positions des générateurs : une partie reprise à ces positions tire le même hasard.
    pub fn rng_positions(&self) -> RngPositions {
        self.rngs.positions()
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn log(&self) -> &EventLog {
        &self.log
    }

    pub fn decision_log(&self) -> &DecisionLog {
        &self.decisions
    }
}
