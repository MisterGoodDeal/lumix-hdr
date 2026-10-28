use crate::error::CameraError;
use crate::types::{CameraSettings, ExposureCapabilities, ShutterLimits};

/// Abstraction appareil (Mock ou SDK LUMIX).
pub trait Camera: Send {
    fn connect(&mut self) -> Result<(), CameraError>;
    fn disconnect(&mut self) -> Result<(), CameraError>;
    fn is_connected(&self) -> bool;

    fn shutter_limits(&self) -> ShutterLimits;
    fn read_settings(&mut self) -> Result<CameraSettings, CameraError>;
    fn apply_settings(&mut self, settings: &CameraSettings) -> Result<(), CameraError>;

    /// Listes de pas pour les overlays +/- (capa appareil ou table standard).
    fn exposure_capabilities(&mut self) -> Result<ExposureCapabilities, CameraError>;

    fn set_shutter(&mut self, shutter_seconds: f64) -> Result<(), CameraError>;
    fn verify_shutter(&mut self, expected_seconds: f64) -> Result<(), CameraError>;

    fn trigger(&mut self) -> Result<(), CameraError>;
    fn wait_capture_done(&mut self) -> Result<(), CameraError>;
    fn download_jpeg(&mut self) -> Result<Vec<u8>, CameraError>;

    /// Démarre le flux Live View (no-op si déjà actif).
    fn start_live_view(&mut self) -> Result<(), CameraError>;
    /// Arrête le flux Live View.
    fn stop_live_view(&mut self) -> Result<(), CameraError>;
    /// Une frame JPEG Live View (vide si BUSY / pas de frame).
    fn poll_live_view_jpeg(&mut self) -> Result<Option<Vec<u8>>, CameraError>;
    fn is_live_view_running(&self) -> bool;

    /// Sonde légère (sans Get_PnP) pour détecter un débranchement.
    fn heartbeat(&mut self) -> Result<(), CameraError> {
        let _ = self.read_settings()?;
        Ok(())
    }
}
