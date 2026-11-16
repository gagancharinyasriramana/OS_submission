// Serial port COM1 (0x3F8) driver for VirtualBox debugging and boot logging
use crate::ports::{inb, outb};
use core::fmt;

const COM1: u16 = 0x3F8;

pub struct Serial;

impl Serial {
    pub fn init(&mut self) {
        unsafe {
            // SAFETY: Standard 16550 UART initialization sequence
            outb(COM1 + 1, 0x00); // Disable all interrupts
            outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(COM1 + 1, 0x00); //                  (hi byte)
            outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
            outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            // SAFETY: wait until transmit FIFO is empty
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, byte);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub static mut SERIAL: Serial = Serial;

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        // SAFETY: Single-threaded OS serial logging
        let _ = SERIAL.write_fmt(args);
    }
}
