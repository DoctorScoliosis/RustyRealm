# Section 14 — Companion Knockouts, Recovery, and Expeditions

**Status:** Planned
**Dependencies:** Sections 9–11 location, companion, and combat foundations; Section 13 status handling where relevant.

## Objective

Connect combat consequences to persistent companion state and the settlement network without introducing permanent companion death.

## Implementation slices

### 14.1 — Knockout state transition

- At zero health, mark the companion knocked out and remove them from eligible combat actions.
- Persist the knockout and prevent the companion from being selected for another expedition until recovery rules allow it.
- Define state transitions between active, available, knocked out, recovering, and recovered/unavailable states. Avoid contradictory combinations of independent flags.

### 14.2 — Rescue destination and treatment

- Use the world graph to resolve the nearest reachable settlement capable of treatment; document deterministic tie-breaking and behaviour when no settlement is reachable.
- Persist recovery destination, recovery duration or completion time, and any treatment cost.
- Add a simple settlement service using a time-based or paid-treatment model. Validate cost and eligibility before charging or changing recovery state.
- Make recovery status and requirements visible to the player.

### 14.3 — Expedition defeat and reload safety

- Define outcomes when the player is defeated or the whole party is incapacitated, including retreat/rescue and a consistent location result.
- Protect location, inventory, quest progress, and companion state from partial updates if defeat handling fails.
- Ensure opening menus, reloading, or repeating actions cannot instantly restore a companion who is still recovering.

## Scope boundaries

- Permanent death is explicitly out of scope.
- Do not duplicate location-graph algorithms inside recovery code.
- Keep recovery state transitions and payment rules testable outside the UI.

## Acceptance criteria

- Knocked-out companions are persistently unavailable for expeditions and combat.
- Recovery destination is calculated from the world graph and stored consistently.
- Recovery rules survive save/load and cannot be bypassed by simple reload/menu exploits.
- Treatment cost/time and current destination are visible to the player.
- Full-party defeat has a defined, tested outcome without corrupting unrelated state.
- No companion can suffer permanent death in this implementation.
