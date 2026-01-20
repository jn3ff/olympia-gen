//! Encounters domain: curated event dispatch and trigger handling.

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::content::{ContentRegistry, EventKind};
use crate::encounters::events::{
    SpawnCombatEventEvent, SpawnNarrativeEventEvent, TriggerCuratedEventEvent,
};

/// Handles trigger events for curated events - validates and prepares them
pub(crate) fn handle_curated_event_triggers(
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
pub(crate) fn dispatch_curated_events(
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
