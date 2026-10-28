# Section 15 — Mouse-Enabled Terminal Interface

**Status:** Planned
**Dependencies:** Existing Ratatui/Crossterm UI; party and combat screens can adopt mouse support as their interaction regions are added.

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

Allow mouse interaction with visible menus, party members, combat targets, and actions while preserving every existing keyboard workflow and a usable keyboard-only fallback.

## Implementation slices

### 15.1 — Terminal lifecycle and event handling

- Add Crossterm mouse event handling and enable mouse capture only while the application owns the terminal.
- Ensure mouse capture, raw mode, cursor state, alternate screen, and other terminal state are restored on normal exit and recoverable errors.
- Integrate mouse events with the existing event loop without starving keyboard input or making the UI unresponsive.

### 15.2 — Coordinate-to-action mapping

- Derive interactive regions from the same layout and visible item ranges used to render the current screen.
- Support clicking menu options, party selections, combat targets, and available actions.
- Handle resize and scrolling so hit regions follow current rendered positions rather than stale coordinates.
- Reject clicks on disabled/unavailable actions using the same domain validation as keyboard-triggered actions.

### 15.3 — Feedback and fallback

- Visually distinguish selected, disabled, unavailable, and actionable entries.
- Explain reasons an action is unavailable, such as insufficient resources or an out-of-range target.
- Preserve keyboard navigation, shortcuts, and Escape/back behaviour.
- On unsupported terminals, retain keyboard-only operation and do not fail startup solely because mouse support is unavailable.

## Scope boundaries

- Stay within the existing terminal UI; do not introduce a separate graphical frontend.
- Mouse clicks must not bypass domain validation.
- Avoid hard-coded coordinates that break on resize/scroll.
- Do not make prior keyboard controls mouse-only or remove accessibility fallbacks.

## Acceptance criteria

- Supported terminal emulators can select visible supported controls with the mouse.
- Existing keyboard workflows continue to work.
- Hit regions remain correct after resizing and scrolling.
- Disabled actions cannot be executed by clicking.
- Terminal state is restored after normal exit and recoverable failure.
- Keyboard-only use remains possible when mouse input is unavailable.

## Section-specific verification

Test coordinate mapping, resize/scroll behaviour, disabled actions, and event precedence where the code permits. Smoke-test terminal restoration paths. Run the complete CI feedback loop, and leave the implementation PR open for owner review.
