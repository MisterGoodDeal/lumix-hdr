use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use opencv::core::Mat;
use opencv::imgcodecs;
use opencv::prelude::*;

use crate::error::HdrError;

pub struct DecodedImage {
    pub mat: Mat,
    pub path: Option<PathBuf>,
    pub exposure_seconds: Option<f32>,
}

pub trait ImageSource {
    fn decode(&self) -> Result<DecodedImage, HdrError>;
}

pub struct JpegFile {
    path: PathBuf,
}

impl JpegFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ImageSource for JpegFile {
    fn decode(&self) -> Result<DecodedImage, HdrError> {
        let mat = imgcodecs::imread(&self.path, imgcodecs::IMREAD_COLOR)?;
        if mat.empty() {
            return Err(HdrError::Inconsistent(format!(
                "fichier illisible: {}",
                self.path.display()
            )));
        }
        if mat.channels() != 3 {
            return Err(HdrError::Inconsistent(format!(
                "{}: {} canaux (attendu: 3)",
                self.path.display(),
                mat.channels()
            )));
        }
        Ok(DecodedImage {
            exposure_seconds: read_exposure_seconds(&self.path),
            path: Some(self.path.clone()),
            mat,
        })
    }
}

fn read_exposure_seconds(path: &Path) -> Option<f32> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)?;
    match field.value {
        exif::Value::Rational(ref values) if !values.is_empty() => {
            let rational = values[0];
            if rational.denom == 0 {
                None
            } else {
                Some(rational.num as f32 / rational.denom as f32)
            }
        }
        _ => None,
    }
}

pub fn jpeg_files_in_dir(dir: &Path) -> Result<Vec<PathBuf>, HdrError> {
    let entries = std::fs::read_dir(dir).map_err(|source| HdrError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| HdrError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_jpeg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg"));
        if is_jpeg {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
