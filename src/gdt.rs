// Global Descriptor Table (GDT) for 32-bit x86 flat memory model
use core::ptr::addr_of;

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (flags & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,
    base: u32,
}

static mut GDT: [GdtEntry; 3] = [
    GdtEntry::new(0, 0, 0, 0),                // Null descriptor
    GdtEntry::new(0, 0xFFFFF, 0x9A, 0xCF),    // 0x08: Kernel Code (exec/read, 4KB granularity, 32-bit)
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xCF),    // 0x10: Kernel Data (read/write, 4KB granularity, 32-bit)
];

static mut GDT_PTR: GdtPtr = GdtPtr { limit: 0, base: 0 };

pub fn init() {
    unsafe {
        // SAFETY: Initialize GDT pointer and load table
        GDT_PTR.limit = (core::mem::size_of::<[GdtEntry; 3]>() - 1) as u16;
        GDT_PTR.base = GDT.as_ptr() as u32;

        core::arch::asm!(
            "lgdt [{0}]",
            in(reg) addr_of!(GDT_PTR),
            options(readonly, nostack, preserves_flags)
        );

        // Reload data segments with 0x10 and perform far return to reload CS with 0x08
        core::arch::asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "push 0x08",
            "lea eax, [2f]",
            "push eax",
            "retf",
            "2:",
            out("eax") _,
            options(nomem, preserves_flags)
        );
    }
}
