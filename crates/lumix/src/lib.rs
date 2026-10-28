#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod camera;
mod error;
mod mock;
mod types;

pub use camera::Camera;
pub use error::CameraError;
pub use mock::{MockCamera, MockFailAt};
pub use types::{
    format_shutter, nearest_shutter, standard_shutter_table, CameraSettings, ExposureMode,
    ShutterLimits,
};
