# Section 18 — Risk–Reward Balancing and Combat Simulation

**Status:** Planned
**Dependencies:** Combat, classes, effects, skill checks, recovery, quests, and economy systems to be evaluated as they stabilise.

## Objective

Create repeatable balance measurements for enemies, classes, equipment, skills, quests, and rewards before substantially expanding realm content.

## Implementation slices

### 18.1 — Establish baseline scenarios and targets

- Audit enemy health, damage, speed, resistances, abilities, drops, and reward tables.
- Define representative character/party builds and encounters for low, middle, and high progression.
- Document expected difficulty ranges, target turn counts, acceptable incoming damage, healing/consumable use, and intended recovery cost.
- Treat target values as explicit design assumptions that can be revised with evidence.

### 18.2 — Repeatable simulation

- Build deterministic or seed-controlled combat simulations that invoke the same domain rules as real encounters.
- Measure turns to defeat, party damage taken, knockout frequency, abilities/resources used, and encounter outcomes.
- Compare class and party matchups against representative enemy archetypes.
- Reassess Stronghold Gauntlet scaling, escalating rewards, and modifiers.

### 18.3 — Progression and economy

- Review gold, XP, loot, crafting requirements, shop prices, skill-check curves, consumable supply, and recovery costs together.
- Look for repeated-action, quest-claim, sale/buyback, Gauntlet, and other reward exploits.
- Add regression thresholds for material balance shifts; document intentional changes instead of relying on vague intuition.

## Scope boundaries

- Do not tune by undocumented intuition alone.
- Do not overfit to a single character build or seed.
- Keep simulations out of interactive gameplay paths and avoid making production play slower.
- Balance changes must not silently alter save interpretation or quest progress.

## Acceptance criteria

- Representative encounters have documented expected difficulty and reward ranges.
- Repeatable simulations compare relevant classes/parties and enemy archetypes.
- Classes have useful strengths and trade-offs without one universally dominating.
- Higher-risk encounters offer proportionate rewards without trivialising progression.
- Gauntlet difficulty and rewards remain worthwhile over its progression.
- Regression checks flag significant unintended changes in combat and economy metrics.
