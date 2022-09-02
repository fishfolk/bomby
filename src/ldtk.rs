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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(LdtkWorldBundle {
            ldtk_handle: asset_server.load("level.ldtk"),
            ..default()
        })
        .insert(Name::new("LDtkWorld"));
}
