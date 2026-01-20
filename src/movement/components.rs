//! Movement domain: components and physics layers for locomotion.

use avian2d::prelude::*;
use bevy::prelude::*;

/// Physics layers for collision filtering
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    /// Ground surfaces (floors, platforms)
    Ground,
    /// Wall surfaces
    Wall,
    /// Player character
    Player,
    /// Enemy characters
    Enemy,
    /// Sensors (portals, triggers) - should not block movement
    Sensor,
    /// Player hitboxes (damage enemies)
    PlayerHitbox,
    /// Enemy hitboxes (damage player)
    EnemyHitbox,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct MovementState {
    pub on_ground: bool,
    pub on_wall: WallContact,
    pub facing: Facing,
    pub coyote_timer: f32,
    pub wall_coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub dash_timer: f32,
    pub dash_cooldown_timer: f32,
    pub is_dashing: bool,
    pub dash_direction: f32,
    pub air_jumps_remaining: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallContact {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

/// Marker for ground colliders
#[derive(Component, Debug)]
pub struct Ground;

/// Marker for wall colliders
#[derive(Component, Debug)]
pub struct Wall;

/// Timer to prevent immediate air control after wall jump
#[derive(Component, Debug, Default)]
pub struct WallJumpLock(pub f32);
