//! Load the settings file for the game. This will be under the config folder by OS convention, for
//! example:
//!
//! Linux: `~/.config/bomby/config.toml`

use bevy::prelude::*;

use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};

use std::fs;

/// Config resource containing runtime settings for the game.
#[derive(Resource, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub window_resizable: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub bgm_volume: f64,
    pub sfx_volume: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_resizable: true,
            window_width: 1600.0,
            window_height: 900.0,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_config);
    }
}

/// Load the [`Config`] or generate a new one and insert it as a resource.
fn load_config(mut commands: Commands) {
    let dirs = ProjectDirs::from("com", "Spicy Lobster", "Bomby");
    let mut config = dirs
        .map(|dirs| {
            let mut path = dirs.config_dir().to_path_buf();
            path.push("config.toml");
            let config_str = fs::read_to_string(&path).unwrap_or_else(|_| "".to_string());
            let mut de = toml::de::Deserializer::new(&config_str);
            let mut unused_keys = Vec::new();
            let config =
                serde_ignored::deserialize(&mut de, |path| unused_keys.push(path.to_string()))
                    .unwrap_or_else(|e| {
                        warn!("failed to parse config file {path:?}: {e}");
                        Config::default()
                    });

            for key in unused_keys {
                warn!("unrecognised config setting: {key}");
            }
            config
        })
        .unwrap_or_else(|| {
            warn!("failed to get config path");
            Config::default()
        });

    // Ensure sensible bounds.
    config.bgm_volume = config.bgm_volume.clamp(0.0, 1.0);
    config.sfx_volume = config.sfx_volume.clamp(0.0, 1.0);

    info!("{config:?}");
    commands.insert_resource(config);
}
