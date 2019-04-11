#![feature(core_intrinsics)]
#![no_main]
#![no_std]

use core::intrinsics;

use rt::entry;

entry!(main);

fn main() -> ! {
    unsafe { intrinsics::abort() }
}

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    // ここで何か面白いことをやります
    loop {}
}
