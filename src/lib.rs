#![no_std]
#![allow(nonstandard_style)]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

use board::{FB_RESERVE_BYTES, MAIN_RAM_SIZE, STACK_SIZE};

/// Offset within the heap region (starts at 0 == `_heap_start`).
static HEAP_PTR: AtomicUsize = AtomicUsize::new(0);

struct BumpAllocator;

// Provided by linker.ld — main RAM is FMC SDRAM after the framebuffer reserve.
unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    static _sdata: u8;
    static _ebss: u8;
}

#[inline]
fn heap_start_addr() -> usize {
    core::ptr::addr_of!(_heap_start) as usize
}

#[inline]
fn heap_end_addr() -> usize {
    core::ptr::addr_of!(_heap_end) as usize
}

#[inline]
fn heap_capacity() -> usize {
    heap_end_addr().saturating_sub(heap_start_addr())
}

/// Snapshot of main RAM (SDRAM) usage for UI / diagnostics.
#[derive(Clone, Copy, Debug)]
pub struct HeapStats {
    /// Framebuffers + statics (.data/.bss) + bump heap consumed.
    pub used: usize,
    /// Bytes still available for bump allocation (below stack).
    pub free: usize,
    /// Full main board RAM window (SDRAM 64 MiB on carrier).
    pub total: usize,
    /// Bump-heap pool size only (between `_heap_start` and stack).
    pub heap_total: usize,
    /// Bump-heap bytes reserved.
    pub heap_used: usize,
}

/// SDRAM-centric usage: total = 64 MiB device; used includes FB + statics + heap.
pub fn heap_stats() -> HeapStats {
    let heap_used = HEAP_PTR.load(Ordering::SeqCst);
    let heap_total = heap_capacity();
    let statics = {
        let s = core::ptr::addr_of!(_sdata) as usize;
        let e = core::ptr::addr_of!(_ebss) as usize;
        e.saturating_sub(s)
    };
    // FB at base of SDRAM + static region + bump used. Stack reservation is
    // treated as used (not allocatable).
    let used = FB_RESERVE_BYTES
        .saturating_add(statics)
        .saturating_add(heap_used)
        .saturating_add(STACK_SIZE);
    // Carrier: FB_RESERVE + MAIN_RAM_SIZE == SDRAM_SIZE (64 MiB).
    let total = FB_RESERVE_BYTES.saturating_add(MAIN_RAM_SIZE);
    HeapStats {
        used: used.min(total),
        free: total.saturating_sub(used.min(total)),
        total,
        heap_total,
        heap_used,
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(1);
        let size = layout.size();
        let cap = heap_capacity();
        let ptr = HEAP_PTR.load(Ordering::SeqCst);
        let aligned = (ptr + align - 1) & !(align - 1);
        let end = aligned.saturating_add(size);
        if end > cap {
            return core::ptr::null_mut();
        }
        HEAP_PTR.store(end, Ordering::SeqCst);
        (heap_start_addr() + aligned) as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

pub mod board;
pub mod drivers;
pub mod storage;
pub mod hal;
pub mod klog;
pub mod proc;
pub mod apps;
pub mod kui;
