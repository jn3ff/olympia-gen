//! Movement domain: locomotion systems for timers and physics.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::movement::{
    Facing, MovementInput, MovementState, MovementTuning, Player, WallContact, WallJumpLock,
};

pub(crate) fn update_timers(
    time: Res<Time>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut WallJumpLock), With<Player>>,
) {
    let dt = time.delta_secs();

    for (mut state, mut wall_lock) in &mut query {
        // Coyote time: starts counting when leaving ground
        if !state.on_ground {
            state.coyote_timer += dt;
        }

        // Wall coyote time
        if state.on_wall == WallContact::None {
            state.wall_coyote_timer += dt;
        }

        // Jump buffer: counts down after pressing jump
        if state.jump_buffer_timer > 0.0 {
            state.jump_buffer_timer -= dt;
        }

        // Dash timer
        if state.is_dashing {
            state.dash_timer -= dt;
            if state.dash_timer <= 0.0 {
                state.is_dashing = false;
            }
        }

        // Dash cooldown
        if state.dash_cooldown_timer > 0.0 {
            state.dash_cooldown_timer -= dt;
        }

        // Wall jump lock
        if wall_lock.0 > 0.0 {
            wall_lock.0 -= dt;
        }

        // Reset dash cooldown when landing
        if state.on_ground && state.dash_cooldown_timer > 0.0 {
            state.dash_cooldown_timer = state.dash_cooldown_timer.min(tuning.dash_cooldown * 0.5);
        }
    }
}

pub(crate) fn apply_horizontal_movement(
    time: Res<Time>,
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&MovementState, &WallJumpLock, &mut LinearVelocity), With<Player>>,
) {
    let dt = time.delta_secs();

    for (state, wall_lock, mut velocity) in &mut query {
        // Skip horizontal control during dash or wall jump lock
        if state.is_dashing || wall_lock.0 > 0.0 {
            continue;
        }

        let target_vx = input.axis.x * tuning.max_speed;

        if input.axis.x.abs() > 0.1 {
            // Accelerate toward target
            let accel = tuning.accel * dt;
            if velocity.x < target_vx {
                velocity.x = (velocity.x + accel).min(target_vx);
            } else {
                velocity.x = (velocity.x - accel).max(target_vx);
            }
        } else {
            // Decelerate to zero
            let decel = tuning.decel * dt;
            if velocity.x > 0.0 {
                velocity.x = (velocity.x - decel).max(0.0);
            } else {
                velocity.x = (velocity.x + decel).min(0.0);
            }
        }
    }
}

pub(crate) fn apply_jump(
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut WallJumpLock, &mut LinearVelocity), With<Player>>,
) {
    for (mut state, mut wall_lock, mut velocity) in &mut query {
        // Skip during dash
        if state.is_dashing {
            continue;
        }

        // Buffer jump input
        if input.jump_just_pressed {
            state.jump_buffer_timer = tuning.jump_buffer_time;
        }

        let wants_jump = state.jump_buffer_timer > 0.0;
        let can_ground_jump = state.on_ground || state.coyote_timer < tuning.coyote_time;
        let can_wall_jump = !state.on_ground
            && (state.on_wall != WallContact::None
                || state.wall_coyote_timer < tuning.wall_coyote_time);

        let can_air_jump = !state.on_ground && state.air_jumps_remaining > 0;

        if wants_jump {
            if can_ground_jump {
                // Ground jump
                velocity.y = tuning.jump_velocity;
                state.jump_buffer_timer = 0.0;
                state.coyote_timer = tuning.coyote_time; // Consume coyote time
                debug!(
                    "Ground jump: on_ground={}, coyote consumed, air_jumps_remaining={}",
                    state.on_ground, state.air_jumps_remaining
                );
            } else if can_wall_jump {
                // Wall jump
                let wall_dir = if state.on_wall == WallContact::Left {
                    1.0
                } else if state.on_wall == WallContact::Right {
                    -1.0
                } else {
                    // Using wall coyote - use facing direction
                    match state.facing {
                        Facing::Left => 1.0,
                        Facing::Right => -1.0,
                    }
                };

                velocity.x = wall_dir * tuning.wall_jump_horizontal;
                velocity.y = tuning.wall_jump_vertical;
                wall_lock.0 = tuning.wall_jump_lock_time;
                state.jump_buffer_timer = 0.0;
                state.wall_coyote_timer = tuning.wall_coyote_time;
                // Wall jump also resets air jumps
                state.air_jumps_remaining = tuning.max_air_jumps;
                debug!(
                    "Wall jump: on_wall={:?}, air_jumps reset to {}",
                    state.on_wall, state.air_jumps_remaining
                );
            } else if can_air_jump {
                // Air jump (double jump, triple jump, etc.)
                velocity.y = tuning.jump_velocity;
                state.jump_buffer_timer = 0.0;
                state.air_jumps_remaining -= 1;
                debug!(
                    "Air jump: air_jumps_remaining now {}",
                    state.air_jumps_remaining
                );
            }
        }

        // Variable jump height - cut velocity when releasing jump
        if !input.jump_held && velocity.y > 0.0 && !state.on_ground {
            velocity.y *= 0.5;
        }
    }
}

pub(crate) fn apply_wall_slide(
    tuning: Res<MovementTuning>,
    input: Res<MovementInput>,
    mut query: Query<(&MovementState, &mut LinearVelocity), With<Player>>,
) {
    for (state, mut velocity) in &mut query {
        // Only wall slide when in the air, touching a wall, and holding toward the wall
        if state.on_ground || state.on_wall == WallContact::None || state.is_dashing {
            continue;
        }

        let holding_toward_wall = match state.on_wall {
            WallContact::Left => input.axis.x < -0.1,
            WallContact::Right => input.axis.x > 0.1,
            WallContact::None => false,
        };

        if holding_toward_wall && velocity.y < 0.0 {
            // Clamp fall speed to wall slide speed
            velocity.y = velocity.y.max(-tuning.wall_slide_speed);
        }
    }
}

pub(crate) fn apply_dash(
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut LinearVelocity), With<Player>>,
) {
    for (mut state, mut velocity) in &mut query {
        // Initiate dash
        if input.dash_just_pressed && state.dash_cooldown_timer <= 0.0 && !state.is_dashing {
            // Ground-only dash check
            if tuning.ground_only_dash && !state.on_ground {
                continue;
            }

            state.is_dashing = true;
            state.dash_timer = tuning.dash_time;
            state.dash_cooldown_timer = tuning.dash_cooldown;

            // Dash in facing direction (or input direction if provided)
            state.dash_direction = if input.axis.x.abs() > 0.1 {
                input.axis.x.signum()
            } else {
                match state.facing {
                    Facing::Right => 1.0,
                    Facing::Left => -1.0,
                }
            };
        }

        // Apply dash velocity
        if state.is_dashing {
            velocity.x = state.dash_direction * tuning.dash_speed;
            velocity.y = 0.0; // Lock vertical movement during dash
        }
    }
}

pub(crate) fn apply_gravity(
    time: Res<Time>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&MovementState, &mut LinearVelocity), With<Player>>,
) {
    let dt = time.delta_secs();

    for (state, mut velocity) in &mut query {
        // No gravity during dash
        if state.is_dashing {
            continue;
        }

        velocity.y -= tuning.gravity * dt;
    }
}

pub(crate) fn update_facing(
    input: Res<MovementInput>,
    mut query: Query<(&mut MovementState, &WallJumpLock), With<Player>>,
) {
    for (mut state, wall_lock) in &mut query {
        // Don't update facing during wall jump lock or dash
        if wall_lock.0 > 0.0 || state.is_dashing {
            continue;
        }

        if input.axis.x > 0.1 {
            state.facing = Facing::Right;
        } else if input.axis.x < -0.1 {
            state.facing = Facing::Left;
        }
    }
}
