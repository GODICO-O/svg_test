use android_activity::AndroidApp;
use log::LevelFilter;

#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("GDLauncherTest"),
    );

    log::info!("=== RUST NATIVE APK SUCCESS STARTED ===");

    loop {
        app.poll_events(None, |_event| {});
    }
}
