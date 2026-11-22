// Car Dodge Game for RustOS
// 3 Lanes, falling obstacles at increasing speed, session high score
extern crate alloc;

use alloc::vec::Vec;
use crate::games::Rng;
use crate::keyboard::{pop_key, Key};
use crate::rtc;
use crate::timer;
use crate::vga::{Color, ColorCode, BUFFER_WIDTH, WRITER};

static mut HIGH_SCORE: u32 = 0;

const ROAD_WIDTH: usize = 25;
const START_X: usize = (BUFFER_WIDTH - ROAD_WIDTH) / 2; // Centered
const ROAD_HEIGHT: usize = 20;
const START_Y: usize = 2;

// Lane centers: Left (lane 0), Middle (lane 1), Right (lane 2)
const LANE_CENTERS: [usize; 3] = [START_X + 4, START_X + 12, START_X + 20];

#[derive(Clone, Copy)]
struct Obstacle {
    lane: usize, // 0, 1, or 2
    y: usize,
}

pub fn run() {
    let mut rng = Rng::new(timer::get_ticks() ^ (rtc::read_datetime().second as u32));

    'restart: loop {
        unsafe {
            WRITER.clear_screen();
        }

        let mut player_lane: usize = 1; // Start in middle lane
        let player_y = START_Y + ROAD_HEIGHT - 3;

        let mut obstacles: Vec<Obstacle> = Vec::new();
        let mut score: u32 = 0;
        let mut tick_counter: u32 = 0;
        let mut spawn_interval: u32 = 8;

        draw_road_borders();
        draw_score_bar(score);

        loop {
            // Check keys
            while let Some(key) = pop_key() {
                match key {
                    Key::Left => {
                        if player_lane > 0 {
                            erase_car(player_lane, player_y);
                            player_lane -= 1;
                        }
                    }
                    Key::Right => {
                        if player_lane < 2 {
                            erase_car(player_lane, player_y);
                            player_lane += 1;
                        }
                    }
                    Key::Esc => return,
                    _ => {}
                }
            }

            draw_car(player_lane, player_y, Color::LightCyan);

            // Move obstacles
            tick_counter += 1;
            if tick_counter % spawn_interval == 0 {
                let lane = rng.range(0, 2) as usize;
                obstacles.push(Obstacle { lane, y: START_Y });
            }

            // Move obstacles down
            let mut i = 0;
            let mut collision = false;

            while i < obstacles.len() {
                erase_obstacle(obstacles[i].lane, obstacles[i].y);
                obstacles[i].y += 1;

                // Check collision with player car
                if obstacles[i].lane == player_lane && obstacles[i].y >= player_y && obstacles[i].y <= player_y + 2 {
                    collision = true;
                    break;
                }

                if obstacles[i].y >= START_Y + ROAD_HEIGHT {
                    obstacles.remove(i);
                    score += 10;
                    draw_score_bar(score);
                    // Increase difficulty
                    if score > 200 {
                        spawn_interval = 4;
                    } else if score > 100 {
                        spawn_interval = 6;
                    }
                } else {
                    draw_obstacle(obstacles[i].lane, obstacles[i].y);
                    i += 1;
                }
            }

            if collision {
                break;
            }

            // Speed delay
            let delay = if score > 150 { 50 } else if score > 70 { 75 } else { 100 };
            timer::sleep_ms(delay);
        }

        // Game over
        unsafe {
            if score > HIGH_SCORE {
                HIGH_SCORE = score;
            }
        }
        draw_game_over(score);

        loop {
            match crate::keyboard::get_key() {
                Key::Enter => continue 'restart,
                Key::Esc => return,
                _ => {}
            }
        }
    }
}

fn draw_road_borders() {
    let color = ColorCode::new(Color::White, Color::DarkGray);
    unsafe {
        for y in START_Y..START_Y + ROAD_HEIGHT {
            WRITER.write_char_at(START_X, y, b'|', color);
            WRITER.write_char_at(START_X + 8, y, b':', ColorCode::new(Color::Yellow, Color::Black));
            WRITER.write_char_at(START_X + 16, y, b':', ColorCode::new(Color::Yellow, Color::Black));
            WRITER.write_char_at(START_X + ROAD_WIDTH - 1, y, b'|', color);
        }
    }
}

fn draw_car(lane: usize, y: usize, color: Color) {
    let center = LANE_CENTERS[lane];
    unsafe {
        let code = ColorCode::new(color, Color::Black);
        // Car shape:
        //  /=\
        // [###]
        //  \=/
        WRITER.write_char_at(center - 1, y, b'/', code);
        WRITER.write_char_at(center, y, b'=', code);
        WRITER.write_char_at(center + 1, y, b'\\', code);

        WRITER.write_char_at(center - 1, y + 1, b'[', code);
        WRITER.write_char_at(center, y + 1, b'#', code);
        WRITER.write_char_at(center + 1, y + 1, b']', code);

        WRITER.write_char_at(center - 1, y + 2, b'\\', code);
        WRITER.write_char_at(center, y + 2, b'=', code);
        WRITER.write_char_at(center + 1, y + 2, b'/', code);
    }
}

fn erase_car(lane: usize, y: usize) {
    let center = LANE_CENTERS[lane];
    unsafe {
        let code = ColorCode::new(Color::Black, Color::Black);
        for row in 0..3 {
            WRITER.write_char_at(center - 1, y + row, b' ', code);
            WRITER.write_char_at(center, y + row, b' ', code);
            WRITER.write_char_at(center + 1, y + row, b' ', code);
        }
    }
}

fn draw_obstacle(lane: usize, y: usize) {
    let center = LANE_CENTERS[lane];
    unsafe {
        let code = ColorCode::new(Color::LightRed, Color::Black);
        WRITER.write_char_at(center - 1, y, b'<', code);
        WRITER.write_char_at(center, y, b'X', code);
        WRITER.write_char_at(center + 1, y, b'>', code);
    }
}

fn erase_obstacle(lane: usize, y: usize) {
    let center = LANE_CENTERS[lane];
    unsafe {
        let code = ColorCode::new(Color::Black, Color::Black);
        WRITER.write_char_at(center - 1, y, b' ', code);
        WRITER.write_char_at(center, y, b' ', code);
        WRITER.write_char_at(center + 1, y, b' ', code);
    }
}

fn draw_score_bar(score: u32) {
    unsafe {
        let hs = HIGH_SCORE;
        let s = alloc::format!(" CAR DODGE | Score: {:<4} | High Score: {:<4} | Left/Right: Steer | Esc: Exit ", score, hs);
        let color = ColorCode::new(Color::White, Color::Blue);
        for (i, b) in s.bytes().enumerate() {
            if i < BUFFER_WIDTH {
                WRITER.write_char_at(i, 0, b, color);
            }
        }
    }
}

fn draw_game_over(score: u32) {
    let msg1 = "=== CAR CRASHED! GAME OVER ===";
    let msg2 = alloc::format!("Score: {}  High Score: {}", score, unsafe { HIGH_SCORE });
    let msg3 = "Press [Enter] to Retry or [Esc] to Return";

    unsafe {
        let y = START_Y + 7;
        let start_x1 = (BUFFER_WIDTH - msg1.len()) / 2;
        let start_x2 = (BUFFER_WIDTH - msg2.len()) / 2;
        let start_x3 = (BUFFER_WIDTH - msg3.len()) / 2;

        let red = ColorCode::new(Color::LightRed, Color::Black);
        let yellow = ColorCode::new(Color::Yellow, Color::Black);
        let white = ColorCode::new(Color::White, Color::Black);

        for (i, b) in msg1.bytes().enumerate() {
            WRITER.write_char_at(start_x1 + i, y, b, red);
        }
        for (i, b) in msg2.bytes().enumerate() {
            WRITER.write_char_at(start_x2 + i, y + 1, b, yellow);
        }
        for (i, b) in msg3.bytes().enumerate() {
            WRITER.write_char_at(start_x3 + i, y + 2, b, white);
        }
    }
}
