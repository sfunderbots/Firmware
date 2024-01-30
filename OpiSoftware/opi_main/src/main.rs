mod tinymovr;
mod wheel_conversion;
use embedded_graphics::prelude::{WebColors, RgbColor};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::PinState;
use linux_embedded_hal::spidev::{SpidevOptions, SpiModeFlags};
use tinymovr::Tinymovr;
use wheel_conversion::{LocalVelocity, WheelVelocity};
use anyhow::Context;
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::env;
use embedded_graphics::prelude::*;
use embedded_graphics::*;
use embedded_graphics::{
    pixelcolor::Rgb565,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};
    use embedded_graphics::{
        mono_font::{ascii::*, MonoTextStyle},
        prelude::*,
        text::{Alignment, Text},
    };
use embedded_graphics::text::*;
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::PwmPin;
use linux_embedded_hal::{Delay, Pin, Spidev};
use display_interface_spi::SPIInterface;
use embedded_graphics::draw_target::DrawTarget;
use rppal::gpio::Gpio;
use embedded_graphics::{image::Image, prelude::*};
use tinybmp::Bmp;

struct PwmDud;
use embedded_graphics_framebuf::FrameBuf;


impl PwmPin for PwmDud {
    type Duty = u16;

    fn disable(&mut self) {}
    fn enable(&mut self) {}
    fn get_duty(&self) -> Self::Duty { 0 }
    fn get_max_duty(&self) -> Self::Duty { 0 }
    fn set_duty(&mut self, _duty: Self::Duty) {}
}

struct GpioDud;

impl OutputPin for GpioDud {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> { Ok(()) }
}

fn main() -> anyhow::Result<()> {
    
    let gpio = Gpio::new()?;
    //let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
    //let hal = hal::FtHal::init_freq(device, 30_000_000)?;
    //let mut spi = hal.spi()?;
    //let mut spi_device = BufferedSpi::new(spi);
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

    // Create a text at position (20, 30) and draw it using the previously defined style
    Text::with_alignment(
        "Hello Rust!",
        Point::new(100, 100),
        blue_style,
        Alignment::Center,
    )
    .draw(&mut display_left)
    .unwrap();

    Text::with_alignment(
        "Hello Rust!",
        Point::new(100, 100),
        red_style,
        Alignment::Center,
    )
    .draw(&mut display_right)
    .unwrap();

let mut data_left = [Rgb565::WHITE; 240 * 240];
let mut data_right = [Rgb565::WHITE; 240 * 240];

        // load current time
        let now = std::time::Instant::now();
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
        display_right.fill_contiguous(&right_area, data_right).unwrap();
    }

    //let mut sock = CanSocket::open("can0")
        //.with_context(|| format!("Failed to open socket on interface can0"))?;

    //let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    //let spi = hal.spi()?;
    //let gpio = hal.ad6();

    //println!("Socket opened");

    //let mut tiny1 = Tinymovr::new(1, &mut sock);
    //let mut tiny2 = Tinymovr::new(2, &mut sock);
    //let mut tiny3 = Tinymovr::new(3, &mut sock);

    //let mut gilrs = Gilrs::new().unwrap();

    //// Iterate over all connected gamepads
    //for (_id, gamepad) in gilrs.gamepads() {
        //println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    //}

    //let mut active_gamepad = None;

    //loop {
        //// Examine new events
        //while let Some(Event { id, event, time }) = gilrs.next_event() {
            //println!("{:?} New event from {}: {:?}", time, id, event);
            //active_gamepad = Some(id);
        //}

        //// You can also use cached gamepad state
        //if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            //if gamepad.is_pressed(Button::South) {
                //println!("Button South is pressed (XBox - A, PS - X)");
            //}
        //}
    //}

    //println!("START CALIBRATION");
    //tiny1.calibrate(&mut sock);
    //tiny2.calibrate(&mut sock);
    //tiny3.calibrate(&mut sock);

    //tiny1.velocity_control(&mut sock);
    //tiny2.velocity_control(&mut sock);
    //tiny3.velocity_control(&mut sock);

    //// Sleep
    //tiny1.set_vel_integrator_params(0.02, 300.0, &mut sock);
    //tiny2.set_vel_integrator_params(0.02, 300.0, &mut sock);
    //tiny3.set_vel_integrator_params(0.02, 300.0, &mut sock);

    //tiny1.set_vel_setpoint(500000.0, &mut sock);
    //tiny2.set_vel_setpoint(500000.0, &mut sock);
    //tiny3.set_vel_setpoint(500000.0, &mut sock);

    //std::thread::sleep(std::time::Duration::from_secs(5));

    //tiny1.set_vel_setpoint(0.0, &mut sock);
    //tiny2.set_vel_setpoint(0.0, &mut sock);
    //tiny3.set_vel_setpoint(0.0, &mut sock);

    //loop {
        //std::thread::sleep(std::time::Duration::from_secs(5));
    //}
    Ok(())
}

