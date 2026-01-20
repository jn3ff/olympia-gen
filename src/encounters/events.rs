//! Encounters domain: event definitions for tags and curated encounters.

use bevy::ecs::message::Message;
use bevy::prelude::*;

use crate::content::EventDef;

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
