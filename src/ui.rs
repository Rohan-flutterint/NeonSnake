use macroquad::prelude::*;

use crate::game::{BASE_STEP_DELAY, BOARD_PADDING, Direction, GRID_SIZE, Game, Phase};

struct Layout {
    board: Rect,
    grid: Rect,
    panel: Rect,
}

pub fn draw_scene(game: &Game) {
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
    draw_power_up(&layout, game, time);
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

fn draw_power_up(layout: &Layout, game: &Game, time: f32) {
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
        Color::new(color.r, color.g, color.b, 0.10),
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
    let stats: [(&str, String); 5] = [
        ("Status", game.status_label().to_string()),
        ("Heading", game.direction.label().to_string()),
        ("Bombs", game.bombs.len().to_string()),
        (
            "Speed",
            format!("{:.1}x", BASE_STEP_DELAY / game.effective_step_delay()),
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

    y += 8.0;
    draw_text_ex(
        "Power-Ups",
        left,
        y,
        TextParams {
            font_size: 22,
            color: Color::new(0.36, 0.94, 0.84, 1.0),
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
            color: Color::new(0.78, 0.87, 0.88, 1.0),
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
                color: Color::new(0.62, 0.78, 0.80, 1.0),
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
                    color: Color::new(0.92, 0.98, 0.98, 1.0),
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
        Phase::Playing if game.power_up.is_some() => {
            "Grab the power-up before it expires and manage the bombs."
        }
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
            "Use WASD or arrow keys to start.\nCollect food, grab power-ups, dodge bombs, and avoid your own trail.",
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
