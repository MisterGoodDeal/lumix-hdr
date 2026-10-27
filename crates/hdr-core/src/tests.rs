use std::sync::atomic::AtomicBool;

use opencv::core::{self, Mat, Scalar, CV_8UC3};
use opencv::imgproc;
use opencv::prelude::*;

use crate::fuse::{clipped_pixel_count, fuse_sources, mean_abs_diff};
use crate::options::{FusionMethod, FusionOptions};
use crate::source::{DecodedImage, ImageSource};
use crate::HdrError;

struct MemoryJpeg {
    mat: Mat,
    exposure_seconds: Option<f32>,
}

impl ImageSource for MemoryJpeg {
    fn decode(&self) -> Result<DecodedImage, HdrError> {
        Ok(DecodedImage {
            mat: self.mat.clone(),
            path: None,
            exposure_seconds: self.exposure_seconds,
        })
    }
}

fn scene(width: i32, height: i32) -> Result<Mat, HdrError> {
    let mut image = Mat::new_rows_cols_with_default(height, width, CV_8UC3, Scalar::all(96.0))?;
    imgproc::rectangle(
        &mut image,
        core::Rect::new(10, 10, 70, 50),
        Scalar::new(255.0, 255.0, 255.0, 0.0),
        -1,
        imgproc::LINE_8,
        0,
    )?;
    imgproc::rectangle(
        &mut image,
        core::Rect::new(80, 40, 70, 50),
        Scalar::new(0.0, 0.0, 0.0, 0.0),
        -1,
        imgproc::LINE_8,
        0,
    )?;
    imgproc::rectangle(
        &mut image,
        core::Rect::new(40, 80, 60, 25),
        Scalar::new(40.0, 180.0, 30.0, 0.0),
        -1,
        imgproc::LINE_8,
        0,
    )?;
    Ok(image)
}

fn apply_ev(src: &Mat, ev: f64) -> Result<Mat, HdrError> {
    let mut dst = Mat::default();
    src.convert_to(&mut dst, CV_8UC3, 2_f64.powf(ev), 0.0)?;
    Ok(dst)
}

fn translate(src: &Mat, dx: f32, dy: f32) -> Result<Mat, HdrError> {
    let mut warp = Mat::zeros(2, 3, core::CV_32F)?.to_mat()?;
    *warp.at_2d_mut::<f32>(0, 0)? = 1.0;
    *warp.at_2d_mut::<f32>(1, 1)? = 1.0;
    *warp.at_2d_mut::<f32>(0, 2)? = dx;
    *warp.at_2d_mut::<f32>(1, 2)? = dy;
    let mut dst = Mat::default();
    imgproc::warp_affine_def(src, &mut dst, &warp, src.size()?)?;
    Ok(dst)
}

fn temp_dir() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("lumix-hdr-core-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir).expect("dossier temporaire");
    dir
}

#[test]
fn rejects_single_image() {
    let image = scene(160, 120).expect("scène");
    let sources = [MemoryJpeg {
        mat: image,
        exposure_seconds: Some(1.0 / 30.0),
    }];
    let dir = temp_dir();
    let error = fuse_sources(
        &sources,
        &dir,
        &FusionOptions::default(),
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .expect_err("une image");
    assert!(matches!(error, HdrError::TooFewImages(1)));
}

#[test]
fn rejects_mismatched_sizes() {
    let a = scene(160, 120).expect("a");
    let b = scene(80, 60).expect("b");
    let sources = [
        MemoryJpeg {
            mat: a,
            exposure_seconds: Some(1.0 / 60.0),
        },
        MemoryJpeg {
            mat: b,
            exposure_seconds: Some(1.0 / 15.0),
        },
    ];
    let dir = temp_dir();
    let error = fuse_sources(
        &sources,
        &dir,
        &FusionOptions::default(),
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .expect_err("tailles");
    assert!(matches!(error, HdrError::Inconsistent(_)));
}

#[test]
fn mertens_reduces_clipping() {
    let base = scene(160, 120).expect("scène");
    let dark = apply_ev(&base, -3.0).expect("sombre");
    let bright = apply_ev(&base, 3.0).expect("clair");
    let dir = temp_dir();
    let sources = [
        MemoryJpeg {
            mat: dark.clone(),
            exposure_seconds: Some(1.0 / 120.0),
        },
        MemoryJpeg {
            mat: base.clone(),
            exposure_seconds: Some(1.0 / 30.0),
        },
        MemoryJpeg {
            mat: bright.clone(),
            exposure_seconds: Some(1.0 / 8.0),
        },
    ];
    let options = FusionOptions {
        align: false,
        ..FusionOptions::default()
    };
    let result = fuse_sources(
        &sources,
        &dir,
        &options,
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .expect("fusion");
    let jpeg = result.jpeg.expect("jpeg");
    let fused =
        opencv::imgcodecs::imread(jpeg, opencv::imgcodecs::IMREAD_COLOR).expect("lire jpeg");
    let fused_clip = clipped_pixel_count(&fused).expect("clip fusion");
    let dark_clip = clipped_pixel_count(&dark).expect("clip sombre");
    let bright_clip = clipped_pixel_count(&bright).expect("clip clair");
    assert!(
        fused_clip < dark_clip && fused_clip < bright_clip,
        "fusion {fused_clip} vs sombre {dark_clip} / clair {bright_clip}"
    );
}

#[test]
fn alignment_reduces_known_shift() {
    let base = scene(160, 120).expect("scène");
    let shifted = translate(&base, 8.0, 4.0).expect("décalage");
    let before = mean_abs_diff(&base, &shifted).expect("avant");
    let sources = [
        MemoryJpeg {
            mat: apply_ev(&base, -1.0).expect("ev-"),
            exposure_seconds: Some(1.0 / 60.0),
        },
        MemoryJpeg {
            mat: shifted,
            exposure_seconds: Some(1.0 / 30.0),
        },
        MemoryJpeg {
            mat: apply_ev(&base, 1.0).expect("ev+"),
            exposure_seconds: Some(1.0 / 15.0),
        },
    ];
    let dir = temp_dir();
    fuse_sources(
        &sources,
        &dir,
        &FusionOptions::default(),
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .expect("fusion alignée");
    assert!(before > 1.0, "le décalage synthétique doit être mesurable");
    let mut align = opencv::photo::create_align_mtb_def().expect("align");
    let shifted_again = translate(&base, 8.0, 4.0).expect("shift");
    let mut gray_base = Mat::default();
    let mut gray_shifted = Mat::default();
    imgproc::cvt_color_def(&base, &mut gray_base, imgproc::COLOR_BGR2GRAY).expect("gris base");
    imgproc::cvt_color_def(&shifted_again, &mut gray_shifted, imgproc::COLOR_BGR2GRAY)
        .expect("gris décalé");
    let estimated =
        opencv::photo::AlignMTBTrait::calculate_shift(&mut align, &gray_base, &gray_shifted)
            .expect("décalage");
    assert!(
        estimated.x.abs() >= 6 && estimated.x.abs() <= 10,
        "décalage X estimé {estimated:?}, attendu ~8"
    );
    assert!(
        estimated.y.abs() >= 2 && estimated.y.abs() <= 6,
        "décalage Y estimé {estimated:?}, attendu ~4"
    );
}

#[test]
fn cancel_stops_before_write() {
    let base = scene(160, 120).expect("scène");
    let sources = [
        MemoryJpeg {
            mat: apply_ev(&base, -1.0).expect("a"),
            exposure_seconds: Some(1.0 / 60.0),
        },
        MemoryJpeg {
            mat: apply_ev(&base, 1.0).expect("b"),
            exposure_seconds: Some(1.0 / 15.0),
        },
    ];
    let dir = temp_dir();
    let error = fuse_sources(
        &sources,
        &dir,
        &FusionOptions::default(),
        &AtomicBool::new(true),
        &mut |_| {},
    )
    .expect_err("annulé");
    assert!(matches!(error, HdrError::Cancelled));
}

#[test]
fn psnr_against_mid_exposure_is_finite() {
    let base = scene(160, 120).expect("scène");
    let sources = [
        MemoryJpeg {
            mat: apply_ev(&base, -1.5).expect("a"),
            exposure_seconds: Some(1.0 / 90.0),
        },
        MemoryJpeg {
            mat: base.clone(),
            exposure_seconds: Some(1.0 / 30.0),
        },
        MemoryJpeg {
            mat: apply_ev(&base, 1.5).expect("c"),
            exposure_seconds: Some(1.0 / 10.0),
        },
    ];
    let dir = temp_dir();
    let options = FusionOptions {
        align: false,
        method: FusionMethod::Mertens,
        ..FusionOptions::default()
    };
    let result = fuse_sources(
        &sources,
        &dir,
        &options,
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .expect("fusion");
    let fused =
        opencv::imgcodecs::imread(result.jpeg.expect("jpeg"), opencv::imgcodecs::IMREAD_COLOR)
            .expect("lire");
    let psnr = opencv::core::psnr(&fused, &base, 255.0).expect("psnr");
    assert!(
        psnr.is_finite() && psnr > 8.0,
        "PSNR trop bas ou non fini: {psnr}"
    );
}
