//! Rooms domain: tests for room data, transitions, and helper logic.

use bevy::prelude::Entity;
use std::time::Duration;

use super::systems::opposite_direction;
use super::{
    ArenaPortal, PlayerInPortalZone, PortalBarrier, PortalCondition, PortalDisabled,
    PortalEnableCondition, PortalEnabled, PortalFloor, RoomData, RoomExit, RoomExitConfig,
    RoomGraph, RoomInstance, RoomRegistry, RoomTransition, TransitionCooldown,
};
use crate::content::Direction;

// -----------------------------------------------------------------------------
// Direction tests
// -----------------------------------------------------------------------------

#[test]
fn test_opposite_direction() {
    assert_eq!(opposite_direction(Direction::Up), Direction::Down);
    assert_eq!(opposite_direction(Direction::Down), Direction::Up);
    assert_eq!(opposite_direction(Direction::Left), Direction::Right);
    assert_eq!(opposite_direction(Direction::Right), Direction::Left);
}

#[test]
fn test_opposite_direction_is_symmetric() {
    for dir in [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ] {
        assert_eq!(opposite_direction(opposite_direction(dir)), dir);
    }
}

// -----------------------------------------------------------------------------
// RoomGraph tests
// -----------------------------------------------------------------------------

#[test]
fn test_room_graph_default_state() {
    let graph = RoomGraph::default();
    assert!(graph.current_room_id.is_none());
    assert!(graph.rooms_cleared.is_empty());
    assert!(graph.pending_transition.is_none());
}

#[test]
fn test_room_graph_transition_tracking() {
    let mut graph = RoomGraph::default();

    graph.pending_transition = Some(RoomTransition {
        from_room: None,
        to_room: "room_1".to_string(),
        entry_direction: Direction::Left,
    });

    assert!(graph.pending_transition.is_some());
    let transition = graph.pending_transition.as_ref().unwrap();
    assert_eq!(transition.to_room, "room_1");
    assert_eq!(transition.entry_direction, Direction::Left);
}

#[test]
fn test_room_graph_cleared_rooms() {
    let mut graph = RoomGraph::default();

    graph.rooms_cleared.push("room_1".to_string());
    graph.rooms_cleared.push("room_2".to_string());

    assert!(graph.rooms_cleared.contains(&"room_1".to_string()));
    assert!(graph.rooms_cleared.contains(&"room_2".to_string()));
    assert!(!graph.rooms_cleared.contains(&"room_3".to_string()));
    assert_eq!(graph.rooms_cleared.len(), 2);
}

// -----------------------------------------------------------------------------
// TransitionCooldown tests
// -----------------------------------------------------------------------------

#[test]
fn test_transition_cooldown_default_blocks_until_expired() {
    let cooldown = TransitionCooldown::default();
    assert!(!cooldown.can_transition());
}

#[test]
fn test_transition_cooldown_reset_blocks_transitions() {
    let mut cooldown = TransitionCooldown::default();
    cooldown.reset();

    assert!(!cooldown.can_transition());
}

#[test]
fn test_transition_cooldown_tick() {
    let mut cooldown = TransitionCooldown::default();
    cooldown.reset();

    cooldown.tick(Duration::from_secs_f32(0.1));

    assert!(!cooldown.can_transition());
}

#[test]
fn test_transition_cooldown_expires() {
    let mut cooldown = TransitionCooldown::default();
    cooldown.reset();

    cooldown.tick(Duration::from_secs_f32(0.5));

    assert!(cooldown.can_transition());
}

// -----------------------------------------------------------------------------
// PortalEnableCondition tests
// -----------------------------------------------------------------------------

#[test]
fn test_portal_enable_condition_default() {
    let condition = PortalEnableCondition::default();
    assert!(matches!(condition, PortalEnableCondition::AlwaysEnabled));
}

#[test]
fn test_room_exit_config_always_enabled() {
    let config = RoomExitConfig::always_enabled(Direction::Up);
    assert_eq!(config.direction, Direction::Up);
    assert!(matches!(
        config.condition,
        PortalEnableCondition::AlwaysEnabled
    ));
}

#[test]
fn test_room_exit_config_when_cleared() {
    let config = RoomExitConfig::when_cleared(Direction::Left);
    assert_eq!(config.direction, Direction::Left);
    assert!(matches!(
        config.condition,
        PortalEnableCondition::NoEnemiesRemaining
    ));
}

// -----------------------------------------------------------------------------
// RoomData tests
// -----------------------------------------------------------------------------

#[test]
fn test_room_data_creation() {
    let room = RoomData {
        id: "test_room".to_string(),
        name: "Test Room".to_string(),
        width: 800.0,
        height: 600.0,
        exits: vec![Direction::Up, Direction::Down],
        exit_configs: None,
        boss_room: false,
    };

    assert_eq!(room.id, "test_room");
    assert_eq!(room.width, 800.0);
    assert_eq!(room.height, 600.0);
    assert!(room.exits.contains(&Direction::Up));
    assert!(room.exits.contains(&Direction::Down));
    assert!(!room.exits.contains(&Direction::Left));
    assert!(!room.boss_room);
}

#[test]
fn test_room_data_boss_room() {
    let room = RoomData {
        id: "boss_arena".to_string(),
        name: "Boss Arena".to_string(),
        width: 1000.0,
        height: 800.0,
        exits: vec![Direction::Down],
        exit_configs: None,
        boss_room: true,
    };

    assert!(room.boss_room);
    assert_eq!(room.exits.len(), 1);
}

#[test]
fn test_room_data_with_exit_configs() {
    let room = RoomData {
        id: "configured_room".to_string(),
        name: "Configured Room".to_string(),
        width: 800.0,
        height: 600.0,
        exits: vec![Direction::Up, Direction::Down],
        exit_configs: Some(vec![
            RoomExitConfig::always_enabled(Direction::Up),
            RoomExitConfig::when_cleared(Direction::Down),
        ]),
        boss_room: false,
    };

    assert!(room.exit_configs.is_some());
    let configs = room.exit_configs.unwrap();
    assert_eq!(configs.len(), 2);
}

// -----------------------------------------------------------------------------
// RoomRegistry tests
// -----------------------------------------------------------------------------

#[test]
fn test_room_registry_default_empty() {
    let registry = RoomRegistry::default();
    assert!(registry.rooms.is_empty());
}

#[test]
fn test_room_registry_find_by_id() {
    let mut registry = RoomRegistry::default();
    registry.rooms.push(RoomData {
        id: "room_1".to_string(),
        name: "Room 1".to_string(),
        width: 800.0,
        height: 600.0,
        exits: vec![Direction::Up],
        exit_configs: None,
        boss_room: false,
    });
    registry.rooms.push(RoomData {
        id: "room_2".to_string(),
        name: "Room 2".to_string(),
        width: 900.0,
        height: 700.0,
        exits: vec![Direction::Left, Direction::Right],
        exit_configs: None,
        boss_room: false,
    });

    let found = registry.rooms.iter().find(|r| r.id == "room_1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Room 1");

    let not_found = registry.rooms.iter().find(|r| r.id == "room_nonexistent");
    assert!(not_found.is_none());
}

// -----------------------------------------------------------------------------
// RoomInstance tests
// -----------------------------------------------------------------------------

#[test]
fn test_room_instance_creation() {
    let instance = RoomInstance {
        id: "current_room".to_string(),
        boss_room: false,
    };

    assert_eq!(instance.id, "current_room");
    assert!(!instance.boss_room);
}

// -----------------------------------------------------------------------------
// Portal component tests
// -----------------------------------------------------------------------------

#[test]
fn test_arena_portal_direction() {
    let portal = ArenaPortal {
        direction: Direction::Up,
    };
    assert_eq!(portal.direction, Direction::Up);
}

#[test]
fn test_room_exit_creation() {
    let exit = RoomExit {
        direction: Direction::Right,
        target_room_id: Some("next_room".to_string()),
    };

    assert_eq!(exit.direction, Direction::Right);
    assert_eq!(exit.target_room_id, Some("next_room".to_string()));
}

#[test]
fn test_room_exit_no_target() {
    let exit = RoomExit {
        direction: Direction::Left,
        target_room_id: None,
    };

    assert!(exit.target_room_id.is_none());
}

// -----------------------------------------------------------------------------
// PlayerInPortalZone tests
// -----------------------------------------------------------------------------

#[test]
fn test_player_in_portal_zone_stores_entity() {
    let test_entity = Entity::from_bits(42);
    let zone = PlayerInPortalZone {
        portal_entity: test_entity,
    };

    assert_eq!(zone.portal_entity, test_entity);
}

// -----------------------------------------------------------------------------
// RoomTransition tests
// -----------------------------------------------------------------------------

#[test]
fn test_room_transition_from_arena() {
    let transition = RoomTransition {
        from_room: None,
        to_room: "first_room".to_string(),
        entry_direction: Direction::Left,
    };

    assert!(transition.from_room.is_none());
    assert_eq!(transition.to_room, "first_room");
    assert_eq!(transition.entry_direction, Direction::Left);
}

#[test]
fn test_room_transition_between_rooms() {
    let transition = RoomTransition {
        from_room: Some("room_a".to_string()),
        to_room: "room_b".to_string(),
        entry_direction: Direction::Up,
    };

    assert_eq!(transition.from_room, Some("room_a".to_string()));
    assert_eq!(transition.to_room, "room_b");
}

// -----------------------------------------------------------------------------
// PortalCondition tests
// -----------------------------------------------------------------------------

#[test]
fn test_portal_condition_new() {
    let condition = PortalCondition::new(PortalEnableCondition::NoEnemiesRemaining);
    assert!(matches!(
        condition.condition,
        PortalEnableCondition::NoEnemiesRemaining
    ));
}

// -----------------------------------------------------------------------------
// Integration-style tests for portal zone logic
// -----------------------------------------------------------------------------

#[test]
fn test_portal_enabled_and_disabled_markers() {
    let _enabled = PortalEnabled;
    let _disabled = PortalDisabled;
}

#[test]
fn test_room_exit_config_builder_pattern() {
    let config = RoomExitConfig::new(Direction::Up)
        .with_condition(PortalEnableCondition::NoEnemiesRemaining);

    assert_eq!(config.direction, Direction::Up);
    assert!(matches!(
        config.condition,
        PortalEnableCondition::NoEnemiesRemaining
    ));
}

// -----------------------------------------------------------------------------
// PortalBarrier tests
// -----------------------------------------------------------------------------

#[test]
fn test_portal_barrier_creation() {
    let barrier = PortalBarrier {
        direction: Direction::Up,
    };
    assert_eq!(barrier.direction, Direction::Up);
}

#[test]
fn test_portal_barrier_all_directions() {
    for dir in [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ] {
        let barrier = PortalBarrier { direction: dir };
        assert_eq!(barrier.direction, dir);
    }
}

#[test]
fn test_portal_floor_exists() {
    let _floor = PortalFloor;
}
