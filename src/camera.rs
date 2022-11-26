use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, Perlin};

use crate::{bomb::BombExplodeEvent, GameState};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_enter_system(GameState::InGame, center_camera)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(apply_shake)
                    .with_system(decay_trauma)
                    .with_system(apply_trauma_on_explosion)
                    .into(),
            );
    }
}

#[derive(Component, Inspectable, Reflect, Default)]
pub struct CameraShake {
    #[inspectable(min = 0.0, max = 1.0)]
    trauma: f32,
    max_angle_rad: f32,
}

impl CameraShake {
    pub fn new(max_angle_deg: f32) -> Self {
        Self {
            max_angle_rad: max_angle_deg * (std::f32::consts::PI / 180.0),
            ..default()
        }
    }

    #[allow(dead_code)]
    pub fn with_trauma(trauma: f32, max_angle_deg: f32) -> Self {
        let mut shake = Self::new(max_angle_deg);
        shake.trauma = trauma;
        shake
    }

    /// Adds trauma to the camera, capping it at 1.0
    pub fn add_trauma(&mut self, value: f32) {
        self.trauma += value;
        if 1.0 < self.trauma {
            self.trauma = 1.0;
        }
    }
}

fn apply_trauma_on_explosion(
    mut cameras: Query<&mut CameraShake>,
    ev_explosion: EventReader<BombExplodeEvent>,
) {
    const BOMB_TRAUMA: f32 = 0.3;
    cameras
        .iter_mut()
        .for_each(|mut c| c.add_trauma(ev_explosion.len() as f32 * BOMB_TRAUMA));
}

/// Decay the trauma linearly over time
fn decay_trauma(mut q: Query<&mut CameraShake>, time: Res<Time>) {
    // Decays at a rate of DECAY_RATE per second. This could be converted into a field of
    // `CameraShake` if needed.
    const DECAY_RATE: f32 = 0.5;

    for mut shake in q.iter_mut() {
        shake.trauma = 0.0f32.max(shake.trauma - DECAY_RATE * time.delta_seconds());
    }
}

#[derive(Resource)]
struct ShakeNoise(Perlin);

/// Apply camera shake based on the current trauma.
fn apply_shake(
    mut q: Query<(&CameraShake, &mut Transform)>,
    time: Res<Time>,
    noise: Res<ShakeNoise>,
) {
    const SHAKE_SPEED: f32 = 3.0;

    for (shake, mut transform) in q.iter_mut() {
        transform.rotation = Quat::from_axis_angle(
            Vec3::Z,
            shake.trauma
                * shake.trauma
                * noise.0.get([(time.elapsed_seconds() * SHAKE_SPEED).into()]) as f32
                * shake.max_angle_rad,
        );
    }
}

/// Centers the camera on the LDtk world. Must have a single entity with `LdtkAsset` or this system
/// will panic.
fn center_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    ldtk_query: Query<&Handle<LdtkAsset>>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    level: Res<LevelSelection>,
) {
    // Get coordinates to center the camera on the level
    let ldtk_asset_handle = ldtk_query.single();
    let ldtk_level = ldtk_assets
        .get(ldtk_asset_handle)
        .unwrap()
        .get_level(&level)
        .unwrap();
    let level_dimensions = Vec2::new(ldtk_level.px_wid as f32, ldtk_level.px_hei as f32);

    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation = (level_dimensions / 2.0).extend(999.9);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.5,
                ..default()
            },
            ..default()
        },
        CameraShake::new(90.0),
    ));
    commands.insert_resource(ShakeNoise(Perlin::default()));
}
