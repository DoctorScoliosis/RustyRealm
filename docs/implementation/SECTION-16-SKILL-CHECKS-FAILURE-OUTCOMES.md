# Section 16 — Skill Checks and Failure Outcomes

**Status:** Implemented — review via the Section 16 pull request  
**Dependencies:** Existing skills, crafting, stamina, inventory/economy, market turn progression and companion recovery.

## Implementation summary

- Add `src/skill_checks.rs`, a UI-independent bounded probability model.
- Formula: `clamp(40 + floor(min(skill_level, 100) / 2) - difficulty + modifier, 10, 95)` percent.
- A controlled integer roll in `0..=99` succeeds when `roll < success_chance`; invalid rolls are rejected.
- Skill level and fatigue affect probability. Fatigue is calculated after paying the action stamina cost: 75–100% stamina = no penalty; 50–74% = −5 points; 25–49% = −15; 0–24% = −25.
- Checks are integrated into Woodcutting, Mining, Fishing, Cooking, Thieving and all current crafting recipes. Prerequisites are validated before spending stamina, advancing time, or consuming ingredients.

## Failure and resource policy

| Action | Difficulty | Failure outcome |
|---|---:|---|
| Woodcutting | 25 | No logs or XP; stamina and world time are still spent |
| Mining | 30 | No ore or XP; stamina and world time are still spent |
| Fishing | 25 | Bait is consumed; no fish or XP |
| Cooking | 15 for an ordinary recipe; 10 for mastery batches | Raw food is consumed/spoiled; no food or XP |
| Thieving | 25 | No gold, thread or XP; stamina and world time are still spent |
| Smelt ingot | 25 | Ore is consumed and converted to Metal Scraps |
| Forge iron sword | 35 | Ingots are consumed and converted to Metal Scraps |
| Recover ingot from scraps | 50 | The scrap is consumed; no ingot or replacement scrap |
| Other recipes | Recipe-specific; see `src/crafting.rs` | Ingredients are consumed; no product or recipe XP |

Smithing failures return Metal Scraps based on the effective metal input quantity after the existing Master Artisan discount. The recovery recipe consumes one scrap per attempt and has a lower, skill-dependent success rate. A failed recovery attempt does not return another scrap, preventing an unlimited retry loop without new materials.

At maximum skill, gathering and processing still benefit from mastery but are bounded to manageable attempts: Woodcutting and Mining's large-yield actions cost 20 stamina and five world turns; Cooking, Fishing and Smithing process batches of at most 50 units, with stamina and world-turn costs increasing with batch size. Other checked recipes have explicit stamina costs and turn costs.

## World time and rest

- `GameState.world_turn` is persisted with a default for old saves and is visible in the interface.
- Every attempted skill action advances world time, including failures. Existing market-volatility decay and companion recovery progress on each world turn.
- Main-menu actions continue to advance a turn. Skill sessions and rest manage their own turns so returning from those menus does not add an extra turn.
- Rest is a meaningful trade-off: at a settlement, an inn costs 25 gold, restores stamina and up to 25 HP, and advances two turns; camping with food consumes one Food for 50 stamina and up to 10 HP over two turns; sleeping rough restores 20 stamina over four turns.
- Each skill attempt pays stamina before rolling. As stamina drops, later checks become less reliable, encouraging players to take shorter work sessions and plan food, gold and salvage.

## Transaction policy

Recipe attempts validate skill level, ingredient quantities, stamina and potential output capacity before committing. A successful attempt commits inputs, outputs, XP and stamina together. A failed attempt commits its documented costs and failure products together; validation errors leave the player unchanged. Direct tests can inject the roll to deterministically cover both paths.

## Tests

Coverage includes low/medium/high skill checks, bounds, fatigue modifiers, controlled-roll boundaries, invalid rolls, consuming crafting failures, smithing scrap recovery, insufficient-stamina atomicity, world-turn persistence and backward compatibility for saves without the new time field.

## Scope boundaries

Equipment, class abilities and environmental modifiers are not applied unless a specific relevant modifier has been audited. Failed recipes other than Smithing currently consume their ingredients without producing an alternate by-product; additional skill-specific salvage rules can be added in later sections.
