//! ContentRegistry resource providing HashMap lookups for all loaded content.

use bevy::prelude::*;
use std::collections::HashMap;

use super::data::*;

/// Central registry for all loaded game content.
/// Provides O(1) lookup by id for any content type.
#[derive(Resource, Default)]
pub struct ContentRegistry {
    pub characters: HashMap<String, CharacterDef>,
    pub gods: HashMap<String, GodDef>,
    pub blessings: HashMap<String, BlessingDef>,
    pub skills: HashMap<String, SkillDef>,
    pub skill_trees: HashMap<String, SkillTreeDef>,
    pub weapon_categories: HashMap<String, WeaponCategoryDef>,
    pub movesets: HashMap<String, MovesetDef>,
    pub weapon_items: HashMap<String, WeaponItemDef>,
    pub enemies: HashMap<String, EnemyDef>,
    pub enemy_pools: HashMap<String, EnemyPoolDef>,
    pub encounter_tables: HashMap<String, EncounterTableDef>,
    pub encounter_tags: HashMap<String, EncounterTagDef>,
    pub rooms: HashMap<String, RoomDef>,
    pub biomes: HashMap<String, BiomeDef>,
    pub equipment_items: HashMap<String, EquipmentItemDef>,
    pub minor_items: HashMap<String, MinorItemDef>,
    pub reward_tables: HashMap<String, RewardTableDef>,
    pub reward_pools: HashMap<String, RewardPoolDef>,
    pub shops: HashMap<String, ShopDef>,
    pub shop_inventory_tables: HashMap<String, ShopInventoryTableDef>,
    pub events: HashMap<String, EventDef>,
}

impl ContentRegistry {
    /// Returns a summary of loaded content counts for logging.
    pub fn summary(&self) -> String {
        format!(
            "ContentRegistry loaded:\n\
             - Characters: {}\n\
             - Gods: {}\n\
             - Blessings: {}\n\
             - Skills: {}\n\
             - Skill Trees: {}\n\
             - Weapon Categories: {}\n\
             - Movesets: {}\n\
             - Weapon Items: {}\n\
             - Enemies: {}\n\
             - Enemy Pools: {}\n\
             - Encounter Tables: {}\n\
             - Encounter Tags: {}\n\
             - Rooms: {}\n\
             - Biomes: {}\n\
             - Equipment Items: {}\n\
             - Minor Items: {}\n\
             - Reward Tables: {}\n\
             - Reward Pools: {}\n\
             - Shops: {}\n\
             - Shop Inventory Tables: {}\n\
             - Events: {}",
            self.characters.len(),
            self.gods.len(),
            self.blessings.len(),
            self.skills.len(),
            self.skill_trees.len(),
            self.weapon_categories.len(),
            self.movesets.len(),
            self.weapon_items.len(),
            self.enemies.len(),
            self.enemy_pools.len(),
            self.encounter_tables.len(),
            self.encounter_tags.len(),
            self.rooms.len(),
            self.biomes.len(),
            self.equipment_items.len(),
            self.minor_items.len(),
            self.reward_tables.len(),
            self.reward_pools.len(),
            self.shops.len(),
            self.shop_inventory_tables.len(),
            self.events.len(),
        )
    }

    /// Returns total count of all loaded items.
    pub fn total_count(&self) -> usize {
        self.characters.len()
            + self.gods.len()
            + self.blessings.len()
            + self.skills.len()
            + self.skill_trees.len()
            + self.weapon_categories.len()
            + self.movesets.len()
            + self.weapon_items.len()
            + self.enemies.len()
            + self.enemy_pools.len()
            + self.encounter_tables.len()
            + self.encounter_tags.len()
            + self.rooms.len()
            + self.biomes.len()
            + self.equipment_items.len()
            + self.minor_items.len()
            + self.reward_tables.len()
            + self.reward_pools.len()
            + self.shops.len()
            + self.shop_inventory_tables.len()
            + self.events.len()
    }
}
