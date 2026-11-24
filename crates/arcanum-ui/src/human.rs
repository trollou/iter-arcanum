use std::cell::RefCell;
use std::rc::Rc;

use arcanum_core::{Action, Player, PlayerView};

/// Demande en cours adressée au joueur humain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanRequest {
    pub view: PlayerView,
    pub legal_actions: Vec<Action>,
}

/// Ce que partagent les deux moitiés : la demande en cours et le choix déposé.
#[derive(Default)]
struct Mailbox {
    request: Option<HumanRequest>,
    choice: Option<usize>,
}

/// Moitié `Player` du joueur humain, confiée au `GameRunner`.
pub struct HumanPlayer {
    mailbox: Rc<RefCell<Mailbox>>,
}

/// Poignée gardée par l'UI : voit la demande en cours et y dépose le choix.
#[derive(Clone)]
pub struct HumanHandle {
    mailbox: Rc<RefCell<Mailbox>>,
}

/// Crée les deux moitiés du joueur humain, qui partagent la même demande.
/// Même thread que l'UI : `Rc<RefCell>` suffit, sans verrou.
pub fn human_player() -> (HumanPlayer, HumanHandle) {
    let mailbox = Rc::new(RefCell::new(Mailbox::default()));
    (
        HumanPlayer {
            mailbox: Rc::clone(&mailbox),
        },
        HumanHandle { mailbox },
    )
}

impl HumanHandle {
    pub fn pending(&self) -> Option<HumanRequest> {
        self.mailbox.borrow().request.clone()
    }

    /// Ignoré sans demande en cours, si un choix est déjà déposé, ou si l'index est hors des actions légales.
    pub fn submit(&self, index: usize) {
        let mut mailbox = self.mailbox.borrow_mut();
        let valid = mailbox
            .request
            .as_ref()
            .is_some_and(|request| index < request.legal_actions.len());
        if valid && mailbox.choice.is_none() {
            mailbox.choice = Some(index);
        }
    }
}

impl Player for HumanPlayer {
    fn request(&mut self, view: &PlayerView, legal_actions: &[Action]) {
        *self.mailbox.borrow_mut() = Mailbox {
            request: Some(HumanRequest {
                view: view.clone(),
                legal_actions: legal_actions.to_vec(),
            }),
            choice: None,
        };
    }

    fn poll(&mut self) -> Option<usize> {
        let mut mailbox = self.mailbox.borrow_mut();
        let choice = mailbox.choice.take()?;
        mailbox.request = None;
        Some(choice)
    }
}
