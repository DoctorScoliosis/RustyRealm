# Section 17 — Composite Quests and Exploration Objectives

**Status:** Planned
**Dependencies:** Sections 9 and 16 for location and skill-check objectives; Section 19 integrates the full adventure loop.

## Objective

Expand quests from a single objective to structured quests containing sequential stages, parallel objectives, optional goals, and multiple gameplay activities.

## Implementation slices

### 17.1 — Objective and stage model

Support typed objective records for:

- Defeating a specified number or type of enemy.
- Collecting or handing in specified items.
- Gathering resources or crafting an item.
- Reaching or traversing a location.
- Interacting with a landmark or NPC.
- Completing a prerequisite quest.

Represent sequential stages and parallel objective groups explicitly. Optional objectives may grant bonus rewards without blocking main completion unless the quest definition says otherwise.

### 17.2 — Event-driven progress

- Audit the current `QuestObjective`, `Quest`, and `GameEvent` model before choosing a migration shape.
- Progress quests from typed events emitted after successful gameplay actions, not display-text matching.
- Define event payloads and matching rules so unrelated kills, items, locations, or recipes do not advance an objective accidentally.
- Ensure a single successful action advances each applicable objective at most once per emitted event.

### 17.3 — Rewards, UI, and save migration

- Maintain clear accepted, in-progress, completed, and claimed states or equivalent explicit lifecycle semantics.
- Guarantee rewards are granted exactly once even if the claim action is repeated.
- Provide clear progress, next-stage, optional-objective, and reward information in the terminal UI.
- Migrate existing single-objective quests and legacy saves explicitly. Use serde defaults only where they preserve meaning; test migration rather than silently discarding or resetting progress.

## Scope boundaries

- Do not replace existing quest structures without a compatibility plan.
- Do not make objectives depend on UI labels or incidental call order.
- Avoid a large quest-content expansion until reusable objective types and migration tests pass.

## Acceptance criteria

- A quest can track multiple objectives with independent progress.
- Sequential stages unlock in the right order while parallel objectives progress together.
- Location objectives use the world model; gathering/crafting objectives use successful domain events.
- Rewards are granted exactly once.
- Existing quest saves remain valid or use an explicit, tested migration.
- UI progress corresponds to domain state and does not determine it.
