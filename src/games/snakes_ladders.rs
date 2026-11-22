// Snake and Ladders Board Game (Human vs Computer)
extern crate alloc;

use crate::games::Rng;
use crate::keyboard::{get_key, Key};
use crate::rtc;
use crate::timer;
use crate::vga::{Color, ColorCode, BUFFER_WIDTH, WRITER};

const LADDERS: [(u8, u8); 6] = [
    (4, 14), (9, 31), (20, 38), (28, 84), (40, 59), (71, 91),
];

const SNAKES: [(u8, u8); 6] = [
    (17, 7), (54, 34), (62, 18), (87, 24), (93, 73), (99, 78),
];

pub fn run() {
    let mut rng = Rng::new(timer::get_ticks() ^ (rtc::read_datetime().second as u32));

    'restart: loop {
        let mut player_pos: u8 = 1;
        let mut cpu_pos: u8 = 1;
        let mut turn = 0; // 0 = Player, 1 = CPU
        let mut message = alloc::format!("Game started! Press [Space] or [Enter] to roll dice.");

        loop {
            draw_screen(player_pos, cpu_pos, &message, turn);

            if player_pos >= 100 {
                message = alloc::format!("CONGRATULATIONS! You WON! Press [Enter] to restart.");
                draw_screen(player_pos, cpu_pos, &message, turn);
                wait_for_restart_or_esc();
                continue 'restart;
            }
            if cpu_pos >= 100 {
                message = alloc::format!("COMPUTER WON! Better luck next time! Press [Enter] to restart.");
                draw_screen(player_pos, cpu_pos, &message, turn);
                wait_for_restart_or_esc();
                continue 'restart;
            }

            if turn == 0 {
                // Player's turn: wait for Space or Enter
                loop {
                    match get_key() {
                        Key::Char(' ') | Key::Enter => break,
                        Key::Esc => return,
                        _ => {}
                    }
                }

                let roll = rng.range(1, 6) as u8;
                let (new_pos, event) = move_token(player_pos, roll);
                player_pos = new_pos;
                message = alloc::format!("You rolled a {}! {}", roll, event);
                turn = 1;
            } else {
                // CPU's turn
                timer::sleep_ms(700);
                let roll = rng.range(1, 6) as u8;
                let (new_pos, event) = move_token(cpu_pos, roll);
                cpu_pos = new_pos;
                message = alloc::format!("Computer rolled a {}! {}", roll, event);
                turn = 0;
            }
        }
    }
}

fn move_token(curr: u8, roll: u8) -> (u8, &'static str) {
    if curr + roll > 100 {
        return (curr, "Overshot 100, stay in place.");
    }

    let pos = curr + roll;
    for &(start, end) in LADDERS.iter() {
        if pos == start {
            return (end, "CLIMBED A LADDER! ^_^");
        }
    }
    for &(start, end) in SNAKES.iter() {
        if pos == start {
            return (end, "BITTEN BY A SNAKE! v_v");
        }
    }

    (pos, "Moved forward.")
}

fn draw_screen(player_pos: u8, cpu_pos: u8, msg: &str, turn: usize) {
    unsafe {
        WRITER.clear_screen();
    }

    // Title bar
    let title = " SNAKES & LADDERS (1-100) | Human (P) vs Computer (C) | Esc: Menu ";
    draw_line_at(0, title, Color::White, Color::Blue);

    // 10x10 Grid representation
    // Row 0 has 100..91, Row 1 has 81..90 (boustrophedon / alternating direction)
    let grid_x = 4;
    let grid_y = 2;

    for row in 0..10 {
        let y = grid_y + row * 2;
        for col in 0..10 {
            let x = grid_x + col * 5;
            let square_num = get_square_number(row, col);

            let mut fg = Color::White;
            let mut bg = Color::Black;

            // Highlight Ladders & Snakes
            for &(start, _) in LADDERS.iter() {
                if square_num == start {
                    fg = Color::LightGreen;
                }
            }
            for &(start, _) in SNAKES.iter() {
                if square_num == start {
                    fg = Color::LightRed;
                }
            }

            let mut token = "  ";
            if square_num == player_pos && square_num == cpu_pos {
                token = "PC";
                bg = Color::Magenta;
            } else if square_num == player_pos {
                token = " P";
                bg = Color::Cyan;
                fg = Color::Black;
            } else if square_num == cpu_pos {
                token = " C";
                bg = Color::Brown;
                fg = Color::White;
            }

            let cell_str = alloc::format!("{:2}{}", square_num, token);
            draw_str_at(x, y, &cell_str, fg, bg);
        }
    }

    // Status on the right side
    let info_x = 56;
    draw_str_at(info_x, 3, "=== GAME STATUS ===", Color::Yellow, Color::Black);
    draw_str_at(info_x, 5, &alloc::format!("Player (P) Pos:   {}", player_pos), Color::Cyan, Color::Black);
    draw_str_at(info_x, 6, &alloc::format!("Computer (C) Pos: {}", cpu_pos), Color::Brown, Color::Black);

    let turn_str = if turn == 0 { "YOUR TURN (P)" } else { "COMPUTER'S TURN (C)" };
    draw_str_at(info_x, 8, "Current Turn:", Color::White, Color::Black);
    draw_str_at(info_x, 9, turn_str, if turn == 0 { Color::LightGreen } else { Color::LightRed }, Color::Black);

    draw_str_at(info_x, 12, "Ladders (Green):", Color::LightGreen, Color::Black);
    draw_str_at(info_x, 13, "4->14, 9->31, 28->84", Color::LightGray, Color::Black);
    draw_str_at(info_x, 14, "40->59, 71->91", Color::LightGray, Color::Black);

    draw_str_at(info_x, 16, "Snakes (Red):", Color::LightRed, Color::Black);
    draw_str_at(info_x, 17, "17->7, 54->34, 62->18", Color::LightGray, Color::Black);
    draw_str_at(info_x, 18, "87->24, 93->73, 99->78", Color::LightGray, Color::Black);

    // Event message at bottom
    draw_line_at(22, msg, Color::Yellow, Color::Black);
}

fn get_square_number(row: usize, col: usize) -> u8 {
    let rev_row = 9 - row; // 0 is top (row 9), 9 is bottom (row 0)
    if rev_row % 2 == 0 {
        // Left to right
        (rev_row * 10 + col + 1) as u8
    } else {
        // Right to left
        (rev_row * 10 + (9 - col) + 1) as u8
    }
}

fn draw_str_at(x: usize, y: usize, s: &str, fg: Color, bg: Color) {
    unsafe {
        let color = ColorCode::new(fg, bg);
        for (i, b) in s.bytes().enumerate() {
            if x + i < BUFFER_WIDTH {
                WRITER.write_char_at(x + i, y, b, color);
            }
        }
    }
}

fn draw_line_at(y: usize, s: &str, fg: Color, bg: Color) {
    unsafe {
        let color = ColorCode::new(fg, bg);
        for col in 0..BUFFER_WIDTH {
            let b = if col < s.len() { s.as_bytes()[col] } else { b' ' };
            WRITER.write_char_at(col, y, b, color);
        }
    }
}

fn wait_for_restart_or_esc() {
    loop {
        match get_key() {
            Key::Enter => break,
            Key::Esc => return,
            _ => {}
        }
    }
}
