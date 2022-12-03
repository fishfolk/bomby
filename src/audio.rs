use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;

use crate::{GameRng, GameState};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfx>()
            .add_audio_channel::<BgmChannel>()
            .add_startup_system(load_audio)
            .add_enter_system(GameState::MainMenu, start_title_bgm)
            .add_enter_system(GameState::InGame, start_fight_bgm)
            .add_exit_system(GameState::MainMenu, stop_bgm)
            .add_exit_system(GameState::InGame, stop_bgm)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(play_sfx)
                    .into(),
            );
    }
}

/// Resource for the background music channel. Exists so in future a user may change BGM volume
/// independently of SFX.
#[derive(Resource)]
struct BgmChannel;

/// Event for SFX.
pub enum PlaySfx {
    PlayerDeath,
    BombExplosion,
    BombFuse,
}

fn start_title_bgm(audio: Res<AudioChannel<BgmChannel>>, bgm: Res<Bgm>) {
    audio.play(bgm.title_screen.clone()).looped();
}

fn start_fight_bgm(
    audio: Res<AudioChannel<BgmChannel>>,
    bgm: Res<Bgm>,
    assets: Res<AssetServer>,
    mut rng: ResMut<GameRng>,
) {
    audio
        .play(assets.load(*bgm.in_game.choose(&mut rng.0).unwrap()))
        .looped();
}

fn stop_bgm(audio: Res<AudioChannel<BgmChannel>>) {
    audio.stop();
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

/// Handles to BGM used in the game.
#[derive(Resource)]
struct Bgm {
    title_screen: Handle<AudioSource>,
    // Unlike the other cases, we are using the pathnames here because most of the tracks won't
    // ever be used and so loading them is wasteful.
    in_game: &'static [&'static str],
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

    commands.insert_resource(Bgm {
        title_screen: assets.load("music/Fishycuffs [title screen loop].ogg"),
        in_game: &[
            "music/Ahoy! [fight].ogg",
            "music/Bait the Hook [fight].ogg",
            "music/Jolly Roger [fight] .ogg",
            "music/Krill or be Krilled [fight].ogg",
            "music/Landlubber [fight].ogg",
            "music/Whalecome [fight].ogg",
        ],
    });
}
