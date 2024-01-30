mod tinymovr;
mod wheel_conversion;
use anyhow::Context;
use anyhow::Context;
use embedded_graphics::prelude::{RgbColor, WebColors};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::PinState;
use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::env;
use tinymovr::Tinymovr;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use display_interface_spi::SPIInterface;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::*;
use embedded_graphics::prelude::{RgbColor, WebColors};
use embedded_graphics::text::*;
use embedded_graphics::*;
use embedded_graphics::{image::Image, prelude::*};
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    prelude::*,
    text::{Alignment, Text},
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use linux_embedded_hal::{Delay, Pin, Spidev};
use rppal::gpio::Gpio;
use tinybmp::Bmp;

struct PwmDud;
use embedded_graphics_framebuf::FrameBuf;

use linux_embedded_hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::env;
use tinymovr::Tinymovr;
use wheel_conversion::{LocalVelocity, WheelVelocity};

struct Dud;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pub speed: f32,
    #[arg(short, long)]
    pub calibrate: bool,
}

impl PwmPin for Dud {
    type Duty = u16;

    fn disable(&mut self) {}
    fn enable(&mut self) {}
    fn get_duty(&self) -> Self::Duty {
        0
    }
    fn get_max_duty(&self) -> Self::Duty {
        0
    }
    fn set_duty(&mut self, _duty: Self::Duty) {}
}

struct GpioDud;

impl OutputPin for GpioDud {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let gpio = Gpio::new()?;
    let args = Args::parse();
    let options = SpidevOptions::new()
        .max_speed_hz(62_500_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    let mut spi_device_left = Spidev::open("/dev/spidev3.0")?;
    let mut spi_device_right = Spidev::open("/dev/spidev4.0")?;
    spi_device_left.configure(&options)?;
    spi_device_right.configure(&options)?;
    let mut cs_left = GpioDud;
    let mut cs_right = GpioDud;

    let mut rst_left = gpio.get(17)?.into_output();
    let mut rst_right = GpioDud;

    let mut dc_left = gpio.get(27)?.into_output();
    let mut dc_right = gpio.get(16)?.into_output();
    dc_left.set_high();
    dc_right.set_high();

    let spi_interface_left = SPIInterface::new(spi_device_left, dc_left, cs_left);
    let spi_interface_right = SPIInterface::new(spi_device_right, dc_right, cs_right);
    let pwm_dud_left = PwmDud;
    let pwm_dud_right = PwmDud;
    let mut delay = Delay;
    let mut display_left = gc9a01a::GC9A01A::new(spi_interface_left, rst_left, pwm_dud_left);
    let mut display_right = gc9a01a::GC9A01A::new(spi_interface_right, rst_right, pwm_dud_right);

    display_left.reset(&mut delay).unwrap();
    display_right.reset(&mut delay).unwrap();
    // Initialize registers
    display_left.initialize(&mut delay).unwrap();
    display_right.initialize(&mut delay).unwrap();

    // Clear the display
    display_left.clear(Rgb565::WHITE).unwrap();
    display_right.clear(Rgb565::WHITE).unwrap();

    // Create a new character style
    let blue_style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLUE);
    let red_style = MonoTextStyle::new(&FONT_10X20, Rgb565::RED);

    let mut data_left = [Rgb565::WHITE; 240 * 240];
    let mut data_right = [Rgb565::WHITE; 240 * 240];

    // load current time
    let now = std::time::Instant::now();
    if args.run_display {
        loop {
            let mut fbuf_left = FrameBuf::new(&mut data_left, 240, 240);
            let mut fbuf_right = FrameBuf::new(&mut data_right, 240, 240);

            fbuf_left.reset();
            fbuf_right.reset();

            Text::with_alignment(
                &format!("Time: {:?}", now.elapsed().as_millis()),
                Point::new(100, 100),
                blue_style,
                Alignment::Center,
            )
            .draw(&mut fbuf_left)
            .unwrap();

            Text::with_alignment(
                &format!("Time: {:?}", now.elapsed().as_millis()),
                Point::new(100, 100),
                red_style,
                Alignment::Center,
            )
            .draw(&mut fbuf_right)
            .unwrap();

            let left_area = Rectangle::new(Point::new(0, 0), fbuf_left.size());
            let right_area = Rectangle::new(Point::new(0, 0), fbuf_right.size());

            display_left.fill_contiguous(&left_area, data_left).unwrap();
            display_right
                .fill_contiguous(&right_area, data_right)
                .unwrap();
        }
    }
    let mut can = CanSocket::open("can0")
        .with_context(|| format!("Failed to open socket on interface can0"))?;
    let mut tiny1 = Tinymovr::new(1, &mut can);
    let mut tiny2 = Tinymovr::new(2, &mut can);
    let mut tiny3 = Tinymovr::new(3, &mut can);

    if args.calibrate {
        tiny1.calibrate(&mut can);
        tiny2.calibrate(&mut can);
        tiny3.calibrate(&mut can);
        return Ok(());
    }

    tiny1.velocity_control(&mut can);
    tiny2.velocity_control(&mut can);
    tiny3.velocity_control(&mut can);

    println!("Device info: {:?}", tiny1.device_info());
    println!("Device info: {:?}", tiny2.device_info());
    println!("Device info: {:?}", tiny3.device_info());

    tiny1.set_vel_integrator_params(0.02, 300.0, &mut can);
    tiny2.set_vel_integrator_params(0.02, 300.0, &mut can);
    tiny3.set_vel_integrator_params(0.02, 300.0, &mut can);

    tiny1.set_vel_setpoint(args.speed, &mut can);
    tiny2.set_vel_setpoint(args.speed, &mut can);
    tiny3.set_vel_setpoint(args.speed, &mut can);

    std::thread::sleep(std::time::Duration::from_secs(5));

    tiny1.set_vel_setpoint(0.0, &mut can);
    tiny2.set_vel_setpoint(0.0, &mut can);
    tiny3.set_vel_setpoint(0.0, &mut can);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Ok(())
}
