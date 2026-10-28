# Section 12 — Character Classes, Positioning, Range, and Targeting

**Status:** Planned
**Dependencies:** Section 11 combatant and action-resolution foundation.

## Objective

Make class identity and relative battlefield position influence tactical decisions without committing prematurely to a full grid.

## Core design

- Keep class identity separate from attributes, equipment, abilities, and mastery perks.
- Start with four distinct archetypes: Warrior (frontline durability/control), Ranger (ranged precision), Mage (elemental damage/status effects), and Rogue (speed/critical hits/evasion). Treat these as initial roles to balance, not hard-coded exceptions throughout the engine.
- Define attributes including speed, accuracy, evasion, power, and resistance, with sensible defaults and bounded values.
- Introduce measurable positions or range bands, initially melee, near, and far. Do not build a full square-grid battlefield in this phase.
- Define which attacks and abilities can target which range bands, including movement or range-changing actions where needed.

## Implementation slices

### 12.1 — Classes and attributes

Implement reusable class definitions and documented class trade-offs. Make speed contribute predictably to the initiative policy established in Section 11.

### 12.2 — Range and target validation

Represent relative position in combat state. Validate target existence, allegiance, availability, and range before action costs or effects are applied.

### 12.3 — Accuracy and combat UI

Define a documented hit-chance formula that uses relevant attacker/defender attributes, class modifiers, and distance. Clamp it to configured bounds. Show target information, distance, and reasons an action is unavailable in the interface.

## Scope boundaries

- No full tactical grid.
- No UI-only implementation of hit chance or legal-target rules.
- Do not assume class labels alone create meaningful gameplay; tests must demonstrate mechanical differences and trade-offs.

## Acceptance criteria

- Each initial class has distinct, testable advantages and disadvantages.
- Faster combatants generally act earlier according to a documented formula.
- Out-of-range actions are rejected before resource consumption.
- Accuracy changes predictably with distance and class capability.
- Positioning and targeting choices can change encounter outcomes.
- Tests cover probability bounds, range boundaries, and invalid targets.
