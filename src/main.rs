use macroquad::prelude::*;
use rand::ChooseRandom;

fn window_conf() -> Conf {
    Conf {
        window_title: "Agical Macroquad Tutorial".to_owned(),
        window_resizable: false,
        ..Default::default()
    }
}


struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color
}


#[macroquad::main(window_conf)]
async fn main() {
    // seeding the RNG
    rand::srand(miniquad::date::now() as u64);

    // Setting shapes
    let mut squares = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        color: Color::from_hex(0xb1de78),
    };

    // "Player entity"
    const SPEED: f32 = 200.0;

    let enemy_clr: Vec<Color> = vec![
        Color::from_hex(0xfca78b),
        Color::from_hex(0xf6957d),
        Color::from_hex(0xf0826e),
        Color::from_hex(0xea7060),
        Color::from_hex(0xe45d51),
        Color::from_hex(0xde4b43),
        Color::from_hex(0xd83834),
    ];

    loop {
        clear_background(Color::from_hex(0x643c80));

        // Get delta time
        let delta_time = get_frame_time();

        // Input setup
        if is_key_down(KeyCode::Right) {
            circle.x += circle.speed * delta_time;
        }
        if is_key_down(KeyCode::Left) {
            circle.x -= circle.speed * delta_time;
        }
        if is_key_down(KeyCode::Down) {
            circle.y += circle.speed * delta_time;
        }
        if is_key_down(KeyCode::Up) {
            circle.y -= circle.speed * delta_time;
        }

        circle.x = clamp(circle.x, 0.0 + circle.size, screen_width() - circle.size);
        circle.y = clamp(circle.y, 0.0 + circle.size, screen_height() - circle.size);

        // Draw the circle
        draw_circle(circle.x, circle.y, circle.size, circle.color);

        // Create a new square
        if rand::gen_range(0, 99) >= 95 {
            let size = rand::gen_range(16.0, 64.0);

            squares.push(Shape {
                size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                y: -size,
                color: match enemy_clr.choose() {
                    Some(choice) => *choice,
                    None => Color::from_hex(0x000000),
                }
            })
        }

        // Moving squares
        for square in &mut squares {
            square.y += square.speed * delta_time;
        }

        // Keep squares still on-screen
        squares.retain(|square| square.y < screen_height() + square.size);

        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0, 
                square.y - square.size / 2.0, 
                square.size, 
                square.size,
                // enemy_clr.choose(),
                square.color,
            );
        }

        next_frame().await
    }
}