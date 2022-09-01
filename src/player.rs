use bevy::prelude::*;

pub struct PlayerPlugin;

const SPEED: f32 = 100.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_player)
            .add_system(handle_input);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, texture: Res<PlayerSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::splat(0.0)),
            texture_atlas: texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            ..default()
        })
        .insert(Player);
}

fn handle_input(
    mut player_transforms: Query<&mut Transform, With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in player_transforms.iter_mut() {
        if keyboard.pressed(KeyCode::D) {
            transform.translation.x += SPEED * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::A) {
            transform.translation.x -= SPEED * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::W) {
            transform.translation.y += SPEED * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::S) {
            transform.translation.y -= SPEED * time.delta_seconds();
        }
    }
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
