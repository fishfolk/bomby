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
    pub resizable_window: bool,
    pub bgm_volume: f64,
    pub sfx_volume: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resizable_window: true,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(log_config);
    }
}

/// System to log the config on startup.
fn log_config(config: Res<Config>) {
    info!("{:?}", *config);
}

/// Load the [`Config`] or generate a new one.
pub fn load_config() -> Config {
    let dirs = ProjectDirs::from("com", "Spicy Lobster", "Bomby");
    dirs.map(|dirs| {
        let mut path = dirs.config_dir().to_path_buf();
        path.push("config.toml");
        let config_str = fs::read_to_string(&path).unwrap_or_else(|_| "".to_string());
        toml::from_str(&config_str).unwrap_or_else(|e| {
            warn!("failed to parse config file {path:?}: {e}");
            Config::default()
        })
    })
    .unwrap_or_else(|| {
        warn!("failed to get config path");
        Config::default()
    })
}
