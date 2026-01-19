# CHANGES

Playtest bug fix paths for the latest feedback round.

## 1) Arena hub spawn drops player into a portal

Observed: On entering the arena hub, the player immediately falls into a portal and transitions before they can explore shops or choose a direction.

Fix path:
1. In `src/rooms/mod.rs`, replace `handle_arena_portal_interaction` auto-transition-on-collision with an explicit confirm flow (press E).
2. Add arena portal zone tracking that mirrors room exits:
   - Set a `PlayerInPortalZone` (or a new arena-specific marker) when the player overlaps an arena portal sensor.
   - Show the "Press [E] to enter" tooltip when inside a portal zone.
3. On E press, trigger the same room selection logic currently in `handle_arena_portal_interaction` (set `RoomGraph.pending_transition` and `RunState::Room`).
4. Optional safety: spawn the player a small distance away from portal lines or add a short grace timer before portal zones become active.

Acceptance check:
- Spawn into the arena hub, land safely, and can walk around shops without transitioning.
- Approaching a portal only shows the tooltip; transition happens only after pressing E.

## 2) Room exit portals become permeable and do not show tooltip

Observed: After clearing a room, the exit turns green but becomes permeable. The player falls out of the side, and no "Press [E]" tooltip appears.

Fix path:
1. In `src/rooms/mod.rs` `spawn_room_geometry`, add a solid barrier collider for each exit gap (Up/Left/Right/Down). Keep it always solid so the boundary remains impermeable even when portals are enabled.
2. Keep the exit sensor as a separate trigger volume positioned just inside the room so the player can overlap it without leaving the room.
3. Ensure portal sensors can generate collision events with the player:
   - Either leave collision layers as-is if sensors already collide, or explicitly assign `CollisionLayers::new(GameLayer::Sensor, [GameLayer::Player])` and add `GameLayer::Sensor` to the player's collision mask.
4. Verify `track_player_portal_zone` adds `PlayerInPortalZone` and `update_portal_tooltip` spawns `PortalTooltipUI` when the portal is enabled.
5. Confirm `confirm_portal_entry` handles E presses and sends `ExitRoomEvent` while the barrier remains intact.

Acceptance check:
- After enemies are cleared, the portal turns green, remains solid, shows "Press [E] to enter", and E transitions to the next room.
