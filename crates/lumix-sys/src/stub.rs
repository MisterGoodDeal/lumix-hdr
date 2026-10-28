//! Stub hors Windows : le SDK LUMIX n'est pas disponible.

use std::os::raw::c_int;

pub const LMX_BOOL_TRUE: u8 = 1;
pub const LMX_BOOL_FALSE: u8 = 0;
pub const LMX_CONNECT_VER: u32 = 0x0001_0001;
pub const LMX_PTP_ARRAY_MAX: usize = 512;
pub const LMX_MODEL_UTF8_STRIDE: usize = 256;
pub const LMX_DEF_LIB_EVENT_ID_ISO: u32 = 0x0200_0020;
pub const LMX_DEF_LIB_EVENT_ID_SHUTTER: u32 = 0x0200_0030;
pub const LMX_DEF_LIB_EVENT_ID_APERTURE: u32 = 0x0200_0040;
pub const LMX_DEF_LIB_EVENT_ID_OBJCT_ADD: u32 = 0x1000_0040;
pub const LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER: u32 = 0x1000_0043;
pub const LMX_DEF_OBJ_FORMAT_UNKNOWN: u32 = 0;
pub const LMX_DEF_OBJ_FORMAT_JPEG: u32 = 1;
pub const LMX_DEF_OBJ_FORMAT_RAW: u32 = 2;
pub const LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M: u32 = 3;
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_CAMERA: u16 = 0;
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_PC: u16 = 1;
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_PC_AND_CAMERA: u16 = 2;
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_BULB: u32 = 0xFFFF_FFFF;
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_UNKNOWN: u32 = 0x0FFF_FFFE;
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_AUTO: u32 = 0x0FFF_FFFF;
pub const LMX_SS_SECONDS_FLAG: u32 = 0x8000_0000;
pub const LMX_DEF_F_UNKNOWN: u16 = 0xFFFE;
pub const LMX_DEF_F_AUTO: u16 = 0xFFFF;
pub const LMX_DEF_ERR_COM_DATA_RCV: u32 = 0x0004_0003;
pub const LMX_DEF_ERR_COM_DATA_BUSY: u32 = 0x0004_0004;
pub const LMX_DEF_LIVEVIEW_STREAMDATA_SIZE_MAX: usize = 1024 * 1024;
pub const LUMIX_SDK_DLL: Option<&str> = None;

pub type LmxSysCallbackFn =
    Option<unsafe extern "system" fn(event_type: u32, event_param: u32) -> c_int>;

fn unavailable() -> ! {
    panic!("SDK LUMIX indisponible hors Windows");
}

pub unsafe fn lmx_sys_init() {
    unavailable();
}

pub fn decode_ss_seconds(code: u32) -> Option<f64> {
    match code {
        LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_BULB
        | LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_UNKNOWN
        | LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_AUTO => None,
        _ if code & LMX_SS_SECONDS_FLAG == 0 => {
            let denom = code as f64 / 1000.0;
            if denom <= 0.0 {
                None
            } else {
                Some(1.0 / denom)
            }
        }
        _ => {
            let scaled = (code & !LMX_SS_SECONDS_FLAG) as f64 / 1000.0;
            if scaled <= 0.0 {
                None
            } else {
                Some(scaled)
            }
        }
    }
}

pub fn encode_ss_seconds(seconds: f64) -> Option<u32> {
    if !(seconds.is_finite()) || seconds <= 0.0 {
        return None;
    }
    if seconds < 1.0 {
        let code = ((1.0 / seconds) * 1000.0).round() as u32;
        if code == 0 {
            None
        } else {
            Some(code)
        }
    } else {
        let scaled = (seconds * 1000.0).round() as u32;
        if scaled == 0 {
            None
        } else {
            Some(scaled | LMX_SS_SECONDS_FLAG)
        }
    }
}

pub fn nearest_ss_code(seconds: f64, codes: &[u32]) -> Option<(u32, f64)> {
    let mut best: Option<(u32, f64, f64)> = None;
    for &code in codes {
        let Some(sec) = decode_ss_seconds(code) else {
            continue;
        };
        let err = (sec - seconds).abs();
        match best {
            None => best = Some((code, sec, err)),
            Some((_, _, best_err)) if err < best_err => best = Some((code, sec, err)),
            _ => {}
        }
    }
    best.map(|(code, sec, _)| (code, sec))
}

pub fn decode_aperture_f(code: u16) -> Option<f32> {
    match code {
        LMX_DEF_F_UNKNOWN | LMX_DEF_F_AUTO => None,
        _ => Some((code as f32) / 10.0),
    }
}

pub fn encode_aperture_f(aperture: f32) -> u32 {
    ((aperture * 10.0).round() as i32).clamp(0, u16::MAX as i32) as u32
}

pub fn decode_iso(code: u32) -> u32 {
    code & 0x0FFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fractional_ss() {
        let sec = decode_ss_seconds(60_000).expect("1/60");
        assert!((sec - 1.0 / 60.0).abs() < 1e-9);
    }
}
