use std::io::Cursor;

use image::{ImageBuffer, ImageFormat, Rgb};
use tracing::{debug, info};

use crate::camera::Camera;
use crate::error::CameraError;
use crate::types::{format_shutter, nearest_shutter, CameraSettings, ExposureMode, ShutterLimits};

/// Point d'injection d'erreur pour les tests / démo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockFailAt {
    Connect,
    ReadSettings,
    SetExposure,
    VerifyExposure,
    Trigger,
    WaitCaptureDone,
    Download,
    /// Déconnexion après N déclenchements réussis (0 = au prochain trigger).
    DisconnectAfterTriggers(u32),
}

#[derive(Debug, Clone)]
pub struct MockCamera {
    connected: bool,
    settings: CameraSettings,
    limits: ShutterLimits,
    /// EV relatif courant (par rapport à la vitesse de base au connect).
    current_ev: f64,
    base_shutter: f64,
    fail_at: Option<MockFailAt>,
    triggers_done: u32,
    pending_download: Option<Vec<u8>>,
    busy: bool,
}

impl Default for MockCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCamera {
    pub fn new() -> Self {
        let settings = CameraSettings::default();
        Self {
            connected: false,
            base_shutter: settings.shutter_seconds,
            current_ev: 0.0,
            settings,
            limits: ShutterLimits::default(),
            fail_at: None,
            triggers_done: 0,
            pending_download: None,
            busy: false,
        }
    }

    pub fn with_settings(settings: CameraSettings) -> Self {
        let mut camera = Self::new();
        camera.base_shutter = settings.shutter_seconds;
        camera.settings = settings;
        camera
    }

    pub fn inject_failure(&mut self, fail_at: MockFailAt) {
        self.fail_at = Some(fail_at);
    }

    pub fn clear_failure(&mut self) {
        self.fail_at = None;
    }

    fn ensure_connected(&self) -> Result<(), CameraError> {
        if !self.connected {
            return Err(CameraError::NotConnected);
        }
        Ok(())
    }

    fn check_fail(&mut self, point: MockFailAt) -> Result<(), CameraError> {
        match self.fail_at {
            Some(MockFailAt::DisconnectAfterTriggers(n))
                if matches!(point, MockFailAt::Trigger) =>
            {
                if self.triggers_done >= n {
                    self.connected = false;
                    self.fail_at = None;
                    return Err(CameraError::Disconnected);
                }
            }
            Some(fail) if fail == point => {
                self.fail_at = None;
                return Err(match point {
                    MockFailAt::Connect => CameraError::Timeout,
                    MockFailAt::WaitCaptureDone => CameraError::Timeout,
                    MockFailAt::Trigger if self.busy => CameraError::Busy,
                    MockFailAt::Download => CameraError::Busy,
                    MockFailAt::DisconnectAfterTriggers(_) => CameraError::Disconnected,
                    _ => CameraError::Message(format!("échec simulé à {point:?}")),
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn synthesize_jpeg(&self) -> Result<Vec<u8>, CameraError> {
        let width = 640u32;
        let height = 480u32;
        let mid = 96.0_f64;
        let scale = 2.0_f64.powf(self.current_ev);
        let base = (mid * scale).clamp(8.0, 240.0);

        let mut buffer = ImageBuffer::new(width, height);
        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            let xn = f64::from(x) / f64::from(width - 1);
            let yn = f64::from(y) / f64::from(height - 1);
            // Scène synthétique : dégradé + « fenêtre » claire + ombre.
            let window = if (0.55..=0.85).contains(&xn) && (0.15..=0.55).contains(&yn) {
                70.0
            } else {
                0.0
            };
            let shadow = if xn < 0.25 { -40.0 } else { 0.0 };
            let v = (base + window + shadow + (xn - yn) * 12.0).clamp(0.0, 255.0) as u8;
            *pixel = Rgb([
                v,
                v.saturating_add(4.min(255 - v)),
                v.saturating_sub(4.min(v)),
            ]);
        }

        let mut bytes = Cursor::new(Vec::new());
        buffer
            .write_to(&mut bytes, ImageFormat::Jpeg)
            .map_err(|error| CameraError::Message(format!("encodage JPEG mock: {error}")))?;
        Ok(bytes.into_inner())
    }
}

impl Camera for MockCamera {
    fn connect(&mut self) -> Result<(), CameraError> {
        self.check_fail(MockFailAt::Connect)?;
        self.connected = true;
        self.triggers_done = 0;
        self.pending_download = None;
        self.busy = false;
        self.base_shutter = self.settings.shutter_seconds;
        self.current_ev = 0.0;
        info!("MockCamera connectée");
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), CameraError> {
        self.connected = false;
        self.pending_download = None;
        self.busy = false;
        info!("MockCamera déconnectée");
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
        self.check_fail(MockFailAt::ReadSettings)?;
        Ok(self.settings.clone())
    }

    fn apply_settings(&mut self, settings: &CameraSettings) -> Result<(), CameraError> {
        self.ensure_connected()?;
        if settings.mode != ExposureMode::M {
            return Err(CameraError::Message(
                "seul le mode M est supporté en phase 3".to_string(),
            ));
        }
        if settings.shutter_seconds < self.limits.min_seconds
            || settings.shutter_seconds > self.limits.max_seconds
        {
            return Err(CameraError::OutOfRange(format!(
                "vitesse {} hors plage",
                settings.shutter_label
            )));
        }
        self.settings = settings.clone();
        let (seconds, label) = nearest_shutter(settings.shutter_seconds);
        self.settings.shutter_seconds = seconds;
        self.settings.shutter_label = label;
        self.current_ev = (seconds / self.base_shutter).log2();
        debug!(shutter = %self.settings.shutter_label, ev = self.current_ev, "réglages appliqués");
        Ok(())
    }

    fn set_shutter(&mut self, shutter_seconds: f64) -> Result<(), CameraError> {
        self.ensure_connected()?;
        self.check_fail(MockFailAt::SetExposure)?;
        if shutter_seconds < self.limits.min_seconds || shutter_seconds > self.limits.max_seconds {
            return Err(CameraError::OutOfRange(format_shutter(shutter_seconds)));
        }
        let (seconds, label) = nearest_shutter(shutter_seconds);
        self.settings.shutter_seconds = seconds;
        self.settings.shutter_label = label;
        self.current_ev = (seconds / self.base_shutter).log2();
        Ok(())
    }

    fn verify_shutter(&mut self, expected_seconds: f64) -> Result<(), CameraError> {
        self.ensure_connected()?;
        self.check_fail(MockFailAt::VerifyExposure)?;
        let (expected, _) = nearest_shutter(expected_seconds);
        let actual = self.settings.shutter_seconds;
        let ratio = (actual / expected).max(expected / actual);
        if ratio > 1.08 {
            return Err(CameraError::VerifyFailed(format!(
                "attendu {}, obtenu {}",
                format_shutter(expected),
                self.settings.shutter_label
            )));
        }
        Ok(())
    }

    fn trigger(&mut self) -> Result<(), CameraError> {
        self.ensure_connected()?;
        if self.busy {
            return Err(CameraError::Busy);
        }
        self.check_fail(MockFailAt::Trigger)?;
        self.busy = true;
        self.pending_download = Some(self.synthesize_jpeg()?);
        self.triggers_done = self.triggers_done.saturating_add(1);
        Ok(())
    }

    fn wait_capture_done(&mut self) -> Result<(), CameraError> {
        self.ensure_connected()?;
        self.check_fail(MockFailAt::WaitCaptureDone)?;
        if !self.busy {
            return Err(CameraError::Message("aucune capture en cours".to_string()));
        }
        self.busy = false;
        Ok(())
    }

    fn download_jpeg(&mut self) -> Result<Vec<u8>, CameraError> {
        self.ensure_connected()?;
        self.check_fail(MockFailAt::Download)?;
        self.pending_download
            .take()
            .ok_or_else(|| CameraError::Message("aucun fichier à télécharger".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_follows_exposure() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        camera.set_shutter(1.0 / 60.0).expect("base");
        camera.trigger().expect("trig");
        camera.wait_capture_done().expect("wait");
        let mid = camera.download_jpeg().expect("dl");

        camera.set_shutter(1.0 / 15.0).expect("+2IL approx");
        camera.trigger().expect("trig");
        camera.wait_capture_done().expect("wait");
        let bright = camera.download_jpeg().expect("dl");

        assert!(bright.len() > 100);
        assert!(mid.len() > 100);
        // JPEG plus exposé : octets souvent un peu plus lourds / différent.
        assert_ne!(mid, bright);
    }

    #[test]
    fn inject_disconnect() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        camera.inject_failure(MockFailAt::DisconnectAfterTriggers(0));
        let err = camera.trigger().expect_err("doit échouer");
        assert_eq!(err, CameraError::Disconnected);
        assert!(!camera.is_connected());
    }
}
