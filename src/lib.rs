use bevy::{
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};

#[bevy_main]
fn main() {
    // 1. Inisialisasi Android Logger untuk Bevy 0.19 / android_logger 0.15
    #[cfg(target_os = "android")]
    {
        android_logger::init(
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
                    // Bevy 0.19 Automatic RenderCreation menggunakan Box<WgpuSettings>
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::GL | bevy::render::settings::Backends::VULKAN),
                        ..default()
                    })),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_ui)
        .run();
}

fn setup_ui(mut commands: Commands) {
    // 1. Kamera 2D di Bevy 0.19
    commands.spawn(Camera2d);

    // 2. Root UI Node Container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        ))
        .with_children(|parent| {
            // Bingkai Tombol Vektor Hijau
            parent
                .spawn((
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(160.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Status Text
            parent.spawn((
                Text::new("GD Launcher - Bevy 0.19 Active"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}
