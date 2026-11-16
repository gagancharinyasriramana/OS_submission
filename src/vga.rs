// VGA text mode driver (80x25, buffer at 0xB8000)
use crate::ports::outb;
use core::fmt;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDR: usize = 0xB8000;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    buffer: *mut ScreenChar,
}

// SAFETY: Writer is single-threaded or protected by interrupt disabling
unsafe impl Send for Writer {}

impl Writer {
    pub const fn new() -> Self {
        Writer {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            buffer: VGA_BUFFER_ADDR as *mut ScreenChar,
        }
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.column_position = 0,
            0x08 => {
                // Backspace
                if self.column_position > 0 {
                    self.column_position -= 1;
                    self.write_char_at(self.column_position, self.row_position, b' ', self.color_code);
                }
            }
            0x09 => {
                // Tab (advance to next multiple of 4)
                let next_tab = (self.column_position + 4) & !3;
                while self.column_position < next_tab && self.column_position < BUFFER_WIDTH {
                    self.write_char_at(self.column_position, self.row_position, b' ', self.color_code);
                    self.column_position += 1;
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                self.write_char_at(col, row, byte, self.color_code);
                self.column_position += 1;
            }
        }
        self.update_hardware_cursor();
    }

    pub fn write_char_at(&mut self, col: usize, row: usize, ascii: u8, color: ColorCode) {
        if col < BUFFER_WIDTH && row < BUFFER_HEIGHT {
            let offset = row * BUFFER_WIDTH + col;
            unsafe {
                // SAFETY: offset is within 0..2000, pointing into 0xB8000..0xB8FA0
                core::ptr::write_volatile(
                    self.buffer.add(offset),
                    ScreenChar {
                        ascii_character: ascii,
                        color_code: color,
                    },
                );
            }
        }
    }

    pub fn read_char_at(&self, col: usize, row: usize) -> u8 {
        if col < BUFFER_WIDTH && row < BUFFER_HEIGHT {
            let offset = row * BUFFER_WIDTH + col;
            unsafe {
                core::ptr::read_volatile(self.buffer.add(offset)).ascii_character
            }
        } else {
            b' '
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn new_line(&mut self) {
        self.column_position = 0;
        // Don't scroll into the status bar (row 24)
        if self.row_position < BUFFER_HEIGHT - 2 {
            self.row_position += 1;
        } else {
            self.scroll();
        }
    }

    pub fn scroll(&mut self) {
        // Scroll lines 0..23 up by 1 line, leaving row 24 as status bar
        for row in 1..BUFFER_HEIGHT - 1 {
            for col in 0..BUFFER_WIDTH {
                let src_offset = row * BUFFER_WIDTH + col;
                let dst_offset = (row - 1) * BUFFER_WIDTH + col;
                unsafe {
                    let character = core::ptr::read_volatile(self.buffer.add(src_offset));
                    core::ptr::write_volatile(self.buffer.add(dst_offset), character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 2);
        self.row_position = BUFFER_HEIGHT - 2;
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            let offset = row * BUFFER_WIDTH + col;
            unsafe {
                core::ptr::write_volatile(self.buffer.add(offset), blank);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let offset = row * BUFFER_WIDTH + col;
                unsafe {
                    core::ptr::write_volatile(self.buffer.add(offset), blank);
                }
            }
        }
        self.column_position = 0;
        self.row_position = 0;
        self.update_hardware_cursor();
    }

    pub fn set_cursor_pos(&mut self, col: usize, row: usize) {
        self.column_position = col.min(BUFFER_WIDTH - 1);
        self.row_position = row.min(BUFFER_HEIGHT - 1);
        self.update_hardware_cursor();
    }

    pub fn update_hardware_cursor(&self) {
        let pos = (self.row_position * BUFFER_WIDTH + self.column_position) as u16;
        unsafe {
            // SAFETY: VGA ports 0x3D4 (index) and 0x3D5 (data) control hardware cursor
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn hide_cursor(&self) {
        unsafe {
            outb(0x3D4, 0x0A);
            outb(0x3D5, 0x20); // Bit 5 disables cursor
        }
    }

    pub fn show_cursor(&self) {
        unsafe {
            outb(0x3D4, 0x0A);
            outb(0x3D5, 0x0D); // Cursor start line
            outb(0x3D4, 0x0B);
            outb(0x3D5, 0x0E); // Cursor end line
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static mut WRITER: Writer = Writer::new();

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        // SAFETY: Single-threaded text output
        let _ = WRITER.write_fmt(args);
    }
}
