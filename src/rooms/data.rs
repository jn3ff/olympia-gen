//! Rooms domain: room data definitions and exit rules.

use bevy::prelude::*;

use crate::content::Direction;

/// What happens when a room sequence ends
#[derive(Debug, Clone, Default)]
pub enum SequenceCulmination {
    /// Return directly to hub
    #[default]
    ReturnToHub,
    /// Trigger a boss room before hub
    BossRoom { boss_room_id: String },
}

/// A pre-planned sequence of rooms to traverse before returning to hub
#[derive(Debug, Clone)]
pub struct RoomSequence {
    /// Ordered list of room IDs to traverse
    pub rooms: Vec<String>,
    /// Current index in the sequence (0 = first room)
    pub current_index: usize,
    /// The direction the player entered the current room from
    pub entry_direction: Direction,
    /// What happens at the end of the sequence
    pub culmination: SequenceCulmination,
}

impl RoomSequence {
    pub fn new(rooms: Vec<String>, entry_direction: Direction) -> Self {
        Self {
            rooms,
            current_index: 0,
            entry_direction,
            culmination: SequenceCulmination::default(),
        }
    }

    /// Get the current room ID
    pub fn current_room(&self) -> Option<&String> {
        self.rooms.get(self.current_index)
    }

    /// Check if we're at the final room
    pub fn is_final_room(&self) -> bool {
        self.current_index >= self.rooms.len().saturating_sub(1)
    }

    /// Advance to next room, returns the next room ID or None if sequence complete
    pub fn advance(&mut self) -> Option<&String> {
        if self.current_index + 1 < self.rooms.len() {
            self.current_index += 1;
            Some(&self.rooms[self.current_index])
        } else {
            None
        }
    }

    /// Get the portal direction that should be blocked (the one player entered from)
    pub fn blocked_exit_direction(&self) -> Direction {
        self.entry_direction
    }
}

/// Condition that determines when a portal/exit becomes enabled
#[derive(Debug, Clone, PartialEq)]
pub enum PortalEnableCondition {
    /// Portal is always enabled from the start
    AlwaysEnabled,
    /// Portal enables when no enemies remain in the room
    NoEnemiesRemaining,
    /// Portal is never enabled (blocked entry portal)
    Never,
    /// All sub-conditions must be met
    All(Vec<PortalEnableCondition>),
    /// Any sub-condition must be met
    Any(Vec<PortalEnableCondition>),
}

impl Default for PortalEnableCondition {
    fn default() -> Self {
        Self::AlwaysEnabled
    }
}

/// Configuration for a single exit in a room
#[derive(Debug, Clone)]
pub struct RoomExitConfig {
    pub direction: Direction,
    pub condition: PortalEnableCondition,
}

impl RoomExitConfig {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            condition: PortalEnableCondition::AlwaysEnabled,
        }
    }

    pub fn with_condition(mut self, condition: PortalEnableCondition) -> Self {
        self.condition = condition;
        self
    }

    pub fn always_enabled(direction: Direction) -> Self {
        Self::new(direction)
    }

    pub fn when_cleared(direction: Direction) -> Self {
        Self {
            direction,
            condition: PortalEnableCondition::NoEnemiesRemaining,
        }
    }
}

/// Component that holds the enable condition for a portal/exit
#[derive(Component, Debug, Clone)]
pub struct PortalCondition {
    pub condition: PortalEnableCondition,
}

impl PortalCondition {
    pub fn new(condition: PortalEnableCondition) -> Self {
        Self { condition }
    }
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: String,
    #[allow(dead_code)]
    pub name: String,
    pub exits: Vec<Direction>,
    /// Optional per-exit configuration. If provided, overrides the default condition for exits.
    /// Exits listed in `exits` but not in `exit_configs` use AlwaysEnabled by default.
    pub exit_configs: Option<Vec<RoomExitConfig>>,
    pub boss_room: bool,
    pub width: f32,
    pub height: f32,
}

impl Default for RoomData {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Room".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: None,
            boss_room: false,
            width: 800.0,
            height: 500.0,
        }
    }
}

impl RoomData {
    /// Get the condition for a specific exit direction.
    /// Returns the configured condition if exit_configs is set, otherwise defaults based on room type.
    pub fn get_exit_condition(&self, direction: Direction) -> PortalEnableCondition {
        // Check if we have explicit exit configs
        if let Some(configs) = &self.exit_configs {
            if let Some(config) = configs.iter().find(|c| c.direction == direction) {
                return config.condition.clone();
            }
        }

        // Default behavior: boss rooms require clearing, regular rooms are always enabled
        if self.boss_room {
            PortalEnableCondition::NoEnemiesRemaining
        } else {
            PortalEnableCondition::AlwaysEnabled
        }
    }
}
