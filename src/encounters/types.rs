//! Encounters domain: core components and resources for tag tracking.

use bevy::prelude::*;

/// Component attached to entities that have active encounter buffs.
#[derive(Component, Debug, Clone)]
pub struct EncounterBuffs {
    /// Active buff tag IDs and their effect tags
    pub active_buffs: Vec<ActiveBuff>,
}

impl Default for EncounterBuffs {
    fn default() -> Self {
        Self {
            active_buffs: Vec::new(),
        }
    }
}

/// Represents an active buff from an encounter tag
#[derive(Debug, Clone)]
pub struct ActiveBuff {
    /// The encounter tag ID (e.g., "buff_tag_fury")
    pub tag_id: String,
    /// The display name of the buff
    pub name: String,
    /// Effect tags that systems can check for (e.g., ["parry_damage_bonus"])
    pub effect_tags: Vec<String>,
    /// Tier of the buff (affects potency)
    pub tier: u32,
}

/// Marker component for entities currently in an encounter with active tags
#[derive(Component, Debug)]
pub struct InTaggedEncounter;

/// Tracks active encounter state including specialty tags
#[derive(Resource, Debug, Default)]
pub struct ActiveEncounter {
    /// The room ID where the encounter is taking place
    pub room_id: Option<String>,
    /// The specialty tag(s) applied to this encounter (usually 1 by default)
    pub specialty_tags: Vec<String>,
    /// Whether this encounter has a curated event active
    pub curated_event_id: Option<String>,
    /// Whether the encounter is currently active
    pub is_active: bool,
    /// Encounter completion status
    pub is_completed: bool,
}

impl ActiveEncounter {
    /// Start a new encounter in a room
    pub fn start(&mut self, room_id: String, specialty_tags: Vec<String>) {
        self.room_id = Some(room_id);
        self.specialty_tags = specialty_tags;
        self.curated_event_id = None;
        self.is_active = true;
        self.is_completed = false;
    }

    /// Mark encounter as completed
    pub fn complete(&mut self) {
        self.is_active = false;
        self.is_completed = true;
    }

    /// Reset for next encounter
    pub fn reset(&mut self) {
        self.room_id = None;
        self.specialty_tags.clear();
        self.curated_event_id = None;
        self.is_active = false;
        self.is_completed = false;
    }
}

/// Tracks which curated tags have been used this run (for variety)
#[derive(Resource, Debug, Default)]
pub struct EncounterTagHistory {
    /// Tags used in the last N encounters (for soft cooldown)
    pub recent_tags: Vec<String>,
    /// Maximum recent tags to track
    pub max_recent: usize,
}

impl EncounterTagHistory {
    pub fn new() -> Self {
        Self {
            recent_tags: Vec::new(),
            max_recent: 5,
        }
    }

    /// Record a tag as used
    pub fn record_used(&mut self, tag_id: &str) {
        self.recent_tags.push(tag_id.to_string());
        while self.recent_tags.len() > self.max_recent {
            self.recent_tags.remove(0);
        }
    }

    /// Check if a tag was used recently
    pub fn was_recent(&self, tag_id: &str) -> bool {
        self.recent_tags.contains(&tag_id.to_string())
    }
}
