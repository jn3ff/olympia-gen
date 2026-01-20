//! Rooms domain: room flow plugin wiring and public exports.

mod components;
mod data;
mod events;
mod graph;
mod registry;
mod spawn;
mod systems;
mod ui;

#[cfg(test)]
mod tests;

pub use events::{BossDefeatedEvent, EnterRoomEvent, ExitRoomEvent, RoomClearedEvent};
pub use graph::{RoomGraph, TransitionCooldown};
pub use registry::RoomRegistry;

use bevy::prelude::*;

use crate::core::RunState;
use crate::rooms::registry::setup_room_registry;
use crate::rooms::spawn::{spawn_arena_hub, spawn_current_room};
use crate::rooms::systems::{
    check_segment_completion, cleanup_arena, cleanup_room, confirm_arena_portal_entry,
    confirm_portal_entry, confirm_shop_entry, detect_room_cleared, drain_stale_collision_events,
    emit_encounter_completed, emit_encounter_started, evaluate_portal_conditions,
    handle_boss_defeated, handle_room_clear_coins, populate_segment_room_pool,
    process_room_transitions, reset_transition_cooldown, tick_transition_cooldown,
    track_arena_portal_zone, track_player_portal_zone, track_player_shop_zone,
    update_portal_exit_animations,
};
use crate::rooms::ui::{update_arena_portal_tooltip, update_portal_tooltip, update_shop_tooltip};

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomGraph>()
            .init_resource::<RoomRegistry>()
            .init_resource::<TransitionCooldown>()
            .add_message::<RoomClearedEvent>()
            .add_message::<BossDefeatedEvent>()
            .add_message::<EnterRoomEvent>()
            .add_message::<ExitRoomEvent>()
            .add_systems(Startup, setup_room_registry)
            .add_systems(
                OnEnter(RunState::Arena),
                (
                    reset_transition_cooldown,
                    drain_stale_collision_events,
                    populate_segment_room_pool,
                    spawn_arena_hub,
                )
                    .chain(),
            )
            .add_systems(OnExit(RunState::Arena), cleanup_arena)
            .add_systems(
                OnEnter(RunState::Room),
                (
                    reset_transition_cooldown,
                    drain_stale_collision_events,
                    spawn_current_room,
                )
                    .chain(),
            )
            .add_systems(OnExit(RunState::Room), cleanup_room)
            .add_systems(Update, tick_transition_cooldown)
            .add_systems(
                Update,
                (
                    check_segment_completion,
                    track_arena_portal_zone,
                    confirm_arena_portal_entry,
                    track_player_shop_zone,
                    update_shop_tooltip,
                    confirm_shop_entry,
                    update_arena_portal_tooltip,
                )
                    .run_if(in_state(RunState::Arena)),
            )
            .add_systems(
                Update,
                (
                    evaluate_portal_conditions,
                    track_player_portal_zone,
                    confirm_portal_entry,
                    process_room_transitions,
                    handle_boss_defeated,
                    update_portal_tooltip,
                    update_portal_exit_animations,
                )
                    .chain()
                    .run_if(in_state(RunState::Room)),
            )
            .add_systems(
                Update,
                handle_room_clear_coins.run_if(in_state(RunState::Room)),
            )
            .add_systems(
                Update,
                (
                    emit_encounter_started,
                    detect_room_cleared,
                    emit_encounter_completed,
                )
                    .chain()
                    .run_if(in_state(RunState::Room)),
            );
    }
}
