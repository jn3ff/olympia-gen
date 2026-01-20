//! Rewards domain: shop UI components and builders.

use bevy::prelude::*;

use crate::rewards::economy::PlayerWallet;
use crate::rewards::shop::{EnchantOption, ShopItem, ShopState, UpgradeOption};

/// Marker for the shop UI root
#[derive(Component, Debug)]
pub struct ShopUI;

/// Marker for shop item buttons
#[derive(Component, Debug)]
pub struct ShopItemButton {
    pub index: usize,
}

/// Marker for close shop button
#[derive(Component, Debug)]
pub struct CloseShopButton;

/// Marker for reroll shop button
#[derive(Component, Debug)]
pub struct RerollShopButton;

/// Marker for upgrade item buttons (Blacksmith)
#[derive(Component, Debug)]
pub struct UpgradeItemButton {
    pub index: usize,
}

/// Marker for enchant item buttons (Enchanter)
#[derive(Component, Debug)]
pub struct EnchantItemButton {
    pub index: usize,
}

/// Spawn the shop UI overlay
pub(crate) fn spawn_shop_ui(
    commands: &mut Commands,
    shop_state: &ShopState,
    wallet: &PlayerWallet,
    shop_id: &str,
) {
    let bg_color = Color::srgba(0.05, 0.05, 0.1, 0.95);
    let panel_color = Color::srgb(0.12, 0.12, 0.18);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);
    let gold_color = Color::srgb(0.9, 0.75, 0.2);

    let shop_name = match shop_id {
        "shop_armory" => "Armory",
        "shop_blacksmith" => "Blacksmith",
        "shop_enchanter" => "Enchanter",
        _ => "Shop",
    };

    commands
        .spawn((
            ShopUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            ZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Px(700.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|header| {
                    header.spawn((
                        Text::new(shop_name.to_uppercase()),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));

                    header
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|coins| {
                            coins.spawn((
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(gold_color),
                            ));
                            coins.spawn((
                                Text::new(format!("{}", wallet.coins)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(gold_color),
                            ));
                        });
                });

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    column_gap: Val::Px(15.0),
                    row_gap: Val::Px(15.0),
                    width: Val::Px(720.0),
                    max_height: Val::Px(450.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },))
                .with_children(|grid| match shop_id {
                    "shop_blacksmith" => {
                        if shop_state.upgrade_options.is_empty() {
                            grid.spawn((
                                Text::new("No equipment to upgrade\n(Equip items first)"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(muted_text),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        } else {
                            for (index, option) in shop_state.upgrade_options.iter().enumerate() {
                                spawn_upgrade_card(
                                    grid,
                                    index,
                                    option,
                                    wallet.coins,
                                    panel_color,
                                    text_color,
                                    muted_text,
                                    gold_color,
                                );
                            }
                        }
                    }
                    "shop_enchanter" => {
                        if shop_state.enchant_options.is_empty() {
                            grid.spawn((
                                Text::new("No enchantments available\n(Equip items first)"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(muted_text),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        } else {
                            for (index, option) in shop_state.enchant_options.iter().enumerate() {
                                spawn_enchant_card(
                                    grid,
                                    index,
                                    option,
                                    wallet.coins,
                                    panel_color,
                                    text_color,
                                    muted_text,
                                    gold_color,
                                );
                            }
                        }
                    }
                    _ => {
                        if shop_state.inventory.is_empty() {
                            grid.spawn((
                                Text::new("No items available"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(muted_text),
                            ));
                        } else {
                            for (index, item) in shop_state.inventory.iter().enumerate() {
                                spawn_shop_item_card(
                                    grid,
                                    index,
                                    item,
                                    wallet.coins,
                                    panel_color,
                                    text_color,
                                    muted_text,
                                    gold_color,
                                );
                            }
                        }
                    }
                });

            let reroll_cost = 25u32;
            let can_afford_reroll = wallet.coins >= reroll_cost;
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(25.0)),
                    ..default()
                },))
                .with_children(|buttons| {
                    if shop_id == "shop_armory" {
                        let reroll_border = if can_afford_reroll {
                            Color::srgb(0.5, 0.6, 0.4)
                        } else {
                            Color::srgb(0.4, 0.3, 0.3)
                        };
                        buttons
                            .spawn((
                                RerollShopButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderColor::all(reroll_border),
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                            ))
                            .with_child((
                                Text::new(format!("Reroll [R] - {} gold", reroll_cost)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(if can_afford_reroll {
                                    gold_color
                                } else {
                                    Color::srgb(0.5, 0.4, 0.4)
                                }),
                            ));
                    }

                    buttons
                        .spawn((
                            CloseShopButton,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(30.0), Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                        ))
                        .with_child((
                            Text::new("Close [Esc]"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(muted_text),
                        ));
                });

            parent.spawn((
                Text::new("Press 1-5 to purchase, R to reroll, Esc to close"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_shop_item_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    item: &ShopItem,
    player_coins: u32,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let tier_accent = item.tier.accent_color();
    let can_afford = player_coins >= item.price;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };
    let border_thickness = 2.0 + (item.tier.level() as f32 - 1.0) * 0.5;

    parent
        .spawn((
            ShopItemButton { index },
            Button,
            Node {
                width: Val::Px(160.0),
                min_height: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(border_thickness)),
                ..default()
            },
            BorderColor::all(if can_afford {
                tier_accent
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            card.spawn((
                Text::new(item.tier.display_name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(tier_accent),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(item.slot.name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(&item.name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford { text_color } else { muted_text }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(&item.description),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(format!("{} coins", item.price)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}

fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    option: &UpgradeOption,
    player_coins: u32,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let current_accent = option.current_tier.accent_color();
    let target_accent = option.target_tier.accent_color();
    let can_afford = player_coins >= option.cost;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };

    parent
        .spawn((
            UpgradeItemButton { index },
            Button,
            Node {
                width: Val::Px(180.0),
                min_height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(if can_afford {
                target_accent
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            card.spawn((
                Text::new(option.slot.name().to_uppercase()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(&option.name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford { text_color } else { muted_text }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            card.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },))
                .with_children(|row| {
                    row.spawn((
                        Text::new(option.current_tier.display_name()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(current_accent),
                    ));
                    row.spawn((
                        Text::new("→"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));
                    row.spawn((
                        Text::new(option.target_tier.display_name()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(target_accent),
                    ));
                });

            card.spawn((
                Text::new(format!("{} coins", option.cost)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}

fn spawn_enchant_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    option: &EnchantOption,
    player_coins: u32,
    panel_color: Color,
    _text_color: Color,
    muted_text: Color,
    gold_color: Color,
) {
    let can_afford = player_coins >= option.cost;
    let price_color = if can_afford {
        gold_color
    } else {
        Color::srgb(0.7, 0.3, 0.3)
    };
    let enchant_color = Color::srgb(0.6, 0.4, 0.9);

    parent
        .spawn((
            EnchantItemButton { index },
            Button,
            Node {
                width: Val::Px(180.0),
                min_height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(if can_afford {
                enchant_color
            } else {
                Color::srgb(0.4, 0.4, 0.4)
            }),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            if index < 9 {
                card.spawn((
                    Text::new(format!("[{}]", index + 1)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(muted_text),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            card.spawn((
                Text::new(format!("{} ({})", option.item_name, option.slot.name())),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(&option.passive_name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if can_afford {
                    enchant_color
                } else {
                    muted_text
                }),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(&option.passive_description),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(format!("{} coins", option.cost)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(price_color),
            ));
        });
}
