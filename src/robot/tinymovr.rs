use core::ops::Shl;

use cortex_m_semihosting::hprintln;
use nb::block;

/// Command ID's
use embedded_hal::can::{Can, Frame, StandardId};

const RECV_DELAY_US: f32 = 160.0;
const CAN_EP_SIZE: u8 = 6;
const GET_DEVICE_INFO_EP_ID: u16 = 0x1A;
const GET_STATE_EP_ID: u16 = 0x03;
const GET_ENCODER_ESTIMATES_EP_ID: u16 = 0x09;
const GET_IQ_SET_EST_EP_ID: u16 = 0x14;
const GET_SETPOINTS_EP_ID: u16 = 0x0A;
const SET_STATE_EP_ID: u16 = 0x07;
const SET_POS_SETPOINT_EP_ID: u16 = 0x0C;
const SET_VEL_SETPOINT_EP_ID: u16 = 0x0D;
const SET_IQ_SETPOINT_EP_ID: u16 = 0x0E;
const SET_LIMITS_EP_ID: u16 = 0x0F;
const GET_LIMITS_EP_ID: u16 = 0x15;
const RESET_EP_ID: u16 = 0x16;
const MOVE_TO_POS_WITH_VEL_LIMIT_EP_ID: u16 = 0x21;
const SET_MAX_PLAN_ACCEL_DECEL_EP_ID: u16 = 0x22;
const GET_MAX_PLAN_ACCEL_DECEL_EP_ID: u16 = 0x23;

#[derive(Default, Debug, Copy, Clone)]
pub struct TinymovrDeviceInfo {
    pub device_id: u32,
    pub fw_major: u8,
    pub fw_minor: u8,
    pub fw_patch: u8,
    pub temperature: u8,
}

pub struct Tinymovr {
    device_id: u16,
    pub device_info: TinymovrDeviceInfo,
}

impl Tinymovr {
    pub fn new<CAN: Can>(device_id: u16, mut can: &mut CAN) -> Tinymovr {
        let id = StandardId::new(device_id.shl(CAN_EP_SIZE) | GET_DEVICE_INFO_EP_ID).unwrap();
        let frame = Frame::new_remote(id, 0).unwrap();

        block!(can.transmit(&frame)).unwrap();
        let data = block!(can.receive()).unwrap();
        let data = data.data();

        Tinymovr {
            device_id,
            device_info: TinymovrDeviceInfo {
                device_id: data[0] as u32, // TODO FIX ME!
                fw_major: data[4],
                fw_minor: data[5],
                fw_patch: data[6],
                temperature: data[7],
            },
        }
    }

    pub fn device_info(&self) -> TinymovrDeviceInfo {
        self.device_info
    }

    fn set_state<CAN: Can>(&mut self, state: u8, mode: u8, can: &mut CAN) {
        let id = StandardId::new(self.device_id.shl(CAN_EP_SIZE) | SET_STATE_EP_ID).unwrap();
        let frame = Frame::new(id, &[state, mode]).unwrap();
        block!(can.transmit(&frame)).unwrap();
    }

    pub fn idle<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(0, 0, can);
    }

    pub fn calibrate<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(1, 0, can);
    }

    pub fn cl_control<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(2, 0, can);
    }

    pub fn position_control<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(2, 2, can);
    }

    pub fn velocity_control<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(2, 1, can);
    }

    pub fn current_control<CAN: Can>(&mut self, can: &mut CAN) {
        self.set_state(2, 0, can);
    }
}
