use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_player);
    }
}

fn spawn_player(mut commands: Commands, texture: Res<PlayerSheet>) {
    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform::from_translation(Vec3::splat(0.0)),
        texture_atlas: texture.0.clone(),
        sprite: TextureAtlasSprite {
            index: 0,
            ..default()
        },
        ..default()
    });
}

struct PlayerSheet(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("PlayerPescy(64x64).png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::splat(64.0),
        14,
        7,
        Vec2::new(32.0, 16.0),
        Vec2::new(16.0, 0.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(PlayerSheet(atlas_handle));
}
