//! A heap allocator for Cortex-M processors.
//!
//! Note that using this as your global allocator requires nightly Rust.
//!
//! # Example
//!
//! For a usage example, see `examples/global_alloc.rs`.
//!
//! # Features
//!
//! The First Level Index (FLI) must be specified. The FLI determines the tlsf overhead required,
//! the largest object that can be stored on the heap, and the largest block that can be used to
//! _extend_ the memory pool.
//!
//! The FLI can be specified by setting one of the FLIx features in the range of FLI6..=FLI15.
//!
//! |FLI|`size_of(Tlsf)`|`MAX_REQUEST_SIZE`|`MAX_BLOCK_SIZE`|
//! |---|---------------|------------------|----------------|
//! |6  |36             |60                |60              |
//! |7  |70             |120               |124             |
//! |8  |104            |240               |252             |
//! |9  |138            |480               |508             |
//! |10 |172            |960               |1,020           |
//! |11 |206            |1,920             |2,044           |
//! |12 |240            |3,840             |4,092           |
//! |13 |274            |7,680             |8,188           |
//! |14 |308            |15,360            |16,380          |
//! |15 |342            |30,720            |32,764          |
//!
//! *All sizes are in bytes*

#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::NonNull;

use cortex_m::interrupt::Mutex;
use tlsf::Tlsf as Heap;

pub struct CortexMHeap {
    heap: Mutex<RefCell<Heap>>,
}

impl CortexMHeap {
    /// Create a new heap allocator, with an empty memory pool
    ///
    /// The allocator's memory pool must be extended using the
    /// [`extend`](struct.CortexMHeap.html#method.extend) method before using the allocator.
    pub const fn empty() -> CortexMHeap {
        CortexMHeap {
            heap: Mutex::new(RefCell::new(Heap::new())),
        }
    }

    /// Adds a memory _chunk_ to the allocator's memory pool
    ///
    /// This function must be called at least once BEFORE any code makes use of the
    /// allocator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alloc_cortex_m::CortexMHeap;
    ///
    /// static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
    ///
    /// static mut CHUNK: [u8; tlsf::MAX_BLOCK_SIZE as usize] =
    ///         [0; tlsf::MAX_BLOCK_SIZE as usize];
    ///
    /// unsafe { ALLOCATOR.extend(&mut CHUNK) }
    /// ```
    pub fn extend(&self, block: &'static mut [u8]) {
        cortex_m::interrupt::free(move |cs| {
            self.heap.borrow(cs).borrow_mut().extend(block);
        });
    }

    /// Returns an estimate of the amount of bytes in use.
    pub fn used(&self) -> usize {
        cortex_m::interrupt::free(|cs| {
            self.heap
                .borrow(cs)
                .borrow_mut()
                .used()
        })
    }

    /// Returns an estimate of the amount of bytes available.
    pub fn free(&self) -> usize {
        cortex_m::interrupt::free(|cs| {
            self.heap
                .borrow(cs)
                .borrow_mut()
                .free()
        })
    }
}

unsafe impl GlobalAlloc for CortexMHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        cortex_m::interrupt::free(|cs| {
            let returned_mem = self.heap.borrow(cs).borrow_mut().alloc(layout);
            match returned_mem {
                Ok(mem_ptr) => mem_ptr.as_ptr(),
                Err(_) => 0_usize as *mut u8,
            }
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if let Some(mem_ptr) = NonNull::new(ptr) {
            cortex_m::interrupt::free(|cs| {
                self.heap.borrow(cs).borrow_mut().dealloc(mem_ptr);
            });
        }
    }
}
