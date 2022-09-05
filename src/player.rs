use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

use crate::ldtk;

pub struct PlayerPlugin;

const SPEED: f32 = 125.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_system(spawn_player.run_if(ldtk::level_spawned))
            .add_system(handle_input.chain(update_position))
            .add_system(animate_player);
    }
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct Velocity(Vec2);

#[derive(Component, Debug)]
pub struct PlayerAnimator {
    /// Used to determine if the player's sprite should flip on the Y axis. This is only updated
    /// when the sprite flips.
    pub prev_x_velocity_sign: f32,
}

/// Spawns the player "Fishy" in the Player_1 spot. In future, this will change to spawn an
/// arbitrary player at a specified player spot
fn spawn_player(
    mut commands: Commands,
    texture: Res<PlayerSheet>,
    spawn_points: Query<(&Transform, &EntityInstance)>,
) {
    let translation = spawn_points
        .iter()
        .filter(|(_, ldtk_entity)| ldtk_entity.identifier == "Player_1")
        .map(|(transform, _)| transform.translation.truncate())
        .next()
        .expect("no spawn point found for player!")
        + Vec2::Y * 16.0;

    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_translation(translation.extend(10.0)),
            texture_atlas: texture.0.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            ..default()
        })
        .insert(Player)
        .insert(Velocity::default())
        .insert(PlayerAnimator {
            prev_x_velocity_sign: 0.0,
        })
        .insert(Name::from("Player"));
}

fn animate_player(
    mut players: Query<(&mut TextureAtlasSprite, &mut PlayerAnimator, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    const MILLIS_BETWEEN_FRAMES: u128 = 100;

    for (mut sprite, mut animator, velocity) in players.iter_mut() {
        sprite.index = if velocity.0.length_squared() == 0.0 {
            ((time.time_since_startup().as_millis() / MILLIS_BETWEEN_FRAMES) % 14) as usize
        } else {
            ((time.time_since_startup().as_millis() / MILLIS_BETWEEN_FRAMES) % 6) as usize + 14
        };

        // Determine if the sprite should be flipped
        if velocity.0.x != 0.0 {
            if velocity.0.x.signum() != animator.prev_x_velocity_sign.signum() {
                sprite.flip_x = !sprite.flip_x;
                animator.prev_x_velocity_sign = velocity.0.x;
            }
        }
    }
}

/// Get keyboard input and update the `Velocity` component of `Player`.
/// For now, this is hardcoded to use WASD and will update every player. In the future, we will use
/// a more sophisticated system.
fn handle_input(
    mut players: Query<&mut Velocity, With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut velocity in players.iter_mut() {
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

        velocity.0 = Vec2::new(delta_x, delta_y).normalize_or_zero() * SPEED * time.delta_seconds();
    }
}

/// Update `Transform` components based on `Velocity`.
// This is a pretty generic system, and if in future we have different entities that need a
// velocity (such as enemies) then we should move some of this stuff to a `movement` module or
// similar.
fn update_position(mut q: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += velocity.0.extend(0.0);
    }
}

struct PlayerSheet(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("player/PlayerPescy(64x64).png");
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
