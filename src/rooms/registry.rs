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
    registry.rooms = vec![
        RoomData {
            id: "room_left_1".to_string(),
            name: "Western Chamber".to_string(),
            exits: vec![Direction::Right, Direction::Up],
            exit_configs: Some(vec![
                // Right exit requires clearing enemies first
                RoomExitConfig::when_cleared(Direction::Right),
                // Up exit is always enabled (escape route)
                RoomExitConfig::always_enabled(Direction::Up),
            ]),
            boss_room: false,
            width: 800.0,
            height: 500.0,
        },
        RoomData {
            id: "room_right_1".to_string(),
            name: "Eastern Hall".to_string(),
            exits: vec![Direction::Left, Direction::Down],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::always_enabled(Direction::Down),
            ]),
            boss_room: false,
            width: 900.0,
            height: 450.0,
        },
        RoomData {
            id: "room_up_1".to_string(),
            name: "Upper Sanctum".to_string(),
            exits: vec![Direction::Down, Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::always_enabled(Direction::Down),
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 1000.0,
            height: 600.0,
        },
        RoomData {
            id: "room_down_1".to_string(),
            name: "Lower Depths".to_string(),
            exits: vec![Direction::Up],
            exit_configs: Some(vec![RoomExitConfig::when_cleared(Direction::Up)]),
            boss_room: false,
            width: 700.0,
            height: 400.0,
        },
        RoomData {
            id: "boss_room".to_string(),
            name: "Champion's Arena".to_string(),
            exits: vec![Direction::Down],
            // Boss room exit requires defeating the boss (no enemies remaining)
            exit_configs: Some(vec![RoomExitConfig::when_cleared(Direction::Down)]),
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
    // Simple mapping: direction determines which room we go to
    let target_id = match direction {
        Direction::Left => "room_left_1",
        Direction::Right => "room_right_1",
        Direction::Up => "room_up_1",
        Direction::Down => "room_down_1",
    };

    registry
        .rooms
        .iter()
        .find(|r| r.id == target_id)
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
