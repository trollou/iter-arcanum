use arcanum_core::{GameRunner, RunnerError, StepOutcome};

/// Limite de pas par image, pour ne jamais geler la fenêtre.
pub const MAX_STEPS_PER_FRAME: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameOutcome {
    Waiting,
    Finished,
    /// La limite est atteinte : il faut une autre image pour continuer.
    StepLimitReached,
    Error(RunnerError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameReport {
    pub steps: usize,
    pub outcome: FrameOutcome,
}

/// Fait avancer la partie pendant une image, sans jamais attendre un joueur.
pub fn run_frame(runner: &mut GameRunner) -> FrameReport {
    run_frame_limited(runner, MAX_STEPS_PER_FRAME)
}

/// Comme `run_frame`, avec une limite donnée : une partie réelle s'arrête bien avant
/// `MAX_STEPS_PER_FRAME` pas, les tests de la limite en prennent une plus petite (spec 013).
pub fn run_frame_limited(runner: &mut GameRunner, max_steps: usize) -> FrameReport {
    for steps in 0..max_steps {
        let outcome = match runner.step() {
            Ok(StepOutcome::Advanced) => continue,
            Ok(StepOutcome::Waiting { .. }) => FrameOutcome::Waiting,
            Ok(StepOutcome::Finished) => FrameOutcome::Finished,
            Err(error) => FrameOutcome::Error(error),
        };
        return FrameReport { steps, outcome };
    }
    FrameReport {
        steps: max_steps,
        outcome: FrameOutcome::StepLimitReached,
    }
}
