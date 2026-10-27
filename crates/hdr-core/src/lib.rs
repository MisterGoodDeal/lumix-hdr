#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod error;
mod exif;
mod fuse;
mod io;
mod options;
mod source;

pub use error::HdrError;
pub use fuse::{clipped_pixel_count, fuse_folder, fuse_sources, mean_abs_diff, FusionOutputs};
pub use options::{FusionMethod, FusionOptions, MertensWeights, TonemapKind};
pub use source::{jpeg_files_in_dir, DecodedImage, ImageSource, JpegFile};

#[cfg(test)]
mod tests;
