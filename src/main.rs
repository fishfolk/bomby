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
mod ui;
mod z_sort;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    /// For some reason, the first `StateTransition` schedule seems to happen before `PreStartup`.
    /// So, in order to initialise resources to be used in `MainMenu`, we have to initialise the
    /// state with the dummy variant `PreLoad`, which is immediately transitioned to `MainMenu` in
    /// the first `Startup` schedule. This seems to me like a scheduling bug in bevy, but I haven't
    /// opened an issue yet.
    #[default]
    PreLoad,
    MainMenu,
    LoadingLevel,
    InGame,
}

fn go_to_menu(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
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
                        resolution: (config.window_width, config.window_height).into(),
                        title: "Bomby!".to_string(),
                        resizable: config.window_resizable,
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
            ui::UiPlugin,
            z_sort::ZSortPlugin,
        ))
        .insert_resource(GameRng(SmallRng::from_entropy()))
        .add_systems(Startup, go_to_menu)
        .run();
}
