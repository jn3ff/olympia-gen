use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::combat::BossDefeatedEvent;
use crate::core::{DifficultyScaling, RunConfig, RunState};

// ============================================================================
// Components
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct PlayerBuild {
    pub equipment: EquipmentLoadout,
    pub stats: BaseStats,
    pub unlocked_nodes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EquipmentLoadout {
    pub helmet: Option<String>,
    pub chestplate: Option<String>,
    pub greaves: Option<String>,
    pub boots: Option<String>,
    pub main_hand: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BaseStats {
    pub max_health: f32,
    pub stamina: f32,
    pub attack_power: f32,
}

impl Default for BaseStats {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            stamina: 100.0,
            attack_power: 10.0,
        }
    }
}

// ============================================================================
// Reward Tier System
// ============================================================================

/// Reward tier classification - determines rarity, power, and visual treatment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RewardTier {
    /// Common rewards - baseline power
    #[default]
    TierOne,
    /// Uncommon rewards - slightly better
    TierTwo,
    /// Rare rewards - notably stronger
    TierThree,
    /// Epic rewards - powerful effects
    TierFour,
    /// Legendary rewards - exceptional power
    TierFive,
}

impl RewardTier {
    /// Get the internal tier number (1-5)
    pub fn level(&self) -> u8 {
        match self {
            RewardTier::TierOne => 1,
            RewardTier::TierTwo => 2,
            RewardTier::TierThree => 3,
            RewardTier::TierFour => 4,
            RewardTier::TierFive => 5,
        }
    }

    /// Get tier from a numeric level (clamped to valid range)
    pub fn from_level(level: u8) -> Self {
        match level {
            0 | 1 => RewardTier::TierOne,
            2 => RewardTier::TierTwo,
            3 => RewardTier::TierThree,
            4 => RewardTier::TierFour,
            _ => RewardTier::TierFive,
        }
    }

    /// Placeholder name for the tier (to be replaced with ornament terminology later)
    pub fn name(&self) -> &str {
        match self {
            RewardTier::TierOne => "tier_one",
            RewardTier::TierTwo => "tier_two",
            RewardTier::TierThree => "tier_three",
            RewardTier::TierFour => "tier_four",
            RewardTier::TierFive => "tier_five",
        }
    }

    /// Display name for UI (can be customized later)
    pub fn display_name(&self) -> &str {
        match self {
            RewardTier::TierOne => "Common",
            RewardTier::TierTwo => "Uncommon",
            RewardTier::TierThree => "Rare",
            RewardTier::TierFour => "Epic",
            RewardTier::TierFive => "Legendary",
        }
    }

    /// Brightness/saturation modifier for this tier (applied to base reward color)
    /// Higher tiers are brighter and more saturated
    pub fn color_modifier(&self) -> TierColorModifier {
        match self {
            RewardTier::TierOne => TierColorModifier {
                brightness: 0.7,
                saturation: 0.8,
            },
            RewardTier::TierTwo => TierColorModifier {
                brightness: 0.85,
                saturation: 0.9,
            },
            RewardTier::TierThree => TierColorModifier {
                brightness: 1.0,
                saturation: 1.0,
            },
            RewardTier::TierFour => TierColorModifier {
                brightness: 1.1,
                saturation: 1.1,
            },
            RewardTier::TierFive => TierColorModifier {
                brightness: 1.25,
                saturation: 1.2,
            },
        }
    }

    /// Get the tier's accent color (for borders, glows, etc.)
    pub fn accent_color(&self) -> Color {
        match self {
            RewardTier::TierOne => Color::srgb(0.5, 0.5, 0.5),   // Gray
            RewardTier::TierTwo => Color::srgb(0.3, 0.7, 0.3),   // Green
            RewardTier::TierThree => Color::srgb(0.3, 0.5, 0.9), // Blue
            RewardTier::TierFour => Color::srgb(0.7, 0.3, 0.9),  // Purple
            RewardTier::TierFive => Color::srgb(1.0, 0.8, 0.2),  // Gold
        }
    }

    /// Power multiplier for this tier (affects stat values, etc.)
    pub fn power_multiplier(&self) -> f32 {
        match self {
            RewardTier::TierOne => 1.0,
            RewardTier::TierTwo => 1.25,
            RewardTier::TierThree => 1.5,
            RewardTier::TierFour => 1.85,
            RewardTier::TierFive => 2.25,
        }
    }

    /// Drop weight for this tier (lower = rarer)
    pub fn drop_weight(&self) -> f32 {
        match self {
            RewardTier::TierOne => 40.0,
            RewardTier::TierTwo => 30.0,
            RewardTier::TierThree => 18.0,
            RewardTier::TierFour => 9.0,
            RewardTier::TierFive => 3.0,
        }
    }
}

/// Color modification values for tier-based coloring
#[derive(Debug, Clone, Copy)]
pub struct TierColorModifier {
    /// Multiplier for color brightness (1.0 = no change)
    pub brightness: f32,
    /// Multiplier for color saturation (1.0 = no change)
    pub saturation: f32,
}

impl TierColorModifier {
    /// Apply this modifier to a base color
    pub fn apply(&self, base: Color) -> Color {
        let Srgba { red, green, blue, alpha } = base.to_srgba();

        // Convert to HSL-like space for modification
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let _luminance = (max + min) / 2.0;

        // Apply brightness by lerping toward white or black
        let brightness_adjusted = if self.brightness > 1.0 {
            // Brighten: lerp toward white
            let factor = self.brightness - 1.0;
            (
                red + (1.0 - red) * factor,
                green + (1.0 - green) * factor,
                blue + (1.0 - blue) * factor,
            )
        } else {
            // Darken: lerp toward black
            let factor = self.brightness;
            (red * factor, green * factor, blue * factor)
        };

        // Apply saturation by lerping toward gray
        let gray = (brightness_adjusted.0 + brightness_adjusted.1 + brightness_adjusted.2) / 3.0;
        let (r, g, b) = if self.saturation > 1.0 {
            // Increase saturation: move away from gray
            let factor = self.saturation - 1.0;
            (
                brightness_adjusted.0 + (brightness_adjusted.0 - gray) * factor,
                brightness_adjusted.1 + (brightness_adjusted.1 - gray) * factor,
                brightness_adjusted.2 + (brightness_adjusted.2 - gray) * factor,
            )
        } else {
            // Decrease saturation: move toward gray
            let factor = self.saturation;
            (
                gray + (brightness_adjusted.0 - gray) * factor,
                gray + (brightness_adjusted.1 - gray) * factor,
                gray + (brightness_adjusted.2 - gray) * factor,
            )
        };

        Color::srgba(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), alpha)
    }
}

// ============================================================================
// Reward Types
// ============================================================================

/// The category of reward (determines base color)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardCategory {
    Skill,
    Equipment,
    Stat,
}

impl RewardCategory {
    /// Base color for this reward category
    pub fn base_color(&self) -> Color {
        match self {
            RewardCategory::Skill => Color::srgb(0.6, 0.4, 0.9),     // Purple
            RewardCategory::Equipment => Color::srgb(0.9, 0.7, 0.2), // Gold
            RewardCategory::Stat => Color::srgb(0.3, 0.8, 0.4),      // Green
        }
    }
}

#[derive(Debug, Clone)]
pub enum RewardKind {
    SkillTreeNode {
        tree_id: String,
        node_id: String,
        name: String,
        description: String,
        tier: RewardTier,
    },
    Equipment {
        slot: EquipmentSlot,
        item_id: String,
        name: String,
        description: String,
        tier: RewardTier,
    },
    StatUpgrade {
        stat: StatType,
        amount: f32,
        tier: RewardTier,
    },
}

impl RewardKind {
    pub fn name(&self) -> String {
        match self {
            RewardKind::SkillTreeNode { name, .. } => name.clone(),
            RewardKind::Equipment { name, .. } => name.clone(),
            RewardKind::StatUpgrade { stat, amount, .. } => {
                format!("+{} {}", amount, stat.name())
            }
        }
    }

    pub fn description(&self) -> String {
        match self {
            RewardKind::SkillTreeNode { description, .. } => description.clone(),
            RewardKind::Equipment { description, .. } => description.clone(),
            RewardKind::StatUpgrade { stat, amount, .. } => {
                format!("Permanently increase {} by {}", stat.name(), amount)
            }
        }
    }

    pub fn tier(&self) -> RewardTier {
        match self {
            RewardKind::SkillTreeNode { tier, .. } => *tier,
            RewardKind::Equipment { tier, .. } => *tier,
            RewardKind::StatUpgrade { tier, .. } => *tier,
        }
    }

    pub fn category(&self) -> RewardCategory {
        match self {
            RewardKind::SkillTreeNode { .. } => RewardCategory::Skill,
            RewardKind::Equipment { .. } => RewardCategory::Equipment,
            RewardKind::StatUpgrade { .. } => RewardCategory::Stat,
        }
    }

    /// Get the icon color adjusted for tier
    pub fn icon_color(&self) -> Color {
        let base = self.category().base_color();
        self.tier().color_modifier().apply(base)
    }

    /// Get the tier accent color (for borders, decorations)
    pub fn tier_accent_color(&self) -> Color {
        self.tier().accent_color()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Helmet,
    Chestplate,
    Greaves,
    Boots,
    MainHand,
}

impl EquipmentSlot {
    pub fn name(&self) -> &str {
        match self {
            EquipmentSlot::Helmet => "Helmet",
            EquipmentSlot::Chestplate => "Chestplate",
            EquipmentSlot::Greaves => "Greaves",
            EquipmentSlot::Boots => "Boots",
            EquipmentSlot::MainHand => "Main Hand",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    MaxHealth,
    Stamina,
    AttackPower,
}

impl StatType {
    pub fn name(&self) -> &str {
        match self {
            StatType::MaxHealth => "Max Health",
            StatType::Stamina => "Stamina",
            StatType::AttackPower => "Attack Power",
        }
    }
}

// ============================================================================
// Events
// ============================================================================

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

// ============================================================================
// Resources
// ============================================================================

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

// ============================================================================
// UI Components
// ============================================================================

/// Marker for the reward selection UI root
#[derive(Component, Debug)]
pub struct RewardUI;

/// Marker for a reward choice button
#[derive(Component, Debug)]
pub struct RewardChoiceButton {
    pub index: usize,
}

/// Marker for the skip reward button
#[derive(Component, Debug)]
pub struct SkipRewardButton;

// ============================================================================
// Plugin
// ============================================================================

pub struct RewardsPlugin;

impl Plugin for RewardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerBuild>()
            .init_resource::<CurrentRewardChoices>()
            .init_resource::<EquipmentRegistry>()
            .init_resource::<SkillNodeRegistry>()
            .add_message::<RewardOfferedEvent>()
            .add_message::<RewardChosenEvent>()
            .add_systems(
                Update,
                handle_boss_defeated_for_reward.run_if(in_state(RunState::Room)),
            )
            .add_systems(OnEnter(RunState::Reward), spawn_reward_ui)
            .add_systems(OnExit(RunState::Reward), cleanup_reward_ui)
            .add_systems(
                Update,
                (
                    handle_reward_choice_interaction,
                    handle_reward_keyboard_input,
                    apply_reward_choice,
                )
                    .chain()
                    .run_if(in_state(RunState::Reward)),
            );
    }
}

// ============================================================================
// Systems
// ============================================================================

/// Listen for boss defeated and transition to reward state
fn handle_boss_defeated_for_reward(
    mut boss_defeated_events: MessageReader<BossDefeatedEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
    mut reward_events: MessageWriter<RewardOfferedEvent>,
    run_config: Res<RunConfig>,
    difficulty: Res<DifficultyScaling>,
    equipment_registry: Res<EquipmentRegistry>,
    skill_registry: Res<SkillNodeRegistry>,
    player_build: Res<PlayerBuild>,
    mut current_choices: ResMut<CurrentRewardChoices>,
) {
    for _event in boss_defeated_events.read() {
        // Calculate tier bonus based on segment
        let tier_bonus = difficulty.reward_tier_bonus(run_config.segment_index);

        // Generate reward choices
        let choices = generate_reward_choices(
            &run_config,
            &equipment_registry,
            &skill_registry,
            &player_build,
            tier_bonus,
        );

        current_choices.choices = choices.clone();

        reward_events.write(RewardOfferedEvent { choices });

        info!(
            "Offering rewards at segment {} with tier_bonus: {:.2}",
            run_config.segment_index, tier_bonus
        );

        // Transition to reward state
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
    let mut rng = ChaCha8Rng::seed_from_u64(run_config.seed.wrapping_add(run_config.segment_index as u64));
    let mut choices = Vec::with_capacity(3);

    // Determine what types of rewards to offer
    // For variety, we try to offer one of each type when possible
    let mut available_types = vec![0, 1, 2]; // 0=skill, 1=equipment, 2=stat

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
            // Fallback to stat reward if no valid option
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

    // Calculate adjusted weights - tier_bonus shifts probability toward higher tiers
    // Lower tiers get reduced weight, higher tiers get increased weight
    let adjusted_weights: Vec<f32> = tiers
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let base_weight = t.drop_weight();
            // Tier index 0 = lowest, 4 = highest
            // Low tiers get penalty, high tiers get bonus
            let adjustment = (i as f32 - 2.0) * tier_bonus * 10.0;
            (base_weight + adjustment).max(1.0) // Minimum weight of 1
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
    // Filter to nodes not yet unlocked and with satisfied prerequisites
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

    // Use weighted selection based on tier with bonus applied
    let adjusted_weights: Vec<f32> = available
        .iter()
        .map(|n| {
            let base_weight = n.tier.drop_weight();
            let tier_level = n.tier.level() as f32;
            // Higher tier nodes get bonus weight
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

    // Fallback to first available
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
    // Filter to items the player doesn't already have equipped
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

    // Use weighted selection based on tier with bonus applied
    let adjusted_weights: Vec<f32> = available
        .iter()
        .map(|item| {
            let base_weight = item.tier.drop_weight();
            let tier_level = item.tier.level() as f32;
            // Higher tier items get bonus weight
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

    // Fallback to first available
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

    // Roll for tier with bonus applied
    let tier = roll_tier(rng, tier_bonus);
    let multiplier = tier.power_multiplier();

    // Base amounts scaled by tier
    let base_amount = match stat_type {
        StatType::MaxHealth => rng.random_range(1..=2) as f32 * 10.0,  // 10-20 base
        StatType::Stamina => rng.random_range(1..=2) as f32 * 10.0,    // 10-20 base
        StatType::AttackPower => rng.random_range(1..=2) as f32 * 2.0, // 2-4 base
    };

    let amount = (base_amount * multiplier).round();

    Some(RewardKind::StatUpgrade { stat: stat_type, amount, tier })
}

/// Spawn the reward selection UI
fn spawn_reward_ui(mut commands: Commands, current_choices: Res<CurrentRewardChoices>) {
    let bg_color = Color::srgba(0.1, 0.1, 0.15, 0.95);
    let panel_color = Color::srgb(0.15, 0.15, 0.2);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);

    // Root container
    commands
        .spawn((
            RewardUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            ZIndex(100),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("VICTORY!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Choose Your Reward"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Reward choices container
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(20.0),
                    ..default()
                },))
                .with_children(|choices_parent| {
                    for (index, choice) in current_choices.choices.iter().enumerate() {
                        spawn_reward_card(choices_parent, index, choice, panel_color, text_color, muted_text);
                    }
                });

            // Skip button
            parent
                .spawn((
                    SkipRewardButton,
                    Button,
                    Node {
                        margin: UiRect::top(Val::Px(30.0)),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ))
                .with_child((
                    Text::new("Skip [Esc]"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(muted_text),
                ));

            // Keyboard hints
            parent.spawn((
                Text::new("Press 1, 2, or 3 to select"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_reward_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    reward: &RewardKind,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
) {
    let icon_color = reward.icon_color();
    let tier = reward.tier();
    let tier_accent = reward.tier_accent_color();
    let key_hint = format!("[{}]", index + 1);

    // Border thickness increases with tier
    let border_thickness = 2.0 + (tier.level() as f32 - 1.0) * 0.5;

    parent
        .spawn((
            RewardChoiceButton { index },
            Button,
            Node {
                width: Val::Px(220.0),
                min_height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(border_thickness)),
                ..default()
            },
            BorderColor::all(tier_accent),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            // Key hint
            card.spawn((
                Text::new(key_hint),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Tier label with accent color
            card.spawn((
                Text::new(tier.display_name().to_uppercase()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(tier_accent),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Icon/type indicator with tier-colored border
            card.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    margin: UiRect::bottom(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(tier_accent),
                BackgroundColor(icon_color),
            ));

            // Reward type label
            let type_label = match reward {
                RewardKind::SkillTreeNode { .. } => "SKILL",
                RewardKind::Equipment { slot, .. } => slot.name(),
                RewardKind::StatUpgrade { .. } => "STAT BOOST",
            };

            card.spawn((
                Text::new(type_label.to_uppercase()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(icon_color),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Name
            card.spawn((
                Text::new(reward.name()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Description
            card.spawn((
                Text::new(reward.description()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn cleanup_reward_ui(mut commands: Commands, query: Query<Entity, With<RewardUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_reward_choice_interaction(
    mut choice_query: Query<
        (&RewardChoiceButton, &Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, Without<SkipRewardButton>),
    >,
    mut skip_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<SkipRewardButton>, Changed<Interaction>),
    >,
    mut reward_events: MessageWriter<RewardChosenEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
) {
    // Handle reward choice buttons
    for (button, interaction, mut bg_color, mut border_color) in &mut choice_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
                reward_events.write(RewardChosenEvent {
                    choice_index: button.index,
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(Color::srgb(0.3, 0.3, 0.4));
            }
        }
    }

    // Handle skip button
    for (interaction, mut bg_color, mut border_color) in &mut skip_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.35));
                // Skip - go back to arena without applying reward
                next_run_state.set(RunState::Arena);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

fn handle_reward_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut reward_events: MessageWriter<RewardChosenEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
    current_choices: Res<CurrentRewardChoices>,
) {
    // Number keys 1-3 for choices
    if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        if current_choices.choices.len() > 0 {
            reward_events.write(RewardChosenEvent { choice_index: 0 });
        }
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        if current_choices.choices.len() > 1 {
            reward_events.write(RewardChosenEvent { choice_index: 1 });
        }
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        if current_choices.choices.len() > 2 {
            reward_events.write(RewardChosenEvent { choice_index: 2 });
        }
    } else if keyboard.just_pressed(KeyCode::Escape) {
        // Skip reward
        next_run_state.set(RunState::Arena);
    }
}

fn apply_reward_choice(
    mut reward_events: MessageReader<RewardChosenEvent>,
    current_choices: Res<CurrentRewardChoices>,
    mut player_build: ResMut<PlayerBuild>,
    mut run_config: ResMut<RunConfig>,
    mut next_run_state: ResMut<NextState<RunState>>,
) {
    for event in reward_events.read() {
        if event.choice_index >= current_choices.choices.len() {
            continue;
        }

        let choice = &current_choices.choices[event.choice_index];

        match choice {
            RewardKind::SkillTreeNode { node_id, .. } => {
                if !player_build.unlocked_nodes.contains(node_id) {
                    player_build.unlocked_nodes.push(node_id.clone());
                }
            }
            RewardKind::Equipment { slot, item_id, .. } => {
                match slot {
                    EquipmentSlot::Helmet => player_build.equipment.helmet = Some(item_id.clone()),
                    EquipmentSlot::Chestplate => {
                        player_build.equipment.chestplate = Some(item_id.clone())
                    }
                    EquipmentSlot::Greaves => {
                        player_build.equipment.greaves = Some(item_id.clone())
                    }
                    EquipmentSlot::Boots => player_build.equipment.boots = Some(item_id.clone()),
                    EquipmentSlot::MainHand => {
                        player_build.equipment.main_hand = Some(item_id.clone())
                    }
                }
            }
            RewardKind::StatUpgrade { stat, amount, .. } => match stat {
                StatType::MaxHealth => player_build.stats.max_health += amount,
                StatType::Stamina => player_build.stats.stamina += amount,
                StatType::AttackPower => player_build.stats.attack_power += amount,
            },
        }

        // Increment segment for next reward generation
        run_config.segment_index += 1;

        // Transition back to arena
        next_run_state.set(RunState::Arena);
    }
}
