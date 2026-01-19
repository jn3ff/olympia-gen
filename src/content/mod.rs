//! Content module for data-driven game content.
//!
//! This module handles loading RON files from assets/data/, validating
//! cross-references, and providing a ContentRegistry resource for lookups.

pub mod data;
pub mod loader;
pub mod registry;
pub mod validation;

use bevy::prelude::*;
use std::path::PathBuf;

pub use data::*;
// Re-exports for external use
#[allow(unused_imports)]
pub use registry::ContentRegistry;
#[allow(unused_imports)]
pub use validation::ValidationError;

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        // Register all reflectable types
        app.register_type::<CharacterDef>()
            .register_type::<StartingSkillsDef>()
            .register_type::<CharacterStatsDef>()
            .register_type::<MovementFlagsDef>()
            .register_type::<GodDef>()
            .register_type::<BlessingDef>()
            .register_type::<SkillDef>()
            .register_type::<SkillSlot>()
            .register_type::<SkillTreeDef>()
            .register_type::<SkillTreeNodeDef>()
            .register_type::<WeaponCategoryDef>()
            .register_type::<AttackType>()
            .register_type::<MovesetDef>()
            .register_type::<ComboDef>()
            .register_type::<StrikeDef>()
            .register_type::<HitboxDef>()
            .register_type::<ParryDef>()
            .register_type::<WeaponItemDef>()
            .register_type::<EnemyDef>()
            .register_type::<EnemyTier>()
            .register_type::<EnemyStatsDef>()
            .register_type::<EnemyPoolDef>()
            .register_type::<WeightedEnemyRef>()
            .register_type::<EncounterTableDef>()
            .register_type::<EncounterTagDef>()
            .register_type::<EncounterTagKind>()
            .register_type::<RoomDef>()
            .register_type::<RoomSizeDef>()
            .register_type::<RoomType>()
            .register_type::<Direction>()
            .register_type::<BiomeDef>()
            .register_type::<BiomeMovementMods>()
            .register_type::<EquipmentItemDef>()
            .register_type::<EquipmentSlot>()
            .register_type::<EquipmentStatsDef>()
            .register_type::<MinorItemDef>()
            .register_type::<MinorItemKind>()
            .register_type::<RewardTableDef>()
            .register_type::<RewardEntryDef>()
            .register_type::<RewardEntryKind>()
            .register_type::<StatKind>()
            .register_type::<AmountRange>()
            .register_type::<RewardPoolDef>()
            .register_type::<RewardPoolKind>()
            .register_type::<PoolStrategy>()
            .register_type::<ShopDef>()
            .register_type::<ShopKind>()
            .register_type::<UpgradeRulesDef>()
            .register_type::<UpgradeCostMode>()
            .register_type::<EnchantRulesDef>()
            .register_type::<ShopInventoryTableDef>()
            .register_type::<EventDef>()
            .register_type::<EventKind>()
            .register_type::<GameplayDefaults>()
            .register_type::<SegmentDefaults>()
            .register_type::<WinConditionDef>()
            .register_type::<WinConditionMode>()
            .register_type::<RewardAffinityDef>()
            .register_type::<EconomyDefaults>()
            .register_type::<PriceRange>()
            .register_type::<StanceDefaults>()
            .register_type::<AdversarialEventsDefaults>()
            .register_type::<EncounterDefaults>();

        // Load content at startup
        app.add_systems(Startup, load_content_system);
    }
}

/// System that loads all content at startup and validates cross-references.
fn load_content_system(mut commands: Commands) {
    // Determine the assets/data path
    let data_path = PathBuf::from("assets/data");

    info!("Loading content from {:?}...", data_path);

    match loader::load_all_content(&data_path) {
        Ok((registry, gameplay_defaults)) => {
            // Log summary
            info!("{}", registry.summary());
            info!("Total content items: {}", registry.total_count());

            // Validate cross-references
            let validation_errors = validation::validate_content(&registry);

            if validation_errors.is_empty() {
                info!("Content validation passed: all cross-references valid");
            } else {
                error!(
                    "Content validation failed with {} errors:",
                    validation_errors.len()
                );
                for err in &validation_errors {
                    error!("  - {}", err);
                }
                // Fail fast on validation errors
                panic!(
                    "Content validation failed with {} errors. See logs above.",
                    validation_errors.len()
                );
            }

            // Insert resources
            commands.insert_resource(registry);
            commands.insert_resource(gameplay_defaults);
        }
        Err(errors) => {
            error!("Failed to load content with {} errors:", errors.len());
            for err in &errors {
                error!("  - {}", err);
            }
            panic!(
                "Content loading failed with {} errors. See logs above.",
                errors.len()
            );
        }
    }
}
