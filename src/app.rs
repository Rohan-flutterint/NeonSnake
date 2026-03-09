use macroquad::prelude::*;
use std::path::Path;

use crate::{audio::SoundBank, game::Game, ui::draw_scene};

pub fn window_conf() -> Conf {
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

pub async fn run() {
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
