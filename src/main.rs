use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Agical Macroquad Tutorial".to_owned(),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(Color::from_hex(0x643c80));
        next_frame().await
    }
}