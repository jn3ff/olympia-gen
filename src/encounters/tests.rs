//! Encounters domain: unit tests for encounter tracking and buffs.

use super::{
    ActiveBuff, ActiveEncounter, EncounterBuffs, EncounterTagHistory, get_buff_effect_tier,
    has_buff_effect,
};

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
