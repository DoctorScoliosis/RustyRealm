# Section 9 — World Model and Connected Locations

**Status:** Slice 9.1 implemented and locally verified. Slice 9.2 implemented on the working branch; local verification pending. Slice 9.3 remains open.  
**Branch:** `work/world-model-foundation`

## Goal

Introduce a small, testable domain model for stable locations and travel connections before wiring the world into the main menu or extending combat.

This is an implementation brief, not a claim that save persistence or the travel interface is implemented.

## Progress record — 2026-10-28

- Added [src/world.rs](https://github.com/DoctorScoliosis/RustyRealm/blob/work/world-model-foundation/src/world.rs) with stable `LocationId`, location definitions and categories, directed routes, graph validation, direct and transitive reachability queries, and weighted shortest-path lookup.
- Nearest-settlement lookup uses cumulative route cost and breaks equal-cost ties by stable location ID. Bidirectional route insertion validates both directions before mutating the graph.
- The initial region includes Oakhaven Village, three adjacent landmarks, a dangerous route through Whispering Woods, Sunken Ruins, and Goblin Caves.
- Added eight unit tests covering initial content, direct versus transitive reachability, route-cost ordering, nearest-settlement route costs and tie-breaking, disconnected locations, invalid definitions, and atomic bidirectional-route insertion.
- Registered the module in `src/main.rs`. A documented temporary `dead_code` allowance is in place until the world APIs are consumed by later state/UI integration.
- Initial local validation (2026-10-28): `cargo test --package rustyrealm` passed all 73 tests (0 failures). The first `cargo fmt --check` report included formatting differences in `src/combat.rs` and `src/world.rs`; a later rerun found three residual `rustfmt` differences in `src/world.rs`, corrected in follow-up commit `9ae8b54`. Rerun the check against the latest branch head to confirm.
- Initial `cargo clippy --package rustyrealm --tests` completed with a warning for unused `ItemType::sell_price` and four `unnecessary_map_or` suggestions in this module. The four world-module cases have been changed to `is_some_and` / `is_none_or`. Rerun Clippy to confirm those warnings are resolved; the existing `sell_price` warning remains to be reviewed separately.
- Final local verification (2026-10-28): `cargo fmt --check` passed with no output; `cargo test --package rustyrealm` passed all 73 tests (0 failures); `cargo clippy --package rustyrealm --tests` completed with only the pre-existing unused `ItemType::sell_price` warning. `git status --short --branch` reported `work/world-model-foundation...origin/work/world-model-foundation` with no working-tree changes.
- Slice 9.2 adds `GameState.current_location` as a stable `LocationId`, defaulting to `village_start` for new games and JSON saves missing the field. Save tests now cover a non-default location round trip, version-2 checksum compatibility when the field is absent, and legacy JSON migration with preservation of representative character progression. Local verification is pending; no terminal travel UI has been added, as that belongs to Slice 9.3.

## Repository context

The current `GameState` in `src/state.rs` persists the player, quests, world events, market state, Stronghold progression, achievements, titles, themes, and mastery perks. It has no current-location field. `main.rs` currently exposes `Wander Gielinor` and labels the main menu `Final Realm · Varrock`. Existing saves are serialised through the save layer, and added persisted fields need serde defaults or an explicit migration.

## Slice 9.1 — Domain model only

Create a focused world/location module (for example, `src/world.rs`) without adding travel or encounter behaviour to UI code.

### Required concepts

- A stable `LocationId` newtype or equivalent type; avoid using display names as identity.
- Data definitions for locations, including ID, display name, category, and enough descriptive metadata for future expansion.
- Location categories for settlements, wilderness, ruins, dungeons, and landmarks.
- Directed or bidirectional route definitions with stable endpoints, a positive travel cost, and route danger/encounter metadata.
- A small static starting region: one village, two or three nearby landmarks, one dangerous wilderness route, and one dungeon/boss destination.
- Pure queries for known locations, direct connections, reachable destinations, and the nearest reachable settlement.

### Design constraints

- Keep world rules independent from Ratatui and Crossterm.
- Validate world definitions: unique IDs, existing endpoints, no self-routes unless explicitly supported, and valid travel costs.
- Make route directionality explicit. Do not silently treat a one-way route as bidirectional.
- Define nearest-settlement semantics deterministically. Use route cost, and specify a stable tie-breaker.
- Keep the region data-driven so new locations do not require rewriting navigation logic.
- Do not implement party members, combat encounters, random travel events, recovery, or quest objectives in this slice.

## Slice 9.2 — Persist current location

After the model and its tests are stable, add current location to `GameState` with a backward-compatible serde default resolving old saves to the starting village. Do not change or discard existing player progression, inventories, mastery, achievement, or quest data.

The save/load test must cover both a new state and a legacy JSON payload that lacks the new field.

## Slice 9.3 — Minimal travel integration

Only after slices 9.1 and 9.2 pass, expose reachable destinations in a small terminal navigation menu. Reject disconnected destinations and update current location only after validating the route and travel cost. Keep keyboard navigation intact.

## Acceptance criteria

- The model is testable without launching the terminal UI.
- The starting region includes the content described above.
- Invalid world definitions are detected by tests.
- Reachability and nearest-settlement queries are deterministic and covered by tests, including disconnected components and ties.
- Legacy saves without a location load into the starting village without resetting other state.
- The player can eventually travel only along defined connections; disconnected destinations are rejected.
- No future mechanics are represented as already implemented.

## Verification gate

Run locally before merging each implementation PR:

```sh
cargo fmt --check
cargo test --package rustyrealm
cargo clippy --package rustyrealm --tests
```

Report the exact commands and results. Do not claim verification if the commands were not run.
