use macroquad::{
    audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound},
    prelude::*,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const GRID_SIZE: i32 = 24;
const START_LENGTH: i32 = 5;
const BASE_STEP_DELAY: f32 = 0.16;
const MIN_STEP_DELAY: f32 = 0.07;
const BOARD_PADDING: f32 = 18.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "NeonSnake".to_owned(),
        window_width: 1280,
        window_height: 820,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn vector(self) -> IVec2 {
        match self {
            Self::Up => ivec2(0, -1),
            Self::Down => ivec2(0, 1),
            Self::Left => ivec2(-1, 0),
            Self::Right => ivec2(1, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Title,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy)]
enum AudioCue {
    Key,
    Eat,
    Boom,
    GameOver,
}

struct Layout {
    board: Rect,
    grid: Rect,
    panel: Rect,
}

#[derive(Clone)]
struct HighScoreEntry {
    score: u32,
    length: usize,
}

struct Game {
    snake: Vec<IVec2>,
    direction: Direction,
    queued_direction: Option<Direction>,
    food: IVec2,
    bombs: Vec<IVec2>,
    phase: Phase,
    score: u32,
    best_score: u32,
    rounds_played: u32,
    high_scores: Vec<HighScoreEntry>,
    score_recorded: bool,
    step_timer: f32,
    step_delay: f32,
    food_flash: f32,
    death_flash: f32,
}

struct SoundBank {
    music: Option<Sound>,
    key: Option<Sound>,
    eat: Option<Sound>,
    boom: Option<Sound>,
    game_over: Option<Sound>,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            snake: Vec::new(),
            direction: Direction::Right,
            queued_direction: None,
            food: ivec2(0, 0),
            bombs: Vec::new(),
            phase: Phase::Title,
            score: 0,
            best_score: 0,
            rounds_played: 0,
            high_scores: load_high_scores(),
            score_recorded: false,
            step_timer: 0.0,
            step_delay: BASE_STEP_DELAY,
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

    fn reset_round(&mut self) {
        let center = ivec2(GRID_SIZE / 2, GRID_SIZE / 2);
        self.snake = (0..START_LENGTH)
            .map(|offset| center - ivec2(offset, 0))
            .collect();
        self.direction = Direction::Right;
        self.queued_direction = None;
        self.score = 0;
        self.score_recorded = false;
        self.step_timer = 0.0;
        self.step_delay = BASE_STEP_DELAY;
        self.food_flash = 0.0;
        self.death_flash = 0.0;
        self.bombs = self.spawn_bombs(initial_bomb_count(self.score));
        self.food = self.spawn_food();
    }

    fn start_round(&mut self) {
        self.reset_round();
        self.phase = Phase::Playing;
        self.rounds_played += 1;
    }

    fn configure_showcase_playing(&mut self) {
        self.phase = Phase::Playing;
        self.rounds_played = self.rounds_played.max(3);
        self.direction = Direction::Right;
        self.queued_direction = None;
        self.score = 120;
        self.best_score = 240;
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
    }

    fn configure_showcase_game_over(&mut self) {
        self.configure_showcase_playing();
        self.phase = Phase::GameOver;
        self.score = 190;
        self.best_score = 260;
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
    }

    fn spawn_food(&self) -> IVec2 {
        loop {
            let candidate = ivec2(
                macroquad::rand::gen_range(0, GRID_SIZE),
                macroquad::rand::gen_range(0, GRID_SIZE),
            );
            if !self.snake.contains(&candidate) && !self.bombs.contains(&candidate) {
                return candidate;
            }
        }
    }

    fn spawn_bombs(&self, count: usize) -> Vec<IVec2> {
        let mut bombs = Vec::with_capacity(count);
        while bombs.len() < count {
            let candidate = ivec2(
                macroquad::rand::gen_range(0, GRID_SIZE),
                macroquad::rand::gen_range(0, GRID_SIZE),
            );
            if !self.snake.contains(&candidate)
                && candidate != self.food
                && !bombs.contains(&candidate)
            {
                bombs.push(candidate);
            }
        }
        bombs
    }

    fn sync_bombs(&mut self) {
        let target = target_bomb_count(self.score);
        if self.bombs.len() < target {
            let mut additions = self.spawn_bombs(target - self.bombs.len());
            self.bombs.append(&mut additions);
        } else if self.bombs.len() > target {
            self.bombs.truncate(target);
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

    fn status_label(&self) -> &'static str {
        match self.phase {
            Phase::Title => "Ready",
            Phase::Playing => "Live",
            Phase::Paused => "Paused",
            Phase::GameOver => "Crashed",
        }
    }

    fn handle_input(&mut self) -> Vec<AudioCue> {
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

    fn queue_direction(&mut self, next_direction: Direction) -> bool {
        let reference = self.queued_direction.unwrap_or(self.direction);
        if next_direction != reference && next_direction != reference.opposite() {
            self.queued_direction = Some(next_direction);
            true
        } else {
            false
        }
    }

    fn update(&mut self, dt: f32) -> Vec<AudioCue> {
        let mut cues = Vec::new();
        self.food_flash = (self.food_flash - dt).max(0.0);
        self.death_flash = (self.death_flash - dt).max(0.0);

        if self.phase != Phase::Playing {
            return cues;
        }

        self.step_timer += dt;
        while self.step_timer >= self.step_delay {
            self.step_timer -= self.step_delay;
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

    fn advance(&mut self) -> Option<AudioCue> {
        if let Some(next_direction) = self.queued_direction.take() {
            if next_direction != self.direction.opposite() {
                self.direction = next_direction;
            }
        }

        let next_head = self.snake[0] + self.direction.vector();
        let will_grow = next_head == self.food;
        let hit_bomb = self.bombs.contains(&next_head);
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
            self.record_score();
            self.phase = Phase::GameOver;
            self.death_flash = 0.6;
            return Some(AudioCue::GameOver);
        }

        if hit_bomb {
            self.record_score();
            self.phase = Phase::GameOver;
            self.death_flash = 0.8;
            return Some(AudioCue::Boom);
        }

        self.snake.insert(0, next_head);

        if will_grow {
            self.score += 10;
            self.best_score = self.best_score.max(self.score);
            self.step_delay = speed_for_score(self.score);
            self.food_flash = 0.3;
            self.food = self.spawn_food();
            self.sync_bombs();
            return Some(AudioCue::Eat);
        } else {
            self.snake.pop();
        }

        None
    }
}

impl SoundBank {
    async fn load() -> Self {
        Self {
            music: load_generated_sound(generate_music_loop()).await,
            key: load_generated_sound(generate_key_sound()).await,
            eat: load_generated_sound(generate_eat_sound()).await,
            boom: load_generated_sound(generate_boom_sound()).await,
            game_over: load_generated_sound(generate_game_over_sound()).await,
        }
    }

    fn start_music(&self) {
        if let Some(sound) = &self.music {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 0.22,
                },
            );
        }
    }

    fn play(&self, cue: AudioCue) {
        match cue {
            AudioCue::Key => {
                if let Some(sound) = &self.key {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.30,
                        },
                    );
                }
            }
            AudioCue::Eat => {
                if let Some(sound) = &self.eat {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.45,
                        },
                    );
                }
            }
            AudioCue::Boom => {
                if let Some(sound) = &self.boom {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.72,
                        },
                    );
                }
            }
            AudioCue::GameOver => {
                if let Some(sound) = &self.game_over {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.60,
                        },
                    );
                }
            }
        }
    }
}

fn speed_for_score(score: u32) -> f32 {
    let food_eaten = score as f32 / 10.0;
    (BASE_STEP_DELAY - food_eaten * 0.006).max(MIN_STEP_DELAY)
}

fn initial_bomb_count(score: u32) -> usize {
    target_bomb_count(score).max(1)
}

fn target_bomb_count(score: u32) -> usize {
    (1 + (score / 40) as usize).min(5)
}

fn score_file_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neonsnake_scores")
}

fn load_high_scores() -> Vec<HighScoreEntry> {
    let Ok(contents) = fs::read_to_string(score_file_path()) else {
        return Vec::new();
    };

    let mut entries = contents
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(',');
            let score = parts.next()?.trim().parse().ok()?;
            let length = parts.next()?.trim().parse().ok()?;
            Some(HighScoreEntry { score, length })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.length.cmp(&a.length)));
    entries.truncate(5);
    entries
}

fn save_high_scores(entries: &[HighScoreEntry]) -> std::io::Result<()> {
    let body = entries
        .iter()
        .map(|entry| format!("{},{}", entry.score, entry.length))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(score_file_path(), format!("{body}\n"))
}

fn register_high_score(entries: &mut Vec<HighScoreEntry>, new_entry: HighScoreEntry) {
    entries.push(new_entry);
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.length.cmp(&a.length)));
    entries.truncate(5);
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

fn layout() -> Layout {
    let sw = screen_width();
    let sh = screen_height();
    let margin = 28.0;

    if sw >= 980.0 {
        let panel_width = (sw * 0.28).clamp(300.0, 360.0);
        let board_size = (sw - panel_width - margin * 3.0)
            .min(sh - margin * 2.0)
            .max(320.0);
        let board_x = margin;
        let board_y = (sh - board_size) * 0.5;
        let panel = Rect::new(
            board_x + board_size + margin,
            margin,
            panel_width,
            sh - margin * 2.0,
        );
        let board = Rect::new(board_x, board_y, board_size, board_size);
        let grid = Rect::new(
            board.x + BOARD_PADDING,
            board.y + BOARD_PADDING,
            board.w - BOARD_PADDING * 2.0,
            board.h - BOARD_PADDING * 2.0,
        );

        Layout { board, grid, panel }
    } else {
        let panel_height = 220.0;
        let board_size = (sw - margin * 2.0)
            .min(sh - panel_height - margin * 3.0)
            .max(240.0);
        let panel = Rect::new(margin, margin, sw - margin * 2.0, panel_height);
        let board = Rect::new(
            (sw - board_size) * 0.5,
            panel.y + panel.h + margin,
            board_size,
            board_size,
        );
        let grid = Rect::new(
            board.x + BOARD_PADDING,
            board.y + BOARD_PADDING,
            board.w - BOARD_PADDING * 2.0,
            board.h - BOARD_PADDING * 2.0,
        );

        Layout { board, grid, panel }
    }
}

fn draw_scene(game: &Game) {
    let time = get_time() as f32;
    let layout = layout();

    draw_background(time);
    draw_shadowed_card(
        layout.board,
        Color::new(0.04, 0.09, 0.10, 0.92),
        Color::new(0.13, 0.92, 0.76, 0.22),
    );
    draw_shadowed_card(
        layout.panel,
        Color::new(0.05, 0.08, 0.11, 0.86),
        Color::new(0.30, 0.89, 0.91, 0.18),
    );

    draw_grid(&layout, time);
    draw_food(&layout, game, time);
    draw_bombs(&layout, game, time);
    draw_snake(&layout, game, time);
    draw_panel(&layout, game);
    draw_overlay(&layout, game, time);

    if game.death_flash > 0.0 {
        let alpha = game.death_flash * 0.18;
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(1.0, 0.18, 0.2, alpha),
        );
    }
}

fn draw_background(time: f32) {
    let top = Color::new(0.02, 0.06, 0.08, 1.0);
    let bottom = Color::new(0.01, 0.02, 0.04, 1.0);
    clear_background(bottom);

    let bands = 48.0;
    let band_height = screen_height() / bands;
    for index in 0..bands as i32 {
        let t = index as f32 / (bands - 1.0);
        let shade = lerp_color(top, bottom, t);
        draw_rectangle(
            0.0,
            band_height * index as f32,
            screen_width(),
            band_height + 1.0,
            shade,
        );
    }

    let ambient = [
        (
            vec2(
                screen_width() * 0.18 + time.sin() * 40.0,
                screen_height() * 0.25,
            ),
            screen_width() * 0.26,
            Color::new(0.13, 0.91, 0.77, 0.09),
        ),
        (
            vec2(
                screen_width() * 0.77,
                screen_height() * 0.20 + time.cos() * 28.0,
            ),
            screen_width() * 0.22,
            Color::new(0.23, 0.47, 0.99, 0.08),
        ),
        (
            vec2(screen_width() * 0.62, screen_height() * 0.82),
            screen_width() * 0.28,
            Color::new(1.0, 0.48, 0.22, 0.05),
        ),
    ];

    for (center, radius, color) in ambient {
        draw_circle(center.x, center.y, radius, color);
    }
}

fn draw_shadowed_card(rect: Rect, fill: Color, glow: Color) {
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 14.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.24),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, glow);
}

fn draw_grid(layout: &Layout, time: f32) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let checker = (x + y) % 2;
            let base = if checker == 0 {
                Color::new(0.06, 0.12, 0.13, 0.95)
            } else {
                Color::new(0.05, 0.10, 0.11, 0.95)
            };

            let shimmer =
                (((x as f32 * 0.35) + (y as f32 * 0.27) + time * 1.2).sin() + 1.0) * 0.015;
            let color = Color::new(base.r + shimmer, base.g + shimmer, base.b + shimmer, base.a);
            draw_rectangle(
                layout.grid.x + x as f32 * cell,
                layout.grid.y + y as f32 * cell,
                cell - 1.0,
                cell - 1.0,
                color,
            );
        }
    }
}

fn draw_snake(layout: &Layout, game: &Game, time: f32) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    let len = game.snake.len().max(1) as f32;

    for (index, segment) in game.snake.iter().enumerate().rev() {
        let x = layout.grid.x + segment.x as f32 * cell;
        let y = layout.grid.y + segment.y as f32 * cell;
        let inset = if index == 0 { 3.0 } else { 4.5 };
        let t = index as f32 / len;
        let color = if index == 0 {
            let pulse = 0.08 * ((time * 6.0).sin() + 1.0);
            Color::new(0.43, 0.98 - pulse * 0.2, 0.78 + pulse, 1.0)
        } else {
            lerp_color(
                Color::new(0.11, 0.74, 0.53, 1.0),
                Color::new(0.20, 0.90, 0.84, 1.0),
                1.0 - t,
            )
        };

        draw_rectangle(
            x + inset,
            y + inset,
            cell - inset * 2.0,
            cell - inset * 2.0,
            color,
        );
        draw_rectangle_lines(
            x + inset,
            y + inset,
            cell - inset * 2.0,
            cell - inset * 2.0,
            1.5,
            Color::new(1.0, 1.0, 1.0, 0.08),
        );
    }

    if let Some(head) = game.snake.first() {
        let head_center = vec2(
            layout.grid.x + (head.x as f32 + 0.5) * cell,
            layout.grid.y + (head.y as f32 + 0.5) * cell,
        );
        let eye_offset = match game.direction {
            Direction::Up => (
                vec2(-cell * 0.12, -cell * 0.14),
                vec2(cell * 0.12, -cell * 0.14),
            ),
            Direction::Down => (
                vec2(-cell * 0.12, cell * 0.14),
                vec2(cell * 0.12, cell * 0.14),
            ),
            Direction::Left => (
                vec2(-cell * 0.14, -cell * 0.12),
                vec2(-cell * 0.14, cell * 0.12),
            ),
            Direction::Right => (
                vec2(cell * 0.14, -cell * 0.12),
                vec2(cell * 0.14, cell * 0.12),
            ),
        };

        draw_circle(
            head_center.x + eye_offset.0.x,
            head_center.y + eye_offset.0.y,
            cell * 0.06,
            BLACK,
        );
        draw_circle(
            head_center.x + eye_offset.1.x,
            head_center.y + eye_offset.1.y,
            cell * 0.06,
            BLACK,
        );
    }
}

fn draw_food(layout: &Layout, game: &Game, time: f32) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    let center = vec2(
        layout.grid.x + (game.food.x as f32 + 0.5) * cell,
        layout.grid.y + (game.food.y as f32 + 0.5) * cell,
    );
    let pulse = 0.82 + (time * 5.0).sin() * 0.08 + game.food_flash * 0.25;
    let ring_alpha = 0.12 + game.food_flash * 0.3;

    draw_circle(
        center.x,
        center.y,
        cell * 0.38 * pulse,
        Color::new(1.0, 0.43, 0.27, 0.95),
    );
    draw_circle(
        center.x,
        center.y,
        cell * 0.20,
        Color::new(1.0, 0.85, 0.44, 0.95),
    );
    draw_circle_lines(
        center.x,
        center.y,
        cell * (0.46 + game.food_flash * 0.16),
        2.0,
        Color::new(1.0, 0.52, 0.24, ring_alpha),
    );
}

fn draw_bombs(layout: &Layout, game: &Game, time: f32) {
    let cell = layout.grid.w / GRID_SIZE as f32;

    for (index, bomb) in game.bombs.iter().enumerate() {
        let center = vec2(
            layout.grid.x + (bomb.x as f32 + 0.5) * cell,
            layout.grid.y + (bomb.y as f32 + 0.5) * cell,
        );
        let pulse = 0.85 + ((time * 4.5) + index as f32 * 0.7).sin() * 0.08;
        let shell = cell * 0.26 * pulse;

        draw_circle(
            center.x,
            center.y,
            cell * 0.44,
            Color::new(1.0, 0.22, 0.12, 0.10),
        );
        draw_circle(
            center.x,
            center.y,
            shell,
            Color::new(0.86, 0.12, 0.09, 0.98),
        );
        draw_circle(
            center.x,
            center.y,
            cell * 0.12,
            Color::new(0.15, 0.02, 0.03, 1.0),
        );
        draw_line(
            center.x - cell * 0.08,
            center.y - cell * 0.24,
            center.x + cell * 0.12,
            center.y - cell * 0.34,
            2.0,
            Color::new(1.0, 0.78, 0.28, 0.95),
        );
        draw_circle(
            center.x + cell * 0.15,
            center.y - cell * 0.38,
            cell * 0.05,
            Color::new(1.0, 0.86, 0.32, 0.95),
        );
    }
}

fn draw_panel(layout: &Layout, game: &Game) {
    let left = layout.panel.x + 26.0;
    let mut y = layout.panel.y + 38.0;

    draw_text_ex(
        "NEONSNAKE",
        left,
        y,
        TextParams {
            font_size: 42,
            color: Color::new(0.90, 0.98, 0.98, 1.0),
            ..Default::default()
        },
    );

    y += 34.0;
    draw_text_ex(
        "Arcade-speed desktop snake in Rust",
        left,
        y,
        TextParams {
            font_size: 20,
            color: Color::new(0.62, 0.78, 0.80, 1.0),
            ..Default::default()
        },
    );

    y += 34.0;
    let score_card = Rect::new(left, y, layout.panel.w - 52.0, 112.0);
    draw_rectangle(
        score_card.x,
        score_card.y,
        score_card.w,
        score_card.h,
        Color::new(0.07, 0.13, 0.16, 0.86),
    );
    draw_rectangle_lines(
        score_card.x,
        score_card.y,
        score_card.w,
        score_card.h,
        1.5,
        Color::new(0.18, 0.82, 0.73, 0.18),
    );

    draw_text_ex(
        "Score",
        score_card.x + 18.0,
        score_card.y + 28.0,
        TextParams {
            font_size: 22,
            color: Color::new(0.60, 0.76, 0.79, 1.0),
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("{:03}", game.score),
        score_card.x + 18.0,
        score_card.y + 84.0,
        TextParams {
            font_size: 52,
            color: Color::new(0.95, 0.99, 0.99, 1.0),
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Best {:03}", game.best_score),
        score_card.x + score_card.w - 120.0,
        score_card.y + 84.0,
        TextParams {
            font_size: 24,
            color: Color::new(0.35, 0.94, 0.84, 1.0),
            ..Default::default()
        },
    );

    y += 136.0;
    let stats = [
        ("Status", game.status_label().to_string()),
        ("Heading", game.direction.label().to_string()),
        ("Bombs", game.bombs.len().to_string()),
        (
            "Speed",
            format!("{:.1}x", BASE_STEP_DELAY / game.step_delay),
        ),
        ("Rounds", game.rounds_played.to_string()),
    ];

    for (label, value) in stats {
        draw_text_ex(
            label,
            left,
            y,
            TextParams {
                font_size: 20,
                color: Color::new(0.58, 0.72, 0.74, 1.0),
                ..Default::default()
            },
        );
        draw_text_ex(
            &value,
            layout.panel.x + layout.panel.w - 140.0,
            y,
            TextParams {
                font_size: 24,
                color: Color::new(0.93, 0.98, 0.97, 1.0),
                ..Default::default()
            },
        );
        y += 34.0;
    }

    y += 12.0;
    draw_text_ex(
        "Controls",
        left,
        y,
        TextParams {
            font_size: 22,
            color: Color::new(0.36, 0.94, 0.84, 1.0),
            ..Default::default()
        },
    );
    y += 32.0;

    let controls = [
        "WASD / Arrow keys  Move",
        "Enter / Space      Start or resume",
        "Esc                Pause",
        "R                  Restart instantly",
    ];

    for line in controls {
        draw_text_ex(
            line,
            left,
            y,
            TextParams {
                font_size: 20,
                color: Color::new(0.78, 0.87, 0.88, 1.0),
                ..Default::default()
            },
        );
        y += 28.0;
    }

    y += 10.0;
    draw_text_ex(
        "Top Runs",
        left,
        y,
        TextParams {
            font_size: 22,
            color: Color::new(0.36, 0.94, 0.84, 1.0),
            ..Default::default()
        },
    );
    y += 30.0;

    if game.high_scores.is_empty() {
        draw_text_ex(
            "No saved runs yet.",
            left,
            y,
            TextParams {
                font_size: 18,
                color: Color::new(0.62, 0.78, 0.80, 1.0),
                ..Default::default()
            },
        );
    } else {
        for (index, entry) in game.high_scores.iter().enumerate() {
            draw_text_ex(
                &format!("{}. {:04}", index + 1, entry.score),
                left,
                y,
                TextParams {
                    font_size: 19,
                    color: Color::new(0.92, 0.98, 0.98, 1.0),
                    ..Default::default()
                },
            );
            draw_text_ex(
                &format!("len {}", entry.length),
                layout.panel.x + layout.panel.w - 112.0,
                y,
                TextParams {
                    font_size: 18,
                    color: Color::new(0.62, 0.78, 0.80, 1.0),
                    ..Default::default()
                },
            );
            y += 24.0;
        }
    }

    let footer = match game.phase {
        Phase::Title => "Start moving to launch immediately.",
        Phase::Playing => "Eat the ember core and stay clear of bombs.",
        Phase::Paused => "Paused. Resume with Enter, Space, or Esc.",
        Phase::GameOver => "Crash or bomb hit. Restart with R or Enter.",
    };

    let footer_y = layout.panel.y + layout.panel.h - 42.0;
    draw_text_ex(
        footer,
        left,
        footer_y,
        TextParams {
            font_size: 18,
            color: Color::new(0.62, 0.78, 0.80, 1.0),
            ..Default::default()
        },
    );
}

fn draw_overlay(layout: &Layout, game: &Game, time: f32) {
    let (title, body) = match game.phase {
        Phase::Title => (
            "Press Enter",
            "Use WASD or arrow keys to start.\nCollect food, dodge bombs, and avoid your own trail.",
        ),
        Phase::Paused => (
            "Paused",
            "Take a breath.\nPress Esc, Space, or Enter to resume.",
        ),
        Phase::GameOver => (
            "Round Over",
            "Press R for a clean restart.\nPress Enter to jump straight back in.",
        ),
        Phase::Playing => return,
    };

    draw_rectangle(
        layout.board.x,
        layout.board.y,
        layout.board.w,
        layout.board.h,
        Color::new(0.01, 0.03, 0.04, 0.55),
    );

    let pulse = 1.0 + ((time * 2.4).sin() + 1.0) * 0.02;
    let center_x = layout.board.x + layout.board.w * 0.5;
    let center_y = layout.board.y + layout.board.h * 0.5;

    draw_text_centered(
        title,
        center_x,
        center_y - 30.0,
        52,
        Color::new(0.95, 0.99, 0.98, 1.0),
        pulse,
    );
    draw_multiline_centered(
        body,
        center_x,
        center_y + 14.0,
        24,
        Color::new(0.70, 0.84, 0.85, 1.0),
        30.0,
    );
}

fn draw_text_centered(text: &str, x: f32, y: f32, font_size: u16, color: Color, scale: f32) {
    let metrics = measure_text(text, None, font_size, scale);
    draw_text_ex(
        text,
        x - metrics.width * 0.5,
        y,
        TextParams {
            font_size,
            font_scale: scale,
            color,
            ..Default::default()
        },
    );
}

fn draw_multiline_centered(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    color: Color,
    line_gap: f32,
) {
    for (index, line) in text.lines().enumerate() {
        let metrics = measure_text(line, None, font_size, 1.0);
        draw_text_ex(
            line,
            x - metrics.width * 0.5,
            y + line_gap * index as f32,
            TextParams {
                font_size,
                color,
                ..Default::default()
            },
        );
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let clamped = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * clamped,
        a.g + (b.g - a.g) * clamped,
        a.b + (b.b - a.b) * clamped,
        a.a + (b.a - a.a) * clamped,
    )
}

async fn load_generated_sound(data: Vec<u8>) -> Option<Sound> {
    load_sound_from_bytes(&data).await.ok()
}

fn generate_key_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.05;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let envelope = adsr(progress, 0.003, 0.0, 1.0, 0.55);
        let body = (std::f32::consts::TAU * 1320.0 * t).sin();
        let tail = (std::f32::consts::TAU * 980.0 * t).sin() * 0.35;
        samples.push((body * 0.8 + tail) * envelope * 0.28);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_eat_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.11;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let freq = 700.0 + 520.0 * progress;
        let envelope = adsr(progress, 0.02, 0.0, 1.0, 0.22);
        let tone = (std::f32::consts::TAU * freq * t).sin();
        let sparkle = (std::f32::consts::TAU * (freq * 1.9) * t).sin() * 0.25;
        samples.push((tone * 0.9 + sparkle) * envelope * 0.42);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_boom_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.48;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let envelope = adsr(progress, 0.002, 0.03, 0.70, 0.78);
        let noise = (macroquad::rand::gen_range(-1000, 1000) as f32) / 1000.0;
        let rumble = (std::f32::consts::TAU * (86.0 - progress * 20.0) * t).sin() * 0.65;
        let crack = (std::f32::consts::TAU * 240.0 * t).sin() * (1.0 - progress) * 0.2;
        samples.push((noise * 0.55 + rumble + crack) * envelope * 0.52);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_music_loop() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 6.0;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);
    let progression = [
        [196.00, 246.94, 293.66],
        [220.00, 261.63, 329.63],
        [174.61, 220.00, 261.63],
        [196.00, 246.94, 329.63],
    ];
    let chord_len = SAMPLE_RATE as usize + SAMPLE_RATE as usize / 2;

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let chord_index = (index / chord_len) % progression.len();
        let chord = progression[chord_index];
        let chord_progress = (index % chord_len) as f32 / chord_len as f32;

        let pad = chord
            .iter()
            .enumerate()
            .map(|(voice, freq)| {
                let detune = 1.0 + voice as f32 * 0.003;
                (std::f32::consts::TAU * freq * detune * t).sin() * (0.26 - voice as f32 * 0.04)
            })
            .sum::<f32>();

        let arp_step = ((t * 4.0) as usize) % chord.len();
        let arp_freq = chord[arp_step] * 2.0;
        let arp = (std::f32::consts::TAU * arp_freq * t).sin().max(0.0) * 0.12;

        let sweep = (std::f32::consts::TAU * 0.18 * t).sin() * 0.08;
        let fade = (0.82 - (chord_progress - 0.5).abs() * 0.18).clamp(0.72, 0.88);
        samples.push((pad + arp + sweep) * fade * 0.34);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_game_over_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.38;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let freq = 340.0 - 220.0 * progress;
        let envelope = adsr(progress, 0.01, 0.0, 1.0, 0.70);
        let wobble = (std::f32::consts::TAU * 7.0 * t).sin() * 8.0;
        let base = (std::f32::consts::TAU * (freq + wobble) * t).sin();
        let undertone = (std::f32::consts::TAU * (freq * 0.48) * t).sin() * 0.45;
        samples.push((base * 0.8 + undertone) * envelope * 0.5);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn adsr(progress: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
    if progress < attack {
        return progress / attack.max(f32::EPSILON);
    }

    if progress > 1.0 - release {
        return ((1.0 - progress) / release.max(f32::EPSILON)).clamp(0.0, 1.0) * sustain;
    }

    if decay > 0.0 {
        let decay_end = attack + decay;
        if progress < decay_end {
            let t = (progress - attack) / decay;
            return 1.0 + (sustain - 1.0) * t;
        }
    }

    sustain
}

fn write_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = (samples.len() * block_align as usize) as u32;
    let chunk_size = 36 + data_size;

    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&chunk_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    bytes
}

async fn capture_repo_screenshots() {
    let output_dir = Path::new("docs/screenshots");
    let _ = std::fs::create_dir_all(output_dir);
    let mut game = Game::new();

    capture_frame("docs/screenshots/title.png", &game).await;

    game.configure_showcase_playing();
    capture_frame("docs/screenshots/gameplay.png", &game).await;

    game.configure_showcase_game_over();
    capture_frame("docs/screenshots/game-over.png", &game).await;
}

async fn capture_frame(path: &str, game: &Game) {
    for _ in 0..3 {
        draw_scene(game);
        next_frame().await;
    }
    draw_scene(game);
    get_screen_data().export_png(path);
    next_frame().await;
}

#[macroquad::main(window_conf)]
async fn main() {
    if std::env::var("NEONSNAKE_CAPTURE").as_deref() == Ok("1") {
        capture_repo_screenshots().await;
        return;
    }

    let mut game = Game::new();
    let sounds = SoundBank::load().await;
    sounds.start_music();

    loop {
        let dt = get_frame_time().min(0.05);
        for cue in game.handle_input() {
            sounds.play(cue);
        }
        for cue in game.update(dt) {
            sounds.play(cue);
        }
        draw_scene(&game);
        next_frame().await;
    }
}
