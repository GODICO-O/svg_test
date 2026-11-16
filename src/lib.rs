use android_activity::{AndroidApp, InputEvent};
use log::{error, info};
use std::io::Read;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Inisialisasi logger Android dengan max_level
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::Level::Debug),
    );

    info!("=== APLIKASI NATIVE RUST DIMULAI ===");

    // app.asset_manager() langsung mengembalikan AssetManager (bukan Option)
    let asset_manager = app.asset_manager();
    if let Ok(mut asset) = asset_manager.open("play.png") {
        info!("File play.png berhasil dibuka di dalam assets!");
        let mut buffer = Vec::new();
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

    let mut animating = false;

    // Main event loop Android 0.6
    while !app.is_destroyed() {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                android_activity::Event::InitWindow { .. } => {
                    info!("Window Android aktif!");
                    animating = true;
                }
                android_activity::Event::TermWindow => {
                    info!("Window Android ditutup!");
                    animating = false;
                }
                android_activity::Event::GainedFocus => {
                    info!("Aplikasi fokus.");
                }
                android_activity::Event::LostFocus => {
                    info!("Aplikasi kehilangan fokus.");
                }
                _ => {}
            }
        });

        if animating {
            // Render loop / logika game/UI kustom kamu di sini
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
