use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Agical Macroquad Tutorial".to_owned(),
        window_resizable: false,
        ..Default::default()
    }
}

const SPEED: f32 = 5.0;


#[macroquad::main(window_conf)]
async fn main() {
    // Get screen center
    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;

    loop {
        clear_background(Color::from_hex(0x643c80));

        // Input setup
        if is_key_down(KeyCode::Right) {
            x += SPEED;
        }
        if is_key_down(KeyCode::Left) {
            x -= SPEED;
        }
        if is_key_down(KeyCode::Down) {
            y += SPEED;
        }
        if is_key_down(KeyCode::Up) {
            y -= SPEED;
        }

        // Draw the circle
        draw_circle(x, y, 16.0, Color::from_hex(0xb1de78));

        next_frame().await
    }
}