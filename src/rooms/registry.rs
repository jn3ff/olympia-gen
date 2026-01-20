//! Rooms domain: room registry data and lookup helpers.

use bevy::prelude::*;

use crate::content::Direction;
use crate::rooms::data::{RoomData, RoomExitConfig};

/// Available rooms loaded from definitions
#[derive(Resource, Debug, Default)]
pub struct RoomRegistry {
    pub rooms: Vec<RoomData>,
}

pub(crate) fn setup_room_registry(mut registry: ResMut<RoomRegistry>) {
    // Register default rooms - in a full implementation these would come from RON files
    // NOTE: Currently only Left/Right exits are used. Up/Down portal code exists but is disabled.
    registry.rooms = vec![
        RoomData {
            id: "room_combat_1".to_string(),
            name: "Western Chamber".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 800.0,
            height: 500.0,
        },
        RoomData {
            id: "room_combat_2".to_string(),
            name: "Eastern Hall".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 900.0,
            height: 450.0,
        },
        RoomData {
            id: "room_combat_3".to_string(),
            name: "Central Sanctum".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 1000.0,
            height: 600.0,
        },
        RoomData {
            id: "room_combat_4".to_string(),
            name: "Lower Depths".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 700.0,
            height: 400.0,
        },
        RoomData {
            id: "boss_room".to_string(),
            name: "Champion's Arena".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            // Boss room exit requires defeating the boss (no enemies remaining)
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: true,
            width: 1200.0,
            height: 700.0,
        },
    ];
}

pub(crate) fn find_room_for_direction(
    registry: &RoomRegistry,
    direction: Direction,
) -> Option<String> {
    // Find any non-boss room that has an exit in the given direction
    // (Up/Down directions currently disabled - rooms only have Left/Right exits)
    registry
        .rooms
        .iter()
        .find(|r| !r.boss_room && r.exits.contains(&direction))
        .map(|r| r.id.clone())
}

pub(crate) fn find_room_with_entry(
    registry: &RoomRegistry,
    entry_direction: Direction,
    cleared_rooms: &[String],
) -> Option<String> {
    // Find a room that has an exit matching the entry direction
    // (meaning we can enter from that side)
    registry
        .rooms
        .iter()
        .filter(|r| !cleared_rooms.contains(&r.id))
        .find(|r| r.exits.contains(&entry_direction))
        .map(|r| r.id.clone())
}
