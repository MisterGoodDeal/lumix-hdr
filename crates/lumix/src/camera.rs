use crate::error::CameraError;
use crate::types::{CameraSettings, ShutterLimits};

/// Abstraction appareil. Phase 3 : seule `MockCamera` l'implémente.
pub trait Camera: Send {
    fn connect(&mut self) -> Result<(), CameraError>;
    fn disconnect(&mut self) -> Result<(), CameraError>;
    fn is_connected(&self) -> bool;

    fn shutter_limits(&self) -> ShutterLimits;
    fn read_settings(&mut self) -> Result<CameraSettings, CameraError>;
    fn apply_settings(&mut self, settings: &CameraSettings) -> Result<(), CameraError>;

    fn set_shutter(&mut self, shutter_seconds: f64) -> Result<(), CameraError>;
    fn verify_shutter(&mut self, expected_seconds: f64) -> Result<(), CameraError>;

    fn trigger(&mut self) -> Result<(), CameraError>;
    fn wait_capture_done(&mut self) -> Result<(), CameraError>;
    fn download_jpeg(&mut self) -> Result<Vec<u8>, CameraError>;
}
