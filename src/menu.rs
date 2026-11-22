// Desktop Application Menu for RustOS
extern crate alloc;

use alloc::vec::Vec;
use crate::keyboard::{get_key, Key};
use crate::rtc;
use crate::vga::{Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, WRITER};

pub enum MenuSelection {
    Shell,
    Snake,
    SnakesLadders,
    CarDodge,
    ResearchLab,
    Files,
    About,
    Shutdown,
}

pub struct DesktopMenu {
    pub selected: usize,
    pub items: Vec<(&'static str, &'static str, MenuSelection)>,
}

impl DesktopMenu {
    pub fn new() -> Self {
        let mut items = Vec::new();
        items.push(("Shell", "Interactive command line with 25+ system utilities", MenuSelection::Shell));
        items.push(("Snake", "Classic retro grid snake game with score & food", MenuSelection::Snake));
        items.push(("Snake and Ladders", "Turn-based 10x10 board game vs Computer AI", MenuSelection::SnakesLadders));
        items.push(("Car Dodge", "3-lane high-speed obstacle avoidance racer", MenuSelection::CarDodge));
        items.push(("Workload-Aware Lab", "CPU Scheduling Research & Deadlock Detection", MenuSelection::ResearchLab));
        items.push(("Files (Text Editor)", "Browse in-memory files and launch vi editor", MenuSelection::Files));
        items.push(("About RustOS", "System details, architecture, and author info", MenuSelection::About));
        items.push(("Shutdown", "Cleanly halt processor and prepare VM poweroff", MenuSelection::Shutdown));

        DesktopMenu { selected: 0, items }
    }

    pub fn run(&mut self) -> MenuSelection {
        loop {
            self.draw();
            match get_key() {
                Key::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = self.items.len() - 1;
                    }
                }
                Key::Down => {
                    if self.selected + 1 < self.items.len() {
                        self.selected += 1;
                    } else {
                        self.selected = 0;
                    }
                }
                Key::Enter => {
                    return match self.selected {
                        0 => MenuSelection::Shell,
                        1 => MenuSelection::Snake,
                        2 => MenuSelection::SnakesLadders,
                        3 => MenuSelection::CarDodge,
                        4 => MenuSelection::ResearchLab,
                        5 => MenuSelection::Files,
                        6 => MenuSelection::About,
                        7 => MenuSelection::Shutdown,
                        _ => MenuSelection::Shell,
                    };
                }
                Key::Esc => {
                    return MenuSelection::Shell;
                }
                _ => {}
            }
        }
    }

    fn draw(&self) {
        unsafe {
            WRITER.clear_screen();
        }

        // Header banner
        draw_line_at(0, "================================================================================", Color::Cyan, Color::Black);
        draw_line_at(1, "         R U S T O S   D E S K T O P   A P P L I C A T I O N   M E N U         ", Color::Yellow, Color::Blue);
        draw_line_at(2, "================================================================================", Color::Cyan, Color::Black);

        // Subtitle
        draw_str_center(4, "Select an application using [Up/Down] arrows and press [Enter]", Color::LightGray, Color::Black);
        draw_str_center(5, "Press [Esc] at any time to switch to the Interactive Shell", Color::DarkGray, Color::Black);

        // Menu cards
        let start_y = 7;
        for (i, item) in self.items.iter().enumerate() {
            let y = start_y + i * 2;
            let is_sel = i == self.selected;

            let (fg, bg) = if is_sel {
                (Color::Black, Color::LightCyan)
            } else {
                (Color::White, Color::Black)
            };

            let prefix = if is_sel { " >> " } else { "    " };
            let line = alloc::format!("{}{:<22} - {}", prefix, item.0, item.1);

            let card_x = 8;
            draw_str_padded(card_x, y, &line, 64, fg, bg);
        }

        // Hint box
        let hint_y = 22;
        draw_str_center(hint_y, "Note: Mouse support is planned for a future release; use keyboard.", Color::DarkGray, Color::Black);

        // Status bar at row 24
        let dt = rtc::read_datetime();
        let status = alloc::format!(
            " RustOS 0.1.0 | App: Desktop Menu   | {:02}:{:02}:{:02} {} {:04}-{:02}-{:02} ",
            dt.hour, dt.minute, dt.second, rtc::weekday_name(dt.weekday), dt.year, dt.month, dt.day
        );
        draw_line_at(BUFFER_HEIGHT - 1, &status, Color::Black, Color::LightGray);
    }
}

pub fn show_about() {
    unsafe {
        WRITER.clear_screen();
    }

    draw_line_at(0, " ABOUT RUSTOS OPERATING SYSTEM ", Color::White, Color::Blue);
    let lines = [
        "",
        "RustOS - Educational 32-bit x86 Protected Mode Operating System",
        "Version: 0.1.0 (Faculty Deliverable 2026)",
        "",
        "Key Subsystems:",
        "  - Bootloader: GNU GRUB 2 via Multiboot Specification (ISO image)",
        "  - Target Architecture: 32-bit x86 (i686 bare metal, no_std, no_main)",
        "  - Core Drivers: 80x25 VGA Text Mode, COM1 Serial UART (0x3F8)",
        "  - Interrupts: GDT, IDT (256 vectors), 8259A PIC, 8254 PIT (100 Hz timer)",
        "  - Input: PS/2 Keyboard with IRQ1 interrupt scancode queue",
        "  - Time: CMOS Real-Time Clock (RTC) with BCD decoding & calendar",
        "  - Memory: 4 MiB Linked-List Heap Allocator with block coalescing",
        "  - File System: Virtual in-memory file table with permissions (6/4)",
        "  - Research Lab: Workload-Aware CPU Scheduling & Deadlock Detection",
        "",
        "Author: Senior Systems Engineering Team",
        "",
        "Press [Enter] or [Esc] to return to the menu...",
    ];

    for (i, line) in lines.iter().enumerate() {
        draw_str_at(4, 2 + i, line, Color::White, Color::Black);
    }

    loop {
        match get_key() {
            Key::Enter | Key::Esc => break,
            _ => {}
        }
    }
}

fn draw_line_at(y: usize, s: &str, fg: Color, bg: Color) {
    unsafe {
        let code = ColorCode::new(fg, bg);
        for col in 0..BUFFER_WIDTH {
            let b = if col < s.len() { s.as_bytes()[col] } else { b' ' };
            WRITER.write_char_at(col, y, b, code);
        }
    }
}

fn draw_str_at(x: usize, y: usize, s: &str, fg: Color, bg: Color) {
    unsafe {
        let code = ColorCode::new(fg, bg);
        for (i, b) in s.bytes().enumerate() {
            if x + i < BUFFER_WIDTH {
                WRITER.write_char_at(x + i, y, b, code);
            }
        }
    }
}

fn draw_str_padded(x: usize, y: usize, s: &str, width: usize, fg: Color, bg: Color) {
    unsafe {
        let code = ColorCode::new(fg, bg);
        for i in 0..width {
            let b = if i < s.len() { s.as_bytes()[i] } else { b' ' };
            if x + i < BUFFER_WIDTH {
                WRITER.write_char_at(x + i, y, b, code);
            }
        }
    }
}

fn draw_str_center(y: usize, s: &str, fg: Color, bg: Color) {
    let x = if s.len() < BUFFER_WIDTH {
        (BUFFER_WIDTH - s.len()) / 2
    } else {
        0
    };
    draw_str_at(x, y, s, fg, bg);
}
