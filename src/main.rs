const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");
const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

use std::fs;
use macroquad::prelude::*;
use macroquad_particles::{self as particles, AtlasConfig, Emitter, EmitterConfig};
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use rand::ChooseRandom;

struct ScreenCenter {
    x: f32,
    y: f32,
}

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
    collided: bool,
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}

enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

fn particle_explosion() -> particles::EmitterConfig {
    particles::EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        explosiveness: 0.65,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 400.0,
        initial_velocity_randomness: 0.8,
        size: 16.0,
        size_randomness: 0.3,
        atlas: Some(AtlasConfig::new(5, 1, 0..)),
        ..Default::default()
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Agical Macroquad Tutorial".to_owned(),
        // window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Setting the asset folder
    set_pc_assets_folder("assets");

    // seeding the RNG
    rand::srand(miniquad::date::now() as u64);

    // Setting shapes
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut explosions: Vec<(Emitter, Vec2)> = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        color: Color::from_hex(0xb1de78),
        collided: false,
    };

    let mut pc_last_dir_change: f32 = 0.0;

    let mut game_state = GameState::MainMenu;

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

    let mut last_shot: f64 = 0.0;

    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);


    // Starfield shader setup
    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320,150);
    render_target.texture.set_filter(FilterMode::Nearest);

    let material = load_material(
        ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float2),
                UniformDesc::new("direction_modifier", UniformType::Float1),
            ],
            ..Default::default()
        },
    ).unwrap();

    // Loading Textures
    let ship_texture: Texture2D = load_texture("ship.png")
        .await
        .expect("Could not load file!");
    ship_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D = load_texture("laser-bolts.png")
        .await
        .expect("Could not load file!");
    bullet_texture.set_filter(FilterMode::Nearest);

    // Explosion texture
    let explosion_texture: Texture2D = load_texture("explosion.png")
        .await
        .expect("Could not load file!");
    explosion_texture.set_filter(FilterMode::Nearest);

    // build texture atlas
    build_textures_atlas();

    // Setup bullet sprite
    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "bolt".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    bullet_sprite.set_animation(1);

    // Setup ship sprite
    let mut ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "slight_left".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "slight_right".to_string(),
                row: 3,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    
    loop {
        clear_background(BLACK);

        // Draw Starfield
        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);
        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture, 
            0., 
            0., WHITE, 
            DrawTextureParams{
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        let screen_center = ScreenCenter {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        };

        match game_state {
            GameState::MainMenu => {
                // Quit Game
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                
                // Start Game
                if is_key_pressed(KeyCode::Space) {
                    squares.clear();
                    bullets.clear();
                    explosions.clear();
                    circle.x = screen_center.x;
                    circle.y = screen_center.y;
                    score = 0;
                    game_state = GameState::Playing;
                }

                let text = "Press [Space]";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            GameState::Playing => {
                // Get delta time
                let delta_time = get_frame_time();

                // --- Player ---
                ship_sprite.set_animation(0);

                // Input setup
                // Check for direction change time
                if is_key_released(KeyCode::Right) || is_key_released(KeyCode::Left) {
                    pc_last_dir_change = 0.0;
                }

                if is_key_down(KeyCode::Right) {
                    circle.x += circle.speed * delta_time;
                    direction_modifier += 0.05 * delta_time;

                    // Adapting animation to direction change length
                    if pc_last_dir_change == 0.0 {
                        pc_last_dir_change = get_time() as f32;
                    }
                    if pc_last_dir_change > 0.0 && get_time() as f32 - pc_last_dir_change > 0.2 {
                        ship_sprite.set_animation(4);
                    }
                    else {
                        ship_sprite.set_animation(3);
                    }

                }
                if is_key_down(KeyCode::Left) {
                    circle.x -= circle.speed * delta_time;
                    direction_modifier -= 0.05 * delta_time;
                    
                    // Adapting animation to direction change length
                    if pc_last_dir_change == 0.0 {
                        pc_last_dir_change = get_time() as f32;
                    }

                    if pc_last_dir_change > 0.0 && get_time() as f32 - pc_last_dir_change > 0.2 {
                        ship_sprite.set_animation(2);
                    }
                    else {
                        ship_sprite.set_animation(1);
                    }
                    
                }
                if is_key_down(KeyCode::Down) {
                    circle.y += circle.speed * delta_time;
                }
                if is_key_down(KeyCode::Up) {
                    circle.y -= circle.speed * delta_time;
                }

                // Pause Game
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }

                circle.x = clamp(circle.x, 0.0 + circle.size, screen_width() - circle.size);
                circle.y = clamp(circle.y, 0.0 + circle.size, screen_height() - circle.size);

                // Did player shoot? Has it been 0.25 seconds since the last shot?
                if is_key_down(KeyCode::Space) && (get_time() - last_shot) > 0.25 {
                    bullets.push(Shape {
                        x: circle.x,
                        y: circle.y - 24.0,
                        speed: circle.speed * 2.0,
                        size: 32.0,
                        color: WHITE,
                        collided: false,
                    });
                    last_shot = get_time();
                }

                // --- Squares ---
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
                        },
                        collided: false,
                    })
                }

                // Move the Squares
                for square in &mut squares {
                    square.y += square.speed * delta_time;
                }

                // Move the bullets
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                ship_sprite.update();
                bullet_sprite.update();
                
                // Check for collision (Lose state)
                if squares.iter().any(|square| circle.collides_with(square)) {
                    if score == high_score {
                        fs::write("highscore.dat", high_score.to_string()).ok();
                    }
                    game_state = GameState::GameOver;
                }

                // Check for bullet-square collisions
                for square in squares.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collides_with(square) {
                            bullet.collided = true;
                            square.collided = true;
                            score += square.size.round() as u32;
                            high_score = high_score.max(score);
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 4,
                                    texture: Some(explosion_texture.clone()),
                                    ..particle_explosion()
                                }),
                                vec2(square.x, square.y),
                            ));
                        }
                    }
                }

                // Keep squares that's on-screen
                squares.retain(|square| square.y < screen_height() + square.size);

                // Keep bullets that's on-screen
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

                // Retain active entities
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| !bullet.collided);
                explosions.retain(|(explosion, _)| explosion.config.emitting);
                
                // Draw bullets
                let bullet_frame = bullet_sprite.frame();
                for bullet in &bullets {
                    // draw_circle(bullet.x, bullet.y, bullet.size / 2.0, RED);
                    draw_texture_ex(
                        &bullet_texture, 
                        bullet.x - bullet.size / 2.0, 
                        bullet.y - bullet.size / 2.0, 
                        bullet.color, 
                        DrawTextureParams {
                            dest_size: Some(vec2(bullet.size, bullet.size)),
                            source: Some(bullet_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }

                // Draw the player (ship)
                // draw_circle(circle.x, circle.y, circle.size, circle.color);
                let ship_frame = ship_sprite.frame();
                draw_texture_ex(
                    &ship_texture, 
                    circle.x - ship_frame.dest_size.x, 
                    circle.y - ship_frame.dest_size.y, 
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ship_frame.dest_size * 2.0),
                        source: Some(ship_frame.source_rect),
                        ..Default::default()
                    },
                );

                // Draw the squares
                for square in &squares {
                    draw_rectangle(
                        square.x - square.size / 2.0, 
                        square.y - square.size / 2.0, 
                        square.size, 
                        square.size,
                        square.color,
                    );
                }

                // Draw explosions
                for (explosion, coords) in explosions.iter_mut() {
                    explosion.draw(*coords);
                }

                // Draw scores
                draw_text(
                    format!("Score: {}", score).as_str(),
                    10.0,
                    35.0,
                    25.0,
                    WHITE,
                );

                let highscore_text = format!("High score: {}", high_score);
                let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
                draw_text(
                    highscore_text.as_str(),
                    screen_width() - text_dimensions.width - 10.0,
                    35.0,
                    25.0,
                    YELLOW,
                );
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_center.x - text_dimensions.width / 2.0,
                    screen_center.y,
                    50.0,
                    WHITE,
                );
                let instruction_txt = "Resume with [Space]";
                let instruct_txt_dim = measure_text(instruction_txt, None, 24, 1.0);
                draw_text(
                    instruction_txt,
                    screen_center.x - instruct_txt_dim.width / 2.0,
                    25.0 + (instruct_txt_dim.offset_y / 2.0),
                    24.0,
                    WHITE,
                );
            }
            GameState::GameOver => {
                // Responding to player input
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu;
                }

                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_center.x - text_dimensions.width / 2.0,
                    screen_center.y + text_dimensions.offset_y / 2.0,
                    50.0,
                    RED,
                );

                if score == high_score {
                    let hiscore_congrats_txt = format!("Congrats on beating the high score with {}!", score);
                    let hiscore_text_dim = measure_text(&hiscore_congrats_txt, None, 24, 1.0);
                    draw_text(
                        &hiscore_congrats_txt,
                        screen_center.x - hiscore_text_dim.width / 2.0,
                        screen_center.y + (text_dimensions.offset_y / 2.0) + 25.0 + (hiscore_text_dim.offset_y / 2.0),
                        24.0,
                        WHITE,
                    );
                }

                // Instructions
                let instruction_txt = "Return to main menu with [Space]";
                let instruct_txt_dim = measure_text(instruction_txt, None, 24, 1.0);
                draw_text(
                    instruction_txt,
                    screen_center.x - instruct_txt_dim.width / 2.0,
                    25.0 + (instruct_txt_dim.offset_y / 2.0),
                    24.0,
                    WHITE,
                );
            }
        }


        next_frame().await
    }
}