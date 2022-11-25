use cortex_m::asm::delay;
use stm32f1xx_hal::{usb::{Peripheral, UsbBus}, device::Peripherals};
use stm32f1xx_hal::{can::Can, pac, prelude::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[cfg(feature = "dongle")]
pub struct Dongle 
{
    usb_bus: UsbBusAllocator<Peripheral>,
    usb_dev: UsbDevice<'static, UsbBus<Peripheral>>,
    serial: SerialPort<'static, UsbBus<Peripheral>>,
}

impl Dongle 
{
    pub fn new() -> Self
    {
        let device_peripherals = Peripherals::take().unwrap();
        let mut flash = device_peripherals.FLASH.constrain();
        let mut rcc = device_peripherals.RCC.constrain();

        let mut gpioa = device_peripherals.GPIOA.split();

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

        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        delay(clocks.sysclk().raw() / 100);

        let usb_bus = UsbBus::new(
            Peripheral {
                usb: device_peripherals.USB,
                pin_dm: gpioa.pa11,
                pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
            }
        );

        Self {
            usb_bus,
            usb_dev: UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("The Bots")
            .product("Robot")
            .serial_number("v0.0.1")
            .device_class(USB_CLASS_CDC)
            .build(),
            serial: SerialPort::new(&usb_bus),
        }
    }

    pub fn run(&mut self) {
        loop {

        }
    }
}
