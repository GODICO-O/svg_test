use android_activity::{AndroidApp, Event, MainEvent};
use log::{error, info};
use std::io::Read;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    info!("=== APLIKASI NATIVE RUST DIMULAI ===");

    let asset_manager = app.asset_manager();
    if let Some(mut asset) = asset_manager.open(c"play.png") {
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
    let mut destroy = false;

    while !destroy {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                Event::Main(MainEvent::InitWindow { .. }) => {
                    info!("Window Android aktif!");
                    animating = true;
                }
                Event::Main(MainEvent::TerminateWindow { .. }) => {
                    info!("Window Android ditutup!");
                    animating = false;
                }
                Event::Main(MainEvent::GainedFocus) => {
                    info!("Aplikasi fokus.");
                }
                Event::Main(MainEvent::LostFocus) => {
                    info!("Aplikasi kehilangan fokus.");
                }
                Event::Main(MainEvent::Destroy) => {
                    info!("Aplikasi dihancurkan.");
                    destroy = true;
                }
                _ => {}
            }
        });

        if animating {
            // Render loop / logika UI kustom kamu di sini
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
