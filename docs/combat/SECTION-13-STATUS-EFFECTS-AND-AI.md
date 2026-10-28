# Section 13 — Status Effects, Enemy Archetypes, and Combat AI

## Status lifecycle

The typed implementation lives in `src/status_combat.rs`. Effects are encounter-local and share one representation for players, companions, and enemies in `party_combat::EncounterCombatant`.

- Application occurs only after validation. Duration must be greater than zero and is capped at 12 activations; potency is capped at 100; base chance is capped at 100%.
- Resistance is a separate status resistance percentage, capped at 90%. Effective chance is `floor(base_chance * (100 - resistance) / 100)`. The caller provides a roll in 0–99; the status applies when the roll is below effective chance.
- Each effect of a given kind is unique on a combatant. Reapplication refreshes duration to the greater remaining duration, potency to the greater potency, and source to the new non-empty source. It does not stack damage.
- A newly applied effect does not tick during the action that applied it. At the start of the affected combatant's next activation, Poison, Bleed, and Burn each deal potency damage, and Stun prevents the action. Duration then decreases by one; zero-duration effects expire immediately after that activation's tick. Effects applied by an actor to another combatant therefore wait for that target's next activation.
- Slow reduces initiative by 10 per potency tier, capped at 20. Mark adds 5 hit-chance percentage points per potency tier, capped at 15. WeakenedDefences increases the attack's pre-defence damage by 25% per potency tier, capped at 50%. These effects never change values outside their caps.

The prior `data::StatusEffect` shape is retained for serialization and the legacy single-player battle flow. The new shared party-combat lifecycle is the canonical model for Section 13 party encounters. Legacy combat's existing Poison/Bleed/Burn/Stun tick path remains in place until that UI flow is migrated; this transitional compatibility is not claimed to be a complete unification of the legacy path.

## AI policy

`EnemyProfile::for_name` maps existing roster names to four tactical archetypes. The policy does not add entries to the random encounter pool; the allow-list in `combat.rs` is unchanged. `propose_enemy_action` returns a structured intended action and optional telegraph and never mutates encounter state. All returned actions must still be submitted to `Encounter::apply_action`.

The random choice/tie-breaking input is a caller-provided integer, so tests can repeat decisions exactly. Legal target and range rules are read from Section 12 encounter methods rather than duplicated as a separate validator.

## Telegraphs

A telegraph stores the exact actor, target, intended action, name, description, damage channel and stated damage. Resolution first checks that actor and target remain eligible and hostile, and then calls the shared action resolver. Missing, knocked-out, allied, out-of-range, or already-acted targets/actions cancel the telegraph explicitly rather than mutating the encounter.

## Compatibility boundary

Stronghold's original story route, three guardian archetypes, and Lacoyx's three legacy phases remain untouched in the legacy battle flow. They are not yet driven by the new party-combat encounter API. Section 14 recovery/rescue is not implemented here.
