//! Validation for cross-references between content definitions.

use super::data::*;
use super::registry::ContentRegistry;

/// A validation error with context about what failed.
#[derive(Debug)]
pub struct ValidationError {
    pub source_type: &'static str,
    pub source_id: String,
    pub field: &'static str,
    pub target_type: &'static str,
    pub missing_id: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} '{}' references missing {} '{}' in field '{}'",
            self.source_type, self.source_id, self.target_type, self.missing_id, self.field
        )
    }
}

/// Helper macro for checking a reference exists
macro_rules! check_ref {
    ($errors:expr, $registry_map:expr, $source_type:expr, $source_id:expr, $field:expr, $target_type:expr, $ref_id:expr) => {
        if !$registry_map.contains_key($ref_id) {
            $errors.push(ValidationError {
                source_type: $source_type,
                source_id: $source_id.to_string(),
                field: $field,
                target_type: $target_type,
                missing_id: $ref_id.to_string(),
            });
        }
    };
}

/// Validate all cross-references in the registry.
/// Returns a list of validation errors, empty if all references are valid.
pub fn validate_content(registry: &ContentRegistry) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate characters
    for (id, char) in &registry.characters {
        check_ref!(
            errors,
            registry.gods,
            "Character",
            id,
            "parent_god_id",
            "God",
            &char.parent_god_id
        );
        check_ref!(
            errors,
            registry.weapon_items,
            "Character",
            id,
            "starting_weapon_id",
            "WeaponItem",
            &char.starting_weapon_id
        );
        for affinity_id in &char.weapon_affinity_ids {
            check_ref!(
                errors,
                registry.weapon_categories,
                "Character",
                id,
                "weapon_affinity_ids",
                "WeaponCategory",
                affinity_id
            );
        }
        check_ref!(
            errors,
            registry.skills,
            "Character",
            id,
            "starting_skills.passive",
            "Skill",
            &char.starting_skills.passive
        );
        check_ref!(
            errors,
            registry.skills,
            "Character",
            id,
            "starting_skills.common",
            "Skill",
            &char.starting_skills.common
        );
        if let Some(ref ultimate) = char.starting_skills.ultimate {
            check_ref!(
                errors,
                registry.skills,
                "Character",
                id,
                "starting_skills.ultimate",
                "Skill",
                ultimate
            );
        }
    }

    // Validate gods
    for (id, god) in &registry.gods {
        check_ref!(
            errors,
            registry.skill_trees,
            "God",
            id,
            "primary_tree_id",
            "SkillTree",
            &god.primary_tree_id
        );
    }

    // Validate blessings
    for (id, blessing) in &registry.blessings {
        check_ref!(
            errors,
            registry.gods,
            "Blessing",
            id,
            "god_id",
            "God",
            &blessing.god_id
        );
    }

    // Validate skill trees
    for (id, tree) in &registry.skill_trees {
        check_ref!(
            errors,
            registry.gods,
            "SkillTree",
            id,
            "god_id",
            "God",
            &tree.god_id
        );
        for node in &tree.nodes {
            // reward_id should reference a blessing
            check_ref!(
                errors,
                registry.blessings,
                "SkillTree",
                id,
                "nodes.reward_id",
                "Blessing",
                &node.reward_id
            );
            // parent_id should reference another node in this tree (skip for now, complex validation)
        }
    }

    // Validate weapon categories
    for (id, category) in &registry.weapon_categories {
        check_ref!(
            errors,
            registry.movesets,
            "WeaponCategory",
            id,
            "default_moveset_id",
            "Moveset",
            &category.default_moveset_id
        );
    }

    // Validate weapon items
    for (id, weapon) in &registry.weapon_items {
        check_ref!(
            errors,
            registry.weapon_categories,
            "WeaponItem",
            id,
            "category_id",
            "WeaponCategory",
            &weapon.category_id
        );
        for tag_id in &weapon.curated_tag_ids {
            check_ref!(
                errors,
                registry.encounter_tags,
                "WeaponItem",
                id,
                "curated_tag_ids",
                "EncounterTag",
                tag_id
            );
        }
    }

    // Validate enemy pools
    for (id, pool) in &registry.enemy_pools {
        for enemy_ref in &pool.enemies {
            check_ref!(
                errors,
                registry.enemies,
                "EnemyPool",
                id,
                "enemies",
                "Enemy",
                &enemy_ref.id
            );
        }
    }

    // Validate encounter tables
    for (id, table) in &registry.encounter_tables {
        check_ref!(
            errors,
            registry.enemy_pools,
            "EncounterTable",
            id,
            "enemy_pool_id",
            "EnemyPool",
            &table.enemy_pool_id
        );
        check_ref!(
            errors,
            registry.enemy_pools,
            "EncounterTable",
            id,
            "elite_pool_id",
            "EnemyPool",
            &table.elite_pool_id
        );
    }

    // Validate encounter tags
    for (id, tag) in &registry.encounter_tags {
        if tag.kind == EncounterTagKind::CuratedEvent {
            if let Some(ref event_id) = tag.event_id {
                check_ref!(
                    errors,
                    registry.events,
                    "EncounterTag",
                    id,
                    "event_id",
                    "Event",
                    event_id
                );
            }
        }
    }

    // Validate rooms
    for (id, room) in &registry.rooms {
        check_ref!(
            errors,
            registry.biomes,
            "Room",
            id,
            "biome_id",
            "Biome",
            &room.biome_id
        );
        check_ref!(
            errors,
            registry.encounter_tables,
            "Room",
            id,
            "encounter_table_id",
            "EncounterTable",
            &room.encounter_table_id
        );
    }

    // Validate reward tables
    for (id, table) in &registry.reward_tables {
        for entry in &table.entries {
            if let Some(ref pool_id) = entry.pool_id {
                check_ref!(
                    errors,
                    registry.reward_pools,
                    "RewardTable",
                    id,
                    "entries.pool_id",
                    "RewardPool",
                    pool_id
                );
            }
        }
    }

    // Validate reward pools
    for (id, pool) in &registry.reward_pools {
        if pool.kind == RewardPoolKind::MinorItemPool {
            for item_id in &pool.item_ids {
                check_ref!(
                    errors,
                    registry.minor_items,
                    "RewardPool",
                    id,
                    "item_ids",
                    "MinorItem",
                    item_id
                );
            }
        }
    }

    // Validate shops
    for (id, shop) in &registry.shops {
        if let Some(ref inv_table_id) = shop.inventory_table_id {
            check_ref!(
                errors,
                registry.shop_inventory_tables,
                "Shop",
                id,
                "inventory_table_id",
                "ShopInventoryTable",
                inv_table_id
            );
        }
    }

    // Validate shop inventory tables
    for (id, table) in &registry.shop_inventory_tables {
        check_ref!(
            errors,
            registry.reward_pools,
            "ShopInventoryTable",
            id,
            "weapon_pool_id",
            "RewardPool",
            &table.weapon_pool_id
        );
        check_ref!(
            errors,
            registry.reward_pools,
            "ShopInventoryTable",
            id,
            "armor_pool_id",
            "RewardPool",
            &table.armor_pool_id
        );
    }

    errors
}
