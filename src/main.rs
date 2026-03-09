mod app;
mod audio;
mod game;
mod storage;
mod ui;

fn window_conf() -> macroquad::prelude::Conf {
    app::window_conf()
}

#[macroquad::main(window_conf)]
async fn main() {
    app::run().await;
}
