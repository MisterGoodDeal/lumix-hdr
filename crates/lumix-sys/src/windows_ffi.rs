//! Bindings Windows (shim + constantes headers LUMIX).

use std::os::raw::{c_char, c_int};

/// `LMX_BOOL_TRUE` / `LMX_BOOL_FALSE` (`Lmx_lib_type.h`).
pub const LMX_BOOL_TRUE: u8 = 1;
pub const LMX_BOOL_FALSE: u8 = 0;

/// `Open_Session` sample : `0x00010001`.
pub const LMX_CONNECT_VER: u32 = 0x0001_0001;

/// `LMX_DEF_USER_PTP_ARRAY_MAX`.
pub const LMX_PTP_ARRAY_MAX: usize = 512;

/// Longeur UTF-8 d'un nom de modèle PnP (helper shim).
pub const LMX_MODEL_UTF8_STRIDE: usize = 256;

/// `Lmx_event_id` (`LMX_lib_def.h`).
pub const LMX_DEF_LIB_EVENT_ID_ISO: u32 = 0x0200_0020;
pub const LMX_DEF_LIB_EVENT_ID_SHUTTER: u32 = 0x0200_0030;
pub const LMX_DEF_LIB_EVENT_ID_APERTURE: u32 = 0x0200_0040;
pub const LMX_DEF_LIB_EVENT_ID_OBJCT_ADD: u32 = 0x1000_0040;
pub const LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER: u32 = 0x1000_0043;

/// `Lmx_def_lib_object_format`.
pub const LMX_DEF_OBJ_FORMAT_UNKNOWN: u32 = 0;
pub const LMX_DEF_OBJ_FORMAT_JPEG: u32 = 1;
pub const LMX_DEF_OBJ_FORMAT_RAW: u32 = 2;

/// `Lmx_def_lib_Camera_Mode_Info_Mode_Pos`.
pub const LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M: u32 = 3;

/// `Lmx_def_lib_setup_cfg_filesetting_target_cfg`.
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_CAMERA: u16 = 0;
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_PC: u16 = 1;
pub const LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_PC_AND_CAMERA: u16 = 2;

/// Codes SS spéciaux (`Lmx_def_lib_DevpropEx_ShutterSpeed_param`).
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_BULB: u32 = 0xFFFF_FFFF;
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_UNKNOWN: u32 = 0x0FFF_FFFE;
pub const LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_AUTO: u32 = 0x0FFF_FFFF;

/// Bit « secondes » du code SS (doc / sample `GetStringSS`).
pub const LMX_SS_SECONDS_FLAG: u32 = 0x8000_0000;

/// Aperture spéciales.
pub const LMX_DEF_F_UNKNOWN: u16 = 0xFFFE;
pub const LMX_DEF_F_AUTO: u16 = 0xFFFF;

/// `LMX_DEF_ERR_COM_DATA_RCV` (`LMX_lib_def.h`) — souvent « pas de frame encore ».
pub const LMX_DEF_ERR_COM_DATA_RCV: u32 = 0x0004_0003;
/// `LMX_DEF_ERR_COM_DATA_BUSY` (`LMX_lib_def.h`).
pub const LMX_DEF_ERR_COM_DATA_BUSY: u32 = 0x0004_0004;

/// `LMX_DEF_LIVEVIEW_STREAMDATA_SIZE_MAX`.
pub const LMX_DEF_LIVEVIEW_STREAMDATA_SIZE_MAX: usize = 1024 * 1024;

/// Chemin DLL fourni par `build.rs` (si présent).
pub const LUMIX_SDK_DLL: Option<&str> = option_env!("LUMIX_SDK_DLL");

pub type LmxSysCallbackFn =
    Option<unsafe extern "system" fn(event_type: u32, event_param: u32) -> c_int>;

unsafe extern "C" {
    pub fn lmx_sys_init();
    pub fn lmx_sys_close_device(ret_error: *mut u32) -> u8;
    pub fn lmx_sys_get_pnp_device_info(
        plmx_pnp_dev_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_select_pnp_device(
        dw_dev_index: u32,
        plmx_pnp_dev_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_reg_notify_callback(callback_type: u32, appfunc: LmxSysCallbackFn) -> u32;
    pub fn lmx_sys_delete_callback_info(callback_type: u32) -> u32;

    pub fn lmx_sys_iso_get_param(pul_param: *mut u32, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_iso_set_param(ul_param: u32, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_iso_get_capability(
        p_iso_capa_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;

    pub fn lmx_sys_ss_get_capability(
        p_ss_capa_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_ss_get_param(pul_param: *mut u32, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_ss_set_param(ul_param: u32, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_aperture_get_capability(
        p_aperture_capa_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_aperture_get_param(pul_param: *mut u32, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_aperture_set_param(ul_param: u32, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_camera_mode_get_capability(
        p_camera_mode_capa_info: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_camera_mode_get_mode_pos(pul_param: *mut u32, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_camera_mode_set_mode_pos(ul_param: u32, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_rec_ctrl_release(lp_rec_ctrl: *mut std::ffi::c_void, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_get_object(
        object_handle: u32,
        lp_store_buf: *mut u8,
        store_buf_size: u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_get_object_format_type(
        obj_handle: u32,
        p_format_type: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_get_object_data_size(
        obj_handle: u32,
        p_data_size: *mut u64,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_get_object_file_name(
        obj_handle: u32,
        p_file_name: *mut std::ffi::c_void,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_get_partial_object_ex(
        object_handle: u32,
        lp_store_buf: *mut u8,
        data_offset: u64,
        split_transfer_bytes: u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_skip_object_transfer(object_handle: u32, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_setup_files_config_get_target(pun_param: *mut u16, ret_error: *mut u32) -> u8;
    pub fn lmx_sys_setup_files_config_set_target(un_param: u16, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_open_session(
        ul_connect_ver: u32,
        pul_device_connect_ver: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_close_session(ret_error: *mut u32) -> u8;

    pub fn lmx_sys_list_pnp_models(
        out_models_utf8: *mut c_char,
        model_stride: u32,
        max_models: u32,
        out_count: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_select_pnp_index(dw_dev_index: u32, ret_error: *mut u32) -> u8;

    pub fn lmx_sys_ss_fetch_support(
        out_codes: *mut u32,
        max_codes: u32,
        out_count: *mut u32,
        out_cur: *mut u32,
        out_range_min: *mut u32,
        out_range_max: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_iso_fetch_support(
        out_codes: *mut u32,
        max_codes: u32,
        out_count: *mut u32,
        out_cur: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_aperture_fetch_support(
        out_codes: *mut u16,
        max_codes: u32,
        out_count: *mut u32,
        out_cur: *mut u16,
        ret_error: *mut u32,
    ) -> u8;

    pub fn lmx_sys_rec_ctrl_release_oneshot(ret_error: *mut u32) -> u8;

    pub fn lmx_sys_ctrl_liveview_start(ret_error: *mut u32) -> u8;
    pub fn lmx_sys_ctrl_liveview_stop(ret_error: *mut u32) -> u8;
    pub fn lmx_sys_get_liveview_jpeg(
        jpeg_buf: *mut u8,
        jpeg_buf_capacity: u32,
        out_jpeg_size: *mut u32,
        ret_error: *mut u32,
    ) -> u8;
    pub fn lmx_sys_liveview_set_recom_quality_low(ret_error: *mut u32) -> u8;
}

/// Décode un code SS LUMIX (sample `GetStringSS` / doc HTML §4.4.1).
pub fn decode_ss_seconds(code: u32) -> Option<f64> {
    match code {
        LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_BULB
        | LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_UNKNOWN
        | LMX_DEF_PTP_DEVPROP_EXT_LMX_SS_AUTO => None,
        _ if code & LMX_SS_SECONDS_FLAG == 0 => {
            // Ex. 3200 → dénominateur 3.2 → 1/3.2 s (pas une division entière).
            let denom = code as f64 / 1000.0;
            if denom <= 0.0 {
                None
            } else {
                Some(1.0 / denom)
            }
        }
        _ => {
            // Doc : secondes × 1000 (peut être fractionnaire, ex. 1.6" → 1600).
            let scaled = (code & !LMX_SS_SECONDS_FLAG) as f64 / 1000.0;
            if scaled <= 0.0 {
                None
            } else {
                Some(scaled)
            }
        }
    }
}

/// Encode des secondes → code PTP LUMIX (doc HTML §4.4.1).
pub fn encode_ss_seconds(seconds: f64) -> Option<u32> {
    if !(seconds.is_finite()) || seconds <= 0.0 {
        return None;
    }
    if seconds < 1.0 {
        // Dénominateur × 1000 (ex. 1/3.2 → 3200), pas (round denom)×1000.
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

/// Choisit le code SS supporté le plus proche de `seconds` (via capacités runtime).
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
        // 1/60 → code 60000
        let sec = decode_ss_seconds(60_000).expect("1/60");
        assert!((sec - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn decode_long_ss() {
        // 2" → 2000 | 0x80000000
        let code = 2_000 | LMX_SS_SECONDS_FLAG;
        let sec = decode_ss_seconds(code).expect("2\"");
        assert!((sec - 2.0).abs() < 1e-9);
    }

    #[test]
    fn nearest_from_capa() {
        let codes = [30_000, 60_000, 125_000];
        let (code, sec) = nearest_ss_code(1.0 / 55.0, &codes).expect("near");
        assert_eq!(code, 60_000);
        assert!((sec - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn encode_matches_doc_formula() {
        // Formule doc §4.4.1 (les hex d'exemple du PDF sont parfois faux :
        // 30×1000=30000=0x7530, pas 0xC350).
        assert_eq!(encode_ss_seconds(30.0), Some(30_000 | LMX_SS_SECONDS_FLAG));
        assert_eq!(encode_ss_seconds(1.6), Some(1_600 | LMX_SS_SECONDS_FLAG));
        assert_eq!(encode_ss_seconds(1.0 / 30.0), Some(30_000));
        assert_eq!(encode_ss_seconds(1.0 / 3.2), Some(3_200));
        assert_eq!(decode_ss_seconds(30_000 | LMX_SS_SECONDS_FLAG), Some(30.0));
        assert!((decode_ss_seconds(3_200).expect("1/3.2") - 1.0 / 3.2).abs() < 1e-9);
    }
}
