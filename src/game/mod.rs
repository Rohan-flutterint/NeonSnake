mod state;
mod types;

pub use types::{
    AudioCue, Direction, Game, HazardPattern, HighScoreEntry, LevelTheme, Particle, ParticleShape,
    Phase, PowerUpKind, SpawnedPowerUp,
};

pub const GRID_SIZE: i32 = 24;
pub const START_LENGTH: i32 = 5;
pub const BASE_STEP_DELAY: f32 = 0.16;
pub const MIN_STEP_DELAY: f32 = 0.07;
pub const BOARD_PADDING: f32 = 18.0;

pub(crate) fn speed_for_score(score: u32) -> f32 {
    let food_eaten = score as f32 / 10.0;
    (BASE_STEP_DELAY - food_eaten * 0.006).max(MIN_STEP_DELAY)
}

pub(crate) fn initial_bomb_count(score: u32) -> usize {
    target_bomb_count(score).max(1)
}

pub(crate) fn target_bomb_count(score: u32) -> usize {
    (1 + (score / 40) as usize).min(5)
}
