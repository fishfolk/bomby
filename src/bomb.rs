use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    audio::PlaySfx,
    camera::CameraTrauma,
    ldtk::{GridNormalise, ToGrid},
    player::{Player, PlayerAction},
    z_sort::{ZSort, PLAYER_Z},
    GameState,
};

pub struct BombPlugin;

const MAX_BOMBS_PER_PLAYER: u8 = 2;
const BOMB_TIMER_SECS: f32 = 1.5;

/// The amount of trauma to send to the camera on an explosion.
const BOMB_TRAUMA: f32 = 0.3;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_graphics).add_systems(
            Update,
            (spawn_bombs, update_bombs, animate_bombs).run_if(in_state(GameState::InGame)),
        );
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
    bombs: Query<&Transform, With<Bomb>>,
    mut ev_sfx: EventWriter<PlaySfx>,
) {
    for (entity, translation, mut count_bombs) in players
        .iter_mut()
        .filter(|(_, action_state, _, _)| action_state.just_pressed(&PlayerAction::Bomb))
        .filter(|(_, _, _, count_bombs)| count_bombs.0 < MAX_BOMBS_PER_PLAYER)
        .filter(|(_, _, translation, _)| {
            bombs.iter().all(|bomb_transform| {
                bomb_transform.translation.to_grid() != translation.translation.to_grid()
            })
        })
        .map(|(entity, _, transform, count_bombs)| {
            (entity, transform.translation.grid_normalised(), count_bombs)
        })
    {
        commands.spawn((
            Sprite::from_atlas_image(texture_atlas.0.clone(), texture_atlas.1.clone().into()),
            Transform::from_translation(translation.extend(PLAYER_Z) + Vec3::Y * 2.0),
            ZSort(PLAYER_Z),
            Bomb {
                spawner: entity,
                timer: Timer::from_seconds(BOMB_TIMER_SECS, TimerMode::Once),
            },
        ));

        count_bombs.0 += 1;

        ev_sfx.send(PlaySfx::BombFuse);
    }
}

/// Tick the bomb timers. If fully elapsed, destroy the bomb and surrounding bombable tiles.
#[allow(clippy::too_many_arguments)]
fn update_bombs(
    mut commands: Commands,
    mut bombs: Query<(Entity, &mut Bomb, &Transform)>,
    mut players: Query<(Entity, &mut CountBombs, &Transform), With<Player>>,
    mut ev_trauma: EventWriter<CameraTrauma>,
    mut ev_sfx: EventWriter<PlaySfx>,
    time: Res<Time>,
    tiles: Query<(Entity, &Parent, &GridCoords)>,
    ldtk_layer_meta_q: Query<&LayerMetadata>,
) {
    for (entity, mut bomb, bomb_coords) in bombs
        .iter_mut()
        .map(|(e, b, transform)| (e, b, transform.translation.to_grid()))
    {
        bomb.timer.tick(time.delta());

        if bomb.timer.just_finished() {
            commands.entity(entity).despawn_recursive();

            // Decrement `CountBombs` component on the player that spawned the bomb
            if let Ok((_, mut bomb_count, _)) = players.get_mut(bomb.spawner) {
                bomb_count.0 -= 1;
            }

            // Get tiles within 1 orthogonal tile
            let affected_tiles = tiles
                .iter()
                .filter(|(_, _, coords)| {
                    let displacement = **coords - bomb_coords;
                    displacement.x.pow(2) + displacement.y.pow(2) <= 1
                })
                .collect::<Vec<_>>();

            // Destroy bombable tiles within 1 orthogonal tile
            for tile in affected_tiles.iter().filter(|(_, parent, coords)| {
                ldtk_layer_meta_q.get(***parent).map_or_else(
                    |_| {
                        warn!("LDtk tile not child of a layer with coords: {coords:?}");
                        false
                    },
                    |ldtk_layer| ldtk_layer.identifier == "Bombable",
                )
            }) {
                commands.entity(tile.0).despawn_recursive();
            }

            // Blow up players. Destroying for now, in future probably add a marker component which
            // then causes other behaviour
            for (entity, _, _) in players.iter().filter(|(_, _, player_transform)| {
                affected_tiles
                    .iter()
                    .any(|(_, _, coords)| player_transform.translation.to_grid() == **coords)
            }) {
                ev_sfx.send(PlaySfx::PlayerDeath);
                commands.entity(entity).despawn_recursive();
            }

            ev_sfx.send(PlaySfx::BombExplosion);
            ev_trauma.send(CameraTrauma(BOMB_TRAUMA));
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
            .extend(1.0);
    }
}

#[derive(Resource)]
pub struct BombSprite(Handle<Image>, Handle<TextureAtlasLayout>);

fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = assets.load("Bomb.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 33),
        1,
        1,
        Some(UVec2::ZERO),
        Some(UVec2::new(0, 11)),
    );
    let layout_handle = texture_atlases.add(layout);
    commands.insert_resource(BombSprite(image, layout_handle));
}
