//! Animation state machine and playback.
//!
//! Handles animation states (idle, walk, attack) and frame progression
//! for layered sprites.

#![allow(dead_code)]

use bevy::ecs::message::Message;
use bevy::prelude::*;

#[allow(unused_imports)]
use super::{BodyLayer, SpriteManifest, WeaponLayer};

/// Animation states for characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
    Attack(AttackAnimationType),
    Stagger,
    Death,
}

/// Types of attack animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackAnimationType {
    Light,
    Heavy,
    Special,
}

/// Component for animation playback on a layered sprite.
#[derive(Component, Debug)]
pub struct AnimationController {
    /// Current animation state.
    pub state: AnimationState,
    /// Previous state (for detecting transitions).
    pub previous_state: AnimationState,
    /// Base name for animation sprites (e.g., "player.base.zeus").
    pub animation_base: String,
    /// Current frame index (0-based).
    pub current_frame: u32,
    /// Total frames in current animation.
    pub total_frames: u32,
    /// Time accumulator for frame timing.
    pub frame_timer: f32,
    /// Seconds per frame.
    pub frame_duration: f32,
    /// Whether the animation should loop.
    pub looping: bool,
    /// Whether the animation has finished (for non-looping).
    pub finished: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            animation_base: String::new(),
            current_frame: 0,
            total_frames: 4,
            frame_timer: 0.0,
            frame_duration: 0.15, // ~6-7 FPS for retro feel
            looping: true,
            finished: false,
        }
    }
}

impl AnimationController {
    /// Create a new controller with the given base animation name.
    pub fn new(animation_base: &str) -> Self {
        Self {
            animation_base: animation_base.to_string(),
            ..default()
        }
    }

    /// Set the animation state, resetting frame if state changed.
    pub fn set_state(&mut self, state: AnimationState) {
        if self.state != state {
            self.previous_state = self.state;
            self.state = state;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.finished = false;

            // Set looping based on state
            self.looping = matches!(
                state,
                AnimationState::Idle | AnimationState::Walk | AnimationState::Run
            );

            // Set frame count based on state (could be data-driven)
            self.total_frames = match state {
                AnimationState::Idle => 4,
                AnimationState::Walk => 4,
                AnimationState::Run => 6,
                AnimationState::Jump => 2,
                AnimationState::Fall => 2,
                AnimationState::Attack(_) => 3,
                AnimationState::Stagger => 2,
                AnimationState::Death => 4,
            };

            // Adjust frame duration for different animations
            self.frame_duration = match state {
                AnimationState::Attack(_) => 0.08, // Faster attacks
                AnimationState::Stagger => 0.1,
                _ => 0.15,
            };
        }
    }

    /// Get the current animation name suffix (e.g., "idle", "walk", "attack_1").
    pub fn animation_suffix(&self) -> String {
        match self.state {
            AnimationState::Idle => "idle".to_string(),
            AnimationState::Walk => "walk".to_string(),
            AnimationState::Run => "run".to_string(),
            AnimationState::Jump => "jump".to_string(),
            AnimationState::Fall => "fall".to_string(),
            AnimationState::Attack(t) => match t {
                AttackAnimationType::Light => "attack_light".to_string(),
                AttackAnimationType::Heavy => "attack_heavy".to_string(),
                AttackAnimationType::Special => "attack_special".to_string(),
            },
            AnimationState::Stagger => "stagger".to_string(),
            AnimationState::Death => "death".to_string(),
        }
    }

    /// Get the full sprite key for the current frame.
    pub fn current_sprite_key(&self) -> String {
        format!(
            "{}_{}_{}",
            self.animation_base,
            self.animation_suffix(),
            self.current_frame + 1 // 1-indexed for file naming
        )
    }

    /// Get the weapon pose for the current animation state.
    pub fn weapon_pose(&self) -> &'static str {
        match self.state {
            AnimationState::Idle | AnimationState::Walk | AnimationState::Run => "idle",
            AnimationState::Jump | AnimationState::Fall => "idle",
            AnimationState::Attack(_) => match self.current_frame {
                0 => "raised",
                1 => "swing_h",
                _ => "swing_down",
            },
            AnimationState::Stagger => "idle",
            AnimationState::Death => "idle",
        }
    }
}

/// Message fired when animation state changes.
#[derive(Debug)]
pub struct AnimationStateChanged {
    pub entity: Entity,
    pub from: AnimationState,
    pub to: AnimationState,
}

impl Message for AnimationStateChanged {}

/// Message fired when a non-looping animation completes.
#[derive(Debug)]
pub struct AnimationFinished {
    pub entity: Entity,
    pub state: AnimationState,
}

impl Message for AnimationFinished {}

/// System that updates animation frames based on time.
pub fn update_animation_frames(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationController)>,
    mut finished_events: MessageWriter<AnimationFinished>,
) {
    for (entity, mut controller) in &mut query {
        if controller.finished {
            continue;
        }

        controller.frame_timer += time.delta_secs();

        if controller.frame_timer >= controller.frame_duration {
            controller.frame_timer -= controller.frame_duration;
            controller.current_frame += 1;

            if controller.current_frame >= controller.total_frames {
                if controller.looping {
                    controller.current_frame = 0;
                } else {
                    controller.current_frame = controller.total_frames - 1;
                    controller.finished = true;
                    finished_events.write(AnimationFinished {
                        entity,
                        state: controller.state,
                    });
                }
            }
        }
    }
}

/// System that applies animation state based on movement/combat state.
/// This is a placeholder - integrate with your actual game state.
pub fn animation_state_machine(
    mut query: Query<(Entity, &mut AnimationController)>,
    mut _changed_events: MessageWriter<AnimationStateChanged>,
) {
    for (_entity, mut controller) in &mut query {
        // Placeholder: just stay in idle state
        // Real implementation would check MovementState, AttackState, Health, etc.
        let _new_state = AnimationState::Idle;

        // Only change if different (the set_state method handles this)
        if controller.state != AnimationState::Idle && controller.finished {
            controller.set_state(AnimationState::Idle);
        }
    }
}
