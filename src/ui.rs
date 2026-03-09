use macroquad::prelude::*;

use crate::game::{
    BASE_STEP_DELAY, BOARD_PADDING, Direction, GRID_SIZE, Game, LevelTheme, ParticleShape, Phase,
};

struct Layout {
    board: Rect,
    grid: Rect,
    panel: Rect,
}

#[derive(Clone, Copy)]
struct ThemePalette {
    background_top: Color,
    background_bottom: Color,
    ambient_a: Color,
    ambient_b: Color,
    ambient_c: Color,
    board_fill: Color,
    panel_fill: Color,
    board_glow: Color,
    panel_glow: Color,
    grid_primary: Color,
    grid_secondary: Color,
    grid_highlight: Color,
    snake_head: Color,
    snake_body_a: Color,
    snake_body_b: Color,
    food_outer: Color,
    food_inner: Color,
    bomb_glow: Color,
    bomb_shell: Color,
    title_text: Color,
    accent_text: Color,
    body_text: Color,
    muted_text: Color,
}

#[derive(Clone, Copy)]
enum GridPattern {
    Checker,
    Scanlines,
    Diagonal,
    Pulse,
}

pub fn draw_scene(game: &Game) {
    let time = get_time() as f32;
    let layout = layout();
    let theme = game.level_theme();
    let palette = theme_palette(theme);

    draw_background(time, theme, palette);
    draw_shadowed_card(layout.board, palette.board_fill, palette.board_glow);
    draw_shadowed_card(layout.panel, palette.panel_fill, palette.panel_glow);

    draw_grid(&layout, time, theme, palette);
    draw_food(&layout, game, time, palette);
    draw_bombs(&layout, game, time, palette);
    draw_power_up(&layout, game, time, palette);
    draw_snake(&layout, game, time, palette);
    draw_particles(&layout, game);
    draw_panel(&layout, game, theme, palette);
    draw_overlay(&layout, game, time, palette);

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

fn draw_background(time: f32, theme: LevelTheme, palette: ThemePalette) {
    clear_background(palette.background_bottom);

    let bands = 48.0;
    let band_height = screen_height() / bands;
    for index in 0..bands as i32 {
        let t = index as f32 / (bands - 1.0);
        let shade = lerp_color(palette.background_top, palette.background_bottom, t);
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
            palette.ambient_a,
        ),
        (
            vec2(
                screen_width() * 0.77,
                screen_height() * 0.20 + time.cos() * 28.0,
            ),
            screen_width() * 0.22,
            palette.ambient_b,
        ),
        (
            vec2(screen_width() * 0.62, screen_height() * 0.82),
            screen_width() * theme_radius(theme),
            palette.ambient_c,
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

fn draw_grid(layout: &Layout, time: f32, theme: LevelTheme, palette: ThemePalette) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let base = match grid_pattern(theme) {
                GridPattern::Checker => {
                    if (x + y) % 2 == 0 {
                        palette.grid_primary
                    } else {
                        palette.grid_secondary
                    }
                }
                GridPattern::Scanlines => {
                    let stripe = if y % 2 == 0 { 0.14 } else { -0.04 };
                    shift_color(
                        if x % 3 == 0 {
                            palette.grid_secondary
                        } else {
                            palette.grid_primary
                        },
                        stripe,
                    )
                }
                GridPattern::Diagonal => {
                    let diagonal = ((x - y).abs() % 4) == 0;
                    if diagonal {
                        lerp_color(palette.grid_primary, palette.grid_highlight, 0.38)
                    } else if (x + y) % 2 == 0 {
                        palette.grid_primary
                    } else {
                        palette.grid_secondary
                    }
                }
                GridPattern::Pulse => {
                    let center = vec2(GRID_SIZE as f32 * 0.5, GRID_SIZE as f32 * 0.5);
                    let offset = vec2(x as f32 - center.x, y as f32 - center.y);
                    let distance = offset.length();
                    let wave = ((distance * 0.7) - time * 4.2).sin() * 0.08;
                    shift_color(palette.grid_primary, wave)
                }
            };

            let shimmer = (((x as f32 * 0.35) + (y as f32 * 0.27) + time * 1.2).sin() + 1.0)
                * (0.010 + theme.visual_intensity() * 0.003);
            let color = shift_color(base, shimmer);
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

fn draw_snake(layout: &Layout, game: &Game, time: f32, palette: ThemePalette) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    let len = game.snake.len().max(1) as f32;

    for (index, segment) in game.snake.iter().enumerate().rev() {
        let x = layout.grid.x + segment.x as f32 * cell;
        let y = layout.grid.y + segment.y as f32 * cell;
        let inset = if index == 0 { 3.0 } else { 4.5 };
        let t = index as f32 / len;
        let color = if index == 0 {
            let pulse = 0.08 * ((time * 6.0).sin() + 1.0);
            shift_color(palette.snake_head, pulse)
        } else {
            lerp_color(palette.snake_body_a, palette.snake_body_b, 1.0 - t)
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

fn draw_food(layout: &Layout, game: &Game, time: f32, palette: ThemePalette) {
    let cell = layout.grid.w / GRID_SIZE as f32;
    let center = vec2(
        layout.grid.x + (game.food.x as f32 + 0.5) * cell,
        layout.grid.y + (game.food.y as f32 + 0.5) * cell,
    );
    let pulse = 0.82 + (time * 5.0).sin() * 0.08 + game.food_flash * 0.25;
    let ring_alpha = 0.12 + game.food_flash * 0.3;

    draw_circle(center.x, center.y, cell * 0.38 * pulse, palette.food_outer);
    draw_circle(center.x, center.y, cell * 0.20, palette.food_inner);
    draw_circle_lines(
        center.x,
        center.y,
        cell * (0.46 + game.food_flash * 0.16),
        2.0,
        Color::new(
            palette.food_outer.r,
            palette.food_outer.g,
            palette.food_outer.b,
            ring_alpha,
        ),
    );
}

fn draw_bombs(layout: &Layout, game: &Game, time: f32, palette: ThemePalette) {
    let cell = layout.grid.w / GRID_SIZE as f32;

    for (index, bomb) in game.bombs.iter().enumerate() {
        let center = vec2(
            layout.grid.x + (bomb.x as f32 + 0.5) * cell,
            layout.grid.y + (bomb.y as f32 + 0.5) * cell,
        );
        let pulse = 0.85 + ((time * 4.5) + index as f32 * 0.7).sin() * 0.08;
        let shell = cell * 0.26 * pulse;

        draw_circle(center.x, center.y, cell * 0.44, palette.bomb_glow);
        draw_circle(center.x, center.y, shell, palette.bomb_shell);
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

fn draw_power_up(layout: &Layout, game: &Game, time: f32, palette: ThemePalette) {
    let Some(power_up) = game.power_up else {
        return;
    };

    let cell = layout.grid.w / GRID_SIZE as f32;
    let center = vec2(
        layout.grid.x + (power_up.position.x as f32 + 0.5) * cell,
        layout.grid.y + (power_up.position.y as f32 + 0.5) * cell,
    );
    let glow = 0.88 + (time * 6.0).sin() * 0.08;
    let color = power_up.kind.color();

    draw_circle(
        center.x,
        center.y,
        cell * 0.44,
        Color::new(
            lerp_color(color, palette.grid_highlight, 0.18).r,
            lerp_color(color, palette.grid_highlight, 0.18).g,
            lerp_color(color, palette.grid_highlight, 0.18).b,
            0.12,
        ),
    );
    draw_poly(
        center.x,
        center.y,
        6,
        cell * 0.28 * glow,
        time * 40.0,
        color,
    );
    draw_circle(
        center.x,
        center.y,
        cell * 0.16,
        Color::new(0.05, 0.08, 0.11, 0.92),
    );

    let label = power_up.kind.short_label();
    let metrics = measure_text(label, None, 18, 1.0);
    draw_text_ex(
        label,
        center.x - metrics.width * 0.5,
        center.y + 6.0,
        TextParams {
            font_size: 18,
            color: WHITE,
            ..Default::default()
        },
    );
}

fn draw_panel(layout: &Layout, game: &Game, theme: LevelTheme, palette: ThemePalette) {
    let left = layout.panel.x + 26.0;
    let mut y = layout.panel.y + 38.0;

    draw_text_ex(
        "NEONSNAKE",
        left,
        y,
        TextParams {
            font_size: 42,
            color: palette.title_text,
            ..Default::default()
        },
    );

    y += 34.0;
    draw_text_ex(
        &format!("{} theme live on the board", theme.name()),
        left,
        y,
        TextParams {
            font_size: 20,
            color: palette.body_text,
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
            color: palette.body_text,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("{:03}", game.score),
        score_card.x + 18.0,
        score_card.y + 84.0,
        TextParams {
            font_size: 52,
            color: palette.title_text,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Best {:03}", game.best_score),
        score_card.x + score_card.w - 120.0,
        score_card.y + 84.0,
        TextParams {
            font_size: 24,
            color: palette.accent_text,
            ..Default::default()
        },
    );

    y += 136.0;
    let stats: [(&str, String); 8] = [
        ("Mode", game.mode_label().to_owned()),
        ("Theme", theme.name().to_owned()),
        ("Pattern", game.hazard_pattern_name().to_owned()),
        ("Heading", game.direction.label().to_string()),
        ("Bombs", game.bombs.len().to_string()),
        (
            "Speed",
            format!("{:.1}x", BASE_STEP_DELAY / game.effective_step_delay()),
        ),
        (
            "Next",
            game.next_theme_score()
                .map(|score| format!("{score} pts"))
                .unwrap_or_else(|| "MAX".to_owned()),
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
                color: palette.muted_text,
                ..Default::default()
            },
        );
        draw_text_ex(
            &value,
            layout.panel.x + layout.panel.w - 140.0,
            y,
            TextParams {
                font_size: 24,
                color: palette.title_text,
                ..Default::default()
            },
        );
        y += 34.0;
    }

    if game.is_challenge_mode() {
        y += 8.0;
        draw_text_ex(
            "Challenge",
            left,
            y,
            TextParams {
                font_size: 22,
                color: palette.accent_text,
                ..Default::default()
            },
        );
        y += 28.0;
        draw_text_ex(
            game.challenge_title(),
            left,
            y,
            TextParams {
                font_size: 20,
                color: palette.title_text,
                ..Default::default()
            },
        );
        y += 24.0;
        draw_text_ex(
            game.challenge_detail(),
            left,
            y,
            TextParams {
                font_size: 17,
                color: palette.body_text,
                ..Default::default()
            },
        );
        y += 26.0;
        let progress_rect = Rect::new(left, y, layout.panel.w - 52.0, 12.0);
        draw_rectangle(
            progress_rect.x,
            progress_rect.y,
            progress_rect.w,
            progress_rect.h,
            Color::new(1.0, 1.0, 1.0, 0.06),
        );
        draw_rectangle(
            progress_rect.x,
            progress_rect.y,
            progress_rect.w * game.challenge_progress_ratio(),
            progress_rect.h,
            palette.accent_text,
        );
        y += 24.0;
        draw_text_ex(
            &game.challenge_progress_text(),
            left,
            y,
            TextParams {
                font_size: 18,
                color: palette.muted_text,
                ..Default::default()
            },
        );
        y += 20.0;
    }

    y += 8.0;
    draw_text_ex(
        "Power-Ups",
        left,
        y,
        TextParams {
            font_size: 22,
            color: palette.accent_text,
            ..Default::default()
        },
    );
    y += 28.0;

    let pickup_text = if let Some(power_up) = game.power_up {
        format!("Board: {} {:.1}s", power_up.kind.label(), power_up.ttl)
    } else {
        "Board: none".to_owned()
    };
    draw_text_ex(
        &pickup_text,
        left,
        y,
        TextParams {
            font_size: 18,
            color: palette.body_text,
            ..Default::default()
        },
    );
    y += 24.0;

    let active_effects = game.active_power_up_labels();
    if active_effects.is_empty() {
        draw_text_ex(
            "Active: none",
            left,
            y,
            TextParams {
                font_size: 18,
                color: palette.muted_text,
                ..Default::default()
            },
        );
        y += 24.0;
    } else {
        for effect in active_effects {
            draw_text_ex(
                &format!("Active: {effect}"),
                left,
                y,
                TextParams {
                    font_size: 18,
                    color: palette.title_text,
                    ..Default::default()
                },
            );
            y += 24.0;
        }
    }

    y += 12.0;
    draw_text_ex(
        "Controls",
        left,
        y,
        TextParams {
            font_size: 22,
            color: palette.accent_text,
            ..Default::default()
        },
    );
    y += 32.0;

    let controls = [
        "WASD / Arrow keys  Move",
        "Enter / Space      Start or resume",
        "Esc                Pause",
        "R                  Restart instantly",
        "M                  Toggle mode in menus",
    ];

    for line in controls {
        draw_text_ex(
            line,
            left,
            y,
            TextParams {
                font_size: 20,
                color: palette.body_text,
                ..Default::default()
            },
        );
        y += 28.0;
    }
}

fn draw_particles(layout: &Layout, game: &Game) {
    let cell = layout.grid.w / GRID_SIZE as f32;

    for particle in &game.particles {
        let progress = (particle.ttl / particle.max_ttl.max(f32::EPSILON)).clamp(0.0, 1.0);
        let alpha = particle.color.a * progress;
        let color = Color::new(particle.color.r, particle.color.g, particle.color.b, alpha);
        let center = vec2(
            layout.grid.x + particle.position.x * cell,
            layout.grid.y + particle.position.y * cell,
        );
        let size = particle.size * cell * (0.55 + (1.0 - progress) * 0.8);

        match particle.shape {
            ParticleShape::Dot => draw_circle(center.x, center.y, size * 0.45, color),
            ParticleShape::Shard => draw_poly(
                center.x,
                center.y,
                4,
                size,
                particle.rotation * 57.29578,
                color,
            ),
            ParticleShape::Ring => draw_circle_lines(
                center.x,
                center.y,
                size,
                (cell * 0.08).max(1.6) * progress.max(0.25),
                color,
            ),
        }
    }
}

fn draw_overlay(layout: &Layout, game: &Game, time: f32, palette: ThemePalette) {
    let (title, body) = match game.phase {
        Phase::Title if game.is_challenge_mode() => (
            "Challenge Mode",
            "Press Enter to start the challenge.\nPress M to switch back to arcade mode.",
        ),
        Phase::Title => (
            "Press Enter",
            "Use WASD or arrow keys to start.\nPress M to switch into challenge mode.",
        ),
        Phase::Paused => (
            "Paused",
            "Take a breath.\nPress Esc, Space, or Enter to resume.",
        ),
        Phase::ChallengeClear => (
            "Challenge Clear",
            "Press R for a clean restart.\nPress Enter to queue the next challenge.",
        ),
        Phase::GameOver => (
            if game.is_challenge_mode() {
                "Challenge Failed"
            } else {
                "Round Over"
            },
            "Press R for a clean restart.\nPress Enter to jump straight back in.",
        ),
        Phase::Playing => return,
    };

    let overlay_alpha = if matches!(game.phase, Phase::GameOver | Phase::ChallengeClear) {
        0.68
    } else {
        0.55
    };
    draw_rectangle(
        layout.board.x,
        layout.board.y,
        layout.board.w,
        layout.board.h,
        Color::new(0.01, 0.03, 0.04, overlay_alpha),
    );

    if matches!(game.phase, Phase::GameOver | Phase::ChallengeClear) {
        let card = Rect::new(
            layout.board.x + layout.board.w * 0.17,
            layout.board.y + layout.board.h * 0.19,
            layout.board.w * 0.66,
            layout.board.h * 0.54,
        );
        draw_rectangle(
            card.x + 10.0,
            card.y + 14.0,
            card.w,
            card.h,
            Color::new(0.0, 0.0, 0.0, 0.24),
        );
        draw_rectangle(
            card.x,
            card.y,
            card.w,
            card.h,
            Color::new(0.08, 0.05, 0.09, 0.92),
        );
        draw_rectangle_lines(card.x, card.y, card.w, card.h, 2.0, palette.accent_text);
        draw_rectangle(
            card.x + 28.0,
            card.y + 98.0,
            card.w - 56.0,
            1.5,
            Color::new(
                palette.accent_text.r,
                palette.accent_text.g,
                palette.accent_text.b,
                0.42,
            ),
        );

        draw_text_centered(
            title,
            card.x + card.w * 0.5,
            card.y + 56.0,
            46,
            palette.title_text,
            1.0,
        );
        draw_text_centered(
            &format!("Current score {:03}", game.score),
            card.x + card.w * 0.5,
            card.y + 128.0,
            28,
            palette.body_text,
            1.0,
        );
        if game.is_challenge_mode() {
            draw_text_centered(
                &format!(
                    "{}  •  {}",
                    game.challenge_title(),
                    game.challenge_progress_text()
                ),
                card.x + card.w * 0.5,
                card.y + 160.0,
                20,
                palette.accent_text,
                1.0,
            );
        }
        draw_text_centered(
            &format!(
                "Theme {}   |   Pattern {}",
                game.level_theme().name(),
                game.hazard_pattern_name()
            ),
            card.x + card.w * 0.5,
            card.y + 190.0,
            19,
            palette.muted_text,
            1.0,
        );
        draw_text_centered(
            &format!(
                "Snake length {}   |   Best {:03}",
                game.snake.len(),
                game.best_score
            ),
            card.x + card.w * 0.5,
            card.y + 222.0,
            19,
            palette.muted_text,
            1.0,
        );
        draw_text_centered(
            "Top 3 Runs",
            card.x + card.w * 0.5,
            card.y + 268.0,
            24,
            palette.accent_text,
            1.0,
        );

        let leaderboard_y = card.y + 306.0;
        if game.high_scores.is_empty() {
            draw_text_centered(
                "No saved runs yet",
                card.x + card.w * 0.5,
                leaderboard_y,
                18,
                palette.muted_text,
                1.0,
            );
        } else {
            for (index, entry) in game.high_scores.iter().take(3).enumerate() {
                draw_text_centered(
                    &format!("{}. {:04}   len {}", index + 1, entry.score, entry.length),
                    card.x + card.w * 0.5,
                    leaderboard_y + index as f32 * 28.0,
                    19,
                    if index == 0 {
                        palette.title_text
                    } else {
                        palette.body_text
                    },
                    1.0,
                );
            }
        }
        draw_text_centered(
            "R to restart immediately  •  Enter to jump back in",
            card.x + card.w * 0.5,
            card.y + card.h - 34.0,
            20,
            palette.accent_text,
            1.0,
        );
        return;
    }

    let pulse = 1.0 + ((time * 2.4).sin() + 1.0) * 0.02;
    let center_x = layout.board.x + layout.board.w * 0.5;
    let center_y = layout.board.y + layout.board.h * 0.5;

    draw_text_centered(
        title,
        center_x,
        center_y - 30.0,
        52,
        palette.title_text,
        pulse,
    );
    draw_multiline_centered(body, center_x, center_y + 14.0, 24, palette.body_text, 30.0);
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

fn shift_color(color: Color, amount: f32) -> Color {
    Color::new(
        (color.r + amount).clamp(0.0, 1.0),
        (color.g + amount).clamp(0.0, 1.0),
        (color.b + amount).clamp(0.0, 1.0),
        color.a,
    )
}

fn grid_pattern(theme: LevelTheme) -> GridPattern {
    match theme {
        LevelTheme::Afterglow => GridPattern::Checker,
        LevelTheme::Voltage => GridPattern::Scanlines,
        LevelTheme::Overdrive => GridPattern::Diagonal,
        LevelTheme::Singularity => GridPattern::Pulse,
    }
}

fn theme_radius(theme: LevelTheme) -> f32 {
    match theme {
        LevelTheme::Afterglow => 0.28,
        LevelTheme::Voltage => 0.32,
        LevelTheme::Overdrive => 0.36,
        LevelTheme::Singularity => 0.42,
    }
}

fn theme_palette(theme: LevelTheme) -> ThemePalette {
    match theme {
        LevelTheme::Afterglow => ThemePalette {
            background_top: Color::new(0.02, 0.06, 0.08, 1.0),
            background_bottom: Color::new(0.01, 0.02, 0.04, 1.0),
            ambient_a: Color::new(0.13, 0.91, 0.77, 0.09),
            ambient_b: Color::new(0.23, 0.47, 0.99, 0.08),
            ambient_c: Color::new(1.0, 0.48, 0.22, 0.05),
            board_fill: Color::new(0.04, 0.09, 0.10, 0.92),
            panel_fill: Color::new(0.05, 0.08, 0.11, 0.86),
            board_glow: Color::new(0.13, 0.92, 0.76, 0.22),
            panel_glow: Color::new(0.30, 0.89, 0.91, 0.18),
            grid_primary: Color::new(0.06, 0.12, 0.13, 0.95),
            grid_secondary: Color::new(0.05, 0.10, 0.11, 0.95),
            grid_highlight: Color::new(0.19, 0.83, 0.74, 1.0),
            snake_head: Color::new(0.43, 0.98, 0.78, 1.0),
            snake_body_a: Color::new(0.11, 0.74, 0.53, 1.0),
            snake_body_b: Color::new(0.20, 0.90, 0.84, 1.0),
            food_outer: Color::new(1.0, 0.43, 0.27, 0.95),
            food_inner: Color::new(1.0, 0.85, 0.44, 0.95),
            bomb_glow: Color::new(1.0, 0.22, 0.12, 0.10),
            bomb_shell: Color::new(0.86, 0.12, 0.09, 0.98),
            title_text: Color::new(0.95, 0.99, 0.99, 1.0),
            accent_text: Color::new(0.35, 0.94, 0.84, 1.0),
            body_text: Color::new(0.78, 0.87, 0.88, 1.0),
            muted_text: Color::new(0.62, 0.78, 0.80, 1.0),
        },
        LevelTheme::Voltage => ThemePalette {
            background_top: Color::new(0.02, 0.04, 0.10, 1.0),
            background_bottom: Color::new(0.01, 0.01, 0.05, 1.0),
            ambient_a: Color::new(0.10, 0.72, 1.0, 0.10),
            ambient_b: Color::new(0.55, 0.42, 1.0, 0.08),
            ambient_c: Color::new(0.98, 0.72, 0.22, 0.06),
            board_fill: Color::new(0.03, 0.07, 0.12, 0.93),
            panel_fill: Color::new(0.04, 0.06, 0.13, 0.88),
            board_glow: Color::new(0.18, 0.65, 1.0, 0.24),
            panel_glow: Color::new(0.44, 0.69, 1.0, 0.20),
            grid_primary: Color::new(0.05, 0.11, 0.18, 0.95),
            grid_secondary: Color::new(0.04, 0.08, 0.16, 0.95),
            grid_highlight: Color::new(0.31, 0.74, 1.0, 1.0),
            snake_head: Color::new(0.54, 0.95, 1.0, 1.0),
            snake_body_a: Color::new(0.18, 0.59, 0.96, 1.0),
            snake_body_b: Color::new(0.38, 0.84, 0.99, 1.0),
            food_outer: Color::new(1.0, 0.58, 0.18, 0.95),
            food_inner: Color::new(1.0, 0.91, 0.36, 0.95),
            bomb_glow: Color::new(1.0, 0.26, 0.16, 0.12),
            bomb_shell: Color::new(0.96, 0.20, 0.14, 0.98),
            title_text: Color::new(0.94, 0.98, 1.0, 1.0),
            accent_text: Color::new(0.45, 0.84, 1.0, 1.0),
            body_text: Color::new(0.80, 0.88, 0.97, 1.0),
            muted_text: Color::new(0.64, 0.77, 0.91, 1.0),
        },
        LevelTheme::Overdrive => ThemePalette {
            background_top: Color::new(0.08, 0.03, 0.05, 1.0),
            background_bottom: Color::new(0.03, 0.01, 0.03, 1.0),
            ambient_a: Color::new(1.0, 0.34, 0.18, 0.10),
            ambient_b: Color::new(1.0, 0.16, 0.47, 0.08),
            ambient_c: Color::new(1.0, 0.78, 0.22, 0.06),
            board_fill: Color::new(0.10, 0.04, 0.07, 0.93),
            panel_fill: Color::new(0.09, 0.04, 0.06, 0.88),
            board_glow: Color::new(1.0, 0.36, 0.22, 0.22),
            panel_glow: Color::new(1.0, 0.57, 0.25, 0.18),
            grid_primary: Color::new(0.14, 0.06, 0.08, 0.95),
            grid_secondary: Color::new(0.10, 0.04, 0.07, 0.95),
            grid_highlight: Color::new(1.0, 0.48, 0.24, 1.0),
            snake_head: Color::new(1.0, 0.74, 0.28, 1.0),
            snake_body_a: Color::new(0.99, 0.37, 0.19, 1.0),
            snake_body_b: Color::new(1.0, 0.62, 0.20, 1.0),
            food_outer: Color::new(1.0, 0.26, 0.20, 0.95),
            food_inner: Color::new(1.0, 0.87, 0.33, 0.95),
            bomb_glow: Color::new(1.0, 0.18, 0.12, 0.12),
            bomb_shell: Color::new(0.97, 0.14, 0.10, 0.98),
            title_text: Color::new(1.0, 0.96, 0.94, 1.0),
            accent_text: Color::new(1.0, 0.67, 0.28, 1.0),
            body_text: Color::new(0.97, 0.85, 0.80, 1.0),
            muted_text: Color::new(0.85, 0.68, 0.61, 1.0),
        },
        LevelTheme::Singularity => ThemePalette {
            background_top: Color::new(0.02, 0.02, 0.08, 1.0),
            background_bottom: Color::new(0.00, 0.00, 0.03, 1.0),
            ambient_a: Color::new(0.58, 0.38, 1.0, 0.10),
            ambient_b: Color::new(0.16, 0.92, 1.0, 0.08),
            ambient_c: Color::new(1.0, 0.20, 0.68, 0.07),
            board_fill: Color::new(0.04, 0.03, 0.11, 0.94),
            panel_fill: Color::new(0.03, 0.03, 0.10, 0.89),
            board_glow: Color::new(0.69, 0.42, 1.0, 0.24),
            panel_glow: Color::new(0.24, 0.90, 1.0, 0.18),
            grid_primary: Color::new(0.06, 0.05, 0.16, 0.95),
            grid_secondary: Color::new(0.05, 0.04, 0.12, 0.95),
            grid_highlight: Color::new(0.75, 0.46, 1.0, 1.0),
            snake_head: Color::new(0.76, 0.95, 1.0, 1.0),
            snake_body_a: Color::new(0.48, 0.44, 1.0, 1.0),
            snake_body_b: Color::new(0.20, 0.95, 1.0, 1.0),
            food_outer: Color::new(1.0, 0.38, 0.74, 0.95),
            food_inner: Color::new(0.98, 0.93, 0.44, 0.95),
            bomb_glow: Color::new(1.0, 0.20, 0.54, 0.12),
            bomb_shell: Color::new(0.93, 0.16, 0.44, 0.98),
            title_text: Color::new(0.96, 0.97, 1.0, 1.0),
            accent_text: Color::new(0.52, 0.91, 1.0, 1.0),
            body_text: Color::new(0.83, 0.87, 0.98, 1.0),
            muted_text: Color::new(0.69, 0.75, 0.92, 1.0),
        },
    }
}
