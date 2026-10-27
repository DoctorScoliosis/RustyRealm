# Section 15 — Mouse-Enabled Terminal Interface

**Status:** Planned
**Dependencies:** Existing Ratatui/Crossterm UI; party and combat screens can adopt mouse support as their interaction regions are added.

## Objective

Allow mouse interaction with visible menus, party members, combat targets, and actions while preserving all existing keyboard controls and a usable keyboard-only fallback.

## Implementation slices

### 15.1 — Terminal lifecycle and event handling

- Add Crossterm mouse event handling and enable mouse capture only while the application owns the terminal.
- Ensure mouse capture, raw mode, cursor state, alternate screen, and other terminal state are restored on normal exit and recoverable errors.
- Integrate mouse events with the existing event loop without starving keyboard input or making the UI unresponsive.

### 15.2 — Coordinate-to-action mapping

- Derive interactive regions from the same layout and visible item ranges used to render the current screen.
- Support clicking menu options, party selections, combat targets, and available actions.
- Handle resizing and scrolling so hit regions track the current rendered positions rather than stale coordinates.
- Reject clicks on disabled or unavailable actions using the same domain validation as keyboard-triggered actions.

### 15.3 — Feedback and fallback

- Visually distinguish selected, disabled, unavailable, and actionable entries.
- Explain relevant unavailability reasons, such as insufficient resources or an out-of-range target.
- Preserve keyboard navigation, shortcuts, and Escape/back behaviour.
- On unsupported terminals, retain a usable keyboard-only experience and avoid failing application startup solely because mouse support is unavailable.

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
