# Section 8 — Hostile Encounter Roster Cleanup

**Status:** Implemented and merged via PR #10; local verification remains to be confirmed.
**Related PR:** https://github.com/DoctorScoliosis/RustyRealm/pull/10

## Objective

Keep ordinary wandering combat encounters intentional and hostile by removing generic civilians and ordinary wildlife from the random combat pool.

## Implemented scope

- The wandering pool contains Goblin, Giant, Dragon, Rockbeast, Ent, and Elemental.
- Human, Farmer, Bear, Rat, Thief, Mage, and Sorcerer are excluded from ordinary wandering selection.
- Named Stronghold guardians and existing story-boss encounters are left unchanged.
- Legacy enum variants and reward mappings are retained until wider uses can be audited safely.
- A regression test asserts the approved pool and excludes selected civilian/wildlife types.

## Guardrails

- Do not delete legacy NPC variants or reward mappings as part of this narrow change.
- Do not change Stronghold or story-boss selection while working on unrelated encounter improvements.
- New wandering enemies should be added deliberately to an explicit hostile roster, not by expanding an unexplained numeric roll range.

## Acceptance criteria

- Ordinary wandering encounters draw exclusively from the six approved enemy types.
- Existing Stronghold guardians and story bosses retain their intended behaviour.
- Reward handling for retained enemy variants remains intact.
- The encounter-pool regression test passes.

## Verification still required

The PR description says Rust tooling was not run during its review. Re-run these checks from the merged source before relying on the result as locally verified:

- `cargo fmt --check`
- `cargo test --package rustyrealm`
- `cargo clippy --package rustyrealm --tests`

Record actual results; do not describe commands as passing unless they were executed.
