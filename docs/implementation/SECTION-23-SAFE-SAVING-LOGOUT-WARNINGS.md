# Section 23 — Safe-Location Saving and Unsaved-Progress Warnings

**Status:** Implemented on PR #29. **Dependencies:** Sections 21–22 and the existing save/authentication layer.

## Checkpoint eligibility

A normal gameplay checkpoint is available only when the current location explicitly provides LocationService::Tavern or LocationService::Guildhouse. Settlement category, a bed, a field camp, a healer or a generic location does not imply save permission.

The Section 22 Oakhaven Village definition already declares both Tavern and Guildhouse, so it is the first usable checkpoint. Other locations remain ineligible until their service data explicitly opts them in.

The checkpoint transaction checks both explicit service eligibility and that the supplied location ID matches GameState.current_location. Account creation and the explicit legacy authentication migration remain trusted storage operations rather than gameplay checkpoint actions.

## Session dirtiness

CheckpointSession fingerprints the canonical persisted GameState payload using the save layer's checksum representation. Before fingerprinting it applies the same resource normalisation as save_to_path. The password hash remains outside the payload, as before.

At the start of each login session the current saved payload becomes the baseline. Every main-menu pass compares the live payload with that baseline. This catches persistent mutations from quests, travel, combat, shops, crafting, party management, companion recovery, calendar advancement and startup reconciliation without requiring each subsystem to remember to toggle a dirty flag.

The menu title displays UNSAVED when the live persisted payload differs. The Save Checkpoint option is only shown at an eligible location. The main-menu-loop autosave and automatic logout save have been removed.

A successful atomic write is the only operation that advances the session baseline. A location rejection, serialisation error, authentication/write error or failed storage operation leaves the session dirty.

## Logout behaviour

When the session is clean, logout exits without writing a new gameplay checkpoint.

When unsaved progress exists, the player can:
- Return to the game without changing the checkpoint.
- Save and log out when currently at a tavern or guildhouse.
- Travel along the shortest reachable route to the nearest eligible checkpoint when one is reachable, then return to the game so the player can save explicitly.
- Explicitly discard unsaved progress and log out. The live state is dropped without invoking the save path; the next login loads the last successful checkpoint.

If no safe location is reachable, the travel option is omitted. A failed save leaves the player in the session with the UNSAVED marker intact.

## Compatibility and failure model

The save envelope remains version 2. Newly introduced Section 24 class/attribute fields use serde defaults, and the existing raw-payload checksum validation still checks the data as it was written before applying those defaults. Existing authentication metadata remains separate from gameplay state.

Legacy password migration continues to require the explicit consent flow already present. Its write goes through the existing atomic save path, which preserves a valid pre-migration file as the backup before replacing the active file. The safe-location restriction is for normal gameplay checkpoints, not authentication/account maintenance.

An unexpected process termination, terminal crash, OS crash or power loss cannot present an interactive warning. Unsaved gameplay progress since the last successful checkpoint is therefore lost. This implementation deliberately does not disguise that limitation with a hidden autosave or cloud/recovery subsystem.

## Validation

Tests cover explicit service eligibility, location-ID matching, complete-payload dirtiness, successful baseline advancement, failed/ineligible save behaviour and the save-layer persistence/migration compatibility paths. Manual smoke testing should include save at Oakhaven, a gameplay action, logout/discard, and login to confirm the previous checkpoint is loaded.