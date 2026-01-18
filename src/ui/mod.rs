use bevy::prelude::*;

use crate::combat::{BossAI, Enemy, Health};
use crate::core::RunState;
use crate::movement::Player;

// ============================================================================
// Constants
// ============================================================================

const PLAYER_HEALTHBAR_WIDTH: f32 = 200.0;
const PLAYER_HEALTHBAR_HEIGHT: f32 = 20.0;
const PLAYER_HEALTHBAR_PADDING: f32 = 16.0;

const ENEMY_HEALTHBAR_WIDTH: f32 = 40.0;
const ENEMY_HEALTHBAR_HEIGHT: f32 = 6.0;
const ENEMY_HEALTHBAR_OFFSET_Y: f32 = 25.0;

const BOSS_HEALTHBAR_WIDTH: f32 = 400.0;
const BOSS_HEALTHBAR_HEIGHT: f32 = 24.0;
const BOSS_HEALTHBAR_BOTTOM: f32 = 40.0;

// ============================================================================
// Components
// ============================================================================

/// Marker for the player's HUD health bar container
#[derive(Component)]
pub struct PlayerHealthBarUI;

/// Marker for the player's health bar fill element
#[derive(Component)]
pub struct PlayerHealthBarFill;

/// Marker for enemy world-space health bars
#[derive(Component)]
pub struct EnemyHealthBar {
    pub owner: Entity,
}

/// Marker for the fill portion of enemy health bars
#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub owner: Entity,
}

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

/// Marker for the death screen overlay
#[derive(Component)]
pub struct DeathScreenUI;

/// Marker for the retry button on death screen
#[derive(Component)]
pub struct RetryButton;

/// Resource to track if player has died (prevents multiple death screens)
#[derive(Resource, Default)]
pub struct PlayerDeathState {
    pub is_dead: bool,
}

// ============================================================================
// Plugin
// ============================================================================

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>()
            .add_systems(Startup, spawn_player_healthbar_ui)
            .add_systems(
                Update,
                (
                    update_player_healthbar,
                    spawn_enemy_healthbars,
                    update_enemy_healthbars,
                    cleanup_enemy_healthbars,
                    spawn_boss_healthbar,
                    update_boss_healthbar,
                    cleanup_boss_healthbar,
                    detect_player_death,
                    handle_retry_button,
                ),
            );
    }
}

// ============================================================================
// Player Health Bar Systems
// ============================================================================

fn spawn_player_healthbar_ui(mut commands: Commands) {
    // Root container positioned at top-left
    commands
        .spawn((
            PlayerHealthBarUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PLAYER_HEALTHBAR_PADDING),
                top: Val::Px(PLAYER_HEALTHBAR_PADDING),
                width: Val::Px(PLAYER_HEALTHBAR_WIDTH),
                height: Val::Px(PLAYER_HEALTHBAR_HEIGHT),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|parent| {
            // Health bar fill
            parent.spawn((
                PlayerHealthBarFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.3)),
            ));
        });
}

fn update_player_healthbar(
    player_query: Query<&Health, With<Player>>,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor), With<PlayerHealthBarFill>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    for (mut node, mut bg_color) in &mut fill_query {
        let percent = health.percent();
        node.width = Val::Percent(percent * 100.0);

        // Color gradient: green -> yellow -> red
        let color = if percent > 0.5 {
            let t = (percent - 0.5) * 2.0;
            Color::srgb(1.0 - t * 0.8, 0.8, 0.3 * (1.0 - t))
        } else {
            let t = percent * 2.0;
            Color::srgb(0.9, 0.2 + t * 0.6, 0.2)
        };
        bg_color.0 = color;
    }
}

// ============================================================================
// Enemy Health Bar Systems
// ============================================================================

fn spawn_enemy_healthbars(
    mut commands: Commands,
    enemy_query: Query<Entity, (With<Enemy>, Without<BossAI>, Added<Enemy>)>,
    existing_bars: Query<&EnemyHealthBar>,
) {
    for enemy_entity in &enemy_query {
        // Check if this enemy already has a health bar
        let has_bar = existing_bars.iter().any(|bar| bar.owner == enemy_entity);
        if has_bar {
            continue;
        }

        // Spawn the health bar as a world-space sprite
        // Background (dark)
        commands.spawn((
            EnemyHealthBar {
                owner: enemy_entity,
            },
            Sprite {
                color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                custom_size: Some(Vec2::new(ENEMY_HEALTHBAR_WIDTH + 2.0, ENEMY_HEALTHBAR_HEIGHT + 2.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 5.0),
        ));

        // Foreground (health fill)
        commands.spawn((
            EnemyHealthBarFill {
                owner: enemy_entity,
            },
            Sprite {
                color: Color::srgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::new(ENEMY_HEALTHBAR_WIDTH, ENEMY_HEALTHBAR_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 6.0),
        ));
    }
}

fn update_enemy_healthbars(
    enemy_query: Query<(&Transform, &Health, &Sprite), (With<Enemy>, Without<BossAI>)>,
    mut bar_query: Query<(&EnemyHealthBar, &mut Transform), Without<Enemy>>,
    mut fill_query: Query<
        (&EnemyHealthBarFill, &mut Transform, &mut Sprite),
        (Without<Enemy>, Without<EnemyHealthBar>),
    >,
) {
    // Update background bar positions
    for (bar, mut bar_transform) in &mut bar_query {
        if let Ok((enemy_transform, _health, enemy_sprite)) = enemy_query.get(bar.owner) {
            let enemy_height = enemy_sprite
                .custom_size
                .map(|s| s.y)
                .unwrap_or(32.0);
            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y =
                enemy_transform.translation.y + enemy_height / 2.0 + ENEMY_HEALTHBAR_OFFSET_Y;
        }
    }

    // Update fill bars
    for (fill, mut fill_transform, mut fill_sprite) in &mut fill_query {
        if let Ok((enemy_transform, health, enemy_sprite)) = enemy_query.get(fill.owner) {
            let enemy_height = enemy_sprite
                .custom_size
                .map(|s| s.y)
                .unwrap_or(32.0);
            let percent = health.percent();
            let fill_width = ENEMY_HEALTHBAR_WIDTH * percent;

            // Position the fill bar (left-aligned within the background)
            fill_transform.translation.x =
                enemy_transform.translation.x - (ENEMY_HEALTHBAR_WIDTH - fill_width) / 2.0;
            fill_transform.translation.y =
                enemy_transform.translation.y + enemy_height / 2.0 + ENEMY_HEALTHBAR_OFFSET_Y;

            // Update size
            fill_sprite.custom_size = Some(Vec2::new(fill_width.max(0.0), ENEMY_HEALTHBAR_HEIGHT));

            // Update color based on health percent
            let color = if percent > 0.5 {
                Color::srgb(0.8, 0.2, 0.2)
            } else if percent > 0.25 {
                Color::srgb(0.9, 0.5, 0.1)
            } else {
                Color::srgb(0.6, 0.1, 0.1)
            };
            fill_sprite.color = color;
        }
    }
}

fn cleanup_enemy_healthbars(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    bar_query: Query<(Entity, &EnemyHealthBar)>,
    fill_query: Query<(Entity, &EnemyHealthBarFill)>,
) {
    // Clean up bars for dead enemies
    for (bar_entity, bar) in &bar_query {
        if enemy_query.get(bar.owner).is_err() {
            commands.entity(bar_entity).despawn();
        }
    }

    for (fill_entity, fill) in &fill_query {
        if enemy_query.get(fill.owner).is_err() {
            commands.entity(fill_entity).despawn();
        }
    }
}

// ============================================================================
// Boss Health Bar Systems
// ============================================================================

fn spawn_boss_healthbar(
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
                    BossHealthBarFill {
                        owner: boss_entity,
                    },
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

fn update_boss_healthbar(
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

fn cleanup_boss_healthbar(
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

// ============================================================================
// Death Screen Systems
// ============================================================================

fn detect_player_death(
    mut commands: Commands,
    player_query: Query<&Health, With<Player>>,
    mut death_state: ResMut<PlayerDeathState>,
    existing_death_screen: Query<Entity, With<DeathScreenUI>>,
) {
    // Skip if death screen already shown
    if death_state.is_dead {
        return;
    }

    // Check if player health is zero or below
    let Ok(health) = player_query.single() else {
        return;
    };

    if health.is_dead() {
        death_state.is_dead = true;

        // Only spawn if not already showing
        if existing_death_screen.is_empty() {
            spawn_death_screen(&mut commands);
        }
    }
}

fn spawn_death_screen(commands: &mut Commands) {
    // Full screen dark overlay
    commands
        .spawn((
            DeathScreenUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            // High z-index to be on top of everything
            ZIndex(100),
        ))
        .with_children(|parent| {
            // "YOU DIED" text
            parent.spawn((
                Text::new("YOU DIED"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.15, 0.15)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Subtext - leaving room for narrative
            parent.spawn((
                Text::new("Your journey ends here... for now."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Retry button
            parent
                .spawn((
                    RetryButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                ))
                .with_child((
                    Text::new("RETRY"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));

            // Hint text
            parent.spawn((
                Text::new("Press [Enter] or click to retry"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.45)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

fn handle_retry_button(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (With<RetryButton>, Changed<Interaction>)>,
    death_screen_query: Query<Entity, With<DeathScreenUI>>,
    mut death_state: ResMut<PlayerDeathState>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    // Check if we should retry
    let should_retry = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::NumpadEnter)
        || button_query
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed);

    if !should_retry || !death_state.is_dead {
        return;
    }

    // Reset death state
    death_state.is_dead = false;

    // Despawn death screen
    for entity in &death_screen_query {
        commands.entity(entity).despawn();
    }

    // Reset player health
    for mut health in &mut player_query {
        health.current = health.max;
    }

    // Return to arena for a fresh start
    next_state.set(RunState::Arena);
}
