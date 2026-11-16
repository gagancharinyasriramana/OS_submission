// Interrupt Descriptor Table (IDT) and exception/IRQ dispatch
use crate::println;
use crate::serial_println;
use crate::vga::{Color, WRITER};
use core::ptr::addr_of;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }

    fn set_handler(&mut self, handler: u32) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = 0x08; // Kernel code segment
        self.zero = 0;
        self.type_attr = 0x8E; // Present, Ring 0, 32-bit Interrupt Gate
        self.offset_high = ((handler >> 16) & 0xFFFF) as u16;
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];
static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

pub fn init() {
    unsafe {
        // Populate exception handlers at runtime
        IDT[0].set_handler(isr0 as u32);
        IDT[1].set_handler(isr1 as u32);
        IDT[2].set_handler(isr2 as u32);
        IDT[3].set_handler(isr3 as u32);
        IDT[4].set_handler(isr4 as u32);
        IDT[5].set_handler(isr5 as u32);
        IDT[6].set_handler(isr6 as u32);
        IDT[7].set_handler(isr7 as u32);
        IDT[8].set_handler(isr8 as u32);
        IDT[9].set_handler(isr9 as u32);
        IDT[10].set_handler(isr10 as u32);
        IDT[11].set_handler(isr11 as u32);
        IDT[12].set_handler(isr12 as u32);
        IDT[13].set_handler(isr13 as u32);
        IDT[14].set_handler(isr14 as u32);
        IDT[15].set_handler(isr15 as u32);
        IDT[16].set_handler(isr16 as u32);
        IDT[17].set_handler(isr17 as u32);
        IDT[18].set_handler(isr18 as u32);
        IDT[19].set_handler(isr19 as u32);
        IDT[20].set_handler(isr20 as u32);
        IDT[21].set_handler(isr21 as u32);
        IDT[22].set_handler(isr22 as u32);
        IDT[23].set_handler(isr23 as u32);
        IDT[24].set_handler(isr24 as u32);
        IDT[25].set_handler(isr25 as u32);
        IDT[26].set_handler(isr26 as u32);
        IDT[27].set_handler(isr27 as u32);
        IDT[28].set_handler(isr28 as u32);
        IDT[29].set_handler(isr29 as u32);
        IDT[30].set_handler(isr30 as u32);
        IDT[31].set_handler(isr31 as u32);

        IDT[0x20].set_handler(irq0_stub as u32);
        IDT[0x21].set_handler(irq1_stub as u32);

        IDT_PTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDT_PTR.base = IDT.as_ptr() as u32;

        core::arch::asm!(
            "lidt [{0}]",
            in(reg) addr_of!(IDT_PTR),
            options(readonly, nostack, preserves_flags)
        );
    }
}

// Exception names
static EXCEPTION_NAMES: [&str; 32] = [
    "Division By Zero",
    "Debug",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

#[no_mangle]
pub extern "C" fn generic_exception_handler(vector: u32, error_code: u32) -> ! {
    let name = if (vector as usize) < EXCEPTION_NAMES.len() {
        EXCEPTION_NAMES[vector as usize]
    } else {
        "Unknown Exception"
    };

    unsafe {
        WRITER.set_color(Color::Red, Color::Black);
    }
    println!("\n================== CPU EXCEPTION ==================");
    println!("Vector: {} (0x{:02X}) - {}", vector, vector, name);
    println!("Error Code: 0x{:08X}", error_code);
    println!("System halted.");
    println!("===================================================");

    serial_println!("CPU EXCEPTION: Vector {} ({}) ErrorCode: 0x{:08X}", vector, name, error_code);

    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}

// Assembly stubs for IRQ0 (Timer) and IRQ1 (Keyboard)
extern "C" {
    fn irq0_stub();
    fn irq1_stub();
}

core::arch::global_asm!(
    r#"
    .global irq0_stub
    .type irq0_stub, @function
    irq0_stub:
        pusha
        call irq0_handler
        popa
        iretd

    .global irq1_stub
    .type irq1_stub, @function
    irq1_stub:
        pusha
        call irq1_handler
        popa
        iretd
    "#
);

// Macro to generate exception stubs
macro_rules! exception_stub {
    ($name:ident, $vec:expr, no_err) => {
        core::arch::global_asm!(concat!(
            ".global ", stringify!($name), "\n",
            stringify!($name), ":\n",
            "    push 0\n",
            "    push ", stringify!($vec), "\n",
            "    call generic_exception_handler\n",
            "    add esp, 8\n",
            "    iretd\n"
        ));
    };
    ($name:ident, $vec:expr, has_err) => {
        core::arch::global_asm!(concat!(
            ".global ", stringify!($name), "\n",
            stringify!($name), ":\n",
            "    push ", stringify!($vec), "\n",
            "    call generic_exception_handler\n",
            "    add esp, 8\n",
            "    iretd\n"
        ));
    };
}

extern "C" {
    fn isr0(); fn isr1(); fn isr2(); fn isr3();
    fn isr4(); fn isr5(); fn isr6(); fn isr7();
    fn isr8(); fn isr9(); fn isr10(); fn isr11();
    fn isr12(); fn isr13(); fn isr14(); fn isr15();
    fn isr16(); fn isr17(); fn isr18(); fn isr19();
    fn isr20(); fn isr21(); fn isr22(); fn isr23();
    fn isr24(); fn isr25(); fn isr26(); fn isr27();
    fn isr28(); fn isr29(); fn isr30(); fn isr31();
}

exception_stub!(isr0, 0, no_err);
exception_stub!(isr1, 1, no_err);
exception_stub!(isr2, 2, no_err);
exception_stub!(isr3, 3, no_err);
exception_stub!(isr4, 4, no_err);
exception_stub!(isr5, 5, no_err);
exception_stub!(isr6, 6, no_err);
exception_stub!(isr7, 7, no_err);
exception_stub!(isr8, 8, has_err);
exception_stub!(isr9, 9, no_err);
exception_stub!(isr10, 10, has_err);
exception_stub!(isr11, 11, has_err);
exception_stub!(isr12, 12, has_err);
exception_stub!(isr13, 13, has_err);
exception_stub!(isr14, 14, has_err);
exception_stub!(isr15, 15, no_err);
exception_stub!(isr16, 16, no_err);
exception_stub!(isr17, 17, has_err);
exception_stub!(isr18, 18, no_err);
exception_stub!(isr19, 19, no_err);
exception_stub!(isr20, 20, no_err);
exception_stub!(isr21, 21, no_err);
exception_stub!(isr22, 22, no_err);
exception_stub!(isr23, 23, no_err);
exception_stub!(isr24, 24, no_err);
exception_stub!(isr25, 25, no_err);
exception_stub!(isr26, 26, no_err);
exception_stub!(isr27, 27, no_err);
exception_stub!(isr28, 28, no_err);
exception_stub!(isr29, 29, no_err);
exception_stub!(isr30, 30, has_err);
exception_stub!(isr31, 31, no_err);
