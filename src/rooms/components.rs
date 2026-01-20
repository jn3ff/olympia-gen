//! Rooms domain: entity components and markers for room flow.

use bevy::prelude::*;

use crate::content::Direction;

/// Marker for the current room entity
#[derive(Component, Debug)]
pub struct RoomInstance {
    pub id: String,
    pub boss_room: bool,
}

/// An exit point in a room that leads to another room
#[derive(Component, Debug)]
pub struct RoomExit {
    pub direction: Direction,
    pub target_room_id: Option<String>,
}

/// Marker for exit collision trigger
#[derive(Component, Debug)]
pub struct ExitTrigger;

/// Marker for player spawn point
#[derive(Component, Debug)]
pub struct SpawnPoint {
    pub from_direction: Option<Direction>,
}

/// Marker for arena hub entity
#[derive(Component, Debug)]
pub struct ArenaHub;

/// Marker for directional portal in arena
#[derive(Component, Debug)]
pub struct ArenaPortal {
    pub direction: Direction,
}

/// Marker indicating a portal/exit is enabled and can be used for transitions
#[derive(Component, Debug, Default)]
pub struct PortalEnabled;

/// Marker indicating a portal/exit is disabled and cannot be used for transitions
#[derive(Component, Debug, Default)]
pub struct PortalDisabled;

/// Marker for the solid floor/platform under a portal that allows the player to stand on it
#[derive(Component, Debug)]
pub struct PortalFloor;

/// Marker for an invisible solid barrier in an exit gap that prevents the player from
/// falling through while still allowing portal interaction via a separate sensor
#[derive(Component, Debug)]
pub struct PortalBarrier {
    pub direction: Direction,
}

/// Marker for a portal that is blocked (entry direction - player cannot go back)
#[derive(Component, Debug)]
pub struct BlockedPortal;

/// Marker for shop NPC entities in the arena hub
#[derive(Component, Debug)]
pub struct ShopNPC {
    pub shop_id: String,
}

/// Marker for shop interaction zone (sensor collider)
#[derive(Component, Debug)]
pub struct ShopInteractionZone {
    pub shop_id: String,
}

/// Component attached to player when near a shop
#[derive(Component, Debug)]
pub struct NearShop {
    pub shop_id: String,
}

/// Marker component for a room with active encounter logic
#[derive(Component, Debug)]
pub struct EncounterActive;

/// Marker for a room that has already been cleared (prevents duplicate clear events)
#[derive(Component, Debug)]
pub struct RoomWasCleared;

/// Component for animating portal color after player enters through it
#[derive(Component, Debug)]
pub struct PortalExitAnimation {
    pub timer: Timer,
    pub start_color: Color,
    pub end_color: Color,
}

impl PortalExitAnimation {
    pub fn new(start_color: Color, end_color: Color, duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
            start_color,
            end_color,
        }
    }

    pub fn current_color(&self) -> Color {
        let t = self.timer.fraction();
        let a = self.start_color.to_srgba();
        let b = self.end_color.to_srgba();
        Color::srgba(
            a.red + (b.red - a.red) * t,
            a.green + (b.green - a.green) * t,
            a.blue + (b.blue - a.blue) * t,
            a.alpha + (b.alpha - a.alpha) * t,
        )
    }
}
