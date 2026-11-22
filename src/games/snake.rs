// Classic Snake Game for RustOS
extern crate alloc;

use alloc::vec::Vec;
use crate::games::Rng;
use crate::keyboard::{pop_key, Key};
use crate::rtc;
use crate::timer;
use crate::vga::{Color, ColorCode, BUFFER_WIDTH, WRITER};

const FIELD_WIDTH: usize = 50;
const FIELD_HEIGHT: usize = 20;
const OFFSET_X: usize = 15;
const OFFSET_Y: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn run() {
    let mut rng = Rng::new(timer::get_ticks() ^ (rtc::read_datetime().second as u32));

    'restart: loop {
        unsafe {
            WRITER.clear_screen();
        }

        // Draw border
        draw_border();

        let mut snake: Vec<Point> = Vec::new();
        snake.push(Point { x: FIELD_WIDTH / 2, y: FIELD_HEIGHT / 2 });
        snake.push(Point { x: FIELD_WIDTH / 2 - 1, y: FIELD_HEIGHT / 2 });
        snake.push(Point { x: FIELD_WIDTH / 2 - 2, y: FIELD_HEIGHT / 2 });

        let mut dir = Direction::Right;
        let mut score: u32 = 0;
        let mut food = spawn_food(&snake, &mut rng);

        draw_food(food);
        draw_score(score);

        // Game loop
        loop {
            // Check keyboard input
            while let Some(key) = pop_key() {
                match key {
                    Key::Up if dir != Direction::Down => dir = Direction::Up,
                    Key::Down if dir != Direction::Up => dir = Direction::Down,
                    Key::Left if dir != Direction::Right => dir = Direction::Left,
                    Key::Right if dir != Direction::Left => dir = Direction::Right,
                    Key::Esc => return, // Return to menu
                    _ => {}
                }
            }

            // Calculate new head position
            let head = snake[0];
            let new_head = match dir {
                Direction::Up => {
                    if head.y == 0 {
                        break; // Wall collision
                    }
                    Point { x: head.x, y: head.y - 1 }
                }
                Direction::Down => {
                    if head.y >= FIELD_HEIGHT - 1 {
                        break; // Wall collision
                    }
                    Point { x: head.x, y: head.y + 1 }
                }
                Direction::Left => {
                    if head.x == 0 {
                        break; // Wall collision
                    }
                    Point { x: head.x - 1, y: head.y }
                }
                Direction::Right => {
                    if head.x >= FIELD_WIDTH - 1 {
                        break; // Wall collision
                    }
                    Point { x: head.x + 1, y: head.y }
                }
            };

            // Check self-collision
            if snake.contains(&new_head) {
                break; // Game over
            }

            // Move snake
            snake.insert(0, new_head);
            draw_char(new_head.x, new_head.y, b'O', Color::LightGreen, Color::Black);
            if snake.len() > 1 {
                draw_char(snake[1].x, snake[1].y, b'o', Color::Green, Color::Black);
            }

            // Check if food eaten
            if new_head == food {
                score += 10;
                draw_score(score);
                food = spawn_food(&snake, &mut rng);
                draw_food(food);
            } else {
                let tail = snake.pop().unwrap();
                draw_char(tail.x, tail.y, b' ', Color::Black, Color::Black);
            }

            // Speed based on score
            let delay = if score > 150 { 70 } else if score > 80 { 90 } else { 120 };
            timer::sleep_ms(delay);
        }

        // Game Over screen
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

fn draw_char(x: usize, y: usize, ch: u8, fg: Color, bg: Color) {
    unsafe {
        WRITER.write_char_at(OFFSET_X + x, OFFSET_Y + y, ch, ColorCode::new(fg, bg));
    }
}

fn draw_border() {
    let color = ColorCode::new(Color::Yellow, Color::Black);
    unsafe {
        for x in 0..=FIELD_WIDTH + 1 {
            WRITER.write_char_at(OFFSET_X - 1 + x, OFFSET_Y - 1, b'#', color);
            WRITER.write_char_at(OFFSET_X - 1 + x, OFFSET_Y + FIELD_HEIGHT, b'#', color);
        }
        for y in 0..=FIELD_HEIGHT {
            WRITER.write_char_at(OFFSET_X - 1, OFFSET_Y + y, b'#', color);
            WRITER.write_char_at(OFFSET_X + FIELD_WIDTH, OFFSET_Y + y, b'#', color);
        }
    }
}

fn draw_food(food: Point) {
    draw_char(food.x, food.y, b'*', Color::LightRed, Color::Black);
}

fn draw_score(score: u32) {
    let s = alloc::format!(" SNAKE GAME | Score: {:<4} | Controls: Arrows | Esc: Exit ", score);
    unsafe {
        let color = ColorCode::new(Color::White, Color::Blue);
        for (i, b) in s.bytes().enumerate() {
            if OFFSET_X + i < BUFFER_WIDTH {
                WRITER.write_char_at(OFFSET_X + i, 0, b, color);
            }
        }
    }
}

fn spawn_food(snake: &[Point], rng: &mut Rng) -> Point {
    loop {
        let x = rng.range(0, (FIELD_WIDTH - 1) as u32) as usize;
        let y = rng.range(0, (FIELD_HEIGHT - 1) as u32) as usize;
        let p = Point { x, y };
        if !snake.contains(&p) {
            return p;
        }
    }
}

fn draw_game_over(score: u32) {
    let msg1 = "=== GAME OVER ===";
    let msg2 = alloc::format!("Final Score: {}", score);
    let msg3 = "Press [Enter] to Restart or [Esc] to Return";

    unsafe {
        let fg = Color::LightRed;
        let bg = Color::Black;
        let y = OFFSET_Y + FIELD_HEIGHT / 2 - 1;

        let start_x = (BUFFER_WIDTH - msg1.len()) / 2;
        for (i, b) in msg1.bytes().enumerate() {
            WRITER.write_char_at(start_x + i, y, b, ColorCode::new(fg, bg));
        }

        let start_x2 = (BUFFER_WIDTH - msg2.len()) / 2;
        for (i, b) in msg2.bytes().enumerate() {
            WRITER.write_char_at(start_x2 + i, y + 1, b, ColorCode::new(Color::Yellow, bg));
        }

        let start_x3 = (BUFFER_WIDTH - msg3.len()) / 2;
        for (i, b) in msg3.bytes().enumerate() {
            WRITER.write_char_at(start_x3 + i, y + 2, b, ColorCode::new(Color::White, bg));
        }
    }
}
