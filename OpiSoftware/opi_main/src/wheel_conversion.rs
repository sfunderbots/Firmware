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
        let p = 60. * M_PI / 180.;
        let cos_p = std::cos(p);
        let sin_p = std::sin(p);
        let robot_radius = 18.0;

        let local_to_wheel = Matrix3::new(
            1, 0, robot_radius,
            -cos_p, sin_p, robot_radius,
            -cos_p, -sin_p, robot_radius
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
        let x = (wheel.left + wheel.right + wheel.back) / 3.0;
        let y = (-wheel.left + wheel.right + wheel.back) / 3.0;
        let theta = (-wheel.left - wheel.right + wheel.back) / 3.0;
        Self { x, y, theta }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_to_local() {
        let wheel = WheelVelocity {
            left: 1.0,
            right: 1.0,
            back: 1.0,
        };
        let local = LocalVelocity::from(wheel);
        assert_eq!(local.x, 0.0);
        assert_eq!(local.y, 0.0);
        assert_eq!(local.theta, 1.0);
    }

    #[test]
    fn test_local_to_wheel() {
        let local = LocalVelocity {
            x: 1.0,
            y: 0.0,
            theta: 0.0,
        };
        let wheel = WheelVelocity::from(local);
        assert_eq!(wheel.left, 1.0);
        assert_eq!(wheel.right, 1.0);
        assert_eq!(wheel.back, 1.0);
    }
}
