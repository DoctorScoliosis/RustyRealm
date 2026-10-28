//! Shared deterministic status lifecycle, tactical enemy policies, and boss telegraphs.
//!
//! Durations are counted in *affected-combatant activations*: effects applied
//! during an action do not tick immediately; at the start of that combatant's
//! next activation, damage/status is resolved, then remaining duration decreases.
//! Thus duration 1 affects exactly the next activation. Poison/Bleed/Burn deal
//! potency damage (minimum zero); Stun skips an activation; Slow subtracts up to
//! 20 initiative points (10 per potency tier); Mark adds up to 15 hit-chance points;
//! WeakenedDefences increases received damage by up to 50%. Same-kind effects
//! refresh duration and keep the stronger potency rather than stacking. Potency,
//! duration and probabilities are bounded. Resistance decreases application chance.
//!
//! This module is domain-only: it has no UI or terminal dependencies.

use crate::data::{AbilityId, DamageType};
use crate::party_combat::{
    Allegiance, CombatAction, CombatantId, Encounter, EncounterCombatant, EncounterError,
    RangeBand, RangeProfile,
};
use rand::Rng;

pub const MAX_EFFECT_DURATION: u32 = 12;
pub const MAX_EFFECT_POTENCY: u32 = 100;
pub const MAX_EFFECT_CHANCE_PCT: u8 = 100;
pub const MAX_SLOW_INITIATIVE_PENALTY: i16 = 20;
pub const MAX_MARK_HIT_BONUS: u8 = 15;
pub const MAX_DEFENCE_VULNERABILITY_PCT: u8 = 50;
pub const MAX_STATUS_RESISTANCE_PCT: u8 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffectKind {
    Poison,
    Bleed,
    Burn,
    Stun,
    Slow,
    Mark,
    WeakenedDefences,
}

impl StatusEffectKind {
    pub fn is_damage_over_time(self) -> bool {
        matches!(self, Self::Poison | Self::Bleed | Self::Burn)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub source: Option<CombatantId>,
    pub potency: u32,
    pub remaining_activations: u32,
    pub application_chance_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusResistance {
    /// Percentage chance to reject a status, capped at 90%.
    pub resistance_pct: u8,
}

impl StatusResistance {
    pub fn effective_chance(base_chance_pct: u8, resistance_pct: u8) -> u8 {
        let chance = base_chance_pct.min(MAX_EFFECT_CHANCE_PCT);
        let resistance = resistance_pct.min(MAX_STATUS_RESISTANCE_PCT);
        chance.saturating_mul(100_u8.saturating_sub(resistance)) / 100
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusTick {
    Damage { kind: StatusEffectKind, amount: u32 },
    Stunned,
    Expired(StatusEffectKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusApplication {
    Applied,
    Refreshed,
    Resisted,
    FailedChance,
    InvalidDuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyArchetype {
    AggressiveMelee,
    RangedPressure,
    DefensiveProtector,
    StatusSpecialist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyTendency {
    PreferDamagedTargets,
    PreferLowDefenceTargets,
    PreferMarkedTargets,
    PreferStatusFreeTargets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyProfile {
    pub archetype: EnemyArchetype,
    pub preferred_range: RangeProfile,
    pub abilities: Vec<AbilityId>,
    pub damage_type: DamageType,
    pub status_payload: Option<(StatusEffectKind, u32, u32, u8)>,
    pub tendency: EnemyTendency,
}

impl EnemyProfile {
    /// Map existing roster entries onto four behaviours without expanding the
    /// set of possible random encounter enemies.
    pub fn for_name(name: &str) -> Self {
        let base = name.strip_prefix("Elite ").unwrap_or(name);
        match base {
            "Dragon" | "Elemental" | "Mage" | "Sorcerer" | "Stronghold Mage" => Self {
                archetype: EnemyArchetype::StatusSpecialist,
                preferred_range: RangeProfile::RANGED,
                abilities: vec![AbilityId::LifeDrain],
                damage_type: if base == "Dragon" || base == "Elemental" { DamageType::Fire } else { DamageType::Arcane },
                status_payload: Some((if base == "Dragon" || base == "Elemental" { StatusEffectKind::Burn } else { StatusEffectKind::Slow }, 2, 8, 35)),
                tendency: EnemyTendency::PreferStatusFreeTargets,
            },
            "Stronghold Ranger" => Self {
                archetype: EnemyArchetype::RangedPressure,
                preferred_range: RangeProfile::RANGED,
                abilities: vec![AbilityId::HeavyStrike],
                damage_type: DamageType::Physical,
                status_payload: Some((StatusEffectKind::Mark, 2, 1, 40)),
                tendency: EnemyTendency::PreferMarkedTargets,
            },
            "Stronghold Mage" => Self {
                archetype: EnemyArchetype::StatusSpecialist,
                preferred_range: RangeProfile::RANGED,
                abilities: vec![AbilityId::LifeDrain],
                damage_type: DamageType::Arcane,
                status_payload: Some((StatusEffectKind::Slow, 2, 1, 35)),
                tendency: EnemyTendency::PreferStatusFreeTargets,
            },
            "Stronghold Warrior" | "Giant" | "Rockbeast" | "Ent" => Self {
                archetype: EnemyArchetype::AggressiveMelee,
                preferred_range: RangeProfile::MELEE,
                abilities: vec![AbilityId::HeavyStrike, AbilityId::Cleave],
                damage_type: DamageType::Physical,
                status_payload: Some((StatusEffectKind::WeakenedDefences, 2, 1, 25)),
                tendency: EnemyTendency::PreferDamagedTargets,
            },
            _ => Self {
                archetype: EnemyArchetype::DefensiveProtector,
                preferred_range: RangeProfile::MELEE,
                abilities: vec![AbilityId::Cleave],
                damage_type: DamageType::Physical,
                status_payload: Some((StatusEffectKind::Bleed, 2, 6, 25)),
                tendency: EnemyTendency::PreferLowDefenceTargets,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntendedAction {
    pub action: CombatAction,
    pub telegraph: Option<TelegraphedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphedAction {
    pub actor: CombatantId,
    pub target: CombatantId,
    pub action: CombatAction,
    pub name: String,
    pub description: String,
    pub damage_type: DamageType,
    pub damage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegraphState {
    Announced(TelegraphedAction),
    Resolved(TelegraphedAction),
    Cancelled { name: String, reason: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyDecision {
    pub intended: IntendedAction,
    pub selected_target: CombatantId,
}

/// Pure policy except for caller-supplied random roll. The returned action must
/// be passed to Encounter::apply_action; this policy never mutates the encounter.
pub fn propose_enemy_action(
    encounter: &Encounter,
    actor_id: CombatantId,
    roll: u32,
) -> Option<EnemyDecision> {
    let actor = encounter.combatant(actor_id)?;
    if actor.allegiance != Allegiance::Enemies || !actor.can_act() {
        return None;
    }
    let profile = EnemyProfile::for_name(&actor.name);
    let mut targets: Vec<&EncounterCombatant> = encounter
        .combatants()
        .iter()
        .filter(|target| target.allegiance != actor.allegiance && target.can_act())
        .collect();
    if targets.is_empty() {
        return None;
    }
    targets.sort_by_key(|target| {
        let range = encounter.range_between(actor_id, target.id).unwrap_or(RangeBand::Far);
        let range_penalty = if profile.preferred_range.allows(range) { 0 } else { 1 };
        let tactical = match profile.tendency {
            EnemyTendency::PreferDamagedTargets => target.hp,
            EnemyTendency::PreferLowDefenceTargets => target.defence,
            EnemyTendency::PreferMarkedTargets => target.id.0 as u32,
            EnemyTendency::PreferStatusFreeTargets => target.stamina,
        };
        (range_penalty, tactical, target.id)
    });
    // The roll rotates only among equal-policy candidates, preserving reproducibility.
    let best_key = {
        let target = targets[0];
        let range = encounter.range_between(actor_id, target.id).unwrap_or(RangeBand::Far);
        let penalty = if profile.preferred_range.allows(range) { 0 } else { 1 };
        let tactical = match profile.tendency {
            EnemyTendency::PreferDamagedTargets => target.hp,
            EnemyTendency::PreferLowDefenceTargets => target.defence,
            EnemyTendency::PreferMarkedTargets => target.id.0 as u32,
            EnemyTendency::PreferStatusFreeTargets => target.stamina,
        };
        (penalty, tactical)
    };
    let tied: Vec<_> = targets.into_iter().filter(|target| {
        let range = encounter.range_between(actor_id, target.id).unwrap_or(RangeBand::Far);
        let penalty = if profile.preferred_range.allows(range) { 0 } else { 1 };
        let tactical = match profile.tendency {
            EnemyTendency::PreferDamagedTargets => target.hp,
            EnemyTendency::PreferLowDefenceTargets => target.defence,
            EnemyTendency::PreferMarkedTargets => target.id.0 as u32,
            EnemyTendency::PreferStatusFreeTargets => target.stamina,
        };
        (penalty, tactical) == best_key
    }).collect();
    let target = tied[(roll as usize) % tied.len()];
    let range = encounter.range_between(actor_id, target.id).unwrap_or(RangeBand::Far);
    let intended_action = if profile.preferred_range.allows(range) {
        if !profile.abilities.is_empty() && roll % 3 == 0 {
            CombatAction::UseAbility {
                actor: actor_id,
                target: target.id,
                ability: profile.abilities[(roll as usize / 3) % profile.abilities.len()],
            }
        } else {
            CombatAction::BasicAttack { actor: actor_id, target: target.id }
        }
    } else {
        CombatAction::Move { actor: actor_id }
    };
    let telegraph = match intended_action {
        CombatAction::BasicAttack { .. } | CombatAction::UseAbility { .. } if roll % 4 == 0 => Some(TelegraphedAction {
            actor: actor_id,
            target: target.id,
            action: intended_action.clone(),
            name: format!("{} prepares an attack", actor.name),
            description: format!("{} prepares {:?} against {}.", actor.name, intended_action, target.name),
            damage_type: profile.damage_type,
            damage: actor.attack,
        }),
        _ => None,
    };
    Some(EnemyDecision {
        intended: IntendedAction { action: intended_action, telegraph },
        selected_target: target.id,
    })
}

/// A telegraph resolves only if the original target remains valid at resolution.
/// Caller validates its action through the common resolver; cancellation is explicit.
pub fn resolve_telegraph(
    encounter: &mut Encounter,
    telegraph: TelegraphedAction,
    hit_roll: u32,
    critical_roll: u32,
) -> Result<TelegraphState, EncounterError> {
    let actor = encounter.combatant(telegraph.actor);
    let target = encounter.combatant(telegraph.target);
    if actor.is_none_or(|a| !a.can_act()) || target.is_none_or(|t| !t.can_act() || t.allegiance == actor.unwrap().allegiance) {
        return Ok(TelegraphState::Cancelled { name: telegraph.name, reason: "actor or target is no longer valid" });
    }
    match encounter.apply_action(telegraph.action.clone(), hit_roll, critical_roll) {
        Ok(_) => Ok(TelegraphState::Resolved(telegraph)),
        Err(EncounterError::MissingActor(_) | EncounterError::MissingTarget(_) | EncounterError::TargetIneligible(_) | EncounterError::WrongAllegiance | EncounterError::OutOfRange { .. } | EncounterError::TurnAlreadyUsed(_)) => Ok(TelegraphState::Cancelled { name: telegraph.name, reason: "action is no longer legal" }),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::CompanionId;
    use crate::party_combat::{CombatantSource, CombatAttributes, CharacterClass};

    fn encounter() -> Encounter {
        let mut hero = EncounterCombatant::new(CombatantId(1), CombatantSource::Player, "Hero", Allegiance::Allies, 100, 40, 25, 30, 5);
        hero.set_class(CharacterClass::Ranger);
        let mut companion = EncounterCombatant::new(CombatantId(2), CombatantSource::Companion(CompanionId::BranTheShield), "Bran", Allegiance::Allies, 70, 30, 10, 20, 10);
        companion.base_attributes = CombatAttributes { speed: 50, accuracy: 50, evasion: 50, power: 50, resistance: 50 };
        let enemy = EncounterCombatant::new(CombatantId(3), CombatantSource::Enemy, "Dragon", Allegiance::Enemies, 100, 20, 0, 20, 10);
        Encounter::new(vec![hero, companion, enemy]).unwrap()
    }

    fn effect(kind: StatusEffectKind, duration: u32, potency: u32) -> StatusEffect {
        StatusEffect { kind, source: Some(CombatantId(3)), potency, remaining_activations: duration, application_chance_pct: 100 }
    }

    #[test]
    fn status_duration_and_stacking_policy_are_deterministic() {
        let mut effects = Vec::new();
        assert_eq!(apply_status(&mut effects, effect(StatusEffectKind::Poison, 2, 3), 0, 0), StatusApplication::Applied);
        assert_eq!(apply_status(&mut effects, effect(StatusEffectKind::Poison, 4, 2), 0, 0), StatusApplication::Refreshed);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].remaining_activations, 4);
        assert_eq!(effects[0].potency, 3);
        assert_eq!(tick_status_effects(&mut effects), vec![StatusTick::Damage { kind: StatusEffectKind::Poison, amount: 3 }]);
        assert_eq!(effects[0].remaining_activations, 3);
    }

    #[test]
    fn resistance_and_chance_are_explicitly_bounded() {
        assert_eq!(StatusResistance::effective_chance(80, 25), 60);
        assert_eq!(StatusResistance::effective_chance(255, 255), 10);
        let mut effects = Vec::new();
        let mut e = effect(StatusEffectKind::Burn, 2, 4);
        e.application_chance_pct = 1;
        assert_eq!(apply_status(&mut effects, e, 99, 0), StatusApplication::FailedChance);
        assert!(effects.is_empty());
        assert_eq!(StatusResistance::effective_chance(100, 255), 10);
    }

    #[test]
    fn every_effect_has_bounded_modifiers_and_tick_policy() {
        let mut effects = vec![
            effect(StatusEffectKind::Slow, 2, u32::MAX),
            effect(StatusEffectKind::Mark, 2, u32::MAX),
            effect(StatusEffectKind::WeakenedDefences, 2, u32::MAX),
            effect(StatusEffectKind::Stun, 1, 0),
        ];
        let ticks = tick_status_effects(&mut effects);
        assert!(ticks.contains(&StatusTick::Stunned));
        assert_eq!(slow_penalty(&effects), MAX_SLOW_INITIATIVE_PENALTY);
        assert_eq!(mark_hit_bonus(&effects), MAX_MARK_HIT_BONUS);
        assert_eq!(defence_vulnerability_pct(&effects), MAX_DEFENCE_VULNERABILITY_PCT);
    }

    #[test]
    fn enemy_policy_is_repeatable_and_profiles_are_distinct() {
        let e = encounter();
        assert_eq!(propose_enemy_action(&e, CombatantId(3), 7), propose_enemy_action(&e, CombatantId(3), 7));
        assert_ne!(EnemyProfile::for_name("Dragon").archetype, EnemyProfile::for_name("Goblin").archetype);
        assert_ne!(EnemyProfile::for_name("Stronghold Ranger").archetype, EnemyProfile::for_name("Stronghold Warrior").archetype);
    }

    #[test]
    fn telegraph_cancelled_when_target_becomes_invalid_and_resolution_uses_resolver() {
        let mut e = encounter();
        e.set_position(CombatantId(1), RangeBand::Melee).unwrap();
        let telegraph = TelegraphedAction {
            actor: CombatantId(3),
            target: CombatantId(1),
            action: CombatAction::BasicAttack { actor: CombatantId(3), target: CombatantId(1) },
            name: "Dragon Bite".into(),
            description: "A physical bite at Hero".into(),
            damage_type: DamageType::Physical,
            damage: 20,
        };
        let resolved = resolve_telegraph(&mut e, telegraph.clone(), 0, 99).unwrap();
        assert!(matches!(resolved, TelegraphState::Resolved(_)));
        assert!(e.combatant(CombatantId(1)).unwrap().hp < 100);

        let mut e = encounter();
        let telegraph = TelegraphedAction {
            actor: CombatantId(3),
            target: CombatantId(1),
            action: CombatAction::BasicAttack { actor: CombatantId(3), target: CombatantId(1) },
            name: "Dragon Bite".into(),
            description: "A physical bite at Hero".into(),
            damage_type: DamageType::Physical,
            damage: 20,
        };
        e.combatant_mut_for_test_set_hp(CombatantId(1), 0);
        assert!(matches!(resolve_telegraph(&mut e, telegraph, 0, 0).unwrap(), TelegraphState::Cancelled { .. }));
    }

    #[test]
    fn action_is_cancelled_not_mutated_when_out_of_range() {
        let mut e = encounter();
        let telegraph = TelegraphedAction {
            actor: CombatantId(3),
            target: CombatantId(1),
            action: CombatAction::BasicAttack { actor: CombatantId(3), target: CombatantId(1) },
            name: "Dragon Bite".into(),
            description: "A physical bite at Hero".into(),
            damage_type: DamageType::Physical,
            damage: 20,
        };
        let before = e.clone();
        assert!(matches!(resolve_telegraph(&mut e, telegraph, 0, 0).unwrap(), TelegraphState::Cancelled { .. }));
        assert_eq!(e, before);
    }
}
