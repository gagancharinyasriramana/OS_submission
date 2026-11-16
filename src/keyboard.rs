// Keyboard driver (IRQ1, port 0x60, PS/2 Set 1 scancodes)
use crate::pic;
use crate::ports::inb;

const KEY_BUFFER_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
    Tab,
    Esc,
}

struct KeyBuffer {
    buffer: [Key; KEY_BUFFER_SIZE],
    head: usize,
    tail: usize,
}

impl KeyBuffer {
    const fn new() -> Self {
        KeyBuffer {
            buffer: [Key::Esc; KEY_BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    fn push(&mut self, key: Key) {
        let next = (self.tail + 1) % KEY_BUFFER_SIZE;
        if next != self.head {
            self.buffer[self.tail] = key;
            self.tail = next;
        }
    }

    fn pop(&mut self) -> Option<Key> {
        if self.head == self.tail {
            None
        } else {
            let key = self.buffer[self.head];
            self.head = (self.head + 1) % KEY_BUFFER_SIZE;
            Some(key)
        }
    }
}

static mut KEY_BUFFER: KeyBuffer = KeyBuffer::new();
static mut SHIFT_PRESSED: bool = false;
static mut EXTENDED_SCANCODE: bool = false;

pub fn init() {
    pic::clear_mask(1); // Unmask IRQ1
}

#[no_mangle]
pub extern "C" fn irq1_handler() {
    unsafe {
        // SAFETY: Read scancode from keyboard data port 0x60
        let scancode = inb(0x60);
        process_scancode(scancode);
    }
    pic::send_eoi(1);
}

unsafe fn process_scancode(code: u8) {
    if code == 0xE0 {
        EXTENDED_SCANCODE = true;
        return;
    }

    if EXTENDED_SCANCODE {
        EXTENDED_SCANCODE = false;
        // Extended scancode
        let is_release = (code & 0x80) != 0;
        let make_code = code & 0x7F;

        if !is_release {
            match make_code {
                0x48 => KEY_BUFFER.push(Key::Up),
                0x50 => KEY_BUFFER.push(Key::Down),
                0x4B => KEY_BUFFER.push(Key::Left),
                0x4D => KEY_BUFFER.push(Key::Right),
                _ => {}
            }
        }
        return;
    }

    let is_release = (code & 0x80) != 0;
    let make_code = code & 0x7F;

    // Shift handling (0x2A = Left Shift, 0x36 = Right Shift)
    if make_code == 0x2A || make_code == 0x36 {
        SHIFT_PRESSED = !is_release;
        return;
    }

    if is_release {
        return;
    }

    // Key press
    let shift = SHIFT_PRESSED;
    let key = match make_code {
        0x01 => Some(Key::Esc),
        0x0E => Some(Key::Backspace),
        0x0F => Some(Key::Tab),
        0x1C => Some(Key::Enter),

        // Number row
        0x02 => Some(Key::Char(if shift { '!' } else { '1' })),
        0x03 => Some(Key::Char(if shift { '@' } else { '2' })),
        0x04 => Some(Key::Char(if shift { '#' } else { '3' })),
        0x05 => Some(Key::Char(if shift { '$' } else { '4' })),
        0x06 => Some(Key::Char(if shift { '%' } else { '5' })),
        0x07 => Some(Key::Char(if shift { '^' } else { '6' })),
        0x08 => Some(Key::Char(if shift { '&' } else { '7' })),
        0x09 => Some(Key::Char(if shift { '*' } else { '8' })),
        0x0A => Some(Key::Char(if shift { '(' } else { '9' })),
        0x0B => Some(Key::Char(if shift { ')' } else { '0' })),
        0x0C => Some(Key::Char(if shift { '_' } else { '-' })),
        0x0D => Some(Key::Char(if shift { '+' } else { '=' })),

        // QWERTY Row 1
        0x10 => Some(Key::Char(if shift { 'Q' } else { 'q' })),
        0x11 => Some(Key::Char(if shift { 'W' } else { 'w' })),
        0x12 => Some(Key::Char(if shift { 'E' } else { 'e' })),
        0x13 => Some(Key::Char(if shift { 'R' } else { 'r' })),
        0x14 => Some(Key::Char(if shift { 'T' } else { 't' })),
        0x15 => Some(Key::Char(if shift { 'Y' } else { 'y' })),
        0x16 => Some(Key::Char(if shift { 'U' } else { 'u' })),
        0x17 => Some(Key::Char(if shift { 'I' } else { 'i' })),
        0x18 => Some(Key::Char(if shift { 'O' } else { 'o' })),
        0x19 => Some(Key::Char(if shift { 'P' } else { 'p' })),
        0x1A => Some(Key::Char(if shift { '{' } else { '[' })),
        0x1B => Some(Key::Char(if shift { '}' } else { ']' })),
        0x2B => Some(Key::Char(if shift { '|' } else { '\\' })),

        // ASDF Row 2
        0x1E => Some(Key::Char(if shift { 'A' } else { 'a' })),
        0x1F => Some(Key::Char(if shift { 'S' } else { 's' })),
        0x20 => Some(Key::Char(if shift { 'D' } else { 'd' })),
        0x21 => Some(Key::Char(if shift { 'F' } else { 'f' })),
        0x22 => Some(Key::Char(if shift { 'G' } else { 'g' })),
        0x23 => Some(Key::Char(if shift { 'H' } else { 'h' })),
        0x24 => Some(Key::Char(if shift { 'J' } else { 'j' })),
        0x25 => Some(Key::Char(if shift { 'K' } else { 'k' })),
        0x26 => Some(Key::Char(if shift { 'L' } else { 'l' })),
        0x27 => Some(Key::Char(if shift { ':' } else { ';' })),
        0x28 => Some(Key::Char(if shift { '"' } else { '\'' })),
        0x29 => Some(Key::Char(if shift { '~' } else { '`' })),

        // ZXCV Row 3
        0x2C => Some(Key::Char(if shift { 'Z' } else { 'z' })),
        0x2D => Some(Key::Char(if shift { 'X' } else { 'x' })),
        0x2E => Some(Key::Char(if shift { 'C' } else { 'c' })),
        0x2F => Some(Key::Char(if shift { 'V' } else { 'v' })),
        0x30 => Some(Key::Char(if shift { 'B' } else { 'b' })),
        0x31 => Some(Key::Char(if shift { 'N' } else { 'n' })),
        0x32 => Some(Key::Char(if shift { 'M' } else { 'm' })),
        0x33 => Some(Key::Char(if shift { '<' } else { ',' })),
        0x34 => Some(Key::Char(if shift { '>' } else { '.' })),
        0x35 => Some(Key::Char(if shift { '?' } else { '/' })),

        // Space
        0x39 => Some(Key::Char(' ')),

        _ => None,
    };

    if let Some(k) = key {
        KEY_BUFFER.push(k);
    }
}

pub fn pop_key() -> Option<Key> {
    unsafe {
        // Critical section: disable interrupts briefly while popping
        let flags: u32;
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            "cli",
            out(reg) flags,
            options(nomem, nostack)
        );

        let key = KEY_BUFFER.pop();

        if (flags & 0x200) != 0 {
            core::arch::asm!("sti");
        }
        key
    }
}

pub fn get_key() -> Key {
    loop {
        if let Some(key) = pop_key() {
            return key;
        }
        unsafe {
            // Sleep until interrupt arrives
            core::arch::asm!("sti; hlt");
        }
    }
}
