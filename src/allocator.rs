// Heap allocator: Linked-list allocator with block splitting and coalescing
// Backed by a fixed 4 MiB static heap region
use core::alloc::{GlobalAlloc, Layout};

pub const HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

#[repr(align(16))]
struct HeapMemory([u8; HEAP_SIZE]);
static mut HEAP_SPACE: HeapMemory = HeapMemory([0; HEAP_SIZE]);

#[repr(C)]
struct BlockHeader {
    size: usize,        // usable size in bytes (excluding header)
    is_free: bool,
    next: *mut BlockHeader,
}

pub struct LinkedListAllocator {
    head: *mut BlockHeader,
}

// SAFETY: Single-threaded OS kernel heap protected by critical section (cli/sti)
unsafe impl Sync for LinkedListAllocator {}
unsafe impl Send for LinkedListAllocator {}

impl LinkedListAllocator {
    pub const fn empty() -> Self {
        LinkedListAllocator {
            head: core::ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            let start = HEAP_SPACE.0.as_mut_ptr() as *mut BlockHeader;
            let header_size = core::mem::size_of::<BlockHeader>();
            let usable_size = HEAP_SIZE - header_size;

            core::ptr::write(
                start,
                BlockHeader {
                    size: usable_size,
                    is_free: true,
                    next: core::ptr::null_mut(),
                },
            );

            self.head = start;
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(core::mem::align_of::<BlockHeader>());
        let size = (layout.size() + align - 1) & !(align - 1);
        let header_size = core::mem::size_of::<BlockHeader>();

        let mut current = self.head;

        while !current.is_null() {
            let block = &mut *current;
            if block.is_free && block.size >= size {
                // Check if we can split the block
                if block.size >= size + header_size + 16 {
                    let next_addr = (current as usize) + header_size + size;
                    let next_block = next_addr as *mut BlockHeader;
                    let remaining_size = block.size - size - header_size;

                    core::ptr::write(
                        next_block,
                        BlockHeader {
                            size: remaining_size,
                            is_free: true,
                            next: block.next,
                        },
                    );

                    block.size = size;
                    block.is_free = false;
                    block.next = next_block;
                } else {
                    block.is_free = false;
                }

                let data_ptr = (current as usize + header_size) as *mut u8;
                return data_ptr;
            }
            current = block.next;
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let header_size = core::mem::size_of::<BlockHeader>();
        let block_ptr = (ptr as usize - header_size) as *mut BlockHeader;
        let block = &mut *block_ptr;
        block.is_free = true;

        // Coalesce adjacent free blocks
        let mut curr = self.head;
        while !curr.is_null() {
            let curr_block = &mut *curr;
            let next = curr_block.next;
            if !next.is_null() {
                let next_block = &mut *next;
                if curr_block.is_free && next_block.is_free {
                    let expected_next_addr = (curr as usize) + header_size + curr_block.size;
                    if expected_next_addr == (next as usize) {
                        curr_block.size += header_size + next_block.size;
                        curr_block.next = next_block.next;
                        continue; // try coalescing again from curr
                    }
                }
            }
            curr = curr_block.next;
        }
    }
}

pub struct LockedAllocator {
    inner: LinkedListAllocator,
}

impl LockedAllocator {
    pub const fn new() -> Self {
        LockedAllocator {
            inner: LinkedListAllocator::empty(),
        }
    }

    pub fn init(&mut self) {
        self.inner.init();
    }
}

#[global_allocator]
pub static mut ALLOCATOR: LockedAllocator = LockedAllocator::new();

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let flags: u32;
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            "cli",
            out(reg) flags,
            options(nomem, nostack)
        );

        let ptr = ALLOCATOR.inner.alloc(layout);

        if (flags & 0x200) != 0 {
            core::arch::asm!("sti");
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let flags: u32;
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            "cli",
            out(reg) flags,
            options(nomem, nostack)
        );

        ALLOCATOR.inner.dealloc(ptr, layout);

        if (flags & 0x200) != 0 {
            core::arch::asm!("sti");
        }
    }
}
