//! Rooms domain: events for room transitions and clears.

use bevy::ecs::message::Message;

use crate::content::Direction;

#[derive(Debug)]
pub struct RoomClearedEvent {
    #[allow(dead_code)]
    pub room_id: String,
}

impl Message for RoomClearedEvent {}

#[derive(Debug)]
pub struct BossDefeatedEvent {
    #[allow(dead_code)]
    pub boss_id: String,
}

impl Message for BossDefeatedEvent {}

#[derive(Debug)]
pub struct EnterRoomEvent {
    #[allow(dead_code)]
    pub room_id: String,
    #[allow(dead_code)]
    pub entry_direction: Direction,
}

impl Message for EnterRoomEvent {}

#[derive(Debug)]
pub struct ExitRoomEvent {
    #[allow(dead_code)]
    pub direction: Direction,
}

impl Message for ExitRoomEvent {}
