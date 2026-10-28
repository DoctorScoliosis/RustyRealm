//! Party-aware encounter rules, independent of terminal UI and event handling.
//!
//! Initial initiative is deterministic roster order: player, party members in
//! their saved party order, then enemies in encounter insertion order. An actor
//! receives at most one action per round. The caller supplies the critical-roll
//! value; domain tests can therefore reproduce outcomes without ambient RNG.

use crate::data::{get_ability, AbilityId, DamageType};
use crate::party::{CompanionId, CompanionState, Party};
use crate::state::Player;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Allegiance {
    Allies,
    Enemies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatantSource {
    Player,
    Companion(CompanionId),
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatantCondition {
    Ready,
    KnockedOut,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterCombatant {
    pub id: CombatantId,
    pub source: CombatantSource,
    pub name: String,
    pub allegiance: Allegiance,
    pub hp: u32,
    pub max_hp: u32,
    pub stamina: u32,
    pub max_stamina: u32,
    pub mana: u32,
    pub max_mana: u32,
    pub attack: u32,
    pub defence: u32,
    pub condition: CombatantCondition,
}

impl EncounterCombatant {
    pub fn new(
        id: CombatantId,
        source: CombatantSource,
        name: impl Into<String>,
        allegiance: Allegiance,
        hp: u32,
        stamina: u32,
        mana: u32,
        attack: u32,
        defence: u32,
    ) -> Self {
        let condition = if hp == 0 {
            CombatantCondition::KnockedOut
        } else {
            CombatantCondition::Ready
        };
        Self {
            id, source, name: name.into(), allegiance, hp, max_hp: hp,
            stamina, max_stamina: stamina, mana, max_mana: mana,
            attack, defence, condition,
        }
    }

    pub fn can_act(&self) -> bool {
        self.condition == CombatantCondition::Ready && self.hp > 0
    }

    fn sync_condition(&mut self) {
        if self.hp == 0 {
            self.condition = CombatantCondition::KnockedOut;
        } else if self.condition == CombatantCondition::KnockedOut {
            self.condition = CombatantCondition::Ready;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterOutcome {
    Ongoing,
    Victory,
    PartyDefeated,
    Retreated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterPhase {
    Ready,
    Active,
    Finished(EncounterOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncounterError {
    DuplicateId(CombatantId),
    MissingActor(CombatantId),
    MissingTarget(CombatantId),
    ActorIneligible(CombatantId),
    TargetIneligible(CombatantId),
    WrongAllegiance,
    InsufficientStamina,
    InsufficientMana,
    TurnAlreadyUsed(CombatantId),
    EncounterFinished,
    NoHostiles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatAction {
    BasicAttack { actor: CombatantId, target: CombatantId },
    UseAbility { actor: CombatantId, target: CombatantId, ability: AbilityId },
    Defend { actor: CombatantId },
    Move { actor: CombatantId },
    Retreat { actor: CombatantId },
}

impl CombatAction {
    fn actor(&self) -> CombatantId {
        match *self {
            Self::BasicAttack { actor, .. }
            | Self::UseAbility { actor, .. }
            | Self::Defend { actor }
            | Self::Move { actor }
            | Self::Retreat { actor } => actor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub actor: CombatantId,
    pub target: Option<CombatantId>,
    pub damage: u32,
    pub healing: u32,
    pub outcome: EncounterOutcome,
    pub message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encounter {
    combatants: Vec<EncounterCombatant>,
    initiative: Vec<CombatantId>,
    turn_index: usize,
    round: u64,
    acted_this_round: HashSet<CombatantId>,
    pub outcome: EncounterOutcome,
    pub phase: EncounterPhase,
}

impl Encounter {
    pub fn new(combatants: Vec<EncounterCombatant>) -> Result<Self, EncounterError> {
        let mut seen = HashSet::new();
        for combatant in &combatants {
            if !seen.insert(combatant.id) {
                return Err(EncounterError::DuplicateId(combatant.id));
            }
        }
        if !combatants.iter().any(|c| c.allegiance == Allegiance::Enemies && c.can_act()) {
            return Err(EncounterError::NoHostiles);
        }
        let initiative = combatants.iter().map(|c| c.id).collect();
        let mut encounter = Self {
            combatants, initiative, turn_index: 0, round: 1,
            acted_this_round: HashSet::new(), outcome: EncounterOutcome::Ongoing,
            phase: EncounterPhase::Active,
        };
        encounter.refresh_outcome();
        Ok(encounter)
    }

    /// Builds encounter-local combatants from Section 10 player/party state.
    /// IDs are assigned in deterministic order; names are never used as identity.
    pub fn from_party(player: &Player, party: &Party, companions: &[CompanionState], enemies: Vec<EncounterCombatant>) -> Result<Self, EncounterError> {
        let mut combatants = Vec::with_capacity(1 + party.companion_count() + enemies.len());
        combatants.push(EncounterCombatant::new(
            CombatantId(1), CombatantSource::Player, player.username.clone(),
            Allegiance::Allies, player.hp, player.stamina, player.mana,
            player.weapon.max_damage().max(1), player.armour.max_dr(),
        ));
        let mut next_id = 2;
        for id in party.members() {
            if let Some(companion) = companions.iter().find(|c| c.id == *id) {
                let combatant = EncounterCombatant::new(
                    CombatantId(next_id), CombatantSource::Companion(companion.id),
                    companion.display_name(), Allegiance::Allies, companion.current_hp,
                    companion.current_stamina, companion.current_mana, companion.attack,
                    companion.defence,
                );
                if companion.condition.can_join_party() && companion.current_hp > 0 {
                    combatants.push(combatant);
                    next_id += 1;
                }
            }
        }
        for enemy in enemies {
            if enemy.allegiance != Allegiance::Enemies {
                return Err(EncounterError::WrongAllegiance);
            }
            combatants.push(EncounterCombatant { id: CombatantId(next_id), ..enemy });
            next_id += 1;
        }
        Self::new(combatants)
    }

    pub fn combatants(&self) -> &[EncounterCombatant] { &self.combatants }
    pub fn round(&self) -> u64 { self.round }
    pub fn current_actor(&self) -> Option<CombatantId> {
        if self.outcome != EncounterOutcome::Ongoing { return None; }
        self.initiative.iter().cycle().skip(self.turn_index).find(|id| {
            !self.acted_this_round.contains(id)
                && self.combatants.iter().any(|c| c.id == **id && c.can_act())
        }).copied()
    }
    pub fn eligible_actors(&self) -> Vec<CombatantId> {
        self.combatants.iter().filter(|c| c.can_act()).map(|c| c.id).collect()
    }

    pub fn combatant(&self, id: CombatantId) -> Option<&EncounterCombatant> {
        self.combatants.iter().find(|c| c.id == id)
    }

    fn combatant_mut(&mut self, id: CombatantId) -> Option<&mut EncounterCombatant> {
        self.combatants.iter_mut().find(|c| c.id == id)
    }

    pub fn advance_turn(&mut self) {
        if self.outcome != EncounterOutcome::Ongoing { return; }
        let Some(actor) = self.current_actor() else { return; };
        self.acted_this_round.insert(actor);
        self.turn_index = self.initiative.iter().position(|id| *id == actor).unwrap_or(0) + 1;
        if self.acted_this_round.len() >= self.initiative.len()
            || self.initiative.iter().all(|id| self.acted_this_round.contains(id)
                || !self.combatant(*id).is_some_and(EncounterCombatant::can_act))
        {
            self.acted_this_round.clear();
            self.round = self.round.saturating_add(1);
            self.turn_index = 0;
        }
    }

    pub fn apply_action(&mut self, action: CombatAction, critical_roll: u32) -> Result<ActionResult, EncounterError> {
        if self.outcome != EncounterOutcome::Ongoing { return Err(EncounterError::EncounterFinished); }
        let actor_id = action.actor();
        let actor = self.combatant(actor_id).ok_or(EncounterError::MissingActor(actor_id))?;
        if !actor.can_act() { return Err(EncounterError::ActorIneligible(actor_id)); }
        if self.acted_this_round.contains(&actor_id) { return Err(EncounterError::TurnAlreadyUsed(actor_id)); }

        if let CombatAction::Retreat { actor } = action {
            self.acted_this_round.insert(actor);
            self.outcome = EncounterOutcome::Retreated;
            self.phase = EncounterPhase::Finished(self.outcome);
            return Ok(ActionResult { actor, target: None, damage: 0, healing: 0, outcome: self.outcome, message: "retreated" });
        }
        if let CombatAction::Move { actor } | CombatAction::Defend { actor } = action {
            // Section 12 owns positional rules. These actions consume a turn but do not yet alter position.
            self.acted_this_round.insert(actor);
            self.advance_turn();
            return Ok(ActionResult { actor, target: None, damage: 0, healing: 0, outcome: self.outcome, message: if matches!(action, CombatAction::Move { .. }) { "movement deferred to section 12" } else { "defended" } });
        }

        let (target_id, ability) = match action {
            CombatAction::BasicAttack { target, .. } => (target, None),
            CombatAction::UseAbility { target, ability, .. } => (target, Some(ability)),
            _ => unreachable!(),
        };
        let target = self.combatant(target_id).ok_or(EncounterError::MissingTarget(target_id))?;
        if !target.can_act() { return Err(EncounterError::TargetIneligible(target_id)); }
        if target.allegiance == actor.allegiance { return Err(EncounterError::WrongAllegiance); }
        if let Some(ability) = ability {
            let definition = get_ability(ability);
            if actor.stamina < definition.stamina_cost { return Err(EncounterError::InsufficientStamina); }
            if actor.mana < definition.mana_cost { return Err(EncounterError::InsufficientMana); }
        }

        // Validation is complete; only now may the encounter be mutated.
        let ability = ability.map(get_ability);
        let (damage, healing) = {
            let source = self.combatant(actor_id).expect("validated actor");
            let base = source.attack.saturating_mul(ability.as_ref().map(|a| a.damage_multiplier).unwrap_or(100)) / 100;
            let critical = critical_roll % 100 < if source.allegiance == Allegiance::Allies { 5 } else { 0 };
            let damage = if critical { base.saturating_mul(2) } else { base };
            let target = self.combatant(target_id).expect("validated target");
            let reduced = damage.saturating_sub(target.defence);
            (if damage > 0 { reduced.max(1) } else { 0 }, 0)
        };
        if let Some(definition) = ability {
            let source = self.combatant_mut(actor_id).expect("validated actor");
            source.stamina -= definition.stamina_cost;
            source.mana -= definition.mana_cost;
        }
        let target = self.combatant_mut(target_id).expect("validated target");
        target.hp = target.hp.saturating_sub(damage);
        target.sync_condition();
        self.acted_this_round.insert(actor_id);
        self.refresh_outcome();
        if self.outcome == EncounterOutcome::Ongoing { self.advance_turn_after_actor(actor_id); }
        Ok(ActionResult {
            actor: actor_id, target: Some(target_id), damage, healing, outcome: self.outcome,
            message: if self.outcome == EncounterOutcome::Victory { "victory" } else { "action resolved" },
        })
    }

    fn advance_turn_after_actor(&mut self, actor: CombatantId) {
        self.turn_index = self.initiative.iter().position(|id| *id == actor).unwrap_or(0) + 1;
        let remaining = self.initiative.iter().any(|id| {
            !self.acted_this_round.contains(id)
                && self.combatant(*id).is_some_and(EncounterCombatant::can_act)
        });
        if !remaining {
            self.acted_this_round.clear();
            self.round = self.round.saturating_add(1);
            self.turn_index = 0;
        }
    }

    fn refresh_outcome(&mut self) {
        if self.combatants.iter().any(|c| c.allegiance == Allegiance::Enemies && c.can_act()) {
            if !self.combatants.iter().any(|c| c.allegiance == Allegiance::Allies && c.can_act()) {
                self.outcome = EncounterOutcome::PartyDefeated;
            }
        } else {
            self.outcome = EncounterOutcome::Victory;
        }
        self.phase = if self.outcome == EncounterOutcome::Ongoing { EncounterPhase::Active } else { EncounterPhase::Finished(self.outcome) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encounter() -> Encounter {
        Encounter::new(vec![
            EncounterCombatant::new(CombatantId(1), CombatantSource::Player, "Hero", Allegiance::Allies, 100, 40, 25, 30, 5),
            EncounterCombatant::new(CombatantId(2), CombatantSource::Companion(CompanionId::BranTheShield), "Bran", Allegiance::Allies, 10, 30, 5, 20, 10),
            EncounterCombatant::new(CombatantId(3), CombatantSource::Enemy, "Goblin", Allegiance::Enemies, 15, 20, 0, 8, 1),
            EncounterCombatant::new(CombatantId(4), CombatantSource::Enemy, "Goblin", Allegiance::Enemies, 30, 20, 0, 8, 1),
        ]).unwrap()
    }

    #[test]
    fn stable_ids_allow_duplicate_display_names() {
        let e = encounter();
        assert_eq!(e.combatants()[2].name, e.combatants()[3].name);
        assert_ne!(e.combatants()[2].id, e.combatants()[3].id);
    }

    #[test]
    fn invalid_target_and_ineligible_actor_are_rejected_without_mutation() {
        let mut e = encounter();
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(99) }, 0), Err(EncounterError::MissingTarget(CombatantId(99))));
        assert_eq!(e, before);
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(999), target: CombatantId(3) }, 0), Err(EncounterError::MissingActor(CombatantId(999))));
        assert_eq!(e, before);
    }

    #[test]
    fn insufficient_resources_leave_encounter_unchanged() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(1)).unwrap().stamina = 0;
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::UseAbility { actor: CombatantId(1), target: CombatantId(3), ability: AbilityId::HeavyStrike }, 0), Err(EncounterError::InsufficientStamina));
        assert_eq!(e, before);
    }

    #[test]
    fn partial_knockout_does_not_end_encounter_and_remaining_actor_can_act() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(2)).unwrap().hp = 0;
        e.combatant_mut(CombatantId(2)).unwrap().sync_condition();
        e.refresh_outcome();
        assert_eq!(e.outcome, EncounterOutcome::Ongoing);
        assert!(e.eligible_actors().contains(&CombatantId(1)));
    }

    #[test]
    fn turns_are_single_use_and_initiative_is_deterministic() {
        let mut e = encounter();
        let first = e.current_actor().unwrap();
        assert_eq!(first, CombatantId(1));
        e.apply_action(CombatAction::BasicAttack { actor: first, target: CombatantId(3) }, 99).unwrap();
        let after = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: first, target: CombatantId(3) }, 99), Err(EncounterError::TurnAlreadyUsed(first)));
        assert_eq!(e, after);
        assert_eq!(e.current_actor(), Some(CombatantId(2)));
    }

    #[test]
    fn victory_and_party_defeat_are_terminal() {
        let mut e = Encounter::new(vec![
            EncounterCombatant::new(CombatantId(1), CombatantSource::Player, "Hero", Allegiance::Allies, 10, 10, 10, 20, 0),
            EncounterCombatant::new(CombatantId(2), CombatantSource::Enemy, "Rat", Allegiance::Enemies, 1, 0, 0, 2, 0),
        ]).unwrap();
        let result = e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(2) }, 99).unwrap();
        assert_eq!(result.outcome, EncounterOutcome::Victory);
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(2) }, 99), Err(EncounterError::EncounterFinished));
    }

    #[test]
    fn defeated_enemy_does_not_count_as_valid_target() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(3)).unwrap().hp = 0;
        e.combatant_mut(CombatantId(3)).unwrap().sync_condition();
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(3) }, 0), Err(EncounterError::TargetIneligible(CombatantId(3))));
        assert_eq!(e, before);
    }

    #[test]
    fn retreat_is_terminal_and_deterministic() {
        let mut e = encounter();
        let result = e.apply_action(CombatAction::Retreat { actor: CombatantId(1) }, 0).unwrap();
        assert_eq!(result.outcome, EncounterOutcome::Retreated);
        assert_eq!(e.current_actor(), None);
        assert_eq!(e.apply_action(CombatAction::Defend { actor: CombatantId(1) }, 0), Err(EncounterError::EncounterFinished));
    }
}
