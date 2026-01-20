//! Rooms domain: events for room transitions and clears.

use bevy::ecs::message::Message;

use crate::content::Direction;

#[derive(Debug)]
pub struct RoomClearedEvent {
    pub room_id: String,
}

impl Message for RoomClearedEvent {}

#[derive(Debug)]
pub struct BossDefeatedEvent {
    pub boss_id: String,
}

impl Message for BossDefeatedEvent {}

#[derive(Debug)]
pub struct EnterRoomEvent {
    pub room_id: String,
    pub entry_direction: Direction,
}

impl Message for EnterRoomEvent {}

#[derive(Debug)]
pub struct ExitRoomEvent {
    pub direction: Direction,
}

impl Message for ExitRoomEvent {}
