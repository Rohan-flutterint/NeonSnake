use macroquad::prelude::{Color, IVec2, Vec2};

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
    ChallengeClear,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Arcade,
    Challenge,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Arcade => "Arcade",
            Self::Challenge => "Challenge",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Arcade => Self::Challenge,
            Self::Challenge => Self::Arcade,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChallengeKind {
    Survive60,
    Collect20,
    NoWallHits,
}

impl ChallengeKind {
    pub fn for_round(round_number: u32) -> Self {
        match round_number.saturating_sub(1) as usize % 3 {
            0 => Self::Survive60,
            1 => Self::Collect20,
            _ => Self::NoWallHits,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Survive60 => "Survive 60",
            Self::Collect20 => "Collect 20",
            Self::NoWallHits => "No Wall Hits",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Survive60 => "Stay alive for 60 seconds.",
            Self::Collect20 => "Collect 20 food in one run.",
            Self::NoWallHits => "Reach 120 score without a wall hit.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelTheme {
    Afterglow,
    Voltage,
    Overdrive,
    Singularity,
}

impl LevelTheme {
    pub fn for_score(score: u32) -> Self {
        match score {
            0..=79 => Self::Afterglow,
            80..=179 => Self::Voltage,
            180..=319 => Self::Overdrive,
            _ => Self::Singularity,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Afterglow => "Afterglow",
            Self::Voltage => "Voltage",
            Self::Overdrive => "Overdrive",
            Self::Singularity => "Singularity",
        }
    }

    pub fn next_threshold(self) -> Option<u32> {
        match self {
            Self::Afterglow => Some(80),
            Self::Voltage => Some(180),
            Self::Overdrive => Some(320),
            Self::Singularity => None,
        }
    }

    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Afterglow => 1.00,
            Self::Voltage => 0.96,
            Self::Overdrive => 0.92,
            Self::Singularity => 0.88,
        }
    }

    pub fn music_volume(self) -> f32 {
        match self {
            Self::Afterglow => 0.20,
            Self::Voltage => 0.24,
            Self::Overdrive => 0.28,
            Self::Singularity => 0.33,
        }
    }

    pub fn sfx_gain(self) -> f32 {
        match self {
            Self::Afterglow => 1.00,
            Self::Voltage => 1.07,
            Self::Overdrive => 1.14,
            Self::Singularity => 1.22,
        }
    }

    pub fn visual_intensity(self) -> f32 {
        match self {
            Self::Afterglow => 0.00,
            Self::Voltage => 1.00,
            Self::Overdrive => 2.00,
            Self::Singularity => 3.00,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HazardPattern {
    SplitGates,
    DiamondRun,
    ReactorLadder,
    CornerHooks,
    Pinwheel,
}

impl HazardPattern {
    pub fn for_round(round_number: u32) -> Self {
        match round_number.saturating_sub(1) as usize % 5 {
            0 => Self::SplitGates,
            1 => Self::DiamondRun,
            2 => Self::ReactorLadder,
            3 => Self::CornerHooks,
            _ => Self::Pinwheel,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::SplitGates => "Split Gates",
            Self::DiamondRun => "Diamond Run",
            Self::ReactorLadder => "Reactor Ladder",
            Self::CornerHooks => "Corner Hooks",
            Self::Pinwheel => "Pinwheel",
        }
    }
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

#[derive(Clone, Copy)]
pub enum ParticleShape {
    Dot,
    Shard,
    Ring,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: f32,
    pub max_ttl: f32,
    pub size: f32,
    pub color: Color,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub shape: ParticleShape,
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
    pub hazard_pattern: HazardPattern,
    pub hazard_layout: Vec<IVec2>,
    pub mode: GameMode,
    pub challenge: ChallengeKind,
    pub power_up: Option<SpawnedPowerUp>,
    pub particles: Vec<Particle>,
    pub phase: Phase,
    pub score: u32,
    pub best_score: u32,
    pub rounds_played: u32,
    pub foods_eaten: u32,
    pub survival_time: f32,
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
