//! Core domain: game state definitions for the run flow.

use bevy::prelude::*;

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum GameState {
    #[default]
    Boot,
    MainMenu,
    CharacterSelect,
    Run,
    Reward,
    Paused,
    Victory,
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum RunState {
    #[default]
    Arena,
    Room,
    Boss,
    Reward,
}
