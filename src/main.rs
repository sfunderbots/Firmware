// The Bots Mainboard Firmware
//
// Build it in release mode if it doens't fit on the stm32f103c8t6
// We gonna use it no matter what.
#![no_std]
#![no_main]
mod robot;
mod dongle;

use robot::Robot;
use dongle::Dongle;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {

    #[cfg(feature = "robot")]
    let robot = Robot::new();

    #[cfg(feature = "dongle")]
    let dongle = Dongle::new();

    loop {
        #[cfg(feature = "robot")]
        robot.run();

        #[cfg(feature = "dongle")]
        dongle.run();
    }
}
