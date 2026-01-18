use bevy::prelude::*;

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

#[derive(Debug, Clone, Default)]
pub struct BaseStats {
    pub max_health: f32,
    pub stamina: f32,
    pub attack_power: f32,
}

#[derive(Debug, Clone)]
pub enum RewardKind {
    SkillTreeNode { tree_id: String, node_id: String },
    Equipment { slot: EquipmentSlot, item_id: String },
    StatUpgrade { stat: StatType, amount: f32 },
}

#[derive(Debug, Clone, Copy)]
pub enum EquipmentSlot {
    Helmet,
    Chestplate,
    Greaves,
    Boots,
    MainHand,
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    MaxHealth,
    Stamina,
    AttackPower,
}

#[derive(Event, Debug)]
pub struct RewardOfferedEvent {
    pub choices: Vec<RewardKind>,
}

#[derive(Event, Debug)]
pub struct RewardChosenEvent {
    pub choice: RewardKind,
}

pub struct RewardsPlugin;

impl Plugin for RewardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerBuild>()
            .add_event::<RewardOfferedEvent>()
            .add_event::<RewardChosenEvent>()
            .add_systems(Update, rewards_flow_stub);
    }
}

fn rewards_flow_stub() {}
