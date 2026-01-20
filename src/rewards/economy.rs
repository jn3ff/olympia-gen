//! Rewards domain: currency tracking and coin events.

use bevy::ecs::message::{Message, MessageReader};
use bevy::prelude::*;

/// Resource tracking player currency within a run
#[derive(Resource, Debug, Default)]
pub struct PlayerWallet {
    pub coins: u32,
}

impl PlayerWallet {
    pub fn add(&mut self, amount: u32) {
        self.coins = self.coins.saturating_add(amount);
    }

    pub fn spend(&mut self, amount: u32) -> bool {
        if self.coins >= amount {
            self.coins -= amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, amount: u32) -> bool {
        self.coins >= amount
    }
}

/// Source of coin gain for tracking/analytics
#[derive(Debug, Clone, Copy)]
pub enum CoinSource {
    EnemyDrop,
    RoomReward,
    BossReward,
    SellItem,
}

/// Event fired when player gains coins
#[derive(Debug)]
pub struct CoinGainedEvent {
    pub amount: u32,
    pub source: CoinSource,
}

impl Message for CoinGainedEvent {}

/// Reason for spending coins
#[derive(Debug, Clone)]
pub enum SpendReason {
    ShopPurchase(String),
    Upgrade(String),
    Enchant(String),
    ShopReroll,
}

/// Event fired when player spends coins
#[derive(Debug)]
pub struct CoinSpentEvent {
    pub amount: u32,
    pub reason: SpendReason,
}

impl Message for CoinSpentEvent {}

/// Process coin gained events and update wallet
pub(crate) fn process_coin_events(
    mut coin_events: MessageReader<CoinGainedEvent>,
    mut wallet: ResMut<PlayerWallet>,
) {
    for event in coin_events.read() {
        wallet.add(event.amount);
        info!(
            "Gained {} coins from {:?}. Total: {}",
            event.amount, event.source, wallet.coins
        );
    }
}
