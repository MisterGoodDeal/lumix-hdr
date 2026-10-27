use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use opencv::core::{self, Mat, Vector, CV_16UC3, CV_8UC3};
use opencv::photo::{
    self, AlignMTBTrait, CalibrateCRFTrait, MergeDebevecTrait, MergeMertensTrait, TonemapTrait,
};
use opencv::prelude::*;
use serde::Serialize;

use crate::error::HdrError;
use crate::exif::copy_exif_jpeg;
use crate::io::{unique_path, write_jpeg_atomic, write_tiff_atomic};
use crate::options::{FusionMethod, FusionOptions, TonemapKind};
use crate::source::{jpeg_files_in_dir, DecodedImage, ImageSource, JpegFile};

pub type ProgressFn<'a> = dyn FnMut(f32) + Send + 'a;

#[derive(Debug, Clone, Serialize)]
pub struct FusionOutputs {
    pub jpeg: Option<PathBuf>,
    pub tiff: Option<PathBuf>,
    pub image_count: usize,
    pub method: FusionMethod,
    pub aligned: bool,
}

pub fn fuse_folder(
    dir: &std::path::Path,
    options: &FusionOptions,
    cancel: &AtomicBool,
    progress: &mut ProgressFn<'_>,
) -> Result<FusionOutputs, HdrError> {
    progress(0.05);
    check_cancel(cancel)?;
    let files = jpeg_files_in_dir(dir)?;
    if files.is_empty() {
        return Err(HdrError::NoImages);
    }
    if files.len() < 2 {
        return Err(HdrError::TooFewImages(files.len()));
    }
    let sources: Vec<JpegFile> = files.into_iter().map(JpegFile::new).collect();
    fuse_sources(&sources, dir, options, cancel, progress)
}

pub fn fuse_sources<S: ImageSource>(
    sources: &[S],
    output_dir: &std::path::Path,
    options: &FusionOptions,
    cancel: &AtomicBool,
    progress: &mut ProgressFn<'_>,
) -> Result<FusionOutputs, HdrError> {
    if sources.len() < 2 {
        return Err(HdrError::TooFewImages(sources.len()));
    }
    if !options.output_jpeg && !options.output_tiff {
        return Err(HdrError::Inconsistent(
            "choisissez au moins un format de sortie (JPEG ou TIFF)".to_string(),
        ));
    }
    progress(0.1);
    check_cancel(cancel)?;

    let mut decoded = Vec::with_capacity(sources.len());
    for source in sources {
        check_cancel(cancel)?;
        decoded.push(source.decode()?);
    }
    validate_geometry(&decoded)?;
    progress(0.25);

    let mut mats = Vector::<Mat>::new();
    for image in &decoded {
        mats.push(image.mat.clone());
    }
    if options.align {
        check_cancel(cancel)?;
        mats = align_mtb(&mats)?;
    }
    progress(0.45);

    check_cancel(cancel)?;
    let fused_float = match options.method {
        FusionMethod::Mertens => merge_mertens(&mats, options)?,
        FusionMethod::Debevec => merge_debevec(&mats, &decoded, options)?,
    };
    progress(0.75);

    let mut jpeg8 = Mat::default();
    fused_float.convert_to(&mut jpeg8, CV_8UC3, 255.0, 0.0)?;
    let mut tiff16 = Mat::default();
    fused_float.convert_to(&mut tiff16, CV_16UC3, 65535.0, 0.0)?;

    let reference = decoded
        .iter()
        .find(|image| image.path.is_some())
        .and_then(|image| image.path.as_deref());

    check_cancel(cancel)?;
    let jpeg_path = if options.output_jpeg {
        let path = unique_path(output_dir, "hdr-fused", "jpg")?;
        write_jpeg_atomic(&path, &jpeg8, options.jpeg_quality)?;
        if let Some(reference) = reference {
            match std::fs::read(&path) {
                Ok(bytes) => match copy_exif_jpeg(reference, &bytes) {
                    Ok(with_exif) => {
                        if let Err(error) = std::fs::write(&path, with_exif) {
                            tracing::warn!(
                                path = %path.display(),
                                %error,
                                "impossible de recopier l'EXIF"
                            );
                        }
                    }
                    Err(error) => tracing::warn!(%error, "EXIF non recopié"),
                },
                Err(error) => tracing::warn!(%error, "relecture JPEG impossible pour l'EXIF"),
            }
        }
        Some(path)
    } else {
        None
    };

    check_cancel(cancel)?;
    let tiff_path = if options.output_tiff {
        let path = unique_path(output_dir, "hdr-fused", "tiff")?;
        write_tiff_atomic(&path, &tiff16)?;
        Some(path)
    } else {
        None
    };

    progress(1.0);
    Ok(FusionOutputs {
        jpeg: jpeg_path,
        tiff: tiff_path,
        image_count: sources.len(),
        method: options.method,
        aligned: options.align,
    })
}

pub fn clipped_pixel_count(image: &Mat) -> Result<u64, HdrError> {
    if image.typ() != CV_8UC3 {
        return Err(HdrError::Inconsistent(
            "comptage d'écrêtage: image 8 bits BGR requise".to_string(),
        ));
    }
    let bytes = image.data_bytes()?;
    let mut count = 0_u64;
    for pixel in bytes.as_chunks::<3>().0 {
        let black = pixel.iter().all(|value| *value == 0);
        let white = pixel.iter().all(|value| *value == 255);
        if black || white {
            count += 1;
        }
    }
    Ok(count)
}

fn validate_geometry(images: &[DecodedImage]) -> Result<(), HdrError> {
    let Some(first) = images.first() else {
        return Err(HdrError::TooFewImages(0));
    };
    let rows = first.mat.rows();
    let cols = first.mat.cols();
    let channels = first.mat.channels();
    for (index, image) in images.iter().enumerate().skip(1) {
        if image.mat.rows() != rows || image.mat.cols() != cols || image.mat.channels() != channels
        {
            return Err(HdrError::Inconsistent(format!(
                "image {index}: {}x{}x{} (attendu {cols}x{rows}x{channels})",
                image.mat.cols(),
                image.mat.rows(),
                image.mat.channels()
            )));
        }
    }
    Ok(())
}

fn align_mtb(images: &Vector<Mat>) -> Result<Vector<Mat>, HdrError> {
    let mut align = photo::create_align_mtb_def()?;
    let mut aligned = Vector::<Mat>::new();
    AlignMTBTrait::process(&mut align, images, &mut aligned)?;
    if aligned.len() != images.len() {
        return Err(HdrError::Inconsistent(
            "l'alignement n'a pas renvoyé toutes les images".to_string(),
        ));
    }
    Ok(aligned)
}

fn merge_mertens(images: &Vector<Mat>, options: &FusionOptions) -> Result<Mat, HdrError> {
    let mut merge = photo::create_merge_mertens(
        options.mertens.contrast,
        options.mertens.saturation,
        options.mertens.exposure,
    )?;
    let mut fused = Mat::default();
    MergeMertensTrait::process(&mut merge, images, &mut fused)?;
    Ok(fused)
}

fn merge_debevec(
    images: &Vector<Mat>,
    decoded: &[DecodedImage],
    options: &FusionOptions,
) -> Result<Mat, HdrError> {
    let mut times = Vector::<f32>::new();
    for image in decoded {
        times.push(image.exposure_seconds.unwrap_or(1.0 / 30.0));
    }
    let mut calibrate = photo::create_calibrate_debevec_def()?;
    let mut response = Mat::default();
    CalibrateCRFTrait::process(&mut calibrate, images, &mut response, &times)?;
    let mut merge = photo::create_merge_debevec()?;
    let mut hdr = Mat::default();
    MergeDebevecTrait::process_with_response(&mut merge, images, &mut hdr, &times, &response)?;
    let mut ldr = Mat::default();
    match options.tonemap {
        TonemapKind::Reinhard => {
            let mut tonemap = photo::create_tonemap_reinhard_def()?;
            TonemapTrait::process(&mut tonemap, &hdr, &mut ldr)?;
        }
        TonemapKind::Mantiuk => {
            let mut tonemap = photo::create_tonemap_mantiuk_def()?;
            TonemapTrait::process(&mut tonemap, &hdr, &mut ldr)?;
        }
    }
    if ldr.channels() == 3 {
        Ok(ldr)
    } else {
        let mut bgr = Mat::default();
        opencv::imgproc::cvt_color_def(&ldr, &mut bgr, opencv::imgproc::COLOR_GRAY2BGR)?;
        Ok(bgr)
    }
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), HdrError> {
    if cancel.load(Ordering::Relaxed) {
        Err(HdrError::Cancelled)
    } else {
        Ok(())
    }
}

pub fn mean_abs_diff(a: &Mat, b: &Mat) -> Result<f64, HdrError> {
    let mut diff = Mat::default();
    core::absdiff(a, b, &mut diff)?;
    let mean = core::mean_def(&diff)?;
    Ok((mean[0] + mean[1] + mean[2]) / 3.0)
}
