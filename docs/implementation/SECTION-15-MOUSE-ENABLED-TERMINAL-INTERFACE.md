# Section 15 — Mouse-Enabled Terminal Interface

**Status:** In review — PR #21
**Dependencies:** Existing Ratatui/Crossterm UI, party domain model, and party-aware combat engine; Section 14 is merged into `main`.

## Objective

Allow mouse interaction with visible menus, party members, combat targets, and actions while preserving keyboard controls and supporting terminals that accept only whole text lines rather than live key events.

## Implementation slices

### 15.1 — Terminal lifecycle and event handling

- Add Crossterm mouse event handling and enable mouse capture only while the application owns the terminal.
- Ensure mouse capture, raw mode, cursor state, alternate screen, and other terminal state are restored on normal exit and recoverable errors.
- Treat mouse capture as optional; continue with keyboard events when mouse capture itself is unsupported.
- Fall back to line-input mode when standard input is not a terminal or raw-mode setup fails.

### 15.2 — Coordinate-to-action mapping

- Derive interactive regions from the same layout and visible item ranges used to render the current screen.
- Support clicking menu options, party selections, combat targets, and available actions.
- Handle resizing and scrolling so hit regions track the current rendered positions rather than stale coordinates.
- Reject clicks on disabled or unavailable actions using the same domain validation as keyboard-triggered actions.

### 15.3 — Feedback and fallback

- Visually distinguish selected, disabled, unavailable, and actionable entries.
- Explain relevant unavailability reasons, such as insufficient stamina/mana or an out-of-range target.
- Do not return disabled/unavailable choices to domain-action handlers, whether selected by mouse, Enter, number shortcut, or text input.
- Preserve keyboard navigation, shortcuts, and Escape/back behaviour in event-driven mode.
- Retain a usable line-input experience on wrappers where per-key event monitoring is unavailable.

### 15.4 — Line-input fallback

- Select line-input mode automatically when stdin is not a terminal or event-driven raw-mode setup fails.
- Allow wrappers that cannot deliver live key events to opt in with `RUSTYREALM_INPUT_MODE=text`.
- Render menus as numbered choices; accept a number or exact label and confirm with Enter. Text prompts, messages, and long text screens use line input and Enter-to-continue.
- Keep the same disabled/unavailable choice checks in line-input mode so text submissions cannot bypass action gating.
- Warn that masked prompts cannot hide characters in line-input mode.

## Scope boundaries

- Keep the application within the existing terminal UI; do not introduce a separate graphical frontend.
- Mouse clicks must not bypass domain validation.
- Avoid hard-coded coordinates that break when terminal dimensions change.

## Acceptance criteria

- Supported terminal emulators can select visible supported controls with the mouse.
- Existing keyboard workflows continue to work.
- Hit regions remain correct after resize and scrolling.
- Disabled actions cannot be executed by clicking.
- Terminal state is restored after normal exits and recoverable failures.
- Keyboard-only operation remains possible when mouse input is unavailable.
- Line-input-only wrappers can operate using numbered choices and Enter-confirmed lines.
- Disabled/unavailable options expose a reason and cannot be returned as actionable choices in any supported input mode.
