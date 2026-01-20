//! Combat domain: tuning and input resources.

use bevy::prelude::*;

use crate::combat::components::EnemyTier;

/// Represents a range of gold values that can be dropped
#[derive(Debug, Clone, Copy)]
pub struct GoldDropRange {
    pub min: u32,
    pub max: u32,
}

impl GoldDropRange {
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Fixed value (no variance)
    pub fn fixed(value: u32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    /// Roll a random value within range (inclusive)
    pub fn roll(&self) -> u32 {
        if self.min >= self.max {
            return self.min;
        }
        let range = self.max - self.min + 1;
        self.min + (rand::random::<u32>() % range)
    }
}

/// Configuration for gold drops per enemy tier
#[derive(Resource, Debug, Clone)]
pub struct GoldDropConfig {
    /// Gold range for Minor tier enemies (default: 2-5)
    pub minor: GoldDropRange,
    /// Gold range for Major tier enemies (default: 10-30)
    pub major: GoldDropRange,
    /// Gold range for Special/Elite tier enemies (default: 80-120)
    pub special: GoldDropRange,
    /// Gold range for Boss tier enemies (default: 180-220)
    pub boss: GoldDropRange,
}

impl Default for GoldDropConfig {
    fn default() -> Self {
        Self {
            minor: GoldDropRange::new(2, 5),
            major: GoldDropRange::new(10, 30),
            special: GoldDropRange::new(80, 120),
            boss: GoldDropRange::new(180, 220),
        }
    }
}

impl GoldDropConfig {
    /// Get the gold drop range for a given tier
    pub fn range_for_tier(&self, tier: &EnemyTier) -> GoldDropRange {
        match tier {
            EnemyTier::Minor => self.minor,
            EnemyTier::Major => self.major,
            EnemyTier::Special => self.special,
            EnemyTier::Boss => self.boss,
        }
    }
}

/// Per-attack-type configuration
#[derive(Debug, Clone)]
pub struct AttackConfig {
    pub damage: f32,
    pub knockback: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub hitbox_length: f32,
    pub hitbox_width: f32,
    pub hitbox_offset: f32,
    pub hitbox_duration: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct AttackTuning {
    pub light: AttackConfig,
    pub heavy: AttackConfig,
    pub special: AttackConfig,
    pub combo_window: f32,
}

impl Default for AttackTuning {
    fn default() -> Self {
        Self {
            light: AttackConfig {
                damage: 8.0,
                knockback: 150.0,
                duration: 0.12,
                cooldown: 0.05,
                hitbox_length: 45.0,
                hitbox_width: 35.0,
                hitbox_offset: 28.0,
                hitbox_duration: 0.08,
            },
            heavy: AttackConfig {
                damage: 25.0,
                knockback: 400.0,
                duration: 0.5,
                cooldown: 0.4,
                hitbox_length: 50.0,
                hitbox_width: 45.0,
                hitbox_offset: 32.0,
                hitbox_duration: 0.15,
            },
            special: AttackConfig {
                damage: 40.0,
                knockback: 600.0,
                duration: 0.6,
                cooldown: 0.8,
                hitbox_length: 60.0,
                hitbox_width: 50.0,
                hitbox_offset: 35.0,
                hitbox_duration: 0.2,
            },
            combo_window: 0.3,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyTuning {
    pub move_speed: f32,
    pub chase_speed: f32,
    pub attack_damage: f32,
    pub attack_knockback: f32,
    pub attack_duration: f32,
    pub attack_cooldown: f32,
    pub stagger_duration: f32,
    pub patrol_pause_time: f32,
}

impl Default for EnemyTuning {
    fn default() -> Self {
        Self {
            move_speed: 80.0,
            chase_speed: 150.0,
            attack_damage: 15.0,
            attack_knockback: 300.0,
            attack_duration: 0.4,
            attack_cooldown: 1.0,
            stagger_duration: 0.3,
            patrol_pause_time: 1.0,
        }
    }
}

/// Configuration for boss encounter spawning
#[derive(Resource, Debug, Clone)]
pub struct BossConfig {
    /// Probability of a boss encounter per room (0.0 to 1.0)
    pub encounter_rate: f32,
    /// Minimum rooms between boss encounters
    pub min_rooms_between: u32,
    /// Maximum rooms without a boss before forcing one
    pub max_rooms_between: u32,
}

impl Default for BossConfig {
    fn default() -> Self {
        Self {
            encounter_rate: 0.15,
            min_rooms_between: 3,
            max_rooms_between: 8,
        }
    }
}

/// Tracks boss encounter state across the run
#[derive(Resource, Debug, Default)]
pub struct BossEncounterState {
    /// Rooms cleared since last boss
    pub rooms_since_boss: u32,
    /// Total bosses defeated this run
    pub bosses_defeated: u32,
}

impl BossEncounterState {
    /// Determine if the next room should have a boss
    pub fn should_spawn_boss(&self, config: &BossConfig, rng_roll: f32) -> bool {
        // Force boss if we've hit max rooms
        if self.rooms_since_boss >= config.max_rooms_between {
            return true;
        }

        // Never spawn boss before min rooms
        if self.rooms_since_boss < config.min_rooms_between {
            return false;
        }

        // Roll for boss based on encounter rate
        rng_roll < config.encounter_rate
    }

    /// Record a room cleared (no boss)
    pub fn room_cleared(&mut self) {
        self.rooms_since_boss += 1;
    }

    /// Record a boss defeated
    pub fn boss_defeated(&mut self) {
        self.rooms_since_boss = 0;
        self.bosses_defeated += 1;
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CombatTuning {
    pub iframes_duration: f32,
    pub stagger_duration: f32,
    pub damage_flash_duration: f32,
}

impl Default for CombatTuning {
    fn default() -> Self {
        Self {
            iframes_duration: 0.5,
            stagger_duration: 0.2,
            damage_flash_duration: 0.1,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct CombatInput {
    pub light_attack: bool,
    pub heavy_attack: bool,
    pub special_attack: bool,
    pub parry: bool,
}
