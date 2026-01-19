use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::{
    AttackState, AttackTuning, Combatant, ComboState, Health, Invulnerable, ParryState,
    PlayerMoveset, SkillSlots, Stagger, Team, Weapon,
};
use crate::content::ContentRegistry;
use crate::core::{GameState, SelectedCharacter, gameplay_active};
use crate::encounters::EncounterBuffs;
use crate::rewards::{ActiveSkills, BaseStats, MovementFlags, PlayerBuild};

// ============================================================================
// Physics Layers
// ============================================================================

/// Physics layers for collision filtering
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    /// Ground surfaces (floors, platforms)
    Ground,
    /// Wall surfaces
    Wall,
    /// Player character
    Player,
    /// Enemy characters
    Enemy,
    /// Sensors (portals, triggers) - should not block movement
    Sensor,
    /// Player hitboxes (damage enemies)
    PlayerHitbox,
    /// Enemy hitboxes (damage player)
    EnemyHitbox,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct MovementState {
    pub on_ground: bool,
    pub on_wall: WallContact,
    pub facing: Facing,
    pub coyote_timer: f32,
    pub wall_coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub dash_timer: f32,
    pub dash_cooldown_timer: f32,
    pub is_dashing: bool,
    pub dash_direction: f32,
    pub air_jumps_remaining: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallContact {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Resource, Debug, Clone)]
pub struct MovementTuning {
    pub max_speed: f32,
    pub accel: f32,
    pub decel: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
    pub coyote_time: f32,
    pub wall_coyote_time: f32,
    pub jump_buffer_time: f32,
    /// Maximum air jumps (0 = no double jump, 1 = double jump, 2 = triple, etc.)
    pub max_air_jumps: u8,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_cooldown: f32,
    pub ground_only_dash: bool,
    pub wall_slide_speed: f32,
    pub wall_jump_horizontal: f32,
    pub wall_jump_vertical: f32,
    pub wall_jump_lock_time: f32,
}

impl Default for MovementTuning {
    fn default() -> Self {
        Self {
            max_speed: 320.0,
            accel: 3000.0,
            decel: 2600.0,
            jump_velocity: 680.0,
            gravity: 1800.0,
            coyote_time: 0.12,
            wall_coyote_time: 0.08,
            jump_buffer_time: 0.12,
            max_air_jumps: 0, // No double jump by default
            dash_speed: 900.0,
            dash_time: 0.16,
            dash_cooldown: 0.35,
            ground_only_dash: true,
            wall_slide_speed: 100.0,
            wall_jump_horizontal: 400.0,
            wall_jump_vertical: 600.0,
            wall_jump_lock_time: 0.15,
        }
    }
}

impl MovementTuning {
    /// Calculate the maximum height reachable from a single ground jump.
    /// Uses physics formula: h = v² / (2g)
    pub fn single_jump_height(&self) -> f32 {
        self.jump_velocity * self.jump_velocity / (2.0 * self.gravity)
    }

    /// Calculate the maximum height reachable with all available jumps.
    /// Each air jump adds additional height (assuming optimal timing at apex).
    /// This is a conservative estimate - actual height may vary with timing.
    pub fn max_reachable_height(&self) -> f32 {
        let base_height = self.single_jump_height();
        // Air jumps from apex add full jump height each
        // (1 ground jump + max_air_jumps air jumps)
        base_height * (1.0 + self.max_air_jumps as f32)
    }

    /// Calculate max reachable height with a safety margin for comfortable platforming.
    /// The margin accounts for imperfect jump timing and player collision box.
    pub fn safe_reachable_height(&self) -> f32 {
        // Use 80% of theoretical max for safe/comfortable gameplay
        self.max_reachable_height() * 0.8
    }
}

#[derive(Resource, Debug, Default)]
pub struct MovementInput {
    pub axis: Vec2,
    pub jump_just_pressed: bool,
    pub jump_held: bool,
    pub dash_just_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

/// Marker for ground colliders
#[derive(Component, Debug)]
pub struct Ground;

/// Marker for wall colliders
#[derive(Component, Debug)]
pub struct Wall;

/// Timer to prevent immediate air control after wall jump
#[derive(Component, Debug, Default)]
pub struct WallJumpLock(pub f32);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementTuning>()
            .init_resource::<MovementInput>()
            // Spawn player when entering Run state (after character selection)
            .add_systems(OnEnter(GameState::Run), bootstrap_player_from_data)
            .add_systems(
                Update,
                (
                    read_input,
                    detect_ground,
                    detect_walls,
                    update_timers,
                    apply_horizontal_movement,
                    apply_jump,
                    apply_wall_slide,
                    apply_dash,
                    apply_gravity,
                    update_facing,
                )
                    .chain()
                    .run_if(gameplay_active),
            );
    }
}

/// Bootstrap player from ContentRegistry data based on selected character.
/// This system runs on entering GameState::Run after character selection.
fn bootstrap_player_from_data(
    mut commands: Commands,
    selected_character: Res<SelectedCharacter>,
    registry: Option<Res<ContentRegistry>>,
    existing_player: Query<Entity, With<Player>>,
    mut player_build: ResMut<PlayerBuild>,
    mut tuning: ResMut<MovementTuning>,
    mut attack_tuning: ResMut<AttackTuning>,
) {
    // Don't spawn if player already exists
    if !existing_player.is_empty() {
        info!("Player already exists, skipping spawn");
        return;
    }

    // Get selected character ID or use default
    let char_id = selected_character
        .character_id
        .clone()
        .unwrap_or_else(|| "character_ares_sword".to_string());

    // Try to load from registry
    let (stats, skills, movement_flags, weapon_id, weapon_category, moveset_id, parent_god_id) =
        if let Some(reg) = &registry {
            load_character_data(reg, &char_id, &mut attack_tuning)
        } else {
            // Fallback to defaults
            warn!("ContentRegistry not available, using default player stats");
            (
                BaseStats::default(),
                ActiveSkills::default(),
                MovementFlags {
                    wall_jump: true,
                    air_dash_unlocked: false,
                },
                "weapon_sword_ares_basic".to_string(),
                "sword".to_string(),
                "moveset_sword_basic".to_string(),
                "ares".to_string(),
            )
        };

    // Update PlayerBuild resource
    *player_build = PlayerBuild::from_character(
        &char_id,
        &parent_god_id,
        &weapon_id,
        &weapon_category,
        &moveset_id,
        stats.clone(),
        skills.clone(),
        movement_flags.clone(),
    );

    // Update movement tuning based on character
    if movement_flags.air_dash_unlocked {
        tuning.ground_only_dash = false;
    }
    if movement_flags.wall_jump {
        // Wall jump is enabled by default, no change needed
    }

    // Apply stat multipliers to movement tuning
    let base_max_speed = 320.0;
    let base_jump_velocity = 680.0;
    tuning.max_speed = base_max_speed * stats.move_speed_mult;
    tuning.jump_velocity = base_jump_velocity * stats.jump_height_mult;

    // Calculate effective air jumps (unlocked via movement flags or skill tree)
    let air_jumps = if movement_flags.air_dash_unlocked {
        1
    } else {
        0
    };
    tuning.max_air_jumps = air_jumps;

    info!(
        "Spawning player: char={}, weapon={}, moveset={}, health={}, atk={}, speed_mult={}, jump_mult={}",
        char_id,
        weapon_id,
        moveset_id,
        stats.max_health,
        stats.attack_power,
        stats.move_speed_mult,
        stats.jump_height_mult
    );

    // Determine player color based on parent god
    let player_color = match parent_god_id.as_str() {
        "ares" => Color::srgb(0.95, 0.85, 0.85),     // Reddish white
        "demeter" => Color::srgb(0.85, 0.95, 0.85),  // Greenish white
        "poseidon" => Color::srgb(0.85, 0.85, 0.95), // Bluish white
        "zeus" => Color::srgb(0.95, 0.93, 0.8),      // Golden white
        _ => Color::srgb(0.9, 0.9, 0.9),
    };

    // Spawn the player with data-driven stats
    commands.spawn((
        // Identity & Movement
        (
            Player,
            Combatant,
            Team::Player,
            MovementState {
                air_jumps_remaining: tuning.max_air_jumps,
                ..default()
            },
            WallJumpLock::default(),
        ),
        // Combat
        (
            Health::new(stats.max_health),
            Stagger::default(),
            Invulnerable::default(),
            Weapon {
                damage_multiplier: stats.attack_power / 10.0, // Normalize around base 10
                knockback_multiplier: 1.0,
                speed_multiplier: 1.0,
            },
            AttackState::default(),
            SkillSlots {
                passive: skills.passive_id.clone(),
                common: skills.common_id.clone(),
                heavy: skills.ultimate_id.clone(),
            },
            // Data-driven moveset system (M3)
            PlayerMoveset {
                moveset_id: moveset_id.clone(),
            },
            ComboState::default(),
            ParryState::new(0.2), // Default parry window, will be updated from moveset
            EncounterBuffs::default(), // M6: Track active encounter buffs
        ),
        // Rendering
        Sprite {
            color: player_color,
            custom_size: Some(Vec2::new(24.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 0.0),
        // Physics
        (
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 48.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            GravityScale(0.0), // We handle gravity manually for more control
            Friction::new(0.0),
            CollisionEventsEnabled,
            CollisionLayers::new(
                GameLayer::Player,
                [
                    GameLayer::Ground,
                    GameLayer::Wall,
                    GameLayer::EnemyHitbox,
                    GameLayer::Sensor,
                ],
            ),
        ),
    ));
}

/// Load character data from ContentRegistry and configure attack tuning
fn load_character_data(
    registry: &ContentRegistry,
    char_id: &str,
    attack_tuning: &mut AttackTuning,
) -> (
    BaseStats,
    ActiveSkills,
    MovementFlags,
    String,
    String,
    String,
    String,
) {
    // Get character definition
    let char_def = registry.characters.get(char_id);

    if let Some(char_def) = char_def {
        // Build base stats from character data
        let stats = BaseStats {
            max_health: char_def.base_stats.max_health,
            stamina: 100.0, // Not in CharacterStatsDef yet
            attack_power: char_def.base_stats.attack_power,
            move_speed_mult: char_def.base_stats.move_speed_mult,
            jump_height_mult: char_def.base_stats.jump_height_mult,
        };

        // Build skills from starting skills
        let skills = ActiveSkills {
            passive_id: Some(char_def.starting_skills.passive.clone()),
            common_id: Some(char_def.starting_skills.common.clone()),
            ultimate_id: char_def.starting_skills.ultimate.clone(),
        };

        // Build movement flags
        let movement_flags = MovementFlags {
            wall_jump: char_def.movement_flags.wall_jump,
            air_dash_unlocked: char_def.movement_flags.air_dash_unlocked,
        };

        // Get weapon data
        let weapon_id = char_def.starting_weapon_id.clone();
        let parent_god_id = char_def.parent_god_id.clone();

        // Look up weapon to get category
        let (weapon_category, moveset_id) =
            if let Some(weapon_def) = registry.weapon_items.get(&weapon_id) {
                let category_id = weapon_def.category_id.clone();

                // Look up category to get default moveset
                let moveset = if let Some(cat_def) = registry.weapon_categories.get(&category_id) {
                    cat_def.default_moveset_id.clone()
                } else {
                    "moveset_sword_basic".to_string()
                };

                (category_id, moveset)
            } else {
                ("sword".to_string(), "moveset_sword_basic".to_string())
            };

        // Apply moveset data to attack tuning
        if let Some(moveset_def) = registry.movesets.get(&moveset_id) {
            apply_moveset_to_attack_tuning(moveset_def, attack_tuning);
        }

        info!(
            "Loaded character '{}': god={}, weapon={}, moveset={}",
            char_def.name, parent_god_id, weapon_id, moveset_id
        );

        (
            stats,
            skills,
            movement_flags,
            weapon_id,
            weapon_category,
            moveset_id,
            parent_god_id,
        )
    } else {
        warn!(
            "Character '{}' not found in registry, using defaults",
            char_id
        );
        (
            BaseStats::default(),
            ActiveSkills::default(),
            MovementFlags {
                wall_jump: true,
                air_dash_unlocked: false,
            },
            "weapon_sword_ares_basic".to_string(),
            "sword".to_string(),
            "moveset_sword_basic".to_string(),
            "ares".to_string(),
        )
    }
}

/// Apply moveset data to AttackTuning resource
fn apply_moveset_to_attack_tuning(
    moveset: &crate::content::MovesetDef,
    attack_tuning: &mut AttackTuning,
) {
    // Apply light attack from first strike in combo
    if let Some(first_light) = moveset.light_combo.strikes.first() {
        attack_tuning.light.damage = first_light.damage;
        attack_tuning.light.cooldown = first_light.cooldown;
        attack_tuning.light.hitbox_length = first_light.hitbox.length;
        attack_tuning.light.hitbox_width = first_light.hitbox.width;
        attack_tuning.light.hitbox_offset = first_light.hitbox.offset;
    }

    // Apply heavy attack from first strike in combo
    if let Some(first_heavy) = moveset.heavy_combo.strikes.first() {
        attack_tuning.heavy.damage = first_heavy.damage;
        attack_tuning.heavy.cooldown = first_heavy.cooldown;
        attack_tuning.heavy.hitbox_length = first_heavy.hitbox.length;
        attack_tuning.heavy.hitbox_width = first_heavy.hitbox.width;
        attack_tuning.heavy.hitbox_offset = first_heavy.hitbox.offset;
    }

    // Apply special attack
    attack_tuning.special.damage = moveset.special.damage;
    attack_tuning.special.cooldown = moveset.special.cooldown;
    attack_tuning.special.hitbox_length = moveset.special.hitbox.length;
    attack_tuning.special.hitbox_width = moveset.special.hitbox.width;
    attack_tuning.special.hitbox_offset = moveset.special.hitbox.offset;

    info!(
        "Applied moveset '{}': light_dmg={}, heavy_dmg={}, special_dmg={}",
        moveset.name,
        attack_tuning.light.damage,
        attack_tuning.heavy.damage,
        attack_tuning.special.damage
    );
}

/// Old spawn_player function kept for reference/fallback
#[allow(dead_code)]
fn spawn_player_legacy(mut commands: Commands, tuning: Res<MovementTuning>) {
    commands.spawn((
        // Identity & Movement
        (
            Player,
            Combatant,
            Team::Player,
            MovementState {
                air_jumps_remaining: tuning.max_air_jumps,
                ..default()
            },
            WallJumpLock::default(),
        ),
        // Combat
        (
            Health::new(100.0),
            Stagger::default(),
            Invulnerable::default(),
            Weapon::default(),
            AttackState::default(),
            SkillSlots::default(),
            PlayerMoveset::default(),
            ComboState::default(),
            ParryState::new(0.2),
        ),
        // Rendering
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(24.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 0.0),
        // Physics
        (
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 48.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            GravityScale(0.0), // We handle gravity manually for more control
            Friction::new(0.0),
            CollisionEventsEnabled,
            CollisionLayers::new(
                GameLayer::Player,
                [
                    GameLayer::Ground,
                    GameLayer::Wall,
                    GameLayer::EnemyHitbox,
                    GameLayer::Sensor,
                ],
            ),
        ),
    ));
}

fn spawn_test_room(mut commands: Commands) {
    let wall_color = Color::srgb(0.3, 0.3, 0.4);
    let ground_color = Color::srgb(0.4, 0.5, 0.4);
    let platform_color = Color::srgb(0.5, 0.4, 0.3);

    let ground_layers =
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Ground
    commands.spawn((
        Ground,
        Sprite {
            color: ground_color,
            custom_size: Some(Vec2::new(800.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(800.0, 40.0),
        ground_layers,
    ));

    // Left wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 500.0)),
            ..default()
        },
        Transform::from_xyz(-420.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 500.0),
        wall_layers,
    ));

    // Right wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 500.0)),
            ..default()
        },
        Transform::from_xyz(420.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 500.0),
        wall_layers,
    ));

    // Platform 1 - left side
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(150.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-250.0, -50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(150.0, 20.0),
        ground_layers,
    ));

    // Platform 2 - right side, higher
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(150.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(250.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(150.0, 20.0),
        ground_layers,
    ));

    // Platform 3 - center, highest
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(120.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 150.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(120.0, 20.0),
        ground_layers,
    ));

    // Small pillar for wall jumping practice
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(30.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(-100.0, -80.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(30.0, 200.0),
        wall_layers,
    ));
}

fn read_input(keyboard: Res<ButtonInput<KeyCode>>, mut input: ResMut<MovementInput>) {
    // Horizontal axis
    let mut x = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        x += 1.0;
    }

    // Vertical axis (for wall cling direction, etc.)
    let mut y = 0.0;
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        y += 1.0;
    }

    input.axis = Vec2::new(x, y);
    input.jump_just_pressed =
        keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyK);
    input.jump_held = keyboard.pressed(KeyCode::Space) || keyboard.pressed(KeyCode::KeyK);
    input.dash_just_pressed =
        keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::KeyJ);
}

fn detect_ground(
    spatial_query: SpatialQuery,
    tuning: Res<MovementTuning>,
    mut query: Query<(&Transform, &Collider, &mut MovementState), With<Player>>,
) {
    // Filter to only hit Ground layer entities (not enemies, portals, etc.)
    let ground_filter = SpatialQueryFilter::from_mask(GameLayer::Ground);

    for (transform, collider, mut state) in &mut query {
        let was_on_ground = state.on_ground;

        // Cast a short ray downward from the player's feet
        let player_half_height = match collider.shape_scaled().as_cuboid() {
            Some(c) => c.half_extents.y,
            None => 24.0,
        };

        let ray_origin = transform.translation.truncate() - Vec2::new(0.0, player_half_height);
        let ray_direction = Dir2::NEG_Y;
        let ray_distance = 4.0;

        let hit = spatial_query.cast_ray(
            ray_origin,
            ray_direction,
            ray_distance,
            true,
            &ground_filter,
        );

        state.on_ground = hit.is_some();

        // Reset coyote timer and air jumps when landing
        if state.on_ground && !was_on_ground {
            state.coyote_timer = 0.0;
            state.air_jumps_remaining = tuning.max_air_jumps;
            debug!(
                "Landed: on_ground={}, air_jumps_remaining={}",
                state.on_ground, state.air_jumps_remaining
            );
        } else if !state.on_ground && was_on_ground {
            debug!(
                "Left ground: on_ground={}, air_jumps_remaining={}",
                state.on_ground, state.air_jumps_remaining
            );
        }
    }
}

fn detect_walls(
    spatial_query: SpatialQuery,
    mut query: Query<(&Transform, &Collider, &mut MovementState), With<Player>>,
) {
    // Filter to only hit Wall layer entities
    let wall_filter = SpatialQueryFilter::from_mask(GameLayer::Wall);

    for (transform, collider, mut state) in &mut query {
        let was_on_wall = state.on_wall;

        let player_half_width = match collider.shape_scaled().as_cuboid() {
            Some(c) => c.half_extents.x,
            None => 12.0,
        };

        let origin = transform.translation.truncate();
        let ray_distance = 4.0;

        // Check left
        let left_hit = spatial_query.cast_ray(
            origin,
            Dir2::NEG_X,
            player_half_width + ray_distance,
            true,
            &wall_filter,
        );

        // Check right
        let right_hit = spatial_query.cast_ray(
            origin,
            Dir2::X,
            player_half_width + ray_distance,
            true,
            &wall_filter,
        );

        state.on_wall = match (left_hit.is_some(), right_hit.is_some()) {
            (true, false) => WallContact::Left,
            (false, true) => WallContact::Right,
            _ => WallContact::None,
        };

        // Reset wall coyote timer when touching wall
        if state.on_wall != WallContact::None && was_on_wall == WallContact::None {
            state.wall_coyote_timer = 0.0;
        }
    }
}

fn update_timers(
    time: Res<Time>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut WallJumpLock), With<Player>>,
) {
    let dt = time.delta_secs();

    for (mut state, mut wall_lock) in &mut query {
        // Coyote time: starts counting when leaving ground
        if !state.on_ground {
            state.coyote_timer += dt;
        }

        // Wall coyote time
        if state.on_wall == WallContact::None {
            state.wall_coyote_timer += dt;
        }

        // Jump buffer: counts down after pressing jump
        if state.jump_buffer_timer > 0.0 {
            state.jump_buffer_timer -= dt;
        }

        // Dash timer
        if state.is_dashing {
            state.dash_timer -= dt;
            if state.dash_timer <= 0.0 {
                state.is_dashing = false;
            }
        }

        // Dash cooldown
        if state.dash_cooldown_timer > 0.0 {
            state.dash_cooldown_timer -= dt;
        }

        // Wall jump lock
        if wall_lock.0 > 0.0 {
            wall_lock.0 -= dt;
        }

        // Reset dash cooldown when landing
        if state.on_ground && state.dash_cooldown_timer > 0.0 {
            state.dash_cooldown_timer = state.dash_cooldown_timer.min(tuning.dash_cooldown * 0.5);
        }
    }
}

fn apply_horizontal_movement(
    time: Res<Time>,
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&MovementState, &WallJumpLock, &mut LinearVelocity), With<Player>>,
) {
    let dt = time.delta_secs();

    for (state, wall_lock, mut velocity) in &mut query {
        // Skip horizontal control during dash or wall jump lock
        if state.is_dashing || wall_lock.0 > 0.0 {
            continue;
        }

        let target_vx = input.axis.x * tuning.max_speed;

        if input.axis.x.abs() > 0.1 {
            // Accelerate toward target
            let accel = tuning.accel * dt;
            if velocity.x < target_vx {
                velocity.x = (velocity.x + accel).min(target_vx);
            } else {
                velocity.x = (velocity.x - accel).max(target_vx);
            }
        } else {
            // Decelerate to zero
            let decel = tuning.decel * dt;
            if velocity.x > 0.0 {
                velocity.x = (velocity.x - decel).max(0.0);
            } else {
                velocity.x = (velocity.x + decel).min(0.0);
            }
        }
    }
}

fn apply_jump(
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut WallJumpLock, &mut LinearVelocity), With<Player>>,
) {
    for (mut state, mut wall_lock, mut velocity) in &mut query {
        // Skip during dash
        if state.is_dashing {
            continue;
        }

        // Buffer jump input
        if input.jump_just_pressed {
            state.jump_buffer_timer = tuning.jump_buffer_time;
        }

        let wants_jump = state.jump_buffer_timer > 0.0;
        let can_ground_jump = state.on_ground || state.coyote_timer < tuning.coyote_time;
        let can_wall_jump = !state.on_ground
            && (state.on_wall != WallContact::None
                || state.wall_coyote_timer < tuning.wall_coyote_time);

        let can_air_jump = !state.on_ground && state.air_jumps_remaining > 0;

        if wants_jump {
            if can_ground_jump {
                // Ground jump
                velocity.y = tuning.jump_velocity;
                state.jump_buffer_timer = 0.0;
                state.coyote_timer = tuning.coyote_time; // Consume coyote time
                debug!(
                    "Ground jump: on_ground={}, coyote consumed, air_jumps_remaining={}",
                    state.on_ground, state.air_jumps_remaining
                );
            } else if can_wall_jump {
                // Wall jump
                let wall_dir = if state.on_wall == WallContact::Left {
                    1.0
                } else if state.on_wall == WallContact::Right {
                    -1.0
                } else {
                    // Using wall coyote - use facing direction
                    match state.facing {
                        Facing::Left => 1.0,
                        Facing::Right => -1.0,
                    }
                };

                velocity.x = wall_dir * tuning.wall_jump_horizontal;
                velocity.y = tuning.wall_jump_vertical;
                wall_lock.0 = tuning.wall_jump_lock_time;
                state.jump_buffer_timer = 0.0;
                state.wall_coyote_timer = tuning.wall_coyote_time;
                // Wall jump also resets air jumps
                state.air_jumps_remaining = tuning.max_air_jumps;
                debug!(
                    "Wall jump: on_wall={:?}, air_jumps reset to {}",
                    state.on_wall, state.air_jumps_remaining
                );
            } else if can_air_jump {
                // Air jump (double jump, triple jump, etc.)
                velocity.y = tuning.jump_velocity;
                state.jump_buffer_timer = 0.0;
                state.air_jumps_remaining -= 1;
                debug!(
                    "Air jump: air_jumps_remaining now {}",
                    state.air_jumps_remaining
                );
            }
        }

        // Variable jump height - cut velocity when releasing jump
        if !input.jump_held && velocity.y > 0.0 && !state.on_ground {
            velocity.y *= 0.5;
        }
    }
}

fn apply_wall_slide(
    tuning: Res<MovementTuning>,
    input: Res<MovementInput>,
    mut query: Query<(&MovementState, &mut LinearVelocity), With<Player>>,
) {
    for (state, mut velocity) in &mut query {
        // Only wall slide when in the air, touching a wall, and holding toward the wall
        if state.on_ground || state.on_wall == WallContact::None || state.is_dashing {
            continue;
        }

        let holding_toward_wall = match state.on_wall {
            WallContact::Left => input.axis.x < -0.1,
            WallContact::Right => input.axis.x > 0.1,
            WallContact::None => false,
        };

        if holding_toward_wall && velocity.y < 0.0 {
            // Clamp fall speed to wall slide speed
            velocity.y = velocity.y.max(-tuning.wall_slide_speed);
        }
    }
}

fn apply_dash(
    input: Res<MovementInput>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&mut MovementState, &mut LinearVelocity), With<Player>>,
) {
    for (mut state, mut velocity) in &mut query {
        // Initiate dash
        if input.dash_just_pressed && state.dash_cooldown_timer <= 0.0 && !state.is_dashing {
            // Ground-only dash check
            if tuning.ground_only_dash && !state.on_ground {
                continue;
            }

            state.is_dashing = true;
            state.dash_timer = tuning.dash_time;
            state.dash_cooldown_timer = tuning.dash_cooldown;

            // Dash in facing direction (or input direction if provided)
            state.dash_direction = if input.axis.x.abs() > 0.1 {
                input.axis.x.signum()
            } else {
                match state.facing {
                    Facing::Right => 1.0,
                    Facing::Left => -1.0,
                }
            };
        }

        // Apply dash velocity
        if state.is_dashing {
            velocity.x = state.dash_direction * tuning.dash_speed;
            velocity.y = 0.0; // Lock vertical movement during dash
        }
    }
}

fn apply_gravity(
    time: Res<Time>,
    tuning: Res<MovementTuning>,
    mut query: Query<(&MovementState, &mut LinearVelocity), With<Player>>,
) {
    let dt = time.delta_secs();

    for (state, mut velocity) in &mut query {
        // No gravity during dash
        if state.is_dashing {
            continue;
        }

        velocity.y -= tuning.gravity * dt;
    }
}

fn update_facing(
    input: Res<MovementInput>,
    mut query: Query<(&mut MovementState, &WallJumpLock), With<Player>>,
) {
    for (mut state, wall_lock) in &mut query {
        // Don't update facing during wall jump lock or dash
        if wall_lock.0 > 0.0 || state.is_dashing {
            continue;
        }

        if input.axis.x > 0.1 {
            state.facing = Facing::Right;
        } else if input.axis.x < -0.1 {
            state.facing = Facing::Left;
        }
    }
}
