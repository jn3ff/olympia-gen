//! Rewards domain: economy, reward selection, shop flow, and faith systems.

mod build;
mod choices;
mod economy;
pub mod faith;
mod shop;
mod types;
pub mod ui;

pub use build::{ActiveSkills, BaseStats, MovementFlags, PlayerBuild};
pub use choices::{CurrentRewardChoices, RewardChosenEvent, RewardOfferedEvent};
pub use economy::{CoinGainedEvent, CoinSource, CoinSpentEvent, PlayerWallet};
pub use faith::{
    AdversarialEventScheduledEvent, FaithChangedEvent, RunFaith, TriggerAdversarialEvent,
};
pub use shop::{
    CloseShopEvent, ItemEnchantedEvent, ItemPurchasedEvent, ItemUpgradedEvent, OpenShopEvent,
    RerollShopEvent, ShopState,
};

use bevy::prelude::*;

use crate::core::RunState;
use crate::rewards::choices::{apply_reward_choice, handle_boss_defeated_for_reward};
use crate::rewards::economy::process_coin_events;
use crate::rewards::faith::{
    check_faith_and_schedule_adversarial, check_trigger_adversarial_events,
};
use crate::rewards::shop::{
    handle_close_shop, handle_enchant_purchase, handle_open_shop, handle_shop_keyboard_input,
    handle_shop_purchase, handle_shop_reroll, handle_upgrade_purchase,
};
use crate::rewards::ui::rewards::{
    cleanup_reward_ui, handle_reward_choice_interaction, handle_reward_keyboard_input,
    spawn_reward_ui,
};

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
