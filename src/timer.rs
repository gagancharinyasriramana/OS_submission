// 8254 Programmable Interval Timer (PIT) channel 0 driver at 100 Hz
use crate::pic;
use crate::ports::outb;
use core::sync::atomic::{AtomicU32, Ordering};

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1193182;
const TARGET_HZ: u32 = 100;
const DIVISOR: u16 = (PIT_FREQUENCY / TARGET_HZ) as u16;

static TICKS: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    unsafe {
        outb(PIT_COMMAND, 0x36);
        outb(PIT_CHANNEL0, (DIVISOR & 0xFF) as u8);
        outb(PIT_CHANNEL0, ((DIVISOR >> 8) & 0xFF) as u8);
    }
    pic::clear_mask(0); // Unmask IRQ0
}

#[no_mangle]
pub extern "C" fn irq0_handler() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    pic::send_eoi(0);
}

pub fn get_ticks() -> u32 {
    TICKS.load(Ordering::Relaxed)
}

pub fn sleep_ms(ms: u32) {
    let ticks_to_wait = (ms + 9) / 10;
    let target = get_ticks().wrapping_add(ticks_to_wait);
    while get_ticks() < target {
        unsafe {
            core::arch::asm!("sti; hlt");
        }
    }
}
