//! Encounters domain: tag selection and application logic.

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::content::{ContentRegistry, EncounterTagKind, GameplayDefaults};
use crate::encounters::events::{
    EncounterStartedEvent, TagsAppliedEvent, TriggerCuratedEventEvent,
};
use crate::encounters::types::{ActiveEncounter, EncounterTagHistory};

/// Selects and applies specialty tags when an encounter starts.
/// Uses weapon curated tags when available, otherwise picks from available tags.
pub(crate) fn select_and_apply_tags(
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
