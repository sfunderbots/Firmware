extern crate panic_semihosting;

use bxcan::Fifo;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1xx_hal::device::Peripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::timer::delay;
use stm32f1xx_hal::{can, spi, timer};
use ws2812_spi::{Ws2812, MODE};

use smart_leds::{SmartLedsWrite, RGB8};

use embedded_nrf24l01::{Configuration, Device};
use embedded_nrf24l01::{CrcMode, DataRate, NRF24L01};

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

    // Setup Radio
    let nrf24 = NRF24L01::new(ce, ncs, radio_spi).unwrap();
    let mut nrf = nrf24.tx().unwrap();

    // Configure Radio
    nrf.flush_tx().unwrap();
    nrf.flush_rx().unwrap();

    // TODO make this configurable
    nrf.set_frequency(0x4c).unwrap();
    nrf.set_auto_retransmit(0x0f, 0x0f).unwrap();
    nrf.set_auto_ack(&[false; 6]).unwrap();
    nrf.set_rf(&DataRate::R1Mbps, 3).unwrap();
    nrf.set_crc(CrcMode::TwoBytes).unwrap();
    nrf.set_tx_addr(&b"2Node"[..]).unwrap();
    nrf.set_rx_addr(1, &b"3Node"[..]).unwrap();

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

    let mut delay = device_peripherals.TIM2.delay_us(&clocks);

    loop {
        if nrf.can_send().unwrap() {
            nrf.send(&[1; 32]).unwrap();
        }
        ws.write(data.iter().cloned()).unwrap();
        data.rotate_right(1);
        delay.delay_ms(30_u16);
    }
}
