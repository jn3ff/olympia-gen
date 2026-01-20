//! Rooms domain: room flow systems and transition logic.

use avian2d::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::combat::{ArenaLock, BossDefeatedEvent as CombatBossDefeatedEvent, Enemy};
use crate::content::{ContentRegistry, Direction, GameplayDefaults, RoomType};
use crate::core::{RunConfig, RunState, RunVictoryEvent, SegmentCompletedEvent, SegmentProgress};
use crate::encounters::{EncounterCompletedEvent, EncounterStartedEvent};
use crate::movement::{Ground, Player, Wall};
use crate::rewards::{CoinGainedEvent, CoinSource, OpenShopEvent, PlayerBuild};
use crate::rooms::components::{
    ArenaPortal, BlockedPortal, EncounterActive, NearShop, PortalDisabled, PortalEnabled,
    PortalExitAnimation, PortalFloor, RoomExit, RoomInstance, RoomWasCleared, ShopInteractionZone,
};
use crate::rooms::data::{
    PortalCondition, PortalEnableCondition, RoomSequence, SequenceCulmination,
};
use crate::rooms::events::{ExitRoomEvent, RoomClearedEvent};
use crate::rooms::graph::{PlayerInPortalZone, RoomGraph, RoomTransition, TransitionCooldown};
use crate::rooms::registry::{RoomRegistry, find_room_for_direction, find_room_with_entry};
use crate::rooms::ui::{ArenaSegmentInfo, PortalTooltipUI, ShopNameLabel, ShopTooltipUI};

pub(crate) fn reset_transition_cooldown(mut cooldown: ResMut<TransitionCooldown>) {
    cooldown.reset();
    info!("[TRANSITION] Cooldown reset on state enter");
}

pub(crate) fn tick_transition_cooldown(mut cooldown: ResMut<TransitionCooldown>, time: Res<Time>) {
    cooldown.tick(time.delta());
}

pub(crate) fn drain_stale_collision_events(
    mut collision_start_events: MessageReader<CollisionStart>,
) {
    let count = collision_start_events.read().count();
    if count > 0 {
        info!(
            "[TRANSITION] Drained {} stale collision events on state enter",
            count
        );
    }
}

/// Populate the room pool for the current segment from ContentRegistry.
/// Called when entering the Arena hub if pools need initialization.
/// Uses ContentRegistry rooms if available, falls back with error!() log otherwise.
pub(crate) fn populate_segment_room_pool(
    content_registry: Option<Res<ContentRegistry>>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    mut segment_progress: ResMut<SegmentProgress>,
    run_config: Res<RunConfig>,
) {
    // Skip if pools are already initialized
    if !segment_progress.needs_pool_init {
        return;
    }

    let defaults = gameplay_defaults.map(|d| d.into_inner());
    let rooms_needed = defaults
        .as_ref()
        .map(|d| d.segment_defaults.rooms_per_segment)
        .unwrap_or(5);
    let bosses_needed = defaults
        .as_ref()
        .map(|d| d.segment_defaults.bosses_per_segment)
        .unwrap_or(2);

    // Try to use ContentRegistry if available
    if let Some(registry) = content_registry {
        // Select a biome for this segment (cycle through available biomes)
        let biome_ids: Vec<&String> = registry.biomes.keys().collect();
        let biome_id = if biome_ids.is_empty() {
            None
        } else {
            let biome_index = run_config.segment_index as usize % biome_ids.len();
            Some(biome_ids[biome_index].clone())
        };
        segment_progress.current_biome_id = biome_id.clone();

        // Collect rooms from ContentRegistry
        let mut combat_rooms: Vec<String> = registry
            .rooms
            .values()
            .filter(|r| {
                // Filter by biome if we have one selected
                if let Some(ref biome) = biome_id {
                    if !r.biome_id.is_empty() && &r.biome_id != biome {
                        return false;
                    }
                }
                // Include Combat and Traversal rooms
                matches!(r.room_type, RoomType::Combat | RoomType::Traversal)
            })
            .map(|r| r.id.clone())
            .collect();

        let mut boss_rooms: Vec<String> = registry
            .rooms
            .values()
            .filter(|r| {
                if let Some(ref biome) = biome_id {
                    if !r.biome_id.is_empty() && &r.biome_id != biome {
                        return false;
                    }
                }
                r.room_type == RoomType::Boss
            })
            .map(|r| r.id.clone())
            .collect();

        // Shuffle and select the needed number
        let mut rng = rand::rng();
        combat_rooms.shuffle(&mut rng);
        boss_rooms.shuffle(&mut rng);

        segment_progress.room_pool = combat_rooms
            .into_iter()
            .take(rooms_needed as usize)
            .collect();
        segment_progress.boss_room_pool = boss_rooms
            .into_iter()
            .take(bosses_needed as usize)
            .collect();

        // Check if we got rooms from ContentRegistry
        if !segment_progress.room_pool.is_empty() && !segment_progress.boss_room_pool.is_empty() {
            segment_progress.needs_pool_init = false;
            info!(
                "Segment {} room pool from ContentRegistry: {} rooms, {} bosses (biome: {:?})",
                run_config.segment_index,
                segment_progress.room_pool.len(),
                segment_progress.boss_room_pool.len(),
                segment_progress.current_biome_id
            );
            return;
        }
    }

    // Fallback: ContentRegistry didn't have usable rooms
    error!(
        "No rooms found in ContentRegistry for segment {}! Using hardcoded fallback rooms. \
         Add rooms with room_type Combat/Traversal/Boss to assets/data/rooms.ron",
        run_config.segment_index
    );

    segment_progress.room_pool = vec![
        "room_left_1".to_string(),
        "room_right_1".to_string(),
        "room_up_1".to_string(),
        "room_down_1".to_string(),
    ];
    segment_progress.boss_room_pool = vec!["boss_room".to_string()];
    segment_progress.current_biome_id = None;
    segment_progress.needs_pool_init = false;

    info!(
        "Segment {} using fallback room pool: {} rooms, {} bosses",
        run_config.segment_index,
        segment_progress.room_pool.len(),
        segment_progress.boss_room_pool.len()
    );
}

pub(crate) fn cleanup_arena(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<crate::rooms::components::ArenaHub>,
            With<ArenaPortal>,
            With<Ground>,
            With<Wall>,
            With<ArenaSegmentInfo>,
            With<crate::rooms::components::ShopNPC>,
            With<ShopInteractionZone>,
            With<ShopTooltipUI>,
            With<ShopNameLabel>,
            With<PortalTooltipUI>,
        )>,
    >,
    mut player_query: Query<Entity, With<Player>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove PlayerInPortalZone and NearShop from player when leaving arena
    for player_entity in &mut player_query {
        commands
            .entity(player_entity)
            .remove::<PlayerInPortalZone>()
            .remove::<NearShop>();
    }
}

/// Tracks when the player enters/exits arena portal interaction zones.
/// Adds PlayerInPortalZone to player when touching an enabled arena portal sensor.
pub(crate) fn track_arena_portal_zone(
    mut commands: Commands,
    mut collision_start_events: MessageReader<CollisionStart>,
    mut collision_end_events: MessageReader<CollisionEnd>,
    portal_query: Query<(Entity, &ArenaPortal, Option<&PortalEnabled>)>,
    player_query: Query<Entity, With<Player>>,
    player_zone_query: Query<&PlayerInPortalZone, With<Player>>,
) {
    let Some(player_entity) = player_query.iter().next() else {
        // Consume events if no player
        for _ in collision_start_events.read() {}
        for _ in collision_end_events.read() {}
        return;
    };

    // Handle collision starts - player enters portal zone
    for event in collision_start_events.read() {
        let (portal_entity, other) = if portal_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if portal_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        if let Ok((entity, portal, portal_enabled)) = portal_query.get(portal_entity) {
            // Only track enabled portals
            if portal_enabled.is_some() {
                info!(
                    "[PORTAL] Player entered arena portal zone {:?}",
                    portal.direction
                );
                commands.entity(player_entity).insert(PlayerInPortalZone {
                    portal_entity: entity,
                });
            }
        }
    }

    // Handle collision ends - player exits portal zone
    for event in collision_end_events.read() {
        let (portal_entity, other) = if portal_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if portal_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        // Check if player is leaving the portal they're currently in
        if let Ok(player_zone) = player_zone_query.get(player_entity) {
            if player_zone.portal_entity == portal_entity {
                info!("[PORTAL] Player exited arena portal zone");
                commands
                    .entity(player_entity)
                    .remove::<PlayerInPortalZone>();
            }
        }
    }
}

/// Confirms arena portal entry when player presses E while in an arena portal zone.
pub(crate) fn confirm_arena_portal_entry(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerInPortalZone, With<Player>>,
    portal_query: Query<(&ArenaPortal, Option<&PortalEnabled>)>,
    cooldown: Res<TransitionCooldown>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    segment_progress: Res<SegmentProgress>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    // Check if player pressed E
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    // Check cooldown
    if !cooldown.can_transition() {
        return;
    }

    // Check if player is in a portal zone
    let Ok(player_zone) = player_query.single() else {
        return;
    };

    // Verify the portal is still enabled and get its direction
    if let Ok((portal, portal_enabled)) = portal_query.get(player_zone.portal_entity) {
        if portal_enabled.is_some() {
            let entry_dir = opposite_direction(portal.direction);

            // Generate room sequence
            let sequence = generate_room_sequence(
                &registry,
                &segment_progress,
                gameplay_defaults.as_deref(),
                entry_dir,
            );

            if let Some(first_room) = sequence.current_room().cloned() {
                info!(
                    "[SEQUENCE] Generated room sequence with {} rooms, culmination: {:?}",
                    sequence.rooms.len(),
                    sequence.culmination
                );
                info!(
                    "[TRANSITION] Player confirmed arena portal {:?} with E key -> starting sequence at '{}'",
                    portal.direction, first_room
                );
                room_graph.pending_transition = Some(RoomTransition {
                    from_room: None,
                    to_room: first_room,
                    entry_direction: entry_dir,
                });
                room_graph.active_sequence = Some(sequence);
                next_state.set(RunState::Room);
            } else {
                // Fallback to old behavior if no rooms available
                let target_room = find_room_for_direction(&registry, portal.direction);
                if let Some(room_id) = target_room {
                    info!(
                        "[TRANSITION] Fallback: Player confirmed arena portal {:?} -> room '{}'",
                        portal.direction, room_id
                    );
                    room_graph.pending_transition = Some(RoomTransition {
                        from_room: None,
                        to_room: room_id.clone(),
                        entry_direction: entry_dir,
                    });
                    next_state.set(RunState::Room);
                }
            }
        }
    }
}

/// Generate a sequence of rooms for traversing from the arena
fn generate_room_sequence(
    registry: &RoomRegistry,
    segment_progress: &SegmentProgress,
    defaults: Option<&GameplayDefaults>,
    entry_direction: Direction,
) -> RoomSequence {
    // Default to 3 rooms per sequence, configurable later
    let rooms_per_sequence: usize = 3;

    // Get rooms per segment to determine culmination
    let rooms_per_segment = defaults
        .map(|d| d.segment_defaults.rooms_per_segment)
        .unwrap_or(5);

    let mut rng = rand::rng();

    // Helper to check if a room has the required entry exit
    let room_has_entry = |room_id: &str| -> bool {
        registry
            .rooms
            .iter()
            .find(|r| r.id == room_id)
            .map(|r| r.exits.contains(&entry_direction))
            .unwrap_or(false)
    };

    // Select rooms from segment pool (or registry as fallback)
    // IMPORTANT: Only select rooms that have an exit matching entry_direction,
    // otherwise the player will spawn facing a solid wall!
    let mut available_rooms: Vec<String> = if !segment_progress.room_pool.is_empty() {
        segment_progress
            .room_pool
            .iter()
            .filter(|id| {
                !segment_progress
                    .encountered_significant_enemies
                    .contains(*id)
                    && room_has_entry(id)
            })
            .cloned()
            .collect()
    } else {
        registry
            .rooms
            .iter()
            .filter(|r| !r.boss_room && r.exits.contains(&entry_direction))
            .map(|r| r.id.clone())
            .collect()
    };

    available_rooms.shuffle(&mut rng);
    let selected_rooms: Vec<String> = available_rooms
        .into_iter()
        .take(rooms_per_sequence)
        .collect();

    // Determine culmination based on segment progress
    let rooms_after_sequence =
        segment_progress.rooms_cleared_this_segment + selected_rooms.len() as u32;
    let culmination = if rooms_after_sequence >= rooms_per_segment {
        // Segment complete after this sequence - trigger boss if available
        if let Some(boss_id) = segment_progress.boss_room_pool.first() {
            SequenceCulmination::BossRoom {
                boss_room_id: boss_id.clone(),
            }
        } else {
            SequenceCulmination::ReturnToHub
        }
    } else {
        SequenceCulmination::ReturnToHub
    };

    RoomSequence {
        rooms: selected_rooms,
        current_index: 0,
        entry_direction,
        culmination,
    }
}

pub(crate) fn cleanup_room(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<RoomInstance>,
            With<RoomExit>,
            With<Ground>,
            With<Wall>,
            With<Enemy>,
            With<ArenaLock>,
            With<PortalFloor>,
            With<crate::rooms::components::PortalBarrier>,
            With<PortalTooltipUI>,
        )>,
    >,
    mut player_query: Query<Entity, With<Player>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove PlayerInPortalZone from player when leaving room
    for player_entity in &mut player_query {
        commands
            .entity(player_entity)
            .remove::<PlayerInPortalZone>();
    }

    // NOTE: Don't consume pending_transition here - spawn_current_room needs it!
    // The transition is consumed after spawn_current_room uses it.
}

/// Tracks when the player enters/exits portal interaction zones.
/// Adds PlayerInPortalZone to player when touching an enabled portal sensor.
pub(crate) fn track_player_portal_zone(
    mut commands: Commands,
    mut collision_start_events: MessageReader<CollisionStart>,
    mut collision_end_events: MessageReader<CollisionEnd>,
    exit_query: Query<(Entity, &RoomExit, Option<&PortalEnabled>)>,
    player_query: Query<Entity, With<Player>>,
    player_zone_query: Query<&PlayerInPortalZone, With<Player>>,
    arena_lock_query: Query<Entity, With<ArenaLock>>,
) {
    let Some(player_entity) = player_query.iter().next() else {
        // Consume events if no player
        for _ in collision_start_events.read() {}
        for _ in collision_end_events.read() {}
        return;
    };

    // If arena is locked (boss fight), don't track portal zones
    if !arena_lock_query.is_empty() {
        return;
    }

    // Handle collision starts - player enters portal zone
    for event in collision_start_events.read() {
        let (exit_entity, other) = if exit_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if exit_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        if let Ok((entity, exit, portal_enabled)) = exit_query.get(exit_entity) {
            // Only track enabled exits
            if portal_enabled.is_some() {
                info!(
                    "[PORTAL] Player entered room exit zone {:?}",
                    exit.direction
                );
                commands.entity(player_entity).insert(PlayerInPortalZone {
                    portal_entity: entity,
                });
            }
        }
    }

    // Handle collision ends - player exits portal zone
    for event in collision_end_events.read() {
        let (exit_entity, other) = if exit_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if exit_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        // Check if player is leaving the portal they're currently in
        if let Ok(player_zone) = player_zone_query.get(player_entity) {
            if player_zone.portal_entity == exit_entity {
                info!("[PORTAL] Player exited room exit zone");
                commands
                    .entity(player_entity)
                    .remove::<PlayerInPortalZone>();
            }
        }
    }
}

/// Confirms portal entry when player presses E while in a portal zone.
pub(crate) fn confirm_portal_entry(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerInPortalZone, With<Player>>,
    exit_query: Query<(&RoomExit, Option<&PortalEnabled>)>,
    cooldown: Res<TransitionCooldown>,
    mut exit_events: MessageWriter<ExitRoomEvent>,
) {
    // Check if player pressed E
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    // Check cooldown
    if !cooldown.can_transition() {
        return;
    }

    // Check if player is in a portal zone
    let Ok(player_zone) = player_query.single() else {
        return;
    };

    // Verify the portal is still enabled and get its direction
    if let Ok((exit, portal_enabled)) = exit_query.get(player_zone.portal_entity) {
        if portal_enabled.is_some() {
            info!(
                "[TRANSITION] Player confirmed exit {:?} with E key",
                exit.direction
            );
            exit_events.write(ExitRoomEvent {
                direction: exit.direction,
            });
        }
    }
}

pub(crate) fn handle_boss_defeated(
    mut commands: Commands,
    mut boss_defeated_events: MessageReader<CombatBossDefeatedEvent>,
    arena_lock_query: Query<Entity, With<ArenaLock>>,
    mut segment_progress: ResMut<SegmentProgress>,
) {
    for _event in boss_defeated_events.read() {
        // Track boss defeat for segment and run progress
        segment_progress.bosses_defeated_this_segment += 1;
        segment_progress.total_bosses_defeated += 1;

        info!(
            "Boss defeated! Segment: {}, Total: {}",
            segment_progress.bosses_defeated_this_segment, segment_progress.total_bosses_defeated
        );

        // Unlock the arena when boss is defeated
        for lock_entity in arena_lock_query.iter() {
            commands.entity(lock_entity).despawn();
        }
    }
}

pub(crate) fn process_room_transitions(
    mut exit_events: MessageReader<ExitRoomEvent>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    _segment_progress: ResMut<SegmentProgress>,
    room_instance_query: Query<&RoomInstance>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    for event in exit_events.read() {
        let current_room_id = room_graph.current_room_id.clone();

        // Update room sequence if active
        if let Some(sequence) = room_graph.active_sequence.as_mut() {
            if sequence.is_final_room() {
                match &sequence.culmination {
                    SequenceCulmination::BossRoom { boss_room_id } => {
                        info!(
                            "[SEQUENCE] Final room cleared, transitioning to boss room '{}'",
                            boss_room_id
                        );
                        room_graph.pending_transition = Some(RoomTransition {
                            from_room: current_room_id.clone(),
                            to_room: boss_room_id.clone(),
                            entry_direction: opposite_direction(event.direction),
                        });
                        // Clear sequence after boss room
                        room_graph.active_sequence = None;
                        next_state.set(RunState::Room);
                        continue;
                    }
                    SequenceCulmination::ReturnToHub => {
                        info!("[SEQUENCE] Final room cleared, returning to hub");
                        room_graph.active_sequence = None;
                        next_state.set(RunState::Arena);
                        continue;
                    }
                }
            }

            // Advance to next room in sequence
            if let Some(next_room) = sequence.advance() {
                info!(
                    "[SEQUENCE] Advancing to next room '{}' in sequence",
                    next_room
                );
                room_graph.pending_transition = Some(RoomTransition {
                    from_room: current_room_id.clone(),
                    to_room: next_room.clone(),
                    entry_direction: opposite_direction(event.direction),
                });
                next_state.set(RunState::Room);
                continue;
            }
        }

        // Standard room-to-room transition logic
        let entry_dir = opposite_direction(event.direction);

        // Find a new room to enter based on direction, avoiding cleared rooms
        let target_room = find_room_with_entry(&registry, entry_dir, &room_graph.rooms_cleared);

        if let Some(room_id) = target_room {
            info!(
                "[TRANSITION] Player exiting {:?} to room '{}'",
                event.direction, room_id
            );
            room_graph.pending_transition = Some(RoomTransition {
                from_room: current_room_id.clone(),
                to_room: room_id,
                entry_direction: entry_dir,
            });
            next_state.set(RunState::Room);
        } else {
            info!(
                "[TRANSITION] No available room for direction {:?}, returning to hub",
                event.direction
            );
            next_state.set(RunState::Arena);
        }

        // Track room as cleared
        if let Some(room_instance) = room_instance_query.iter().next() {
            if !room_graph.rooms_cleared.contains(&room_instance.id) {
                room_graph.rooms_cleared.push(room_instance.id.clone());
            }
        }
    }
}

pub(crate) fn check_segment_completion(
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    run_config: Res<RunConfig>,
    mut segment_events: MessageWriter<SegmentCompletedEvent>,
    mut victory_events: MessageWriter<RunVictoryEvent>,
    segment_progress: Res<SegmentProgress>,
) {
    let defaults = gameplay_defaults.map(|d| d.into_inner());
    let rooms_required = defaults
        .as_ref()
        .map(|d| d.segment_defaults.rooms_per_segment)
        .unwrap_or(5);
    let bosses_required = defaults
        .as_ref()
        .map(|d| d.segment_defaults.bosses_per_segment)
        .unwrap_or(2);
    let boss_target = defaults
        .as_ref()
        .map(|d| d.win_condition.boss_target)
        .unwrap_or(5);

    let segment_complete = segment_progress.rooms_cleared_this_segment >= rooms_required
        && segment_progress.bosses_defeated_this_segment >= bosses_required;

    if segment_complete {
        if segment_progress.total_bosses_defeated >= boss_target {
            info!(
                "Victory condition met! {} bosses defeated.",
                segment_progress.total_bosses_defeated
            );
            victory_events.write(RunVictoryEvent {
                total_bosses_defeated: segment_progress.total_bosses_defeated,
            });
        } else {
            info!(
                "Segment {} complete! Rooms: {}/{}, Bosses: {}/{}",
                run_config.segment_index,
                segment_progress.rooms_cleared_this_segment,
                rooms_required,
                segment_progress.bosses_defeated_this_segment,
                bosses_required
            );
            segment_events.write(SegmentCompletedEvent {
                segment_index: run_config.segment_index,
            });
        }
    }
}

pub(crate) fn evaluate_portal_conditions(
    mut commands: Commands,
    room_instance_query: Query<&RoomInstance>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut exit_query: Query<
        (
            Entity,
            &PortalCondition,
            &RoomExit,
            &mut Sprite,
            Option<&PortalDisabled>,
        ),
        (
            With<RoomExit>,
            Without<BlockedPortal>,
            Without<PortalEnabled>,
        ),
    >,
) {
    let Some(room_instance) = room_instance_query.iter().next() else {
        return;
    };

    let enemy_count = enemy_query.iter().count();

    let exit_enabled_color = Color::srgb(0.3, 0.7, 0.4);
    let boss_exit_enabled_color = Color::srgb(0.7, 0.3, 0.3);

    for (entity, condition, exit, mut sprite, is_disabled) in exit_query.iter_mut() {
        if is_disabled.is_none() {
            continue;
        }

        let should_enable = evaluate_condition(&condition.condition, enemy_count);
        if should_enable {
            commands.entity(entity).remove::<PortalDisabled>();
            commands.entity(entity).insert(PortalEnabled);

            sprite.color = if room_instance.boss_room {
                boss_exit_enabled_color
            } else {
                exit_enabled_color
            };

            info!(
                "[PORTAL] Enabled exit {:?} (condition: {:?}, enemies remaining: {})",
                exit.direction, condition.condition, enemy_count
            );
        }
    }
}

fn evaluate_condition(condition: &PortalEnableCondition, enemy_count: usize) -> bool {
    match condition {
        PortalEnableCondition::AlwaysEnabled => true,
        PortalEnableCondition::NoEnemiesRemaining => enemy_count == 0,
        PortalEnableCondition::Never => false,
        PortalEnableCondition::All(conditions) => conditions
            .iter()
            .all(|c| evaluate_condition(c, enemy_count)),
        PortalEnableCondition::Any(conditions) => conditions
            .iter()
            .any(|c| evaluate_condition(c, enemy_count)),
    }
}

pub(crate) fn opposite_direction(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

/// Award coins when a room is cleared
pub(crate) fn handle_room_clear_coins(
    mut room_events: MessageReader<RoomClearedEvent>,
    run_config: Res<RunConfig>,
    mut coin_events: MessageWriter<CoinGainedEvent>,
) {
    for _event in room_events.read() {
        // Base bonus + scaling with segment
        let base_bonus = 15u32;
        let segment_bonus = run_config.segment_index as u32 * 3;
        let total = base_bonus + segment_bonus;

        coin_events.write(CoinGainedEvent {
            amount: total,
            source: CoinSource::RoomReward,
        });
    }
}

/// Emit EncounterStartedEvent when entering a room with enemies.
/// This runs after spawn_current_room to trigger encounter tag selection.
pub(crate) fn emit_encounter_started(
    room_instance_query: Query<(Entity, &RoomInstance), Without<EncounterActive>>,
    enemy_query: Query<Entity, With<Enemy>>,
    player_build: Option<Res<PlayerBuild>>,
    mut commands: Commands,
    mut encounter_events: MessageWriter<EncounterStartedEvent>,
) {
    // Only emit if we have a room instance and it hasn't been marked as active yet
    for (room_entity, room_instance) in room_instance_query.iter() {
        // Only start encounter if there are enemies in the room
        let enemy_count = enemy_query.iter().count();
        if enemy_count == 0 {
            continue;
        }

        // Mark encounter as active to prevent re-emit
        commands.entity(room_entity).insert(EncounterActive);

        // Get player's current weapon ID for curated tag selection
        let weapon_id = player_build.as_ref().and_then(|b| b.weapon_id.clone());

        info!(
            "Starting encounter in room '{}' with {} enemies, weapon: {:?}",
            room_instance.id, enemy_count, weapon_id
        );

        encounter_events.write(EncounterStartedEvent {
            room_id: room_instance.id.clone(),
            player_weapon_id: weapon_id,
        });
    }
}

/// Detect when a room is cleared (all enemies defeated) and emit RoomClearedEvent.
pub(crate) fn detect_room_cleared(
    room_instance_query: Query<
        (Entity, &RoomInstance, Option<&RoomWasCleared>),
        With<EncounterActive>,
    >,
    enemy_query: Query<Entity, With<Enemy>>,
    mut commands: Commands,
    mut room_cleared_events: MessageWriter<RoomClearedEvent>,
) {
    for (room_entity, room_instance, was_cleared) in room_instance_query.iter() {
        // Skip if already marked as cleared
        if was_cleared.is_some() {
            continue;
        }

        // Check if all enemies are dead
        let enemy_count = enemy_query.iter().count();
        if enemy_count > 0 {
            continue;
        }

        // Mark room as cleared to prevent re-emit
        commands.entity(room_entity).insert(RoomWasCleared);

        info!("Room '{}' cleared - all enemies defeated", room_instance.id);

        room_cleared_events.write(RoomClearedEvent {
            room_id: room_instance.id.clone(),
        });
    }
}

/// Emit EncounterCompletedEvent when RoomClearedEvent fires.
/// This triggers the transformation of curated tags into buffs.
pub(crate) fn emit_encounter_completed(
    mut room_cleared_events: MessageReader<RoomClearedEvent>,
    mut encounter_completed_events: MessageWriter<EncounterCompletedEvent>,
) {
    for event in room_cleared_events.read() {
        info!(
            "Encounter completed in room '{}', triggering tag transformation",
            event.room_id
        );

        encounter_completed_events.write(EncounterCompletedEvent {
            room_id: event.room_id.clone(),
        });
    }
}

/// Update progress tracking when a room is cleared.
pub(crate) fn update_progress_on_room_clear(
    mut room_cleared_events: MessageReader<RoomClearedEvent>,
    mut room_graph: ResMut<RoomGraph>,
    mut segment_progress: ResMut<SegmentProgress>,
) {
    for event in room_cleared_events.read() {
        // Track room as cleared in room graph
        if !room_graph.rooms_cleared.contains(&event.room_id) {
            room_graph.rooms_cleared.push(event.room_id.clone());
            info!(
                "Room '{}' added to cleared list (total: {})",
                event.room_id,
                room_graph.rooms_cleared.len()
            );
        }

        // Increment segment progress counter
        segment_progress.rooms_cleared_this_segment += 1;
        info!(
            "Segment progress: {}/{} rooms cleared",
            segment_progress.rooms_cleared_this_segment, 5 // TODO: get from config
        );
    }
}

/// Track when player enters or exits shop interaction zones
pub(crate) fn track_player_shop_zone(
    mut commands: Commands,
    mut collision_start: MessageReader<CollisionStart>,
    mut collision_end: MessageReader<CollisionEnd>,
    shop_zones: Query<&ShopInteractionZone>,
    player_query: Query<Entity, With<Player>>,
    player_near_shop: Query<&NearShop, With<Player>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Handle entering shop zones
    for event in collision_start.read() {
        let (zone_entity, other_entity) = if shop_zones.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if shop_zones.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other_entity == player_entity {
            if let Ok(zone) = shop_zones.get(zone_entity) {
                if player_near_shop.is_empty() {
                    commands.entity(player_entity).insert(NearShop {
                        shop_id: zone.shop_id.clone(),
                    });
                    info!("Player entered shop zone: {}", zone.shop_id);
                }
            }
        }
    }

    // Handle exiting shop zones
    for event in collision_end.read() {
        let (zone_entity, other_entity) = if shop_zones.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if shop_zones.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other_entity == player_entity {
            if let Ok(_zone) = shop_zones.get(zone_entity) {
                commands.entity(player_entity).remove::<NearShop>();
                info!("Player exited shop zone");
            }
        }
    }
}

/// Opens shop when player presses E while near a shop NPC
pub(crate) fn confirm_shop_entry(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_near_shop: Query<&NearShop, With<Player>>,
    mut shop_events: MessageWriter<OpenShopEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(near_shop) = player_near_shop.single() else {
        return;
    };

    info!("Opening shop: {}", near_shop.shop_id);
    shop_events.write(OpenShopEvent {
        shop_id: near_shop.shop_id.clone(),
    });
}

/// Ticks portal exit animations and updates sprite colors
pub(crate) fn update_portal_exit_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PortalExitAnimation, &mut Sprite)>,
) {
    for (entity, mut animation, mut sprite) in &mut query {
        animation.timer.tick(time.delta());
        sprite.color = animation.current_color();

        if animation.timer.remaining_secs() == 0.0 {
            commands.entity(entity).remove::<PortalExitAnimation>();
        }
    }
}
