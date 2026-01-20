//! Combat domain: attack direction and boss attack sequencing types.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Light,
    Heavy,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackDirection {
    Up,
    Down,
    #[default]
    Left,
    Right,
}

impl AttackDirection {
    /// Get the offset vector for hitbox placement
    pub fn to_offset(self, distance: f32) -> Vec2 {
        match self {
            AttackDirection::Up => Vec2::new(0.0, distance),
            AttackDirection::Down => Vec2::new(0.0, -distance),
            AttackDirection::Left => Vec2::new(-distance, 0.0),
            AttackDirection::Right => Vec2::new(distance, 0.0),
        }
    }

    /// Get hitbox dimensions (width, height) - elongated in attack direction
    pub fn hitbox_size(self, length: f32, width: f32) -> Vec2 {
        match self {
            AttackDirection::Up | AttackDirection::Down => Vec2::new(width, length),
            AttackDirection::Left | AttackDirection::Right => Vec2::new(length, width),
        }
    }
}

/// A sequence of attack steps that a boss executes
#[derive(Debug, Clone)]
pub struct AttackSequence {
    pub name: String,
    pub steps: Vec<AttackStep>,
    pub cooldown: f32,
}

impl Default for AttackSequence {
    fn default() -> Self {
        Self {
            name: "Basic Attack".to_string(),
            steps: vec![
                AttackStep::Telegraph { duration: 0.3 },
                AttackStep::Hitbox {
                    damage: 15.0,
                    knockback: 300.0,
                    size: Vec2::new(50.0, 50.0),
                    offset: Vec2::new(40.0, 0.0),
                    duration: 0.2,
                },
                AttackStep::Recovery { duration: 0.4 },
            ],
            cooldown: 1.5,
        }
    }
}

/// Individual step in an attack sequence
#[derive(Debug, Clone)]
pub enum AttackStep {
    /// Show warning indicator
    Telegraph { duration: f32 },
    /// Spawn a hitbox
    Hitbox {
        damage: f32,
        knockback: f32,
        size: Vec2,
        offset: Vec2,
        duration: f32,
    },
    /// Move/dash in a direction
    Move {
        direction: Vec2,
        speed: f32,
        duration: f32,
    },
    /// Jump to height
    Jump { height: f32, duration: f32 },
    /// Wait/pause
    Wait { duration: f32 },
    /// Recovery period (vulnerable)
    Recovery { duration: f32 },
    /// Spawn projectile
    Projectile {
        damage: f32,
        speed: f32,
        direction: Vec2,
    },
}
