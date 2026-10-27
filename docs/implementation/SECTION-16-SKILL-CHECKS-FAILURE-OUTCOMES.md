# Section 16 — Skill Checks and Failure Outcomes

**Status:** Planned
**Dependencies:** Existing skills and economy/inventory operations; later used by composite quests and realm content.

## Objective

Introduce reusable, bounded uncertainty for selected non-combat actions while keeping outcomes predictable, testable, and fair.

## Implementation slices

### 16.1 — Skill-check model and probability

- Define a domain-level check input containing relevant skill value, action difficulty, and applicable modifiers.
- Document the success-probability formula and configurable lower/upper bounds.
- Allow equipment, class abilities, mastery, and environmental conditions to modify a check only when they are relevant to the action.
- Use controlled randomness or an injected roll source so probability and boundary tests are reproducible.

### 16.2 — Action-specific outcomes

- Apply checks selectively to activities such as gathering, thieving, crafting, and exploration; do not add a roll to every routine interaction by default.
- Define outcomes for each activity: for example, wasted time, reduced yield, lost opportunity, or alerted enemies where contextually appropriate.
- Specify whether any materials, currency, tools, or other resources are consumed on failure before implementing the action.
- Keep routine low-risk actions from becoming excessively frustrating.

### 16.3 — Transactional validation

- Validate prerequisites before rolling or mutating inventory/resources.
- Apply the documented success or failure outcome as one coherent operation so a failed validation cannot partially consume resources.
- Keep action policy separate from terminal rendering.

## Scope boundaries

- Do not use one universal penalty for every failure.
- Do not let equipment, mastery, or skill bonuses exceed configured probability bounds.
- Avoid adding checks to existing activities until their current resource and reward semantics have been audited.

## Acceptance criteria

- Every implemented check has a documented difficulty, formula, bounds, and failure outcome.
- Higher relevant skill generally improves success probability.
- Resource consumption is explicitly defined and transactional.
- Tests cover low, medium, and high skill values, probability bounds, and edge cases.
- Controlled rolls make tests deterministic.
- Bonuses cannot push success beyond the configured maximum or below the configured minimum.
