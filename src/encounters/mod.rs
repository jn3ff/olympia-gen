//! Encounters domain: curated encounter tags, events, and buffs.

mod buffs;
mod events;
mod selection;
#[cfg(test)]
mod tests;
mod triggers;
mod types;

pub use buffs::{get_buff_effect_tier, has_buff_effect};
pub use events::{
    EncounterCompletedEvent, EncounterStartedEvent, SpawnCombatEventEvent,
    SpawnNarrativeEventEvent, TagTransformedEvent, TagsAppliedEvent, TriggerCuratedEventEvent,
};
pub use types::{ActiveBuff, ActiveEncounter, EncounterBuffs, EncounterTagHistory};

use bevy::prelude::*;

use crate::encounters::buffs::{apply_buff_effects, handle_encounter_completion};
use crate::encounters::selection::select_and_apply_tags;
use crate::encounters::triggers::{dispatch_curated_events, handle_curated_event_triggers};

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
