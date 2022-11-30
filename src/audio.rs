use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;

use crate::{
    bomb::{self, WallDestroyedEvent},
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
    sfx_explosion: Res<WallExplosionSFX>,
    mut ev_destruction: EventReader<WallDestroyedEvent>,
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
    ev_explosion: EventReader<crate::camera::CameraTrauma>, // FIXME temporary hack
) {
    if !ev_explosion.is_empty() {
        audio.play(sfx_explosion.0.clone());
    }
}

fn play_fuse(audio: Res<Audio>, sfx_fuse: Res<BombFuseSFX>, q: Query<Added<bomb::Bomb>>) {
    if q.iter().any(|b| b == true) {
        audio.play(sfx_fuse.0.clone());
    }
}

#[derive(Resource)]
struct BombFuseSFX(Handle<AudioSource>);

#[derive(Resource)]
struct WallExplosionSFX(Vec<Handle<AudioSource>>);

#[derive(Resource)]
struct BombExplosionSFX(Handle<AudioSource>);

fn load_audio(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(BombFuseSFX(assets.load("sfx/fuse.ogg")));
    commands.insert_resource(BombExplosionSFX(assets.load("sfx/8bit_bomb_explosion.wav")));

    let sfx_explosions: Vec<_> = (1..9)
        // Since we know at compile time all the filepaths, is there a way to avoid heap allocation
        // here?
        .map(|i| assets.load(format!("sfx/explosions/explosion0{i}.wav")))
        .collect();
    commands.insert_resource(WallExplosionSFX(sfx_explosions));
}
