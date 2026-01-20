//! Movement domain: player bootstrap and data-driven movement setup.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::{
    AttackState, AttackTuning, Combatant, ComboState, Health, Invulnerable, ParryState,
    PlayerMoveset, SkillSlots, Stagger, Team, Weapon,
};
use crate::content::ContentRegistry;
use crate::core::SelectedCharacter;
use crate::encounters::EncounterBuffs;
use crate::movement::{GameLayer, MovementState, Player, WallJumpLock};
use crate::rewards::{ActiveSkills, BaseStats, MovementFlags, PlayerBuild};

/// Bootstrap player from ContentRegistry data based on selected character.
/// This system runs on entering GameState::Run after character selection.
pub(crate) fn bootstrap_player_from_data(
    mut commands: Commands,
    selected_character: Res<SelectedCharacter>,
    registry: Option<Res<ContentRegistry>>,
    existing_player: Query<Entity, With<Player>>,
    mut player_build: ResMut<PlayerBuild>,
    mut tuning: ResMut<crate::movement::MovementTuning>,
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
