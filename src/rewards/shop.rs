//! Rewards domain: shop state, events, and purchase systems.

use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::Rng;

use crate::content::{ContentRegistry, GameplayDefaults};
use crate::core::GameplayPaused;
use crate::rewards::build::PlayerBuild;
use crate::rewards::choices::convert_equipment_slot;
use crate::rewards::economy::{CoinSpentEvent, PlayerWallet, SpendReason};
use crate::rewards::types::{EquipmentSlot, RewardTier};
use crate::rewards::ui::shop::spawn_shop_ui;
use crate::rewards::ui::{
    CloseShopButton, EnchantItemButton, RerollShopButton, ShopItemButton, ShopUI, UpgradeItemButton,
};

/// Resource tracking the current shop state
#[derive(Resource, Debug, Default)]
pub struct ShopState {
    /// Currently open shop ID (None if no shop is open)
    pub active_shop_id: Option<String>,
    /// Items available for purchase in the current shop (Armory)
    pub inventory: Vec<ShopItem>,
    /// Items available for upgrade (Blacksmith)
    pub upgrade_options: Vec<UpgradeOption>,
    /// Enchantments available for purchase (Enchanter)
    pub enchant_options: Vec<EnchantOption>,
}

impl ShopState {
    pub fn is_open(&self) -> bool {
        self.active_shop_id.is_some()
    }

    pub fn close(&mut self) {
        self.active_shop_id = None;
        self.inventory.clear();
        self.upgrade_options.clear();
        self.enchant_options.clear();
    }
}

/// An upgrade option at the Blacksmith
#[derive(Debug, Clone)]
pub struct UpgradeOption {
    /// The equipment slot being upgraded
    pub slot: EquipmentSlot,
    /// Current item ID
    pub item_id: String,
    /// Item name
    pub name: String,
    /// Current tier
    pub current_tier: RewardTier,
    /// Target tier after upgrade
    pub target_tier: RewardTier,
    /// Cost to upgrade
    pub cost: u32,
}

/// An enchantment option at the Enchanter
#[derive(Debug, Clone)]
pub struct EnchantOption {
    /// The equipment slot to enchant
    pub slot: EquipmentSlot,
    /// Item being enchanted
    pub item_id: String,
    /// Item name
    pub item_name: String,
    /// Passive/blessing ID to add
    pub passive_id: String,
    /// Passive name
    pub passive_name: String,
    /// Description of the passive
    pub passive_description: String,
    /// Cost to enchant
    pub cost: u32,
}

/// An item available for purchase in a shop
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub tier: RewardTier,
    pub price: u32,
    pub slot: EquipmentSlot,
}

/// Event to open a shop
#[derive(Debug)]
pub struct OpenShopEvent {
    pub shop_id: String,
}

impl Message for OpenShopEvent {}

/// Event to close the shop
#[derive(Debug)]
pub struct CloseShopEvent;

impl Message for CloseShopEvent {}

/// Event to reroll shop inventory (costs coins)
#[derive(Debug)]
pub struct RerollShopEvent;

impl Message for RerollShopEvent {}

/// Event fired when an item is purchased
#[derive(Debug)]
pub struct ItemPurchasedEvent {
    pub item_id: String,
    pub price: u32,
}

impl Message for ItemPurchasedEvent {}

/// Event fired when an item is upgraded
#[derive(Debug)]
pub struct ItemUpgradedEvent {
    pub slot: EquipmentSlot,
    pub item_id: String,
    pub new_tier: RewardTier,
    pub cost: u32,
}

impl Message for ItemUpgradedEvent {}

/// Event fired when an item is enchanted
#[derive(Debug)]
pub struct ItemEnchantedEvent {
    pub slot: EquipmentSlot,
    pub item_id: String,
    pub passive_id: String,
    pub cost: u32,
}

impl Message for ItemEnchantedEvent {}

/// Handle opening a shop when OpenShopEvent is received
pub(crate) fn handle_open_shop(
    mut commands: Commands,
    mut shop_events: MessageReader<OpenShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut gameplay_paused: ResMut<GameplayPaused>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    player_build: Res<PlayerBuild>,
    wallet: Res<PlayerWallet>,
    existing_shop_ui: Query<Entity, With<ShopUI>>,
) {
    for event in shop_events.read() {
        if shop_state.is_open() {
            continue;
        }

        if !existing_shop_ui.is_empty() {
            continue;
        }

        gameplay_paused.pause("shop");

        info!("Opening shop: {}", event.shop_id);

        shop_state.active_shop_id = Some(event.shop_id.clone());

        match event.shop_id.as_str() {
            "shop_armory" => {
                shop_state.inventory =
                    generate_shop_inventory(&event.shop_id, &content_registry, &gameplay_defaults);
            }
            "shop_blacksmith" => {
                shop_state.upgrade_options =
                    generate_upgrade_options(&player_build, &content_registry, &gameplay_defaults);
            }
            "shop_enchanter" => {
                shop_state.enchant_options =
                    generate_enchant_options(&player_build, &content_registry, &gameplay_defaults);
            }
            _ => {
                shop_state.inventory =
                    generate_shop_inventory(&event.shop_id, &content_registry, &gameplay_defaults);
            }
        }

        spawn_shop_ui(&mut commands, &shop_state, &wallet, &event.shop_id);
    }
}

/// Generate shop inventory based on shop ID and content registry
/// Returns exactly 5 items - one for each equipment slot
fn generate_shop_inventory(
    shop_id: &str,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<ShopItem> {
    use std::collections::HashMap;

    let base_price_min = defaults.economy.item_price_range.min as u32;
    let base_price_max = defaults.economy.item_price_range.max as u32;

    let mut items_by_slot: HashMap<EquipmentSlot, Vec<ShopItem>> = HashMap::new();

    match shop_id {
        "shop_armory" | _ => {
            for equip in registry.equipment_items.values() {
                let tier = RewardTier::from_level(equip.tier as u8);
                let price = calculate_item_price(tier, base_price_min, base_price_max);
                let slot = convert_equipment_slot(equip.slot);

                let item = ShopItem {
                    item_id: equip.id.clone(),
                    name: equip.name.clone(),
                    description: format!(
                        "+{:.0} HP, {:.1}% DR",
                        equip.base_stats.max_health_bonus,
                        equip.base_stats.damage_reduction * 100.0
                    ),
                    tier,
                    price,
                    slot,
                };

                items_by_slot.entry(slot).or_default().push(item);
            }
        }
    }

    let mut rng = rand::rng();
    let mut result = Vec::with_capacity(5);

    let slots = [
        EquipmentSlot::Helmet,
        EquipmentSlot::Chestplate,
        EquipmentSlot::Greaves,
        EquipmentSlot::Boots,
        EquipmentSlot::MainHand,
    ];

    for slot in slots {
        if let Some(slot_items) = items_by_slot.get_mut(&slot) {
            if !slot_items.is_empty() {
                let idx = rng.random_range(0..slot_items.len());
                result.push(slot_items.swap_remove(idx));
            }
        }
    }

    result
}

/// Calculate item price based on tier
fn calculate_item_price(tier: RewardTier, min_price: u32, max_price: u32) -> u32 {
    let tier_mult = match tier {
        RewardTier::TierOne => 1.0,
        RewardTier::TierTwo => 1.5,
        RewardTier::TierThree => 2.2,
        RewardTier::TierFour => 3.5,
        RewardTier::TierFive => 5.0,
    };

    let base = (min_price + max_price) / 2;
    (base as f32 * tier_mult) as u32
}

/// Generate upgrade options based on player's current equipment
fn generate_upgrade_options(
    player_build: &PlayerBuild,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<UpgradeOption> {
    let mut options = Vec::new();
    let base_price_min = defaults.economy.item_price_range.min as u32;
    let base_price_max = defaults.economy.item_price_range.max as u32;

    let mut add_option = |slot: EquipmentSlot, item_id: Option<&String>| {
        if let Some(id) = item_id {
            if let Some(equip) = registry.equipment_items.get(id) {
                let current_tier = RewardTier::from_level(equip.tier as u8);

                if current_tier != RewardTier::TierFive {
                    let target_tier = RewardTier::from_level(current_tier.level() + 1);
                    let cost = calculate_item_price(target_tier, base_price_min, base_price_max);

                    options.push(UpgradeOption {
                        slot,
                        item_id: id.clone(),
                        name: equip.name.clone(),
                        current_tier,
                        target_tier,
                        cost,
                    });
                }
            }
        }
    };

    add_option(
        EquipmentSlot::Helmet,
        player_build.equipment.helmet.as_ref(),
    );
    add_option(
        EquipmentSlot::Chestplate,
        player_build.equipment.chestplate.as_ref(),
    );
    add_option(
        EquipmentSlot::Greaves,
        player_build.equipment.greaves.as_ref(),
    );
    add_option(EquipmentSlot::Boots, player_build.equipment.boots.as_ref());
    add_option(
        EquipmentSlot::MainHand,
        player_build.equipment.main_hand.as_ref(),
    );

    options
}

/// Generate enchant options based on player's equipment and available blessings
fn generate_enchant_options(
    player_build: &PlayerBuild,
    registry: &ContentRegistry,
    defaults: &GameplayDefaults,
) -> Vec<EnchantOption> {
    let mut options = Vec::new();
    let base_enchant_cost = defaults.economy.item_price_range.min as u32;

    let available_blessings: Vec<_> = registry
        .blessings
        .values()
        .filter(|b| !player_build.unlocked_nodes.contains(&b.id))
        .collect();

    let mut add_options = |slot: EquipmentSlot, item_id: Option<&String>| {
        if let Some(id) = item_id {
            if let Some(equip) = registry.equipment_items.get(id) {
                for blessing in available_blessings.iter().take(3) {
                    let tier = RewardTier::from_level(blessing.tier as u8);
                    let cost = base_enchant_cost + (tier.level() as u32 * 20);

                    options.push(EnchantOption {
                        slot,
                        item_id: id.clone(),
                        item_name: equip.name.clone(),
                        passive_id: blessing.id.clone(),
                        passive_name: blessing.name.clone(),
                        passive_description: blessing.description.clone(),
                        cost,
                    });
                }
            }
        }
    };

    add_options(
        EquipmentSlot::Helmet,
        player_build.equipment.helmet.as_ref(),
    );
    add_options(
        EquipmentSlot::Chestplate,
        player_build.equipment.chestplate.as_ref(),
    );
    add_options(
        EquipmentSlot::Greaves,
        player_build.equipment.greaves.as_ref(),
    );
    add_options(EquipmentSlot::Boots, player_build.equipment.boots.as_ref());

    options
}

/// Handle closing the shop
pub(crate) fn handle_close_shop(
    mut commands: Commands,
    mut close_events: MessageReader<CloseShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut gameplay_paused: ResMut<GameplayPaused>,
    shop_ui_query: Query<Entity, With<ShopUI>>,
) {
    for _ in close_events.read() {
        gameplay_paused.unpause("shop");

        shop_state.close();

        for entity in shop_ui_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Shop closed");
    }
}

/// Handle shop reroll (regenerates inventory for a cost)
pub(crate) fn handle_shop_reroll(
    mut commands: Commands,
    mut reroll_events: MessageReader<RerollShopEvent>,
    mut shop_state: ResMut<ShopState>,
    mut wallet: ResMut<PlayerWallet>,
    content_registry: Res<ContentRegistry>,
    gameplay_defaults: Res<GameplayDefaults>,
    shop_ui_query: Query<Entity, With<ShopUI>>,
    reroll_button_query: Query<(&Interaction, &RerollShopButton), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let reroll_cost = 25u32;

    let clicked = reroll_button_query
        .iter()
        .any(|(interaction, _)| *interaction == Interaction::Pressed);

    let key_pressed = keyboard.just_pressed(KeyCode::KeyR);

    let should_reroll = !reroll_events.is_empty() || clicked || key_pressed;
    reroll_events.clear();

    if should_reroll && shop_state.is_open() {
        if shop_state.active_shop_id.as_deref() != Some("shop_armory") {
            return;
        }

        if !wallet.spend(reroll_cost) {
            info!("Cannot afford reroll (need {} gold)", reroll_cost);
            return;
        }

        info!("Rerolling shop inventory for {} gold", reroll_cost);

        shop_state.inventory =
            generate_shop_inventory("shop_armory", &content_registry, &gameplay_defaults);

        for entity in shop_ui_query.iter() {
            commands.entity(entity).despawn();
        }

        spawn_shop_ui(&mut commands, &shop_state, &wallet, "shop_armory");
    }
}

/// Handle shop item purchase via click interaction
pub(crate) fn handle_shop_purchase(
    mut button_query: Query<
        (&ShopItemButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, Without<CloseShopButton>),
    >,
    mut close_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            With<CloseShopButton>,
            Changed<Interaction>,
            Without<ShopItemButton>,
        ),
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    mut purchase_events: MessageWriter<ItemPurchasedEvent>,
    mut close_events: MessageWriter<CloseShopEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(item) = shop_state.inventory.get(button.index) {
                    if wallet.spend(item.price) {
                        match item.slot {
                            EquipmentSlot::Helmet => {
                                player_build.equipment.helmet = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Chestplate => {
                                player_build.equipment.chestplate = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Greaves => {
                                player_build.equipment.greaves = Some(item.item_id.clone())
                            }
                            EquipmentSlot::Boots => {
                                player_build.equipment.boots = Some(item.item_id.clone())
                            }
                            EquipmentSlot::MainHand => {
                                player_build.equipment.main_hand = Some(item.item_id.clone())
                            }
                        }

                        purchase_events.write(ItemPurchasedEvent {
                            item_id: item.item_id.clone(),
                            price: item.price,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: item.price,
                            reason: SpendReason::ShopPurchase(item.name.clone()),
                        });

                        info!("Purchased {} for {} coins", item.name, item.price);
                    } else {
                        info!(
                            "Cannot afford {} (need {} coins, have {})",
                            item.name, item.price, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }

    for (interaction, mut bg_color, mut border_color) in &mut close_button_query {
        match interaction {
            Interaction::Pressed => {
                close_events.write(CloseShopEvent);
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.35));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

/// Handle keyboard input for shop (number keys to buy, Esc to close)
pub(crate) fn handle_shop_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    shop_state: Res<ShopState>,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    mut purchase_events: MessageWriter<ItemPurchasedEvent>,
    mut close_events: MessageWriter<CloseShopEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    if !shop_state.is_open() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        close_events.write(CloseShopEvent);
        return;
    }

    let key_pressed = if keyboard.just_pressed(KeyCode::Digit1)
        || keyboard.just_pressed(KeyCode::Numpad1)
    {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        Some(3)
    } else if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5) {
        Some(4)
    } else if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6) {
        Some(5)
    } else if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7) {
        Some(6)
    } else if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8) {
        Some(7)
    } else if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9) {
        Some(8)
    } else {
        None
    };

    if let Some(index) = key_pressed {
        if let Some(item) = shop_state.inventory.get(index) {
            if wallet.spend(item.price) {
                match item.slot {
                    EquipmentSlot::Helmet => {
                        player_build.equipment.helmet = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Chestplate => {
                        player_build.equipment.chestplate = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Greaves => {
                        player_build.equipment.greaves = Some(item.item_id.clone())
                    }
                    EquipmentSlot::Boots => {
                        player_build.equipment.boots = Some(item.item_id.clone())
                    }
                    EquipmentSlot::MainHand => {
                        player_build.equipment.main_hand = Some(item.item_id.clone())
                    }
                }

                purchase_events.write(ItemPurchasedEvent {
                    item_id: item.item_id.clone(),
                    price: item.price,
                });
                coin_events.write(CoinSpentEvent {
                    amount: item.price,
                    reason: SpendReason::ShopPurchase(item.name.clone()),
                });

                info!(
                    "Purchased {} for {} coins via keyboard",
                    item.name, item.price
                );
            } else {
                info!(
                    "Cannot afford {} (need {} coins, have {})",
                    item.name, item.price, wallet.coins
                );
            }
        }
    }
}

/// Handle upgrade item purchase via click interaction (Blacksmith)
pub(crate) fn handle_upgrade_purchase(
    mut button_query: Query<
        (&UpgradeItemButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    content_registry: Res<ContentRegistry>,
    mut upgrade_events: MessageWriter<ItemUpgradedEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(option) = shop_state.upgrade_options.get(button.index) {
                    if wallet.spend(option.cost) {
                        let upgraded_item_id = find_upgraded_item(
                            &option.item_id,
                            option.target_tier,
                            &content_registry,
                        )
                        .unwrap_or_else(|| option.item_id.clone());

                        match option.slot {
                            EquipmentSlot::Helmet => {
                                player_build.equipment.helmet = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Chestplate => {
                                player_build.equipment.chestplate = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Greaves => {
                                player_build.equipment.greaves = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::Boots => {
                                player_build.equipment.boots = Some(upgraded_item_id.clone())
                            }
                            EquipmentSlot::MainHand => {
                                player_build.equipment.main_hand = Some(upgraded_item_id.clone())
                            }
                        }

                        upgrade_events.write(ItemUpgradedEvent {
                            slot: option.slot,
                            item_id: upgraded_item_id,
                            new_tier: option.target_tier,
                            cost: option.cost,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: option.cost,
                            reason: SpendReason::Upgrade(option.name.clone()),
                        });

                        info!(
                            "Upgraded {} from {} to {} for {} coins",
                            option.name,
                            option.current_tier.display_name(),
                            option.target_tier.display_name(),
                            option.cost
                        );
                    } else {
                        info!(
                            "Cannot afford upgrade (need {} coins, have {})",
                            option.cost, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }
}

/// Find an item of the target tier in the same slot category
fn find_upgraded_item(
    current_item_id: &str,
    target_tier: RewardTier,
    registry: &ContentRegistry,
) -> Option<String> {
    let current_item = registry.equipment_items.get(current_item_id)?;
    let target_slot = current_item.slot;

    for (id, item) in registry.equipment_items.iter() {
        if item.slot == target_slot && item.tier == target_tier.level() as u32 {
            return Some(id.clone());
        }
    }

    None
}

/// Handle enchant item purchase via click interaction (Enchanter)
pub(crate) fn handle_enchant_purchase(
    mut button_query: Query<
        (&EnchantItemButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut wallet: ResMut<PlayerWallet>,
    mut player_build: ResMut<PlayerBuild>,
    shop_state: Res<ShopState>,
    mut enchant_events: MessageWriter<ItemEnchantedEvent>,
    mut coin_events: MessageWriter<CoinSpentEvent>,
) {
    for (button, interaction, mut bg_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                if let Some(option) = shop_state.enchant_options.get(button.index) {
                    if wallet.spend(option.cost) {
                        if !player_build.unlocked_nodes.contains(&option.passive_id) {
                            player_build.unlocked_nodes.push(option.passive_id.clone());
                        }

                        enchant_events.write(ItemEnchantedEvent {
                            slot: option.slot,
                            item_id: option.item_id.clone(),
                            passive_id: option.passive_id.clone(),
                            cost: option.cost,
                        });
                        coin_events.write(CoinSpentEvent {
                            amount: option.cost,
                            reason: SpendReason::Enchant(option.passive_name.clone()),
                        });

                        info!(
                            "Enchanted {} with {} for {} coins",
                            option.item_name, option.passive_name, option.cost
                        );
                    } else {
                        info!(
                            "Cannot afford enchant (need {} coins, have {})",
                            option.cost, wallet.coins
                        );
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.35, 0.45));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.22, 0.28));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
            }
        }
    }
}
