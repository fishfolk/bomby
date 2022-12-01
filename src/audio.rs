use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;

use crate::{
    bomb::{self, BombExplodeEvent, PlayerDeathEvent},
    GameState,
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_audio).add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(play_fuse)
                .with_system(bomb_explosion)
                .with_system(wall_explosion)
                .into(),
        );
    }
}

fn wall_explosion(
    audio: Res<Audio>,
    sfx_explosion: Res<PlayerDeathSFX>,
    mut ev_destruction: EventReader<PlayerDeathEvent>,
) {
    let mut rng = thread_rng();
    ev_destruction.iter().for_each(|_| {
        audio.play(
            sfx_explosion
                .0
                .choose(&mut rng)
                .expect("resource should always contain at least one handle")
                .clone(),
        );
    })
}

fn bomb_explosion(
    audio: Res<Audio>,
    sfx_explosion: Res<BombExplosionSFX>,
    mut ev_explosion: EventReader<BombExplodeEvent>, // FIXME temporary hack
) {
    let mut rng = thread_rng();
    ev_explosion.iter().for_each(|_| {
        audio.play(
            sfx_explosion
                .0
                .choose(&mut rng)
                .expect("resource should always contain at least one handle")
                .clone(),
        );
    });
}

fn play_fuse(audio: Res<Audio>, sfx_fuse: Res<BombFuseSFX>, q: Query<Added<bomb::Bomb>>) {
    if q.iter().any(|b| b) {
        audio.play(sfx_fuse.0.clone());
    }
}

/// Handle to the SFX for the fuse sound when a bomb is placed.
#[derive(Resource)]
struct BombFuseSFX(Handle<AudioSource>);

/// Handles to sounds used when a bomb explodes.
#[derive(Resource)]
struct BombExplosionSFX(Vec<Handle<AudioSource>>);

/// Handles to sounds used when a wall is destroyed.
#[derive(Resource)]
struct PlayerDeathSFX(Vec<Handle<AudioSource>>);

fn load_audio(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(BombFuseSFX(assets.load("sfx/fuse.ogg")));

    commands.insert_resource(BombExplosionSFX(vec![
        assets.load("sfx/explosions/explosion_1.wav"),
        assets.load("sfx/explosions/explosion_2.wav"),
    ]));

    commands.insert_resource(PlayerDeathSFX(vec![
        assets.load("sfx/explosions/explosion_4.wav"),
        assets.load("sfx/explosions/explosion_5.wav"),
    ]));
}
