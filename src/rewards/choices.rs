//! Rewards domain: reward choice generation and application.

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::combat::BossDefeatedEvent;
use crate::content::{
    BlessingDef, ContentRegistry, EquipmentItemDef, GameplayDefaults, StatKind as ContentStatKind,
};
use crate::core::{DifficultyScaling, RunConfig, RunState};
use crate::rewards::build::PlayerBuild;
use crate::rewards::economy::{CoinGainedEvent, CoinSource};
use crate::rewards::faith::{
    FAITH_GAIN_BLESSING_CHOSEN, FAITH_LOSS_BLESSING_REJECTED, FaithChangedEvent, RunFaith,
};
use crate::rewards::types::{EquipmentSlot, RewardKind, RewardTier, StatType};

#[derive(Debug)]
pub struct RewardOfferedEvent {
    pub choices: Vec<RewardKind>,
}

impl Message for RewardOfferedEvent {}

#[derive(Debug)]
pub struct RewardChosenEvent {
    pub choice_index: usize,
}

impl Message for RewardChosenEvent {}

/// Holds the current reward choices being offered
#[derive(Resource, Debug, Default)]
pub struct CurrentRewardChoices {
    pub choices: Vec<RewardKind>,
}

/// Registry of available equipment items
#[derive(Resource, Debug)]
pub struct EquipmentRegistry {
    pub items: Vec<EquipmentItem>,
}

#[derive(Debug, Clone)]
pub struct EquipmentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slot: EquipmentSlot,
    pub tier: RewardTier,
}

impl Default for EquipmentRegistry {
    fn default() -> Self {
        Self {
            items: vec![
                // Helmets
                EquipmentItem {
                    id: "helm_cloth".to_string(),
                    name: "Cloth Hood".to_string(),
                    description: "Basic head protection".to_string(),
                    slot: EquipmentSlot::Helmet,
                    tier: RewardTier::TierOne,
                },
                EquipmentItem {
                    id: "helm_iron".to_string(),
                    name: "Iron Helm".to_string(),
                    description: "A sturdy iron helmet".to_string(),
                    slot: EquipmentSlot::Helmet,
                    tier: RewardTier::TierTwo,
                },
                EquipmentItem {
                    id: "helm_guardian".to_string(),
                    name: "Guardian's Crown".to_string(),
                    description: "Blessed by ancient protectors".to_string(),
                    slot: EquipmentSlot::Helmet,
                    tier: RewardTier::TierThree,
                },
                EquipmentItem {
                    id: "helm_dragon".to_string(),
                    name: "Dragonscale Helm".to_string(),
                    description: "Forged from dragon scales".to_string(),
                    slot: EquipmentSlot::Helmet,
                    tier: RewardTier::TierFour,
                },
                EquipmentItem {
                    id: "helm_olympian".to_string(),
                    name: "Olympian Visage".to_string(),
                    description: "Worn by the gods themselves".to_string(),
                    slot: EquipmentSlot::Helmet,
                    tier: RewardTier::TierFive,
                },
                // Chestplates
                EquipmentItem {
                    id: "chest_cloth".to_string(),
                    name: "Padded Shirt".to_string(),
                    description: "Light padding for basic defense".to_string(),
                    slot: EquipmentSlot::Chestplate,
                    tier: RewardTier::TierOne,
                },
                EquipmentItem {
                    id: "chest_leather".to_string(),
                    name: "Leather Cuirass".to_string(),
                    description: "Light but protective".to_string(),
                    slot: EquipmentSlot::Chestplate,
                    tier: RewardTier::TierTwo,
                },
                EquipmentItem {
                    id: "chest_plate".to_string(),
                    name: "Steel Breastplate".to_string(),
                    description: "Heavy duty protection".to_string(),
                    slot: EquipmentSlot::Chestplate,
                    tier: RewardTier::TierThree,
                },
                EquipmentItem {
                    id: "chest_titan".to_string(),
                    name: "Titan's Aegis".to_string(),
                    description: "Impenetrable titan armor".to_string(),
                    slot: EquipmentSlot::Chestplate,
                    tier: RewardTier::TierFour,
                },
                EquipmentItem {
                    id: "chest_divine".to_string(),
                    name: "Divine Raiment".to_string(),
                    description: "Woven from celestial threads".to_string(),
                    slot: EquipmentSlot::Chestplate,
                    tier: RewardTier::TierFive,
                },
                // Greaves
                EquipmentItem {
                    id: "greaves_cloth".to_string(),
                    name: "Padded Greaves".to_string(),
                    description: "Comfortable leg protection".to_string(),
                    slot: EquipmentSlot::Greaves,
                    tier: RewardTier::TierOne,
                },
                EquipmentItem {
                    id: "greaves_chain".to_string(),
                    name: "Chainmail Greaves".to_string(),
                    description: "Flexible metal protection".to_string(),
                    slot: EquipmentSlot::Greaves,
                    tier: RewardTier::TierTwo,
                },
                EquipmentItem {
                    id: "greaves_plate".to_string(),
                    name: "Plate Greaves".to_string(),
                    description: "Full metal leg armor".to_string(),
                    slot: EquipmentSlot::Greaves,
                    tier: RewardTier::TierThree,
                },
                EquipmentItem {
                    id: "greaves_storm".to_string(),
                    name: "Stormforged Greaves".to_string(),
                    description: "Crackling with lightning".to_string(),
                    slot: EquipmentSlot::Greaves,
                    tier: RewardTier::TierFour,
                },
                EquipmentItem {
                    id: "greaves_eternal".to_string(),
                    name: "Eternal Legguards".to_string(),
                    description: "Unbending, unbreaking".to_string(),
                    slot: EquipmentSlot::Greaves,
                    tier: RewardTier::TierFive,
                },
                // Boots
                EquipmentItem {
                    id: "boots_leather".to_string(),
                    name: "Leather Boots".to_string(),
                    description: "Simple walking boots".to_string(),
                    slot: EquipmentSlot::Boots,
                    tier: RewardTier::TierOne,
                },
                EquipmentItem {
                    id: "boots_swift".to_string(),
                    name: "Swift Boots".to_string(),
                    description: "Light and fast".to_string(),
                    slot: EquipmentSlot::Boots,
                    tier: RewardTier::TierTwo,
                },
                EquipmentItem {
                    id: "boots_warden".to_string(),
                    name: "Warden's Treads".to_string(),
                    description: "Sturdy and reliable".to_string(),
                    slot: EquipmentSlot::Boots,
                    tier: RewardTier::TierThree,
                },
                EquipmentItem {
                    id: "boots_wind".to_string(),
                    name: "Windwalkers".to_string(),
                    description: "Move like the wind itself".to_string(),
                    slot: EquipmentSlot::Boots,
                    tier: RewardTier::TierFour,
                },
                EquipmentItem {
                    id: "boots_hermes".to_string(),
                    name: "Hermes' Sandals".to_string(),
                    description: "Wings at your feet".to_string(),
                    slot: EquipmentSlot::Boots,
                    tier: RewardTier::TierFive,
                },
                // Main Hand
                EquipmentItem {
                    id: "sword_rusty".to_string(),
                    name: "Rusty Blade".to_string(),
                    description: "A worn but serviceable sword".to_string(),
                    slot: EquipmentSlot::MainHand,
                    tier: RewardTier::TierOne,
                },
                EquipmentItem {
                    id: "sword_iron".to_string(),
                    name: "Iron Sword".to_string(),
                    description: "A reliable blade".to_string(),
                    slot: EquipmentSlot::MainHand,
                    tier: RewardTier::TierTwo,
                },
                EquipmentItem {
                    id: "sword_flame".to_string(),
                    name: "Flamebrand".to_string(),
                    description: "Burns with inner fire".to_string(),
                    slot: EquipmentSlot::MainHand,
                    tier: RewardTier::TierThree,
                },
                EquipmentItem {
                    id: "sword_void".to_string(),
                    name: "Voidedge".to_string(),
                    description: "Cuts through reality".to_string(),
                    slot: EquipmentSlot::MainHand,
                    tier: RewardTier::TierFour,
                },
                EquipmentItem {
                    id: "sword_zeus".to_string(),
                    name: "Zeus' Thunderbolt".to_string(),
                    description: "The storm incarnate".to_string(),
                    slot: EquipmentSlot::MainHand,
                    tier: RewardTier::TierFive,
                },
            ],
        }
    }
}

/// Registry of available skill tree nodes
#[derive(Resource, Debug)]
pub struct SkillNodeRegistry {
    pub nodes: Vec<SkillNode>,
}

#[derive(Debug, Clone)]
pub struct SkillNode {
    pub id: String,
    pub tree_id: String,
    pub name: String,
    pub description: String,
    pub tier: RewardTier,
    pub prerequisites: Vec<String>,
}

impl Default for SkillNodeRegistry {
    fn default() -> Self {
        Self {
            nodes: vec![
                // Warrior tree
                SkillNode {
                    id: "warrior_strength_1".to_string(),
                    tree_id: "warrior".to_string(),
                    name: "Brute Force".to_string(),
                    description: "Increase base damage by 15%".to_string(),
                    tier: RewardTier::TierOne,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "warrior_vitality_1".to_string(),
                    tree_id: "warrior".to_string(),
                    name: "Iron Constitution".to_string(),
                    description: "Increase max health by 20".to_string(),
                    tier: RewardTier::TierOne,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "warrior_combo_1".to_string(),
                    tree_id: "warrior".to_string(),
                    name: "Combo Master".to_string(),
                    description: "Light attacks chain faster".to_string(),
                    tier: RewardTier::TierTwo,
                    prerequisites: vec!["warrior_strength_1".to_string()],
                },
                SkillNode {
                    id: "warrior_rage_1".to_string(),
                    tree_id: "warrior".to_string(),
                    name: "Berserker Rage".to_string(),
                    description: "Gain damage as health decreases".to_string(),
                    tier: RewardTier::TierThree,
                    prerequisites: vec!["warrior_combo_1".to_string()],
                },
                SkillNode {
                    id: "warrior_immortal".to_string(),
                    tree_id: "warrior".to_string(),
                    name: "Undying Will".to_string(),
                    description: "Survive fatal damage once per room".to_string(),
                    tier: RewardTier::TierFive,
                    prerequisites: vec!["warrior_rage_1".to_string()],
                },
                // Rogue tree
                SkillNode {
                    id: "rogue_speed_1".to_string(),
                    tree_id: "rogue".to_string(),
                    name: "Quick Step".to_string(),
                    description: "Dash cooldown reduced".to_string(),
                    tier: RewardTier::TierOne,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "rogue_crit_1".to_string(),
                    tree_id: "rogue".to_string(),
                    name: "Precision Strike".to_string(),
                    description: "Critical hit chance +10%".to_string(),
                    tier: RewardTier::TierOne,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "rogue_shadow_1".to_string(),
                    tree_id: "rogue".to_string(),
                    name: "Shadow Dance".to_string(),
                    description: "Brief invulnerability after dash".to_string(),
                    tier: RewardTier::TierTwo,
                    prerequisites: vec!["rogue_speed_1".to_string()],
                },
                SkillNode {
                    id: "rogue_assassin".to_string(),
                    tree_id: "rogue".to_string(),
                    name: "Assassinate".to_string(),
                    description: "Massive damage to low health enemies".to_string(),
                    tier: RewardTier::TierFour,
                    prerequisites: vec!["rogue_crit_1".to_string()],
                },
                // Mage tree
                SkillNode {
                    id: "mage_power_1".to_string(),
                    tree_id: "mage".to_string(),
                    name: "Arcane Focus".to_string(),
                    description: "Special attacks deal 20% more damage".to_string(),
                    tier: RewardTier::TierOne,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "mage_regen_1".to_string(),
                    tree_id: "mage".to_string(),
                    name: "Mana Siphon".to_string(),
                    description: "Recover stamina on kills".to_string(),
                    tier: RewardTier::TierTwo,
                    prerequisites: vec!["mage_power_1".to_string()],
                },
                SkillNode {
                    id: "mage_nova".to_string(),
                    tree_id: "mage".to_string(),
                    name: "Arcane Nova".to_string(),
                    description: "Charged special creates explosion".to_string(),
                    tier: RewardTier::TierThree,
                    prerequisites: vec!["mage_regen_1".to_string()],
                },
                SkillNode {
                    id: "mage_transcend".to_string(),
                    tree_id: "mage".to_string(),
                    name: "Transcendence".to_string(),
                    description: "Transform into pure energy briefly".to_string(),
                    tier: RewardTier::TierFive,
                    prerequisites: vec!["mage_nova".to_string()],
                },
            ],
        }
    }
}

/// Listen for boss defeated and transition to reward state
pub(crate) fn handle_boss_defeated_for_reward(
    mut boss_defeated_events: MessageReader<BossDefeatedEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
    mut reward_events: MessageWriter<RewardOfferedEvent>,
    mut coin_events: MessageWriter<CoinGainedEvent>,
    run_config: Res<RunConfig>,
    difficulty: Res<DifficultyScaling>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    player_build: Res<PlayerBuild>,
    mut current_choices: ResMut<CurrentRewardChoices>,
) {
    for _event in boss_defeated_events.read() {
        let tier_bonus = difficulty.reward_tier_bonus(run_config.segment_index);

        let mut rng = ChaCha8Rng::seed_from_u64(
            run_config
                .seed
                .wrapping_add(run_config.segment_index as u64),
        );

        let choices = generate_data_driven_choices(
            &content_registry,
            &gameplay_defaults,
            &player_build,
            &mut rng,
            tier_bonus,
        );

        current_choices.choices = choices.clone();

        reward_events.write(RewardOfferedEvent { choices });

        let base_boss_coins = 50u32;
        let scaled_coins =
            (base_boss_coins as f32 * (1.0 + run_config.segment_index as f32 * 0.2)) as u32;
        coin_events.write(CoinGainedEvent {
            amount: scaled_coins,
            source: CoinSource::BossReward,
        });

        info!(
            "Offering rewards at segment {} with tier_bonus: {:.2}. Boss coins: {}",
            run_config.segment_index, tier_bonus, scaled_coins
        );

        next_run_state.set(RunState::Reward);
    }
}

/// Generate 3 random reward choices based on run seed
fn generate_reward_choices(
    run_config: &RunConfig,
    equipment_registry: &EquipmentRegistry,
    skill_registry: &SkillNodeRegistry,
    player_build: &PlayerBuild,
    tier_bonus: f32,
) -> Vec<RewardKind> {
    let mut rng = ChaCha8Rng::seed_from_u64(
        run_config
            .seed
            .wrapping_add(run_config.segment_index as u64),
    );
    let mut choices = Vec::with_capacity(3);

    let mut available_types = vec![0, 1, 2];

    for _ in 0..3 {
        if available_types.is_empty() {
            available_types = vec![0, 1, 2];
        }

        let type_index = rng.random_range(0..available_types.len());
        let reward_type = available_types.remove(type_index);

        let reward = match reward_type {
            0 => generate_skill_reward(&mut rng, skill_registry, player_build, tier_bonus),
            1 => generate_equipment_reward(&mut rng, equipment_registry, player_build, tier_bonus),
            _ => generate_stat_reward(&mut rng, tier_bonus),
        };

        if let Some(r) = reward {
            choices.push(r);
        } else {
            choices.push(generate_stat_reward(&mut rng, tier_bonus).unwrap());
        }
    }

    choices
}

/// Select a random tier based on drop weights with tier bonus shifting toward higher tiers
fn roll_tier(rng: &mut ChaCha8Rng, tier_bonus: f32) -> RewardTier {
    let tiers = [
        RewardTier::TierOne,
        RewardTier::TierTwo,
        RewardTier::TierThree,
        RewardTier::TierFour,
        RewardTier::TierFive,
    ];

    let adjusted_weights: Vec<f32> = tiers
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let base_weight = t.drop_weight();
            let adjustment = (i as f32 - 2.0) * tier_bonus * 10.0;
            (base_weight + adjustment).max(1.0)
        })
        .collect();

    let total_weight: f32 = adjusted_weights.iter().sum();
    let mut roll: f32 = rng.random_range(0.0..total_weight);

    for (i, tier) in tiers.iter().enumerate() {
        roll -= adjusted_weights[i];
        if roll <= 0.0 {
            return *tier;
        }
    }

    RewardTier::TierOne
}

fn generate_skill_reward(
    rng: &mut ChaCha8Rng,
    registry: &SkillNodeRegistry,
    player_build: &PlayerBuild,
    tier_bonus: f32,
) -> Option<RewardKind> {
    let available: Vec<_> = registry
        .nodes
        .iter()
        .filter(|node| {
            !player_build.unlocked_nodes.contains(&node.id)
                && node
                    .prerequisites
                    .iter()
                    .all(|prereq| player_build.unlocked_nodes.contains(prereq))
        })
        .collect();

    if available.is_empty() {
        return None;
    }

    let adjusted_weights: Vec<f32> = available
        .iter()
        .map(|n| {
            let base_weight = n.tier.drop_weight();
            let tier_level = n.tier.level() as f32;
            let adjustment = (tier_level - 3.0) * tier_bonus * 5.0;
            (base_weight + adjustment).max(1.0)
        })
        .collect();

    let total_weight: f32 = adjusted_weights.iter().sum();
    let mut roll: f32 = rng.random_range(0.0..total_weight);

    for (i, node) in available.iter().enumerate() {
        roll -= adjusted_weights[i];
        if roll <= 0.0 {
            return Some(RewardKind::SkillTreeNode {
                tree_id: node.tree_id.clone(),
                node_id: node.id.clone(),
                name: node.name.clone(),
                description: node.description.clone(),
                tier: node.tier,
            });
        }
    }

    let node = available[0];
    Some(RewardKind::SkillTreeNode {
        tree_id: node.tree_id.clone(),
        node_id: node.id.clone(),
        name: node.name.clone(),
        description: node.description.clone(),
        tier: node.tier,
    })
}

fn generate_equipment_reward(
    rng: &mut ChaCha8Rng,
    registry: &EquipmentRegistry,
    player_build: &PlayerBuild,
    tier_bonus: f32,
) -> Option<RewardKind> {
    let equipped_ids: Vec<&str> = [
        player_build.equipment.helmet.as_deref(),
        player_build.equipment.chestplate.as_deref(),
        player_build.equipment.greaves.as_deref(),
        player_build.equipment.boots.as_deref(),
        player_build.equipment.main_hand.as_deref(),
    ]
    .iter()
    .filter_map(|x| *x)
    .collect();

    let available: Vec<_> = registry
        .items
        .iter()
        .filter(|item| !equipped_ids.contains(&item.id.as_str()))
        .collect();

    if available.is_empty() {
        return None;
    }

    let adjusted_weights: Vec<f32> = available
        .iter()
        .map(|item| {
            let base_weight = item.tier.drop_weight();
            let tier_level = item.tier.level() as f32;
            let adjustment = (tier_level - 3.0) * tier_bonus * 5.0;
            (base_weight + adjustment).max(1.0)
        })
        .collect();

    let total_weight: f32 = adjusted_weights.iter().sum();
    let mut roll: f32 = rng.random_range(0.0..total_weight);

    for (i, item) in available.iter().enumerate() {
        roll -= adjusted_weights[i];
        if roll <= 0.0 {
            return Some(RewardKind::Equipment {
                slot: item.slot,
                item_id: item.id.clone(),
                name: item.name.clone(),
                description: item.description.clone(),
                tier: item.tier,
            });
        }
    }

    let item = available[0];
    Some(RewardKind::Equipment {
        slot: item.slot,
        item_id: item.id.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        tier: item.tier,
    })
}

fn generate_stat_reward(rng: &mut ChaCha8Rng, tier_bonus: f32) -> Option<RewardKind> {
    let stat_type = match rng.random_range(0..3) {
        0 => StatType::MaxHealth,
        1 => StatType::Stamina,
        _ => StatType::AttackPower,
    };

    let tier = roll_tier(rng, tier_bonus);
    let multiplier = tier.power_multiplier();

    let base_amount = match stat_type {
        StatType::MaxHealth => rng.random_range(1..=2) as f32 * 10.0,
        StatType::Stamina => rng.random_range(1..=2) as f32 * 10.0,
        StatType::AttackPower => rng.random_range(1..=2) as f32 * 2.0,
    };

    let amount = (base_amount * multiplier).round();

    Some(RewardKind::StatUpgrade {
        stat: stat_type,
        amount,
        tier,
    })
}

/// Convert content EquipmentSlot to rewards EquipmentSlot
pub(crate) fn convert_equipment_slot(slot: crate::content::EquipmentSlot) -> EquipmentSlot {
    match slot {
        crate::content::EquipmentSlot::Helmet => EquipmentSlot::Helmet,
        crate::content::EquipmentSlot::Chestplate => EquipmentSlot::Chestplate,
        crate::content::EquipmentSlot::Gloves => EquipmentSlot::Greaves,
        crate::content::EquipmentSlot::Boots => EquipmentSlot::Boots,
        crate::content::EquipmentSlot::Accessory => EquipmentSlot::MainHand,
    }
}

/// Convert content StatKind to rewards StatType
fn convert_stat_kind(stat: ContentStatKind) -> StatType {
    match stat {
        ContentStatKind::MaxHealth => StatType::MaxHealth,
        ContentStatKind::AttackPower => StatType::AttackPower,
        ContentStatKind::MoveSpeed => StatType::Stamina,
    }
}

/// Generate reward choices using ContentRegistry data
fn generate_data_driven_choices(
    registry: &ContentRegistry,
    _gameplay_defaults: &GameplayDefaults,
    player_build: &PlayerBuild,
    rng: &mut ChaCha8Rng,
    tier_bonus: f32,
) -> Vec<RewardKind> {
    let mut choices = Vec::with_capacity(3);

    if let Some(reward) = generate_blessing_from_registry(registry, player_build, rng, tier_bonus) {
        choices.push(reward);
    }

    if let Some(reward) = generate_equipment_from_registry(registry, player_build, rng, tier_bonus)
    {
        choices.push(reward);
    }

    while choices.len() < 3 {
        if let Some(reward) = generate_stat_reward(rng, tier_bonus) {
            choices.push(reward);
        }
    }

    choices
}

/// Generate a blessing reward from ContentRegistry
fn generate_blessing_from_registry(
    registry: &ContentRegistry,
    player_build: &PlayerBuild,
    rng: &mut ChaCha8Rng,
    tier_bonus: f32,
) -> Option<RewardKind> {
    let blessings: Vec<&BlessingDef> = if let Some(god_id) = &player_build.parent_god_id {
        let god_blessings: Vec<_> = registry
            .blessings
            .values()
            .filter(|b| &b.god_id == god_id)
            .collect();

        if god_blessings.is_empty() {
            registry.blessings.values().collect()
        } else {
            god_blessings
        }
    } else {
        registry.blessings.values().collect()
    };

    if blessings.is_empty() {
        return None;
    }

    let available: Vec<_> = blessings
        .iter()
        .filter(|b| !player_build.unlocked_nodes.contains(&b.id))
        .collect();

    if available.is_empty() {
        return None;
    }

    let weights: Vec<f32> = available
        .iter()
        .map(|b| {
            let tier = RewardTier::from_level(b.tier as u8);
            let base_weight = tier.drop_weight();
            let adjustment = (b.tier as f32 - 3.0) * tier_bonus * 5.0;
            (base_weight + adjustment).max(1.0)
        })
        .collect();

    let total: f32 = weights.iter().sum();
    let mut roll = rng.random_range(0.0..total);

    for (i, blessing) in available.iter().enumerate() {
        roll -= weights[i];
        if roll <= 0.0 {
            return Some(RewardKind::SkillTreeNode {
                tree_id: blessing.god_id.clone(),
                node_id: blessing.id.clone(),
                name: blessing.name.clone(),
                description: blessing.description.clone(),
                tier: RewardTier::from_level(blessing.tier as u8),
            });
        }
    }

    let blessing = available[0];
    Some(RewardKind::SkillTreeNode {
        tree_id: blessing.god_id.clone(),
        node_id: blessing.id.clone(),
        name: blessing.name.clone(),
        description: blessing.description.clone(),
        tier: RewardTier::from_level(blessing.tier as u8),
    })
}

/// Generate an equipment reward from ContentRegistry
fn generate_equipment_from_registry(
    registry: &ContentRegistry,
    player_build: &PlayerBuild,
    rng: &mut ChaCha8Rng,
    tier_bonus: f32,
) -> Option<RewardKind> {
    let equipment: Vec<&EquipmentItemDef> = registry.equipment_items.values().collect();

    if equipment.is_empty() {
        return None;
    }

    let equipped_ids: Vec<&str> = [
        player_build.equipment.helmet.as_deref(),
        player_build.equipment.chestplate.as_deref(),
        player_build.equipment.greaves.as_deref(),
        player_build.equipment.boots.as_deref(),
        player_build.equipment.main_hand.as_deref(),
    ]
    .iter()
    .filter_map(|x| *x)
    .collect();

    let available: Vec<_> = equipment
        .iter()
        .filter(|e| !equipped_ids.contains(&e.id.as_str()))
        .collect();

    if available.is_empty() {
        return None;
    }

    let weights: Vec<f32> = available
        .iter()
        .map(|e| {
            let tier = RewardTier::from_level(e.tier as u8);
            let base_weight = tier.drop_weight();
            let adjustment = (e.tier as f32 - 3.0) * tier_bonus * 5.0;
            (base_weight + adjustment).max(1.0)
        })
        .collect();

    let total: f32 = weights.iter().sum();
    let mut roll = rng.random_range(0.0..total);

    for (i, equip) in available.iter().enumerate() {
        roll -= weights[i];
        if roll <= 0.0 {
            return Some(RewardKind::Equipment {
                slot: convert_equipment_slot(equip.slot),
                item_id: equip.id.clone(),
                name: equip.name.clone(),
                description: format!(
                    "+{:.0} HP, {:.1}% DR",
                    equip.base_stats.max_health_bonus,
                    equip.base_stats.damage_reduction * 100.0
                ),
                tier: RewardTier::from_level(equip.tier as u8),
            });
        }
    }

    let equip = available[0];
    Some(RewardKind::Equipment {
        slot: convert_equipment_slot(equip.slot),
        item_id: equip.id.clone(),
        name: equip.name.clone(),
        description: format!(
            "+{:.0} HP, {:.1}% DR",
            equip.base_stats.max_health_bonus,
            equip.base_stats.damage_reduction * 100.0
        ),
        tier: RewardTier::from_level(equip.tier as u8),
    })
}

pub(crate) fn apply_reward_choice(
    mut reward_events: MessageReader<RewardChosenEvent>,
    current_choices: Res<CurrentRewardChoices>,
    mut player_build: ResMut<PlayerBuild>,
    mut run_config: ResMut<RunConfig>,
    mut next_run_state: ResMut<NextState<RunState>>,
    mut run_faith: ResMut<RunFaith>,
    mut faith_events: MessageWriter<FaithChangedEvent>,
    content_registry: Res<ContentRegistry>,
) {
    for event in reward_events.read() {
        if event.choice_index >= current_choices.choices.len() {
            continue;
        }

        let choice = &current_choices.choices[event.choice_index];

        let mut god_blessings_available: Vec<String> = Vec::new();
        let mut chosen_god_id: Option<String> = None;

        for (i, reward) in current_choices.choices.iter().enumerate() {
            if let RewardKind::SkillTreeNode { tree_id, .. } = reward {
                if content_registry.gods.contains_key(tree_id) {
                    god_blessings_available.push(tree_id.clone());
                    if i == event.choice_index {
                        chosen_god_id = Some(tree_id.clone());
                    }
                }
            }
        }

        if !god_blessings_available.is_empty() {
            if let Some(ref chosen) = chosen_god_id {
                run_faith.modify_faith(chosen, FAITH_GAIN_BLESSING_CHOSEN, "Blessing chosen");
                faith_events.write(FaithChangedEvent {
                    god_id: chosen.clone(),
                    delta: FAITH_GAIN_BLESSING_CHOSEN,
                    new_value: run_faith.get_faith(chosen),
                    reason: "Blessing chosen".to_string(),
                });
                info!(
                    "Faith +{} with {} (blessing chosen). New faith: {}",
                    FAITH_GAIN_BLESSING_CHOSEN,
                    chosen,
                    run_faith.get_faith(chosen)
                );

                for god_id in &god_blessings_available {
                    if god_id != chosen {
                        run_faith.modify_faith(
                            god_id,
                            FAITH_LOSS_BLESSING_REJECTED,
                            "Blessing rejected",
                        );
                        faith_events.write(FaithChangedEvent {
                            god_id: god_id.clone(),
                            delta: FAITH_LOSS_BLESSING_REJECTED,
                            new_value: run_faith.get_faith(god_id),
                            reason: "Blessing rejected".to_string(),
                        });
                        info!(
                            "Faith {} with {} (blessing rejected). New faith: {}",
                            FAITH_LOSS_BLESSING_REJECTED,
                            god_id,
                            run_faith.get_faith(god_id)
                        );
                    }
                }
            } else {
                for god_id in &god_blessings_available {
                    run_faith.modify_faith(
                        god_id,
                        FAITH_LOSS_BLESSING_REJECTED,
                        "Blessing ignored",
                    );
                    faith_events.write(FaithChangedEvent {
                        god_id: god_id.clone(),
                        delta: FAITH_LOSS_BLESSING_REJECTED,
                        new_value: run_faith.get_faith(god_id),
                        reason: "Blessing ignored".to_string(),
                    });
                    info!(
                        "Faith {} with {} (blessing ignored). New faith: {}",
                        FAITH_LOSS_BLESSING_REJECTED,
                        god_id,
                        run_faith.get_faith(god_id)
                    );
                }
            }
        }

        match choice {
            RewardKind::SkillTreeNode { node_id, .. } => {
                if !player_build.unlocked_nodes.contains(node_id) {
                    player_build.unlocked_nodes.push(node_id.clone());
                }
            }
            RewardKind::Equipment { slot, item_id, .. } => match slot {
                EquipmentSlot::Helmet => player_build.equipment.helmet = Some(item_id.clone()),
                EquipmentSlot::Chestplate => {
                    player_build.equipment.chestplate = Some(item_id.clone())
                }
                EquipmentSlot::Greaves => player_build.equipment.greaves = Some(item_id.clone()),
                EquipmentSlot::Boots => player_build.equipment.boots = Some(item_id.clone()),
                EquipmentSlot::MainHand => player_build.equipment.main_hand = Some(item_id.clone()),
            },
            RewardKind::StatUpgrade { stat, amount, .. } => match stat {
                StatType::MaxHealth => player_build.stats.max_health += amount,
                StatType::Stamina => player_build.stats.stamina += amount,
                StatType::AttackPower => player_build.stats.attack_power += amount,
            },
        }

        run_config.segment_index += 1;

        next_run_state.set(RunState::Arena);
    }
}
