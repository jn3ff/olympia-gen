//! Core domain: shared resources for run configuration and progression.

use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

/// Resource tracking if gameplay should be paused.
/// Gameplay is paused if any source is active.
#[derive(Resource, Debug, Default)]
pub struct GameplayPaused {
    pub sources: HashSet<String>,
}

impl GameplayPaused {
    pub fn is_paused(&self) -> bool {
        !self.sources.is_empty()
    }

    pub fn pause(&mut self, source: impl Into<String>) {
        self.sources.insert(source.into());
    }

    pub fn unpause(&mut self, source: impl Into<String>) {
        self.sources.remove(&source.into());
    }
}

/// Run condition: returns true only when gameplay is not paused
pub fn gameplay_active(paused: Res<GameplayPaused>) -> bool {
    !paused.is_paused()
}

/// Resource tracking the currently selected character
#[derive(Resource, Debug, Default)]
pub struct SelectedCharacter {
    pub character_id: Option<String>,
}

impl SelectedCharacter {
    pub fn select(&mut self, character_id: impl Into<String>) {
        self.character_id = Some(character_id.into());
    }

    pub fn is_selected(&self) -> bool {
        self.character_id.is_some()
    }
}

#[derive(Resource, Debug)]
pub struct RunConfig {
    pub seed: u64,
    pub segment_index: u32,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            seed: rand::rng().random(),
            segment_index: 0,
        }
    }
}

/// Tracks progress within the current segment and across the entire run.
#[derive(Resource, Debug, Default)]
pub struct SegmentProgress {
    /// Rooms cleared in current segment (excludes boss rooms)
    pub rooms_cleared_this_segment: u32,
    /// Bosses defeated in current segment
    pub bosses_defeated_this_segment: u32,
    /// Total bosses defeated across entire run (for win condition)
    pub total_bosses_defeated: u32,
    /// The biome_id for this segment (selected at segment start)
    pub current_biome_id: Option<String>,
    /// Pool of available room_ids for this segment (pre-selected at segment start)
    pub room_pool: Vec<String>,
    /// Pool of available boss room_ids for this segment
    pub boss_room_pool: Vec<String>,
    /// IDs of significant enemies already encountered this run (for no-repeat logic)
    pub encountered_significant_enemies: HashSet<String>,
    /// Flag indicating segment pools need initialization
    pub needs_pool_init: bool,
}

impl SegmentProgress {
    /// Reset for a new run
    pub fn reset(&mut self) {
        self.rooms_cleared_this_segment = 0;
        self.bosses_defeated_this_segment = 0;
        self.total_bosses_defeated = 0;
        self.current_biome_id = None;
        self.room_pool.clear();
        self.boss_room_pool.clear();
        self.encountered_significant_enemies.clear();
        self.needs_pool_init = true;
    }

    /// Reset segment-specific counters for next segment (preserves run-wide tracking)
    pub fn advance_segment(&mut self) {
        self.rooms_cleared_this_segment = 0;
        self.bosses_defeated_this_segment = 0;
        self.current_biome_id = None;
        self.room_pool.clear();
        self.boss_room_pool.clear();
        self.needs_pool_init = true;
        // NOTE: encountered_significant_enemies persists across segments
    }
}

/// Configuration for how difficulty scales with segment progression
#[derive(Resource, Debug, Clone)]
pub struct DifficultyScaling {
    /// Base multiplier applied to all scaling (adjust for overall difficulty)
    pub base_multiplier: f32,
    /// How much enemy health increases per segment (e.g., 0.15 = +15% per segment)
    pub enemy_health_per_segment: f32,
    /// How much enemy damage increases per segment
    pub enemy_damage_per_segment: f32,
    /// How much enemy count increases per segment (additive)
    pub enemy_count_per_segment: f32,
    /// How much boss health increases per segment
    pub boss_health_per_segment: f32,
    /// How much boss damage increases per segment
    pub boss_damage_per_segment: f32,
    /// Bonus to higher tier reward drop rates per segment
    pub reward_tier_bonus_per_segment: f32,
    /// Maximum scaling multiplier (caps the difficulty growth)
    pub max_scaling_multiplier: f32,
}

impl Default for DifficultyScaling {
    fn default() -> Self {
        Self {
            base_multiplier: 1.0,
            enemy_health_per_segment: 0.20,
            enemy_damage_per_segment: 0.15,
            enemy_count_per_segment: 0.5,
            boss_health_per_segment: 0.25,
            boss_damage_per_segment: 0.20,
            reward_tier_bonus_per_segment: 0.05,
            max_scaling_multiplier: 5.0,
        }
    }
}

impl DifficultyScaling {
    /// Calculate health multiplier for enemies at the given segment
    pub fn enemy_health_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.enemy_health_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate damage multiplier for enemies at the given segment
    pub fn enemy_damage_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.enemy_damage_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate additional enemy count for the given segment
    pub fn bonus_enemy_count(&self, segment: u32) -> usize {
        (segment as f32 * self.enemy_count_per_segment).floor() as usize
    }

    /// Calculate health multiplier for bosses at the given segment
    pub fn boss_health_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.boss_health_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate damage multiplier for bosses at the given segment
    pub fn boss_damage_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.boss_damage_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate tier drop bonus for rewards at the given segment
    /// Returns a value to shift tier probabilities toward higher tiers
    pub fn reward_tier_bonus(&self, segment: u32) -> f32 {
        let raw = segment as f32 * self.reward_tier_bonus_per_segment;
        raw.min(0.5) // Cap at 50% bonus to preserve some randomness
    }
}
