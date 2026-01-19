use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::combat::BossDefeatedEvent;
use crate::content::{
    BlessingDef, ContentRegistry, EquipmentItemDef, GameplayDefaults, StatKind as ContentStatKind,
};
use crate::core::{DifficultyScaling, GameplayPaused, RunConfig, RunState, SegmentCompletedEvent};

pub mod faith;
pub use faith::{
    ADVERSARIAL_MIN_DELAY, AdversarialEventScheduledEvent, FAITH_GAIN_BLESSING_CHOSEN,
    FAITH_LOSS_BLESSING_REJECTED, FaithChangedEvent, RunFaith, TriggerAdversarialEvent,
};

// ============================================================================
// Components
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct PlayerBuild {
    /// Character definition ID (e.g., "character_ares_sword")
    pub character_id: Option<String>,
    /// Parent god ID (e.g., "ares")
    pub parent_god_id: Option<String>,
    /// Current weapon ID
    pub weapon_id: Option<String>,
    /// Current weapon category (e.g., "sword", "spear")
    pub weapon_category: Option<String>,
    /// Current moveset ID
    pub moveset_id: Option<String>,
    /// Equipment loadout
    pub equipment: EquipmentLoadout,
    /// Base stats (from character + equipment)
    pub stats: BaseStats,
    /// Active skills
    pub skills: ActiveSkills,
    /// Movement flags
    pub movement_flags: MovementFlags,
    /// Unlocked skill tree nodes
    pub unlocked_nodes: Vec<String>,
}

impl PlayerBuild {
    /// Initialize build from character data
    pub fn from_character(
        character_id: &str,
        parent_god_id: &str,
        weapon_id: &str,
        weapon_category: &str,
        moveset_id: &str,
        stats: BaseStats,
        skills: ActiveSkills,
        movement_flags: MovementFlags,
    ) -> Self {
        Self {
            character_id: Some(character_id.to_string()),
            parent_god_id: Some(parent_god_id.to_string()),
            weapon_id: Some(weapon_id.to_string()),
            weapon_category: Some(weapon_category.to_string()),
            moveset_id: Some(moveset_id.to_string()),
            stats,
            skills,
            movement_flags,
            equipment: EquipmentLoadout {
                main_hand: Some(weapon_id.to_string()),
                ..default()
            },
            unlocked_nodes: Vec::new(),
        }
    }
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
    pub move_speed_mult: f32,
    pub jump_height_mult: f32,
}

impl Default for BaseStats {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            stamina: 100.0,
            attack_power: 10.0,
            move_speed_mult: 1.0,
            jump_height_mult: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActiveSkills {
    pub passive_id: Option<String>,
    pub common_id: Option<String>,
    pub ultimate_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MovementFlags {
    pub wall_jump: bool,
    pub air_dash_unlocked: bool,
}

// ============================================================================
// Currency System
// ============================================================================

/// Resource tracking player currency within a run
#[derive(Resource, Debug, Default)]
pub struct PlayerWallet {
    pub coins: u32,
}

impl PlayerWallet {
    pub fn add(&mut self, amount: u32) {
        self.coins = self.coins.saturating_add(amount);
    }

    pub fn spend(&mut self, amount: u32) -> bool {
        if self.coins >= amount {
            self.coins -= amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, amount: u32) -> bool {
        self.coins >= amount
    }
}

/// Source of coin gain for tracking/analytics
#[derive(Debug, Clone, Copy)]
pub enum CoinSource {
    EnemyDrop,
    RoomReward,
    BossReward,
    SellItem,
}

/// Event fired when player gains coins
#[derive(Debug)]
pub struct CoinGainedEvent {
    pub amount: u32,
    pub source: CoinSource,
}

impl Message for CoinGainedEvent {}

/// Reason for spending coins
#[derive(Debug, Clone)]
pub enum SpendReason {
    ShopPurchase(String),
    Upgrade(String),
    Enchant(String),
    ShopReroll,
}

/// Event fired when player spends coins
#[derive(Debug)]
pub struct CoinSpentEvent {
    pub amount: u32,
    pub reason: SpendReason,
}

impl Message for CoinSpentEvent {}

// ============================================================================
// Shop System
// ============================================================================

/// Resource tracking the current shop state
#[derive(Resource, Debug, Default)]
pub struct ShopState {
    /// Currently open shop ID (None if no shop is open)
    pub active_shop_id: Option<String>,
    /// Items available for purchase in the current shop (Armory)
    pub inventory: Vec<ShopItem>,
    /// Items available for upgrade (Blacksmith)
    pub upgrade_options: Vec<UpgradeOption>,
    /// Enchantments available for purchase (Enchanter)
    pub enchant_options: Vec<EnchantOption>,
}

impl ShopState {
    pub fn is_open(&self) -> bool {
        self.active_shop_id.is_some()
    }

    pub fn close(&mut self) {
        self.active_shop_id = None;
        self.inventory.clear();
        self.upgrade_options.clear();
        self.enchant_options.clear();
    }
}

/// An upgrade option at the Blacksmith
#[derive(Debug, Clone)]
pub struct UpgradeOption {
    /// The equipment slot being upgraded
    pub slot: EquipmentSlot,
    /// Current item ID
    pub item_id: String,
    /// Item name
    pub name: String,
    /// Current tier
    pub current_tier: RewardTier,
    /// Target tier after upgrade
    pub target_tier: RewardTier,
    /// Cost to upgrade
    pub cost: u32,
}

/// An enchantment option at the Enchanter
#[derive(Debug, Clone)]
pub struct EnchantOption {
    /// The equipment slot to enchant
    pub slot: EquipmentSlot,
    /// Item being enchanted
    pub item_id: String,
    /// Item name
    pub item_name: String,
    /// Passive/blessing ID to add
    pub passive_id: String,
    /// Passive name
    pub passive_name: String,
    /// Description of the passive
    pub passive_description: String,
    /// Cost to enchant
    pub cost: u32,
}

/// An item available for purchase in a shop
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub tier: RewardTier,
    pub price: u32,
    pub slot: EquipmentSlot,
}

/// Event to open a shop
#[derive(Debug)]
pub struct OpenShopEvent {
    pub shop_id: String,
}

impl Message for OpenShopEvent {}

/// Event to close the shop
#[derive(Debug)]
pub struct CloseShopEvent;

impl Message for CloseShopEvent {}

/// Event to reroll shop inventory (costs coins)
#[derive(Debug)]
pub struct RerollShopEvent;

impl Message for RerollShopEvent {}

/// Event fired when an item is purchased
#[derive(Debug)]
pub struct ItemPurchasedEvent {
    pub item_id: String,
    pub price: u32,
}

impl Message for ItemPurchasedEvent {}

/// Event fired when an item is upgraded
#[derive(Debug)]
pub struct ItemUpgradedEvent {
    pub slot: EquipmentSlot,
    pub item_id: String,
    pub new_tier: RewardTier,
    pub cost: u32,
}

impl Message for ItemUpgradedEvent {}

/// Event fired when an item is enchanted
#[derive(Debug)]
pub struct ItemEnchantedEvent {
    pub slot: EquipmentSlot,
    pub item_id: String,
    pub passive_id: String,
    pub cost: u32,
}

impl Message for ItemEnchantedEvent {}

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
            RewardTier::TierOne => Color::srgb(0.5, 0.5, 0.5), // Gray
            RewardTier::TierTwo => Color::srgb(0.3, 0.7, 0.3), // Green
            RewardTier::TierThree => Color::srgb(0.3, 0.5, 0.9), // Blue
            RewardTier::TierFour => Color::srgb(0.7, 0.3, 0.9), // Purple
            RewardTier::TierFive => Color::srgb(1.0, 0.8, 0.2), // Gold
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
        let Srgba {
            red,
            green,
            blue,
            alpha,
        } = base.to_srgba();

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

        Color::srgba(
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0),
            alpha,
        )
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
            RewardCategory::Skill => Color::srgb(0.6, 0.4, 0.9), // Purple
            RewardCategory::Equipment => Color::srgb(0.9, 0.7, 0.2), // Gold
            RewardCategory::Stat => Color::srgb(0.3, 0.8, 0.4),  // Green
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Marker for the shop UI root
#[derive(Component, Debug)]
pub struct ShopUI;

/// Marker for shop item buttons
#[derive(Component, Debug)]
pub struct ShopItemButton {
    pub index: usize,
}

/// Marker for close shop button
#[derive(Component, Debug)]
pub struct CloseShopButton;

/// Marker for reroll shop button
#[derive(Component, Debug)]
pub struct RerollShopButton;

/// Marker for upgrade item buttons (Blacksmith)
#[derive(Component, Debug)]
pub struct UpgradeItemButton {
    pub index: usize,
}

/// Marker for enchant item buttons (Enchanter)
#[derive(Component, Debug)]
pub struct EnchantItemButton {
    pub index: usize,
}

// ============================================================================
// Plugin
// ============================================================================

pub struct RewardsPlugin;

impl Plugin for RewardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerBuild>()
            .init_resource::<CurrentRewardChoices>()
            .init_resource::<PlayerWallet>()
            .init_resource::<ShopState>()
            .init_resource::<RunFaith>()
            .add_message::<RewardOfferedEvent>()
            .add_message::<RewardChosenEvent>()
            .add_message::<CoinGainedEvent>()
            .add_message::<CoinSpentEvent>()
            .add_message::<OpenShopEvent>()
            .add_message::<CloseShopEvent>()
            .add_message::<RerollShopEvent>()
            .add_message::<ItemPurchasedEvent>()
            .add_message::<ItemUpgradedEvent>()
            .add_message::<ItemEnchantedEvent>()
            .add_message::<FaithChangedEvent>()
            .add_message::<AdversarialEventScheduledEvent>()
            .add_message::<TriggerAdversarialEvent>()
            .add_systems(Update, process_coin_events)
            .add_systems(
                Update,
                (
                    handle_open_shop,
                    handle_close_shop,
                    handle_shop_reroll,
                    handle_shop_purchase,
                    handle_upgrade_purchase,
                    handle_enchant_purchase,
                    handle_shop_keyboard_input,
                )
                    .run_if(in_state(RunState::Arena)),
            )
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
            )
            // Faith and adversarial event systems
            .add_systems(
                Update,
                (
                    check_faith_and_schedule_adversarial,
                    check_trigger_adversarial_events,
                )
                    .run_if(in_state(RunState::Arena)),
            );
    }
}

// ============================================================================
// Systems
// ============================================================================

/// Process coin gained events and update wallet
fn process_coin_events(
    mut coin_events: MessageReader<CoinGainedEvent>,
    mut wallet: ResMut<PlayerWallet>,
) {
    for event in coin_events.read() {
        wallet.add(event.amount);
        info!(
            "Gained {} coins from {:?}. Total: {}",
            event.amount, event.source, wallet.coins
        );
    }
}

/// Listen for boss defeated and transition to reward state
fn handle_boss_defeated_for_reward(
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
        // Calculate tier bonus based on segment
        let tier_bonus = difficulty.reward_tier_bonus(run_config.segment_index);

        // Create seeded RNG for deterministic rewards
        let mut rng = ChaCha8Rng::seed_from_u64(
            run_config
                .seed
                .wrapping_add(run_config.segment_index as u64),
        );

        // Generate reward choices from ContentRegistry
        let choices = generate_data_driven_choices(
            &content_registry,
            &gameplay_defaults,
            &player_build,
            &mut rng,
            tier_bonus,
        );

        current_choices.choices = choices.clone();

        reward_events.write(RewardOfferedEvent { choices });

        // Award boss coins (scales with segment)
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
    let mut rng = ChaCha8Rng::seed_from_u64(
        run_config
            .seed
            .wrapping_add(run_config.segment_index as u64),
    );
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
        StatType::MaxHealth => rng.random_range(1..=2) as f32 * 10.0, // 10-20 base
        StatType::Stamina => rng.random_range(1..=2) as f32 * 10.0,   // 10-20 base
        StatType::AttackPower => rng.random_range(1..=2) as f32 * 2.0, // 2-4 base
    };

    let amount = (base_amount * multiplier).round();

    Some(RewardKind::StatUpgrade {
        stat: stat_type,
        amount,
        tier,
    })
}

// ============================================================================
// Data-Driven Reward Generation (ContentRegistry)
// ============================================================================

/// Convert content EquipmentSlot to rewards EquipmentSlot
fn convert_equipment_slot(slot: crate::content::EquipmentSlot) -> EquipmentSlot {
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
        ContentStatKind::MoveSpeed => StatType::Stamina, // Map to stamina for now
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

    // Try to get one blessing, one equipment, and one stat
    if let Some(reward) = generate_blessing_from_registry(registry, player_build, rng, tier_bonus) {
        choices.push(reward);
    }

    if let Some(reward) = generate_equipment_from_registry(registry, player_build, rng, tier_bonus)
    {
        choices.push(reward);
    }

    // Fill remaining slots with stat upgrades
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
    // Get blessings, optionally filtered by parent god
    let blessings: Vec<&BlessingDef> = if let Some(god_id) = &player_build.parent_god_id {
        // First try to get blessings for the player's god
        let god_blessings: Vec<_> = registry
            .blessings
            .values()
            .filter(|b| &b.god_id == god_id)
            .collect();

        if god_blessings.is_empty() {
            // Fall back to all blessings
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

    // Filter out already unlocked blessings
    let available: Vec<_> = blessings
        .iter()
        .filter(|b| !player_build.unlocked_nodes.contains(&b.id))
        .collect();

    if available.is_empty() {
        return None;
    }

    // Weight by tier with tier_bonus applied
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

    // Fallback
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
    // Get all equipment items
    let equipment: Vec<&EquipmentItemDef> = registry.equipment_items.values().collect();

    if equipment.is_empty() {
        return None;
    }

    // Filter out already equipped items
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

    // Weight by tier
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

    // Fallback
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
                        spawn_reward_card(
                            choices_parent,
                            index,
                            choice,
                            panel_color,
                            text_color,
                            muted_text,
                        );
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
        (
            &RewardChoiceButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
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
    mut run_faith: ResMut<RunFaith>,
    mut faith_events: MessageWriter<FaithChangedEvent>,
    content_registry: Res<ContentRegistry>,
) {
    for event in reward_events.read() {
        if event.choice_index >= current_choices.choices.len() {
            continue;
        }

        let choice = &current_choices.choices[event.choice_index];

        // Track which god blessings were in the choices for faith modification
        let mut god_blessings_available: Vec<String> = Vec::new();
        let mut chosen_god_id: Option<String> = None;

        // Check all choices for blessings (SkillTreeNode where tree_id is a god_id)
        for (i, reward) in current_choices.choices.iter().enumerate() {
            if let RewardKind::SkillTreeNode { tree_id, .. } = reward {
                // Check if tree_id is actually a god_id (blessings have god_id as tree_id)
                if content_registry.gods.contains_key(tree_id) {
                    god_blessings_available.push(tree_id.clone());
                    if i == event.choice_index {
                        chosen_god_id = Some(tree_id.clone());
                    }
                }
            }
        }

        // Apply faith changes for blessing choices
        if !god_blessings_available.is_empty() {
            if let Some(ref chosen) = chosen_god_id {
                // Player chose a blessing - gain faith with that god
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

                // Lose faith with other gods whose blessings were available but not chosen
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
                // Player chose something else over available blessings - minor faith loss to all
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

        // Apply the reward itself
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

        // Increment segment for next reward generation
        run_config.segment_index += 1;

        // Transition back to arena
        next_run_state.set(RunState::Arena);
    }
}

// ============================================================================
// Shop Systems
// ============================================================================

/// Handle opening a shop when OpenShopEvent is received
fn handle_open_shop(
    mut commands: Commands,
    mut shop_events: MessageReader<OpenShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut gameplay_paused: ResMut<GameplayPaused>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    player_build: Res<PlayerBuild>,
    wallet: Res<PlayerWallet>,
    existing_shop_ui: Query<Entity, With<ShopUI>>,
) {
    for event in shop_events.read() {
        // Don't open if already open
        if shop_state.is_open() {
            continue;
        }

        // Don't spawn duplicate UI
        if !existing_shop_ui.is_empty() {
            continue;
        }

        // Pause gameplay while shop is open
        gameplay_paused.pause("shop");

        info!("Opening shop: {}", event.shop_id);

        // Set active shop
        shop_state.active_shop_id = Some(event.shop_id.clone());

        // Generate inventory based on shop type
        match event.shop_id.as_str() {
            "shop_armory" => {
                shop_state.inventory =
                    generate_shop_inventory(&event.shop_id, &content_registry, &gameplay_defaults);
            }
            "shop_blacksmith" => {
                shop_state.upgrade_options =
                    generate_upgrade_options(&player_build, &content_registry, &gameplay_defaults);
            }
            "shop_enchanter" => {
                shop_state.enchant_options =
                    generate_enchant_options(&player_build, &content_registry, &gameplay_defaults);
            }
            _ => {
                shop_state.inventory =
                    generate_shop_inventory(&event.shop_id, &content_registry, &gameplay_defaults);
            }
        }

        // Spawn shop UI
        spawn_shop_ui(&mut commands, &shop_state, &wallet, &event.shop_id);
    }
}

/// Generate shop inventory based on shop ID and content registry
/// Returns exactly 5 items - one for each equipment slot
fn generate_shop_inventory(
    shop_id: &str,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<ShopItem> {
    use std::collections::HashMap;

    // Get base prices from defaults
    let base_price_min = defaults.economy.item_price_range.min as u32;
    let base_price_max = defaults.economy.item_price_range.max as u32;

    // Group all equipment items by slot
    let mut items_by_slot: HashMap<EquipmentSlot, Vec<ShopItem>> = HashMap::new();

    match shop_id {
        "shop_armory" | _ => {
            // Collect all items grouped by slot
            for equip in registry.equipment_items.values() {
                let tier = RewardTier::from_level(equip.tier as u8);
                let price = calculate_item_price(tier, base_price_min, base_price_max);
                let slot = convert_equipment_slot(equip.slot);

                let item = ShopItem {
                    item_id: equip.id.clone(),
                    name: equip.name.clone(),
                    description: format!(
                        "+{:.0} HP, {:.1}% DR",
                        equip.base_stats.max_health_bonus,
                        equip.base_stats.damage_reduction * 100.0
                    ),
                    tier,
                    price,
                    slot,
                };

                items_by_slot.entry(slot).or_default().push(item);
            }
        }
    }

    // Pick one random item per slot
    let mut rng = rand::rng();
    let mut result = Vec::with_capacity(5);

    // Iterate through slots in a consistent order
    let slots = [
        EquipmentSlot::Helmet,
        EquipmentSlot::Chestplate,
        EquipmentSlot::Greaves,
        EquipmentSlot::Boots,
        EquipmentSlot::MainHand,
    ];

    for slot in slots {
        if let Some(slot_items) = items_by_slot.get_mut(&slot) {
            if !slot_items.is_empty() {
                // Pick a random item from this slot
                let idx = rng.random_range(0..slot_items.len());
                result.push(slot_items.swap_remove(idx));
            }
        }
    }

    result
}

/// Calculate item price based on tier
fn calculate_item_price(tier: RewardTier, min_price: u32, max_price: u32) -> u32 {
    let tier_mult = match tier {
        RewardTier::TierOne => 1.0,
        RewardTier::TierTwo => 1.5,
        RewardTier::TierThree => 2.2,
        RewardTier::TierFour => 3.5,
        RewardTier::TierFive => 5.0,
    };

    let base = (min_price + max_price) / 2;
    (base as f32 * tier_mult) as u32
}

/// Generate upgrade options based on player's current equipment
fn generate_upgrade_options(
    player_build: &PlayerBuild,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<UpgradeOption> {
    let mut options = Vec::new();
    let base_price_min = defaults.economy.item_price_range.min as u32;
    let base_price_max = defaults.economy.item_price_range.max as u32;

    // Helper to create upgrade option for an equipment slot
    let mut add_option = |slot: EquipmentSlot, item_id: Option<&String>| {
        if let Some(id) = item_id {
            // Look up the item in the registry to get its current tier
            if let Some(equip) = registry.equipment_items.get(id) {
                let current_tier = RewardTier::from_level(equip.tier as u8);

                // Can only upgrade if not already at max tier
                if current_tier != RewardTier::TierFive {
                    let target_tier = RewardTier::from_level(current_tier.level() + 1);
                    // TierParity: cost equals what item would cost at target tier
                    let cost = calculate_item_price(target_tier, base_price_min, base_price_max);

                    options.push(UpgradeOption {
                        slot,
                        item_id: id.clone(),
                        name: equip.name.clone(),
                        current_tier,
                        target_tier,
                        cost,
                    });
                }
            }
        }
    };

    // Check each equipment slot
    add_option(
        EquipmentSlot::Helmet,
        player_build.equipment.helmet.as_ref(),
    );
    add_option(
        EquipmentSlot::Chestplate,
        player_build.equipment.chestplate.as_ref(),
    );
    add_option(
        EquipmentSlot::Greaves,
        player_build.equipment.greaves.as_ref(),
    );
    add_option(EquipmentSlot::Boots, player_build.equipment.boots.as_ref());
    add_option(
        EquipmentSlot::MainHand,
        player_build.equipment.main_hand.as_ref(),
    );

    options
}

/// Generate enchant options based on player's equipment and available blessings
fn generate_enchant_options(
    player_build: &PlayerBuild,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<EnchantOption> {
    let mut options = Vec::new();
    let base_enchant_cost = defaults.economy.item_price_range.min as u32;

    // Get available blessings/passives that could be added
    let available_blessings: Vec<_> = registry
        .blessings
        .values()
        .filter(|b| !player_build.unlocked_nodes.contains(&b.id))
        .collect();

    // Helper to create enchant options for an equipment slot
    let mut add_options = |slot: EquipmentSlot, item_id: Option<&String>| {
        if let Some(id) = item_id {
            if let Some(equip) = registry.equipment_items.get(id) {
                // For each available blessing, create an enchant option
                // Limit to 3 options per slot for UI simplicity
                for blessing in available_blessings.iter().take(3) {
                    let tier = RewardTier::from_level(blessing.tier as u8);
                    let cost = base_enchant_cost + (tier.level() as u32 * 20);

                    options.push(EnchantOption {
                        slot,
                        item_id: id.clone(),
                        item_name: equip.name.clone(),
                        passive_id: blessing.id.clone(),
                        passive_name: blessing.name.clone(),
                        passive_description: blessing.description.clone(),
                        cost,
                    });
                }
            }
        }
    };

    // Only enchant armor pieces (not weapons for now)
    add_options(
        EquipmentSlot::Helmet,
        player_build.equipment.helmet.as_ref(),
    );
    add_options(
        EquipmentSlot::Chestplate,
        player_build.equipment.chestplate.as_ref(),
    );
    add_options(
        EquipmentSlot::Greaves,
        player_build.equipment.greaves.as_ref(),
    );
    add_options(EquipmentSlot::Boots, player_build.equipment.boots.as_ref());

    options
}

/// Spawn the shop UI overlay
fn spawn_shop_ui(
    commands: &mut Commands,
    shop_state: &ShopState,
    wallet: &PlayerWallet,
    shop_id: &str,
) {
    let bg_color = Color::srgba(0.05, 0.05, 0.1, 0.95);
    let panel_color = Color::srgb(0.12, 0.12, 0.18);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);
    let gold_color = Color::srgb(0.9, 0.75, 0.2);

    let shop_name = match shop_id {
        "shop_armory" => "Armory",
        "shop_blacksmith" => "Blacksmith",
        "shop_enchanter" => "Enchanter",
        _ => "Shop",
    };

    commands
        .spawn((
            ShopUI,
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
            // Header with shop name and coins
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Px(700.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|header| {
                    // Shop name
                    header.spawn((
                        Text::new(shop_name.to_uppercase()),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));

                    // Coin display
                    header
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|coins| {
                            // Coin icon
                            coins.spawn((
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(gold_color),
                            ));
                            // Coin amount
                            coins.spawn((
                                Text::new(format!("{}", wallet.coins)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(gold_color),
                            ));
                        });
                });

            // Items grid - content varies by shop type
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    column_gap: Val::Px(15.0),
                    row_gap: Val::Px(15.0),
                    width: Val::Px(720.0),
                    max_height: Val::Px(450.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },))
                .with_children(|grid| {
                    match shop_id {
                        "shop_blacksmith" => {
                            if shop_state.upgrade_options.is_empty() {
                                grid.spawn((
                                    Text::new("No equipment to upgrade\n(Equip items first)"),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(muted_text),
                                    TextLayout::new_with_justify(Justify::Center),
                                ));
                            } else {
                                for (index, option) in shop_state.upgrade_options.iter().enumerate()
                                {
                                    spawn_upgrade_card(
                                        grid,
                                        index,
                                        option,
                                        wallet.coins,
                                        panel_color,
                                        text_color,
                                        muted_text,
                                        gold_color,
                                    );
                                }
                            }
                        }
                        "shop_enchanter" => {
                            if shop_state.enchant_options.is_empty() {
                                grid.spawn((
                                    Text::new("No enchantments available\n(Equip items first)"),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(muted_text),
                                    TextLayout::new_with_justify(Justify::Center),
                                ));
                            } else {
                                for (index, option) in shop_state.enchant_options.iter().enumerate()
                                {
                                    spawn_enchant_card(
                                        grid,
                                        index,
                                        option,
                                        wallet.coins,
                                        panel_color,
                                        text_color,
                                        muted_text,
                                        gold_color,
                                    );
                                }
                            }
                        }
                        _ => {
                            // Armory and default - show inventory
                            if shop_state.inventory.is_empty() {
                                grid.spawn((
                                    Text::new("No items available"),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(muted_text),
                                ));
                            } else {
                                for (index, item) in shop_state.inventory.iter().enumerate() {
                                    spawn_shop_item_card(
                                        grid,
                                        index,
                                        item,
                                        wallet.coins,
                                        panel_color,
                                        text_color,
                                        muted_text,
                                        gold_color,
                                    );
                                }
                            }
                        }
                    }
                });

            // Button row (Reroll + Close)
            let reroll_cost = 25u32;
            let can_afford_reroll = wallet.coins >= reroll_cost;
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(25.0)),
                    ..default()
                },))
                .with_children(|buttons| {
                    // Reroll button (only for armory)
                    if shop_id == "shop_armory" {
                        let reroll_border = if can_afford_reroll {
                            Color::srgb(0.5, 0.6, 0.4)
                        } else {
                            Color::srgb(0.4, 0.3, 0.3)
                        };
                        buttons
                            .spawn((
                                RerollShopButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderColor::all(reroll_border),
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                            ))
                            .with_child((
                                Text::new(format!("Reroll [R] - {} gold", reroll_cost)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(if can_afford_reroll {
                                    gold_color
                                } else {
                                    Color::srgb(0.5, 0.4, 0.4)
                                }),
                            ));
                    }

                    // Close button
                    buttons
                        .spawn((
                            CloseShopButton,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(30.0), Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                        ))
                        .with_child((
                            Text::new("Close [Esc]"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(muted_text),
                        ));
                });

            // Keyboard hint
            parent.spawn((
                Text::new("Press 1-5 to purchase, R to reroll, Esc to close"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
        });
}

/// Spawn a single shop item card
fn spawn_shop_item_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    item: &ShopItem,
    player_coins: u32,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let tier_accent = item.tier.accent_color();
    let can_afford = player_coins >= item.price;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };
    let border_thickness = 2.0 + (item.tier.level() as f32 - 1.0) * 0.5;

    parent
        .spawn((
            ShopItemButton { index },
            Button,
            Node {
                width: Val::Px(160.0),
                min_height: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(border_thickness)),
                ..default()
            },
            BorderColor::all(if can_afford {
                tier_accent
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            // Key hint
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            // Tier label
            card.spawn((
                Text::new(item.tier.display_name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(tier_accent),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Slot indicator
            card.spawn((
                Text::new(item.slot.name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Item name
            card.spawn((
                Text::new(&item.name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford { text_color } else { muted_text }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Description
            card.spawn((
                Text::new(&item.description),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Price
            card.spawn((
                Text::new(format!("{} coins", item.price)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}

/// Spawn a single upgrade option card (Blacksmith)
fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    option: &UpgradeOption,
    player_coins: u32,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let current_accent = option.current_tier.accent_color();
    let target_accent = option.target_tier.accent_color();
    let can_afford = player_coins >= option.cost;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };

    parent
        .spawn((
            UpgradeItemButton { index },
            Button,
            Node {
                width: Val::Px(180.0),
                min_height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(if can_afford {
                target_accent
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            // Key hint
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            // Slot name
            card.spawn((
                Text::new(option.slot.name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Item name
            card.spawn((
                Text::new(&option.name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford { text_color } else { muted_text }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Tier upgrade indicator
            card.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },))
                .with_children(|row| {
                    // Current tier
                    row.spawn((
                        Text::new(option.current_tier.display_name()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(current_accent),
                    ));
                    // Arrow
                    row.spawn((
                        Text::new("→"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));
                    // Target tier
                    row.spawn((
                        Text::new(option.target_tier.display_name()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(target_accent),
                    ));
                });

            // Price
            card.spawn((
                Text::new(format!("{} coins", option.cost)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}

/// Spawn a single enchant option card (Enchanter)
fn spawn_enchant_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    option: &EnchantOption,
    player_coins: u32,
    panel_color: Color,
    _text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let can_afford = player_coins >= option.cost;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };
    let enchant_color = Color::srgb(0.6, 0.4, 0.9); // Purple for enchants

    parent
        .spawn((
            EnchantItemButton { index },
            Button,
            Node {
                width: Val::Px(180.0),
                min_height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(if can_afford {
                enchant_color
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            // Key hint
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            // Item being enchanted
            card.spawn((
                Text::new(format!("{} ({})", option.item_name, option.slot.name())),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Passive name (what's being added)
            card.spawn((
                Text::new(&option.passive_name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford {
                    enchant_color
                } else {
                    muted_text
                }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Passive description
            card.spawn((
                Text::new(&option.passive_description),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Price
            card.spawn((
                Text::new(format!("{} coins", option.cost)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}

/// Handle closing the shop
fn handle_close_shop(
    mut commands: Commands,
    mut close_events: MessageReader<CloseShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut gameplay_paused: ResMut<GameplayPaused>,
    shop_ui_query: Query<Entity, With<ShopUI>>,
) {
    for _ in close_events.read() {
        // Unpause gameplay when shop closes
        gameplay_paused.unpause("shop");

        shop_state.close();

        for entity in shop_ui_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Shop closed");
    }
}

/// Handle shop reroll (regenerates inventory for a cost)
fn handle_shop_reroll(
    mut commands: Commands,
    mut reroll_events: MessageReader<RerollShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut wallet: ResMut<PlayerWallet>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    shop_ui_query: Query<Entity, With<ShopUI>>,
    reroll_button_query: Query<(&Interaction, &RerollShopButton), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let reroll_cost = 25u32;

    // Check for reroll button click
    let clicked = reroll_button_query
        .iter()
        .any(|(interaction, _)| *interaction == Interaction::Pressed);

    // Check for R key press
    let key_pressed = keyboard.just_pressed(KeyCode::KeyR);

    // Process reroll if triggered by event, click, or key
    let should_reroll = !reroll_events.is_empty() || clicked || key_pressed;
    reroll_events.clear();

    if should_reroll && shop_state.is_open() {
        // Only allow reroll in armory
        if shop_state.active_shop_id.as_deref() != Some("shop_armory") {
            return;
        }

        // Check if player can afford
        if !wallet.spend(reroll_cost) {
            info!("Cannot afford reroll (need {} gold)", reroll_cost);
            return;
        }

        info!("Rerolling shop inventory for {} gold", reroll_cost);

        // Regenerate inventory
        shop_state.inventory =
            generate_shop_inventory("shop_armory", &content_registry, &gameplay_defaults);

        // Despawn old UI and respawn
        for entity in shop_ui_query.iter() {
            commands.entity(entity).despawn();
        }

        spawn_shop_ui(&mut commands, &shop_state, &wallet, "shop_armory");
    }
}

/// Handle shop item purchase via click interaction
fn handle_shop_purchase(
    mut button_query: Query<
        (&ShopItemButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, Without<CloseShopButton>),
    >,
    mut close_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            With<CloseShopButton>,
            Changed<Interaction>,
            Without<ShopItemButton>,
        ),
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    mut purchase_events: MessageWriter<ItemPurchasedEvent>,
    mut close_events: MessageWriter<CloseShopEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    // Handle item button interactions
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(item) = shop_state.inventory.get(button.index) {
                    if wallet.spend(item.price) {
                        // Update player equipment
                        match item.slot {
                            EquipmentSlot::Helmet => {
                                player_build.equipment.helmet = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Chestplate => {
                                player_build.equipment.chestplate = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Greaves => {
                                player_build.equipment.greaves = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Boots => {
                                player_build.equipment.boots = Some(item.item_id.clone())
                            }
                            EquipmentSlot::MainHand => {
                                player_build.equipment.main_hand = Some(item.item_id.clone())
                            }
                        }

                        // Fire events
                        purchase_events.write(ItemPurchasedEvent {
                            item_id: item.item_id.clone(),
                            price: item.price,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: item.price,
                            reason: SpendReason::ShopPurchase(item.name.clone()),
                        });

                        info!("Purchased {} for {} coins", item.name, item.price);
                    } else {
                        info!(
                            "Cannot afford {} (need {} coins, have {})",
                            item.name, item.price, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }

    // Handle close button interaction
    for (interaction, mut bg_color, mut border_color) in &mut close_button_query {
        match interaction {
            Interaction::Pressed => {
                close_events.write(CloseShopEvent);
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.35));
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

/// Handle keyboard input for shop (number keys to buy, Esc to close)
fn handle_shop_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    shop_state: Res<ShopState>,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    mut purchase_events: MessageWriter<ItemPurchasedEvent>,
    mut close_events: MessageWriter<CloseShopEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    // Only process if shop is open
    if !shop_state.is_open() {
        return;
    }

    // Escape to close
    if keyboard.just_pressed(KeyCode::Escape) {
        close_events.write(CloseShopEvent);
        return;
    }

    // Number keys 1-9 to purchase
    let key_pressed = if keyboard.just_pressed(KeyCode::Digit1)
        || keyboard.just_pressed(KeyCode::Numpad1)
    {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        Some(3)
    } else if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5) {
        Some(4)
    } else if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6) {
        Some(5)
    } else if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7) {
        Some(6)
    } else if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8) {
        Some(7)
    } else if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9) {
        Some(8)
    } else {
        None
    };

    if let Some(index) = key_pressed {
        if let Some(item) = shop_state.inventory.get(index) {
            if wallet.spend(item.price) {
                // Update player equipment
                match item.slot {
                    EquipmentSlot::Helmet => {
                        player_build.equipment.helmet = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Chestplate => {
                        player_build.equipment.chestplate = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Greaves => {
                        player_build.equipment.greaves = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Boots => {
                        player_build.equipment.boots = Some(item.item_id.clone())
                    }
                    EquipmentSlot::MainHand => {
                        player_build.equipment.main_hand = Some(item.item_id.clone())
                    }
                }

                purchase_events.write(ItemPurchasedEvent {
                    item_id: item.item_id.clone(),
                    price: item.price,
                });
                coin_events.write(CoinSpentEvent {
                    amount: item.price,
                    reason: SpendReason::ShopPurchase(item.name.clone()),
                });

                info!(
                    "Purchased {} for {} coins via keyboard",
                    item.name, item.price
                );
            } else {
                info!(
                    "Cannot afford {} (need {} coins, have {})",
                    item.name, item.price, wallet.coins
                );
            }
        }
    }
}

/// Handle upgrade item purchase via click interaction (Blacksmith)
fn handle_upgrade_purchase(
    mut button_query: Query<
        (&UpgradeItemButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    content_registry: Res<ContentRegistry>,
    mut upgrade_events: MessageWriter<ItemUpgradedEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(option) = shop_state.upgrade_options.get(button.index) {
                    if wallet.spend(option.cost) {
                        // Find the next tier item with the same slot
                        // For now, we just mark the upgrade as complete - actual item replacement
                        // would require finding an item of the target tier in the registry
                        let upgraded_item_id = find_upgraded_item(
                            &option.item_id,
                            option.target_tier,
                            &content_registry,
                        )
                        .unwrap_or_else(|| option.item_id.clone());

                        // Update player equipment to the upgraded item
                        match option.slot {
                            EquipmentSlot::Helmet => {
                                player_build.equipment.helmet = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Chestplate => {
                                player_build.equipment.chestplate = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Greaves => {
                                player_build.equipment.greaves = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Boots => {
                                player_build.equipment.boots = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::MainHand => {
                                player_build.equipment.main_hand = Some(upgraded_item_id.clone())
                            }
                        }

                        upgrade_events.write(ItemUpgradedEvent {
                            slot: option.slot,
                            item_id: upgraded_item_id,
                            new_tier: option.target_tier,
                            cost: option.cost,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: option.cost,
                            reason: SpendReason::Upgrade(option.name.clone()),
                        });

                        info!(
                            "Upgraded {} from {} to {} for {} coins",
                            option.name,
                            option.current_tier.display_name(),
                            option.target_tier.display_name(),
                            option.cost
                        );
                    } else {
                        info!(
                            "Cannot afford upgrade (need {} coins, have {})",
                            option.cost, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }
}

/// Find an item of the target tier in the same slot category
fn find_upgraded_item(
    current_item_id: &str,
    target_tier: RewardTier,
    registry: &ContentRegistry,
) -> Option<String> {
    // Get the current item to find its slot
    let current_item = registry.equipment_items.get(current_item_id)?;
    let target_slot = current_item.slot;

    // Find an item with the target tier and same slot
    for (id, item) in registry.equipment_items.iter() {
        if item.slot == target_slot && item.tier == target_tier.level() as u32 {
            return Some(id.clone());
        }
    }

    // If no exact match, return the current item (upgrade in place conceptually)
    None
}

/// Handle enchant item purchase via click interaction (Enchanter)
fn handle_enchant_purchase(
    mut button_query: Query<
        (&EnchantItemButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    mut enchant_events: MessageWriter<ItemEnchantedEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(option) = shop_state.enchant_options.get(button.index) {
                    if wallet.spend(option.cost) {
                        // Add the passive to the player's unlocked nodes
                        if !player_build.unlocked_nodes.contains(&option.passive_id) {
                            player_build.unlocked_nodes.push(option.passive_id.clone());
                        }

                        enchant_events.write(ItemEnchantedEvent {
                            slot: option.slot,
                            item_id: option.item_id.clone(),
                            passive_id: option.passive_id.clone(),
                            cost: option.cost,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: option.cost,
                            reason: SpendReason::Enchant(option.passive_name.clone()),
                        });

                        info!(
                            "Enchanted {} with {} for {} coins",
                            option.item_name, option.passive_name, option.cost
                        );
                    } else {
                        info!(
                            "Cannot afford enchant (need {} coins, have {})",
                            option.cost, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }
}

// ============================================================================
// Faith and Adversarial Event Systems
// ============================================================================

/// Check if any god has negative faith and schedule adversarial events.
/// Runs when entering the Arena (hub) after segments.
fn check_faith_and_schedule_adversarial(
    mut segment_events: MessageReader<SegmentCompletedEvent>,
    mut run_faith: ResMut<RunFaith>,
    run_config: Res<RunConfig>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    mut schedule_events: MessageWriter<AdversarialEventScheduledEvent>,
) {
    for event in segment_events.read() {
        info!("Checking faith at end of segment {}", event.segment_index);

        // Check each god with negative faith
        let gods_needing_event: Vec<String> = run_faith
            .faith_by_god
            .iter()
            .filter(|(god_id, faith)| **faith < 0 && run_faith.needs_adversarial_event(god_id))
            .map(|(god_id, _)| god_id.clone())
            .collect();

        for god_id in gods_needing_event {
            // Find an appropriate adversarial event for this god
            // Look for events with reward_tags containing the god_id
            let event_id = content_registry
                .events
                .values()
                .find(|e| e.reward_tags.contains(&god_id))
                .map(|e| e.id.clone());

            if let Some(event_id) = event_id {
                // Schedule the event within the configured number of segments
                let trigger_within = gameplay_defaults.adversarial_events.trigger_within_segments;
                let trigger_at = run_config.segment_index
                    + ADVERSARIAL_MIN_DELAY
                    + (rand::random::<u32>() % trigger_within.max(1));

                run_faith.schedule_adversarial_event(&god_id, &event_id, trigger_at);

                schedule_events.write(AdversarialEventScheduledEvent {
                    god_id: god_id.clone(),
                    event_id: event_id.clone(),
                    trigger_at_segment: trigger_at,
                });

                info!(
                    "Scheduled adversarial event '{}' from {} at segment {} (current: {})",
                    event_id, god_id, trigger_at, run_config.segment_index
                );
            } else {
                warn!("No adversarial event found for god '{}' - skipping", god_id);
            }
        }
    }
}

/// Check if any scheduled adversarial events should trigger at the current segment.
fn check_trigger_adversarial_events(
    mut run_faith: ResMut<RunFaith>,
    run_config: Res<RunConfig>,
    mut trigger_events: MessageWriter<TriggerAdversarialEvent>,
) {
    // Get events that should trigger this segment
    let events_to_trigger: Vec<(String, String)> = run_faith
        .get_events_for_segment(run_config.segment_index)
        .iter()
        .map(|e| (e.god_id.clone(), e.event_id.clone()))
        .collect();

    for (god_id, event_id) in events_to_trigger {
        // Mark as triggered so it won't repeat
        run_faith.mark_adversarial_triggered(&god_id);

        trigger_events.write(TriggerAdversarialEvent {
            god_id: god_id.clone(),
            event_id: event_id.clone(),
        });

        info!(
            "Triggering adversarial event '{}' from {} at segment {}",
            event_id, god_id, run_config.segment_index
        );
    }
}
