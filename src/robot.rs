extern crate panic_semihosting;

use bxcan::Fifo;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1xx_hal::device::adc2::jsqr::W;
use stm32f1xx_hal::device::Peripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity};
use stm32f1xx_hal::{can, pac, prelude::*, spi};

use embedded_nrf24l01::Configuration;
use embedded_nrf24l01::{CrcMode, DataRate, NRF24L01};

pub fn run() {
    let device_peripherals = Peripherals::take().unwrap();
    let mut cortex_peripherals = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut flash = device_peripherals.FLASH.constrain();
    let rcc = device_peripherals.RCC.constrain();

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

    let mut gpioa = device_peripherals.GPIOA.split();
    let mut gpiob = device_peripherals.GPIOB.split();
    let mut gpioc = device_peripherals.GPIOC.split();

    let mut afio = device_peripherals.AFIO.constrain();

    let mut ncs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    ncs.set_high();

    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let ce = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

    let spi = spi::Spi::spi1(
        device_peripherals.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        },
        10.MHz(),
        clocks,
    );

    // nRF24L01 library specific starts here.
    let mut nrf24 = NRF24L01::new(ce, ncs, spi).unwrap();

    nrf24.set_frequency(10).unwrap();
    nrf24.set_auto_retransmit(0, 0).unwrap();
    nrf24.set_rf(&DataRate::R2Mbps, 3).unwrap();
    nrf24
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    nrf24.set_auto_ack(&[false; 6]).unwrap();
    nrf24.set_crc(CrcMode::Disabled).unwrap();
    nrf24.set_tx_addr(&b"fnord"[..]).unwrap();

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

    loop {
        //if !nrf24.rx().unwrap().ready().is_err() {
        //let mut buf = [0u8; 32];
        //let len = nrf24.read(&mut buf).unwrap();
        //hprintln!("Received: {:?}", buf);
        //}
    }
}
