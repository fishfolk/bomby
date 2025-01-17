use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::GameState;

pub struct BombyLdtkPlugin;

impl Plugin for BombyLdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::index(0))
            .add_systems(OnEnter(GameState::LoadingLevel), setup)
            .add_systems(
                Update,
                finish_loading.run_if(in_state(GameState::LoadingLevel).and(level_spawned)),
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

pub trait GridNormalise {
    /// Take some coordinate system and normalise it based on `GridCoords`, such that the new value
    /// is the equivalent world coordinate centered on its `GridCoords`.
    fn grid_normalised(&self) -> Vec2;
}

impl<T> GridNormalise for T
where
    T: ToGrid,
{
    fn grid_normalised(&self) -> Vec2 {
        self.to_grid().to_world()
    }
}

/// Detect if there is a `Spawned` event from [`bevy_ecs_ldtk`], indicating that the level has spawned.
/// This means we can rely on entities existing such as the player spawn points.
pub fn level_spawned(mut level_events: EventReader<LevelEvent>) -> bool {
    level_events
        .read()
        .any(|e| matches!(e, LevelEvent::Spawned(_)))
}

fn finish_loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("level.ldtk").into(),
            ..default()
        },
        Name::new("LDtkWorld"),
    ));
}
