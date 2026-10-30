//! Persistent character class, primary attributes and deterministic growth.

use serde::{Deserialize, Serialize};

use crate::data::WeaponFamily;

/// Durable player class, separate from temporary encounter state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterClass {
    Warrior,
    Barbarian,
    Mage,
    Ranger,
    Rogue,
}

impl Default for CharacterClass {
    fn default() -> Self {
        Self::Warrior
    }
}

impl CharacterClass {
    pub const ALL: [Self; 5] = [
        Self::Warrior,
        Self::Barbarian,
        Self::Mage,
        Self::Ranger,
        Self::Rogue,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Warrior => "Warrior",
            Self::Barbarian => "Barbarian",
            Self::Mage => "Mage",
            Self::Ranger => "Ranger",
            Self::Rogue => "Rogue",
        }
    }

    /// Weights use Strength, Dexterity, Constitution, Intelligence, Wisdom,
    /// Charisma order. Every class totals exactly 100.
    pub const fn growth_weights(self) -> [u8; 6] {
        match self {
            Self::Warrior => [30, 15, 25, 5, 10, 15],
            Self::Barbarian => [32, 12, 30, 5, 11, 10],
            Self::Mage => [5, 10, 10, 35, 25, 15],
            Self::Ranger => [10, 30, 15, 10, 25, 10],
            Self::Rogue => [10, 35, 10, 15, 10, 20],
        }
    }

    /// Class familiarity is a bounded damage bonus, not an equipment lock.
    pub const fn weapon_proficiency_pct(self, family: WeaponFamily) -> u8 {
        use WeaponFamily::*;
        match (self, family) {
            (Self::Warrior, Sword) => 12,
            (Self::Warrior, Mace) => 10,
            (Self::Warrior, Spear) => 8,
            (Self::Warrior, Axe | Heavy) => 5,
            (Self::Warrior, Unarmed) => 3,

            (Self::Barbarian, Axe | Heavy) => 12,
            (Self::Barbarian, Mace) => 10,
            (Self::Barbarian, Spear) => 6,
            (Self::Barbarian, Sword) => 5,
            (Self::Barbarian, Unarmed) => 3,

            (Self::Mage, Staff | Wand) => 12,
            (Self::Mage, Sword | Dagger) => 2,
            (Self::Mage, Unarmed) => 3,

            (Self::Ranger, Bow) => 12,
            (Self::Ranger, Spear) => 8,
            (Self::Ranger, Dagger) => 6,
            (Self::Ranger, Sword) => 4,
            (Self::Ranger, Unarmed) => 3,

            (Self::Rogue, Dagger) => 12,
            (Self::Rogue, Bow) => 8,
            (Self::Rogue, Sword) => 8,
            (Self::Rogue, Spear) => 3,
            (Self::Rogue, Unarmed) => 4,

            _ => 0,
        }
    }
}

/// Primary attributes use a 1..=20 scale and stay distinct from derived
/// combat values such as speed, accuracy, power and resistance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimaryAttributes {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl Default for PrimaryAttributes {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl PrimaryAttributes {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 20;

    pub const fn values(self) -> [u8; 6] {
        [
            self.strength,
            self.dexterity,
            self.constitution,
            self.intelligence,
            self.wisdom,
            self.charisma,
        ]
    }

    pub fn total(self) -> u16 {
        self.values().into_iter().map(u16::from).sum()
    }

    pub fn cap_values(&mut self) {
        self.strength = self.strength.clamp(Self::MIN, Self::MAX);
        self.dexterity = self.dexterity.clamp(Self::MIN, Self::MAX);
        self.constitution = self.constitution.clamp(Self::MIN, Self::MAX);
        self.intelligence = self.intelligence.clamp(Self::MIN, Self::MAX);
        self.wisdom = self.wisdom.clamp(Self::MIN, Self::MAX);
        self.charisma = self.charisma.clamp(Self::MIN, Self::MAX);
    }

    /// Allocate exactly one level-up point with smooth weighted round-robin.
    /// Capped attributes leave the eligible pool; the remaining weights are
    /// renormalised for that point so growth is redistributed.
    pub fn add_weighted_point(
        &mut self,
        class: CharacterClass,
        accumulator: &mut [i32; 6],
    ) -> Option<usize> {
        self.cap_values();
        let values = self.values();
        let weights = class.growth_weights();
        let total_weight: i32 = (0..6)
            .filter(|index| values[*index] < Self::MAX)
            .map(|index| i32::from(weights[index]))
            .sum();
        if total_weight == 0 {
            return None;
        }

        for index in 0..6 {
            if values[index] < Self::MAX {
                accumulator[index] = accumulator[index].saturating_add(i32::from(weights[index]));
            } else {
                accumulator[index] = 0;
            }
        }

        // Strict greater-than keeps ties in the documented attribute order.
        let mut selected: Option<usize> = None;
        for index in 0..6 {
            if values[index] >= Self::MAX {
                continue;
            }
            if selected.is_none_or(|best| accumulator[index] > accumulator[best]) {
                selected = Some(index);
            }
        }
        let selected = selected?;
        accumulator[selected] = accumulator[selected].saturating_sub(total_weight);
        match selected {
            0 => self.strength += 1,
            1 => self.dexterity += 1,
            2 => self.constitution += 1,
            3 => self.intelligence += 1,
            4 => self.wisdom += 1,
            5 => self.charisma += 1,
            _ => unreachable!("only six primary attributes exist"),
        }
        Some(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::WeaponTier;

    #[test]
    fn every_class_weight_set_totals_one_hundred() {
        for class in CharacterClass::ALL {
            assert_eq!(
                class
                    .growth_weights()
                    .into_iter()
                    .map(u16::from)
                    .sum::<u16>(),
                100
            );
        }
    }

    #[test]
    fn weighted_growth_is_deterministic_and_tracks_class_weights() {
        let mut left = PrimaryAttributes::default();
        let mut right = PrimaryAttributes::default();
        let mut left_accumulator = [0; 6];
        let mut right_accumulator = [0; 6];
        for _ in 0..10 {
            left.add_weighted_point(CharacterClass::Warrior, &mut left_accumulator);
            right.add_weighted_point(CharacterClass::Warrior, &mut right_accumulator);
        }
        assert_eq!(left, right);
        assert_eq!(left.values(), [13, 12, 12, 11, 11, 11]);
        assert_eq!(left_accumulator, right_accumulator);
    }

    #[test]
    fn capped_attributes_redistribute_the_next_point() {
        let mut attributes = PrimaryAttributes {
            strength: 20,
            ..PrimaryAttributes::default()
        };
        let before = attributes.total();
        let mut accumulator = [0; 6];
        let selected = attributes.add_weighted_point(CharacterClass::Warrior, &mut accumulator);
        assert_ne!(selected, Some(0));
        assert_eq!(attributes.strength, 20);
        assert_eq!(attributes.total(), before + 1);
    }

    #[test]
    fn legacy_weapon_identity_resolves_to_its_intended_family() {
        assert_eq!(WeaponTier::Omega.profile().family, WeaponFamily::Mace);
        assert_eq!(WeaponTier::Chaotic.profile().family, WeaponFamily::Spear);
        assert_eq!(WeaponTier::Wooden.profile().family, WeaponFamily::Sword);
        assert_eq!(WeaponTier::Omega.profile().tier, WeaponTier::Omega);
    }

    #[test]
    fn all_classes_have_a_primary_weapon_family_and_bounded_familiarity() {
        for class in CharacterClass::ALL {
            let preferred = match class {
                CharacterClass::Warrior => WeaponFamily::Sword,
                CharacterClass::Barbarian => WeaponFamily::Axe,
                CharacterClass::Mage => WeaponFamily::Staff,
                CharacterClass::Ranger => WeaponFamily::Bow,
                CharacterClass::Rogue => WeaponFamily::Dagger,
            };
            assert_eq!(class.weapon_proficiency_pct(preferred), 12);
        }
    }

    #[test]
    fn fully_capped_attributes_receive_no_further_growth() {
        let mut attributes = PrimaryAttributes {
            strength: 20,
            dexterity: 20,
            constitution: 20,
            intelligence: 20,
            wisdom: 20,
            charisma: 20,
        };
        let mut accumulator = [0; 6];
        assert_eq!(
            attributes.add_weighted_point(CharacterClass::Warrior, &mut accumulator),
            None
        );
        assert_eq!(attributes.total(), 120);
    }

    #[test]
    fn familiarity_is_bounded_and_does_not_define_equipment_eligibility() {
        for class in CharacterClass::ALL {
            for family in WeaponFamily::ALL {
                assert!(class.weapon_proficiency_pct(family) <= 12);
            }
        }
        assert_eq!(
            CharacterClass::Mage.weapon_proficiency_pct(WeaponFamily::Mace),
            0
        );
    }
}
