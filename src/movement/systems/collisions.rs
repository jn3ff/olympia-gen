//! Movement domain: ground and wall detection systems.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::movement::{GameLayer, MovementState, MovementTuning, Player, WallContact};

pub(crate) fn detect_ground(
    spatial_query: SpatialQuery,
    tuning: Res<MovementTuning>,
    mut query: Query<(&Transform, &Collider, &mut MovementState), With<Player>>,
) {
    // Filter to only hit Ground layer entities (not enemies, portals, etc.)
    let ground_filter = SpatialQueryFilter::from_mask(GameLayer::Ground);

    for (transform, collider, mut state) in &mut query {
        let was_on_ground = state.on_ground;

        // Cast a short ray downward from the player's feet
        let player_half_height = match collider.shape_scaled().as_cuboid() {
            Some(c) => c.half_extents.y,
            None => 24.0,
        };

        let ray_origin = transform.translation.truncate() - Vec2::new(0.0, player_half_height);
        let ray_direction = Dir2::NEG_Y;
        let ray_distance = 4.0;

        let hit = spatial_query.cast_ray(
            ray_origin,
            ray_direction,
            ray_distance,
            true,
            &ground_filter,
        );

        state.on_ground = hit.is_some();

        // Reset coyote timer and air jumps when landing
        if state.on_ground && !was_on_ground {
            state.coyote_timer = 0.0;
            state.air_jumps_remaining = tuning.max_air_jumps;
            debug!(
                "Landed: on_ground={}, air_jumps_remaining={}",
                state.on_ground, state.air_jumps_remaining
            );
        } else if !state.on_ground && was_on_ground {
            debug!(
                "Left ground: on_ground={}, air_jumps_remaining={}",
                state.on_ground, state.air_jumps_remaining
            );
        }
    }
}

pub(crate) fn detect_walls(
    spatial_query: SpatialQuery,
    mut query: Query<(&Transform, &Collider, &mut MovementState), With<Player>>,
) {
    // Filter to only hit Wall layer entities
    let wall_filter = SpatialQueryFilter::from_mask(GameLayer::Wall);

    for (transform, collider, mut state) in &mut query {
        let was_on_wall = state.on_wall;

        let player_half_width = match collider.shape_scaled().as_cuboid() {
            Some(c) => c.half_extents.x,
            None => 12.0,
        };

        let origin = transform.translation.truncate();
        let ray_distance = 4.0;

        // Check left
        let left_hit = spatial_query.cast_ray(
            origin,
            Dir2::NEG_X,
            player_half_width + ray_distance,
            true,
            &wall_filter,
        );

        // Check right
        let right_hit = spatial_query.cast_ray(
            origin,
            Dir2::X,
            player_half_width + ray_distance,
            true,
            &wall_filter,
        );

        state.on_wall = match (left_hit.is_some(), right_hit.is_some()) {
            (true, false) => WallContact::Left,
            (false, true) => WallContact::Right,
            _ => WallContact::None,
        };

        // Reset wall coyote timer when touching wall
        if state.on_wall != WallContact::None && was_on_wall == WallContact::None {
            state.wall_coyote_timer = 0.0;
        }
    }
}
