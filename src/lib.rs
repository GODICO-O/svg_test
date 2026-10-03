use bevy::prelude::*;
use bevy_svg::prelude::*;

// Entry point untuk platform Android
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("GD_Launcher_Test"),
    );

    create_app(android_app);
}

// Fungsi pembangun utama aplikasi Bevy untuk Android
#[cfg(target_os = "android")]
fn create_app(android_app: android_activity::AndroidApp) {
    let mut app = App::new();

    // Di Bevy 0.19, konfigurasi Window dipasang di WindowPlugin, bukan WinitPlugin!
    app.add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            resizable: false,
            mode: bevy::window::WindowMode::Fullscreen(
                bevy::window::MonitorSelection::Current,
                bevy::window::VideoModeSelection::Current,
            ),
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(SvgPlugin)
       .add_systems(Startup, setup)
       .run();
}

// Fungsi pembangun versi Desktop lokal
#[cfg(not(target_os = "android"))]
fn create_app() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
       .add_plugins(SvgPlugin)
       .add_systems(Startup, setup)
       .run();
}

// Fungsi Startup ECS Bevy 0.19
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Memanggil komponen kamera 2D tunggal
    commands.spawn(Camera2d);

    // 2. Memuat berkas SVG menggunakan arsitektur Tuple Struct Svg2d bawaan bevy_svg 0.19
    commands.spawn((
        Svg2d(asset_server.load("ui/button_test.svg")),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
    ));
}

// Entry point utama Desktop jika Anda menjalankan 'cargo run' secara lokal
#[cfg(not(target_os = "android"))]
fn main() {
    create_app();
}
