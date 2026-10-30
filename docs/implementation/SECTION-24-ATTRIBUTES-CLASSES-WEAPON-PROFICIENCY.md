# Section 24 — Attributes, Class-Weighted Growth and Weapon Proficiency

**Status:** Implemented on PR #29. **Dependencies:** Existing party-combat, player state, equipment and save systems.

## Primary attributes

The persistent primary attributes are Strength, Dexterity, Constitution, Intelligence, Wisdom and Charisma. Each is clamped to 1–20 and starts at 10. They are stored separately from derived combat values (speed, accuracy, evasion, power and resistance).

The player combat profile translates primary attributes into derived values before applying the existing class, equipment and mastery modifiers. The current conversion is:

| Derived value | Primary-attribute contribution |
|---|---|
| Speed | 30 + 2 × Dexterity |
| Accuracy | 30 + Dexterity + Wisdom |
| Evasion | 30 + 2 × Dexterity |
| Power | 30 + 2 × Strength (Warrior/Barbarian), Dexterity (Ranger/Rogue) or Intelligence (Mage) |
| Resistance | 30 + Constitution + Wisdom |

Each Constitution point assigned during level-up also adds 5 maximum HP and 2 maximum stamina; the newly available capacity is filled at that level-up. Constitution additionally contributes to derived resistance through the formula above. Charisma remains a persistent social/leadership attribute for the future dialogue and access systems; it does not silently inflate combat statistics. Equipment resistance and mastery are additive sources applied once after the primary-attribute conversion. The default attribute set (10 in every stat) therefore produces a neutral 50-point base for all five derived combat values before class modifiers.

## Classes and automatic growth

The player chooses one durable class when creating an account. The five classes are Warrior, Barbarian, Mage, Ranger and Rogue. Existing combat-side class modifiers remain in place; Barbarian receives its own modifier profile. Legacy saves without a class field default deterministically to Warrior.

Every combat-level increase grants one primary-attribute point. The persisted growth weights use Strength, Dexterity, Constitution, Intelligence, Wisdom and Charisma order:

| Class | STR | DEX | CON | INT | WIS | CHA |
|---|---:|---:|---:|---:|---:|---:|
| Warrior | 30 | 15 | 25 | 5 | 10 | 15 |
| Barbarian | 32 | 12 | 30 | 5 | 11 | 10 |
| Mage | 5 | 10 | 10 | 35 | 25 | 15 |
| Ranger | 10 | 30 | 15 | 10 | 25 | 10 |
| Rogue | 10 | 35 | 10 | 15 | 10 | 20 |

Every row totals 100. Growth uses a deterministic smooth weighted accumulator stored in the save, not random rolls. Stable attribute order resolves ties. Attributes already at 20 are removed from the eligible pool and their weights are renormalised over the remaining attributes, so every available level-up point is redistributed. If every attribute is capped, no additional point is allocated.

The optional elective point every fifth level was not adopted. Multiclassing, respec costs and talent trees remain out of scope.

## Weapon families and proficiency

WeaponFamily is distinct from the historical WeaponTier enum. A WeaponProfile exposes both family and tier/quality without changing existing serialized inventory values, stable tier variants, prices or item-property methods. Existing inventory entries resolve to their established identities:

- Bare Hands → Unarmed
- Omega Maul → Mace
- Chaotic Spear → Spear
- The remaining existing tier items retain their established sword/blade identity.

Axe, dagger, bow, staff, wand and heavy-weapon families are defined for future weapon data. Their definitions do not invent new store stock or alter current item prices.

Class familiarity currently grants a bounded 0–12% damage bonus for a matching weapon family. Warrior favours swords, maces and spears; Barbarian favours axes, maces and heavy weapons; Mage favours staffs and wands; Ranger favours bows, spears and daggers; Rogue favours daggers, light swords and bows. The bonus is applied to the player's combat attack value and never determines whether an item can be equipped. Existing equipment menus and inventory identities remain unchanged.

## Compatibility and validation

The save envelope remains version 2. Legacy players receive Warrior, six attributes at 10 and a zeroed growth accumulator through serde defaults. The class, attributes and accumulator survive save/load.

Tests cover 100-point class weight totals, deterministic weighted distribution, cap redistribution, level-up allocation, attribute-to-combat conversion without double counting, family/tier mapping, bounded proficiency, save/load persistence and legacy defaults. The current proficiency bonuses are tuning values and can be adjusted without changing the save schema.