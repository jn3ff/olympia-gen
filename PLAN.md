# Olympia Core Loop Bootstrap Plan

This plan focuses on scaffolding the core loop in Bevy 0.18 with modular, data-driven systems.

Phase 0: Project skeleton and data pipeline
- Set up Bevy app with plugin boundaries: core, movement, combat, rooms, rewards, ui, content.
- Choose physics stack: bevy_xpbd_2d.
- Define GameState/RunState and RunConfig.
- Add RON schemas for Character, GodBlessing, Weapon, Skill, SkillTree, Room.

Phase 1: Player controller (Hollow Knight-inspired)
- Implement run, jump, coyote, wall slide/cling, wall jump.
- Add ground-only dash/roll (initially).
- Build a small test room to tune movement.

Phase 2: Combat baseline
- Add hitbox/hurtbox, health, stagger, iframes.
- Implement basic enemy AI: patrol, chase, attack.
- Wire weapon actions: light, heavy, special.
- Add skill slots: passive, common, heavy.

Phase 3: Rooms and run graph
- Implement RoomDef-driven loading with exits (up/down/left/right).
- Build arena hub and directional choice UI.
- Add room transitions and spawn flow.

Phase 4: Boss encounter
- Boss state machine with telegraphs and arena lock/unlock.
- Boss defeat triggers reward flow.

Phase 5: Rewards and build growth
- Reward selection UI (tree node, equipment, base stat).
- Apply to PlayerBuild and RunState.
- Equipment slots: helmet, chestplate, greaves, boots, main hand.

Phase 6: Core loop integration
- Wire arena -> room(s) -> boss -> reward -> next segment.
- Add difficulty scaling hooks and run seed.

Open questions
- Finalize xpbd version alignment with Bevy 0.18.
- Decide RON asset loading approach (custom loader vs helper crate).
- Define initial roster of 12 characters and god blessings.
