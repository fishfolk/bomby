use bevy::prelude::*;
use iyes_loopless::prelude::*;

mod audio;
mod bomb;
mod camera;
mod debug;
mod ldtk;
mod player;
mod ui;
mod z_sort;

const RESOLUTION: f32 = 16.0 / 9.0;
const WINDOW_HEIGHT: f32 = 900.0;
const WINDOW_WIDTH: f32 = WINDOW_HEIGHT * RESOLUTION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    LoadingLevel,
    InGame,
}

fn main() {
    App::new()
        .add_loopless_state(GameState::MainMenu)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: WINDOW_WIDTH,
                        height: WINDOW_HEIGHT,
                        title: "Bomby!".to_string(),
                        resizable: false,
                        ..default()
                    },
                    ..default()
                }),
        )
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(audio::AudioPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ldtk::BombyLdtkPlugin)
        .add_plugin(bomb::BombPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(z_sort::ZSortPlugin)
        .run();
}
