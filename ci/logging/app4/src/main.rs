#![no_main]
#![no_std]

use cortex_m_semihosting::{
    debug,
    hio::{self, HStdout},
};

use log::{error, warn, Log};
use rt::entry;

entry!(main);

fn main() -> ! {
    let hstdout = hio::hstdout().unwrap();
    let mut logger = Logger { hstdout };

    warn!(logger, "Hello, world!"); // <- 変更!

    error!(logger, "Goodbye"); // <- 変更!

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}

struct Logger {
    hstdout: HStdout,
}

impl Log for Logger {
    type Error = ();

    fn log(&mut self, address: u8) -> Result<(), ()> {
        self.hstdout.write_all(&[address])
    }
}
