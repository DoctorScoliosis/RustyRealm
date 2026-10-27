# Section 13 — Status Effects, Enemy Archetypes, and Combat AI

**Status:** Planned
**Dependencies:** Sections 11–12 combatant, action, initiative, and targeting foundations.

## Objective

Replace ad hoc combat conditions with a consistent effect system and make enemies tactically distinct through explicit, testable decisions.

## Implementation slices

### 13.1 — Shared status-effect lifecycle

- Define typed effect identifiers and effect data such as source, duration, potency, and resistance interactions.
- Support initial conditions including poison, bleed, burn, stun, slow, mark, and weakened defences.
- Document exactly when effects tick and expire relative to initiative, an actor's action, and turn completion.
- Define stacking, refreshing, replacement, and immunity rules per effect or through a documented common policy.
- Share the same resolution rules across player characters, companions, and enemies.

### 13.2 — Enemy archetypes and decisions

- Define reusable enemy archetypes with preferred range, ability preferences, resistances, and target-selection policies.
- Keep AI decisions in domain logic and return explicit intended actions for the action resolver to validate.
- Add ally protection, ranged pressure, and other behaviours incrementally, each with focused tests.
- Avoid decisions based on terminal output, display names, or fragile text matching.

### 13.3 — Telegraphs and boss compatibility

- Implement telegraphed attacks so a dangerous action is communicated before it resolves and can be tested as a multi-step sequence.
- Preserve existing Stronghold guardians, Lacoyx phases, and special story encounters while migrating to shared effects and AI.

## Scope boundaries

- Do not implement effects only for one combatant type.
- Do not make random AI outcomes impossible to reproduce in tests.
- Avoid introducing large numbers of effects or enemy variants before lifecycle and stacking behaviour is stable.

## Acceptance criteria

- Effects apply, tick, resist, stack/refresh, and expire consistently for all combatant types.
- Duration timing and stacking policies are documented and covered by deterministic tests.
- Resistance modifies application or impact according to explicit rules.
- AI chooses actions through explicit domain decisions that can be unit-tested.
- Telegraphs precede and accurately describe the associated dangerous action.
- Existing bosses and special encounters remain compatible.
