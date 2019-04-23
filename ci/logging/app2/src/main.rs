#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hio};

use rt::entry;

entry!(main);

fn main() -> ! {
    let mut hstdout = hio::hstdout().unwrap();

    #[export_name = "Hello, world!"]
    #[link_section = ".log"] // <- 追加!
    static A: u8 = 0;

    let address = &A as *const u8 as usize as u8;
    hstdout.write_all(&[address]).unwrap(); // <- 変更!

    #[export_name = "Goodbye"]
    #[link_section = ".log"] // <- 追加!
    static B: u8 = 0;

    let address = &B as *const u8 as usize as u8;
    hstdout.write_all(&[address]).unwrap(); // <- 変更!

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
