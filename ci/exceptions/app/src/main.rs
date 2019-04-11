#![feature(core_intrinsics)]
#![no_main]
#![no_std]

use core::intrinsics;

use rt::entry;

entry!(main);

fn main() -> ! {
    // これは未定義命令（UDF）を実行し、HardFault例外を引き起こします
    unsafe { intrinsics::abort() }
}
