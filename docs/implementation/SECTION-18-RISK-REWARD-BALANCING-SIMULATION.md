# Section 18 — Risk–Reward Balancing and Combat Simulation

**Status:** Planned
**Dependencies:** Combat, classes, effects, skill checks, recovery, quests, and economy systems should be sufficiently stable to measure.

## Instructions for the implementing agent

Treat this as an executable implementation brief, not a request to return only a proposal. Any capable coding agent may do the work; ChatGPT is the primary implementation partner in the current workflow, but no particular model or vendor is mandatory. Use the repository and tools actually available to you, and be transparent about limitations.

Before editing, inspect the latest `main`, the relevant implementation, save/data models, tests, and dependencies. Treat the current source as authoritative: the names and examples in this brief describe desired behaviour, not guaranteed existing APIs. Confirm that prerequisite sections have landed in the selected base. If a required foundation is absent, stop before building on speculative types and report the dependency blocker.

Make focused, reviewable changes on a feature branch from the latest `main`. Preserve user work. Do not push feature work directly to `main`, reset or force-push, disable CI, merge your own PR, or bundle unrelated cleanup. Keep domain rules independently testable and separate from terminal presentation. Use deterministic/injected randomness where relevant. Never claim a command or check passed unless it was actually run.

## Required CI feedback loop and PR handoff

RustyRealm has GitHub Actions CI in `.github/workflows/rust.yml`, using GitHub-hosted **Ubuntu 24.04** runners. It triggers on pushes, pull requests, and manual dispatch, with three checks:

- Formatting: `cargo fmt --check`
- Tests: `cargo test --package rustyrealm`
- Clippy: `cargo clippy --package rustyrealm --tests`

1. Run the equivalent checks locally whenever your environment supports Rust. Then commit and push the feature branch and open a pull request against `main` for the repository owner to review. Do not merge it.
2. Inspect the real Actions run associated with the PR's latest head commit. Confirm that **all three jobs** have completed successfully; do not infer success from the PR merely existing or from a previous commit's run.
3. If any job fails, inspect its logs and identify the actual root cause. Make the smallest appropriate fix on the same feature branch, commit and push it, then inspect the fresh run for that new commit. Repeat this feedback loop until all checks pass.
4. Do not silence warnings by weakening checks, add `continue-on-error`, skip a failing job, loosen tests to manufacture a pass, or state that CI is green without evidence. If a failure is an external outage, an environment limitation, or a product decision that requires the owner's judgement, report the exact blocker and leave the PR open rather than pretending it passed.
5. Hand off with the PR link, a concise summary, files changed, local commands actually run, the latest CI run/result, and any remaining limitations. Leave the PR **open and unmerged** for the owner.

The CI workflow is the shared verification feedback loop for ChatGPT and any other agent willing and able to participate. Where an agent cannot access GitHub Actions or push updates, it must provide a truthful limitation and a usable patch/handoff rather than claiming the loop completed.

## Objective

Create repeatable balance measurements for enemies, classes, equipment, skills, quests, and rewards before substantially expanding realm content.

## Implementation slices

### 18.1 — Establish baseline scenarios and targets

- Audit enemy health, damage, speed, resistances, abilities, drops, and reward tables.
- Define representative character/party builds and encounters for early, middle, and late progression.
- Document expected difficulty ranges, target turn counts, acceptable incoming damage, healing/consumable use, and intended recovery costs.
- Treat target values as explicit design assumptions to be revised using evidence.

### 18.2 — Repeatable simulation

- Build deterministic or seed-controlled combat simulations that invoke the same domain rules as live encounters.
- Measure turns to defeat, party damage taken, knockout frequency, abilities/resources used, and encounter outcomes.
- Compare class and party matchups against representative enemy archetypes.
- Reassess Stronghold Gauntlet scaling, escalating rewards, and modifiers.

### 18.3 — Progression and economy

- Review gold, XP, loot, crafting requirements, shop prices, skill-check curves, consumable supply, and recovery costs together.
- Look for repeated-action, quest-claim, sale/buyback, Gauntlet, and other reward exploits.
- Add regression thresholds for material balance shifts. Document intentional changes instead of relying on vague intuition.
- Keep costly simulations out of the normal interactive path and make their inputs reproducible.

## Scope boundaries

- Do not tune by undocumented intuition alone or overfit to one character build/seed.
- Do not slow ordinary gameplay with simulation work.
- Balance changes must not silently alter save interpretation or quest progress.
- Do not use noisy or unstable thresholds that make CI flaky; document variance and use fixed seeds or justified ranges.

## Acceptance criteria

- Representative encounters have documented expected difficulty and reward ranges.
- Repeatable simulations compare relevant classes/parties and enemy archetypes.
- Classes have useful strengths and trade-offs without one universally dominating.
- Higher-risk encounters offer proportionate rewards without trivialising progression.
- Gauntlet difficulty/rewards remain worthwhile across progression.
- Regression checks flag significant unintended shifts in combat and economy metrics.

## Section-specific verification

Run balance scenarios with fixed seeds or documented seed sets and assert robust bounds rather than a single brittle expected outcome. Keep simulation checks appropriate for CI runtime, then confirm all standard CI jobs pass before PR handoff.
