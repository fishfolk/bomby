use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_enter_system(GameState::InGame, center_camera);
    }
}

fn center_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
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

    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation = (level_dimensions / 2.0).extend(999.9);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..default()
        },
        ..default()
    });
}
