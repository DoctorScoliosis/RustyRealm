# Section 8 — Hostile Encounter Roster Cleanup

**Status:** Implemented, merged, and command-verified on 2026-10-28. One non-fatal `dead_code` warning remains.
**Related PR:** https://github.com/DoctorScoliosis/RustyRealm/pull/10
**Merge commit:** https://github.com/DoctorScoliosis/RustyRealm/commit/ece9a31d64f53de5bc43de11ee90488a084cff1f

## Objective

Keep ordinary wandering combat encounters intentional and hostile by removing generic civilians and ordinary wildlife from the random combat pool.

## Implemented scope

- The wandering pool contains Goblin, Giant, Dragon, Rockbeast, Ent, and Elemental.
- Human, Farmer, Bear, Rat, Thief, Mage, and Sorcerer are excluded from ordinary wandering selection.
- Named Stronghold guardians and existing story-boss encounters are left unchanged.
- Legacy enum variants and reward mappings are retained until wider uses can be audited safely.
- A regression test asserts the exact approved six-enemy pool and checks for excluded civilian/wildlife types.

## Merge and source verification

- Confirmed PR #10 is closed and merged into `main`.
- Confirmed the merge commit is `ece9a31d64f53de5bc43de11ee90488a084cff1f`.
- Inspected the merged `src/combat.rs` on `main`: the explicit six-entry `WANDERING_ENEMIES` array is present, wandering selection draws from that array, and the regression test is present.

## Local verification results (2026-10-28)

The developer ran the following commands against their local checkout:

| Command | Result |
|---|---|
| `cargo fmt --check` | Completed with no reported formatting differences. |
| `cargo test --package rustyrealm` | Passed: 64 passed, 0 failed, 0 ignored. |
| `cargo clippy --package rustyrealm --tests` | Completed with a non-fatal warning; no Clippy error was shown in the supplied output. |

Warning recorded:

```text
warning: method `sell_price` is never used
   --> src/data.rs:439:12
```

This warning concerns `ItemType::sell_price` and is not identified in the supplied output as a failure of the encounter-roster change. It should be reviewed separately; do not suppress or delete the method without checking its intended use.

## Guardrails

- Do not delete legacy NPC variants or reward mappings as part of this narrow change.
- Do not change Stronghold or story-boss selection while working on unrelated encounter improvements.
- New wandering enemies should be added deliberately to an explicit hostile roster, not by expanding an unexplained numeric roll range.

## Acceptance criteria

- Ordinary wandering encounters draw exclusively from the six approved enemy types.
- Existing Stronghold guardians and story bosses retain their intended behaviour.
- Reward handling for retained enemy variants remains intact.
- The encounter-pool regression test passes.
- Formatting, tests, and Clippy have been run locally; current results are recorded above.
