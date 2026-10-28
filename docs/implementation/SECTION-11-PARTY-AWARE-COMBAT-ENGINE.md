# Section 11 — Party-Aware Combat Engine Foundation

**Status:** Planned
**Dependencies:** Section 10 companion data; Section 12 adds class, range, and targeting rules.

## Objective

Refactor the combat rules so an encounter can contain multiple allies and multiple enemies, while keeping action resolution deterministic, independently testable, and incrementally compatible with existing encounters.

## Implementation slices

### 11.1 — Combatant and encounter model

- Introduce stable combatant identities and explicit faction/allegiance.
- Represent combatant statistics and current resources separately from equipment, abilities, effects, and later positioning/range data.
- Model encounter state, eligible combatants, turn order, and terminal outcomes without depending on Ratatui.
- Preserve existing player and enemy data at the boundary while migrating callers incrementally; avoid a wholesale rewrite of every encounter entry point in one change.

### 11.2 — Action-resolution pipeline

- Establish a consistent pipeline for validating an action, checking target eligibility and resource costs, resolving effects, and recording the result.
- Define attack, defence, ability use, movement hooks, and retreat/escape as supported action categories or explicit extension points.
- Validate all prerequisites before consuming resources or changing health.
- Inject or control randomness so tests can reproduce hit, damage, and effect outcomes.

### 11.3 — Turn order and outcomes

- Define and document initiative/turn-order policy. Section 12 will refine it with speed and class mechanics.
- Define victory, party defeat, retreat, and partial-party-knockout results.
- A single companion's defeat must not end the encounter while other eligible allies can still act.
- Keep the existing story encounters, Stronghold guardians, and Lacoyx phases functional during migration.

## Scope boundaries

- Do not implement a full square-grid battlefield.
- Do not mix combat rules with terminal rendering or input polling.
- Do not add all class abilities or status effects in this section; later sections extend the foundation.

## Acceptance criteria

- An encounter supports multiple allies and multiple enemies.
- Every eligible combatant acts under a documented turn-order policy.
- Invalid targets and insufficient resources are rejected before effects are committed.
- One ally being knocked out does not terminate combat when other eligible allies remain.
- Combat rules are unit-testable without launching the terminal UI.
- Existing story encounters and Stronghold battles remain functional during incremental migration.
