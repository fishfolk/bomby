#![warn(clippy::semicolon_if_nothing_returned, clippy::uninlined_format_args)]

use bevy::prelude::*;

use rand::{rngs::SmallRng, SeedableRng};

mod audio;
mod bomb;
mod camera;
mod config;
mod debug;
mod ldtk;
mod player;
//mod ui;
mod z_sort;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    #[default]
    LoadingLevel,
    InGame,
}

#[derive(Resource)]
pub struct GameRng(SmallRng);

fn main() {
    let config = config::load_config();
    info!("Initialised config: {config:?}");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (config.window_width.clone(), config.window_height.clone())
                            .into(),
                        title: "Bomby!".to_string(),
                        resizable: config.window_resizable.clone(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .insert_resource(config)
        .add_plugins((
            bevy_kira_audio::AudioPlugin,
            audio::AudioPlugin,
            debug::DebugPlugin,
            player::PlayerPlugin,
            ldtk::BombyLdtkPlugin,
            bomb::BombPlugin,
            camera::CameraPlugin,
            // ui::UiPlugin
            z_sort::ZSortPlugin,
        ))
        .insert_resource(GameRng(SmallRng::from_entropy()))
        .run();
}
