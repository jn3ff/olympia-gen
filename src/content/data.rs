//! Data definitions for all RON content files.
//!
//! These structs mirror the structure in assets/data/*.ron and are used
//! for deserialization. The ContentRegistry provides lookup by id.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Common wrapper for RON files with schema_version and items
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataFile<T> {
    pub schema_version: u32,
    pub items: Vec<T>,
}

// ============================================================================
// Characters (characters.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct CharacterDef {
    pub id: String,
    pub name: String,
    pub parent_god_id: String,
    pub starting_weapon_id: String,
    pub weapon_affinity_ids: Vec<String>,
    pub starting_skills: StartingSkillsDef,
    pub base_stats: CharacterStatsDef,
    pub movement_flags: MovementFlagsDef,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct StartingSkillsDef {
    pub passive: String,
    pub common: String,
    pub ultimate: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct CharacterStatsDef {
    pub max_health: f32,
    pub attack_power: f32,
    pub move_speed_mult: f32,
    pub jump_height_mult: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct MovementFlagsDef {
    pub wall_jump: bool,
    pub air_dash_unlocked: bool,
}

// ============================================================================
// Gods (gods.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct GodDef {
    pub id: String,
    pub name: String,
    pub epithet: String,
    pub description: String,
    pub element_tags: Vec<String>,
    pub primary_tree_id: String,
}

// ============================================================================
// Blessings (blessings.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct BlessingDef {
    pub id: String,
    pub name: String,
    pub god_id: String,
    pub tier: u32,
    pub description: String,
    pub effect_tags: Vec<String>,
}

// ============================================================================
// Skills (skills.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum SkillSlot {
    #[default]
    Passive,
    Common,
    Heavy,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub slot: SkillSlot,
    pub cooldown_seconds: f32,
    pub description: String,
    pub effect_tags: Vec<String>,
    pub tags: Vec<String>,
}

// ============================================================================
// Skill Trees (skill_trees.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillTreeDef {
    pub id: String,
    pub name: String,
    pub god_id: String,
    pub nodes: Vec<SkillTreeNodeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillTreeNodeDef {
    pub id: String,
    pub parent_id: Option<String>,
    pub reward_id: String,
    pub tier: u32,
}

// ============================================================================
// Weapon Categories (weapon_categories.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum AttackType {
    #[default]
    Light,
    Heavy,
    Special,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct WeaponCategoryDef {
    pub id: String,
    pub name: String,
    pub default_moveset_id: String,
    pub attack_types: Vec<AttackType>,
    pub tags: Vec<String>,
}

// ============================================================================
// Movesets (movesets.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct MovesetDef {
    pub id: String,
    pub name: String,
    pub light_combo: ComboDef,
    pub heavy_combo: ComboDef,
    pub special: StrikeDef,
    pub parry: ParryDef,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct ComboDef {
    pub loop_from: usize,
    pub strikes: Vec<StrikeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct StrikeDef {
    pub id: String,
    pub damage: f32,
    pub stance_damage: f32,
    pub cooldown: f32,
    pub hitbox: HitboxDef,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct HitboxDef {
    pub length: f32,
    pub width: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct ParryDef {
    pub enabled: bool,
    pub window_seconds: f32,
}

// ============================================================================
// Weapon Items (weapon_items.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct WeaponItemDef {
    pub id: String,
    pub name: String,
    pub category_id: String,
    pub tier: u32,
    pub base_damage_mult: f32,
    pub base_stance_mult: f32,
    pub passive_slots: u32,
    pub base_passives: Vec<String>,
    pub buff_tag_slots: u32,
    pub curated_tag_ids: Vec<String>,
    pub tags: Vec<String>,
}

// ============================================================================
// Enemies (enemies.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum EnemyTier {
    #[default]
    Minor,
    Major,
    Elite,
    Boss,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub tier: EnemyTier,
    pub base_stats: EnemyStatsDef,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EnemyStatsDef {
    pub health: f32,
    pub damage: f32,
    pub move_speed: f32,
}

// ============================================================================
// Enemy Pools (enemy_pools.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EnemyPoolDef {
    pub id: String,
    pub enemies: Vec<WeightedEnemyRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct WeightedEnemyRef {
    pub id: String,
    pub weight: f32,
}

// ============================================================================
// Encounter Tables (encounter_tables.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EncounterTableDef {
    pub id: String,
    pub description: String,
    pub enemy_pool_id: String,
    pub elite_pool_id: String,
}

// ============================================================================
// Encounter Tags (encounter_tags.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum EncounterTagKind {
    #[default]
    CuratedEvent,
    Buff,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EncounterTagDef {
    pub id: String,
    pub name: String,
    pub kind: EncounterTagKind,
    pub description: String,
    pub tier: u32,
    // Optional fields depending on kind
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub effect_tags: Vec<String>,
}

// ============================================================================
// Rooms (rooms.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum RoomType {
    #[default]
    Combat,
    Traversal,
    Boss,
    Hub,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RoomSizeDef {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RoomDef {
    pub id: String,
    pub name: String,
    pub size: RoomSizeDef,
    pub exits: Vec<Direction>,
    pub room_type: RoomType,
    pub biome_id: String,
    pub encounter_table_id: String,
    pub modifiers: Vec<String>,
    pub tags: Vec<String>,
}

// ============================================================================
// Biomes (biomes.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct BiomeDef {
    pub id: String,
    pub name: String,
    pub movement_mods: BiomeMovementMods,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct BiomeMovementMods {
    pub gravity_mult: f32,
    pub dash_cooldown_mult: f32,
    pub jump_height_mult: f32,
}

// ============================================================================
// Equipment Items (equipment_items.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum EquipmentSlot {
    #[default]
    Helmet,
    Chestplate,
    Gloves,
    Boots,
    Accessory,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EquipmentItemDef {
    pub id: String,
    pub name: String,
    pub slot: EquipmentSlot,
    pub tier: u32,
    pub base_stats: EquipmentStatsDef,
    pub passive_slots: u32,
    pub base_passives: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EquipmentStatsDef {
    pub max_health_bonus: f32,
    pub damage_reduction: f32,
}

// ============================================================================
// Minor Items (minor_items.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum MinorItemKind {
    #[default]
    Money,
    Heal,
    Buff,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct MinorItemDef {
    pub id: String,
    pub name: String,
    pub kind: MinorItemKind,
    pub value: u32,
    pub description: String,
}

// ============================================================================
// Reward Tables (reward_tables.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum RewardEntryKind {
    #[default]
    Money,
    MinorItem,
    StatBoost,
    Blessing,
    Equipment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum StatKind {
    #[default]
    MaxHealth,
    AttackPower,
    MoveSpeed,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RewardTableDef {
    pub id: String,
    pub entries: Vec<RewardEntryDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RewardEntryDef {
    pub kind: RewardEntryKind,
    pub weight: f32,
    // Optional fields based on kind
    #[serde(default)]
    pub amount_range: Option<AmountRange>,
    #[serde(default)]
    pub pool_id: Option<String>,
    #[serde(default)]
    pub stat: Option<StatKind>,
    #[serde(default)]
    pub amount: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct AmountRange {
    pub min: u32,
    pub max: u32,
}

// ============================================================================
// Reward Pools (reward_pools.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum RewardPoolKind {
    #[default]
    BlessingPool,
    EquipmentPool,
    MinorItemPool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum PoolStrategy {
    #[default]
    ParentGod,
    Tiered,
    Random,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RewardPoolDef {
    pub id: String,
    pub kind: RewardPoolKind,
    #[serde(default)]
    pub strategy: Option<PoolStrategy>,
    #[serde(default)]
    pub tier_bias: Option<f32>,
    #[serde(default)]
    pub item_ids: Vec<String>,
}

// ============================================================================
// Shops (shops.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum ShopKind {
    #[default]
    EquipmentShop,
    UpgradeShop,
    EnchantShop,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct ShopDef {
    pub id: String,
    pub name: String,
    pub kind: ShopKind,
    #[serde(default)]
    pub inventory_table_id: Option<String>,
    #[serde(default)]
    pub price_multiplier: Option<f32>,
    #[serde(default)]
    pub upgrade_rules: Option<UpgradeRulesDef>,
    #[serde(default)]
    pub enchant_rules: Option<EnchantRulesDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct UpgradeRulesDef {
    pub max_tier: u32,
    pub upgrade_cost_mode: UpgradeCostMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum UpgradeCostMode {
    #[default]
    TierParity,
    Flat,
    Scaling,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EnchantRulesDef {
    pub max_passives: u32,
}

// ============================================================================
// Shop Inventory Tables (shop_inventory_tables.ron)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct ShopInventoryTableDef {
    pub id: String,
    pub weapon_pool_id: String,
    pub armor_pool_id: String,
    pub reroll_cost: u32,
}

// ============================================================================
// Events (events.ron)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum EventKind {
    #[default]
    CombatEncounter,
    NarrativeEncounter,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EventDef {
    pub id: String,
    pub name: String,
    pub kind: EventKind,
    pub description: String,
    pub reward_tags: Vec<String>,
}

// ============================================================================
// Gameplay Defaults (gameplay_defaults.ron) - Single struct, not a list
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Reflect, Resource)]
pub struct GameplayDefaults {
    pub schema_version: u32,
    pub segment_defaults: SegmentDefaults,
    pub win_condition: WinConditionDef,
    pub reward_affinity: RewardAffinityDef,
    pub economy: EconomyDefaults,
    pub stance_defaults: StanceDefaults,
    pub adversarial_events: AdversarialEventsDefaults,
    pub encounter_defaults: EncounterDefaults,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SegmentDefaults {
    pub rooms_per_segment: u32,
    pub bosses_per_segment: u32,
    pub hub_after_segment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Reflect, Default)]
pub enum WinConditionMode {
    #[default]
    BossCount,
    SegmentsCleared,
    FaithGained,
    QuestSteps,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct WinConditionDef {
    pub mode: WinConditionMode,
    pub boss_target: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RewardAffinityDef {
    pub min_in_class_options: u32,
    pub max_in_class_options: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EconomyDefaults {
    pub segment_value: u32,
    pub item_price_range: PriceRange,
    pub rare_item_price_range: PriceRange,
    pub upgrade_cost_mode: UpgradeCostMode,
    pub passive_slots_by_tier: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct PriceRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct StanceDefaults {
    pub heavy_break_count: u32,
    pub light_break_count: u32,
    pub regen_light_per_seconds: f32,
    pub parry_heavy_equivalent: u32,
    pub break_refill_multiplier: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct AdversarialEventsDefaults {
    pub trigger_within_segments: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct EncounterDefaults {
    pub specialty_tag_count: u32,
}
