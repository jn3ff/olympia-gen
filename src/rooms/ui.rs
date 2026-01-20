//! Rooms domain: UI helpers for arena and portal/shop tooltips.

use bevy::prelude::*;

use crate::content::GameplayDefaults;
use crate::core::SegmentProgress;
use crate::movement::Player;
use crate::rooms::components::{ArenaPortal, NearShop, PortalEnabled, RoomExit};
use crate::rooms::graph::PlayerInPortalZone;

/// Marker for the arena segment info UI
#[derive(Component, Debug)]
pub struct ArenaSegmentInfo;

/// Marker for the "Press [E] to enter" tooltip UI
#[derive(Component, Debug)]
pub struct PortalTooltipUI;

/// Marker for shop tooltip UI
#[derive(Component, Debug)]
pub struct ShopTooltipUI;

/// Marker for shop name label (Text2d above shop NPC)
#[derive(Component, Debug)]
pub struct ShopNameLabel;

pub(crate) fn spawn_segment_info_ui(
    commands: &mut Commands,
    segment_index: u32,
    segment_progress: &SegmentProgress,
    gameplay_defaults: Option<&GameplayDefaults>,
) {
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let accent_color = Color::srgb(0.8, 0.7, 0.3);
    let muted_color = Color::srgb(0.6, 0.6, 0.7);
    let progress_color = Color::srgb(0.5, 0.8, 0.5);

    // Get targets from defaults or use fallbacks
    let rooms_target = gameplay_defaults
        .map(|d| d.segment_defaults.rooms_per_segment)
        .unwrap_or(5);
    let bosses_target = gameplay_defaults
        .map(|d| d.segment_defaults.bosses_per_segment)
        .unwrap_or(2);
    let total_boss_target = gameplay_defaults
        .map(|d| d.win_condition.boss_target)
        .unwrap_or(5);

    commands
        .spawn((
            ArenaSegmentInfo,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Segment title
            parent.spawn((
                Text::new(format!("SEGMENT {}", segment_index + 1)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(accent_color),
            ));

            // Difficulty indicator
            let difficulty_text = match segment_index {
                0 => "Difficulty: Normal",
                1 => "Difficulty: Moderate",
                2 => "Difficulty: Challenging",
                3..=4 => "Difficulty: Hard",
                5..=6 => "Difficulty: Very Hard",
                _ => "Difficulty: Extreme",
            };

            parent.spawn((
                Text::new(difficulty_text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Segment progress
            parent.spawn((
                Text::new(format!(
                    "Rooms: {}/{}  |  Bosses: {}/{}",
                    segment_progress.rooms_cleared_this_segment,
                    rooms_target,
                    segment_progress.bosses_defeated_this_segment,
                    bosses_target
                )),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(progress_color),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Total boss progress (win condition)
            parent.spawn((
                Text::new(format!(
                    "Total Bosses Defeated: {}/{}",
                    segment_progress.total_bosses_defeated, total_boss_target
                )),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_color),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Instructions
            parent.spawn((
                Text::new("Choose a direction to begin"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_color),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
        });
}

/// Updates the portal tooltip UI - shows "Press [E] to enter" when player is in portal zone.
pub(crate) fn update_portal_tooltip(
    mut commands: Commands,
    player_query: Query<Option<&PlayerInPortalZone>, With<Player>>,
    exit_query: Query<(&RoomExit, Option<&PortalEnabled>)>,
    existing_tooltip: Query<Entity, With<PortalTooltipUI>>,
) {
    // Check if player is in a portal zone
    let Ok(maybe_zone) = player_query.single() else {
        // No player, cleanup any tooltip
        for entity in &existing_tooltip {
            commands.entity(entity).despawn();
        }
        return;
    };

    match maybe_zone {
        Some(player_zone) => {
            // Player is in a portal zone - check if portal is enabled
            let portal_enabled = exit_query
                .get(player_zone.portal_entity)
                .map(|(_, enabled)| enabled.is_some())
                .unwrap_or(false);

            if portal_enabled {
                // Show tooltip if not already shown
                if existing_tooltip.is_empty() {
                    spawn_portal_tooltip(&mut commands);
                }
            } else {
                // Portal not enabled, hide tooltip
                for entity in &existing_tooltip {
                    commands.entity(entity).despawn();
                }
            }
        }
        None => {
            // Player not in portal zone, hide tooltip
            for entity in &existing_tooltip {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_portal_tooltip(commands: &mut Commands) {
    commands
        .spawn((
            PortalTooltipUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                    BorderColor::all(Color::srgb(0.3, 0.6, 0.4)),
                ))
                .with_child((
                    Text::new("Press [E] to enter"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.3, 0.9, 0.4)),
                ));
        });
}

/// Updates the portal tooltip UI for arena portals - shows "Press [E] to enter" when player is in portal zone.
pub(crate) fn update_arena_portal_tooltip(
    mut commands: Commands,
    player_query: Query<Option<&PlayerInPortalZone>, With<Player>>,
    portal_query: Query<(&ArenaPortal, Option<&PortalEnabled>)>,
    existing_tooltip: Query<Entity, With<PortalTooltipUI>>,
) {
    // Check if player is in a portal zone
    let Ok(maybe_zone) = player_query.single() else {
        // No player, cleanup any tooltip
        for entity in &existing_tooltip {
            commands.entity(entity).despawn();
        }
        return;
    };

    match maybe_zone {
        Some(player_zone) => {
            // Player is in a portal zone - check if it's an arena portal and if it's enabled
            let portal_enabled = portal_query
                .get(player_zone.portal_entity)
                .map(|(_, enabled)| enabled.is_some())
                .unwrap_or(false);

            if portal_enabled {
                // Show tooltip if not already shown
                if existing_tooltip.is_empty() {
                    spawn_portal_tooltip(&mut commands);
                }
            } else {
                // Portal not enabled, hide tooltip
                for entity in &existing_tooltip {
                    commands.entity(entity).despawn();
                }
            }
        }
        None => {
            // Player not in portal zone, hide tooltip
            for entity in &existing_tooltip {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Show tooltip when player is near a shop
pub(crate) fn update_shop_tooltip(
    mut commands: Commands,
    player_near_shop: Query<&NearShop, With<Player>>,
    existing_tooltip: Query<Entity, With<ShopTooltipUI>>,
) {
    // If player is near shop and no tooltip exists, spawn one
    if let Ok(near_shop) = player_near_shop.single() {
        if existing_tooltip.is_empty() {
            let shop_name = match near_shop.shop_id.as_str() {
                "shop_armory" => "Armory",
                "shop_blacksmith" => "Blacksmith",
                "shop_enchanter" => "Enchanter",
                _ => "Shop",
            };

            commands.spawn((
                ShopTooltipUI,
                Text2d::new(format!("Press [E] to open {}", shop_name)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                Transform::from_xyz(0.0, 80.0, 10.0),
            ));
        }
    } else {
        // Player not near shop, remove tooltip
        for entity in &existing_tooltip {
            commands.entity(entity).despawn();
        }
    }
}
