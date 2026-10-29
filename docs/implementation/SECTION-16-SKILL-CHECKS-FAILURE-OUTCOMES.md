# Section 16 — Skill Checks and Failure Outcomes

**Status:** Implemented — review via the Section 16 pull request
**Dependencies:** Existing skills, mastery and economy/inventory operations; later used by composite quests and realm content.

## Implementation summary

- Add `src/skill_checks.rs`, a UI-independent bounded probability model.
- Formula: `clamp(30 + floor(min(skill_level, 100) / 2) - difficulty + modifier, 10, 95)` percent.
- A controlled integer roll in `0..=99` succeeds when `roll < success_chance`. Invalid rolls are rejected.
- Woodcutting difficulty is 25; Mining difficulty is 30; Thieving difficulty is 25. These are the only currently integrated actions.
- Gathering failure is a wasted attempt: no item yield, mastery bonus, or skill XP. Thieving failure awards no gold, thread, or skill XP. No input materials or tools are consumed on these failures.
- Existing Cooking, Fishing, Smithing/recipe actions and exploration routes are not yet checked: their current resource/reward semantics are deliberately preserved until separate activity-specific policies are implemented.

## Probability model and bounds

Skill level is capped at 100 before calculation. Modifier values are action-specific percentage points and are clamped through the final probability bounds, so neither positive nor negative modifiers can exceed 95% or fall below 10%. The model accepts only integer rolls 0 through 99, supporting deterministic boundary tests.

## Transaction policy

The integrated gathering and thieving actions validate existing prerequisites before the check. These current checked actions have no consumed inputs, so failure is transactional by construction: failure only produces a result message and does not mutate inventory, currency, or skill XP. Success retains the existing yield, XP, mastery and thread reward behaviour. Resource-consuming recipes and fishing/cooking/smithing have no new rolls in this slice.

## Tests

Unit tests cover low/medium/high skill values, monotonically better chances at higher relevant levels, bounds, oversized skill levels, extreme modifiers, exact success threshold, invalid rolls and activity-specific difficulties/outcomes.

## Remaining scope

Crafting and exploration are intentionally not modified yet. Equipment and class abilities do not currently affect these checks because no audited, directly relevant modifiers exist for the selected actions. Mastery remains relevant only to its existing successful gathering yield behaviour; it does not alter check probability.
