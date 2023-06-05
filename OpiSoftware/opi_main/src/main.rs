mod tinymovr;
mod wheel_conversion;
use tinymovr::Tinymovr;
use wheel_conversion::{LocalVelocity, WheelVelocity};
use anyhow::Context;
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::env;
use ftdi_embedded_hal as hal;

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    
    let device = libftd2xx::Ft2232h::with_description("Single RS232-HS A")?;

    //let mut sock = CanSocket::open("can0")
        //.with_context(|| format!("Failed to open socket on interface can0"))?;
    //let device = hal::find_by_vid_pid(0x0403, 0x6014)
        //.interface(hal::Interface::A)
        //.open()?;
    
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

