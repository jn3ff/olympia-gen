//! Faith tracking system for M7.
//!
//! Tracks run faith per god with a floor at 0 for meta faith. If run faith
//! goes below 0, schedules adversarial events within a configurable number
//! of segments.

use bevy::ecs::message::Message;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Resources
// ============================================================================

/// Tracks faith values for each god during a run.
///
/// - Positive faith: The god is pleased with the player's choices
/// - Zero faith: Neutral standing
/// - Negative faith: The god is displeased and may trigger adversarial events
///
/// The `floor` value (default 0) determines when meta faith is affected.
/// Run faith can go negative, but meta faith (persistent) cannot go below floor.
#[derive(Resource, Debug, Clone)]
pub struct RunFaith {
    /// Faith values per god (indexed by god_id)
    pub faith_by_god: HashMap<String, i32>,
    /// Floor for meta faith (run faith can go below, meta faith cannot)
    pub floor: i32,
    /// Track which gods have already triggered adversarial events this run
    pub triggered_adversarial: HashSet<String>,
    /// Scheduled adversarial events: (god_id, trigger_at_segment)
    pub scheduled_events: Vec<ScheduledAdversarialEvent>,
    /// History of faith changes for post-run display
    pub faith_history: Vec<FaithChange>,
}

impl Default for RunFaith {
    fn default() -> Self {
        Self {
            faith_by_god: HashMap::new(),
            floor: 0,
            triggered_adversarial: HashSet::new(),
            scheduled_events: Vec::new(),
            faith_history: Vec::new(),
        }
    }
}

impl RunFaith {
    /// Reset for a new run
    pub fn reset(&mut self) {
        self.faith_by_god.clear();
        self.triggered_adversarial.clear();
        self.scheduled_events.clear();
        self.faith_history.clear();
    }

    /// Get faith for a god (returns 0 if not tracked)
    pub fn get_faith(&self, god_id: &str) -> i32 {
        *self.faith_by_god.get(god_id).unwrap_or(&0)
    }

    /// Modify faith for a god by delta amount
    pub fn modify_faith(&mut self, god_id: &str, delta: i32, reason: &str) {
        let current = self.get_faith(god_id);
        let new_value = current + delta;
        self.faith_by_god.insert(god_id.to_string(), new_value);

        // Record in history
        self.faith_history.push(FaithChange {
            god_id: god_id.to_string(),
            delta,
            new_value,
            reason: reason.to_string(),
        });
    }

    /// Get all gods with negative faith
    pub fn gods_with_negative_faith(&self) -> Vec<(&String, i32)> {
        self.faith_by_god
            .iter()
            .filter(|(_, faith)| **faith < 0)
            .map(|(god_id, faith)| (god_id, *faith))
            .collect()
    }

    /// Check if a god needs an adversarial event scheduled
    pub fn needs_adversarial_event(&self, god_id: &str) -> bool {
        let faith = self.get_faith(god_id);
        faith < 0
            && !self.triggered_adversarial.contains(god_id)
            && !self.scheduled_events.iter().any(|e| e.god_id == god_id)
    }

    /// Schedule an adversarial event for a god
    pub fn schedule_adversarial_event(
        &mut self,
        god_id: &str,
        event_id: &str,
        trigger_at_segment: u32,
    ) {
        self.scheduled_events.push(ScheduledAdversarialEvent {
            god_id: god_id.to_string(),
            event_id: event_id.to_string(),
            trigger_at_segment,
        });
    }

    /// Get events that should trigger at the given segment
    pub fn get_events_for_segment(&self, segment: u32) -> Vec<&ScheduledAdversarialEvent> {
        self.scheduled_events
            .iter()
            .filter(|e| e.trigger_at_segment == segment)
            .collect()
    }

    /// Mark an adversarial event as triggered
    pub fn mark_adversarial_triggered(&mut self, god_id: &str) {
        self.triggered_adversarial.insert(god_id.to_string());
        // Remove from scheduled
        self.scheduled_events.retain(|e| e.god_id != god_id);
    }

    /// Get the total faith delta during this run (sum of all changes)
    pub fn total_delta_for_god(&self, god_id: &str) -> i32 {
        self.faith_history
            .iter()
            .filter(|c| c.god_id == god_id)
            .map(|c| c.delta)
            .sum()
    }

    /// Get summary of faith changes for all gods
    pub fn get_faith_summary(&self) -> Vec<FaithSummary> {
        let mut summaries: HashMap<String, FaithSummary> = HashMap::new();

        for change in &self.faith_history {
            let summary = summaries
                .entry(change.god_id.clone())
                .or_insert(FaithSummary {
                    god_id: change.god_id.clone(),
                    total_delta: 0,
                    final_value: 0,
                    positive_changes: 0,
                    negative_changes: 0,
                });
            summary.total_delta += change.delta;
            summary.final_value = change.new_value;
            if change.delta > 0 {
                summary.positive_changes += 1;
            } else if change.delta < 0 {
                summary.negative_changes += 1;
            }
        }

        summaries.into_values().collect()
    }
}

/// A scheduled adversarial event
#[derive(Debug, Clone)]
pub struct ScheduledAdversarialEvent {
    pub god_id: String,
    pub event_id: String,
    pub trigger_at_segment: u32,
}

/// A single faith change record
#[derive(Debug, Clone)]
pub struct FaithChange {
    pub god_id: String,
    pub delta: i32,
    pub new_value: i32,
    pub reason: String,
}

/// Summary of faith changes for a god
#[derive(Debug, Clone)]
pub struct FaithSummary {
    pub god_id: String,
    pub total_delta: i32,
    pub final_value: i32,
    pub positive_changes: i32,
    pub negative_changes: i32,
}

// ============================================================================
// Events
// ============================================================================

/// Event fired when faith changes for a god
#[derive(Debug)]
pub struct FaithChangedEvent {
    pub god_id: String,
    pub delta: i32,
    pub new_value: i32,
    pub reason: String,
}

impl Message for FaithChangedEvent {}

/// Event fired when an adversarial event is scheduled
#[derive(Debug)]
pub struct AdversarialEventScheduledEvent {
    pub god_id: String,
    pub event_id: String,
    pub trigger_at_segment: u32,
}

impl Message for AdversarialEventScheduledEvent {}

/// Event fired when an adversarial event should be triggered
#[derive(Debug)]
pub struct TriggerAdversarialEvent {
    pub god_id: String,
    pub event_id: String,
}

impl Message for TriggerAdversarialEvent {}

// ============================================================================
// Faith Constants
// ============================================================================

/// Faith gain when choosing a blessing from a god
pub const FAITH_GAIN_BLESSING_CHOSEN: i32 = 10;

/// Faith loss for other gods when their blessing is available but not chosen
pub const FAITH_LOSS_BLESSING_REJECTED: i32 = -3;

/// Faith gain from completing a god-aligned event
pub const FAITH_GAIN_EVENT_COMPLETED: i32 = 5;

/// Minimum segments before adversarial event can trigger
pub const ADVERSARIAL_MIN_DELAY: u32 = 1;
