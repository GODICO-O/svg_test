use android_activity::AndroidApp;
use log::{error, info};

#[no_mangle]
fn android_main(app: AndroidApp) {
    // Inisialisasi logger Android
    android_logger::init_once(
        android_logger::Config::default().with_min_level(log::Level::Debug),
    );

    info!("=== APLIKASI NATIVE RUST DIMULAI ===");

    // Contoh membaca file play.png dari folder assets Android
    // Di native activity, file di folder assets bisa diakses via AAssetManager
    if let Some(asset_manager) = app.asset_manager() {
        if let Ok(mut asset) = asset_manager.open("play.png") {
            info!("File play.png berhasil dibuka di dalam assets!");
            // Di sini kita bisa membaca byte buffer asset dan mendekodernya pakai crate `image`
            let mut buffer = Vec::new();
            use std::io::Read;
            if asset.read_to_end(&mut buffer).is_ok() {
                match image::load_from_memory(&buffer) {
                    Ok(img) => {
                        info!("PNG berhasil didekode! Resolusi: {}x{}", img.width(), img.height());
                    }
                    Err(e) => {
                        error!("Gagal mendecode PNG: {}", e);
                    }
                }
            }
        } else {
            error!("Gagal menemukan play.png di folder assets!");
        }
    }

    // Main event loop Android
    let mut animating = false;
    while !app.is_destroyed() {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                android_activity::AppEvent::InitWindow { .. } => {
                    info!("Window Android aktif!");
                    animating = true;
                }
                android_activity::AppEvent::TermWindow => {
                    info!("Window Android ditutup!");
                    animating = false;
                }
                android_activity::AppEvent::GainedFocus => {
                    info!("Aplikasi fokus.");
                }
                android_activity::AppEvent::LostFocus => {
                    info!("Aplikasi kehilangan fokus.");
                }
                _ => {}
            }
        });

        if animating {
            // Render loop / logika game/UI kustom kamu di sini tanpa Bevy
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
