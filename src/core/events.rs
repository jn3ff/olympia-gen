//! Core domain: events for run flow and selection.

use bevy::ecs::message::Message;

/// Event fired when a character is selected
#[derive(Debug)]
pub struct CharacterSelectedEvent {
    pub character_id: String,
}

impl Message for CharacterSelectedEvent {}

/// Event fired when a segment is completed
#[derive(Debug)]
pub struct SegmentCompletedEvent {
    pub segment_index: u32,
}

impl Message for SegmentCompletedEvent {}

/// Event fired when the run is won (boss_target reached)
#[derive(Debug)]
pub struct RunVictoryEvent {
    pub total_bosses_defeated: u32,
}

impl Message for RunVictoryEvent {}
