//! Loader for RON content files at startup.

use bevy::prelude::*;
use ron::Options;
use std::fs;
use std::path::Path;

use super::data::*;
use super::registry::ContentRegistry;

/// Error type for content loading failures.
#[derive(Debug)]
pub struct ContentLoadError {
    pub file: String,
    pub message: String,
}

impl std::fmt::Display for ContentLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load {}: {}", self.file, self.message)
    }
}

/// Create RON options with extensions enabled for more flexible parsing.
fn ron_options() -> Options {
    Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// Load a RON file containing a DataFile<T> wrapper.
fn load_data_file<T>(path: &Path) -> Result<Vec<T>, ContentLoadError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let file_name = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|e| ContentLoadError {
        file: file_name.clone(),
        message: format!("IO error: {}", e),
    })?;

    let data: DataFile<T> = ron_options()
        .from_str(&contents)
        .map_err(|e| ContentLoadError {
            file: file_name,
            message: format!("Parse error: {}", e),
        })?;

    Ok(data.items)
}

/// Load a single RON struct (not wrapped in DataFile).
fn load_single_file<T>(path: &Path) -> Result<T, ContentLoadError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let file_name = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|e| ContentLoadError {
        file: file_name.clone(),
        message: format!("IO error: {}", e),
    })?;

    ron_options()
        .from_str(&contents)
        .map_err(|e| ContentLoadError {
            file: file_name,
            message: format!("Parse error: {}", e),
        })
}

/// Load all content from assets/data/*.ron into a ContentRegistry.
/// Returns errors for any files that fail to load.
pub fn load_all_content(
    base_path: &Path,
) -> Result<(ContentRegistry, GameplayDefaults), Vec<ContentLoadError>> {
    let mut registry = ContentRegistry::default();
    let mut errors = Vec::new();

    // Helper macro to reduce boilerplate
    macro_rules! load_into {
        ($registry_field:expr, $file:expr, $type:ty, $id_field:ident) => {
            let path = base_path.join($file);
            match load_data_file::<$type>(&path) {
                Ok(items) => {
                    for item in items {
                        $registry_field.insert(item.$id_field.clone(), item);
                    }
                }
                Err(e) => errors.push(e),
            }
        };
    }

    // Load all data files
    load_into!(registry.characters, "characters.ron", CharacterDef, id);
    load_into!(registry.gods, "gods.ron", GodDef, id);
    load_into!(registry.blessings, "blessings.ron", BlessingDef, id);
    load_into!(registry.skills, "skills.ron", SkillDef, id);
    load_into!(registry.skill_trees, "skill_trees.ron", SkillTreeDef, id);
    load_into!(
        registry.weapon_categories,
        "weapon_categories.ron",
        WeaponCategoryDef,
        id
    );
    load_into!(registry.movesets, "movesets.ron", MovesetDef, id);
    load_into!(registry.weapon_items, "weapon_items.ron", WeaponItemDef, id);
    load_into!(registry.enemies, "enemies.ron", EnemyDef, id);
    load_into!(registry.enemy_pools, "enemy_pools.ron", EnemyPoolDef, id);
    load_into!(
        registry.encounter_tables,
        "encounter_tables.ron",
        EncounterTableDef,
        id
    );
    load_into!(
        registry.encounter_tags,
        "encounter_tags.ron",
        EncounterTagDef,
        id
    );
    load_into!(registry.rooms, "rooms.ron", RoomDef, id);
    load_into!(registry.biomes, "biomes.ron", BiomeDef, id);
    load_into!(
        registry.equipment_items,
        "equipment_items.ron",
        EquipmentItemDef,
        id
    );
    load_into!(registry.minor_items, "minor_items.ron", MinorItemDef, id);
    load_into!(
        registry.reward_tables,
        "reward_tables.ron",
        RewardTableDef,
        id
    );
    load_into!(registry.reward_pools, "reward_pools.ron", RewardPoolDef, id);
    load_into!(registry.shops, "shops.ron", ShopDef, id);
    load_into!(
        registry.shop_inventory_tables,
        "shop_inventory_tables.ron",
        ShopInventoryTableDef,
        id
    );
    load_into!(registry.events, "events.ron", EventDef, id);

    // Load gameplay defaults (single struct, not a list)
    let defaults_path = base_path.join("gameplay_defaults.ron");
    let gameplay_defaults = match load_single_file::<GameplayDefaults>(&defaults_path) {
        Ok(defaults) => defaults,
        Err(e) => {
            errors.push(e);
            // Return early if gameplay_defaults fails - it's required
            return Err(errors);
        }
    };

    if errors.is_empty() {
        Ok((registry, gameplay_defaults))
    } else {
        Err(errors)
    }
}
