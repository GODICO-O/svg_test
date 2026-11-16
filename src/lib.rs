use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::{error, info};
use std::io::Read;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Inisialisasi sistem log Android
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    info!("=== APLIKASI NATIVE RUST DIMULAI ===");

    // 1. Load file play.png dari folder assets/ menggunakan crate `image`
    let asset_manager = app.asset_manager();
    if let Some(mut asset) = asset_manager.open(c"play.png") {
        let mut buffer = Vec::new();
        if asset.read_to_end(&mut buffer).is_ok() {
            match image::load_from_memory(&buffer) {
                Ok(img) => {
                    info!(
                        "SUKSES: PNG berhasil didekode! Resolusi: {}x{}, Format Warna: {:?}",
                        img.width(),
                        img.height(),
                        img.color()
                    );
                }
                Err(e) => {
                    error!("GAGAL mendecode PNG: {}", e);
                }
            }
        } else {
            error!("GAGAL membaca isi file play.png!");
        }
    } else {
        error!("GAGAL menemukan play.png di dalam folder assets!");
    }

    let mut animating = false;
    let mut destroy = false;

    // 2. Event Loop utama Android
    while !destroy {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(MainEvent::InitWindow { .. }) => {
                    info!("Window Android aktif dan siap!");
                    animating = true;
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                    info!("Window Android ditutup.");
                    animating = false;
                }
                PollEvent::Main(MainEvent::GainedFocus) => {
                    info!("Aplikasi mendapatkan fokus.");
                }
                PollEvent::Main(MainEvent::LostFocus) => {
                    info!("Aplikasi kehilangan fokus.");
                }
                PollEvent::Main(MainEvent::Destroy) => {
                    info!("Aplikasi dihancurkan oleh sistem.");
                    destroy = true;
                }
                _ => {}
            }
        });

        if animating {
            // Di sini kita jaga agar aplikasi tetap berjalan stabil tanpa melakukan FFI OpenGL yang berisiko crash
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
