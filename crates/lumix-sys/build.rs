use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let sdk_root = env::var("LUMIX_SDK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../vendor/lumix-sdk"));
    let library = sdk_root.join("Library");
    let include = library.clone();
    let lib_file = library.join("Lmxptpif.lib");
    let dll_file = library.join("Lmxptpif.dll");

    if !lib_file.is_file() {
        panic!(
            "Lmxptpif.lib introuvable dans {}. Placez le SDK dans vendor/lumix-sdk ou définissez LUMIX_SDK_DIR.",
            library.display()
        );
    }

    println!("cargo:rerun-if-env-changed=LUMIX_SDK_DIR");
    println!(
        "cargo:rerun-if-changed={}",
        library.join("LMX_func_api.h").display()
    );
    println!("cargo:rerun-if-changed=shim/lmx_shim.cpp");
    println!("cargo:rerun-if-changed=shim/lmx_shim.h");

    cc::Build::new()
        .cpp(true)
        .file("shim/lmx_shim.cpp")
        .include(&include)
        .flag_if_supported("/EHsc")
        .compile("lmx_shim");

    println!("cargo:rustc-link-search=native={}", library.display());
    println!("cargo:rustc-link-lib=dylib=Lmxptpif");
    // WideCharToMultiByte / callbacks SDK.
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=user32");
    // Ne pas charger Lmxptpif.dll avant main : son DllMain initialise COM en MTA
    // et fait planter tao/Tauri (OleInitialize → RPC_E_CHANGED_MODE).
    println!("cargo:rustc-link-lib=dylib=delayimp");
    println!("cargo:rustc-link-arg=/DELAYLOAD:Lmxptpif.dll");

    // Aide les runtimes (probe / tauri) à trouver la DLL.
    if dll_file.is_file() {
        println!("cargo:rustc-env=LUMIX_SDK_DLL={}", dll_file.display());
        println!("cargo:LUMIX_SDK_LIBRARY={}", library.display());
    }
}
