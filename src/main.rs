#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use core::arch::global_asm;
use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod idt;
pub mod keyboard;
pub mod pic;
pub mod ports;
pub mod rtc;
pub mod serial;
pub mod timer;
pub mod vga;

use alloc::string::String;
use alloc::vec::Vec;
use vga::{Color, WRITER};

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
        .skip 65536
    stack_top:
    "#
);

fn print_boot_step(step: &str) {
    print!("  * {:<32} [", step);
    unsafe {
        WRITER.set_color(Color::LightGreen, Color::Black);
    }
    print!(" OK ");
    unsafe {
        WRITER.set_color(Color::White, Color::Black);
    }
    println!("]");
    serial_println!("[OK] {}", step);
}

#[no_mangle]
pub extern "C" fn kernel_main(_mb_info: u32, _mb_magic: u32) -> ! {
    // 1. Serial Port COM1
    unsafe {
        serial::SERIAL.init();
    }
    serial_println!("==================================================");
    serial_println!("RustOS Kernel 0.1.0 Initializing (i686-pc-none)...");
    serial_println!("==================================================");

    // 2. VGA Text Driver
    unsafe {
        WRITER.clear_screen();
        WRITER.set_color(Color::LightCyan, Color::Black);
    }
    println!("================================================================================");
    println!("       RustOS Operating System - 32-bit x86 Protected Mode Kernel       ");
    println!("================================================================================");
    unsafe {
        WRITER.set_color(Color::White, Color::Black);
    }
    println!("\nInitializing core kernel subsystems:\n");

    print_boot_step("Serial COM1 (0x3F8)");
    print_boot_step("VGA Text Driver (80x25)");

    // 3. GDT
    gdt::init();
    print_boot_step("Global Descriptor Table (GDT)");

    // 4. PIC Remap
    pic::remap(0x20, 0x28);
    print_boot_step("8259A PIC Remap (0x20-0x2F)");

    // 5. IDT
    idt::init();
    print_boot_step("Interrupt Descriptor Table (IDT)");

    // 6. PIT Timer
    timer::init();
    print_boot_step("8254 PIT Timer (100 Hz)");

    // 7. Keyboard Driver
    keyboard::init();
    print_boot_step("PS/2 Keyboard (IRQ1)");

    // 8. Heap Allocator
    unsafe {
        allocator::ALLOCATOR.init();
    }
    print_boot_step("4 MiB Linked-List Heap");

    // Enable CPU Interrupts
    unsafe {
        core::arch::asm!("sti");
    }
    print_boot_step("CPU Interrupts Enabled (STI)");

    // Test Allocator with Vec and String
    let mut test_vec = Vec::new();
    test_vec.push("Vec OK");
    test_vec.push("String OK");
    let test_str = String::from("RustOS Heap Active");
    serial_println!("{}: {} elements", test_str, test_vec.len());

    // Read RTC
    let now = rtc::read_datetime();
    serial_println!(
        "RTC Initialized: {:04}-{:02}-{:02} {:02}:{:02}:{:02} ({})",
        now.year, now.month, now.day, now.hour, now.minute, now.second,
        rtc::weekday_name(now.weekday)
    );

    println!("\nRustOS booted successfully!");
    serial_println!("RustOS booted");

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        WRITER.set_color(Color::LightRed, Color::Black);
    }
    println!("\n!!!!!!!!!!!!!!!! KERNEL PANIC !!!!!!!!!!!!!!!!");
    if let Some(location) = info.location() {
        println!("Location: {}:{}", location.file(), location.line());
    }
    if let Some(msg) = info.message() {
        println!("Message: {}", msg);
        serial_println!("KERNEL PANIC: {}", msg);
    } else {
        println!("Message: (no message)");
        serial_println!("KERNEL PANIC: (no message)");
    }

    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}
