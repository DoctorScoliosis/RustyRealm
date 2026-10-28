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


/// Apply/refresh one effect using caller-provided 0..99 roll and resistance.
/// The current effect set is changed only after duration/chance checks pass.
pub fn apply_status(
    effects: &mut Vec<StatusEffect>,
    mut incoming: StatusEffect,
    resistance_pct: u8,
    roll: u32,
) -> StatusApplication {
    if incoming.remaining_activations == 0 {
        return StatusApplication::InvalidDuration;
    }
    incoming.remaining_activations = incoming.remaining_activations.min(MAX_EFFECT_DURATION);
    incoming.potency = incoming.potency.min(MAX_EFFECT_POTENCY);
    incoming.application_chance_pct = incoming.application_chance_pct.min(MAX_EFFECT_CHANCE_PCT);
    let chance = StatusResistance::effective_chance(
        incoming.application_chance_pct,
        resistance_pct,
    );
    if roll % 100 >= u32::from(chance) {
        return if resistance_pct.min(MAX_STATUS_RESISTANCE_PCT) > 0 && incoming.application_chance_pct > 0 {
            StatusApplication::Resisted
        } else {
            StatusApplication::FailedChance
        };
    }
    if let Some(current) = effects.iter_mut().find(|e| e.kind == incoming.kind) {
        current.remaining_activations = current.remaining_activations.max(incoming.remaining_activations);
        current.potency = current.potency.max(incoming.potency);
        current.application_chance_pct = current.application_chance_pct.max(incoming.application_chance_pct);
        if incoming.source.is_some() {
            current.source = incoming.source;
        }
        StatusApplication::Refreshed
    } else {
        effects.push(incoming);
        StatusApplication::Applied
    }
}

/// Resolve all effects at the start of an affected combatant's activation, then
/// decrement their duration and remove expired effects. A newly applied effect
/// is not ticked until its target next activates.
pub fn tick_status_effects(effects: &mut Vec<StatusEffect>) -> Vec<StatusTick> {
    let mut ticks = Vec::new();
    for effect in effects.iter_mut() {
        match effect.kind {
            StatusEffectKind::Poison | StatusEffectKind::Bleed | StatusEffectKind::Burn => {
                ticks.push(StatusTick::Damage {
                    kind: effect.kind,
                    amount: effect.potency.min(MAX_EFFECT_POTENCY),
                });
            }
            StatusEffectKind::Stun => ticks.push(StatusTick::Stunned),
            StatusEffectKind::Slow | StatusEffectKind::Mark | StatusEffectKind::WeakenedDefences => {}
        }
        effect.remaining_activations = effect.remaining_activations.saturating_sub(1);
    }
    effects.retain(|effect| {
        if effect.remaining_activations == 0 {
            ticks.push(StatusTick::Expired(effect.kind));
            false
        } else {
            true
        }
    });
    ticks
}

pub fn slow_penalty(effects: &[StatusEffect]) -> i16 {
    let potency: u32 = effects.iter()
        .filter(|e| e.kind == StatusEffectKind::Slow)
        .map(|e| e.potency.min(2))
        .sum();
    ((potency.saturating_mul(10)).min(MAX_SLOW_INITIATIVE_PENALTY as u32)) as i16
}

pub fn mark_hit_bonus(effects: &[StatusEffect]) -> u8 {
    effects.iter()
        .filter(|e| e.kind == StatusEffectKind::Mark)
        .fold(0_u32, |sum, e| sum.saturating_add(e.potency.min(3).saturating_mul(5)))
        .min(u32::from(MAX_MARK_HIT_BONUS)) as u8
}

pub fn defence_vulnerability_pct(effects: &[StatusEffect]) -> u8 {
    effects.iter()
        .filter(|e| e.kind == StatusEffectKind::WeakenedDefences)
        .fold(0_u32, |sum, e| sum.saturating_add(e.potency.min(2).saturating_mul(25)))
        .min(u32::from(MAX_DEFENCE_VULNERABILITY_PCT)) as u8
}

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
        let chance = u16::from(base_chance_pct.min(MAX_EFFECT_CHANCE_PCT));
        let resistance = u16::from(resistance_pct.min(MAX_STATUS_RESISTANCE_PCT));
        (chance * (100 - resistance) / 100) as u8
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
    /// Maps the existing hostile/story roster onto a handful of explicit behaviours.
    /// This does not change the random encounter allow-list in combat.rs.
    pub fn for_name(name: &str) -> Self {
        let base = name.strip_prefix("Elite ").unwrap_or(name);
        match base {
            "Dragon" | "Elemental" | "Mage" | "Sorcerer" => Self {
                archetype: EnemyArchetype::StatusSpecialist,
                preferred_range: RangeProfile::RANGED,
                abilities: vec![AbilityId::LifeDrain],
                damage_type: if base == "Dragon" || base == "Elemental" { DamageType::Fire } else { DamageType::Arcane },
                status_payload: Some((if base == "Dragon" || base == "Elemental" { StatusEffectKind::Burn } else { StatusEffectKind::Slow }, 2, 1, 35)),
                tendency: EnemyTendency::PreferStatusFreeTargets,
            },
            "Stronghold Mage" => Self {
                archetype: EnemyArchetype::StatusSpecialist,
                preferred_range: RangeProfile::RANGED,
                abilities: vec![AbilityId::LifeDrain],
                damage_type: DamageType::Arcane,
                status_payload: Some((StatusEffectKind::Slow, 2, 1, 35)),
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
    let valid = match (actor, target) {
        (Some(a), Some(t)) => a.can_act() && t.can_act() && t.allegiance != a.allegiance,
        _ => false,
    };
    if !valid {
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
        let companion = EncounterCombatant::new(CombatantId(2), CombatantSource::Companion(CompanionId::BranTheShield), "Bran", Allegiance::Allies, 70, 30, 10, 20, 10);
        let mut enemy = EncounterCombatant::new(CombatantId(3), CombatantSource::Enemy, "Dragon", Allegiance::Enemies, 100, 20, 0, 20, 10);
        enemy.attack_range = RangeProfile::RANGED;
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
        assert_eq!(StatusResistance::effective_chance(100, 255), 10);
        let mut effects = Vec::new();
        let mut e = effect(StatusEffectKind::Burn, 2, 4);
        e.application_chance_pct = 1;
        assert_eq!(apply_status(&mut effects, e, 99, 0), StatusApplication::Resisted);
        assert!(effects.is_empty());
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
        assert_eq!(effects.len(), 3, "duration-two effects remain for one more activation");
        let _ = tick_status_effects(&mut effects);
        assert!(effects.is_empty());
        let effects = vec![effect(StatusEffectKind::Slow, 2, u32::MAX), effect(StatusEffectKind::Mark, 2, u32::MAX), effect(StatusEffectKind::WeakenedDefences, 2, u32::MAX)];
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
            action: CombatAction::BasicAttack {
                actor: CombatantId(3),
                target: CombatantId(1),
            },
            name: "Dragon Bite".into(),
            description: "A physical bite at Hero".into(),
            damage_type: DamageType::Physical,
            damage: 20,
        };
        let resolved = resolve_telegraph(&mut e, telegraph.clone(), 0, 99).unwrap();
        assert!(matches!(resolved, TelegraphState::Resolved(_)));
        assert!(e.combatant(CombatantId(1)).unwrap().hp < 100);

        let mut e = encounter();
        e.apply_action(
            CombatAction::BasicAttack { actor: CombatantId(3), target: CombatantId(1) },
            0,
            0,
        ).unwrap();
        // The actor has already acted; a second telegraph is cancelled by common validation.
        assert!(matches!(
            resolve_telegraph(&mut e, telegraph, 0, 0).unwrap(),
            TelegraphState::Cancelled { .. }
        ));
    }

    #[test]
    fn action_is_cancelled_not_mutated_when_out_of_range() {
        let mut e = encounter();
        let telegraph = TelegraphedAction {
            actor: CombatantId(3),
            target: CombatantId(1),
            action: CombatAction::UseAbility {
                actor: CombatantId(3),
                target: CombatantId(1),
                ability: AbilityId::HeavyStrike,
            },
            name: "Dragon Heavy Strike".into(),
            description: "A heavy melee strike at Hero".into(),
            damage_type: DamageType::Physical,
            damage: 40,
        };
        e.set_position(CombatantId(3), RangeBand::Far).unwrap();
        e.set_position(CombatantId(1), RangeBand::Melee).unwrap();
        let before = e.clone();
        assert!(matches!(
            resolve_telegraph(&mut e, telegraph, 0, 0).unwrap(),
            TelegraphState::Cancelled { .. }
        ));
        assert_eq!(e, before);
    }
}
