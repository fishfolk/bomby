use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

mod debug;
mod ldtk;
mod player;

const RESOLUTION: f32 = 16.0 / 9.0;
const WINDOW_HEIGHT: f32 = 900.0;
const WINDOW_WIDTH: f32 = WINDOW_HEIGHT * RESOLUTION;

fn main() {
    App::new()
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
        .add_system(spawn_camera.run_if(ldtk::level_spawned))
        .run();
}

// Current implementation requires that an entity with a Handle to `LdtkAsset` already exists and
// the `LevelSelection` resource is present
fn spawn_camera(
    mut commands: Commands,
    ldtk_query: Query<&Handle<LdtkAsset>>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    level: Res<LevelSelection>,
) {
    // Get coordinates to center the camera on the level
    let ldtk_asset_handle = ldtk_query.single();
    let ldtk_level = ldtk_assets
        .get(ldtk_asset_handle)
        .unwrap()
        .get_level(&level)
        .unwrap();
    let level_dimensions = Vec2::new(ldtk_level.px_wid as f32, ldtk_level.px_hei as f32);

    println!("{:?}", level);
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..default()
        },
        transform: Transform::from_translation((level_dimensions / 2.0).extend(999.9)),
        ..default()
    });
}
