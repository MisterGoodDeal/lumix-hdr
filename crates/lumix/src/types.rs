use serde::{Deserialize, Serialize};

/// Mode d'exposition. Phase 3 : mode M uniquement pour le bracketing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExposureMode {
    #[default]
    M,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraSettings {
    pub mode: ExposureMode,
    /// Ouverture (ex. 2.8 pour f/2.8).
    pub aperture: f32,
    pub iso: u32,
    /// Vitesse d'obturation en secondes (ex. 1.0/60.0).
    pub shutter_seconds: f64,
    pub shutter_label: String,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            mode: ExposureMode::M,
            aperture: 2.8,
            iso: 200,
            shutter_seconds: 1.0 / 60.0,
            shutter_label: "1/60".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShutterLimits {
    pub min_seconds: f64,
    pub max_seconds: f64,
}

impl Default for ShutterLimits {
    fn default() -> Self {
        Self {
            min_seconds: 1.0 / 8000.0,
            max_seconds: 30.0,
        }
    }
}

/// Vitesses standard ~1/3 IL, du plus rapide au plus lent.
pub fn standard_shutter_table() -> &'static [(f64, &'static str)] {
    &[
        (1.0 / 8000.0, "1/8000"),
        (1.0 / 6400.0, "1/6400"),
        (1.0 / 5000.0, "1/5000"),
        (1.0 / 4000.0, "1/4000"),
        (1.0 / 3200.0, "1/3200"),
        (1.0 / 2500.0, "1/2500"),
        (1.0 / 2000.0, "1/2000"),
        (1.0 / 1600.0, "1/1600"),
        (1.0 / 1250.0, "1/1250"),
        (1.0 / 1000.0, "1/1000"),
        (1.0 / 800.0, "1/800"),
        (1.0 / 640.0, "1/640"),
        (1.0 / 500.0, "1/500"),
        (1.0 / 400.0, "1/400"),
        (1.0 / 320.0, "1/320"),
        (1.0 / 250.0, "1/250"),
        (1.0 / 200.0, "1/200"),
        (1.0 / 160.0, "1/160"),
        (1.0 / 125.0, "1/125"),
        (1.0 / 100.0, "1/100"),
        (1.0 / 80.0, "1/80"),
        (1.0 / 60.0, "1/60"),
        (1.0 / 50.0, "1/50"),
        (1.0 / 40.0, "1/40"),
        (1.0 / 30.0, "1/30"),
        (1.0 / 25.0, "1/25"),
        (1.0 / 20.0, "1/20"),
        (1.0 / 15.0, "1/15"),
        (1.0 / 13.0, "1/13"),
        (1.0 / 10.0, "1/10"),
        (1.0 / 8.0, "1/8"),
        (1.0 / 6.0, "1/6"),
        (1.0 / 5.0, "1/5"),
        (1.0 / 4.0, "1/4"),
        (0.3, "0\"3"),
        (0.4, "0\"4"),
        (0.5, "0\"5"),
        (0.6, "0\"6"),
        (0.8, "0\"8"),
        (1.0, "1\""),
        (1.3, "1\"3"),
        (1.6, "1\"6"),
        (2.0, "2\""),
        (2.5, "2\"5"),
        (3.2, "3\"2"),
        (4.0, "4\""),
        (5.0, "5\""),
        (6.0, "6\""),
        (8.0, "8\""),
        (10.0, "10\""),
        (13.0, "13\""),
        (15.0, "15\""),
        (20.0, "20\""),
        (25.0, "25\""),
        (30.0, "30\""),
    ]
}

pub fn nearest_shutter(seconds: f64) -> (f64, String) {
    let table = standard_shutter_table();
    let mut best = table[0];
    let mut best_err = (seconds - best.0).abs();
    for entry in table.iter().skip(1) {
        let err = (seconds - entry.0).abs();
        if err < best_err {
            best = *entry;
            best_err = err;
        }
    }
    (best.0, best.1.to_string())
}

pub fn format_shutter(seconds: f64) -> String {
    nearest_shutter(seconds).1
}
