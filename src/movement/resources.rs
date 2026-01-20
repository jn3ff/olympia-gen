//! Movement domain: tuning and input resources.

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct MovementTuning {
    pub max_speed: f32,
    pub accel: f32,
    pub decel: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
    pub coyote_time: f32,
    pub wall_coyote_time: f32,
    pub jump_buffer_time: f32,
    /// Maximum air jumps (0 = no double jump, 1 = double jump, 2 = triple, etc.)
    pub max_air_jumps: u8,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_cooldown: f32,
    pub ground_only_dash: bool,
    pub wall_slide_speed: f32,
    pub wall_jump_horizontal: f32,
    pub wall_jump_vertical: f32,
    pub wall_jump_lock_time: f32,
}

impl Default for MovementTuning {
    fn default() -> Self {
        Self {
            max_speed: 320.0,
            accel: 3000.0,
            decel: 2600.0,
            jump_velocity: 680.0,
            gravity: 1800.0,
            coyote_time: 0.12,
            wall_coyote_time: 0.08,
            jump_buffer_time: 0.12,
            max_air_jumps: 0, // No double jump by default
            dash_speed: 900.0,
            dash_time: 0.16,
            dash_cooldown: 0.35,
            ground_only_dash: true,
            wall_slide_speed: 100.0,
            wall_jump_horizontal: 400.0,
            wall_jump_vertical: 600.0,
            wall_jump_lock_time: 0.15,
        }
    }
}

impl MovementTuning {
    /// Calculate the maximum height reachable from a single ground jump.
    /// Uses physics formula: h = v² / (2g)
    pub fn single_jump_height(&self) -> f32 {
        self.jump_velocity * self.jump_velocity / (2.0 * self.gravity)
    }

    /// Calculate the maximum height reachable with all available jumps.
    /// Each air jump adds additional height (assuming optimal timing at apex).
    /// This is a conservative estimate - actual height may vary with timing.
    pub fn max_reachable_height(&self) -> f32 {
        let base_height = self.single_jump_height();
        // Air jumps from apex add full jump height each
        // (1 ground jump + max_air_jumps air jumps)
        base_height * (1.0 + self.max_air_jumps as f32)
    }

    /// Calculate max reachable height with a safety margin for comfortable platforming.
    /// The margin accounts for imperfect jump timing and player collision box.
    pub fn safe_reachable_height(&self) -> f32 {
        // Use 80% of theoretical max for safe/comfortable gameplay
        self.max_reachable_height() * 0.8
    }
}

#[derive(Resource, Debug, Default)]
pub struct MovementInput {
    pub axis: Vec2,
    pub jump_just_pressed: bool,
    pub jump_held: bool,
    pub dash_just_pressed: bool,
}
