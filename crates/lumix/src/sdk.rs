//! `LumixCamera` — implémentation réelle du trait `Camera` via `lumix-sys`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, Once};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use crate::camera::Camera;
use crate::error::CameraError;
use crate::types::{
    format_shutter, standard_shutter_table, CameraSettings, ExposureCapabilities, ExposureMode,
    ShutterLimits,
};

use lumix_sys::{
    decode_aperture_f, decode_iso, decode_ss_seconds, encode_aperture_f, encode_ss_seconds,
    lmx_sys_aperture_fetch_support, lmx_sys_aperture_get_param, lmx_sys_aperture_set_param,
    lmx_sys_camera_mode_get_mode_pos, lmx_sys_camera_mode_set_mode_pos, lmx_sys_close_device,
    lmx_sys_close_session, lmx_sys_ctrl_liveview_start, lmx_sys_ctrl_liveview_stop,
    lmx_sys_delete_callback_info, lmx_sys_get_liveview_jpeg, lmx_sys_get_object_data_size,
    lmx_sys_get_object_format_type, lmx_sys_get_partial_object_ex, lmx_sys_init,
    lmx_sys_iso_fetch_support, lmx_sys_iso_get_param, lmx_sys_iso_set_param,
    lmx_sys_list_pnp_models, lmx_sys_liveview_set_recom_quality_low, lmx_sys_open_session,
    lmx_sys_rec_ctrl_release_oneshot, lmx_sys_reg_notify_callback, lmx_sys_select_pnp_index,
    lmx_sys_setup_files_config_set_target, lmx_sys_skip_object_transfer, lmx_sys_ss_fetch_support,
    lmx_sys_ss_get_param, lmx_sys_ss_set_param, nearest_ss_code, LMX_BOOL_TRUE, LMX_CONNECT_VER,
    LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M, LMX_DEF_ERR_COM_DATA_BUSY, LMX_DEF_ERR_COM_DATA_RCV,
    LMX_DEF_LIB_EVENT_ID_APERTURE, LMX_DEF_LIB_EVENT_ID_ISO, LMX_DEF_LIB_EVENT_ID_OBJCT_ADD,
    LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER, LMX_DEF_LIB_EVENT_ID_SHUTTER,
    LMX_DEF_LIVEVIEW_STREAMDATA_SIZE_MAX, LMX_DEF_OBJ_FORMAT_JPEG, LMX_DEF_OBJ_FORMAT_RAW,
    LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_PC, LMX_MODEL_UTF8_STRIDE, LMX_PTP_ARRAY_MAX,
};

const DOWNLOAD_CHUNK: u32 = 10 * 1024 * 1024;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);
const SHUTTER_VERIFY_TOLERANCE: f64 = 0.02;

/// Résultat du transfert — doit être rempli **dans** le callback REQ_TRNSFER
/// (sinon l'appareil reste bloqué en attente du PC, sample ActionParameters01).
enum TransferOutcome {
    Idle,
    Jpeg(Vec<u8>),
    Failed(String),
}

struct PendingCapture {
    outcome: TransferOutcome,
}

static PENDING: Mutex<PendingCapture> = Mutex::new(PendingCapture {
    outcome: TransferOutcome::Idle,
});
static PENDING_CV: Condvar = Condvar::new();
static SDK_INIT: Once = Once::new();
/// Réglages changés sur l'appareil (callback ISO/SS/F) — à relire hors callback.
static SETTINGS_CHANGED: AtomicBool = AtomicBool::new(false);
/// Ignore les notifs juste après un apply UI (évite un aller-retour inutile).
static SUPPRESS_SETTINGS_NOTIFY: AtomicBool = AtomicBool::new(false);
/// Annulation coopérative pendant `wait_transfer_outcome`.
static CAPTURE_CANCEL: AtomicBool = AtomicBool::new(false);

/// À appeler avant un bracket pour réarmer l’attente transfert.
pub fn begin_capture_wait() {
    CAPTURE_CANCEL.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = PENDING.lock() {
        guard.outcome = TransferOutcome::Idle;
    }
}

/// Annule l’attente PTP (réveille le wait) — appelé depuis `cancel_capture`.
pub fn request_capture_cancel() {
    CAPTURE_CANCEL.store(true, Ordering::SeqCst);
    PENDING_CV.notify_all();
}

fn ensure_sdk_init() {
    SDK_INIT.call_once(|| unsafe {
        lmx_sys_init();
    });
}

/// Résultat du callback REQ_TRNSFER.
/// `None` = objet ignoré (ex. RAW) : Skip fait, on continue d’attendre le JPEG.
fn transfer_jpeg_in_callback(handle: u32) -> Result<Option<Vec<u8>>, String> {
    let mut ret_error = 0u32;
    let mut format = 0u32;
    let ok = unsafe { lmx_sys_get_object_format_type(handle, &mut format, &mut ret_error) };
    if ok != LMX_BOOL_TRUE {
        let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
        return Err(format!(
            "Get_Object_FormatType a échoué (0x{ret_error:08X})"
        ));
    }

    // RAW+JPEG : l’appareil envoie souvent le RAW en premier (ou après le JPEG).
    // Skip sans échouer — sinon WaitCaptureDone plante au milieu du bracket.
    if format == LMX_DEF_OBJ_FORMAT_RAW {
        let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
        debug!(handle, "REQ_TRNSFER RAW ignoré (Skip_Object_Transfer)");
        return Ok(None);
    }
    if format != LMX_DEF_OBJ_FORMAT_JPEG {
        let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
        debug!(handle, format, "REQ_TRNSFER format non-JPEG ignoré");
        return Ok(None);
    }

    let mut data_size = 0u64;
    let ok = unsafe { lmx_sys_get_object_data_size(handle, &mut data_size, &mut ret_error) };
    if ok != LMX_BOOL_TRUE {
        let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
        return Err(format!("Get_Object_DataSize a échoué (0x{ret_error:08X})"));
    }
    if data_size == 0 || data_size > (512 * 1024 * 1024) {
        let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
        return Err(format!("taille objet invalide ({data_size})"));
    }

    let mut out = Vec::with_capacity(data_size as usize);
    let mut offset = 0u64;
    let mut buf = vec![0u8; DOWNLOAD_CHUNK as usize];
    while offset < data_size {
        let remain = data_size - offset;
        let chunk = remain.min(u64::from(DOWNLOAD_CHUNK)) as u32;
        let ok = unsafe {
            lmx_sys_get_partial_object_ex(handle, buf.as_mut_ptr(), offset, chunk, &mut ret_error)
        };
        if ok != LMX_BOOL_TRUE {
            let _ = unsafe { lmx_sys_skip_object_transfer(handle, &mut ret_error) };
            return Err(format!(
                "Get_Partial_ObjectEx a échoué à offset={offset} (0x{ret_error:08X})"
            ));
        }
        out.extend_from_slice(&buf[..chunk as usize]);
        offset += u64::from(chunk);
    }
    Ok(Some(out))
}

/// À appeler après `apply_settings` côté UI pour ne pas ré-émettre tout de suite.
pub fn suppress_settings_notify_briefly() {
    SUPPRESS_SETTINGS_NOTIFY.store(true, Ordering::SeqCst);
    SETTINGS_CHANGED.store(false, Ordering::SeqCst);
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(400));
        SUPPRESS_SETTINGS_NOTIFY.store(false, Ordering::SeqCst);
    });
}

/// Consomme le drapeau « réglages appareil modifiés ».
pub fn take_settings_changed() -> bool {
    if SUPPRESS_SETTINGS_NOTIFY.load(Ordering::SeqCst) {
        SETTINGS_CHANGED.store(false, Ordering::SeqCst);
        return false;
    }
    SETTINGS_CHANGED.swap(false, Ordering::SeqCst)
}

unsafe extern "system" fn notify_callback(event_type: u32, event_param: u32) -> i32 {
    // Sample GetSetParameters01 : ISO / SS / F → marquer dirty (Get_Param hors callback).
    if event_type == LMX_DEF_LIB_EVENT_ID_ISO
        || event_type == LMX_DEF_LIB_EVENT_ID_SHUTTER
        || event_type == LMX_DEF_LIB_EVENT_ID_APERTURE
    {
        SETTINGS_CHANGED.store(true, Ordering::SeqCst);
        return 0;
    }
    // Sample : pour les JPEG, ne traiter que REQ_TRNSFER (le transfert PC).
    if event_type == LMX_DEF_LIB_EVENT_ID_OBJCT_ADD {
        return 0;
    }
    if event_type != LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER {
        return -1;
    }

    // Critique : terminer le transfert (Get_Partial ou Skip) avant de quitter le callback.
    let outcome = match std::panic::catch_unwind(|| transfer_jpeg_in_callback(event_param)) {
        Ok(Ok(Some(jpeg))) => Some(TransferOutcome::Jpeg(jpeg)),
        Ok(Ok(None)) => None, // RAW / autre : Skip déjà fait, on attend toujours le JPEG
        Ok(Err(message)) => Some(TransferOutcome::Failed(message)),
        Err(_) => {
            let mut ret_error = 0u32;
            let _ = unsafe { lmx_sys_skip_object_transfer(event_param, &mut ret_error) };
            Some(TransferOutcome::Failed(
                "panic pendant le transfert JPEG".to_string(),
            ))
        }
    };

    match outcome {
        Some(outcome) => {
            if let Ok(mut guard) = PENDING.lock() {
                // Ne pas écraser un JPEG déjà reçu (RAW tardif d'une vue précédente).
                if !matches!(guard.outcome, TransferOutcome::Jpeg(_)) {
                    guard.outcome = outcome;
                    PENDING_CV.notify_all();
                }
            }
        }
        None => {
            // RAW / non-JPEG skip : réveiller le wait pour qu’il continue d’attendre le JPEG.
            PENDING_CV.notify_all();
        }
    }
    0
}

fn sdk_err(op: &str, ret_error: u32) -> CameraError {
    CameraError::Message(format!("{op} a échoué (retError=0x{ret_error:08X})"))
}

fn check_bool(ok: u8, op: &str, ret_error: u32) -> Result<(), CameraError> {
    if ok == LMX_BOOL_TRUE {
        Ok(())
    } else {
        Err(sdk_err(op, ret_error))
    }
}

/// Caméra LUMIX réelle (Windows + feature `sdk`).
#[derive(Debug)]
pub struct LumixCamera {
    connected: bool,
    device_index: u32,
    model_name: String,
    ss_codes: Vec<u32>,
    limits: ShutterLimits,
    /// JPEG déjà récupéré dans le callback REQ_TRNSFER.
    pending_jpeg: Option<Vec<u8>>,
    saved_target: Option<u16>,
    live_view: bool,
    live_view_buf: Vec<u8>,
}

impl Default for LumixCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl LumixCamera {
    pub fn new() -> Self {
        Self {
            connected: false,
            device_index: 0,
            model_name: String::new(),
            ss_codes: Vec::new(),
            limits: ShutterLimits::default(),
            pending_jpeg: None,
            live_view: false,
            live_view_buf: vec![0u8; LMX_DEF_LIVEVIEW_STREAMDATA_SIZE_MAX],
            saved_target: None,
        }
    }

    pub fn with_device_index(index: u32) -> Self {
        let mut cam = Self::new();
        cam.device_index = index;
        cam
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    fn ensure_connected(&self) -> Result<(), CameraError> {
        if self.connected {
            Ok(())
        } else {
            Err(CameraError::NotConnected)
        }
    }

    fn ensure_mode_m(&mut self) -> Result<(), CameraError> {
        let mut ret_error = 0u32;
        let mut mode = 0u32;
        let ok_get = unsafe { lmx_sys_camera_mode_get_mode_pos(&mut mode, &mut ret_error) };
        if ok_get == LMX_BOOL_TRUE && mode == LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M {
            return Ok(());
        }
        let ok_set = unsafe {
            lmx_sys_camera_mode_set_mode_pos(LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M, &mut ret_error)
        };
        if ok_set == LMX_BOOL_TRUE {
            return Ok(());
        }
        // Certains firmwares renvoient FALSE si déjà en M, ou refusent le Set :
        // on re-lit et on accepte si on est bien en M.
        let mut after = 0u32;
        let ok_after = unsafe { lmx_sys_camera_mode_get_mode_pos(&mut after, &mut ret_error) };
        if ok_after == LMX_BOOL_TRUE && after == LMX_DEF_CAMERA_MODE_INFO_MODE_POS_M {
            warn!("CameraMode_Set_Mode_Pos(M) a renvoyé FALSE mais le mode est M");
            return Ok(());
        }
        Err(sdk_err("CameraMode_Set_Mode_Pos(M)", ret_error))
    }

    fn refresh_ss_capabilities(&mut self) -> Result<(), CameraError> {
        let mut codes = vec![0u32; LMX_PTP_ARRAY_MAX];
        let mut count = 0u32;
        let mut cur = 0u32;
        let mut range_min = 0u32;
        let mut range_max = 0u32;
        let mut ret_error = 0u32;
        let ok = unsafe {
            lmx_sys_ss_fetch_support(
                codes.as_mut_ptr(),
                LMX_PTP_ARRAY_MAX as u32,
                &mut count,
                &mut cur,
                &mut range_min,
                &mut range_max,
                &mut ret_error,
            )
        };
        check_bool(ok, "SS_Get_Capability", ret_error)?;
        codes.truncate(count as usize);

        let mappable: Vec<u32> = codes
            .iter()
            .copied()
            .filter(|c| decode_ss_seconds(*c).is_some())
            .collect();

        self.ss_codes = if !mappable.is_empty() {
            mappable
        } else {
            // Enum vide / non décodable : dériver de CurVal_Range + table standard
            // (encodage doc §4.4.1), jamais de codes inventés hors formule officielle.
            let min_s = decode_ss_seconds(range_min).unwrap_or(1.0 / 8000.0);
            let max_s = decode_ss_seconds(range_max).unwrap_or(30.0);
            let lo = min_s.min(max_s);
            let hi = min_s.max(max_s);
            let mut derived = Vec::new();
            for &(sec, _) in standard_shutter_table() {
                if sec + f64::EPSILON < lo || sec - f64::EPSILON > hi {
                    continue;
                }
                if let Some(code) = encode_ss_seconds(sec) {
                    derived.push(code);
                }
            }
            if derived.is_empty() {
                if let Some(code) = encode_ss_seconds(lo) {
                    derived.push(code);
                }
                if let Some(code) = encode_ss_seconds(hi) {
                    if derived.last().copied() != Some(code) {
                        derived.push(code);
                    }
                }
            }
            if let Some(code) = (cur != 0)
                .then_some(cur)
                .filter(|c| decode_ss_seconds(*c).is_some())
            {
                if !derived.contains(&code) {
                    derived.push(code);
                }
            }
            derived
        };

        if self.ss_codes.is_empty() {
            return Err(CameraError::Message(format!(
                "SS_Get_Capability : aucune vitesse mappable \
                 (enum={count}, range=0x{range_min:08X}..0x{range_max:08X}, cur=0x{cur:08X})"
            )));
        }

        let mut min_s = f64::MAX;
        let mut max_s = 0.0_f64;
        for &code in &self.ss_codes {
            if let Some(sec) = decode_ss_seconds(code) {
                min_s = min_s.min(sec);
                max_s = max_s.max(sec);
            }
        }
        if min_s.is_finite() && max_s > 0.0 && min_s <= max_s {
            self.limits = ShutterLimits {
                min_seconds: min_s,
                max_seconds: max_s,
            };
        }
        info!(
            count = self.ss_codes.len(),
            enum_count = count,
            min = self.limits.min_seconds,
            max = self.limits.max_seconds,
            "capacités SS chargées"
        );
        Ok(())
    }

    fn clear_pending() {
        if let Ok(mut guard) = PENDING.lock() {
            guard.outcome = TransferOutcome::Idle;
        }
    }

    fn wait_transfer_outcome(timeout: Duration) -> Result<Vec<u8>, CameraError> {
        let deadline = Instant::now() + timeout;
        let slice = Duration::from_millis(200);
        let mut guard = PENDING
            .lock()
            .map_err(|_| CameraError::Message("verrou callback empoisonné".to_string()))?;
        loop {
            if CAPTURE_CANCEL.load(Ordering::SeqCst) {
                return Err(CameraError::Message("capture annulée".to_string()));
            }
            match std::mem::replace(&mut guard.outcome, TransferOutcome::Idle) {
                TransferOutcome::Jpeg(bytes) => return Ok(bytes),
                TransferOutcome::Failed(message) => {
                    return Err(CameraError::Message(message));
                }
                TransferOutcome::Idle => {}
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(CameraError::Timeout);
            }
            let wait_for = deadline.saturating_duration_since(now).min(slice);
            let (g, _timeout) = PENDING_CV
                .wait_timeout(guard, wait_for)
                .map_err(|_| CameraError::Message("verrou callback empoisonné".to_string()))?;
            guard = g;
        }
    }

    fn ensure_pc_transfer_target(&mut self) -> Result<(), CameraError> {
        let mut ret_error = 0u32;
        let target = LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_PC;
        let ok = unsafe { lmx_sys_setup_files_config_set_target(target, &mut ret_error) };
        if ok != LMX_BOOL_TRUE {
            warn!(
                ret_error,
                "SetupFilesConfig_Set_Target(ONLY_PC) avant capture a échoué"
            );
        } else {
            self.saved_target = Some(target);
        }
        Ok(())
    }
}

impl Camera for LumixCamera {
    fn connect(&mut self) -> Result<(), CameraError> {
        if self.connected {
            return Ok(());
        }

        ensure_sdk_init();

        let mut models = vec![0u8; LMX_PTP_ARRAY_MAX * LMX_MODEL_UTF8_STRIDE];
        let mut count = 0u32;
        let mut ret_error = 0u32;
        let ok = unsafe {
            lmx_sys_list_pnp_models(
                models.as_mut_ptr().cast(),
                LMX_MODEL_UTF8_STRIDE as u32,
                LMX_PTP_ARRAY_MAX as u32,
                &mut count,
                &mut ret_error,
            )
        };
        check_bool(ok, "Get_PnPDeviceInfo", ret_error)?;
        if count == 0 {
            return Err(CameraError::Disconnected);
        }
        if self.device_index >= count {
            return Err(CameraError::Message(format!(
                "index appareil {} hors plage (0..{})",
                self.device_index,
                count - 1
            )));
        }

        let start = self.device_index as usize * LMX_MODEL_UTF8_STRIDE;
        let end = start + LMX_MODEL_UTF8_STRIDE;
        let raw = &models[start..end];
        self.model_name = std::ffi::CStr::from_bytes_until_nul(raw)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        // Get + Select sur la même structure (comme le sample ActionParameters01).
        let ok = unsafe { lmx_sys_select_pnp_index(self.device_index, &mut ret_error) };
        if ok != LMX_BOOL_TRUE {
            return Err(CameraError::Message(format!(
                "Select_PnPDevice a échoué (retError=0x{ret_error:08X}, count={count}). \
                 L'appareil est peut-être déjà ouvert par une autre session — \
                 reconnectez depuis l'écran Connexion."
            )));
        }

        let mut device_ver = 0u32;
        let ok = unsafe { lmx_sys_open_session(LMX_CONNECT_VER, &mut device_ver, &mut ret_error) };
        if ok != LMX_BOOL_TRUE {
            let _ = unsafe { lmx_sys_close_device(&mut ret_error) };
            return Err(sdk_err("Open_Session", ret_error));
        }

        // ONLY_PC : l'appareil envoie REQ_TRNSFER et attend le Get_Partial côté PC.
        let target = LMX_DEF_SETUP_CFG_FILE_TARGET_CFG_ONLY_PC;
        let ok = unsafe { lmx_sys_setup_files_config_set_target(target, &mut ret_error) };
        if ok != LMX_BOOL_TRUE {
            warn!(
                ret_error,
                "SetupFilesConfig_Set_Target(ONLY_PC) a échoué (on continue)"
            );
        } else {
            self.saved_target = Some(target);
        }

        Self::clear_pending();
        SETTINGS_CHANGED.store(false, Ordering::SeqCst);
        let _ = unsafe {
            lmx_sys_reg_notify_callback(LMX_DEF_LIB_EVENT_ID_OBJCT_ADD, Some(notify_callback))
        };
        let _ = unsafe {
            lmx_sys_reg_notify_callback(
                LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER,
                Some(notify_callback),
            )
        };
        let _ =
            unsafe { lmx_sys_reg_notify_callback(LMX_DEF_LIB_EVENT_ID_ISO, Some(notify_callback)) };
        let _ = unsafe {
            lmx_sys_reg_notify_callback(LMX_DEF_LIB_EVENT_ID_SHUTTER, Some(notify_callback))
        };
        let _ = unsafe {
            lmx_sys_reg_notify_callback(LMX_DEF_LIB_EVENT_ID_APERTURE, Some(notify_callback))
        };

        // L'enum SS est souvent vide hors mode M/S : forcer M avant capability.
        if let Err(error) = self.ensure_mode_m() {
            let _ = self.disconnect();
            return Err(error);
        }

        if let Err(error) = self.refresh_ss_capabilities() {
            let _ = self.disconnect();
            return Err(error);
        }

        self.connected = true;
        info!(
            model = %self.model_name,
            device_ver,
            "LumixCamera connectée"
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), CameraError> {
        let mut ret_error = 0u32;
        if self.live_view {
            let _ = unsafe { lmx_sys_ctrl_liveview_stop(&mut ret_error) };
            self.live_view = false;
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = unsafe { lmx_sys_delete_callback_info(LMX_DEF_LIB_EVENT_ID_OBJCT_ADD) };
        let _ = unsafe { lmx_sys_delete_callback_info(LMX_DEF_LIB_EVENT_ID_OBJCT_REQ_TRNSFER) };
        let _ = unsafe { lmx_sys_delete_callback_info(LMX_DEF_LIB_EVENT_ID_ISO) };
        let _ = unsafe { lmx_sys_delete_callback_info(LMX_DEF_LIB_EVENT_ID_SHUTTER) };
        let _ = unsafe { lmx_sys_delete_callback_info(LMX_DEF_LIB_EVENT_ID_APERTURE) };
        SETTINGS_CHANGED.store(false, Ordering::SeqCst);
        let _ = unsafe { lmx_sys_close_session(&mut ret_error) };
        let _ = unsafe { lmx_sys_close_device(&mut ret_error) };
        // Second Close_Device : libère un Select orphelin (sinon Select suivant → retError=0).
        let _ = unsafe { lmx_sys_close_device(&mut ret_error) };
        Self::clear_pending();
        self.pending_jpeg = None;
        self.connected = false;
        self.ss_codes.clear();
        info!("LumixCamera déconnectée");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn shutter_limits(&self) -> ShutterLimits {
        self.limits
    }

    fn read_settings(&mut self) -> Result<CameraSettings, CameraError> {
        self.ensure_connected()?;
        let mut ret_error = 0u32;
        let mut iso = 0u32;
        let mut ss = 0u32;
        let mut aperture = 0u32;
        let mut mode = 0u32;

        check_bool(
            unsafe { lmx_sys_iso_get_param(&mut iso, &mut ret_error) },
            "ISO_Get_Param",
            ret_error,
        )?;
        check_bool(
            unsafe { lmx_sys_ss_get_param(&mut ss, &mut ret_error) },
            "SS_Get_Param",
            ret_error,
        )?;
        check_bool(
            unsafe { lmx_sys_aperture_get_param(&mut aperture, &mut ret_error) },
            "Aperture_Get_Param",
            ret_error,
        )?;
        check_bool(
            unsafe { lmx_sys_camera_mode_get_mode_pos(&mut mode, &mut ret_error) },
            "CameraMode_Get_Mode_Pos",
            ret_error,
        )?;

        let shutter_seconds = decode_ss_seconds(ss).ok_or_else(|| {
            CameraError::OutOfRange(format!("vitesse non décodable (0x{ss:08X})"))
        })?;
        let aperture_f = decode_aperture_f(aperture as u16).unwrap_or(0.0);
        let _ = mode;

        Ok(CameraSettings {
            mode: ExposureMode::M,
            aperture: aperture_f,
            iso: decode_iso(iso),
            shutter_seconds,
            shutter_label: format_shutter(shutter_seconds),
        })
    }

    fn exposure_capabilities(&mut self) -> Result<ExposureCapabilities, CameraError> {
        self.ensure_connected()?;
        let settings = self.read_settings()?;
        let mut ret_error = 0u32;

        let mut iso_codes = vec![0u32; LMX_PTP_ARRAY_MAX];
        let mut iso_count = 0u32;
        let mut iso_cur = 0u32;
        let ok = unsafe {
            lmx_sys_iso_fetch_support(
                iso_codes.as_mut_ptr(),
                LMX_PTP_ARRAY_MAX as u32,
                &mut iso_count,
                &mut iso_cur,
                &mut ret_error,
            )
        };
        let mut iso_values = if ok == LMX_BOOL_TRUE {
            iso_codes
                .into_iter()
                .take(iso_count as usize)
                .map(decode_iso)
                .filter(|&v| v > 0)
                .collect::<Vec<_>>()
        } else {
            warn!(ret_error, "ISO_Get_Capability a échoué ; table vide");
            Vec::new()
        };
        iso_values.sort_unstable();
        iso_values.dedup();
        if iso_values.is_empty() {
            iso_values.push(settings.iso);
        }

        let mut ap_codes = vec![0u16; LMX_PTP_ARRAY_MAX];
        let mut ap_count = 0u32;
        let mut ap_cur = 0u16;
        let ok = unsafe {
            lmx_sys_aperture_fetch_support(
                ap_codes.as_mut_ptr(),
                LMX_PTP_ARRAY_MAX as u32,
                &mut ap_count,
                &mut ap_cur,
                &mut ret_error,
            )
        };
        let mut aperture_values = if ok == LMX_BOOL_TRUE {
            ap_codes
                .into_iter()
                .take(ap_count as usize)
                .filter_map(decode_aperture_f)
                .collect::<Vec<_>>()
        } else {
            warn!(ret_error, "Aperture_Get_Capability a échoué ; table vide");
            Vec::new()
        };
        aperture_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        aperture_values.dedup_by(|a, b| (*a - *b).abs() < 0.05);
        if aperture_values.is_empty() {
            aperture_values.push(settings.aperture);
        }

        if self.ss_codes.is_empty() {
            let _ = self.refresh_ss_capabilities();
        }
        let mut shutter_seconds = Vec::new();
        let mut shutter_labels = Vec::new();
        for &code in &self.ss_codes {
            if let Some(sec) = decode_ss_seconds(code) {
                shutter_seconds.push(sec);
                shutter_labels.push(format_shutter(sec));
            }
        }
        if shutter_seconds.is_empty() {
            for &(sec, label) in standard_shutter_table() {
                shutter_seconds.push(sec);
                shutter_labels.push(label.to_string());
            }
        }

        Ok(ExposureCapabilities {
            settings,
            iso_values,
            aperture_values,
            shutter_seconds,
            shutter_labels,
        })
    }

    fn apply_settings(&mut self, settings: &CameraSettings) -> Result<(), CameraError> {
        self.ensure_connected()?;
        let mut ret_error = 0u32;

        self.ensure_mode_m()?;
        check_bool(
            unsafe { lmx_sys_iso_set_param(settings.iso, &mut ret_error) },
            "ISO_Set_Param",
            ret_error,
        )?;
        check_bool(
            unsafe {
                lmx_sys_aperture_set_param(encode_aperture_f(settings.aperture), &mut ret_error)
            },
            "Aperture_Set_Param",
            ret_error,
        )?;
        self.set_shutter(settings.shutter_seconds)?;
        Ok(())
    }

    fn set_shutter(&mut self, shutter_seconds: f64) -> Result<(), CameraError> {
        self.ensure_connected()?;
        if self.ss_codes.is_empty() {
            self.refresh_ss_capabilities()?;
        }
        if shutter_seconds < self.limits.min_seconds || shutter_seconds > self.limits.max_seconds {
            return Err(CameraError::OutOfRange(format!(
                "vitesse {shutter_seconds} s hors plage [{}, {}]",
                self.limits.min_seconds, self.limits.max_seconds
            )));
        }
        let (code, mapped) = nearest_ss_code(shutter_seconds, &self.ss_codes).ok_or_else(|| {
            CameraError::OutOfRange(format!(
                "aucune vitesse supportée mappable (table={}, demandé={shutter_seconds}s)",
                self.ss_codes.len()
            ))
        })?;
        let mut ret_error = 0u32;
        check_bool(
            unsafe { lmx_sys_ss_set_param(code, &mut ret_error) },
            "SS_Set_Param",
            ret_error,
        )?;
        debug!(code, shutter_seconds, mapped, "SS définie");
        Ok(())
    }

    fn verify_shutter(&mut self, expected_seconds: f64) -> Result<(), CameraError> {
        self.ensure_connected()?;
        let mut ss = 0u32;
        let mut ret_error = 0u32;
        check_bool(
            unsafe { lmx_sys_ss_get_param(&mut ss, &mut ret_error) },
            "SS_Get_Param",
            ret_error,
        )?;
        let actual = decode_ss_seconds(ss).ok_or_else(|| {
            CameraError::VerifyFailed(format!("vitesse lue non décodable (0x{ss:08X})"))
        })?;
        let (_, mapped) = nearest_ss_code(expected_seconds, &self.ss_codes).ok_or_else(|| {
            CameraError::VerifyFailed("vitesse attendue non mappable".to_string())
        })?;
        let rel = if mapped > 0.0 {
            (actual - mapped).abs() / mapped
        } else {
            (actual - mapped).abs()
        };
        if rel > SHUTTER_VERIFY_TOLERANCE {
            return Err(CameraError::VerifyFailed(format!(
                "attendu ~{mapped:.6}s, lu {actual:.6}s"
            )));
        }
        Ok(())
    }

    fn trigger(&mut self) -> Result<(), CameraError> {
        self.ensure_connected()?;
        // Après Live View, réaffirmer ONLY_PC sinon le GH6 peut ne pas envoyer REQ_TRNSFER.
        self.ensure_pc_transfer_target()?;
        Self::clear_pending();
        self.pending_jpeg = None;
        let mut ret_error = 0u32;
        info!("déclenchement ONESHOT");
        check_bool(
            unsafe { lmx_sys_rec_ctrl_release_oneshot(&mut ret_error) },
            "Rec_Ctrl_Release(ONESHOT)",
            ret_error,
        )?;
        debug!("ONESHOT accepté — attente REQ_TRNSFER");
        Ok(())
    }

    fn wait_capture_done(&mut self) -> Result<(), CameraError> {
        self.ensure_connected()?;
        // Le JPEG a déjà été lu dans le callback ; on attend juste la fin.
        let jpeg = Self::wait_transfer_outcome(CAPTURE_TIMEOUT)?;
        self.pending_jpeg = Some(jpeg);
        info!(
            bytes = self.pending_jpeg.as_ref().map_or(0, Vec::len),
            "JPEG reçu"
        );
        Ok(())
    }

    fn download_jpeg(&mut self) -> Result<Vec<u8>, CameraError> {
        self.ensure_connected()?;
        self.pending_jpeg.take().ok_or_else(|| {
            CameraError::Message("aucun JPEG en attente (transfert non reçu)".to_string())
        })
    }

    fn start_live_view(&mut self) -> Result<(), CameraError> {
        self.ensure_connected()?;
        if self.live_view {
            return Ok(());
        }
        let mut ret_error = 0u32;
        let quality_ok = unsafe { lmx_sys_liveview_set_recom_quality_low(&mut ret_error) };
        if quality_ok != LMX_BOOL_TRUE {
            warn!(
                ret_error,
                "LiveView_Str_Set_Trans_Img (qualité basse) a échoué — on tente Start quand même"
            );
        }
        check_bool(
            unsafe { lmx_sys_ctrl_liveview_start(&mut ret_error) },
            "Ctrl_LiveView_Start",
            ret_error,
        )?;
        self.live_view = true;
        // Laisser le flux démarrer (sample : premiers Get peuvent être BUSY/RCV).
        std::thread::sleep(Duration::from_millis(200));
        info!("Live View démarré");
        Ok(())
    }

    fn stop_live_view(&mut self) -> Result<(), CameraError> {
        if !self.live_view {
            return Ok(());
        }
        let mut ret_error = 0u32;
        // Sample : Stop avant d'arrêter le thread poll.
        let ok = unsafe { lmx_sys_ctrl_liveview_stop(&mut ret_error) };
        self.live_view = false;
        if ok != LMX_BOOL_TRUE {
            warn!(ret_error, "Ctrl_LiveView_Stop a échoué");
        }
        // Laisser le boîtier sortir du mode LV avant un still (sinon ONESHOT sans REQ_TRNSFER).
        std::thread::sleep(Duration::from_millis(400));
        info!("Live View arrêté");
        Ok(())
    }

    fn poll_live_view_jpeg(&mut self) -> Result<Option<Vec<u8>>, CameraError> {
        self.ensure_connected()?;
        if !self.live_view {
            return Ok(None);
        }
        let mut ret_error = 0u32;
        let mut jpeg_size = 0u32;
        let ok = unsafe {
            lmx_sys_get_liveview_jpeg(
                self.live_view_buf.as_mut_ptr(),
                self.live_view_buf.len() as u32,
                &mut jpeg_size,
                &mut ret_error,
            )
        };
        if ok != LMX_BOOL_TRUE {
            // BUSY / RCV : pas de frame prête — ne pas traiter comme erreur fatale.
            if ret_error == LMX_DEF_ERR_COM_DATA_BUSY || ret_error == LMX_DEF_ERR_COM_DATA_RCV {
                return Ok(None);
            }
            return Err(sdk_err("Get_LiveView_data", ret_error));
        }
        if jpeg_size == 0 {
            return Ok(None);
        }
        Ok(Some(self.live_view_buf[..jpeg_size as usize].to_vec()))
    }

    fn is_live_view_running(&self) -> bool {
        self.live_view
    }
}
