//! Movement domain: player locomotion, input, and physics helpers.

mod bootstrap;
mod components;
mod dev;
mod resources;
mod systems;

pub use components::{
    Facing, GameLayer, Ground, MovementState, Player, Wall, WallContact, WallJumpLock,
};
pub use resources::{MovementInput, MovementTuning};

use bevy::prelude::*;

use crate::core::{GameState, gameplay_active};
use crate::movement::bootstrap::bootstrap_player_from_data;
use crate::movement::systems::{
    apply_dash, apply_gravity, apply_horizontal_movement, apply_jump, apply_wall_slide,
    detect_ground, detect_walls, read_input, update_facing, update_timers,
};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementTuning>()
            .init_resource::<MovementInput>()
            // Spawn player when entering Run state (after character selection)
            .add_systems(OnEnter(GameState::Run), bootstrap_player_from_data)
            .add_systems(
                Update,
                (
                    read_input,
                    detect_ground,
                    detect_walls,
                    update_timers,
                    apply_horizontal_movement,
                    apply_jump,
                    apply_wall_slide,
                    apply_dash,
                    apply_gravity,
                    update_facing,
                )
                    .chain()
                    .run_if(gameplay_active),
            );
    }
}
