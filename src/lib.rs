#![no_std]
#![allow(nonstandard_style)]

extern crate alloc;

pub mod heap;

pub use heap::{heap_stats, FreeListAllocator, HeapStats};

#[global_allocator]
static ALLOCATOR: FreeListAllocator = FreeListAllocator::new();

pub mod board;
pub mod drivers;
pub mod storage;
pub mod hal;
pub mod klog;
pub mod proc;
pub mod apps;
pub mod kui;
