# Section 19 — Realm Content and End-to-End Adventure Loop

**Status:** Planned
**Dependencies:** World, party, combat, class/range, effects/AI, recovery, quest, skill-check, and balance foundations from earlier sections.

## Objective

Prove that the new systems form one coherent, persistent playable region before scaling up to many realms.

## Required content

- Connect a starting village to wilderness routes and multiple landmarks.
- Include a dangerous destination with distinct enemy archetypes and rewards.
- Add at least one recruitable companion or a companion-related quest.
- Add at least one multi-stage quest that combines travel, combat, and an item, gathering, or crafting objective.
- Integrate settlement services, trade, equipment preparation, and companion recovery into the adventure loop.
- Use reusable world, enemy, companion, and quest definitions rather than one-off hardcoded navigation/control flow.

## Implementation slices

### 19.1 — Content definitions

Define a small connected region and its routes, destinations, encounter tables, landmarks, NPCs, and quest content using the established data models. Include clear travel and danger feedback.

### 19.2 — Complete player-facing loop

The player must be able to:

1. Prepare a party in a village.
2. Travel to a connected location.
3. Encounter and fight a group of enemies.
4. Gain rewards and quest progress.
5. Experience a companion knockout.
6. Return to a settlement or otherwise resolve the expedition.
7. Recover companions and prepare for another attempt.
8. Complete a multi-objective quest and receive its rewards.

### 19.3 — End-to-end and save/load regression

- Exercise the entire loop through domain-level integration tests where possible, with UI smoke checks for the terminal flows.
- Save and reload at meaningful points: before travel, during quest progress, after rewards, while a companion is recovering, and after quest completion.
- Verify that repeated interactions do not duplicate rewards and that travel, inventory, party state, location, recovery, and quest state remain consistent.

## Scope boundaries

- Build one coherent region rather than many partially connected realms.
- Do not introduce multiplayer, permanent companion death, or a full graphical frontend.
- Avoid new mechanics that have not passed their foundational section's acceptance tests.

## Acceptance criteria

- The complete eight-step adventure loop is playable.
- Multi-objective quest progress and rewards persist correctly.
- A knockout has a coherent recovery/rescue outcome.
- Travel and combat rewards use the shared domain systems.
- Save/load cycles do not corrupt location, inventory, party, recovery, or quest state.
- Automated tests cover the important state transitions and reward-idempotency cases.
