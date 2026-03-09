use macroquad::prelude::*;

use super::{
    AudioCue, Direction, GRID_SIZE, Game, HazardPattern, HighScoreEntry, LevelTheme, Particle,
    ParticleShape, Phase, PowerUpKind, START_LENGTH, SpawnedPowerUp, initial_bomb_count,
    speed_for_score, target_bomb_count,
};
use crate::storage::{load_high_scores, register_high_score, save_high_scores};

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            snake: Vec::new(),
            direction: Direction::Right,
            queued_direction: None,
            food: ivec2(0, 0),
            bombs: Vec::new(),
            hazard_pattern: HazardPattern::SplitGates,
            hazard_layout: Vec::new(),
            power_up: None,
            particles: Vec::new(),
            phase: Phase::Title,
            score: 0,
            best_score: 0,
            rounds_played: 0,
            foods_eaten: 0,
            shield_active: false,
            multiplier_timer: 0.0,
            slow_timer: 0.0,
            high_scores: load_high_scores(),
            score_recorded: false,
            step_timer: 0.0,
            step_delay: super::BASE_STEP_DELAY,
            food_flash: 0.0,
            death_flash: 0.0,
        };
        game.best_score = game
            .high_scores
            .first()
            .map(|entry| entry.score)
            .unwrap_or(0);
        game.reset_round();
        game.phase = Phase::Title;
        game
    }

    pub fn reset_round(&mut self) {
        let center = ivec2(GRID_SIZE / 2, GRID_SIZE / 2);
        self.snake = (0..START_LENGTH)
            .map(|offset| center - ivec2(offset, 0))
            .collect();
        self.direction = Direction::Right;
        self.queued_direction = None;
        self.score = 0;
        self.foods_eaten = 0;
        self.shield_active = false;
        self.multiplier_timer = 0.0;
        self.slow_timer = 0.0;
        self.score_recorded = false;
        self.step_timer = 0.0;
        self.step_delay = super::BASE_STEP_DELAY;
        self.food_flash = 0.0;
        self.death_flash = 0.0;
        self.power_up = None;
        self.particles.clear();
        let round_number = self.rounds_played + 1;
        self.hazard_pattern = HazardPattern::for_round(round_number);
        self.hazard_layout = build_hazard_layout(self.hazard_pattern, round_number);
        self.bombs = self.compose_bombs(initial_bomb_count(self.score));
        self.food = self.spawn_food();
    }

    pub fn start_round(&mut self) {
        self.reset_round();
        self.phase = Phase::Playing;
        self.rounds_played += 1;
    }

    pub fn configure_showcase_playing(&mut self) {
        self.phase = Phase::Playing;
        self.rounds_played = self.rounds_played.max(3);
        self.direction = Direction::Right;
        self.queued_direction = None;
        self.score = 120;
        self.best_score = 240;
        self.foods_eaten = 12;
        self.shield_active = true;
        self.multiplier_timer = 6.4;
        self.slow_timer = 0.0;
        self.step_delay = speed_for_score(self.score);
        self.step_timer = 0.0;
        self.food_flash = 0.24;
        self.death_flash = 0.0;
        self.snake = vec![
            ivec2(15, 10),
            ivec2(14, 10),
            ivec2(13, 10),
            ivec2(12, 10),
            ivec2(11, 10),
            ivec2(11, 11),
            ivec2(11, 12),
            ivec2(12, 12),
            ivec2(13, 12),
            ivec2(14, 12),
            ivec2(14, 13),
            ivec2(14, 14),
        ];
        self.food = ivec2(18, 8);
        self.bombs = vec![ivec2(7, 7), ivec2(18, 16), ivec2(8, 17)];
        self.hazard_pattern = HazardPattern::DiamondRun;
        self.hazard_layout = self.bombs.clone();
        self.power_up = Some(SpawnedPowerUp {
            kind: PowerUpKind::Slow,
            position: ivec2(5, 13),
            ttl: 7.0,
        });
        self.particles = vec![
            Particle {
                position: vec2(18.5, 8.5),
                velocity: vec2(0.42, -0.32),
                ttl: 0.7,
                max_ttl: 0.7,
                size: 0.28,
                color: Color::new(1.0, 0.78, 0.22, 0.9),
                rotation: 0.0,
                angular_velocity: 1.2,
                shape: ParticleShape::Ring,
            },
            Particle {
                position: vec2(18.5, 8.5),
                velocity: vec2(-0.7, -0.18),
                ttl: 0.6,
                max_ttl: 0.6,
                size: 0.18,
                color: Color::new(1.0, 0.52, 0.24, 1.0),
                rotation: 0.6,
                angular_velocity: 7.0,
                shape: ParticleShape::Shard,
            },
        ];
    }

    pub fn configure_showcase_game_over(&mut self) {
        self.configure_showcase_playing();
        self.phase = Phase::GameOver;
        self.score = 190;
        self.best_score = 260;
        self.foods_eaten = 19;
        self.shield_active = false;
        self.multiplier_timer = 0.0;
        self.slow_timer = 0.0;
        self.death_flash = 0.25;
        self.food_flash = 0.0;
        self.direction = Direction::Up;
        self.snake = vec![
            ivec2(9, 3),
            ivec2(9, 4),
            ivec2(9, 5),
            ivec2(9, 6),
            ivec2(10, 6),
            ivec2(11, 6),
            ivec2(11, 5),
            ivec2(11, 4),
            ivec2(10, 4),
            ivec2(10, 5),
        ];
        self.food = ivec2(15, 14);
        self.bombs = vec![ivec2(12, 10), ivec2(16, 7), ivec2(5, 15)];
        self.hazard_pattern = HazardPattern::CornerHooks;
        self.hazard_layout = self.bombs.clone();
        self.power_up = None;
        self.particles = vec![
            Particle {
                position: vec2(9.5, 3.5),
                velocity: vec2(0.0, 0.0),
                ttl: 0.9,
                max_ttl: 0.9,
                size: 0.62,
                color: Color::new(1.0, 0.18, 0.22, 0.78),
                rotation: 0.0,
                angular_velocity: 0.0,
                shape: ParticleShape::Ring,
            },
            Particle {
                position: vec2(9.5, 3.5),
                velocity: vec2(0.9, 0.55),
                ttl: 0.7,
                max_ttl: 0.7,
                size: 0.16,
                color: Color::new(1.0, 0.42, 0.22, 0.95),
                rotation: 0.3,
                angular_velocity: 8.0,
                shape: ParticleShape::Shard,
            },
        ];
    }

    pub fn effective_step_delay(&self) -> f32 {
        let themed_delay = self.step_delay * self.level_theme().speed_multiplier();
        if self.slow_timer > 0.0 {
            (themed_delay * 1.55).min(0.24)
        } else {
            themed_delay
        }
    }

    pub fn level_theme(&self) -> LevelTheme {
        LevelTheme::for_score(self.score)
    }

    pub fn next_theme_score(&self) -> Option<u32> {
        self.level_theme().next_threshold()
    }

    pub fn hazard_pattern_name(&self) -> &'static str {
        self.hazard_pattern.name()
    }

    pub fn active_power_up_labels(&self) -> Vec<String> {
        let mut labels = Vec::new();
        if self.shield_active {
            labels.push("Shield ready".to_owned());
        }
        if self.multiplier_timer > 0.0 {
            labels.push(format!("Double {:.1}s", self.multiplier_timer));
        }
        if self.slow_timer > 0.0 {
            labels.push(format!("Slow {:.1}s", self.slow_timer));
        }
        labels
    }

    pub fn handle_input(&mut self) -> Vec<AudioCue> {
        let mut cues = Vec::new();

        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.phase {
                Phase::Title | Phase::GameOver => self.start_round(),
                Phase::Paused => self.phase = Phase::Playing,
                Phase::Playing => {}
            }
            cues.push(AudioCue::Key);
        }

        if is_key_pressed(KeyCode::Escape) {
            match self.phase {
                Phase::Playing => self.phase = Phase::Paused,
                Phase::Paused => self.phase = Phase::Playing,
                Phase::Title | Phase::GameOver => {}
            }
            cues.push(AudioCue::Key);
        }

        if is_key_pressed(KeyCode::R) {
            self.start_round();
            cues.push(AudioCue::Key);
        }

        if let Some(next_direction) = read_direction_input() {
            match self.phase {
                Phase::Playing => {
                    if self.queue_direction(next_direction) {
                        cues.push(AudioCue::Key);
                    }
                }
                Phase::Title | Phase::GameOver => {
                    self.start_round();
                    if self.queue_direction(next_direction) {
                        cues.push(AudioCue::Key);
                    } else {
                        cues.push(AudioCue::Key);
                    }
                }
                Phase::Paused => {}
            }
        }

        cues
    }

    pub fn update(&mut self, dt: f32) -> Vec<AudioCue> {
        let mut cues = Vec::new();
        self.food_flash = (self.food_flash - dt).max(0.0);
        self.death_flash = (self.death_flash - dt).max(0.0);
        self.update_particles(dt);

        if self.phase != Phase::Playing {
            return cues;
        }

        self.multiplier_timer = (self.multiplier_timer - dt).max(0.0);
        self.slow_timer = (self.slow_timer - dt).max(0.0);
        if let Some(power_up) = self.power_up.as_mut() {
            power_up.ttl -= dt;
            if power_up.ttl <= 0.0 {
                self.power_up = None;
            }
        }

        self.step_timer += dt;
        let effective_delay = self.effective_step_delay();
        while self.step_timer >= effective_delay {
            self.step_timer -= effective_delay;
            if let Some(cue) = self.advance() {
                cues.push(cue);
                if matches!(cue, AudioCue::GameOver) {
                    break;
                }
            }
            if self.phase != Phase::Playing {
                break;
            }
        }

        cues
    }

    fn spawn_food(&self) -> IVec2 {
        loop {
            let candidate = ivec2(
                macroquad::rand::gen_range(0, GRID_SIZE),
                macroquad::rand::gen_range(0, GRID_SIZE),
            );
            if !self.snake.contains(&candidate)
                && !self.bombs.contains(&candidate)
                && self
                    .power_up
                    .map(|item| item.position != candidate)
                    .unwrap_or(true)
            {
                return candidate;
            }
        }
    }

    fn compose_bombs(&self, count: usize) -> Vec<IVec2> {
        let mut bombs = Vec::with_capacity(count);

        for slot in &self.hazard_layout {
            if bombs.len() == count {
                break;
            }
            if self.is_valid_bomb_cell(*slot, &bombs) {
                bombs.push(*slot);
            }
        }

        while bombs.len() < count {
            if let Some(candidate) = self.spawn_random_bomb(&bombs) {
                bombs.push(candidate);
            } else {
                break;
            }
        }

        bombs
    }

    fn spawn_random_bomb(&self, existing: &[IVec2]) -> Option<IVec2> {
        for _ in 0..256 {
            let candidate = ivec2(
                macroquad::rand::gen_range(0, GRID_SIZE),
                macroquad::rand::gen_range(0, GRID_SIZE),
            );
            if self.is_valid_bomb_cell(candidate, existing) {
                return Some(candidate);
            }
        }

        None
    }

    fn is_valid_bomb_cell(&self, candidate: IVec2, existing: &[IVec2]) -> bool {
        !self.snake.contains(&candidate)
            && candidate != self.food
            && !existing.contains(&candidate)
            && self
                .power_up
                .map(|item| item.position != candidate)
                .unwrap_or(true)
    }

    fn spawn_power_up(&self) -> Option<SpawnedPowerUp> {
        for _ in 0..128 {
            let candidate = ivec2(
                macroquad::rand::gen_range(0, GRID_SIZE),
                macroquad::rand::gen_range(0, GRID_SIZE),
            );
            if self.snake.contains(&candidate)
                || self.bombs.contains(&candidate)
                || candidate == self.food
            {
                continue;
            }

            let kind = match macroquad::rand::gen_range(0, 3) {
                0 => PowerUpKind::Shield,
                1 => PowerUpKind::Double,
                _ => PowerUpKind::Slow,
            };

            return Some(SpawnedPowerUp {
                kind,
                position: candidate,
                ttl: 10.0,
            });
        }

        None
    }

    fn sync_bombs(&mut self) {
        let target = target_bomb_count(self.score);
        self.bombs = self.compose_bombs(target);
    }

    fn maybe_spawn_power_up(&mut self) {
        if self.power_up.is_none() && self.foods_eaten > 0 && self.foods_eaten % 3 == 0 {
            self.power_up = self.spawn_power_up();
        }
    }

    fn apply_power_up(&mut self, kind: PowerUpKind) {
        match kind {
            PowerUpKind::Shield => self.shield_active = true,
            PowerUpKind::Double => self.multiplier_timer = 9.0,
            PowerUpKind::Slow => self.slow_timer = 7.0,
        }
    }

    fn record_score(&mut self) {
        if self.score_recorded || self.score == 0 {
            return;
        }

        register_high_score(
            &mut self.high_scores,
            HighScoreEntry {
                score: self.score,
                length: self.snake.len(),
            },
        );
        self.best_score = self
            .high_scores
            .first()
            .map(|entry| entry.score)
            .unwrap_or(0);
        let _ = save_high_scores(&self.high_scores);
        self.score_recorded = true;
    }

    fn queue_direction(&mut self, next_direction: Direction) -> bool {
        let reference = self.queued_direction.unwrap_or(self.direction);
        if next_direction != reference && next_direction != reference.opposite() {
            self.queued_direction = Some(next_direction);
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<AudioCue> {
        if let Some(next_direction) = self.queued_direction.take() {
            if next_direction != self.direction.opposite() {
                self.direction = next_direction;
            }
        }

        let next_head = self.snake[0] + self.direction.vector();
        let will_grow = next_head == self.food;
        let hit_bomb = self.bombs.contains(&next_head);
        let picked_power_up = self
            .power_up
            .as_ref()
            .filter(|item| item.position == next_head)
            .map(|item| item.kind);
        let body_to_check = if will_grow {
            self.snake.len()
        } else {
            self.snake.len().saturating_sub(1)
        };
        let collides_with_self = self
            .snake
            .iter()
            .take(body_to_check)
            .any(|&part| part == next_head);
        let out_of_bounds = next_head.x < 0
            || next_head.y < 0
            || next_head.x >= GRID_SIZE
            || next_head.y >= GRID_SIZE;

        if out_of_bounds || collides_with_self {
            self.spawn_crash_particles(next_head, false);
            self.record_score();
            self.phase = Phase::GameOver;
            self.death_flash = 0.6;
            return Some(AudioCue::GameOver);
        }

        if hit_bomb {
            self.spawn_crash_particles(next_head, true);
            if self.shield_active {
                self.shield_active = false;
                self.bombs.retain(|&bomb| bomb != next_head);
            } else {
                self.record_score();
                self.phase = Phase::GameOver;
                self.death_flash = 0.8;
                return Some(AudioCue::Boom);
            }
        }

        self.snake.insert(0, next_head);

        if will_grow {
            self.spawn_food_particles(next_head);
            let gained = if self.multiplier_timer > 0.0 { 20 } else { 10 };
            self.foods_eaten += 1;
            self.score += gained;
            self.best_score = self.best_score.max(self.score);
            self.step_delay = speed_for_score(self.score);
            self.food_flash = 0.3;
            self.food = self.spawn_food();
            self.sync_bombs();
            self.maybe_spawn_power_up();
            return Some(AudioCue::Eat);
        } else {
            self.snake.pop();
        }

        if let Some(kind) = picked_power_up {
            self.power_up = None;
            self.apply_power_up(kind);
            return Some(AudioCue::PowerUp);
        }

        None
    }

    fn update_particles(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.position += particle.velocity * dt;
            particle.velocity *= 1.0 - (dt * 1.6).min(0.45);
            particle.rotation += particle.angular_velocity * dt;
            particle.ttl -= dt;
        }
        self.particles.retain(|particle| particle.ttl > 0.0);
    }

    fn spawn_food_particles(&mut self, cell: IVec2) {
        let origin = grid_center(cell);
        self.particles.push(Particle {
            position: origin,
            velocity: vec2(0.0, 0.0),
            ttl: 0.55,
            max_ttl: 0.55,
            size: 0.32,
            color: Color::new(1.0, 0.78, 0.26, 0.88),
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: ParticleShape::Ring,
        });

        for _ in 0..10 {
            let angle = macroquad::rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = macroquad::rand::gen_range(1.6, 4.4);
            let color = if macroquad::rand::gen_range(0, 2) == 0 {
                Color::new(1.0, 0.50, 0.24, 0.98)
            } else {
                Color::new(1.0, 0.90, 0.38, 0.95)
            };
            self.particles.push(Particle {
                position: origin,
                velocity: vec2(angle.cos(), angle.sin()) * speed,
                ttl: macroquad::rand::gen_range(0.28, 0.52),
                max_ttl: 0.52,
                size: macroquad::rand::gen_range(0.10, 0.18),
                color,
                rotation: angle,
                angular_velocity: macroquad::rand::gen_range(-8.0, 8.0),
                shape: if macroquad::rand::gen_range(0, 3) == 0 {
                    ParticleShape::Shard
                } else {
                    ParticleShape::Dot
                },
            });
        }
    }

    fn spawn_crash_particles(&mut self, cell: IVec2, bomb_hit: bool) {
        let origin = clamped_grid_center(cell);
        let (ring_color, shard_a, shard_b, count) = if bomb_hit {
            (
                Color::new(1.0, 0.24, 0.18, 0.90),
                Color::new(1.0, 0.76, 0.28, 0.96),
                Color::new(1.0, 0.30, 0.16, 0.96),
                18,
            )
        } else {
            (
                Color::new(1.0, 0.16, 0.24, 0.76),
                Color::new(1.0, 0.54, 0.26, 0.92),
                Color::new(1.0, 0.20, 0.36, 0.92),
                12,
            )
        };

        self.particles.push(Particle {
            position: origin,
            velocity: vec2(0.0, 0.0),
            ttl: 0.82,
            max_ttl: 0.82,
            size: if bomb_hit { 0.72 } else { 0.58 },
            color: ring_color,
            rotation: 0.0,
            angular_velocity: 0.0,
            shape: ParticleShape::Ring,
        });

        for _ in 0..count {
            let angle = macroquad::rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = if bomb_hit {
                macroquad::rand::gen_range(2.4, 6.6)
            } else {
                macroquad::rand::gen_range(1.6, 4.8)
            };
            let color = if macroquad::rand::gen_range(0, 2) == 0 {
                shard_a
            } else {
                shard_b
            };
            self.particles.push(Particle {
                position: origin,
                velocity: vec2(angle.cos(), angle.sin()) * speed,
                ttl: macroquad::rand::gen_range(0.38, 0.88),
                max_ttl: 0.88,
                size: macroquad::rand::gen_range(0.12, 0.22),
                color,
                rotation: angle,
                angular_velocity: macroquad::rand::gen_range(-12.0, 12.0),
                shape: ParticleShape::Shard,
            });
        }
    }
}

fn read_direction_input() -> Option<Direction> {
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        Some(Direction::Up)
    } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        Some(Direction::Down)
    } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        Some(Direction::Left)
    } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        Some(Direction::Right)
    } else {
        None
    }
}

fn build_hazard_layout(pattern: HazardPattern, round_number: u32) -> Vec<IVec2> {
    let center = ivec2(GRID_SIZE / 2, GRID_SIZE / 2);
    let rotation = (round_number.saturating_sub(1) % 4) as i32;
    let offsets = match pattern {
        HazardPattern::SplitGates => vec![
            ivec2(-7, -2),
            ivec2(-7, -1),
            ivec2(-7, 0),
            ivec2(7, -2),
            ivec2(7, -1),
            ivec2(7, 0),
        ],
        HazardPattern::DiamondRun => vec![
            ivec2(0, -7),
            ivec2(-1, -6),
            ivec2(1, -6),
            ivec2(-2, -5),
            ivec2(2, -5),
            ivec2(0, -4),
        ],
        HazardPattern::ReactorLadder => vec![
            ivec2(-5, -8),
            ivec2(-5, -6),
            ivec2(-5, -4),
            ivec2(-3, -8),
            ivec2(-3, -6),
            ivec2(-3, -4),
        ],
        HazardPattern::CornerHooks => vec![
            ivec2(-10, -8),
            ivec2(-9, -8),
            ivec2(-9, -7),
            ivec2(10, 8),
            ivec2(9, 8),
            ivec2(9, 7),
        ],
        HazardPattern::Pinwheel => vec![
            ivec2(-8, 1),
            ivec2(-7, 1),
            ivec2(-7, 2),
            ivec2(1, -8),
            ivec2(1, -7),
            ivec2(2, -7),
        ],
    };

    offsets
        .into_iter()
        .map(|offset| rotate_offset(offset, rotation) + center)
        .filter(|cell| cell.x >= 0 && cell.y >= 0 && cell.x < GRID_SIZE && cell.y < GRID_SIZE)
        .collect()
}

fn rotate_offset(offset: IVec2, turns: i32) -> IVec2 {
    match turns.rem_euclid(4) {
        0 => offset,
        1 => ivec2(-offset.y, offset.x),
        2 => ivec2(-offset.x, -offset.y),
        _ => ivec2(offset.y, -offset.x),
    }
}

fn grid_center(cell: IVec2) -> Vec2 {
    vec2(cell.x as f32 + 0.5, cell.y as f32 + 0.5)
}

fn clamped_grid_center(cell: IVec2) -> Vec2 {
    let max_edge = GRID_SIZE as f32 - 0.35;
    vec2(
        (cell.x as f32 + 0.5).clamp(0.35, max_edge),
        (cell.y as f32 + 0.5).clamp(0.35, max_edge),
    )
}
