mod tinymovr;
mod wheel_conversion;
use embedded_graphics::prelude::{WebColors, RgbColor};
use embedded_hal::spi::FullDuplex;
use tinymovr::Tinymovr;
use wheel_conversion::{LocalVelocity, WheelVelocity};
use anyhow::Context;
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::env;
use ftdi_embedded_hal as hal;
use libftd2xx::{Ftdi, FtdiCommon};
use libftd2xx::list_devices;
use embedded_graphics::{
    pixelcolor::Rgb565,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::PwmPin;
use linux_embedded_hal::Delay;
use display_interface_spi::SPIInterface;
use embedded_graphics::draw_target::DrawTarget;

struct PwmDud;

impl PwmPin for PwmDud {
    type Duty = u16;

    fn disable(&mut self) {}
    fn enable(&mut self) {}
    fn get_duty(&self) -> Self::Duty { 0 }
    fn get_max_duty(&self) -> Self::Duty { 0 }
    fn set_duty(&mut self, _duty: Self::Duty) {}
}

fn main() -> anyhow::Result<()> {
    
    // sudo rmmod ftdi_sio
    // sudo rmmod usbserial
    
    let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
    let hal = hal::FtHal::init_freq(device, 30_000_000)?;
    let mut spi = hal.spi()?;
    let spi_cs = hal.ad3()?;
    let dc = hal.ad4()?;
    let rst = hal.ad5()?;

    let spi_interface = SPIInterface::new(spi, dc, spi_cs);
    let pwm_dud = PwmDud;
    let mut delay = linux_embedded_hal::Delay;

    let mut display = gc9a01a::GC9A01A::new(spi_interface, rst, pwm_dud);
    display.reset(&mut delay).unwrap();
    // Initialize registers
    display.initialize(&mut delay).unwrap();
    // Fill screen with single color
    display.clear(Rgb565::WHITE).unwrap();
    println!("ASDASD");
    display.clear(Rgb565::RED).unwrap();
    println!("GOT HERE");


    let mut sock = CanSocket::open("can0")
        .with_context(|| format!("Failed to open socket on interface can0"))?;

    
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

