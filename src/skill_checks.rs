//! Reusable skill checks for optional, non-combat actions.
//!
//! Probability is linear in the relevant skill's 0..=100 level:
//! raw = 30 + skill_level / 2 - difficulty + modifier.
//! The result is clamped to 10..=95 percent. Modifiers are percentage points
//! and should only be supplied when relevant to the checked action. Rolls are
//! integers in 0..100; a roll below the success chance succeeds.
//!
//! Skill checks are pure until a caller commits a separate validated outcome.

pub const MIN_SUCCESS_CHANCE: u8 = 10;
pub const MAX_SUCCESS_CHANCE: u8 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillCheck {
    pub skill_level: u32,
    pub difficulty: i32,
    /// Relevant action-specific modifiers, in percentage points.
    pub modifier: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillCheckResult {
    pub success_chance: u8,
    pub roll: u8,
    pub succeeded: bool,
}

impl SkillCheck {
    pub fn success_chance(self) -> u8 {
        let level = self.skill_level.min(100) as i32;
        (30 + level / 2 - self.difficulty + self.modifier)
            .clamp(i32::from(MIN_SUCCESS_CHANCE), i32::from(MAX_SUCCESS_CHANCE)) as u8
    }

    /// Resolve a check with a controlled 0..99 roll for deterministic tests.
    /// Invalid roll input is rejected instead of being silently normalised.
    pub fn resolve(self, roll: u8) -> Result<SkillCheckResult, SkillCheckError> {
        if roll > 99 {
            return Err(SkillCheckError::InvalidRoll(roll));
        }
        let success_chance = self.success_chance();
        Ok(SkillCheckResult {
            success_chance,
            roll,
            succeeded: roll < success_chance,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCheckError {
    InvalidRoll(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(skill_level: u32, difficulty: i32, modifier: i32) -> SkillCheck {
        SkillCheck { skill_level, difficulty, modifier }
    }

    #[test]
    fn higher_relevant_skill_improves_success_probability() {
        assert!(check(60, 35, 0).success_chance() > check(20, 35, 0).success_chance());
    }

    #[test]
    fn low_medium_and_high_skill_chances_are_bounded() {
        assert_eq!(check(1, 60, 0).success_chance(), MIN_SUCCESS_CHANCE);
        assert_eq!(check(50, 35, 0).success_chance(), 20);
        assert_eq!(check(100, -100, 0).success_chance(), MAX_SUCCESS_CHANCE);
    }

    #[test]
    fn modifiers_cannot_break_probability_bounds() {
        assert_eq!(check(1, 0, -1000).success_chance(), MIN_SUCCESS_CHANCE);
        assert_eq!(check(100, 0, 1000).success_chance(), MAX_SUCCESS_CHANCE);
    }

    #[test]
    fn skill_level_is_capped_before_probability_calculation() {
        assert_eq!(check(u32::MAX, 0, 0).success_chance(), MAX_SUCCESS_CHANCE);
        assert_eq!(check(100, 20, 0).success_chance(), check(1000, 20, 0).success_chance());
    }

    #[test]
    fn controlled_roll_is_deterministic_and_uses_exclusive_threshold() {
        let check = check(50, 35, 0);
        assert_eq!(check.resolve(19).unwrap().succeeded, true);
        assert_eq!(check.resolve(20).unwrap().succeeded, false);
        assert_eq!(check.resolve(99).unwrap().succeeded, false);
    }

    #[test]
    fn invalid_roll_is_rejected() {
        assert_eq!(check(50, 35, 0).resolve(100), Err(SkillCheckError::InvalidRoll(100)));
    }
}
