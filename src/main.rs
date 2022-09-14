use bevy::{prelude::*, render::texture::ImageSettings};
use iyes_loopless::prelude::*;

mod bomb;
mod camera;
mod debug;
mod ldtk;
mod player;
mod ui;

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
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            title: "Bomby!".to_string(),
            resizable: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ldtk::BombyLdtkPlugin)
        .add_plugin(bomb::BombPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(ui::UiPlugin)
        .run();
}
