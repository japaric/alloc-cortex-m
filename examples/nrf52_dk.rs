#![no_main]
#![no_std]
#![feature(alloc_prelude)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate nrf52832_hal;
extern crate panic_rtt;

use alloc::vec::Vec;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::fmt::Write;
use cortex_m_rt::entry;
use jlink_rtt::NonBlockingOutput;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[repr(align(4))]
struct Aligned<T>(T);

#[entry]
fn main() -> ! {
    static mut M: Aligned<[u8; tlsf::MAX_BLOCK_SIZE as usize]> =
        Aligned([0; tlsf::MAX_BLOCK_SIZE as usize]);

    let mut log = NonBlockingOutput::new();
    writeln!(log, "Output stream opened").ok().unwrap();

    ALLOCATOR.extend(&mut M.0);

    writeln!(log, "Heap extended").ok().unwrap();

    let mut xs = Vec::new();
    xs.push(1);

    writeln!(log, "Vector instantiated").ok().unwrap();

    loop {
        xs.push(1);
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("alloc error");
}
