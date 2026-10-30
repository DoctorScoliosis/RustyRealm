# Section 22 — Taverns, Guildhouses and Short/Long Rest

**Status:** Implemented on PR #29. **Dependencies:** Sections 20–21 and the existing HP, stamina, mana, economy and companion-recovery systems.

## Location services

Location category and location services are separate concepts. A settlement does not automatically provide lodging, a guildhouse, a healer or a campsite. Services are explicitly listed on each `Location`; the serde-defaulted service list keeps old location data readable and defaults unknown legacy locations to no services.

The initial region currently declares:

| Location | Explicit services |
|---|---|
| Oakhaven Village | Tavern, Guildhouse, Bed, Healer |
| Crumbling Watchtower | FieldCamp |
| Standing Stones | FieldCamp |
| Abandoned Mill | FieldCamp |
| Whispering Woods | FieldCamp |
| Sunken Ruins | None |
| Goblin Caves | None |

A future location can combine any subset of these services without relying on its category.

## Rest economy

| Rest choice | Eligibility | Duration | Cost | Recovery |
|---|---|---:|---:|---|
| Short rest | Tavern, Guildhouse or Bed | 2 hours | Free | +15% max HP, +15% max stamina, +15% max mana |
| Long rest | Bed | 8 hours | 25 gold | Restore HP, stamina and mana to maximum |
| Field camp | Explicit FieldCamp service | 3 hours | 1 Food | +10% max HP, +15% max stamina, +10% max mana |

Percentages are calculated from maximum resource capacity, rounded down to whole points, with a minimum gain of one point when a resource is below maximum. Gains are capped at the corresponding maximum. The plan is calculated before any cost or state mutation.

The balance intent is to make the short rest a modest safe-haven reset, while the long rest is the dependable and time-efficient full recovery. Repeating short rests is deliberately less time-efficient than the long rest: after four short rests (eight hours), a badly depleted character still will not be fully restored. The long rest's cost buys a complete reset in one predictable block. Field camping remains available at selected outdoor landmarks and wilderness locations, but consumes supplies, takes longer than a short rest and restores materially less. It does not gain the security or efficiency of a bed. There is no random ambush system in this slice, so camping does not fabricate a combat roll or encounter.

A rest that cannot improve any resource is unavailable. Insufficient gold or Food, a missing service, an unknown location, a forbidden execution context, or an exhausted world clock all reject the transaction without charging resources or advancing time. Selecting Back is a strict no-op.

## Status and companion recovery

Combat status effects are scoped to a live encounter; they are not persisted as long-lived player conditions. Rest does not clear or extend those tactical effects and does not mutate companion conditions directly.

Successful rest advances time exclusively through `GameState::advance_world_time`. Existing healer-backed companion recovery timers therefore progress by the exact elapsed hours and can naturally complete. A knockout record with no reachable Healer destination never counts down, is not removed by resting, and remains unavailable until treatment becomes possible. Immediate treatment itself remains a separate paid action requiring travel to the recorded treatment location.

The recovery system now chooses the nearest explicit Healer service instead of treating every `Settlement` category as a treatment facility. Immediate treatment also checks for that service at the recorded destination.

## Validation

Section 22 adds coverage for:
- Explicit rest-service eligibility and the fact that Settlement alone grants no services.
- Exact short-rest, long-rest and field-camp recovery amounts.
- Full-resource, unaffordable, missing-food and exhausted-clock rejection paths.
- Cancellation and combat/scripted-context rejection without mutation.
- Elapsed time advancing healer-backed recovery while preserving records without a treatment destination.
- Separation of immediate companion treatment from ordinary rest.

The PR is intentionally left open and unmerged.
