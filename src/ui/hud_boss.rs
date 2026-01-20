//! UI domain: boss health bar UI.

use bevy::prelude::*;

use crate::combat::{BossAI, Health};

const BOSS_HEALTHBAR_WIDTH: f32 = 400.0;
const BOSS_HEALTHBAR_HEIGHT: f32 = 24.0;
const BOSS_HEALTHBAR_BOTTOM: f32 = 40.0;

/// Marker for the boss health bar UI container
#[derive(Component)]
pub struct BossHealthBarUI;

/// Marker for the boss health bar fill element
#[derive(Component)]
pub struct BossHealthBarFill {
    pub owner: Entity,
}

/// Marker for boss name label
#[derive(Component)]
pub struct BossNameLabel;

pub(crate) fn spawn_boss_healthbar(
    mut commands: Commands,
    boss_query: Query<Entity, Added<BossAI>>,
    existing_bars: Query<&BossHealthBarFill>,
) {
    for boss_entity in &boss_query {
        // Check if this boss already has a health bar
        let has_bar = existing_bars.iter().any(|bar| bar.owner == boss_entity);
        if has_bar {
            continue;
        }

        // Spawn boss health bar UI at bottom center of screen
        commands
            .spawn((
                BossHealthBarUI,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(BOSS_HEALTHBAR_BOTTOM),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-BOSS_HEALTHBAR_WIDTH / 2.0)),
                    width: Val::Px(BOSS_HEALTHBAR_WIDTH),
                    height: Val::Px(BOSS_HEALTHBAR_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9)),
                BorderColor::all(Color::srgb(0.6, 0.1, 0.1)),
            ))
            .with_children(|parent| {
                // Health bar fill
                parent.spawn((
                    BossHealthBarFill { owner: boss_entity },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
                ));
            });

        // Spawn boss name label above the health bar
        commands.spawn((
            BossNameLabel,
            Text::new("BOSS"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOSS_HEALTHBAR_BOTTOM + BOSS_HEALTHBAR_HEIGHT + 8.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-30.0)),
                ..default()
            },
        ));
    }
}

pub(crate) fn update_boss_healthbar(
    boss_query: Query<&Health, With<BossAI>>,
    mut fill_query: Query<(&BossHealthBarFill, &mut Node, &mut BackgroundColor)>,
) {
    for (fill, mut node, mut bg_color) in &mut fill_query {
        if let Ok(health) = boss_query.get(fill.owner) {
            let percent = health.percent();
            node.width = Val::Percent(percent * 100.0);

            // Boss health bar color changes based on phase thresholds
            let color = if percent > 0.5 {
                Color::srgb(0.8, 0.1, 0.1)
            } else if percent > 0.25 {
                Color::srgb(0.9, 0.4, 0.1)
            } else {
                Color::srgb(0.6, 0.0, 0.0)
            };
            bg_color.0 = color;
        }
    }
}

pub(crate) fn cleanup_boss_healthbar(
    mut commands: Commands,
    boss_query: Query<Entity, With<BossAI>>,
    bar_query: Query<Entity, With<BossHealthBarUI>>,
    fill_query: Query<&BossHealthBarFill>,
    name_query: Query<Entity, With<BossNameLabel>>,
) {
    // Check if any boss exists
    let boss_exists = !boss_query.is_empty();

    // Check if any fill still has a valid owner
    let has_valid_owner = fill_query
        .iter()
        .any(|fill| boss_query.get(fill.owner).is_ok());

    // If no boss exists or no valid owner, clean up
    if !boss_exists || !has_valid_owner {
        for bar_entity in &bar_query {
            commands.entity(bar_entity).despawn();
        }
        for name_entity in &name_query {
            commands.entity(name_entity).despawn();
        }
    }
}
