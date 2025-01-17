use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::prelude::*;

use crate::{config::Config, GameRng, GameState};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfx>()
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<SfxChannel>()
            .add_systems(PreStartup, (load_audio, set_volume))
            .add_systems(OnEnter(GameState::MainMenu), start_title_bgm)
            .add_systems(OnEnter(GameState::InGame), start_fight_bgm)
            .add_systems(OnExit(GameState::MainMenu), stop_bgm)
            .add_systems(OnExit(GameState::InGame), stop_bgm)
            .add_systems(Update, play_sfx.run_if(in_state(GameState::InGame)));
    }
}

/// Resource for the background music channel.
#[derive(Resource)]
struct BgmChannel;

/// Resource for the sound effects channel.
#[derive(Resource)]
struct SfxChannel;

/// Event for SFX.
#[derive(Event, Debug)]
pub enum PlaySfx {
    PlayerDeath,
    BombExplosion,
    BombFuse,
}

/// Update the channel volumes based on values in the [`Config`] resource.
fn set_volume(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    config: Res<Config>,
) {
    bgm_channel.set_volume(config.bgm_volume);
    sfx_channel.set_volume(config.sfx_volume);
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
    if let Some(audio_path) = bgm.in_game.choose(&mut rng.0) {
        audio.play(assets.load(*audio_path)).looped();
    } else {
        warn!("no paths to music files found in Bgm.in_game");
    }
}

fn stop_bgm(audio: Res<AudioChannel<BgmChannel>>) {
    audio.stop();
}

fn play_sfx(
    audio: Res<AudioChannel<SfxChannel>>,
    sfx: Res<Sfx>,
    mut rng: ResMut<GameRng>,
    mut ev_sfx: EventReader<PlaySfx>,
) {
    use PlaySfx::*;
    for ev in ev_sfx.read() {
        macro_rules! random_track {
            ($handles:expr) => {
                if let Some(audio_handle) = $handles.choose(&mut rng.0) {
                    audio.play(audio_handle.clone());
                } else {
                    warn!("no handles to SFX for {:?}", ev);
                }
            };
        }
        match ev {
            BombFuse => {
                audio.play(sfx.bomb_fuse.clone()).with_volume(0.5);
            }
            BombExplosion => random_track!(&sfx.bomb_explosion),
            PlayerDeath => random_track!(&sfx.player_death),
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
