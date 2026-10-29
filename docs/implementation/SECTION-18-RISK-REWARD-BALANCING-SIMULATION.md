# Section 18 — Risk–Reward Balancing and Combat Simulation

**Status:** In progress
**Baseline version:** 1
**Implementation:** `src/balance_simulation.rs`

## Purpose and method

The balance harness is separate from the interactive gameplay path. It invokes the shared `party_combat::Encounter` resolver and shared enemy action policy with reproducible, seed-derived hit and critical rolls. It does not use a separate damage formula for attacks. A regression row is reproducible from the scenario ID and seed.

Baseline ranges are initial design assumptions rather than assertions of objectively correct difficulty. When combat, economy, skill checks, recovery or Gauntlet rewards intentionally change, update the related range with evidence and explain the intended change in the corresponding PR.

## Representative baseline scenarios

| ID | Progression band | Encounter | Expected actor activations | Maximum expected damage taken |
| --- | --- | --- | ---: | ---: |
| `low-rat` | Low | Rat | 1–8 | 110 |
| `low-goblin` | Low | Goblin | 1–10 | 150 |
| `middle-stronghold-warrior` | Middle | Stronghold Warrior | 2–18 | 500 |
| `middle-dragon` | Middle | Dragon | 2–20 | 650 |
| `high-lacoyx` | High | Lacoyx | 3–35 | 1,200 |
| `high-gauntlet-wave-10` | High | Gauntlet Fiend, Wave 10 | 4–45 | 1,400 |

The harness builds explicit low, middle and high characters with documented HP, stamina, mana, attack and defence values in source. Enemy health and attack receive a bounded seed-controlled 93–107% variance. The suite uses eight default seeds and is deliberately small enough for the existing unit-test CI job.

## Metrics

Each run captures the scenario, progression band, seed, outcome, actor activations, party damage taken, knockouts, ability use, stamina and mana spent, healing usage, recovery-gold assumption, and reward estimates. Healing currently remains zero because the one-character baseline policy does not consume healing items; the field is retained for future party-policy simulations and must not be read as proof of consumable balance.

The `check_regressions` helper compares the recorded measurements to explicit action-count, incoming-damage and scenario reward envelopes. This creates a starting point for future multi-party and resource/economy sweeps; it does not yet simulate every shop, quest-claim, crafting or resale transaction sequence.

## Stronghold Gauntlet

The simulation uses the production `calculate_gauntlet_rewards` helper for its wave-10 reward row. Existing formula tests retain the known quarter-per-wave reward curve and enemy-stat modifier behaviour. Future balance work should sweep more waves, compare reward growth with health/damage growth, and test whether carried resources/recovery costs make continuing rational, rather than silently changing production rewards on intuition.

## Economy and scope boundaries

- Production economy transaction helpers and their atomicity/caps remain authoritative; the simulation does not mutate a real `GameState`.
- The harness includes a conservative sell-spread invariant check, but repeated-action, quest-claim and sale/buyback exploit checks require dedicated transaction-sequence tests and are not claimed complete here.
- Existing save interpretation, quest progress and live gameplay behavior are unchanged by this addition.
- Thresholds are explicit, versioned design assumptions. A red metric is a request to investigate; it should not be fixed by weakening CI or blindly widening the threshold.

## Running

The existing GitHub Actions workflow runs the harness as part of `cargo test --package rustyrealm`, with the established formatting and Clippy gates:

- `cargo fmt --check`
- `cargo test --package rustyrealm`
- `cargo clippy --package rustyrealm --tests`

The feature branch must be validated on the latest pull-request head. If a check fails, inspect its logs, correct the specific cause on this branch, and repeat. The pull request stays open for owner review; do not merge it from the implementing agent.
