#![no_std]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

const HEAP_SIZE: usize = 32 * 1024;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static HEAP_PTR: AtomicUsize = AtomicUsize::new(0);

struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let ptr = HEAP_PTR.load(Ordering::SeqCst);
        let aligned = (ptr + align - 1) & !(align - 1);
        let end = aligned + size;
        if end > HEAP_SIZE {
            return core::ptr::null_mut();
        }
        HEAP_PTR.store(end, Ordering::SeqCst);
        unsafe { core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(aligned) }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

pub mod board;
pub mod drivers;
pub mod hal;
pub mod klog;
pub mod proc;