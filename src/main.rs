// The Bots Mainboard Firmware
//
// Build it in release mode if it doens't fit on the stm32f103c8t6
// We gonna use it no matter what.
#![no_std]
#![no_main]
// set the panic handler
extern crate panic_semihosting;

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt;

use cortex_m_rt::entry;

mod dongle;
mod robot;

#[entry]
fn main() -> ! {
    loop {
        #[cfg(feature = "robot")]
        robot::run();

        #[cfg(feature = "dongle")]
        dongle::run();
    }
}
