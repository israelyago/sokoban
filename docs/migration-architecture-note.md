# Migration Architecture Note (M0)

This note defines how existing runtime modules map to Bevy building blocks while terminal gameplay remains the active runtime.

## Current Modules and Planned Bevy Mapping

- `coord.rs`
  - Keep as-is.
  - Used by Bevy gameplay/resource systems for grid coordinates and movement directions.

- `level.rs`
  - Keep parsing and validation logic as-is.
  - Load once at startup into a Bevy resource:
    - `LoadedLevels(Vec<Level>)`
    - `ActiveLevelIndex(usize)`

- `state.rs`
  - Keep `GameState` as the authoritative mutable gameplay state.
  - Store in Bevy as a resource:
    - `CurrentGameState(GameState)`
    - `UndoHistory(VecDeque<GameState>)` with current 10k cap.

- `rules.rs`
  - Keep `try_step` as the pure gameplay transition function.
  - Call from a single Bevy input-to-step system handling move keys.

- `render.rs`
  - Terminal-only text renderer today.
  - Later replaced by Bevy board sync/render systems; no gameplay logic moved here.

- `app.rs`
  - Terminal event loop remains active for now.
  - Later replaced by `GamePlugin` with app states:
    - `Loading`
    - `Playing`
    - `LevelComplete`

- `main.rs`
  - Terminal entrypoint remains active for now.
  - Later switched to Bevy bootstrap + plugin registration.

## System Boundaries to Preserve

- Source of truth for game rules remains in domain modules (`level`, `state`, `rules`).
- Bevy systems should mirror and present domain state, not re-implement movement/win logic.
- Level format under `levels/` remains unchanged and data-driven.

## M0 Runtime Status

- Terminal runtime path is unchanged and still functional.
- Added tests lock parser/rules/state behavior before Bevy migration steps.
