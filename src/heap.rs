//! Intrusive free-list allocator over the linker heap region.
//!
//! Free blocks form an address-ordered singly linked list embedded in free memory.
//! Allocation is first-fit with splitting; deallocation inserts and coalesces
//! adjacent free neighbors.

use core::alloc::{GlobalAlloc, Layout};
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Mutex;

/// In-place free node (lives at the start of every free region).
#[repr(C)]
struct FreeNode {
    size: usize,
    next: Option<NonNull<FreeNode>>,
}

/// Header placed at the start of every live allocation block.
/// User payload begins at `block_start + header_size(align)`.
#[repr(C)]
struct AllocHeader {
    /// Total block size including this header (and any absorbed tail padding).
    size: usize,
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Smallest free region we keep: must hold a `FreeNode`.
const MIN_FREE_SIZE: usize = {
    let a = align_of::<FreeNode>();
    let s = size_of::<FreeNode>();
    (s + a - 1) & !(a - 1)
};

/// Bytes currently reserved by live allocations (including headers).
static HEAP_USED: AtomicUsize = AtomicUsize::new(0);

struct FreeList {
    head: Option<NonNull<FreeNode>>,
    heap_start: usize,
    heap_end: usize,
    initialized: bool,
}

// SAFETY: only accessed through `Mutex`; nodes live in the exclusive heap region.
unsafe impl Send for FreeList {}

impl FreeList {
    const fn new() -> Self {
        Self {
            head: None,
            heap_start: 0,
            heap_end: 0,
            initialized: false,
        }
    }

    unsafe fn ensure_init(&mut self) {
        if self.initialized {
            return;
        }
        let start = heap_start_addr();
        let end = heap_end_addr();
        self.heap_start = start;
        self.heap_end = end;
        self.initialized = true;

        let size = end.saturating_sub(start);
        if size >= MIN_FREE_SIZE && start.is_multiple_of(align_of::<FreeNode>()) {
            let node = start as *mut FreeNode;
            // SAFETY: linker guarantees [_heap_start, _heap_end) is exclusive RAM.
            unsafe {
                ptr::write(
                    node,
                    FreeNode {
                        size,
                        next: None,
                    },
                );
            }
            self.head = NonNull::new(node);
        }
    }

    /// Returns `(total_block_size, header_size, align)` for a user `layout`.
    ///
    /// The allocated block is `align`-aligned and starts with `AllocHeader`;
    /// user data begins at `block + header_size` and is therefore also aligned.
    fn block_layout(layout: Layout) -> Option<(usize, usize, usize)> {
        let align = layout.align().max(align_of::<AllocHeader>());
        let header_size = align_up(size_of::<AllocHeader>(), align);
        let payload = layout.size().max(1);
        let total = header_size.checked_add(payload)?;
        // Round up so the block can re-enter the free list later.
        let total = align_up(total, align_of::<FreeNode>()).max(MIN_FREE_SIZE);
        Some((total, header_size, align))
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe { self.ensure_init() };
        let Some((need, header_size, align)) = Self::block_layout(layout) else {
            return ptr::null_mut();
        };

        let mut prev: Option<NonNull<FreeNode>> = None;
        let mut cur = self.head;

        while let Some(node) = cur {
            let node_ptr = node.as_ptr();
            let FreeNode {
                size: block_size,
                next,
            } = unsafe { ptr::read(node_ptr) };

            let block_start = node_ptr as usize;
            let block_end = block_start + block_size;

            // Whole block is `align`-aligned so payload at start+header_size is aligned.
            let start = align_up(block_start, align);
            if start.saturating_add(need) > block_end {
                prev = Some(node);
                cur = next;
                continue;
            }

            let front = start - block_start;
            // Leading gap too small to track as its own free node — skip this region.
            if front > 0 && front < MIN_FREE_SIZE {
                prev = Some(node);
                cur = next;
                continue;
            }

            let end = start + need;
            let tail = block_end - end;

            self.unlink(prev, next);

            if front >= MIN_FREE_SIZE {
                unsafe { self.insert_free(block_start, front) };
            }

            let actual_size = if tail >= MIN_FREE_SIZE {
                unsafe { self.insert_free(end, tail) };
                need
            } else {
                // Absorb a tiny unusable tail into this allocation.
                need + tail
            };

            let header_ptr = start as *mut AllocHeader;
            unsafe {
                ptr::write(
                    header_ptr,
                    AllocHeader {
                        size: actual_size,
                    },
                );
            }

            HEAP_USED.fetch_add(actual_size, Ordering::SeqCst);
            let user = (start + header_size) as *mut u8;
            debug_assert_eq!(user as usize % align, 0);
            return user;
        }

        ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }
        unsafe { self.ensure_init() };

        let Some((_, header_size, align)) = Self::block_layout(layout) else {
            return;
        };

        let user = ptr as usize;
        debug_assert_eq!(user % align, 0);
        let start = user - header_size;
        debug_assert_eq!(start % align.max(align_of::<AllocHeader>()), 0);

        let header_ptr = start as *mut AllocHeader;
        let size = unsafe { (*header_ptr).size };
        debug_assert!(size >= MIN_FREE_SIZE);
        debug_assert!(start >= self.heap_start && start + size <= self.heap_end);

        HEAP_USED.fetch_sub(size, Ordering::SeqCst);
        unsafe { self.insert_free(start, size) };
    }

    fn unlink(&mut self, prev: Option<NonNull<FreeNode>>, next: Option<NonNull<FreeNode>>) {
        match prev {
            Some(p) => unsafe {
                (*p.as_ptr()).next = next;
            },
            None => self.head = next,
        }
    }

    /// Insert `[addr, addr+size)` into the address-ordered free list and coalesce.
    unsafe fn insert_free(&mut self, mut addr: usize, mut size: usize) {
        debug_assert!(size >= MIN_FREE_SIZE);
        debug_assert_eq!(addr % align_of::<FreeNode>(), 0);

        let mut prev: Option<NonNull<FreeNode>> = None;
        let mut cur = self.head;

        // Find insertion point: first node with address > addr.
        while let Some(node) = cur {
            if (node.as_ptr() as usize) > addr {
                break;
            }
            prev = Some(node);
            cur = unsafe { (*node.as_ptr()).next };
        }

        // Coalesce with previous if adjacent.
        if let Some(p) = prev {
            let p_ptr = p.as_ptr();
            let p_addr = p_ptr as usize;
            let p_size = unsafe { (*p_ptr).size };
            if p_addr + p_size == addr {
                addr = p_addr;
                size += p_size;
                // Unlink `p`; new region covers it. Predecessor of `p` becomes `prev`.
                let p_next = unsafe { (*p_ptr).next };
                let pred = Self::predecessor(self.head, p);
                self.unlink(pred, p_next);
                prev = pred;
                cur = p_next;
            }
        }

        // Coalesce with next if adjacent.
        if let Some(n) = cur {
            let n_addr = n.as_ptr() as usize;
            if addr + size == n_addr {
                let n_size = unsafe { (*n.as_ptr()).size };
                let n_next = unsafe { (*n.as_ptr()).next };
                size += n_size;
                cur = n_next;
            }
        }

        let node = addr as *mut FreeNode;
        unsafe {
            ptr::write(
                node,
                FreeNode {
                    size,
                    next: cur,
                },
            );
        }
        let nn = unsafe { NonNull::new_unchecked(node) };
        match prev {
            Some(p) => unsafe {
                (*p.as_ptr()).next = Some(nn);
            },
            None => self.head = Some(nn),
        }
    }

    fn predecessor(
        head: Option<NonNull<FreeNode>>,
        target: NonNull<FreeNode>,
    ) -> Option<NonNull<FreeNode>> {
        let mut prev = None;
        let mut cur = head;
        while let Some(n) = cur {
            if n == target {
                return prev;
            }
            prev = Some(n);
            cur = unsafe { (*n.as_ptr()).next };
        }
        None
    }
}

/// Global free-list allocator.
pub struct FreeListAllocator {
    inner: Mutex<FreeList>,
}

impl FreeListAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(FreeList::new()),
        }
    }
}

impl Default for FreeListAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut list = self.inner.lock();
        // SAFETY: caller upholds GlobalAlloc contract; we hold the heap lock.
        unsafe { list.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut list = self.inner.lock();
        // SAFETY: `ptr` was allocated with this allocator and the same layout.
        unsafe { list.dealloc(ptr, layout) }
    }
}

// Provided by linker.ld — main RAM is FMC SDRAM after the framebuffer reserve.
unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    static _sdata: u8;
    static _ebss: u8;
}

#[inline]
pub fn heap_start_addr() -> usize {
    core::ptr::addr_of!(_heap_start) as usize
}

#[inline]
pub fn heap_end_addr() -> usize {
    core::ptr::addr_of!(_heap_end) as usize
}

#[inline]
pub fn heap_capacity() -> usize {
    heap_end_addr().saturating_sub(heap_start_addr())
}

/// Snapshot of main RAM (SDRAM) usage for UI / diagnostics.
#[derive(Clone, Copy, Debug)]
pub struct HeapStats {
    /// Framebuffers + statics (.data/.bss) + heap consumed + stack reserve.
    pub used: usize,
    /// Bytes still available in the main RAM window.
    pub free: usize,
    /// Full main board RAM window (SDRAM 64 MiB on carrier).
    pub total: usize,
    /// Free-list heap pool size only (between `_heap_start` and stack).
    pub heap_total: usize,
    /// Heap bytes reserved by live allocations (including block headers).
    pub heap_used: usize,
}

/// SDRAM-centric usage: total = 64 MiB device; used includes FB + statics + heap.
pub fn heap_stats() -> HeapStats {
    use crate::board::{FB_RESERVE_BYTES, MAIN_RAM_SIZE, STACK_SIZE};

    let heap_used = HEAP_USED.load(Ordering::SeqCst);
    let heap_total = heap_capacity();
    let statics = {
        let s = core::ptr::addr_of!(_sdata) as usize;
        let e = core::ptr::addr_of!(_ebss) as usize;
        e.saturating_sub(s)
    };
    // FB at base of SDRAM + static region + heap used. Stack reservation is
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
