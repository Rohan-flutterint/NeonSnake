use macroquad::prelude::*;
use std::path::Path;

use crate::{
    audio::SoundBank,
    game::{Game, LevelTheme},
    ui::draw_scene,
};

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
    let mut sounds = SoundBank::load().await;
    let mut active_theme = game.level_theme();
    sounds.start_music(active_theme);

    loop {
        let dt = get_frame_time().min(0.05);
        for cue in game.handle_input() {
            sounds.play(cue);
        }
        for cue in game.update(dt) {
            sounds.play(cue);
        }
        sync_theme_audio(&game, &mut sounds, &mut active_theme);
        draw_scene(&game);
        next_frame().await;
    }
}

fn sync_theme_audio(game: &Game, sounds: &mut SoundBank, active_theme: &mut LevelTheme) {
    let next_theme = game.level_theme();
    if next_theme != *active_theme {
        sounds.apply_theme(next_theme);
        *active_theme = next_theme;
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
