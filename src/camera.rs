use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, Perlin};

use crate::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_enter_system(GameState::InGame, center_camera)
            .add_event::<CameraTrauma>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(apply_shake)
                    .with_system(decay_trauma)
                    .with_system(apply_trauma)
                    .into(),
            );
    }
}

/// Component for an entity with camera shake.
/// NOTE: To update the translation of such an entity, update the `center` field of this component
/// instead. Do not update the Transform component directly.
#[derive(Component, Inspectable, Reflect, Default)]
pub struct CameraShake {
    /// Value from 0-1 that indicates the intensity of the shake. Should be set with
    /// `CameraShake::add_trauma` and not manually decayed.
    #[inspectable(min = 0.0, max = 1.0)]
    trauma: f32,
    max_angle_rad: f32,
    max_offset: Vec2,
    /// The camera will always restore to this position.
    pub center: Vec3,
}

impl CameraShake {
    pub fn new(max_angle_deg: f32, max_offset: Vec2) -> Self {
        Self {
            max_angle_rad: max_angle_deg * (std::f32::consts::PI / 180.0),
            max_offset,
            ..default()
        }
    }

    #[allow(dead_code)]
    pub fn with_trauma(trauma: f32, max_angle_deg: f32, max_offset: Vec2) -> Self {
        let mut shake = Self::new(max_angle_deg, max_offset);
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

/// Event to add trauma to the camera. Provide a value between 0 and 1 for the trauma amount.
pub struct CameraTrauma(pub f32);

/// Apply the trauma sent by the [`CameraTrauma`] event to all the [`CameraShake`] components.
fn apply_trauma(mut cameras: Query<&mut CameraShake>, mut ev_trauma: EventReader<CameraTrauma>) {
    cameras
        .iter_mut()
        .for_each(|mut c| c.add_trauma(ev_trauma.iter().fold(0.0, |acc, trauma| acc + trauma.0)));
}

/// Decay the trauma linearly over time.
fn decay_trauma(mut q: Query<&mut CameraShake>, time: Res<Time>) {
    // Decays at a rate of DECAY_RATE per second. This could be converted into a field of
    // `CameraShake` if needed.
    const DECAY_RATE: f32 = 0.5;

    for mut shake in q.iter_mut() {
        shake.trauma = 0.0f32.max(shake.trauma - DECAY_RATE * time.delta_seconds());
    }
}

/// Resource that provides a source of noise for [`CameraShake`] entities to use.
#[derive(Resource)]
struct ShakeNoise(Perlin);

/// Apply camera shake based on the current trauma.
fn apply_shake(
    mut q: Query<(&CameraShake, &mut Transform)>,
    time: Res<Time>,
    noise: Res<ShakeNoise>,
) {
    const SHAKE_SPEED: f32 = 3.0;
    macro_rules! offset_noise {
        ($offset:expr) => {
            noise
                .0
                .get([((time.elapsed_seconds() + $offset) * SHAKE_SPEED).into()]) as f32
        };
    }

    for (shake, mut transform) in q.iter_mut() {
        (transform.rotation, transform.translation) = if shake.trauma > 0.0 {
            let sqr_trauma = shake.trauma * shake.trauma;

            let rotation = Quat::from_axis_angle(
                Vec3::Z,
                sqr_trauma * offset_noise!(0.0) * shake.max_angle_rad,
            );

            let x_offset = sqr_trauma * offset_noise!(100.0) * shake.max_offset.x;
            let y_offset = sqr_trauma * offset_noise!(200.0) * shake.max_offset.y;

            (rotation, shake.center + Vec3::new(x_offset, y_offset, 0.0))
        } else {
            // In future we may need to provide a rotation field on `CameraShake` should we need to
            // rotate the camera in another context.
            (Quat::IDENTITY, shake.center)
        }
    }
}

/// Centers the camera on the LDtk world. Must have a single entity with `LdtkAsset` or this system
/// will panic.
fn center_camera(
    mut camera_query: Query<&mut CameraShake, With<Camera>>,
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
    camera_transform.center = (level_dimensions / 2.0).extend(999.9);
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
        CameraShake::new(90.0, Vec2::splat(100.0)),
    ));
    commands.insert_resource(ShakeNoise(Perlin::default()));
}
