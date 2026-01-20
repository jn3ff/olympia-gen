//! Rooms domain: room graph and transition tracking.

use bevy::prelude::*;

use crate::content::Direction;
use crate::rooms::data::RoomSequence;

#[derive(Resource, Debug, Default)]
pub struct RoomGraph {
    pub current_room_id: Option<String>,
    pub rooms_cleared: Vec<String>,
    pub pending_transition: Option<RoomTransition>,
    /// Active room sequence (None when in hub)
    pub active_sequence: Option<RoomSequence>,
}

#[derive(Debug, Clone)]
pub struct RoomTransition {
    pub from_room: Option<String>,
    pub to_room: String,
    pub entry_direction: Direction,
}

/// Cooldown timer to prevent rapid/double transitions between rooms
#[derive(Resource, Debug)]
pub struct TransitionCooldown {
    pub timer: Timer,
}

impl Default for TransitionCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

impl TransitionCooldown {
    pub fn reset(&mut self) {
        self.timer.reset();
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
    }

    pub fn can_transition(&self) -> bool {
        self.timer.remaining_secs() == 0.0
    }
}

/// Tracks that the player is currently within a portal's interaction zone
#[derive(Component, Debug)]
pub struct PlayerInPortalZone {
    pub portal_entity: Entity,
}
