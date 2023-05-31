mod tinymovr;
use tinymovr::Tinymovr;

use anyhow::Context;
use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::env;

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let mut sock = CanSocket::open("can0")
        .with_context(|| format!("Failed to open socket on interface can0"))?;
    println!("Socket opened");

    let mut tiny1 = Tinymovr::new(1, &mut sock);
    let mut tiny2 = Tinymovr::new(2, &mut sock);
    let mut tiny3 = Tinymovr::new(3, &mut sock);

    println!("Calibration done");
    tiny1.velocity_control(&mut sock);
    println!("Velocity control done");
    tiny2.velocity_control(&mut sock);
    println!("Velocity control done");
    tiny3.velocity_control(&mut sock);
    println!("Velocity control done");

    // Sleep
    std::thread::sleep(std::time::Duration::from_secs(1));
    tiny1.set_vel_integrator_params(0.02, 300.0, &mut sock);
    tiny2.set_vel_integrator_params(0.02, 300.0, &mut sock);
    tiny3.set_vel_integrator_params(0.02, 300.0, &mut sock);
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Setting vel setpoint");
    tiny1.set_vel_setpoint(500000.0, &mut sock);
    tiny2.set_vel_setpoint(500000.0, &mut sock);
    tiny3.set_vel_setpoint(500000.0, &mut sock);

    std::thread::sleep(std::time::Duration::from_secs(5));

    tiny1.set_vel_setpoint(0.0, &mut sock);
    tiny2.set_vel_setpoint(0.0, &mut sock);
    tiny3.set_vel_setpoint(0.0, &mut sock);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    Ok(())
}

fn frame_to_string<F: Frame>(frame: &F) -> String {
    let id = frame.raw_id();
    let data_string = frame
        .data()
        .iter()
        .fold(String::from(""), |a, b| format!("{} {:02x}", a, b));

    format!("{:X}  [{}] {}", id, frame.dlc(), data_string)
}
