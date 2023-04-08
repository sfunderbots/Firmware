use cortex_m::asm::delay;
use cortex_m_semihosting::hprintln;
use embedded_nrf24l01::{Configuration, Device};
use embedded_nrf24l01::{CrcMode, DataRate, NRF24L01};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{
    device::Peripherals,
    spi,
    usb::{Peripheral, UsbBus},
};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub fn run() {
    let device_peripherals = Peripherals::take().unwrap();

    let mut flash = device_peripherals.FLASH.constrain();
    let rcc = device_peripherals.RCC.constrain();

    // To meet CAN clock accuracy requirements an external crystal or ceramic
    // resonator must be used. The blue pill has a 8MHz external crystal.
    // Other boards might have a crystal with another frequency or none at all.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    hprintln!("Clocks: {:?}", clocks);
    assert!(clocks.usbclk_valid());

    let mut gpioa = device_peripherals.GPIOA.split();
    let mut gpiob = device_peripherals.GPIOB.split();
    let mut gpioc = device_peripherals.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    led.set_high();

    let ce = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let mut ncs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    ncs.set_high();

    let radio_spi = spi::Spi::spi2(
        device_peripherals.SPI2,
        (sck, miso, mosi),
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        },
        8.MHz(), // Recomended SPI clock frequency is 8MHz
        clocks,
    );

    let nrf24 = NRF24L01::new(ce, ncs, radio_spi).unwrap();
    let mut nrf = nrf24.tx().unwrap();
    nrf.flush_tx().unwrap();
    nrf.flush_rx().unwrap();
    nrf.set_frequency(0x4c).unwrap();
    nrf.set_auto_retransmit(0x0f, 0x0f).unwrap();
    nrf.set_auto_ack(&[false; 6]).unwrap();
    nrf.set_rf(&DataRate::R1Mbps, 3).unwrap();
    nrf.set_crc(CrcMode::TwoBytes).unwrap();
    nrf.set_tx_addr(&b"2Node"[..]).unwrap();

    hprintln!("------");
    hprintln!("ADDRWIDTH: {}", nrf.get_address_width().unwrap());
    hprintln!("FREQ: {}", nrf.get_frequency().unwrap());
    hprintln!("INTER: {}", nrf.get_interrupts().unwrap().0);
    hprintln!("------");

    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();

    delay(clocks.sysclk().raw() / 100);

    let usb_bus = UsbBus::new(Peripheral {
        usb: device_peripherals.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    });

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("The Bots")
        .product("Robot")
        .serial_number("v0.0.1")
        .device_class(USB_CLASS_CDC)
        .build();

    loop {
        if nrf.can_send().unwrap() {
            nrf.send(&[1; 32]).unwrap();
            hprintln!("Sent");
        }

        // wait for 1 second
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 1024];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                led.set_low(); // Turn on

                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        led.set_high(); // Turn off
    }
}
