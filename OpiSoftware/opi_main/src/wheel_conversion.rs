use nalgebra::{Matrix3, Vector3};

#[derive(Debug)]
pub struct LocalVelocity {
    x: f64,
    y: f64,
    theta: f64,
}

#[derive(Debug)]
pub struct WheelVelocity {
    left: f64,
    right: f64,
    back: f64,
}

impl From<LocalVelocity> for WheelVelocity {
    fn from(local: LocalVelocity) -> Self {
        let p = 60. * std::f64::consts::PI / 180.;
        let cos_p = p.cos();
        let sin_p = p.sin();
        let robot_radius = 18.0;

        let local_to_wheel = Matrix3::new(
            1.0, 0.0, robot_radius,
            -cos_p, sin_p, robot_radius,
            -cos_p, -sin_p, robot_radius,
        );

        let result = local_to_wheel * Vector3::new(local.x, local.y, local.theta);

        Self {
            left: result[0],
            right: result[1],
            back: result[2],
        }
    }
}

impl From<WheelVelocity> for LocalVelocity {
    fn from(wheel: WheelVelocity) -> Self {
        let p = 60. * std::f64::consts::PI / 180.;
        let cos_p = p.cos();
        let sin_p = p.sin();
        let robot_radius = 18.0;

        let i1: f64 = 1.0 / (cos_p + 1.0);
        let i2: f64 = -1.0 / (2 * cos_p + 2);
        let j = 1 / (2.0 * sin_p);
        let k1 = cos_p / (robot_radius * cos_p + robot_radius);
        let k2 = 1.0 / (2.0 * robot_radius * cos_p + 2.0 * robot_radius);

        let wheel_to_local = Matrix3::new(
            i1, i2, i2,
            0, j, -j,
            k1, k2, k2
        );

        let result = wheel_to_local * Vector3::new(wheel.left, wheel.right, wheel.back);

        Self {
            x: result[0],
            y: result[1],
            theta: result[2],
        }
    }
}
