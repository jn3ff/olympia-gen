//! Encounters domain: buff transformation and effect helpers.

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::content::{ContentRegistry, EncounterTagDef, EncounterTagKind};
use crate::encounters::events::{EncounterCompletedEvent, TagTransformedEvent};
use crate::encounters::types::{ActiveBuff, ActiveEncounter, EncounterBuffs};
use crate::movement::Player;

/// Handles encounter completion - transforms curated tags into buffs.
pub(crate) fn handle_encounter_completion(
    mut completion_events: MessageReader<EncounterCompletedEvent>,
    mut transform_events: MessageWriter<TagTransformedEvent>,
    registry: Option<Res<ContentRegistry>>,
    mut active_encounter: ResMut<ActiveEncounter>,
    mut player_query: Query<(Entity, &mut EncounterBuffs), With<Player>>,
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

/// Applies active buff effects to combat.
/// This system checks for buff effect tags and modifies combat accordingly.
pub(crate) fn apply_buff_effects(
    player_query: Query<&EncounterBuffs, With<Player>>,
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
