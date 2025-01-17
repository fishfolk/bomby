use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

use std::cmp::Ordering;

use bevy::sprite::Anchor;
use itertools::Itertools;

use crate::{
    bomb::{Bomb, CountBombs},
    ldtk::ToGrid,
    z_sort::{ZSort, PLAYER_Z},
    GameState,
};

pub struct PlayerPlugin;

const SPEED: f32 = 125.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_graphics)
            .insert_resource(CountPlayers(4))
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::InGame), spawn_players)
            .add_systems(
                Update,
                (
                    (movement_input, player_collisions, update_position).chain(),
                    animate_player,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

/// Marker component for a Player
#[derive(Component, Debug)]
pub struct Player;

/// Linear velocity. Right now only for Player.
#[derive(Component, Default, Debug)]
pub struct Velocity(Vec2);

#[derive(Component, Default, Debug)]
pub struct PlayerAnimator {
    /// Used to determine if the player's sprite should flip on the Y axis. This is only updated
    /// when the sprite flips.
    pub prev_x_velocity_sign: f32,
}

/// The number of players that will be spawned during setup.
#[derive(Resource)]
pub struct CountPlayers(usize);

/// Spawns the players in their correct spawn points up to the `CountPlayers` resource, which
/// should never exceed 4.
/// Player 1 - Fishy
/// Player 2 - Pescy
/// Player 3 - Sharky
/// Player 4 - Orcy
fn spawn_players(
    mut commands: Commands,
    textures: Res<PlayerSheets>,
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
            .unwrap_or_else(|| panic!("no spawn point found for player: {player_name}"))
            + Vec2::Y * -8.0;

        commands.spawn((
            Player,
            Sprite {
                image: textures
                    .0
                    .get(i)
                    .unwrap_or_else(|| panic!("no sprite sheet for player: {player_name}"))
                    .clone(),
                texture_atlas: Some(textures.1.clone().into()),
                flip_x: i % 2 != 0,
                anchor: Anchor::BottomCenter,
                ..default()
            },
            Transform::from_translation(translation.extend(PLAYER_Z)),
            Velocity::default(),
            PlayerAnimator::default(),
            CollisionBounds {
                x: (-8.0, 8.0),
                y: (0.0, 8.0),
            },
            CountBombs::default(),
            // For testing purposes, all of the keys/controllers are hardcoded and assigned to the
            // same players each time.
            InputManagerBundle::<PlayerAction> {
                input_map: match i {
                    0 => InputMap::new([(PlayerAction::Bomb, KeyCode::Space)]).with_dual_axis(
                        PlayerAction::Move,
                        VirtualDPad::new(
                            KeyCode::KeyW,
                            KeyCode::KeyS,
                            KeyCode::KeyA,
                            KeyCode::KeyD,
                        ),
                    ),
                    1 => InputMap::new([(PlayerAction::Bomb, KeyCode::ShiftRight)]).with_dual_axis(
                        PlayerAction::Move,
                        VirtualDPad::new(
                            KeyCode::ArrowUp,
                            KeyCode::ArrowDown,
                            KeyCode::ArrowLeft,
                            KeyCode::ArrowRight,
                        ),
                    ),
                    // TODO: Since bevy 0.15 introduced "gamepads-as-entities", we will have to add
                    // some gamepad registering logic.
                    //
                    // See: https://github.com/Leafwing-Studios/leafwing-input-manager/blob/main/examples/register_gamepads.rs
                    2 => InputMap::new([(PlayerAction::Bomb, GamepadButton::East)])
                        .with_dual_axis(PlayerAction::Move, GamepadStick::LEFT),
                    //.set_gamepad(Gamepad { id: 0 })
                    3 => InputMap::new([(PlayerAction::Bomb, GamepadButton::East)])
                        .with_dual_axis(PlayerAction::Move, GamepadStick::LEFT),
                    //.set_gamepad(Gamepad { id: 1 })
                    _ => panic!("no input map for player: {player_name}"),
                },
                ..default()
            },
            ZSort(PLAYER_Z),
            Name::new(player_name),
        ));
    }
}

fn animate_player(
    mut players: Query<(&mut Sprite, &mut PlayerAnimator, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    const MILLIS_BETWEEN_FRAMES: u128 = 100;

    const IDLE_FRAMES: usize = 14;
    const RUN_FRAMES: usize = 6;

    for (mut sprite, mut animator, velocity) in players.iter_mut() {
        sprite.texture_atlas.as_mut().unwrap().index = if velocity.0.length_squared() == 0.0 {
            (time.elapsed().as_millis() / MILLIS_BETWEEN_FRAMES) as usize % IDLE_FRAMES
        } else {
            (time.elapsed().as_millis() / MILLIS_BETWEEN_FRAMES) as usize % RUN_FRAMES + IDLE_FRAMES
        };

        // Determine if the sprite should be flipped
        if velocity.0.x != 0.0 && velocity.0.x.signum() != animator.prev_x_velocity_sign.signum() {
            sprite.flip_x = velocity.0.x < 0.0;
            animator.prev_x_velocity_sign = velocity.0.x;
        }
    }
}

// NOTE: If you are adding an Action, remember to update the `InputMap`s!
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    Bomb,
}

/// Get input and update the `Velocity` component of `Player`.
fn movement_input(
    mut players: Query<(&ActionState<PlayerAction>, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    for (action_state, mut velocity) in players.iter_mut() {
        velocity.0 = action_state
            .axis_pair(&PlayerAction::Move)
            .normalize_or_zero()
            * SPEED
            * time.delta_secs();
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
                || ldtk_layer_meta_q.get(***parent).map_or_else(
                    |_| {
                        warn!("LDtk tile not child of a layer with coords: {coords:?}");
                        false
                    },
                    |ldtk_layer| matches!(ldtk_layer.identifier.as_str(), "Maze" | "Bombable"),
                )
        })
        .map(|(_, coords)| coords)
        .collect::<Vec<_>>();

    for (mut player_velocity, player_transform, player_bounds) in players.iter_mut() {
        if unwalkable.iter().any(|coords| {
            let x = player_transform.translation.truncate() + Vec2::X * player_velocity.0.x;
            match player_velocity.0.x.partial_cmp(&0.0) {
                Some(Ordering::Less) => vec![player_bounds.x.0],
                Some(Ordering::Greater) => vec![player_bounds.x.1],
                _ => Vec::new(),
            }
            .iter()
            .cartesian_product([player_bounds.y.0, player_bounds.y.1].iter())
            .map(|(bound_x, bound_y)| (x + Vec2::X * *bound_x + Vec2::Y * *bound_y).to_grid())
            .filter(|player_coord| *player_coord != player_transform.translation.to_grid())
            .any(|player_coord| player_coord == **coords)
        }) {
            player_velocity.0.x = 0.0;
        }

        if unwalkable.iter().any(|coords| {
            let y = player_transform.translation.truncate() + Vec2::Y * player_velocity.0.y;
            match player_velocity.0.y.partial_cmp(&0.0) {
                Some(Ordering::Less) => vec![player_bounds.y.0],
                Some(Ordering::Greater) => vec![player_bounds.y.1],
                _ => Vec::new(),
            }
            .iter()
            .cartesian_product([player_bounds.x.0, player_bounds.x.1].iter())
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

#[derive(Resource)]
struct PlayerSheets(Vec<Handle<Image>>, Handle<TextureAtlasLayout>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        14,
        7,
        Some(UVec2::new(32, 16)),
        Some(UVec2::new(16, 0)),
    );
    let layout = texture_atlas_layouts.add(layout);

    commands.insert_resource(PlayerSheets(
        vec![
            assets.load("player/PlayerFishy(64x64).png"),
            assets.load("player/PlayerPescy(64x64).png"),
            assets.load("player/PlayerSharky(64x64).png"),
            assets.load("player/PlayerOrcy(64x64).png"),
        ],
        layout,
    ));
}
