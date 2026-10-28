use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    let target = env::var("TARGET").unwrap_or_default();
    let is_windows = target.contains("windows")
        || env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows";
    if !is_windows {
        return;
    }

    // Évite le chargement de Lmxptpif.dll avant la fenêtre Tauri (COM MTA vs STA).
    // delayimp.lib doit être sur le binaire final (pas seulement la lib).
    println!("cargo:rustc-link-lib=dylib=delayimp");
    println!("cargo:rustc-link-arg-bins=delayimp.lib");
    println!("cargo:rustc-link-arg-bins=/DELAYLOAD:Lmxptpif.dll");
    println!("cargo:rustc-link-arg=/DELAYLOAD:Lmxptpif.dll");

    // Copie Lmxptpif.dll à côté des binaires debug/release (sans la versionner).
    let dll = env::var_os("DEP_LMXPTPIF_LUMIX_SDK_LIBRARY")
        .map(PathBuf::from)
        .map(|lib| lib.join("Lmxptpif.dll"))
        .or_else(|| {
            env::var_os("LUMIX_SDK_DIR")
                .map(PathBuf::from)
                .map(|root| root.join("Library").join("Lmxptpif.dll"))
        })
        .or_else(|| {
            let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").ok()?);
            manifest
                .join("../vendor/lumix-sdk/Library/Lmxptpif.dll")
                .canonicalize()
                .ok()
        });

    let Some(dll) = dll.filter(|p| p.is_file()) else {
        println!("cargo:warning=Lmxptpif.dll introuvable — placez le SDK dans vendor/lumix-sdk");
        return;
    };

    println!("cargo:rerun-if-changed={}", dll.display());
    println!("cargo:rerun-if-env-changed=LUMIX_SDK_DIR");

    if let Ok(out_dir) = env::var("OUT_DIR") {
        // OUT_DIR ≈ target/<profile>/build/<crate>/out → remonter vers target/<profile>
        let mut profile_dir = PathBuf::from(&out_dir);
        for _ in 0..3 {
            if !profile_dir.pop() {
                break;
            }
        }
        let dest = profile_dir.join("Lmxptpif.dll");
        if let Err(error) = fs::copy(&dll, &dest) {
            println!(
                "cargo:warning=copie Lmxptpif.dll vers {} échouée: {error}",
                dest.display()
            );
        } else {
            println!("cargo:rustc-env=LUMIX_HDR_COPIED_DLL={}", dest.display());
        }
    }
}
