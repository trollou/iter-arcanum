use crate::{Action, PlayerView};

/// Un joueur, quel qu'il soit (humain, règles, MCTS, LLM).
/// Non bloquant : la boucle demande, puis interroge jusqu'à obtenir un choix.
pub trait Player {
    /// Une décision attend ce joueur.
    fn request(&mut self, view: &PlayerView, legal_actions: &[Action]);

    /// `Some(index)` dans la liste des actions légales quand le choix est fait.
    fn poll(&mut self) -> Option<usize>;
}
