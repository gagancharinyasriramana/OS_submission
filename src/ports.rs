// Port I/O helpers for x86 architecture

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    // SAFETY: caller must ensure writing to this I/O port does not cause undefined hardware behavior
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    // SAFETY: caller must ensure reading from this I/O port is safe
    let ret: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") ret,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    // SAFETY: writing a 16-bit word to I/O port
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    // SAFETY: reading a 16-bit word from I/O port
    let ret: u16;
    core::arch::asm!(
        "in ax, dx",
        out("ax") ret,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

#[inline]
pub unsafe fn io_wait() {
    // SAFETY: port 0x80 is commonly used as a dummy I/O delay
    outb(0x80, 0);
}
