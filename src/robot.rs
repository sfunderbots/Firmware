extern crate panic_semihosting;

use bxcan::*;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1xx_hal::device::{Peripherals, CAN1};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::timer::delay;
use stm32f1xx_hal::{can, spi, timer};
use ws2812_spi::{Ws2812, MODE};
use tinymovr::Tinymovr;

use smart_leds::{SmartLedsWrite, RGB8};
mod tinymovr;

pub fn run() {
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
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

    assert!(clocks.usbclk_valid());

    // GPIO parts
    let mut gpioa = device_peripherals.GPIOA.split();
    let mut gpiob = device_peripherals.GPIOB.split();
    let mut afio = device_peripherals.AFIO.constrain();

    // Setup SPI (consumed by NRF24L01)
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
        2.MHz(), // Recomended SPI clock frequency is 8MHz
        clocks,
    );

    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let led_spi = spi::Spi::spi1(
        device_peripherals.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        ws2812_spi::MODE,
        5.MHz(), // Recomended SPI clock frequency is 8MHz
        clocks,
    );

    let mut data: [RGB8; 12] = [RGB8::default(); 12];
    let mut ws = Ws2812::new(led_spi);
    data[0] = RGB8::new(255, 0, 0);
    data[1] = RGB8::new(0, 255, 0);
    data[2] = RGB8::new(0, 0, 255);

    // Setup CAN
    let can = can::Can::new(device_peripherals.CAN1, device_peripherals.USB);
    let rx = gpiob.pb8.into_floating_input(&mut gpiob.crh);
    let tx = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);
    can.assign_pins((tx, rx), &mut afio.mapr);
    hprintln!("BRUH");

    // APB1 (PCLK1): 8MHz, Bit rate: 125kBit/s, Sample Point 87.5%
    // Value was calculated with http://www.bittiming.can-wiki.info/
    let mut can = bxcan::Can::builder(can)
        .set_bit_timing(0x001c0003)
        .enable();

    hprintln!("BRUH2");

    let mut delay = device_peripherals.TIM2.delay_us(&clocks);
    hprintln!("BRUH3");
    //hprintln!("tiny {}", tiny.device_info.device_id);
    let mut tiny = Tinymovr::new(1, can);
    hprintln!("BRUH4");

    loop {
        hprintln!("trying to transmit");
        tiny.bruh();
    hprintln!("BRUH5");
        //ws.write(data.iter().cloned()).unwrap();
        //data.rotate_right(1);
        delay.delay_ms(1000_u16);
    }
}
