use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FusionMethod {
    #[default]
    Mertens,
    Debevec,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TonemapKind {
    #[default]
    Reinhard,
    Mantiuk,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MertensWeights {
    pub contrast: f32,
    pub saturation: f32,
    pub exposure: f32,
}

impl Default for MertensWeights {
    fn default() -> Self {
        Self {
            contrast: 1.0,
            saturation: 1.0,
            exposure: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionOptions {
    #[serde(default = "default_true")]
    pub align: bool,
    #[serde(default)]
    pub method: FusionMethod,
    #[serde(default)]
    pub mertens: MertensWeights,
    #[serde(default)]
    pub tonemap: TonemapKind,
    #[serde(default = "default_true")]
    pub output_jpeg: bool,
    #[serde(default)]
    pub output_tiff: bool,
    #[serde(default = "default_jpeg_quality")]
    pub jpeg_quality: i32,
}

impl Default for FusionOptions {
    fn default() -> Self {
        Self {
            align: true,
            method: FusionMethod::Mertens,
            mertens: MertensWeights::default(),
            tonemap: TonemapKind::Reinhard,
            output_jpeg: true,
            output_tiff: false,
            jpeg_quality: 95,
        }
    }
}

const fn default_true() -> bool {
    true
}

const fn default_jpeg_quality() -> i32 {
    95
}
