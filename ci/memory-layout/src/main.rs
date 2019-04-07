#![no_main]
#![no_std]

use core::panic::PanicInfo;

// リセットハンドラ
#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    let _x = 42;

    // 戻れないため、ここで無限ループに入ります
    loop {}
}

// リセットベクタは、リセットハンドラへのポインタです
#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
