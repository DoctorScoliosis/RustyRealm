# Section 8 — Hostile Encounter Roster Cleanup

**Status:** Implementation merged to `main` via PR #10; source-level verification confirmed on 2026-10-28. **Command-level validation remains outstanding.**
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

## Merge verification performed

- Confirmed PR #10 is closed and merged into `main`.
- Confirmed the merge commit is `ece9a31d64f53de5bc43de11ee90488a084cff1f`.
- Inspected the merged `src/combat.rs` on `main`: the explicit six-entry `WANDERING_ENEMIES` array is present, wandering selection draws from that array, and the regression test is present.
- The review is source-level only. GitHub returned no associated workflow runs or commit status checks for the merge commit, and the PR body records that Rust tooling was not run during review.

## Guardrails

- Do not delete legacy NPC variants or reward mappings as part of this narrow change.
- Do not change Stronghold or story-boss selection while working on unrelated encounter improvements.
- New wandering enemies should be added deliberately to an explicit hostile roster, not by expanding an unexplained numeric roll range.

## Acceptance criteria

- Ordinary wandering encounters draw exclusively from the six approved enemy types.
- Existing Stronghold guardians and story bosses retain their intended behaviour.
- Reward handling for retained enemy variants remains intact.
- The encounter-pool regression test passes.

## Command-level verification still required

Run these checks from the merged source before marking Section 8 fully validated:

- `cargo fmt --check`
- `cargo test --package rustyrealm`
- `cargo clippy --package rustyrealm --tests`

The current environment has no Rust toolchain and cannot reach GitHub from a local shell, so these commands could not be executed here. Record actual results when they are run; do not describe commands as passing unless they were executed.
