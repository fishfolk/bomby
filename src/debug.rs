use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.register_type::<bevy_ecs_ldtk::GridCoords>()
                .add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<crate::camera::CameraShake>()
                .add_system(bevy::window::close_on_esc);
        }
    }
}
