//! Smoke test SDK : Init → liste PnP → Open_Session → lit ISO/SS/Aperture/Mode → Close.
//!
//! Windows uniquement. Placez `Lmxptpif.dll` dans le `PATH` ou à côté du binaire
//! (`cargo run -p lumix-probe`).

#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

#[cfg(not(windows))]
fn main() {
    eprintln!("lumix-probe : disponible uniquement sous Windows (SDK LUMIX).");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(error) = run() {
        eprintln!("échec: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run() -> Result<(), String> {
    use std::ffi::CStr;

    use lumix_sys::{
        decode_aperture_f, decode_iso, decode_ss_seconds, lmx_sys_aperture_get_param,
        lmx_sys_camera_mode_get_mode_pos, lmx_sys_close_device, lmx_sys_close_session,
        lmx_sys_init, lmx_sys_iso_get_param, lmx_sys_list_pnp_models, lmx_sys_open_session,
        lmx_sys_select_pnp_index, lmx_sys_ss_get_param, LMX_BOOL_TRUE, LMX_CONNECT_VER,
        LMX_MODEL_UTF8_STRIDE, LMX_PTP_ARRAY_MAX,
    };

    println!("Lumix probe — Init…");
    unsafe {
        lmx_sys_init();
    }

    let mut models = vec![0u8; LMX_PTP_ARRAY_MAX * LMX_MODEL_UTF8_STRIDE];
    let mut count = 0u32;
    let mut ret_error = 0u32;
    let ok = unsafe {
        lmx_sys_list_pnp_models(
            models.as_mut_ptr().cast(),
            LMX_MODEL_UTF8_STRIDE as u32,
            LMX_PTP_ARRAY_MAX as u32,
            &mut count,
            &mut ret_error,
        )
    };
    if ok != LMX_BOOL_TRUE {
        return Err(format!(
            "Get_PnPDeviceInfo a échoué (retError=0x{ret_error:08X})"
        ));
    }

    println!("Appareils PnP : {count} (retError=0x{ret_error:08X})");
    if count == 0 {
        return Err(
            "aucun appareil LUMIX détecté par le SDK (Get_PnPDeviceInfo).\n\
             Windows doit voir le GH6 comme périphérique WPD/USB *présent*\n\
             (pas « Code 45 / phantom »). Vérifiez :\n\
             - mode USB de l'appareil = « PC (Tether) » (pas Mass Storage / MTP seul)\n\
             - câble USB-C direct, sans hub ni rallonge\n\
             - appareil allumé, écran non en veille USB\n\
             - Gestionnaire de périphériques → actualiser ; drivers Panasonic OK\n\
             - aucune autre app tether (Lumix Tether, etc.) qui garde le port"
                .to_string(),
        );
    }
    for i in 0..count as usize {
        let start = i * LMX_MODEL_UTF8_STRIDE;
        let end = start + LMX_MODEL_UTF8_STRIDE;
        let raw = &models[start..end];
        let name = CStr::from_bytes_until_nul(raw)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "(nom invalide)".to_string());
        println!("  [{}] {name}", i);
    }

    let index = 0u32;
    let ok = unsafe { lmx_sys_select_pnp_index(index, &mut ret_error) };
    if ok != LMX_BOOL_TRUE {
        return Err(format!(
            "Select_PnPDevice({index}) a échoué (retError=0x{ret_error:08X})"
        ));
    }
    println!("Sélectionné : index {index}");

    let mut device_ver = 0u32;
    let ok = unsafe { lmx_sys_open_session(LMX_CONNECT_VER, &mut device_ver, &mut ret_error) };
    if ok != LMX_BOOL_TRUE {
        let _ = unsafe { lmx_sys_close_device(&mut ret_error) };
        return Err(format!(
            "Open_Session a échoué (retError=0x{ret_error:08X})"
        ));
    }
    println!("Session ouverte (deviceConnectVer=0x{device_ver:08X})");

    let mut iso = 0u32;
    let mut ss = 0u32;
    let mut aperture = 0u32;
    let mut mode = 0u32;

    let ok_iso = unsafe { lmx_sys_iso_get_param(&mut iso, &mut ret_error) };
    let ok_ss = unsafe { lmx_sys_ss_get_param(&mut ss, &mut ret_error) };
    let ok_ap = unsafe { lmx_sys_aperture_get_param(&mut aperture, &mut ret_error) };
    let ok_mode = unsafe { lmx_sys_camera_mode_get_mode_pos(&mut mode, &mut ret_error) };

    if ok_iso == LMX_BOOL_TRUE {
        println!("ISO : {}", decode_iso(iso));
    } else {
        println!("ISO : lecture impossible (0x{ret_error:08X})");
    }
    if ok_ss == LMX_BOOL_TRUE {
        match decode_ss_seconds(ss) {
            Some(sec) => println!("SS  : {sec:.6} s (code=0x{ss:08X})"),
            None => println!("SS  : code spécial 0x{ss:08X}"),
        }
    } else {
        println!("SS  : lecture impossible (0x{ret_error:08X})");
    }
    if ok_ap == LMX_BOOL_TRUE {
        match decode_aperture_f(aperture as u16) {
            Some(f) => println!("F   : f/{f:.1} (code={aperture})"),
            None => println!("F   : code spécial {aperture}"),
        }
    } else {
        println!("F   : lecture impossible (0x{ret_error:08X})");
    }
    if ok_mode == LMX_BOOL_TRUE {
        println!("Mode dial pos : {mode}");
    } else {
        println!("Mode dial : lecture impossible (0x{ret_error:08X})");
    }

    let _ = unsafe { lmx_sys_close_session(&mut ret_error) };
    let _ = unsafe { lmx_sys_close_device(&mut ret_error) };
    println!("Fermé. OK.");
    Ok(())
}
