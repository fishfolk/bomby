use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct BombyLdtkPlugin;

impl Plugin for BombyLdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .add_startup_system(setup)
            .insert_resource(LevelSelection::Index(0));
    }
}

/// Detect if there is a `Spawned` event from bevy_ecs_ldtk, indicating that the level has spawned.
/// This means we can rely on entities existing such as the player spawn points.
pub fn level_spawned(mut level_events: EventReader<LevelEvent>) -> bool {
    level_events
        .iter()
        .any(|e| matches!(e, LevelEvent::Spawned(_)))
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(LdtkWorldBundle {
            ldtk_handle: asset_server.load("level.ldtk"),
            ..default()
        })
        .insert(Name::new("LDtkWorld"));
}
