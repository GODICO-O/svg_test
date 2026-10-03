use bevy::prelude::*;
use bevy_svg::prelude::*;

// Entry point untuk platform Android
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: bevy::winit::android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("GD_Launcher_Test"),
    );

    create_app(android_app);
}

// Fungsi pembangun utama aplikasi Bevy
fn create_app(#[cfg(target_os = "android")] android_app: bevy::winit::android_activity::AndroidApp) {
    let mut app = App::new();

    #[cfg(target_os = "android")]
    app.add_plugins(DefaultPlugins.set(bevy::winit::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            resizable: false,
            mode: bevy::window::WindowMode::Fullscreen,
            ..default()
        }),
        ..default()
    }));

    #[cfg(not(target_os = "android"))]
    app.add_plugins(DefaultPlugins);

    app.add_plugins(SvgPlugin)
       .add_systems(Startup, setup)
       .run();
}

// Fungsi Startup ECS Bevy 0.19
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Menggunakan komponen Camera2d baru (bukan Camera2dBundle lagi)
    commands.spawn(Camera2d);

    // Menggunakan komponen Svg2d baru dalam bentuk tuple komponen dasar
    commands.spawn((
        Svg2d {
            handle: asset_server.load("ui/button_test.svg"),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
    ));
}

// Entry point untuk desktop jika dijalankan lokal
#[cfg(not(target_os = "android"))]
fn main() {
    // Pada desktop, kita tidak butuh argumen android_app
    // Fungsi ini disesuaikan jika Anda ingin cargo run lokal
}
