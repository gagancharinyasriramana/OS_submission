// 8259A Programmable Interrupt Controller (PIC) driver
use crate::ports::{inb, io_wait, outb};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

pub fn remap(offset1: u8, offset2: u8) {
    unsafe {
        // SAFETY: Standard 8259A PIC initialization sequence
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Starts the initialization sequence (in cascade mode)
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // ICW2: Vector offset
        outb(PIC1_DATA, offset1);
        io_wait();
        outb(PIC2_DATA, offset2);
        io_wait();

        // ICW3: Tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
        outb(PIC1_DATA, 0x04);
        io_wait();
        // ICW3: Tell Slave PIC its cascade identity (0000 0010)
        outb(PIC2_DATA, 0x02);
        io_wait();

        // ICW4: 8086/88 (MCS-80/85) mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore masks (or set defaults)
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }
}

pub fn send_eoi(irq: u8) {
    unsafe {
        // SAFETY: Send End Of Interrupt signal to PIC(s)
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

pub fn set_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let value = inb(port) | (1 << irq_line);
        outb(port, value);
    }
}

pub fn clear_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let value = inb(port) & !(1 << irq_line);
        outb(port, value);
    }
}
