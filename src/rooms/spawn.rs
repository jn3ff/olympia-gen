//! Rooms domain: room, arena, and enemy spawning helpers.

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::combat::{
    ArenaLock, BossAttackSlots, BossConfig, BossEncounterState, EnemyBundle, EnemyTier,
    EnemyTuning, spawn_boss_scaled,
};
use crate::content::{Direction, GameplayDefaults};
use crate::core::{DifficultyScaling, RunConfig, SegmentProgress};
use crate::movement::{GameLayer, Ground, MovementTuning, Player, Wall};
use crate::rooms::components::{
    ArenaHub, ArenaPortal, BlockedPortal, ExitTrigger, PortalBarrier, PortalDisabled,
    PortalEnabled, PortalExitAnimation, PortalFloor, RoomExit, RoomInstance, ShopInteractionZone,
    ShopNPC,
};
use crate::rooms::data::{PortalCondition, PortalEnableCondition, RoomData};
use crate::rooms::graph::{PlayerInPortalZone, RoomGraph};
use crate::rooms::registry::RoomRegistry;
use crate::rooms::ui::{ShopNameLabel, spawn_segment_info_ui};

pub(crate) fn spawn_arena_hub(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    run_config: Res<RunConfig>,
    segment_progress: Res<SegmentProgress>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    mut room_graph: ResMut<RoomGraph>,
) {
    // Reset rooms cleared for the new segment
    room_graph.rooms_cleared.clear();
    room_graph.current_room_id = None;
    room_graph.pending_transition = None;

    info!(
        "Entering Arena Hub - Segment {} (seed: {})",
        run_config.segment_index, run_config.seed
    );
    info!(
        "Segment progress: Rooms {}, Bosses {}, Total Bosses {}",
        segment_progress.rooms_cleared_this_segment,
        segment_progress.bosses_defeated_this_segment,
        segment_progress.total_bosses_defeated
    );
    let wall_color = Color::srgb(0.25, 0.25, 0.35);
    let ground_color = Color::srgb(0.35, 0.4, 0.35);
    let portal_color = Color::srgb(0.4, 0.6, 0.9);

    let ground_layers =
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Spawn arena container
    commands.spawn((ArenaHub, Transform::default(), Visibility::default()));

    // Ground platform
    commands.spawn((
        Ground,
        Sprite {
            color: ground_color,
            custom_size: Some(Vec2::new(600.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -150.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(600.0, 40.0),
        ground_layers,
    ));

    // Left wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 400.0)),
            ..default()
        },
        Transform::from_xyz(-320.0, 30.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 400.0),
        wall_layers,
    ));

    // Right wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 400.0)),
            ..default()
        },
        Transform::from_xyz(320.0, 30.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 400.0),
        wall_layers,
    ));

    // Arena portals (4 directions)
    let portal_positions = [
        (Direction::Up, Vec2::new(0.0, 150.0)),
        (Direction::Down, Vec2::new(0.0, -150.0)),
        (Direction::Left, Vec2::new(-250.0, -50.0)),
        (Direction::Right, Vec2::new(250.0, -50.0)),
    ];

    for (direction, pos) in portal_positions {
        let visual_size = match direction {
            Direction::Up | Direction::Down => Vec2::new(80.0, 30.0),
            Direction::Left | Direction::Right => Vec2::new(30.0, 80.0),
        };

        let collider_size = match direction {
            Direction::Up | Direction::Down => Vec2::new(80.0, 60.0),
            Direction::Left | Direction::Right => Vec2::new(60.0, 80.0),
        };

        // Portal visual
        let mut portal_cmd = commands.spawn((
            ArenaPortal { direction },
            Sprite {
                color: portal_color,
                custom_size: Some(visual_size),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
        ));

        // Enable all portals by default (can add logic later)
        portal_cmd.insert(PortalEnabled);

        // Portal sensor
        commands.spawn((
            ArenaPortal { direction },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Collider::rectangle(collider_size.x, collider_size.y),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
    }

    // Reset player position if exists - spawn at center, elevated above ground
    // This keeps player away from portals on spawn
    for entity in player_query.iter() {
        commands
            .entity(entity)
            .insert(Transform::from_xyz(0.0, 50.0, 0.0))
            .remove::<PlayerInPortalZone>(); // Clear any stale portal zone state
    }

    // Spawn segment info UI with progress
    spawn_segment_info_ui(
        &mut commands,
        run_config.segment_index,
        &segment_progress,
        gameplay_defaults.as_deref(),
    );

    // Spawn shop NPCs
    spawn_shop_npcs(&mut commands);
}

pub(crate) fn spawn_shop_npcs(commands: &mut Commands) {
    let shop_configs = [
        (
            "shop_armory",
            "Armory",
            Vec2::new(-150.0, -80.0),
            Color::srgb(0.6, 0.4, 0.2),
        ),
        (
            "shop_blacksmith",
            "Blacksmith",
            Vec2::new(0.0, -80.0),
            Color::srgb(0.4, 0.4, 0.5),
        ),
        (
            "shop_enchanter",
            "Enchanter",
            Vec2::new(150.0, -80.0),
            Color::srgb(0.5, 0.3, 0.7),
        ),
    ];

    for (shop_id, name, pos, color) in shop_configs {
        // Shop NPC visual (rectangle)
        commands.spawn((
            ShopNPC {
                shop_id: shop_id.to_string(),
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(30.0, 50.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y + 25.0, 1.0),
        ));

        // Shop name label
        commands.spawn((
            ShopNameLabel,
            Text2d::new(name),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Transform::from_xyz(pos.x, pos.y + 60.0, 2.0),
        ));

        // Shop interaction zone (sensor)
        commands.spawn((
            ShopInteractionZone {
                shop_id: shop_id.to_string(),
            },
            Transform::from_xyz(pos.x, pos.y + 25.0, 0.0),
            Collider::rectangle(60.0, 70.0),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
    }
}

pub(crate) fn spawn_current_room(
    mut commands: Commands,
    room_graph: Res<RoomGraph>,
    registry: Res<RoomRegistry>,
    boss_config: Res<BossConfig>,
    mut boss_state: ResMut<BossEncounterState>,
    enemy_tuning: Res<EnemyTuning>,
    movement_tuning: Res<MovementTuning>,
    run_config: Res<RunConfig>,
    difficulty: Res<DifficultyScaling>,
    segment_progress: Res<SegmentProgress>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Some(transition) = &room_graph.pending_transition else {
        // No transition pending, spawn default room
        spawn_room_geometry(
            &mut commands,
            &RoomData::default(),
            None,
            &movement_tuning,
            None,
        );
        return;
    };

    // Find room data
    let room_data = registry
        .rooms
        .iter()
        .find(|r| r.id == transition.to_room)
        .cloned()
        .unwrap_or_default();

    // Determine if this room should have a boss
    let mut rng = rand::rng();
    let rng_roll: f32 = rng.random();
    let spawn_boss_encounter =
        room_data.boss_room || boss_state.should_spawn_boss(&boss_config, rng_roll);

    // Get blocked direction from active sequence (the entry portal should be blocked)
    let blocked_direction = room_graph
        .active_sequence
        .as_ref()
        .map(|seq| seq.blocked_exit_direction());

    // Spawn room geometry
    spawn_room_geometry(
        &mut commands,
        &room_data,
        Some(transition.entry_direction),
        &movement_tuning,
        blocked_direction,
    );

    // Spawn enemies or boss with difficulty scaling
    if spawn_boss_encounter {
        spawn_boss_room_enemies(&mut commands, &room_data, &run_config, &difficulty);
    } else {
        spawn_room_enemies(
            &mut commands,
            &room_data,
            &enemy_tuning,
            &run_config,
            &difficulty,
            &segment_progress,
            &mut rng,
        );
        boss_state.room_cleared();
    }

    // Position player at spawn point based on entry direction
    let spawn_pos = get_spawn_position(&room_data, transition.entry_direction);
    for mut transform in &mut player_query {
        transform.translation = spawn_pos.extend(0.0);
    }

    info!(
        "Spawned room '{}' at segment {} (health_mult: {:.2}x, damage_mult: {:.2}x)",
        room_data.id,
        run_config.segment_index,
        difficulty.enemy_health_multiplier(run_config.segment_index),
        difficulty.enemy_damage_multiplier(run_config.segment_index)
    );
}

fn spawn_room_enemies(
    commands: &mut Commands,
    room: &RoomData,
    tuning: &EnemyTuning,
    run_config: &RunConfig,
    difficulty: &DifficultyScaling,
    segment_progress: &SegmentProgress,
    rng: &mut impl Rng,
) {
    let half_width = room.width / 2.0 - 80.0;
    let half_height = room.height / 2.0 - 80.0;

    // Determine number of enemies based on room size and segment
    let base_enemies = 2 + (room.width * room.height / 200000.0) as usize;
    let bonus_enemies = difficulty.bonus_enemy_count(run_config.segment_index);
    let num_enemies = (base_enemies + bonus_enemies).min(8); // Cap at 8 enemies

    // Calculate difficulty multipliers
    let health_mult = difficulty.enemy_health_multiplier(run_config.segment_index);
    let _damage_mult = difficulty.enemy_damage_multiplier(run_config.segment_index);

    // Define available enemy types with their base stats
    // In a full implementation, these would come from ContentRegistry
    let enemy_pool = [
        ("enemy_grunt", EnemyTier::Minor, 50.0),
        ("enemy_warrior", EnemyTier::Major, 50.0),
        ("enemy_elite", EnemyTier::Special, 50.0),
    ];

    for _i in 0..num_enemies {
        // Random position within room bounds
        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height * 0.5); // Bias toward lower half

        // Random tier with weighted probabilities (higher segments have tougher enemies)
        let tier_bonus = (run_config.segment_index as f32 * 0.05).min(0.3);
        let tier_roll: f32 = rng.random();
        let tier = if tier_roll < 0.6 - tier_bonus {
            EnemyTier::Minor
        } else if tier_roll < 0.85 - tier_bonus * 0.5 {
            EnemyTier::Major
        } else {
            EnemyTier::Special
        };

        // Find a matching enemy from the pool
        let (def_id, _, base_hp) = enemy_pool
            .iter()
            .find(|(_, t, _)| *t == tier)
            .unwrap_or(&enemy_pool[0]);

        // Check if this enemy type is significant and already encountered
        let effective_health = base_hp * health_mult * tier.stat_multipliers().0;
        let would_be_significant = effective_health / crate::combat::BASELINE_DPS
            > crate::combat::SIGNIFICANT_THRESHOLD_SECONDS;

        if would_be_significant
            && segment_progress
                .encountered_significant_enemies
                .contains(*def_id)
        {
            // Skip this significant enemy - they don't repeat within a run
            info!(
                "Skipping significant enemy '{}' - already encountered this run",
                def_id
            );
            continue;
        }

        // Base health scaled by difficulty
        let base_health = base_hp * health_mult;
        commands.spawn(EnemyBundle::new(
            tier,
            Vec2::new(x, y),
            base_health,
            tuning,
            *def_id,
        ));
    }
}

fn spawn_boss_room_enemies(
    commands: &mut Commands,
    _room: &RoomData,
    run_config: &RunConfig,
    difficulty: &DifficultyScaling,
) {
    // Spawn the boss in the center of the room
    let boss_pos = Vec2::new(0.0, 0.0);

    // Calculate boss scaling based on segment
    let health_mult = difficulty.boss_health_multiplier(run_config.segment_index);
    let damage_mult = difficulty.boss_damage_multiplier(run_config.segment_index);

    spawn_boss_scaled(
        commands,
        boss_pos,
        100.0, // Base health (will be multiplied by tier AND difficulty)
        BossAttackSlots::default(),
        health_mult,
        damage_mult,
    );

    info!(
        "Spawned boss at segment {} (health_mult: {:.2}x, damage_mult: {:.2}x)",
        run_config.segment_index, health_mult, damage_mult
    );

    // Lock the arena exits during boss fight
    commands.spawn((ArenaLock, Transform::default(), Visibility::default()));
}

fn spawn_room_geometry(
    commands: &mut Commands,
    room: &RoomData,
    entry_direction: Option<Direction>,
    movement_tuning: &MovementTuning,
    blocked_direction: Option<Direction>,
) {
    let wall_color = Color::srgb(0.3, 0.3, 0.4);
    let ground_color = Color::srgb(0.4, 0.5, 0.4);
    let exit_enabled_color = Color::srgb(0.3, 0.7, 0.4);
    let exit_disabled_color = Color::srgb(0.5, 0.5, 0.5);
    let exit_blocked_color = Color::srgb(0.25, 0.25, 0.25); // Dark gray for blocked
    let boss_exit_enabled_color = Color::srgb(0.7, 0.3, 0.3);
    let boss_exit_disabled_color = Color::srgb(0.4, 0.3, 0.3);

    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    let wall_thickness = 40.0;

    // Calculate safe reachable height for platform placement
    let safe_jump_height = movement_tuning.safe_reachable_height();

    let ground_layers =
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Room instance marker
    commands.spawn((
        RoomInstance {
            id: room.id.clone(),
            boss_room: room.boss_room,
        },
        Transform::default(),
        Visibility::default(),
    ));

    // Ground (with gap if Down exit exists)
    if room.exits.contains(&Direction::Down) {
        // Split ground with exit gap
        let gap_width = 100.0;
        let side_width = (room.width - gap_width) / 2.0;

        // Left ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(-half_width + side_width / 2.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            ground_layers,
        ));

        // Right ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(half_width - side_width / 2.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            ground_layers,
        ));
    } else {
        // Full ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(room.width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(room.width, wall_thickness),
            ground_layers,
        ));
    }

    // Ceiling (with gap if Up exit exists)
    if room.exits.contains(&Direction::Up) {
        // Split ceiling with exit gap
        let gap_width = 100.0;
        let side_width = (room.width - gap_width) / 2.0;

        // Left ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(-half_width + side_width / 2.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            wall_layers,
        ));

        // Right ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(half_width - side_width / 2.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            wall_layers,
        ));

        // Up exit trigger
        let is_blocked = blocked_direction == Some(Direction::Up);
        let condition = if is_blocked {
            PortalEnableCondition::Never
        } else {
            room.get_exit_condition(Direction::Up)
        };
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if is_blocked {
            exit_blocked_color
        } else if room.boss_room {
            if is_enabled {
                boss_exit_enabled_color
            } else {
                boss_exit_disabled_color
            }
        } else if is_enabled {
            exit_enabled_color
        } else {
            exit_disabled_color
        };

        // Solid floor platform below the Up exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let up_exit_platform_y = half_height - wall_thickness - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = up_exit_platform_y - ground_level;

        // Add stepping stone platforms if the exit is too high to reach
        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Alternate platforms left and right for interesting traversal
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                let step_x = if i % 2 == 1 { -80.0 } else { 80.0 };

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Main Up exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(gap_width + 40.0, platform_height)),
                ..default()
            },
            Transform::from_xyz(0.0, up_exit_platform_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width + 40.0, platform_height),
            ground_layers,
        ));

        // Solid barrier in the Up exit gap - always blocks passage
        // This is invisible but prevents the player from leaving through the gap
        commands.spawn((
            PortalBarrier {
                direction: Direction::Up,
            },
            Wall,
            Transform::from_xyz(0.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width, wall_thickness),
            wall_layers,
        ));

        // Exit sensor flush with wall - collider extends inward for player interaction
        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Up,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(gap_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, half_height, 0.5),
            Collider::rectangle(gap_width, wall_thickness + 40.0),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
        if is_blocked {
            exit_cmd.insert(BlockedPortal);
            exit_cmd.insert(PortalDisabled);
        } else if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }

        // If entry direction is up, animate portal exit color
        if entry_direction == Some(Direction::Up) {
            exit_cmd.insert(PortalExitAnimation::new(color, exit_disabled_color, 1.5));
        }
    } else {
        // Full ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(room.width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(room.width, wall_thickness),
            wall_layers,
        ));
    }

    // Left wall (with gap if Left exit exists)
    if room.exits.contains(&Direction::Left) {
        let gap_height = 100.0;
        let side_height = (room.height - gap_height) / 2.0;

        // Upper wall segment
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, half_height - side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Lower wall segment
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, -half_height + side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Left exit trigger
        let is_blocked = blocked_direction == Some(Direction::Left);
        let condition = if is_blocked {
            PortalEnableCondition::Never
        } else {
            room.get_exit_condition(Direction::Left)
        };
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if is_blocked {
            exit_blocked_color
        } else if room.boss_room {
            if is_enabled {
                boss_exit_enabled_color
            } else {
                boss_exit_disabled_color
            }
        } else if is_enabled {
            exit_enabled_color
        } else {
            exit_disabled_color
        };

        // Solid floor platform to the right of the Left exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let platform_width = wall_thickness + 60.0; // Extends into the room
        let left_exit_platform_y = -gap_height / 2.0 - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = left_exit_platform_y - ground_level;

        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Stagger steps toward the left exit
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                let step_x = -half_width + 100.0 + (i as f32 * 30.0);

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Left exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(platform_width, platform_height)),
                ..default()
            },
            Transform::from_xyz(
                -half_width + platform_width / 2.0,
                left_exit_platform_y,
                0.0,
            ),
            RigidBody::Static,
            Collider::rectangle(platform_width, platform_height),
            ground_layers,
        ));

        // Solid barrier in the Left exit gap - always blocks passage
        commands.spawn((
            PortalBarrier {
                direction: Direction::Left,
            },
            Wall,
            Transform::from_xyz(-half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, gap_height),
            wall_layers,
        ));

        // Exit sensor flush with wall - collider extends inward for player interaction
        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Left,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(wall_thickness, gap_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, 0.0, 0.5),
            Collider::rectangle(wall_thickness + 40.0, gap_height),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
        if is_blocked {
            exit_cmd.insert(BlockedPortal);
            exit_cmd.insert(PortalDisabled);
        } else if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }

        if entry_direction == Some(Direction::Left) {
            exit_cmd.insert(PortalExitAnimation::new(color, exit_disabled_color, 1.5));
        }
    } else {
        // Full left wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, room.height)),
                ..default()
            },
            Transform::from_xyz(-half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, room.height),
            wall_layers,
        ));
    }

    // Right wall (with gap if Right exit exists)
    if room.exits.contains(&Direction::Right) {
        let gap_height = 100.0;
        let side_height = (room.height - gap_height) / 2.0;

        // Upper wall segment
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(half_width, half_height - side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Lower wall segment
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(half_width, -half_height + side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Right exit trigger
        let is_blocked = blocked_direction == Some(Direction::Right);
        let condition = if is_blocked {
            PortalEnableCondition::Never
        } else {
            room.get_exit_condition(Direction::Right)
        };
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if is_blocked {
            exit_blocked_color
        } else if room.boss_room {
            if is_enabled {
                boss_exit_enabled_color
            } else {
                boss_exit_disabled_color
            }
        } else if is_enabled {
            exit_enabled_color
        } else {
            exit_disabled_color
        };

        // Solid floor platform to the left of the Right exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let platform_width = wall_thickness + 60.0; // Extends into the room
        let right_exit_platform_y = -gap_height / 2.0 - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = right_exit_platform_y - ground_level;

        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Stagger steps toward the right exit
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                let step_x = half_width - 100.0 - (i as f32 * 30.0);

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Right exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(platform_width, platform_height)),
                ..default()
            },
            Transform::from_xyz(
                half_width - platform_width / 2.0,
                right_exit_platform_y,
                0.0,
            ),
            RigidBody::Static,
            Collider::rectangle(platform_width, platform_height),
            ground_layers,
        ));

        // Solid barrier in the Right exit gap - always blocks passage
        commands.spawn((
            PortalBarrier {
                direction: Direction::Right,
            },
            Wall,
            Transform::from_xyz(half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, gap_height),
            wall_layers,
        ));

        // Exit sensor flush with wall - collider extends inward for player interaction
        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Right,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(wall_thickness, gap_height)),
                ..default()
            },
            Transform::from_xyz(half_width, 0.0, 0.5),
            Collider::rectangle(wall_thickness + 40.0, gap_height),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
        if is_blocked {
            exit_cmd.insert(BlockedPortal);
            exit_cmd.insert(PortalDisabled);
        } else if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }

        if entry_direction == Some(Direction::Right) {
            exit_cmd.insert(PortalExitAnimation::new(color, exit_disabled_color, 1.5));
        }
    } else {
        // Full right wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, room.height)),
                ..default()
            },
            Transform::from_xyz(half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, room.height),
            wall_layers,
        ));
    }

    // Bottom exit (Down)
    if room.exits.contains(&Direction::Down) {
        let gap_width = 100.0;
        let condition = room.get_exit_condition(Direction::Down);
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if room.boss_room {
            if is_enabled {
                boss_exit_enabled_color
            } else {
                boss_exit_disabled_color
            }
        } else if is_enabled {
            exit_enabled_color
        } else {
            exit_disabled_color
        };

        // Solid floor platform above the Down exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let down_exit_platform_y = -half_height + wall_thickness + platform_height / 2.0;

        // Down exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(gap_width + 40.0, platform_height)),
                ..default()
            },
            Transform::from_xyz(0.0, down_exit_platform_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width + 40.0, platform_height),
            ground_layers,
        ));

        // Solid barrier in the Down exit gap - always blocks passage
        commands.spawn((
            PortalBarrier {
                direction: Direction::Down,
            },
            Wall,
            Transform::from_xyz(0.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width, wall_thickness),
            wall_layers,
        ));

        // Exit sensor flush with wall - collider extends inward for player interaction
        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Down,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(gap_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, -half_height, 0.5),
            Collider::rectangle(gap_width, wall_thickness + 40.0),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player]),
        ));
        if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }

        if entry_direction == Some(Direction::Down) {
            exit_cmd.insert(PortalExitAnimation::new(color, exit_disabled_color, 1.5));
        }
    }

    // Add some platforms in larger rooms for vertical traversal
    if room.width > 800.0 || room.height > 500.0 {
        let platform_color = Color::srgb(0.5, 0.4, 0.3);
        let platforms = [
            Vec2::new(-200.0, -50.0),
            Vec2::new(200.0, 50.0),
            Vec2::new(0.0, 150.0),
        ];

        for pos in platforms {
            commands.spawn((
                Ground,
                Sprite {
                    color: platform_color,
                    custom_size: Some(Vec2::new(120.0, 20.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.0),
                RigidBody::Static,
                Collider::rectangle(120.0, 20.0),
                ground_layers,
            ));
        }
    }
}

fn get_spawn_position(room: &RoomData, entry_direction: Direction) -> Vec2 {
    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    let wall_offset = 80.0;
    let ground_offset = 80.0; // Height above ground level

    match entry_direction {
        // Spawn near left wall, on ground level (away from Left exit which is at y=0)
        Direction::Left => Vec2::new(-half_width + wall_offset, -half_height + ground_offset),
        // Spawn near right wall, on ground level (away from Right exit which is at y=0)
        Direction::Right => Vec2::new(half_width - wall_offset, -half_height + ground_offset),
        Direction::Up => Vec2::new(0.0, half_height - wall_offset),
        Direction::Down => Vec2::new(0.0, -half_height + ground_offset),
    }
}
