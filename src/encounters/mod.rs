//! Encounter Tags and Events System (M6)
//!
//! This module implements the curated encounter tag system and event dispatcher.
//! Each encounter applies one specialty tag by default. Curated tags trigger
//! matching events or modifiers, and can transform into buff tags on completion.

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::content::{
    ContentRegistry, EncounterTagDef, EncounterTagKind, EventDef, EventKind, GameplayDefaults,
};

// ============================================================================
// Components
// ============================================================================

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

// ============================================================================
// Resources
// ============================================================================

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

// ============================================================================
// Events
// ============================================================================

/// Fired when an encounter starts and tags should be applied
#[derive(Debug)]
pub struct EncounterStartedEvent {
    pub room_id: String,
    /// Optional weapon ID to use for curated tag selection
    pub player_weapon_id: Option<String>,
}

impl Message for EncounterStartedEvent {}

/// Fired when tags have been selected for an encounter
#[derive(Debug)]
pub struct TagsAppliedEvent {
    pub room_id: String,
    pub applied_tags: Vec<String>,
}

impl Message for TagsAppliedEvent {}

/// Fired when an encounter is completed (room cleared)
#[derive(Debug)]
pub struct EncounterCompletedEvent {
    pub room_id: String,
}

impl Message for EncounterCompletedEvent {}

/// Fired when a curated tag transforms into a buff
#[derive(Debug)]
pub struct TagTransformedEvent {
    /// The original curated tag ID
    pub curated_tag_id: String,
    /// The buff tag ID it transformed into
    pub buff_tag_id: String,
    /// The player entity receiving the buff
    pub player: Entity,
}

impl Message for TagTransformedEvent {}

/// Fired when a curated event should be triggered
#[derive(Debug)]
pub struct TriggerCuratedEventEvent {
    pub event_id: String,
    pub source_tag_id: String,
}

impl Message for TriggerCuratedEventEvent {}

/// Fired when a combat encounter event should spawn enemies
#[derive(Debug)]
pub struct SpawnCombatEventEvent {
    pub event_def: EventDef,
    pub source_tag_id: String,
}

impl Message for SpawnCombatEventEvent {}

/// Fired when a narrative encounter event should display choices
#[derive(Debug)]
pub struct SpawnNarrativeEventEvent {
    pub event_def: EventDef,
    pub source_tag_id: String,
}

impl Message for SpawnNarrativeEventEvent {}

// ============================================================================
// Plugin
// ============================================================================

pub struct EncountersPlugin;

impl Plugin for EncountersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveEncounter>()
            .init_resource::<EncounterTagHistory>()
            .add_message::<EncounterStartedEvent>()
            .add_message::<TagsAppliedEvent>()
            .add_message::<EncounterCompletedEvent>()
            .add_message::<TagTransformedEvent>()
            .add_message::<TriggerCuratedEventEvent>()
            .add_message::<SpawnCombatEventEvent>()
            .add_message::<SpawnNarrativeEventEvent>()
            .add_systems(
                Update,
                (
                    select_and_apply_tags,
                    handle_curated_event_triggers,
                    dispatch_curated_events,
                    handle_encounter_completion,
                    apply_buff_effects,
                )
                    .chain(),
            );
    }
}

// ============================================================================
// Tag Selection System
// ============================================================================

/// Selects and applies specialty tags when an encounter starts.
/// Uses weapon curated tags when available, otherwise picks from available tags.
fn select_and_apply_tags(
    mut encounter_events: MessageReader<EncounterStartedEvent>,
    mut tags_applied_events: MessageWriter<TagsAppliedEvent>,
    mut trigger_events: MessageWriter<TriggerCuratedEventEvent>,
    registry: Option<Res<ContentRegistry>>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    mut active_encounter: ResMut<ActiveEncounter>,
    mut tag_history: ResMut<EncounterTagHistory>,
) {
    let Some(registry) = registry else {
        return;
    };

    for event in encounter_events.read() {
        // Determine how many specialty tags to apply (default: 1)
        let tag_count = gameplay_defaults
            .as_ref()
            .map(|d| d.encounter_defaults.specialty_tag_count)
            .unwrap_or(1) as usize;

        // Collect candidate tags
        let mut candidate_tags =
            select_candidate_tags(&registry, event.player_weapon_id.as_deref(), &tag_history);

        // Shuffle and select tags
        let mut rng = rand::rng();
        candidate_tags.shuffle(&mut rng);
        let selected_tags: Vec<String> = candidate_tags.into_iter().take(tag_count).collect();

        // Record selected tags in history
        for tag_id in &selected_tags {
            tag_history.record_used(tag_id);
        }

        // Update active encounter
        active_encounter.start(event.room_id.clone(), selected_tags.clone());

        // Check for curated events and trigger them
        for tag_id in &selected_tags {
            if let Some(tag_def) = registry.encounter_tags.get(tag_id) {
                if tag_def.kind == EncounterTagKind::CuratedEvent {
                    if let Some(event_id) = &tag_def.event_id {
                        active_encounter.curated_event_id = Some(event_id.clone());
                        trigger_events.write(TriggerCuratedEventEvent {
                            event_id: event_id.clone(),
                            source_tag_id: tag_id.clone(),
                        });
                    }
                }
            }
        }

        info!(
            "Encounter started in room '{}' with tags: {:?}",
            event.room_id, selected_tags
        );

        tags_applied_events.write(TagsAppliedEvent {
            room_id: event.room_id.clone(),
            applied_tags: selected_tags,
        });
    }
}

/// Collects candidate tags based on weapon curated tags and available tags.
fn select_candidate_tags(
    registry: &ContentRegistry,
    player_weapon_id: Option<&str>,
    tag_history: &EncounterTagHistory,
) -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    // Priority 1: Weapon curated tags (if player has a weapon with curated tags)
    if let Some(weapon_id) = player_weapon_id {
        if let Some(weapon_def) = registry.weapon_items.get(weapon_id) {
            for curated_tag_id in &weapon_def.curated_tag_ids {
                // Skip recently used tags for variety
                if !tag_history.was_recent(curated_tag_id) {
                    if registry.encounter_tags.contains_key(curated_tag_id) {
                        candidates.push(curated_tag_id.clone());
                    }
                }
            }
        }
    }

    // Priority 2: All available curated event tags (if no weapon tags or need more)
    if candidates.is_empty() {
        for (tag_id, tag_def) in &registry.encounter_tags {
            if tag_def.kind == EncounterTagKind::CuratedEvent {
                if !tag_history.was_recent(tag_id) {
                    candidates.push(tag_id.clone());
                }
            }
        }
    }

    // Fallback: Include recently used if nothing else available
    if candidates.is_empty() {
        for (tag_id, tag_def) in &registry.encounter_tags {
            if tag_def.kind == EncounterTagKind::CuratedEvent {
                candidates.push(tag_id.clone());
            }
        }
    }

    candidates
}

// ============================================================================
// Curated Event Handling
// ============================================================================

/// Handles trigger events for curated events - validates and prepares them
fn handle_curated_event_triggers(
    mut trigger_events: MessageReader<TriggerCuratedEventEvent>,
    mut combat_events: MessageWriter<SpawnCombatEventEvent>,
    mut narrative_events: MessageWriter<SpawnNarrativeEventEvent>,
    registry: Option<Res<ContentRegistry>>,
) {
    let Some(registry) = registry else {
        return;
    };

    for trigger in trigger_events.read() {
        if let Some(event_def) = registry.events.get(&trigger.event_id) {
            info!(
                "Triggering curated event '{}' ({}) from tag '{}'",
                event_def.name, trigger.event_id, trigger.source_tag_id
            );

            match event_def.kind {
                EventKind::CombatEncounter => {
                    combat_events.write(SpawnCombatEventEvent {
                        event_def: event_def.clone(),
                        source_tag_id: trigger.source_tag_id.clone(),
                    });
                }
                EventKind::NarrativeEncounter => {
                    narrative_events.write(SpawnNarrativeEventEvent {
                        event_def: event_def.clone(),
                        source_tag_id: trigger.source_tag_id.clone(),
                    });
                }
            }
        } else {
            warn!("Curated event '{}' not found in registry", trigger.event_id);
        }
    }
}

/// Dispatches curated events to appropriate handlers.
/// Combat events spawn additional enemies, narrative events show choices.
fn dispatch_curated_events(
    mut combat_events: MessageReader<SpawnCombatEventEvent>,
    mut narrative_events: MessageReader<SpawnNarrativeEventEvent>,
    // These will be used by future systems to actually spawn content
    // For now, we just log the dispatch
) {
    for event in combat_events.read() {
        info!(
            "Combat event '{}' dispatched: {}",
            event.event_def.name, event.event_def.description
        );
        // TODO: In M6+, this will trigger spawning of themed enemies
        // based on event_def.reward_tags (e.g., ["ares"] for Ares-themed enemies)
    }

    for event in narrative_events.read() {
        info!(
            "Narrative event '{}' dispatched: {}",
            event.event_def.name, event.event_def.description
        );
        // TODO: In M6+, this will show a dialogue/choice UI
        // with rewards tagged by event_def.reward_tags
    }
}

// ============================================================================
// Encounter Completion and Tag Transformation
// ============================================================================

/// Handles encounter completion - transforms curated tags into buffs.
fn handle_encounter_completion(
    mut completion_events: MessageReader<EncounterCompletedEvent>,
    mut transform_events: MessageWriter<TagTransformedEvent>,
    registry: Option<Res<ContentRegistry>>,
    mut active_encounter: ResMut<ActiveEncounter>,
    mut player_query: Query<(Entity, &mut EncounterBuffs), With<crate::movement::Player>>,
) {
    let Some(registry) = registry else {
        return;
    };

    for event in completion_events.read() {
        if !active_encounter.is_active {
            continue;
        }

        info!(
            "Encounter completed in room '{}', transforming tags",
            event.room_id
        );

        // Transform curated tags into buffs
        for tag_id in &active_encounter.specialty_tags {
            if let Some(tag_def) = registry.encounter_tags.get(tag_id) {
                if tag_def.kind == EncounterTagKind::CuratedEvent {
                    // Find a matching buff tag to transform into
                    if let Some(buff_tag_id) = find_related_buff_tag(&registry, tag_def) {
                        // Apply buff to player
                        for (player_entity, mut buffs) in &mut player_query {
                            if let Some(buff_def) = registry.encounter_tags.get(&buff_tag_id) {
                                let active_buff = ActiveBuff {
                                    tag_id: buff_tag_id.clone(),
                                    name: buff_def.name.clone(),
                                    effect_tags: buff_def.effect_tags.clone(),
                                    tier: buff_def.tier,
                                };

                                // Check if player already has this buff
                                if !buffs.active_buffs.iter().any(|b| b.tag_id == buff_tag_id) {
                                    buffs.active_buffs.push(active_buff);
                                    info!(
                                        "Applied buff '{}' to player from curated tag '{}'",
                                        buff_def.name, tag_id
                                    );

                                    transform_events.write(TagTransformedEvent {
                                        curated_tag_id: tag_id.clone(),
                                        buff_tag_id: buff_tag_id.clone(),
                                        player: player_entity,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        active_encounter.complete();
    }
}

/// Finds a related buff tag for a curated event tag.
/// Uses tag naming conventions and god associations.
fn find_related_buff_tag(
    registry: &ContentRegistry,
    curated_tag: &EncounterTagDef,
) -> Option<String> {
    // Extract god name from curated tag ID (e.g., "tag_ares_warpath" -> "ares")
    let tag_parts: Vec<&str> = curated_tag.id.split('_').collect();
    let god_keyword = tag_parts.get(1).map(|s| *s);

    // Map god keywords to related buff tags
    let buff_mapping = [
        ("ares", "buff_tag_fury"),      // Ares -> Fury (parry damage bonus)
        ("demeter", "buff_tag_frost"),  // Demeter -> Frost (slow on light)
        ("poseidon", "buff_tag_surge"), // Poseidon -> Surge (knockback on heavy)
        ("zeus", "buff_tag_surge"),     // Zeus -> Surge (temporary, could add lightning buff)
    ];

    // Find matching buff
    if let Some(god) = god_keyword {
        for (keyword, buff_id) in &buff_mapping {
            if god == *keyword {
                if registry.encounter_tags.contains_key(*buff_id) {
                    return Some(buff_id.to_string());
                }
            }
        }
    }

    // Fallback: return first available buff tag
    for (tag_id, tag_def) in &registry.encounter_tags {
        if tag_def.kind == EncounterTagKind::Buff {
            return Some(tag_id.clone());
        }
    }

    None
}

// ============================================================================
// Buff Effect System
// ============================================================================

/// Applies active buff effects to combat.
/// This system checks for buff effect tags and modifies combat accordingly.
fn apply_buff_effects(
    player_query: Query<&EncounterBuffs, With<crate::movement::Player>>,
    // This is a placeholder for now - buff effects will be integrated with combat in future
) {
    for buffs in &player_query {
        if buffs.active_buffs.is_empty() {
            continue;
        }

        // Log active buffs for debugging
        // In full implementation, these effect_tags would be checked during combat
        // For example:
        // - "parry_damage_bonus" -> increases damage after successful parry
        // - "slow_on_light" -> applies slow debuff on light attack hits
        // - "knockback_on_heavy" -> increases knockback on heavy attacks
    }
}

// ============================================================================
// Buff Effect Queries (for use by other systems)
// ============================================================================

/// Check if the player has a specific buff effect active
pub fn has_buff_effect(buffs: &EncounterBuffs, effect_tag: &str) -> bool {
    buffs
        .active_buffs
        .iter()
        .any(|buff| buff.effect_tags.contains(&effect_tag.to_string()))
}

/// Get the total tier bonus for a specific effect (for stacking buffs)
pub fn get_buff_effect_tier(buffs: &EncounterBuffs, effect_tag: &str) -> u32 {
    buffs
        .active_buffs
        .iter()
        .filter(|buff| buff.effect_tags.contains(&effect_tag.to_string()))
        .map(|buff| buff.tier)
        .sum()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_encounter_lifecycle() {
        let mut encounter = ActiveEncounter::default();
        assert!(!encounter.is_active);

        encounter.start("room_1".to_string(), vec!["tag_ares_warpath".to_string()]);
        assert!(encounter.is_active);
        assert!(!encounter.is_completed);
        assert_eq!(encounter.specialty_tags.len(), 1);

        encounter.complete();
        assert!(!encounter.is_active);
        assert!(encounter.is_completed);

        encounter.reset();
        assert!(encounter.specialty_tags.is_empty());
        assert!(!encounter.is_active);
        assert!(!encounter.is_completed);
    }

    #[test]
    fn test_tag_history() {
        let mut history = EncounterTagHistory::new();
        history.max_recent = 3;

        history.record_used("tag_a");
        history.record_used("tag_b");
        assert!(history.was_recent("tag_a"));
        assert!(history.was_recent("tag_b"));
        assert!(!history.was_recent("tag_c"));

        history.record_used("tag_c");
        history.record_used("tag_d");
        // tag_a should be evicted (max_recent = 3)
        assert!(!history.was_recent("tag_a"));
        assert!(history.was_recent("tag_b"));
        assert!(history.was_recent("tag_c"));
        assert!(history.was_recent("tag_d"));
    }

    #[test]
    fn test_has_buff_effect() {
        let buffs = EncounterBuffs {
            active_buffs: vec![
                ActiveBuff {
                    tag_id: "buff_tag_fury".to_string(),
                    name: "Fury".to_string(),
                    effect_tags: vec!["parry_damage_bonus".to_string()],
                    tier: 1,
                },
                ActiveBuff {
                    tag_id: "buff_tag_frost".to_string(),
                    name: "Frost".to_string(),
                    effect_tags: vec!["slow_on_light".to_string()],
                    tier: 2,
                },
            ],
        };

        assert!(has_buff_effect(&buffs, "parry_damage_bonus"));
        assert!(has_buff_effect(&buffs, "slow_on_light"));
        assert!(!has_buff_effect(&buffs, "knockback_on_heavy"));
    }

    #[test]
    fn test_get_buff_effect_tier() {
        let buffs = EncounterBuffs {
            active_buffs: vec![
                ActiveBuff {
                    tag_id: "buff_1".to_string(),
                    name: "Buff 1".to_string(),
                    effect_tags: vec!["damage_bonus".to_string()],
                    tier: 1,
                },
                ActiveBuff {
                    tag_id: "buff_2".to_string(),
                    name: "Buff 2".to_string(),
                    effect_tags: vec!["damage_bonus".to_string()],
                    tier: 2,
                },
            ],
        };

        assert_eq!(get_buff_effect_tier(&buffs, "damage_bonus"), 3);
        assert_eq!(get_buff_effect_tier(&buffs, "other"), 0);
    }
}
