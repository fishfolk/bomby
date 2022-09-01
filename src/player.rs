use bevy::prelude::*;

use bevy_inspector_egui::Inspectable;

pub struct PlayerPlugin;

const SPEED: f32 = 125.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_startup_system(spawn_player)
            .add_system(handle_input)
            .add_system(animate_player);
    }
}

#[derive(Component, Inspectable, Debug)]
pub enum PlayerAnimationState {
    Idle,
    Run,
}

#[derive(Component, Debug)]
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
        .insert(Player)
        .insert(PlayerAnimationState::Idle)
        .insert(Name::from("Player"));
}

fn animate_player(
    mut players: Query<(&mut TextureAtlasSprite, &PlayerAnimationState), With<Player>>,
    time: Res<Time>,
) {
    use PlayerAnimationState::*;
    const MILLIS_BETWEEN_FRAMES: u128 = 100;

    for (mut sprite, animation_state) in players.iter_mut() {
        sprite.index = match animation_state {
            Idle => ((time.time_since_startup().as_millis() / MILLIS_BETWEEN_FRAMES) % 14) as usize,
            Run => {
                ((time.time_since_startup().as_millis() / MILLIS_BETWEEN_FRAMES) % 6) as usize + 14
            }
        }
    }
}

fn handle_input(
    mut players: Query<(&mut Transform, &mut PlayerAnimationState), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, mut animation_state) in players.iter_mut() {
        let mut delta_x = 0.0;
        if keyboard.pressed(KeyCode::D) {
            delta_x += 1.0;
        }
        if keyboard.pressed(KeyCode::A) {
            delta_x -= 1.0;
        }

        let mut delta_y = 0.0;
        if keyboard.pressed(KeyCode::W) {
            delta_y += 1.0;
        }
        if keyboard.pressed(KeyCode::S) {
            delta_y -= 1.0;
        }

        transform.translation += Vec2::new(delta_x, delta_y).normalize_or_zero().extend(0.0)
            * SPEED
            * time.delta_seconds();

        *animation_state = match delta_x != 0.0 || delta_y != 0.0 {
            true => PlayerAnimationState::Run,
            false => PlayerAnimationState::Idle,
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
