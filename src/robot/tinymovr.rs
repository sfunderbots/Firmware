/// Command ID's
use embedded_can::{Frame, Id, StandardId};
use embedded_can::nb::Can;
const GET_DEVICE_INFO_EP_ID: u32 = 0x01A;
const GET_STATE_EP_ID: u32 = 0x003;
const GET_ENCODER_ESTIMATES_EP_ID: u32 = 0x009;
const GET_IQ_SET_EST_EP_ID: u32 = 0x014;
const GET_SETPOINTS_EP_ID: u32 = 0x00A;
const SET_STATE_EP_ID: u32 = 0x007;
const SET_POS_SETPOINT_EP_ID: u32 = 0x00C;
const SET_VEL_SETPOINT_EP_ID: u32 = 0x00D;
const SET_IQ_SETPOINT_EP_ID: u32 = 0x00E;
const SET_LIMITS_EP_ID: u32 = 0x00F;
const GET_LIMITS_EP_ID: u32 = 0x015;
const RESET_EP_ID: u32 = 0x016;
const MOVE_TO_POS_WITH_VEL_LIMIT_EP_ID: u32 = 0x021;
const SET_MAX_PLAN_ACCEL_DECEL_EP_ID: u32 = 0x022;
const GET_MAX_PLAN_ACCEL_DECEL_EP_ID: u32 = 0x023;

struct TinymovrDeviceInfo
{
    device_id: u32,
    fw_major: u32,
    fw_minor: u32,
    fw_patch: u32,
    temperature: u32,
}

struct Tinymovr<CAN>
{
    can: CAN,
    device_info: TinymovrDeviceInfo,
}

impl <CAN, FRAME, ERROR> Tinymovr<CAN>
where
    CAN: Can<Frame = FRAME, Error = ERROR>

{
    pub fn new(can: CAN) -> Tinymovr<CAN>
    {
        // TODO actually use the CAN interface and query the device for info
        Tinymovr {
            can,
            device_info: TinymovrDeviceInfo {
                device_id: 0,
                fw_major: 0,
                fw_minor: 0,
                fw_patch: 0,
                temperature: 0,
            },
        }
    }
}
