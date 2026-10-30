# Section 26 — Exploration Requirements and Access Prerequisites

**Status:** Implemented in PR #29 on \`work/section-20-realm-progression\`.

## Declarative evaluator
\`src/access.rs\` defines typed \`AccessRequirement\`, \`AccessGate\` and \`AccessResult\` structures. Supported checks include:
- Realm discovery and mandatory completion.
- Quest and individual quest-stage completion.
- Known landmark interaction flags.
- Item quantities, primary attribute thresholds and skill thresholds.
- Companion discovery, recruitment and active-party membership.
- In-game hour windows (including windows crossing midnight) and seasons.
- All-of and any-of groups plus item costs checked before entry and committed atomically only after success.

Unknown realm, quest, stage and landmark IDs fail closed. Invalid thresholds, invalid time windows and zero item quantities fail closed. Denial reasons do not reveal hidden realm or companion names.

## Domain enforcement
- Locations carry optional access gates and a hidden-while-locked flag.
- Directed routes carry independent optional access gates. Graph reachability remains separate from permission to enter.
- Navigation hides destinations marked hidden while locked and explains unmet requirements for visible locked destinations.
- Direct travel re-evaluates both route and destination gates, preventing callers from bypassing a menu restriction.
- Arrival events, world-time advancement and item costs are staged together; the transition is committed only when the full entry succeeds.
- \`QuestStage.access_gate\` is optional and defaults open for legacy saves. Stage events cannot progress a currently gated stage before its conditions pass; Quest Hall displays the reason.

## Compatibility and validation
Access definitions are content data, not player state, so the save envelope/version is unchanged. Tests cover typed requirements, unknown references, any-of/all-of logic, atomic costs, direct route and location enforcement, stage gates, and side-effect-free evaluation.
