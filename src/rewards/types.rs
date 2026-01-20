//! Rewards domain: core reward types and tier metadata.

use bevy::prelude::*;

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

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let _luminance = (max + min) / 2.0;

        let brightness_adjusted = if self.brightness > 1.0 {
            let factor = self.brightness - 1.0;
            (
                red + (1.0 - red) * factor,
                green + (1.0 - green) * factor,
                blue + (1.0 - blue) * factor,
            )
        } else {
            let factor = self.brightness;
            (red * factor, green * factor, blue * factor)
        };

        let gray = (brightness_adjusted.0 + brightness_adjusted.1 + brightness_adjusted.2) / 3.0;
        let (r, g, b) = if self.saturation > 1.0 {
            let factor = self.saturation - 1.0;
            (
                brightness_adjusted.0 + (brightness_adjusted.0 - gray) * factor,
                brightness_adjusted.1 + (brightness_adjusted.1 - gray) * factor,
                brightness_adjusted.2 + (brightness_adjusted.2 - gray) * factor,
            )
        } else {
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
