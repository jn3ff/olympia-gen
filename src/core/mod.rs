//! Core domain: game state, run flow, and core UI.

mod events;
mod resources;
mod state;
mod systems;
pub mod ui;

pub use events::{CharacterSelectedEvent, RunVictoryEvent, SegmentCompletedEvent};
pub use resources::{
    DifficultyScaling, GameplayPaused, RunConfig, SegmentProgress, SelectedCharacter,
    gameplay_active,
};
pub use state::{GameState, RunState};

use bevy::prelude::*;

use crate::core::systems::{
    handle_segment_completed, handle_victory, initialize_run, setup_camera,
    transition_to_character_select,
};
use crate::core::ui::character_select::{
    cleanup_character_select_ui, handle_character_select_click, handle_character_select_input,
    spawn_character_select_ui,
};
use crate::core::ui::victory::{
    cleanup_victory_screen, handle_victory_input, spawn_victory_screen,
};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<RunState>()
            .init_resource::<RunConfig>()
            .init_resource::<DifficultyScaling>()
            .init_resource::<SelectedCharacter>()
            .init_resource::<SegmentProgress>()
            .init_resource::<GameplayPaused>()
            .add_message::<CharacterSelectedEvent>()
            .add_message::<SegmentCompletedEvent>()
            .add_message::<RunVictoryEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Boot), transition_to_character_select)
            .add_systems(
                OnEnter(GameState::CharacterSelect),
                spawn_character_select_ui,
            )
            .add_systems(
                OnExit(GameState::CharacterSelect),
                cleanup_character_select_ui,
            )
            .add_systems(
                Update,
                (handle_character_select_input, handle_character_select_click)
                    .run_if(in_state(GameState::CharacterSelect)),
            )
            .add_systems(OnEnter(GameState::Run), initialize_run)
            .add_systems(
                Update,
                (handle_segment_completed, handle_victory).run_if(in_state(GameState::Run)),
            )
            .add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
            .add_systems(OnExit(GameState::Victory), cleanup_victory_screen)
            .add_systems(
                Update,
                handle_victory_input.run_if(in_state(GameState::Victory)),
            );
    }
}
