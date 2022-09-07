use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    ldtk::{ToGrid, ToWorld},
    player::{Player, PlayerAction},
};

pub struct BombPlugin;

const MAX_BOMBS_PER_PLAYER: u8 = 2;
const BOMB_TIMER_SECS: f32 = 1.5;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_system(spawn_bombs)
            .add_system(update_bombs)
            .add_system(animate_bombs);
    }
}

#[derive(Component, Debug)]
pub struct Bomb {
    spawner: Entity,
    timer: Timer,
}

/// This is used to keep track of the current number of active bombs a player (or other bomb
/// wielding entity) has placed.
#[derive(Component, Default, Debug)]
pub struct CountBombs(u8);

fn spawn_bombs(
    mut commands: Commands,
    mut players: Query<
        (
            Entity,
            &ActionState<PlayerAction>,
            &Transform,
            &mut CountBombs,
        ),
        With<Player>,
    >,
    texture_atlas: Res<BombSprite>,
) {
    for (entity, translation, mut count_bombs) in players
        .iter_mut()
        .filter(|(_, _, _, count_bombs)| count_bombs.0 < MAX_BOMBS_PER_PLAYER)
        .filter(|(_, action_state, _, _)| action_state.just_pressed(PlayerAction::Bomb))
        .map(|(entity, _, transform, count_bombs)| {
            (
                entity,
                transform.translation.to_grid().to_world(),
                count_bombs,
            )
        })
    {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas.0.clone(),
                transform: Transform::from_translation(translation.extend(20.0) + Vec3::Y * 2.0),
                ..default()
            })
            .insert(Bomb {
                spawner: entity,
                timer: Timer::from_seconds(BOMB_TIMER_SECS, false),
            });

        count_bombs.0 += 1;
    }
}

/// Tick the bomb timers. If fully elapsed, destroy the bomb and surrounding bombable tiles.
fn update_bombs(
    mut commands: Commands,
    mut bombs: Query<(Entity, &mut Bomb, &Transform)>,
    mut players: Query<&mut CountBombs, With<Player>>,
    time: Res<Time>,
    tiles: Query<(Entity, &Parent, &GridCoords)>,
    ldtk_layer_meta_q: Query<&LayerMetadata>,
) {
    for (entity, mut bomb, bomb_transform) in bombs.iter_mut() {
        bomb.timer.tick(time.delta());

        if bomb.timer.just_finished() {
            commands.entity(entity).despawn_recursive();

            // Decrement `CountBombs` component on the player that spawned the bomb
            players
                .get_mut(bomb.spawner)
                .expect("exploded bomb had not parent player")
                .0 -= 1;

            // Destroy bombable tiles within 1 orthogonal tile
            for tile in tiles
                .iter()
                .filter(|(_, parent, _)| {
                    ldtk_layer_meta_q
                        .get(***parent)
                        .expect("tile must be a child of a layer")
                        .identifier
                        .as_str()
                        == "Bombable"
                })
                .filter(|(_, _, coords)| {
                    let displacement = **coords - bomb_transform.translation.to_grid();
                    displacement.x.pow(2) + displacement.y.pow(2) <= 1
                })
            {
                commands.entity(tile.0).despawn_recursive();
            }
        }
    }
}

/// Do not tick the bomb timer anywhere other than `update_bombs`.
fn animate_bombs(mut bombs: Query<(&Bomb, &mut Transform)>) {
    for (bomb, mut transform) in bombs.iter_mut() {
        transform.scale = Vec3::ONE
            + (Vec2::ONE
                * 0.1
                * ((16.0 * std::f32::consts::PI / 6.0) * bomb.timer.elapsed_secs()).sin())
            .extend(1.0)
    }
}

pub struct BombSprite(Handle<TextureAtlas>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Bomb.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(32.0, 33.0),
        1,
        1,
        Vec2::ZERO,
        Vec2::new(0.0, 11.0),
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(BombSprite(atlas_handle));
}