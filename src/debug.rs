use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.register_type::<bevy_ecs_ldtk::GridCoords>()
                .add_plugins(WorldInspectorPlugin::new());
            //.add_system(bevy::window::close_on_esc);
        }
    }
}
