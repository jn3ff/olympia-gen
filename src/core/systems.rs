//! Core domain: core run flow systems and setup.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use rand::Rng;

use crate::core::events::{RunVictoryEvent, SegmentCompletedEvent};
use crate::core::resources::{RunConfig, SegmentProgress};
use crate::core::state::{GameState, RunState};
use crate::rewards::RunFaith;

pub(crate) fn transition_to_character_select(mut game_state: ResMut<NextState<GameState>>) {
    // Go to character selection before starting the run
    game_state.set(GameState::CharacterSelect);
}

/// Transition from character select to run state
#[allow(dead_code)]
pub(crate) fn transition_to_run(
    mut game_state: ResMut<NextState<GameState>>,
    mut run_state: ResMut<NextState<RunState>>,
) {
    game_state.set(GameState::Run);
    run_state.set(RunState::Arena);
}

/// Initialize a new run with a fresh seed and reset segment
pub(crate) fn initialize_run(
    mut run_config: ResMut<RunConfig>,
    mut segment_progress: ResMut<SegmentProgress>,
    mut run_faith: ResMut<RunFaith>,
) {
    // Generate a new random seed for this run
    run_config.seed = rand::rng().random();
    run_config.segment_index = 0;

    // Reset segment progress for new run
    segment_progress.reset();

    // Reset faith tracking for new run
    run_faith.reset();

    info!(
        "Starting new run with seed: {}, segment: {}",
        run_config.seed, run_config.segment_index
    );
}

/// Handle segment completion - increment segment and return to hub
pub(crate) fn handle_segment_completed(
    mut events: MessageReader<SegmentCompletedEvent>,
    mut run_config: ResMut<RunConfig>,
    mut segment_progress: ResMut<SegmentProgress>,
    mut next_run_state: ResMut<NextState<RunState>>,
) {
    for event in events.read() {
        info!("Segment {} completed!", event.segment_index);

        // Increment segment
        run_config.segment_index += 1;

        // Reset segment-specific counters (preserves run-wide tracking)
        segment_progress.advance_segment();

        // Return to hub
        next_run_state.set(RunState::Arena);
    }
}

/// Handle victory - transition to victory state
pub(crate) fn handle_victory(
    mut events: MessageReader<RunVictoryEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        info!(
            "Victory! Defeated {} bosses total.",
            event.total_bosses_defeated
        );
        game_state.set(GameState::Victory);
    }
}

pub(crate) fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
