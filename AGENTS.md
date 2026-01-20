# Repository Guidelines

## Project Structure & Module Organization
The project is a Rust/Bevy game prototype. Core wiring happens in `src/main.rs`, which registers domain plugins for gameplay systems. Each major domain lives in its own module folder with a `mod.rs` entry point and focused submodules; `mod.rs` re-exports public types needed by other domains. Examples:
- `src/core/{events,resources,state,systems,ui}/`
- `src/movement/{components,resources,bootstrap,dev,systems}/`
- `src/combat/{components,attacks,resources,events,systems,ai,spawn}/`
- `src/encounters/{types,selection,triggers,buffs,events,tests}/`
- `src/rooms/{components,data,graph,registry,spawn,systems,ui,events,tests}/`
- `src/rewards/{build,economy,types,choices,shop,ui,faith}/`
- `src/ui/{hud_player,hud_wallet,hud_enemy,hud_boss,death}/`
- `src/debug/{state,ui,systems}/`
- `src/sprites/{manifest,layers,animation,weapon}/`

Shared data definitions (assets, enums, data schemas) live in `src/content/mod.rs`. Build artifacts go to `target/`, and `PLAN.md` captures roadmap notes. Each new module file should start with a short `//!` domain description.

## Build, Test, and Development Commands
- `cargo run` — build and launch the game with the default Bevy window settings.
- `cargo build` — compile the project without running it.
- `cargo check` — fast type-checking pass during iteration.
- `cargo test` — run unit/integration tests (none exist yet, but use this when adding tests).

## Issue Tracking

Use 'bd' for issue/task tracking

## Coding Style & Naming Conventions
Use standard Rust style: 4-space indentation, `snake_case` for functions/fields, `PascalCase` for types and enums, and `SCREAMING_SNAKE_CASE` for constants. Bevy patterns are preferred: ECS types should be named clearly (`Component`, `Resource`, `Event` derivations), and plugin structs should follow `XxxPlugin` naming. Module boundaries map to gameplay domains; keep new systems in the most relevant module and expose only necessary types.

Formatting: run `cargo fmt` before submitting changes. Optional linting: `cargo clippy` is recommended for behavior changes.

## Testing Guidelines
No test framework is set up yet. When adding tests, follow Rust conventions (`#[test]` in module files or `tests/` integration files). Name tests descriptively, e.g., `movement_applies_dash_cooldown`. Run `cargo test` locally before opening a PR.

## Commit & Pull Request Guidelines
This repository has no commit history yet, so no established commit-message convention exists. Use short, imperative summaries (e.g., “Add dash cooldown tuning”) and keep unrelated changes split into separate commits.

For PRs, include:
- A short description of the gameplay/system change.
- Any relevant screenshots or short clips for UI/visual changes.
- Notes on how to verify (commands run, manual steps).

## Current Context (Vision + Data)
- Vision decisions live in `concept/GAME-SPEC.md`.
- Data is authored in RON under `assets/data/*.ron`.
- V1 characters: Ares sword, Demeter sword, Poseidon spear, Zeus spear.
- Defaults: segment = 5 rooms / 2 bosses, hub after each segment; win condition = boss count 5.
- No map; no backtracking between rooms; no repeats for significant enemies.
- Combat: light/heavy/special per weapon; stance defaults (4 heavies or 7 lights; parry breaks stance).
- Economy: ~300 coin value per segment; items 50-150, rare 250-800; off-class allowed but 1-2 in-class options.
- Encounters: one specialty tag guaranteed; curated tags tied to weapons; buff tags scale with tier.
- Debug mode must support spawn, apply build templates, and invincible toggle (seed optional).
- Implementation roadmap lives in `PLAN.md`.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
