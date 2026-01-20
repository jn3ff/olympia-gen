//! Debug domain: state and action definitions for debug tooling.

use bevy::prelude::*;

use crate::combat::EnemyTier;

/// Resource tracking debug mode state
#[derive(Resource, Debug)]
pub struct DebugState {
    /// Whether debug UI is visible
    pub ui_visible: bool,
    /// Whether player is invincible
    pub invincible: bool,
    /// Whether to show debug info overlay (position, health, etc.)
    pub show_info: bool,
    /// Custom seed input buffer
    pub seed_input: String,
    /// Currently selected enemy tier for spawning
    pub selected_tier: EnemyTier,
    /// Message to display temporarily in debug UI
    pub status_message: Option<(String, f32)>,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            ui_visible: false,
            invincible: false,
            show_info: false,
            seed_input: String::new(),
            selected_tier: EnemyTier::Minor,
            status_message: None,
        }
    }
}

impl DebugState {
    /// Set a status message that will fade after a duration
    pub fn set_message(&mut self, message: impl Into<String>, duration: f32) {
        self.status_message = Some((message.into(), duration));
    }
}

/// Actions that can be triggered from debug UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    ToggleInvincible,
    SpawnEnemy,
    SpawnBoss,
    WarpToArena,
    WarpToRoom,
    WarpToBoss,
    SetSeed,
    FullHeal,
    CycleTier,
    ToggleInfo,
    Close,
}
