//! Rewards domain: player build state and progression data.

use bevy::prelude::*;

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
