# Section 20 — Realm Progression, Completion and the Lacoyx Gate

**Status:** Implemented on \`work/section-20-realm-progression\`; review pending.  
**Plan:** \`DoctorScoliosis/ProjectPlanDump/RustyRealm/SECTION-20-REALM-PROGRESSION-COMPLETION-LACOYX-GATE.md\`

## Campaign registry

\`src/realm_progression.rs\` defines the stable-ID realm registry and objective definitions. Each realm records discovery, completed objective IDs, mandatory completion and optional mastery separately. The Stronghold gate iterates the registry's \`required_for_lacoyx\` flag; it does not use a realm count or a list embedded in the gate function.

The planned mandatory sequence and entry contracts are:

| Realm ID | Entry location ID | Prerequisite realm |
| --- | --- | --- |
| \`oakhaven\` | \`village_start\` | — |
| \`thornveil_wilds\` | \`thornveil_entry\` | \`oakhaven\` |
| \`ironvein_marches\` | \`ironvein_entry\` | \`thornveil_wilds\` |
| \`drowned_reach\` | \`drowned_reach_entry\` | \`ironvein_marches\` |
| \`ashen_march\` | \`ashen_march_entry\` | \`drowned_reach\` |

The four later realm definitions reserve objective IDs for their future main quest, signature encounter, optional landmark and optional mastery task. They remain undiscovered until their entry locations and prerequisite content are implemented in the corresponding atlas/lore/quest sections. They cannot be completed merely by visiting Oakhaven.

## Oakhaven evidence

Oakhaven deliberately reuses Section 19's existing content and save evidence:

- Required: complete the \`clear_the_ruins\` stage of \`oakhaven_missing_supplies\`.
- Required: complete \`oakhaven_missing_supplies\`.
- Required: clear the \`ruins_guardians\` encounter.
- Optional mastery: inspect the Crumbling Watchtower and Abandoned Mill, clear the Whispering Woods patrol and clear the Goblin Caves expedition.

The quest-stage event is emitted only on the transition to a completed stage. Encounter-clear events are emitted only after the existing one-time encounter-clear reward has been committed. Repeating combat or revisiting a location does not complete a campaign objective. Existing Section 19 encounter and quest reward rules remain unchanged.

## Event and persistence flow

\`GameState.campaign_progress\` is serde-defaulted. Old saves therefore load with empty campaign progress without a save-version bump. During login/main-menu initialisation and when the Quest Hall is opened, progression is reconciled from trustworthy persisted evidence:

- completed/claimed quest state and completed quest stages;
- Section 19's persisted encounter-clear IDs;
- Section 19's persisted landmark-interaction IDs.

A missing campaign record does not mean completion. An old completed Missing Supplies quest can reconstruct its Oakhaven quest and stage evidence, but cannot invent a cleared signature encounter when no encounter-clear record exists. Realm task events persist stable objective IDs and are scoped to their realm.

Realm completion requires discovery and every configured mandatory objective. Mastery additionally requires every configured optional objective. Optional objectives never block the Lacoyx gate.

## Gate and player feedback

The classic Stronghold story path is checked at menu selection and again at the public dungeon and boss entry points, preventing callers from bypassing the gate. The Endless Gauntlet remains a separate challenge mode and is not the Lacoyx campaign finale.

The Quest Hall displays discovered realms, mandatory and optional progress, and outstanding gate requirements. Undiscovered realm names and objectives are intentionally withheld. The gate is evaluated from registry definitions, not a hard-coded realm total.

## Boundaries

This section does not add future realm locations, questlines, lore, calendar/rest/save restrictions, class progression, access-prerequisite evaluation or Lacoyx boss phases. Those remain in their planned sections. Future content should emit the existing typed quest, encounter, landmark or realm-task events and use the reserved stable IDs above.
