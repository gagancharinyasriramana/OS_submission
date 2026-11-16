#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(
    r#"
    .set ALIGN,    1<<0
    .set MEMINFO,  1<<1
    .set FLAGS,    ALIGN | MEMINFO
    .set MAGIC,    0x1BADB002
    .set CHECKSUM, -(MAGIC + FLAGS)

    .section .multiboot, "a"
    .align 4
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

    .section .text
    .global _start
    .type _start, @function
    _start:
        cli
        mov esp, offset stack_top
        push eax
        push ebx
        call kernel_main
    1:  hlt
        jmp 1b

    .section .bss
    .align 16
    stack_bottom:
        .skip 16384
    stack_top:
    "#
);

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") ret,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

pub unsafe fn init_serial() {
    outb(0x3F8 + 1, 0x00); // Disable all interrupts
    outb(0x3F8 + 3, 0x80); // Enable DLAB (set baud rate divisor)
    outb(0x3F8 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    outb(0x3F8 + 1, 0x00); //                  (hi byte)
    outb(0x3F8 + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(0x3F8 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    outb(0x3F8 + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

pub unsafe fn write_serial(byte: u8) {
    while (inb(0x3F8 + 5) & 0x20) == 0 {}
    outb(0x3F8, byte);
}

pub fn serial_print(s: &str) {
    for b in s.bytes() {
        unsafe {
            write_serial(b);
        }
    }
}

#[no_mangle]
pub extern "C" fn kernel_main(_mb_info: u32, _mb_magic: u32) -> ! {
    unsafe {
        init_serial();
    }
    serial_print("RustOS booted\r\n");

    // Write 'OK' to VGA text buffer at 0xB8000
    let vga = 0xB8000 as *mut u16;
    unsafe {
        *vga = 0x0F4F; // White 'O' on black
        *vga.add(1) = 0x0F4B; // White 'K' on black
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_print("KERNEL PANIC!\r\n");
    loop {}
}
