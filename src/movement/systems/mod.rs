//! Movement domain: system modules for locomotion updates.

pub(crate) mod collisions;
pub(crate) mod input;
pub(crate) mod movement;

pub(crate) use collisions::{detect_ground, detect_walls};
pub(crate) use input::read_input;
pub(crate) use movement::{
    apply_dash, apply_gravity, apply_horizontal_movement, apply_jump, apply_wall_slide,
    update_facing, update_timers,
};
