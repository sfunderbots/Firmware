use core::ops::Shl;

/// Command ID's
use embedded_hal::can::{Frame, Can, StandardId};
use cortex_m_semihosting::hprintln;

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

#[derive(Default)]
pub struct TinymovrDeviceInfo
{
    pub device_id: u32,
    pub fw_major: u8,
    pub fw_minor: u8,
    pub fw_patch: u8,
    pub temperature: u8,
}

pub struct Tinymovr<CAN>
{
    can: CAN,
    pub device_info: TinymovrDeviceInfo,
}

impl <CAN> Tinymovr<CAN>
where CAN: Can,
{
    pub fn new(device_id: u16, mut can: CAN) -> Tinymovr<CAN>
    {
        //let id = StandardId::new(
                    //device_id.shl(CAN_EP_SIZE) | GET_DEVICE_INFO_EP_ID).unwrap();
        ////hprintln!("id: {:?}", id);
        //let frame = Frame::new_remote(id, 0).unwrap();
        //can.transmit(&frame).unwrap();

        //let device_info_frame = can.receive();
        //hprintln!("device_info_frame: {:?}", device_info_frame.err());
        //let data = device_info_frame.unwrap().data();

        Tinymovr {
            can,
            device_info: TinymovrDeviceInfo {
                ..Default::default()
                //device_id: u32::from_le_bytes(data[0..3].try_into().unwrap()),
                //fw_major: data[4],
                //fw_minor: data[5],
                //fw_patch: data[6],
                //temperature: data[7],
            },
        }
    }
    pub fn bruh(&mut self) {
        let device_id = 0u16;
        let id = StandardId::new(
                    device_id.shl(CAN_EP_SIZE) | GET_DEVICE_INFO_EP_ID).unwrap();
        let frame = Frame::new_remote(id, 0).unwrap();
        self.can.transmit(&frame).unwrap();
        let device_info_frame = self.can.receive();
        hprintln!("device_info_frame: {:?}", device_info_frame.err());
    }
}
