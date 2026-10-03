use bevy::{
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings, WgpuFeatures},
        RenderPlugin,
    },
};

#[bevy_main]
fn main() {
    // 1. Inisialisasi Android Logger
    #[cfg(target_os = "android")]
    {
        android_logger::init_settings(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("GDLauncherTest"),
        );
    }

    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL RUST PANIC: {:?}", info);
    }));

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "GD Launcher Test".into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    // Pakai OpenGL / WebGL kompatibel backend untuk Android GPU Mali
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::GL | bevy::render::settings::Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_ui)
        .run();
}

fn setup_ui(mut commands: Commands) {
    // 1. Kamera 2D
    commands.spawn(Camera2dBundle::default());

    // 2. Render Khas GD (Kotak Tombol Hijau Vektor-Style via Native Node)
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgb(0.08, 0.08, 0.12).into(),
            ..default()
        })
        .with_children(|parent| {
            // Bingkai Tombol Vektor Hijau
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(160.0),
                        height: Val::Px(160.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.2, 0.8, 0.2).into(),
                    border_color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "PLAY",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });

            // Status Text
            parent.spawn(TextBundle::from_section(
                "GD Launcher - Bevy Native UI Active",
                TextStyle {
                    font_size: 18.0,
                    color: Color::GRAY,
                    ..default()
                },
            ));
        });
}
