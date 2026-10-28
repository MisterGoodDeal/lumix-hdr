use serde::{Deserialize, Serialize};

use lumix::{format_shutter, nearest_shutter, CameraSettings, ShutterLimits};

use crate::error::CaptureError;

pub const ALLOWED_FRAME_COUNTS: [u8; 4] = [3, 5, 7, 9];
pub const MIN_STEP_EV: f32 = 1.0 / 3.0;
pub const MAX_STEP_EV: f32 = 3.0;
pub const MIN_CENTER_EV: f32 = -3.0;
pub const MAX_CENTER_EV: f32 = 3.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BracketRequest {
    pub frame_count: u8,
    pub step_ev: f32,
    pub center_ev: f32,
    pub session_name: String,
}

impl Default for BracketRequest {
    fn default() -> Self {
        Self {
            frame_count: 5,
            step_ev: 1.0,
            center_ev: 0.0,
            session_name: "hdr".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposureSlot {
    pub index: usize,
    pub relative_ev: f32,
    pub shutter_seconds: f64,
    pub shutter_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BracketPlan {
    pub request: BracketRequest,
    pub base: CameraSettings,
    pub slots: Vec<ExposureSlot>,
}

pub fn build_plan(
    request: &BracketRequest,
    base: &CameraSettings,
    limits: ShutterLimits,
) -> Result<BracketPlan, CaptureError> {
    validate_request(request)?;

    let half = (i32::from(request.frame_count) - 1) / 2;
    let mut slots = Vec::with_capacity(usize::from(request.frame_count));

    for i in 0..i32::from(request.frame_count) {
        let offset = i - half;
        let relative_ev = request.center_ev + offset as f32 * request.step_ev;
        let raw_seconds = base.shutter_seconds * 2.0_f64.powf(f64::from(relative_ev));
        if raw_seconds < limits.min_seconds || raw_seconds > limits.max_seconds {
            return Err(CaptureError::InvalidPlan(format!(
                "vitesse {} (EV {relative_ev:+.2}) hors plage appareil ({} … {})",
                format_shutter(raw_seconds),
                format_shutter(limits.min_seconds),
                format_shutter(limits.max_seconds)
            )));
        }
        let (seconds, label) = nearest_shutter(raw_seconds);

        slots.push(ExposureSlot {
            index: i as usize,
            relative_ev,
            shutter_seconds: seconds,
            shutter_label: label,
        });
    }

    Ok(BracketPlan {
        request: request.clone(),
        base: base.clone(),
        slots,
    })
}

fn validate_request(request: &BracketRequest) -> Result<(), CaptureError> {
    if !ALLOWED_FRAME_COUNTS.contains(&request.frame_count) {
        return Err(CaptureError::InvalidPlan(format!(
            "nombre de vues {} non supporté (3, 5, 7 ou 9)",
            request.frame_count
        )));
    }
    if !(MIN_STEP_EV..=MAX_STEP_EV).contains(&request.step_ev) {
        return Err(CaptureError::InvalidPlan(format!(
            "écart IL {} hors plage ({MIN_STEP_EV} … {MAX_STEP_EV})",
            request.step_ev
        )));
    }
    if !(MIN_CENTER_EV..=MAX_CENTER_EV).contains(&request.center_ev) {
        return Err(CaptureError::InvalidPlan(format!(
            "centre IL {} hors plage ({MIN_CENTER_EV} … {MAX_CENTER_EV})",
            request.center_ev
        )));
    }
    let name = request.session_name.trim();
    if name.is_empty() {
        return Err(CaptureError::InvalidPlan("nom de session vide".to_string()));
    }
    if name
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
    {
        return Err(CaptureError::InvalidPlan(
            "nom de session: lettres, chiffres, - et _ uniquement".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumix::CameraSettings;

    #[test]
    fn five_frames_one_ev() {
        let base = CameraSettings::default();
        let plan = build_plan(
            &BracketRequest {
                frame_count: 5,
                step_ev: 1.0,
                center_ev: 0.0,
                session_name: "test".into(),
            },
            &base,
            ShutterLimits::default(),
        )
        .expect("plan");
        assert_eq!(plan.slots.len(), 5);
        let evs: Vec<f32> = plan.slots.iter().map(|s| s.relative_ev).collect();
        assert_eq!(evs, vec![-2.0, -1.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn rejects_out_of_range_shutter() {
        let base = CameraSettings {
            shutter_seconds: 8.0,
            shutter_label: "8\"".into(),
            ..CameraSettings::default()
        };
        let err = build_plan(
            &BracketRequest {
                frame_count: 9,
                step_ev: 1.0,
                center_ev: 2.0,
                session_name: "wide".into(),
            },
            &base,
            ShutterLimits::default(),
        )
        .expect_err("hors plage");
        assert!(matches!(err, CaptureError::InvalidPlan(_)));
    }

    #[test]
    fn rejects_bad_frame_count() {
        let err = build_plan(
            &BracketRequest {
                frame_count: 4,
                step_ev: 1.0,
                center_ev: 0.0,
                session_name: "x".into(),
            },
            &CameraSettings::default(),
            ShutterLimits::default(),
        )
        .expect_err("4 vues");
        assert!(matches!(err, CaptureError::InvalidPlan(_)));
    }
}
