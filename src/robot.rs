extern crate panic_semihosting;

use bxcan::Fifo;
use nb::block;
use stm32f1xx_hal::device::Peripherals;
use stm32f1xx_hal::{can::Can, pac, prelude::*};

#[cfg(feature = "robot")]
pub struct Robot 
{
    clocks: stm32f1xx_hal::rcc::Clocks,
    can: bxcan::Can<stm32f1xx_hal::can::Can<pac::CAN1>>,
}

#[cfg(feature = "robot")]
impl Robot
{
    pub fn new() -> Self {

        let device_peripherals = Peripherals::take().unwrap();

        let mut flash = device_peripherals.FLASH.constrain();
        let mut rcc = device_peripherals.RCC.constrain();

        // To meet CAN clock accuracy requirements an external crystal or ceramic
        // resonator must be used. The blue pill has a 8MHz external crystal.
        // Other boards might have a crystal with another frequency or none at all.
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        Self {
            clocks,
            can: Robot::configure_can(device_peripherals.CAN1, device_peripherals.USB, &device_peripherals),
        }
    }

    fn configure_can(can: pac::CAN1, usb: pac::USB, device_peripherals: &Peripherals) -> bxcan::Can<stm32f1xx_hal::can::Can<pac::CAN1>> {

        let can = Can::new(can, usb);
        let mut afio = device_peripherals.AFIO.constrain();
        let mut gpiob = device_peripherals.GPIOB.split();

        let rx = gpiob.pb8.into_floating_input(&mut gpiob.crh);
        let tx = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

        can.assign_pins((tx, rx), &mut afio.mapr);
        
        // APB1 (PCLK1): 8MHz, Bit rate: 125kBit/s, Sample Point 87.5%
        // Value was calculated with http://www.bittiming.can-wiki.info/
        let mut can = bxcan::Can::builder(can)
                .set_bit_timing(0x001c_0003)
                .leave_disabled();

        // Configure filters so that can frames can be received.
        let mut filters = can.modify_filters();
        filters.enable_bank(0, Fifo::Fifo0, bxcan::filter::Mask32::accept_all());

        // Drop filters to leave filter configuraiton mode.
        drop(filters);

        // Split the peripheral into transmitter and receiver parts.
        block!(can.enable_non_blocking()).unwrap();

        can
    }

    pub fn run(&mut self) {
        loop {
            // Receive a frame
            let frame = block!(self.can.receive()).unwrap().unwrap();

        }
    }
}
