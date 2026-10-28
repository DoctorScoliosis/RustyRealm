# Section 14 — Companion Knockouts, Recovery, and Expeditions

**Status:** Planned — begin only after Sections 9–11 are integrated in the chosen base; Section 13's effects may affect knockout timing.
**Dependencies:** Section 9 connected-location graph, Section 10 persistent companions and party rules, Section 11 combat outcomes, and save/load architecture.

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

Connect combat consequences to persistent companion availability, rescue destinations, settlement treatment, and expedition recovery without permanent companion death.

## Implementation slices

### 14.1 — Authoritative knockout transition

- At zero HP, transition a companion through one authoritative domain operation into a knocked-out state and exclude them from current/future combat until recovery permits it.
- Persist knockout state and the recovery data needed to resume after reload.
- Align transitions with Section 10's availability model. Avoid contradictory flags; define legal states for active/in-party, available, knocked out, recovering, and recovered/unavailable based on the actual type model.
- A companion knockout must not end combat when other eligible allies remain.
- Prevent duplicate side effects if the same combat outcome is processed twice.

### 14.2 — Nearest reachable treatment settlement

- Use the existing world graph and its authoritative reachability/distance logic to choose the nearest reachable settlement able to treat companions.
- Document what “nearest” means, and use deterministic tie-breaking for equally near settlements.
- Define an explicit, safe outcome if no capable settlement is reachable; do not panic or teleport to an arbitrary location.
- Persist the chosen destination and recovery state. Keep the companion's rescue destination separate from the player's current location; do not silently teleport the whole party.
- Reuse existing service/economy rules where suitable.

### 14.3 — Recovery time and paid treatment

- Inspect in-game time/progression first. Reuse it if available; otherwise introduce only the smallest persisted, deterministic recovery clock needed. Tests must not sleep or depend on flaky wall-clock timing.
- Document when recovery time advances and prevent menu navigation, reloads, or repeated calls from bypassing it.
- If paid treatment is supported, validate eligibility and affordability before charging. Failed treatment must not deduct currency; success must update currency/state exactly once.
- Display recovery location, remaining requirement/time, and treatment cost/status in the TUI.
- Recovered companions return to an eligible state but must not silently rejoin the party if explicit selection is required.

### 14.4 — Expedition and defeat resolution

- Define outcomes for player defeat, retreat, and total-party incapacitation, including where the expedition resolves.
- Apply consequences coherently so failure cannot corrupt inventory, currency, quests, location, mastery, achievements, or companions.
- Preserve reward semantics: no victory rewards for a defeated encounter and no duplicate rewards after reload.
- Prevent companions who are knocked out or recovering from joining an expedition.

## Scope boundaries

- Permanent death, companion deletion as a penalty, and irreversible loss are out of scope.
- Do not duplicate route-finding in recovery code, build a full calendar, or redesign the entire save system.
- Keep recovery and payment rules outside UI handlers.

## Acceptance criteria

- Knocked-out companions remain unavailable for combat and new expeditions across save/load.
- The nearest reachable treatment-capable settlement is selected with documented deterministic tie-breaking.
- No-route handling is explicit and tested.
- Recovery cannot be bypassed through menu/reload exploits.
- Time-based recovery and/or paid treatment works as specified with no partial charging.
- Recovered companions become selectable only under legal party rules.
- Defeat/retreat/total-party-defeat outcomes are defined without duplicate rewards or unrelated progress corruption.
- No companion can suffer permanent death.

## Section-specific verification

Test state transitions, route selection/tie-breaking, no reachable settlement, save/load, duplicate processing, failed payment, and defeat outcomes. Confirm all CI jobs pass on the PR head before handoff.
