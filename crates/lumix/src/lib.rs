#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod camera;
mod error;
mod mock;
mod types;

#[cfg(all(windows, feature = "sdk"))]
mod sdk;

pub use camera::Camera;
pub use error::CameraError;
pub use mock::{MockCamera, MockFailAt};
pub use types::{
    format_shutter, nearest_shutter, standard_shutter_table, CameraSettings, ExposureCapabilities,
    ExposureMode, ShutterLimits,
};

#[cfg(all(windows, feature = "sdk"))]
pub use sdk::{
    begin_capture_wait, request_capture_cancel, suppress_settings_notify_briefly,
    take_settings_changed, LumixCamera,
};

#[cfg(not(all(windows, feature = "sdk")))]
pub fn take_settings_changed() -> bool {
    false
}

#[cfg(not(all(windows, feature = "sdk")))]
pub fn suppress_settings_notify_briefly() {}

#[cfg(not(all(windows, feature = "sdk")))]
pub fn begin_capture_wait() {}

#[cfg(not(all(windows, feature = "sdk")))]
pub fn request_capture_cancel() {}
