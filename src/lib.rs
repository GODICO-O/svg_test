use bevy::prelude::*;
use bevy_svg::prelude::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SvgPlugin)
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Kamera 2D
    commands.spawn(Camera2dBundle::default());

    // 2. Load file SVG tombol test
    let svg_handle = asset_server.load("ui/button_test.svg");

    // 3. Render SVG di tengah layar dengan Skala Vektor
    commands.spawn(Svg2dBundle {
        svg: svg_handle,
        origin: Origin::Center,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::splat(2.0), // Di-scale 2x lipat tanpa pecah
            ..default()
        },
        ..default()
    });
}
