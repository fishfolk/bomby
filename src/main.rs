#![warn(clippy::semicolon_if_nothing_returned, clippy::uninlined_format_args)]

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use rand::{rngs::SmallRng, SeedableRng};

mod audio;
mod bomb;
mod camera;
mod config;
mod debug;
mod ldtk;
mod player;
mod ui;
mod z_sort;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    LoadingLevel,
    InGame,
}

#[derive(Resource)]
pub struct GameRng(SmallRng);

fn main() {
    let config = config::load_config();
    info!("Initialised config: {config:?}");

    App::new()
        .add_loopless_state(GameState::MainMenu)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: config.window_width,
                        height: config.window_height,
                        title: "Bomby!".to_string(),
                        resizable: config.window_resizable,
                        ..default()
                    },
                    ..default()
                }),
        )
        .insert_resource(config)
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(audio::AudioPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ldtk::BombyLdtkPlugin)
        .add_plugin(bomb::BombPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(z_sort::ZSortPlugin)
        .insert_resource(GameRng(SmallRng::from_entropy()))
        .run();
}
