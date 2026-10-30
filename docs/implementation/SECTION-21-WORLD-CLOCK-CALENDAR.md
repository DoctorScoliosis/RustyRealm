# Section 21 — World Clock and Calendar

**Status:** Implemented in PR #29 on \`work/section-20-realm-progression\`.  
**Plan:** \`DoctorScoliosis/ProjectPlanDump/RustyRealm/SECTION-21-WORLD-CLOCK-CALENDAR.md\`

## Time unit and single source of truth

The existing persisted \`GameState.world_turn\` remains the only world clock. One committed world turn is exactly **one in-game hour**. No calendar date, season or second timestamp is stored; all calendar values are derived from \`world_turn\` through \`src/calendar.rs\`. This avoids two persisted clocks drifting apart and requires no save-format version bump.

The legacy epoch is fixed:

- \`world_turn == 0\` → Year 1, Firstbloom 1, Spring, 06:00 (Dawn).
- 24 hours per day.
- 30 days per month.
- 12 months and 360 days per year.
- Three consecutive months per season; four seasons per year.

The provisional month names are Firstbloom, Greenswell, Brightreach, Highsun, Emberwane, Goldleaf, Duskhallow, Frostfall, Longnight, Deepwinter, Thawrise and Newlight. Season names are Spring, Summer, Autumn and Winter. They are centralised in \`CalendarConfig\` and can be replaced later without changing the arithmetic or persisted saves.

The time-of-day labels are Deep Night (00:00–04:59), Dawn (05:00–07:59), Morning (08:00–11:59), Afternoon (12:00–16:59), Dusk (17:00–19:59) and Night (20:00–23:59).

## Time-consuming actions

The clock advances only when a committed gameplay action consumes time. Opening menus, inspecting inventory, browsing shops/Quest Hall/settings, or backing out of an action consumes zero hours.

| Action | Duration |
| --- | --- |
| Direct travel | The existing route \`travel_cost\` value is interpreted as elapsed hours. The Oakhaven route values remain unchanged. |
| Inspect/interact with current location | 1 hour after a valid interaction |
| Ordinary gathering action | 1 hour |
| Mastery gathering action | 5 hours for Woodcutting, Mining and Thieving; batch Cooking, Fishing and Smithing use one hour per five processed units, rounded up, with at least one hour |
| Crafting | Existing recipe durations are interpreted as hours: 1–3 hours depending on recipe |
| Combat encounter | 1 hour per resolved encounter; tactical rounds remain sub-turns inside that hour |
| Inn rest | 2 hours |
| Food camp | 2 hours |
| Rough sleep | 4 hours |
| Companion recovery | 3 elapsed hours when a valid treatment destination exists |

Gathering and crafting durations apply on both success and attempted failure, matching their existing stamina/resource-loss rules. Invalid or cancelled actions do not consume time. Companion recovery is passive: any other committed elapsed time advances its remaining duration.

The route-cost field name is retained for compatibility with the existing world graph, but its documented meaning is now travel duration in hours. Direct-route validation still happens before location or time is mutated.

## Central elapsed-time advancement

\`GameState.advance_world_time(hours)\` is the central elapsed-time operation. A zero-hour request is a strict no-op. The old \`advance_world_turn()\` remains as a one-hour compatibility wrapper for existing callers.

Market volatility and companion recovery receive the elapsed duration in one batch:

- Market pressure still decays once per five elapsed hours. Batched decay is equivalent to repeated one-hour advancement.
- Companion recovery subtracts elapsed hours from records with a valid destination. Records without a destination do not silently recover.
- When the world clock approaches \`u64::MAX\`, it saturates. Dependent systems receive only the number of hours that could actually be represented, not the requested amount beyond the limit.

## Calendar helpers and UI

\`calendar_at\` and \`format_calendar\` derive the current date, month, season, hour and time-of-day label. \`CalendarDateTime.is_within_hours\` and \`is_hour_in_window\` support future time-window gates, including windows crossing midnight. Windows are half-open (\`[start, end)\)); invalid endpoints and equal endpoints match no hours.

The main menu, Skill Plot, skill detail and rest surface show the in-game date/season/time. The game clock never reads the host system clock and never advances simply because the game remains open.

## Persistence and compatibility

No calendar fields are added to the save. Existing \`world_turn\` values map deterministically from the documented epoch; saves missing that already-defaulted field continue to map to the epoch. Market-state and companion-recovery serialised fields retain their existing names and serde shape. Calendar configuration is code-owned and replaceable, not player-save state.

## Validation

Acceptance coverage includes the epoch, midnight/month/season/year boundaries, custom names, normal and overnight hour windows, zero-duration no-op, market decay over elapsed hours, companion recovery over elapsed hours, serde round-trip/legacy defaults and saturation at clock overflow. Travel integration verifies the existing route value advances the clock by that many hours.

## Section 22 handoff
Section 22 has now implemented location service tags, safe-haven short/long rests and explicit field camping. It continues to use `advance_world_time`; no parallel clock or persisted status timer was introduced.

