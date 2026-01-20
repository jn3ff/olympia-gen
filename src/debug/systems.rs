//! Debug domain: debug systems for input and runtime tweaks.

use bevy::prelude::*;

use crate::combat::{
    BossAttackSlots, EnemyBundle, EnemyTier, EnemyTuning, Health, Invulnerable, spawn_boss_scaled,
};
use crate::core::{DifficultyScaling, RunConfig, RunState};
use crate::debug::state::{DebugAction, DebugState};
use crate::debug::ui::{
    DebugButton, DebugInfoOverlay, DebugUI, refresh_debug_ui, spawn_debug_info_overlay,
    spawn_debug_ui,
};
use crate::movement::Player;

/// Toggle debug UI with F1 or backtick key
pub(crate) fn toggle_debug_ui(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    existing_ui: Query<Entity, With<DebugUI>>,
) {
    let toggle = keyboard.just_pressed(KeyCode::F1) || keyboard.just_pressed(KeyCode::Backquote);

    if toggle {
        debug_state.ui_visible = !debug_state.ui_visible;

        if debug_state.ui_visible {
            spawn_debug_ui(&mut commands);
        } else {
            for entity in &existing_ui {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handle keyboard shortcuts for debug actions
pub(crate) fn handle_debug_hotkeys(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut run_config: ResMut<RunConfig>,
    mut run_state: ResMut<NextState<RunState>>,
    difficulty: Res<DifficultyScaling>,
    tuning: Res<EnemyTuning>,
    player_query: Query<(&Transform, Entity), With<Player>>,
    existing_ui: Query<Entity, With<DebugUI>>,
) {
    // Only process hotkeys when debug UI is open or Ctrl is held
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if !debug_state.ui_visible && !ctrl {
        return;
    }

    // Ctrl+I: Toggle invincibility
    if ctrl && keyboard.just_pressed(KeyCode::KeyI) {
        debug_state.invincible = !debug_state.invincible;
        let msg = if debug_state.invincible {
            "Invincibility ON"
        } else {
            "Invincibility OFF"
        };
        debug_state.set_message(msg, 2.0);
        info!("[DEBUG] {}", msg);
        refresh_debug_ui(&mut commands, &debug_state, &existing_ui);
    }

    // Ctrl+E: Spawn enemy at player position
    if ctrl && keyboard.just_pressed(KeyCode::KeyE) {
        if let Some((player_transform, _)) = player_query.iter().next() {
            let pos = player_transform.translation.truncate() + Vec2::new(100.0, 0.0);
            let segment = run_config.segment_index;
            let health_mult = difficulty.enemy_health_multiplier(segment);

            let tier = debug_state.selected_tier;
            let base_health = match tier {
                EnemyTier::Minor => 30.0,
                EnemyTier::Major => 60.0,
                EnemyTier::Special => 100.0,
                EnemyTier::Boss => 200.0,
            };

            commands.spawn(EnemyBundle::new(
                tier,
                pos,
                base_health * health_mult,
                &tuning,
                format!("debug_{:?}", tier).to_lowercase(),
            ));

            debug_state.set_message(format!("Spawned {:?} enemy", tier), 2.0);
            info!("[DEBUG] Spawned {:?} enemy at {:?}", tier, pos);
        }
    }

    // Ctrl+B: Spawn boss at player position
    if ctrl && keyboard.just_pressed(KeyCode::KeyB) {
        if let Some((player_transform, _)) = player_query.iter().next() {
            let pos = player_transform.translation.truncate() + Vec2::new(150.0, 0.0);
            let segment = run_config.segment_index;
            let health_mult = difficulty.boss_health_multiplier(segment);
            let damage_mult = difficulty.boss_damage_multiplier(segment);

            spawn_boss_scaled(
                &mut commands,
                pos,
                500.0,
                BossAttackSlots::default(),
                health_mult,
                damage_mult,
            );

            debug_state.set_message("Spawned Boss", 2.0);
            info!("[DEBUG] Spawned boss at {:?}", pos);
        }
    }

    // Ctrl+H: Full heal
    if ctrl && keyboard.just_pressed(KeyCode::KeyH) {
        debug_state.set_message("Full Heal", 2.0);
        info!("[DEBUG] Full heal triggered");
        // Health restoration is handled in apply_invincibility when invincible,
        // but we can trigger it directly here
    }

    // Ctrl+1: Warp to Arena
    if ctrl && keyboard.just_pressed(KeyCode::Digit1) {
        run_state.set(RunState::Arena);
        debug_state.set_message("Warping to Arena", 2.0);
        info!("[DEBUG] Warping to Arena");
    }

    // Ctrl+2: Warp to Room
    if ctrl && keyboard.just_pressed(KeyCode::Digit2) {
        run_state.set(RunState::Room);
        debug_state.set_message("Warping to Room", 2.0);
        info!("[DEBUG] Warping to Room");
    }

    // Ctrl+3: Warp to Boss
    if ctrl && keyboard.just_pressed(KeyCode::Digit3) {
        run_state.set(RunState::Boss);
        debug_state.set_message("Warping to Boss", 2.0);
        info!("[DEBUG] Warping to Boss");
    }

    // Ctrl+T: Cycle enemy tier
    if ctrl && keyboard.just_pressed(KeyCode::KeyT) {
        let new_tier = match debug_state.selected_tier {
            EnemyTier::Minor => EnemyTier::Major,
            EnemyTier::Major => EnemyTier::Special,
            EnemyTier::Special => EnemyTier::Boss,
            EnemyTier::Boss => EnemyTier::Minor,
        };
        debug_state.selected_tier = new_tier;
        debug_state.set_message(format!("Tier: {:?}", new_tier), 2.0);
        info!("[DEBUG] Selected tier: {:?}", new_tier);
        refresh_debug_ui(&mut commands, &debug_state, &existing_ui);
    }

    // Ctrl+D: Toggle debug info overlay
    if ctrl && keyboard.just_pressed(KeyCode::KeyD) {
        debug_state.show_info = !debug_state.show_info;
        let msg = if debug_state.show_info {
            "Debug Info ON"
        } else {
            "Debug Info OFF"
        };
        debug_state.set_message(msg, 2.0);
        info!("[DEBUG] {}", msg);

        // Spawn or despawn info overlay
        if debug_state.show_info {
            spawn_debug_info_overlay(&mut commands);
        } else {
            // Will be cleaned up naturally when condition is false
        }
    }

    // Ctrl+S: Set seed (uses current input buffer)
    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        if let Ok(seed) = debug_state.seed_input.parse::<u64>() {
            run_config.seed = seed;
            debug_state.set_message(format!("Seed set: {}", seed), 2.0);
            info!("[DEBUG] Seed set to {}", seed);
        } else if debug_state.seed_input.is_empty() {
            // Generate and display current seed
            debug_state.set_message(format!("Current seed: {}", run_config.seed), 3.0);
            info!("[DEBUG] Current seed: {}", run_config.seed);
        } else {
            debug_state.set_message("Invalid seed format", 2.0);
        }
    }
}

/// Handle button clicks in debug UI
pub(crate) fn handle_debug_buttons(
    mut commands: Commands,
    mut debug_state: ResMut<DebugState>,
    mut run_config: ResMut<RunConfig>,
    mut run_state: ResMut<NextState<RunState>>,
    difficulty: Res<DifficultyScaling>,
    tuning: Res<EnemyTuning>,
    mut button_query: Query<(&DebugButton, &Interaction), Changed<Interaction>>,
    player_query: Query<(&Transform, &mut Health), With<Player>>,
    existing_ui: Query<Entity, With<DebugUI>>,
) {
    for (button, interaction) in &mut button_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            DebugAction::ToggleInvincible => {
                debug_state.invincible = !debug_state.invincible;
                let msg = if debug_state.invincible {
                    "Invincibility ON"
                } else {
                    "Invincibility OFF"
                };
                debug_state.set_message(msg, 2.0);
                refresh_debug_ui(&mut commands, &debug_state, &existing_ui);
            }
            DebugAction::SpawnEnemy => {
                if let Some((player_transform, _)) = player_query.iter().next() {
                    let pos = player_transform.translation.truncate() + Vec2::new(100.0, 0.0);
                    let segment = run_config.segment_index;
                    let health_mult = difficulty.enemy_health_multiplier(segment);
                    let tier = debug_state.selected_tier;
                    let base_health = match tier {
                        EnemyTier::Minor => 30.0,
                        EnemyTier::Major => 60.0,
                        EnemyTier::Special => 100.0,
                        EnemyTier::Boss => 200.0,
                    };

                    commands.spawn(EnemyBundle::new(
                        tier,
                        pos,
                        base_health * health_mult,
                        &tuning,
                        format!("debug_{:?}", tier).to_lowercase(),
                    ));
                    debug_state.set_message(format!("Spawned {:?}", tier), 2.0);
                }
            }
            DebugAction::SpawnBoss => {
                if let Some((player_transform, _)) = player_query.iter().next() {
                    let pos = player_transform.translation.truncate() + Vec2::new(150.0, 0.0);
                    let segment = run_config.segment_index;
                    let health_mult = difficulty.boss_health_multiplier(segment);
                    let damage_mult = difficulty.boss_damage_multiplier(segment);

                    spawn_boss_scaled(
                        &mut commands,
                        pos,
                        500.0,
                        BossAttackSlots::default(),
                        health_mult,
                        damage_mult,
                    );
                    debug_state.set_message("Spawned Boss", 2.0);
                }
            }
            DebugAction::WarpToArena => {
                run_state.set(RunState::Arena);
                debug_state.set_message("Warping to Arena", 2.0);
            }
            DebugAction::WarpToRoom => {
                run_state.set(RunState::Room);
                debug_state.set_message("Warping to Room", 2.0);
            }
            DebugAction::WarpToBoss => {
                run_state.set(RunState::Boss);
                debug_state.set_message("Warping to Boss", 2.0);
            }
            DebugAction::SetSeed => {
                // Generate a new random seed and display it
                let new_seed = rand::random::<u64>();
                run_config.seed = new_seed;
                debug_state.set_message(format!("New seed: {}", new_seed), 3.0);
            }
            DebugAction::FullHeal => {
                debug_state.set_message("Full Heal", 2.0);
            }
            DebugAction::CycleTier => {
                let new_tier = match debug_state.selected_tier {
                    EnemyTier::Minor => EnemyTier::Major,
                    EnemyTier::Major => EnemyTier::Special,
                    EnemyTier::Special => EnemyTier::Boss,
                    EnemyTier::Boss => EnemyTier::Minor,
                };
                debug_state.selected_tier = new_tier;
                debug_state.set_message(format!("Tier: {:?}", new_tier), 2.0);
                refresh_debug_ui(&mut commands, &debug_state, &existing_ui);
            }
            DebugAction::ToggleInfo => {
                debug_state.show_info = !debug_state.show_info;
                if debug_state.show_info {
                    spawn_debug_info_overlay(&mut commands);
                }
            }
            DebugAction::Close => {
                debug_state.ui_visible = false;
                for entity in &existing_ui {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Update status message timer and fade out
pub(crate) fn update_status_message(time: Res<Time>, mut debug_state: ResMut<DebugState>) {
    if let Some((_, ref mut duration)) = debug_state.status_message {
        *duration -= time.delta_secs();
        if *duration <= 0.0 {
            debug_state.status_message = None;
        }
    }
}

/// Apply invincibility effect to player
pub(crate) fn apply_invincibility(
    debug_state: Res<DebugState>,
    mut player_query: Query<(&mut Health, &mut Invulnerable), With<Player>>,
) {
    if !debug_state.invincible {
        return;
    }

    for (mut health, mut invuln) in &mut player_query {
        // Keep invulnerability frames active
        invuln.timer = 1.0;

        // Restore health if damaged
        if health.current < health.max {
            health.current = health.max;
        }
    }
}

/// Update the debug info overlay with current player state
pub(crate) fn update_debug_info_overlay(
    mut commands: Commands,
    debug_state: Res<DebugState>,
    run_config: Res<RunConfig>,
    run_state: Res<State<RunState>>,
    player_query: Query<(&Transform, &Health), With<Player>>,
    mut overlay_query: Query<&mut Text, With<DebugInfoOverlay>>,
    existing_overlay: Query<Entity, With<DebugInfoOverlay>>,
) {
    if !debug_state.show_info {
        // Cleanup overlay if it exists
        for entity in &existing_overlay {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Ensure overlay exists
    if existing_overlay.is_empty() {
        spawn_debug_info_overlay(&mut commands);
        return;
    }

    // Update text
    if let (Some((transform, health)), Ok(mut text)) =
        (player_query.iter().next(), overlay_query.single_mut())
    {
        let pos = transform.translation;
        **text = format!(
            "Pos: ({:.0}, {:.0})\nHP: {:.0}/{:.0}\nSeed: {}\nSegment: {}\nState: {:?}\nInvincible: {}",
            pos.x,
            pos.y,
            health.current,
            health.max,
            run_config.seed,
            run_config.segment_index,
            run_state.get(),
            debug_state.invincible
        );
    }
}
