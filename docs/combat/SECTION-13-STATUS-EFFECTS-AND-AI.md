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

`EnemyProfile::for_name` maps existing roster names to four tactical archetypes. The policy does not add entries to the random encounter pool; the allow-list in `combat.rs` is unchanged. Ranged/status-focused profiles are assigned a ranged attack profile, and `propose_enemy_action` uses Section 12's legal-target query before choosing an attack; if no attack target is legal it proposes movement. It returns a structured intended action and optional telegraph and never mutates encounter state. All returned actions must still be submitted to `Encounter::apply_action`. Enemy status payloads are applied by that shared resolver only after a successful hit and use the same target resistance/application path as other effects.

The random choice/tie-breaking input is a caller-provided integer, so tests can repeat decisions exactly. Legal target and range rules are read from Section 12 encounter methods rather than duplicated as a separate validator.

## Telegraphs

A telegraph stores the exact actor, target, intended action, name and description. It does not promise a fixed damage number: the committed action resolves later through Section 12's common resolver, including its then-valid range, hit, critical, defence and resource rules. Resolution first checks that actor and target remain eligible and hostile, then calls the shared action resolver. Missing, knocked-out, allied, out-of-range, stunned or already-acted actors/actions cancel the telegraph explicitly rather than mutating the encounter.

## Compatibility boundary

Compatibility verification covers the three Stronghold guardian profiles (Mage/status-focused, Ranger/ranged-pressure, Warrior/aggressive-melee), Lacoyx's three HP-threshold phase transitions, the legacy battle's shared status tick path, and resolution of an already-committed Lacoyx telegraph exactly once. The existing terminal battle flow already presents a defensive response for Lacoyx telegraphs and then resolves the stored attack. However, ordinary legacy encounters still use the legacy basic-attack loop; the new four-archetype `propose_enemy_action` policy is exposed through the Section 11/12 party-combat API and is not yet wired into that separate terminal loop. This is a known integration boundary, not a claim that every terminal encounter uses the new AI. Section 14 recovery/rescue is not implemented here.
