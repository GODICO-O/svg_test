use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::LevelFilter;

#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("GDLauncherTest"),
    );

    log::info!("=== GAME ACTIVITY BERHASIL DILUNCURKAN ===");

    let mut quit = false;
    while !quit {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            if let PollEvent::Main(MainEvent::Destroy) = event {
                log::info!("Aplikasi ditutup oleh sistem.");
                quit = true;
            }
        });
    }
}
