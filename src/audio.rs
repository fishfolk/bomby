use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;

use crate::{GameRng, GameState};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfx>()
            .add_startup_system(load_audio)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(play_sfx)
                    .into(),
            );
    }
}

pub enum PlaySfx {
    PlayerDeath,
    BombExplosion,
    BombFuse,
}

fn play_sfx(
    audio: Res<Audio>,
    sfx: Res<Sfx>,
    mut rng: ResMut<GameRng>,
    mut ev_sfx: EventReader<PlaySfx>,
) {
    use PlaySfx::*;
    for ev in ev_sfx.iter() {
        match ev {
            BombFuse => {
                audio.play(sfx.bomb_fuse.clone()).with_volume(0.5);
            }
            PlayerDeath | BombExplosion => {
                audio.play(
                    match ev {
                        PlayerDeath => &sfx.player_death,
                        BombExplosion => &sfx.bomb_explosion,
                        _ => unreachable!(),
                    }
                    .choose(&mut rng.0)
                    .expect("resource should always contain at least one handle")
                    .clone(),
                );
            }
        }
    }
}

/// Handles to SFX used in the game.
#[derive(Resource)]
struct Sfx {
    bomb_fuse: Handle<AudioSource>,
    bomb_explosion: Box<[Handle<AudioSource>]>,
    player_death: Box<[Handle<AudioSource>]>,
}

fn load_audio(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sfx {
        bomb_fuse: assets.load("sfx/fuse.ogg"),
        bomb_explosion: Box::new([
            assets.load("sfx/explosions/explosion_1.wav"),
            assets.load("sfx/explosions/explosion_2.wav"),
        ]),
        player_death: Box::new([
            assets.load("sfx/explosions/explosion_4.wav"),
            assets.load("sfx/explosions/explosion_5.wav"),
        ]),
    });
}
