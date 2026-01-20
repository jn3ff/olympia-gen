//! Debug domain: UI components and layout helpers.

use bevy::prelude::*;

use crate::debug::state::{DebugAction, DebugState};

/// Marker for the debug UI root
#[derive(Component, Debug)]
pub struct DebugUI;

/// Marker for debug info overlay (position, health, etc.)
#[derive(Component, Debug)]
pub struct DebugInfoOverlay;

/// Marker for status message text
#[derive(Component, Debug)]
pub struct DebugStatusMessage;

/// Debug panel button
#[derive(Component, Debug)]
pub struct DebugButton {
    pub action: DebugAction,
}

pub(crate) fn spawn_debug_ui(commands: &mut Commands) {
    let bg_color = Color::srgba(0.1, 0.1, 0.15, 0.95);
    let button_color = Color::srgb(0.2, 0.2, 0.28);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let title_color = Color::srgb(0.9, 0.7, 0.3);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);

    commands
        .spawn((
            DebugUI,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(Color::srgb(0.4, 0.35, 0.2)),
            ZIndex(500),
        ))
        .with_children(|parent| {
            // Title bar
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("DEBUG MODE"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(title_color),
                    ));

                    // Close button
                    spawn_debug_button(row, "X", DebugAction::Close, button_color, text_color);
                });

            // Hotkeys hint
            parent.spawn((
                Text::new("F1 or ` to toggle | Ctrl+Key for hotkeys"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Status message area
            parent.spawn((
                DebugStatusMessage,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.9, 0.5)),
                Node {
                    min_height: Val::Px(16.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // === Player Section ===
            spawn_section_header(parent, "Player", title_color);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_debug_button(
                        row,
                        "Invincible [Ctrl+I]",
                        DebugAction::ToggleInvincible,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Full Heal [Ctrl+H]",
                        DebugAction::FullHeal,
                        button_color,
                        text_color,
                    );
                });

            // === Spawning Section ===
            spawn_section_header(parent, "Spawning", title_color);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_debug_button(
                        row,
                        "Tier [Ctrl+T]",
                        DebugAction::CycleTier,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Enemy [Ctrl+E]",
                        DebugAction::SpawnEnemy,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Boss [Ctrl+B]",
                        DebugAction::SpawnBoss,
                        button_color,
                        text_color,
                    );
                });

            // === Warp Section ===
            spawn_section_header(parent, "Warp", title_color);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_debug_button(
                        row,
                        "Arena [Ctrl+1]",
                        DebugAction::WarpToArena,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Room [Ctrl+2]",
                        DebugAction::WarpToRoom,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Boss [Ctrl+3]",
                        DebugAction::WarpToBoss,
                        button_color,
                        text_color,
                    );
                });

            // === Cheats Section ===
            spawn_section_header(parent, "Cheats", title_color);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_debug_button(
                        row,
                        "+500 Gold [Ctrl+M]",
                        DebugAction::GiveMoney,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Kill All [Ctrl+K]",
                        DebugAction::KillAllEnemies,
                        button_color,
                        text_color,
                    );
                });

            // === Misc Section ===
            spawn_section_header(parent, "Misc", title_color);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_debug_button(
                        row,
                        "New Seed [Ctrl+S]",
                        DebugAction::SetSeed,
                        button_color,
                        text_color,
                    );
                    spawn_debug_button(
                        row,
                        "Info [Ctrl+D]",
                        DebugAction::ToggleInfo,
                        button_color,
                        text_color,
                    );
                });
        });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str, color: Color) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(color),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(4.0)),
            ..default()
        },
    ));
}

fn spawn_debug_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: DebugAction,
    bg_color: Color,
    text_color: Color,
) {
    parent
        .spawn((
            DebugButton { action },
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(Color::srgb(0.35, 0.35, 0.45)),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(text_color),
        ));
}

pub(crate) fn spawn_debug_info_overlay(commands: &mut Commands) {
    commands.spawn((
        DebugInfoOverlay,
        Text::new("Loading..."),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.9, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ZIndex(500),
    ));
}

pub(crate) fn refresh_debug_ui(
    commands: &mut Commands,
    debug_state: &DebugState,
    existing_ui: &Query<Entity, With<DebugUI>>,
) {
    // Despawn and respawn to refresh state
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn();
    }
    if debug_state.ui_visible {
        spawn_debug_ui(commands);
    }
}
