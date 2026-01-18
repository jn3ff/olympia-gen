use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct CharacterDef {
    pub id: String,
    pub name: String,
    pub blessing_id: String,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct GodBlessingDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_weapon_id: String,
    pub passive_skill_id: String,
    pub common_skill_id: String,
    pub heavy_skill_id: String,
    pub skill_tree_id: String,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    pub weapon_type: String,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slot: SkillSlot,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillTreeDef {
    pub id: String,
    pub nodes: Vec<SkillNodeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct SkillNodeDef {
    pub id: String,
    pub parent: Option<String>,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct RoomDef {
    pub id: String,
    pub name: String,
    pub exits: Vec<Direction>,
    pub boss_room: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
pub enum SkillSlot {
    Passive,
    Common,
    Heavy,
}

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterDef>()
            .register_type::<GodBlessingDef>()
            .register_type::<WeaponDef>()
            .register_type::<SkillDef>()
            .register_type::<SkillTreeDef>()
            .register_type::<SkillNodeDef>()
            .register_type::<RoomDef>()
            .register_type::<Direction>()
            .register_type::<SkillSlot>();
    }
}
