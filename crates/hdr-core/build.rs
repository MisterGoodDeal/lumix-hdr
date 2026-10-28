fn main() {
    // OpenCV static (vcpkg) référence CoCreateGuid (ole32) via cv::tempfile.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        println!("cargo:rustc-link-lib=ole32");
    }
}
