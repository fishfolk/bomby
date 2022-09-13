use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct BombyLdtkPlugin;

impl Plugin for BombyLdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .insert_resource(LevelSelection::Index(0))
            .add_enter_system(GameState::LoadingLevel, setup)
            .add_system(
                finish_loading
                    .run_if(level_spawned)
                    .run_in_state(GameState::LoadingLevel),
            );
    }
}

pub const TILE_SIZE_PX: f32 = 32.0;
pub const TILE_SIZE_PX_INV: f32 = 1.0 / TILE_SIZE_PX;

pub trait ToWorld {
    /// Convert the LDtk grid coordinates into bevy world coordinates
    fn to_world(&self) -> Vec2;
}

impl ToWorld for GridCoords {
    fn to_world(&self) -> Vec2 {
        // NOTE: It would be good to be able to query the tile size at startup or compile time and
        // use it here, instead of hardcoding.
        Vec2::new(
            self.x as f32 * TILE_SIZE_PX + TILE_SIZE_PX / 2.0,
            self.y as f32 * TILE_SIZE_PX + TILE_SIZE_PX / 2.0,
        )
    }
}

pub trait ToGrid {
    /// Convert the bevy world coordinates into LDtk grid coordinates
    fn to_grid(&self) -> GridCoords;
}

impl ToGrid for Vec3 {
    fn to_grid(&self) -> GridCoords {
        GridCoords::new(
            (self.x * TILE_SIZE_PX_INV) as i32,
            (self.y * TILE_SIZE_PX_INV) as i32,
        )
    }
}

impl ToGrid for Vec2 {
    fn to_grid(&self) -> GridCoords {
        GridCoords::new(
            (self.x * TILE_SIZE_PX_INV) as i32,
            (self.y * TILE_SIZE_PX_INV) as i32,
        )
    }
}

/// Detect if there is a `Spawned` event from bevy_ecs_ldtk, indicating that the level has spawned.
/// This means we can rely on entities existing such as the player spawn points.
pub fn level_spawned(mut level_events: EventReader<LevelEvent>) -> bool {
    level_events
        .iter()
        .any(|e| matches!(e, LevelEvent::Spawned(_)))
}

fn finish_loading(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::InGame));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(LdtkWorldBundle {
            ldtk_handle: asset_server.load("level.ldtk"),
            ..default()
        })
        .insert(Name::new("LDtkWorld"));
}
