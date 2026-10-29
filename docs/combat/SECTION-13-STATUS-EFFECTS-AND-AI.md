# Section 13 — Status Effects, Enemy Archetypes, and Combat AI

## Status lifecycle

The typed implementation lives in `src/status_combat.rs`. Effects are encounter-local and share one representation for players, companions, and enemies in `party_combat::EncounterCombatant`.

- Application occurs only after validation. Duration must be greater than zero and is capped at 12 activations; potency is capped at 100; base chance is capped at 100%.
- Resistance is a separate status resistance percentage, capped at 90%. Effective chance is `floor(base_chance * (100 - resistance) / 100)`. The caller provides a roll in 0–99; the status applies when the roll is below effective chance.
- Each effect of a given kind is unique on a combatant. Reapplication refreshes duration to the greater remaining duration, potency to the greater potency, and source to the new non-empty source. It does not stack damage.
- A newly applied effect does not tick during the action that applied it. At the start of the affected combatant's next activation, Poison, Bleed, and Burn each deal potency damage, and Stun prevents the action. Duration then decreases by one; zero-duration effects expire immediately after that activation's tick. Effects applied by an actor to another combatant therefore wait for that target's next activation.
- Slow reduces initiative by 10 per potency tier, capped at 20. Initiative is recomputed when a round closes, so a Slow applied mid-round affects subsequent round ordering. Mark adds 5 hit-chance percentage points per potency tier, capped at 15. WeakenedDefences increases the attack's pre-defence damage by 25% per potency tier, capped at 50%. These effects never change values outside their caps.

Both the legacy single-player battle and the Section 11/12 party-combat domain now use the same typed StatusEffect representation, application/refresh policy, duration clock and tick function. The encounter API guards against ticking the same actor twice in a single activation. The legacy battle UI keeps its own presentation and action loop, but it no longer has a competing status-effect storage or tick implementation. The party-combat domain additionally consumes Slow, Mark and WeakenedDefences in initiative, hit-chance and damage calculations.

## AI policy

`EnemyProfile::for_name` maps the existing roster to four tactical archetypes and is shared by both combat paths. The policy does not add entries to the random encounter pool; the allow-list in `combat.rs` is unchanged.

In party encounters, `propose_enemy_action` uses Section 12's legal-target query and returns a structured `CombatAction` plus an optional exact-action telegraph. It does not mutate encounter state; the resulting action is passed to `Encounter::apply_action` for validation and mutation.

In the legacy terminal battle, `CombatEngine::propose_legacy_enemy_intent` consumes the same profile and returns a typed intent. Ordinary enemies now select a profile-specific telegraph tendency and carry their status payload in the committed telegraph. The battle loop presents its existing defensive-response menu, resolves the stored attack, and applies that payload through the shared status lifecycle only when the attack connects. Application chance is then combined with the target's bounded status resistance. Lacoyx retains its bespoke phase-specific telegraphs rather than using an ordinary archetype.

The random inputs to both domain policies are controllable in tests. The legacy runtime supplies random rolls, while unit tests call the deterministic intent helper directly. Section 12 remains the legal-target/range authority for party combat; the legacy single-enemy battle keeps its existing damage and defence resolver rather than pretending it is a multi-combatant `Encounter`.

## Telegraphs

Party-combat telegraphs store the exact actor, target and `CombatAction`; resolution revalidates the actor, target, range and action through Section 12's resolver. Missing, knocked-out, allied, stunned or already-acted actors and targets cancel without mutation. The legacy battle commits the exact damage type, base damage, display name and optional status payload together in `TelegraphedAttack`, so warning and resolution cannot drift apart. The player's existing defensive-response choice is preserved, and an enemy stunned at its next activation has its pending telegraph cancelled before damage resolution.

## Compatibility boundary

Compatibility tests cover all three existing Stronghold guardian choices and levels, their distinct archetype/status policies, Lacoyx's three HP-threshold transitions and phase-specific telegraph names/damage types, single-resolution telegraphs, and Stun interruption. Stronghold still enters Lacoyx after the existing ten guardian victories; those encounters continue to use the legacy terminal presentation, now with the shared status lifecycle and profile-driven ordinary-enemy intent. Section 14 recovery/rescue is not implemented here.
