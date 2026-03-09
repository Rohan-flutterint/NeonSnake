use macroquad::prelude::{Color, IVec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn vector(self) -> IVec2 {
        match self {
            Self::Up => IVec2::new(0, -1),
            Self::Down => IVec2::new(0, 1),
            Self::Left => IVec2::new(-1, 0),
            Self::Right => IVec2::new(1, 0),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Title,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy)]
pub enum AudioCue {
    Key,
    Eat,
    PowerUp,
    Boom,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    Shield,
    Double,
    Slow,
}

impl PowerUpKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shield => "Shield",
            Self::Double => "Double",
            Self::Slow => "Slow",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Shield => "S",
            Self::Double => "x2",
            Self::Slow => "SL",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Shield => Color::new(0.30, 0.72, 1.0, 1.0),
            Self::Double => Color::new(1.0, 0.84, 0.34, 1.0),
            Self::Slow => Color::new(0.48, 0.97, 0.90, 1.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpawnedPowerUp {
    pub kind: PowerUpKind,
    pub position: IVec2,
    pub ttl: f32,
}

#[derive(Clone)]
pub struct HighScoreEntry {
    pub score: u32,
    pub length: usize,
}

pub struct Game {
    pub snake: Vec<IVec2>,
    pub direction: Direction,
    pub queued_direction: Option<Direction>,
    pub food: IVec2,
    pub bombs: Vec<IVec2>,
    pub power_up: Option<SpawnedPowerUp>,
    pub phase: Phase,
    pub score: u32,
    pub best_score: u32,
    pub rounds_played: u32,
    pub foods_eaten: u32,
    pub shield_active: bool,
    pub multiplier_timer: f32,
    pub slow_timer: f32,
    pub high_scores: Vec<HighScoreEntry>,
    pub score_recorded: bool,
    pub step_timer: f32,
    pub step_delay: f32,
    pub food_flash: f32,
    pub death_flash: f32,
}
