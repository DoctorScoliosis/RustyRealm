# Section 10 — Party and Companion Foundations

**Status:** Planned
**Dependencies:** Section 9 location model; Section 11 combat integration follows this foundation.

## Objective

Introduce recruitable NPC companions and persistent single-player party composition without coupling companion data to the player's inventory or progression.

## Implementation slices

### 10.1 — Definitions and persistent state

- Introduce stable companion IDs and companion definitions separate from each companion's mutable saved state.
- Define class/role, base combat attributes, equipment options, abilities, and any recruitment requirements in reusable structures.
- Track current condition and availability explicitly. At minimum, model active, available, knocked out, recovering, and otherwise unavailable states.
- Add a small initial roster covering frontline defence, ranged damage, and magic or support.
- Keep companion inventory/equipment and progression data separate from `Player` fields unless a shared abstraction is justified by the current code.

### 10.2 — Party operations

- Add a configurable party-size limit.
- Implement domain operations for recruiting, dismissing, selecting, and removing party members.
- Validate eligibility, duplicates, recruitment requirements, availability, and capacity before mutating state.
- Define clearly whether the player character counts toward the limit and apply that rule consistently in UI and tests.

### 10.3 — Save compatibility and minimal management UI

- Persist companion records and party composition.
- Use deliberate serde defaults or an explicit migration for older saves; never reset unrelated character progression, inventory, quests, mastery, achievements, or bank data.
- Add only the minimum menu flow needed to inspect companions and assemble a valid party. Keep rules in domain logic, not Ratatui rendering code.

## Scope boundaries

- Do not implement full party combat here; Section 11 owns combatant integration.
- Do not implement settlement recovery timers or rescue outcomes here; Section 14 owns recovery.
- Do not add permanent companion death.
- Avoid a large companion-content roster before recruitment and persistence work end to end.

## Acceptance criteria

- Eligible companions can be recruited and a valid party assembled.
- Capacity, recruitment requirements, duplicate selection, and unavailable states are enforced by domain operations.
- Companion definitions are independent from saved companion state.
- Companion state and party composition survive save/load cycles.
- Old saves load with sensible default party state without losing existing progress.
- Tests exercise valid operations, invalid operations, and serialization compatibility.
