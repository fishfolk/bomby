use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

use bevy::sprite::Anchor;
use itertools::Itertools;

use crate::{
    bomb::{Bomb, CountBombs},
    ldtk::{self, ToGrid},
};

pub struct PlayerPlugin;

const SPEED: f32 = 125.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .insert_resource(CountPlayers(4))
            .add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_system(spawn_players.run_if(ldtk::level_spawned))
            .add_system(
                movement_input
                    .chain(player_collisions)
                    .chain(update_position),
            )
            .add_system(animate_player);
    }
}

/// Marker component for a Player
#[derive(Component, Debug)]
pub struct Player;

/// Linear velocity. Right now only for Player.
#[derive(Component, Default, Debug)]
pub struct Velocity(Vec2);

#[derive(Component, Debug)]
pub struct PlayerAnimator {
    /// Used to determine if the player's sprite should flip on the Y axis. This is only updated
    /// when the sprite flips.
    pub prev_x_velocity_sign: f32,
}

pub struct CountPlayers(usize);

/// Spawns the players in their correct spawn points up to the `CountPlayers` resource, which
/// should never exceed 4.
/// Player 1 - Fishy
/// Player 2 - Pescy
/// Player 3 - Sharky
/// Player 4 - Orcy
fn spawn_players(
    mut commands: Commands,
    texture: Res<PlayerSheets>,
    spawn_points: Query<(&Transform, &EntityInstance)>,
    count_players: Res<CountPlayers>,
) {
    for i in 0..count_players.0 {
        let player_name = format!("Player_{}", i + 1);

        let translation = spawn_points
            .iter()
            .filter(|(_, ldtk_entity)| ldtk_entity.identifier == player_name)
            .map(|(transform, _)| transform.translation.truncate())
            .next()
            .unwrap_or_else(|| panic!("no spawn point found for player: {}", player_name))
            + Vec2::Y * -8.0;

        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_translation(translation.extend(10.0)),
                texture_atlas: texture
                    .0
                    .get(i)
                    .unwrap_or_else(|| panic!("no sprite sheet for player: {}", player_name))
                    .clone(),
                sprite: TextureAtlasSprite {
                    index: 0,
                    flip_x: i % 2 != 0,
                    anchor: Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            })
            .insert(Player)
            .insert(Velocity::default())
            .insert(PlayerAnimator {
                prev_x_velocity_sign: 0.0,
            })
            .insert(CollisionBounds {
                x: (-8.0, 8.0),
                y: (0.0, 8.0),
            })
            .insert(CountBombs::default())
            // For testing purposes, all of the keys/controllers are hardcoded and assigned to the
            // same players each time.
            .insert_bundle(InputManagerBundle::<PlayerAction> {
                input_map: match i {
                    0 => InputMap::new([(
                        VirtualDPad {
                            up: KeyCode::W.into(),
                            down: KeyCode::S.into(),
                            left: KeyCode::A.into(),
                            right: KeyCode::D.into(),
                        },
                        PlayerAction::Move,
                    )])
                    .insert(KeyCode::Space, PlayerAction::Bomb)
                    .build(),
                    1 => InputMap::new([(
                        VirtualDPad {
                            up: KeyCode::Up.into(),
                            down: KeyCode::Down.into(),
                            left: KeyCode::Left.into(),
                            right: KeyCode::Right.into(),
                        },
                        PlayerAction::Move,
                    )])
                    .insert(KeyCode::RShift, PlayerAction::Bomb)
                    .build(),
                    2 => InputMap::new([(DualAxis::left_stick(), PlayerAction::Move)])
                        .insert(VirtualDPad::dpad(), PlayerAction::Move)
                        .insert(GamepadButtonType::East, PlayerAction::Bomb)
                        .set_gamepad(Gamepad { id: 0 })
                        .build(),
                    3 => InputMap::new([(DualAxis::left_stick(), PlayerAction::Move)])
                        .insert(VirtualDPad::dpad(), PlayerAction::Move)
                        .insert(GamepadButtonType::East, PlayerAction::Bomb)
                        .set_gamepad(Gamepad { id: 1 })
                        .build(),
                    _ => panic!("no input map for player: {}", player_name),
                },
                ..default()
            })
            .insert(Name::new(player_name));
    }
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
        if velocity.0.x != 0.0 && velocity.0.x.signum() != animator.prev_x_velocity_sign.signum() {
            sprite.flip_x = velocity.0.x < 0.0;
            animator.prev_x_velocity_sign = velocity.0.x;
        }
    }
}

// NOTE: If you are adding an Action, remember to update the `InputMap`s!
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Move,
    Bomb,
}

/// Get input and update the `Velocity` component of `Player`.
fn movement_input(
    mut players: Query<(&ActionState<PlayerAction>, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    for (action_state, mut velocity) in players.iter_mut() {
        let mut axis_data = Vec2::ZERO;
        if action_state.pressed(PlayerAction::Move) {
            axis_data = action_state.axis_pair(PlayerAction::Move).unwrap().xy();
        }
        velocity.0 = axis_data.normalize_or_zero() * SPEED * time.delta_seconds();
    }
}

/// Collision bounds from entity `Transform` of form (min, max)
#[derive(Component, Debug)]
pub struct CollisionBounds {
    pub x: (f32, f32),
    pub y: (f32, f32),
}

/// Detect player collisions with walls and bombs to restrict movement
fn player_collisions(
    mut players: Query<(&mut Velocity, &Transform, &CollisionBounds), With<Player>>,
    tiles: Query<(&Parent, &GridCoords)>,
    bombs: Query<&Transform, With<Bomb>>,
    ldtk_layer_meta_q: Query<&LayerMetadata>,
) {
    // Get the coords of tiles with a bomb on them
    let bomb_tiles = bombs
        .iter()
        .map(|t| t.translation.to_grid())
        .collect::<Vec<_>>();

    let unwalkable = tiles
        .iter()
        .filter(|(parent, coords)| {
            bomb_tiles.iter().any(|b| b == *coords)
                || matches!(
                    ldtk_layer_meta_q
                        .get(***parent)
                        .expect("tile must be a child of a layer")
                        .identifier
                        .as_str(),
                    "Maze" | "Bombable"
                )
        })
        .map(|(_, coords)| coords)
        .collect::<Vec<_>>();

    for (mut player_velocity, player_transform, player_bounds) in players.iter_mut() {
        if unwalkable.iter().any(|coords| {
            let x = player_transform.translation.truncate() + Vec2::X * player_velocity.0.x;
            match player_velocity.0.x.signum() as i8 {
                -1 => vec![player_bounds.x.0],
                1 => vec![player_bounds.x.1],
                _ => Vec::new(),
            }
            .iter()
            .cartesian_product(vec![player_bounds.y.0, player_bounds.y.1].iter())
            .map(|(bound_x, bound_y)| (x + Vec2::X * *bound_x + Vec2::Y * *bound_y).to_grid())
            .filter(|player_coord| *player_coord != player_transform.translation.to_grid())
            .any(|player_coord| player_coord == **coords)
        }) {
            player_velocity.0.x = 0.0;
        }

        if unwalkable.iter().any(|coords| {
            let y = player_transform.translation.truncate() + Vec2::Y * player_velocity.0.y;
            match player_velocity.0.y.signum() as i8 {
                -1 => vec![player_bounds.y.0],
                1 => vec![player_bounds.y.1],
                _ => Vec::new(),
            }
            .iter()
            .cartesian_product(vec![player_bounds.x.0, player_bounds.x.1].iter())
            .map(|(bound_y, bound_x)| (y + Vec2::X * *bound_x + Vec2::Y * *bound_y).to_grid())
            .filter(|player_coord| *player_coord != player_transform.translation.to_grid())
            .any(|player_coord| player_coord == **coords)
        }) {
            player_velocity.0.y = 0.0;
        }
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

struct PlayerSheets(Vec<Handle<TextureAtlas>>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    macro_rules! load {
        ($path:literal) => {{
            let image = assets.load($path);
            let atlas = TextureAtlas::from_grid_with_padding(
                image,
                Vec2::splat(64.0),
                14,
                7,
                Vec2::new(32.0, 16.0),
                Vec2::new(16.0, 0.0),
            );
            texture_atlases.add(atlas)
        }};
    }

    commands.insert_resource(PlayerSheets(vec![
        load!("player/PlayerFishy(64x64).png"),
        load!("player/PlayerPescy(64x64).png"),
        load!("player/PlayerSharky(64x64).png"),
        load!("player/PlayerOrcy(64x64).png"),
    ]));
}
