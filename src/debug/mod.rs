//! Debug domain: developer UI, hotkeys, and runtime tweaks.

mod state;
mod systems;
mod ui;

pub use state::DebugState;

use bevy::prelude::*;

use crate::debug::systems::{
    apply_invincibility, handle_debug_buttons, handle_debug_hotkeys, toggle_debug_ui,
    update_debug_info_overlay, update_status_message,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .add_systems(
                Update,
                (
                    toggle_debug_ui,
                    handle_debug_hotkeys,
                    handle_debug_buttons,
                    update_status_message,
                    apply_invincibility,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                update_debug_info_overlay.run_if(|state: Res<DebugState>| state.show_info),
            );
    }
}
