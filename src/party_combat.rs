//! Party-aware encounter rules, independent of terminal UI and event polling.
//!
//! Initiative is deterministic insertion order: player, companions in saved
//! party order, then enemies in encounter insertion order. Each eligible actor
//! may act once per round. A caller supplies the critical roll, making outcomes
//! reproducible with a controlled RNG. Positional rules remain an extension seam
//! for Section 12; status lifecycles remain owned by Section 13.

use crate::data::{get_ability, AbilityId};
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
        Self {
            id,
            source,
            name: name.into(),
            allegiance,
            hp,
            max_hp: hp,
            stamina,
            max_stamina: stamina,
            mana,
            max_mana: mana,
            attack,
            defence,
            condition: if hp == 0 {
                CombatantCondition::KnockedOut
            } else {
                CombatantCondition::Ready
            },
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
    BasicAttack {
        actor: CombatantId,
        target: CombatantId,
    },
    UseAbility {
        actor: CombatantId,
        target: CombatantId,
        ability: AbilityId,
    },
    Defend {
        actor: CombatantId,
    },
    Move {
        actor: CombatantId,
    },
    Retreat {
        actor: CombatantId,
    },
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
        for c in &combatants {
            if !seen.insert(c.id) {
                return Err(EncounterError::DuplicateId(c.id));
            }
        }
        if !combatants
            .iter()
            .any(|c| c.allegiance == Allegiance::Enemies && c.can_act())
        {
            return Err(EncounterError::NoHostiles);
        }
        let initiative = combatants.iter().map(|c| c.id).collect();
        let mut encounter = Self {
            combatants,
            initiative,
            turn_index: 0,
            round: 1,
            acted_this_round: HashSet::new(),
            outcome: EncounterOutcome::Ongoing,
            phase: EncounterPhase::Active,
        };
        encounter.refresh_outcome();
        Ok(encounter)
    }

    /// Builds encounter-local combatants from Section 10 player/party state.
    /// Encounter IDs are deterministic and independent of display names.
    pub fn from_party(
        player: &Player,
        party: &Party,
        companions: &[CompanionState],
        enemies: Vec<EncounterCombatant>,
    ) -> Result<Self, EncounterError> {
        let mut combatants = Vec::with_capacity(1 + party.companion_count() + enemies.len());
        combatants.push(EncounterCombatant::new(
            CombatantId(1),
            CombatantSource::Player,
            player.username.clone(),
            Allegiance::Allies,
            player.hp,
            player.stamina,
            player.mana,
            player.weapon.max_damage().max(1),
            player.armour.max_dr(),
        ));
        let mut next_id = 2;
        for id in party.members() {
            if let Some(c) = companions.iter().find(|c| c.id == *id) {
                if c.condition.can_join_party() && c.current_hp > 0 {
                    combatants.push(EncounterCombatant::new(
                        CombatantId(next_id),
                        CombatantSource::Companion(c.id),
                        c.display_name(),
                        Allegiance::Allies,
                        c.current_hp,
                        c.current_stamina,
                        c.current_mana,
                        c.attack,
                        c.defence,
                    ));
                    next_id += 1;
                }
            }
        }
        for enemy in enemies {
            if enemy.allegiance != Allegiance::Enemies {
                return Err(EncounterError::WrongAllegiance);
            }
            combatants.push(EncounterCombatant {
                id: CombatantId(next_id),
                ..enemy
            });
            next_id += 1;
        }
        Self::new(combatants)
    }

    pub fn combatants(&self) -> &[EncounterCombatant] {
        &self.combatants
    }
    pub fn round(&self) -> u64 {
        self.round
    }
    pub fn eligible_actors(&self) -> Vec<CombatantId> {
        self.combatants
            .iter()
            .filter(|c| c.can_act())
            .map(|c| c.id)
            .collect()
    }
    pub fn combatant(&self, id: CombatantId) -> Option<&EncounterCombatant> {
        self.combatants.iter().find(|c| c.id == id)
    }
    fn combatant_mut(&mut self, id: CombatantId) -> Option<&mut EncounterCombatant> {
        self.combatants.iter_mut().find(|c| c.id == id)
    }
    pub fn current_actor(&self) -> Option<CombatantId> {
        if self.outcome != EncounterOutcome::Ongoing || self.initiative.is_empty() {
            return None;
        }
        (0..self.initiative.len()).find_map(|offset| {
            let idx = (self.turn_index + offset) % self.initiative.len();
            let id = self.initiative[idx];
            (!self.acted_this_round.contains(&id)
                && self.combatant(id).is_some_and(EncounterCombatant::can_act))
            .then_some(id)
        })
    }

    fn consume_turn(&mut self, actor: CombatantId) {
        self.acted_this_round.insert(actor);
        if let Some(idx) = self.initiative.iter().position(|id| *id == actor) {
            self.turn_index = (idx + 1) % self.initiative.len().max(1);
        }
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

    /// Validates the entire action before mutation. Supplying the critical roll
    /// keeps the domain deterministic and easy to test.
    pub fn apply_action(&mut self, action: CombatAction, critical_roll: u32) -> Result<ActionResult, EncounterError> {
        if self.outcome != EncounterOutcome::Ongoing {
            return Err(EncounterError::EncounterFinished);
        }
        let actor_id = action.actor();
        let actor = self.combatant(actor_id).ok_or(EncounterError::MissingActor(actor_id))?;
        if !actor.can_act() {
            return Err(EncounterError::ActorIneligible(actor_id));
        }
        if self.acted_this_round.contains(&actor_id) {
            return Err(EncounterError::TurnAlreadyUsed(actor_id));
        }

        if matches!(&action, CombatAction::Retreat { .. }) {
            self.consume_turn(actor_id);
            self.outcome = EncounterOutcome::Retreated;
            self.phase = EncounterPhase::Finished(self.outcome);
            return Ok(ActionResult {
                actor: actor_id,
                target: None,
                damage: 0,
                healing: 0,
                outcome: self.outcome,
                message: "retreated",
            });
        }
        if matches!(&action, CombatAction::Move { .. } | CombatAction::Defend { .. }) {
            self.consume_turn(actor_id);
            return Ok(ActionResult {
                actor: actor_id,
                target: None,
                damage: 0,
                healing: 0,
                outcome: self.outcome,
                message: if matches!(&action, CombatAction::Move { .. }) {
                    "movement reserved for section 12"
                } else {
                    "defended"
                },
            });
        }

        let (target_id, ability_id) = match action {
            CombatAction::BasicAttack { target, .. } => (target, None),
            CombatAction::UseAbility { target, ability, .. } => (target, Some(ability)),
            _ => unreachable!("non-targeted actions returned above"),
        };
        let target = self.combatant(target_id).ok_or(EncounterError::MissingTarget(target_id))?;
        if !target.can_act() {
            return Err(EncounterError::TargetIneligible(target_id));
        }
        if target.allegiance == actor.allegiance {
            return Err(EncounterError::WrongAllegiance);
        }
        let ability = ability_id.map(get_ability);
        if let Some(def) = &ability {
            if actor.stamina < def.stamina_cost {
                return Err(EncounterError::InsufficientStamina);
            }
            if actor.mana < def.mana_cost {
                return Err(EncounterError::InsufficientMana);
            }
        }

        // Compute from snapshots after validation, then commit all effects.
        let source = self.combatant(actor_id).expect("validated actor");
        let target = self.combatant(target_id).expect("validated target");
        let multiplier = ability.as_ref().map(|a| a.damage_multiplier).unwrap_or(100);
        let raw = source.attack.saturating_mul(multiplier) / 100;
        let critical = critical_roll % 100 < if source.allegiance == Allegiance::Allies { 5 } else { 0 };
        let scaled = if critical { raw.saturating_mul(2) } else { raw };
        let damage = if scaled == 0 {
            0
        } else {
            scaled.saturating_sub(target.defence).max(1).min(target.hp)
        };

        if let Some(def) = &ability {
            let source = self.combatant_mut(actor_id).expect("validated actor");
            source.stamina -= def.stamina_cost;
            source.mana -= def.mana_cost;
        }
        let target = self.combatant_mut(target_id).expect("validated target");
        target.hp -= damage;
        target.sync_condition();
        self.consume_turn(actor_id);
        self.refresh_outcome();
        Ok(ActionResult {
            actor: actor_id,
            target: Some(target_id),
            damage,
            healing: 0,
            outcome: self.outcome,
            message: if self.outcome == EncounterOutcome::Victory {
                "victory"
            } else {
                "action resolved"
            },
        })
    }

    fn refresh_outcome(&mut self) {
        let enemies = self
            .combatants
            .iter()
            .any(|c| c.allegiance == Allegiance::Enemies && c.can_act());
        let allies = self
            .combatants
            .iter()
            .any(|c| c.allegiance == Allegiance::Allies && c.can_act());
        self.outcome = if !enemies {
            EncounterOutcome::Victory
        } else if !allies {
            EncounterOutcome::PartyDefeated
        } else {
            EncounterOutcome::Ongoing
        };
        self.phase = if self.outcome == EncounterOutcome::Ongoing {
            EncounterPhase::Active
        } else {
            EncounterPhase::Finished(self.outcome)
        };
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
    fn invalid_actions_are_rejected_without_mutation() {
        let mut e = encounter();
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(99) }, 0), Err(EncounterError::MissingTarget(CombatantId(99))));
        assert_eq!(e, before);
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(999), target: CombatantId(3) }, 0), Err(EncounterError::MissingActor(CombatantId(999))));
        assert_eq!(e, before);
    }
    #[test]
    fn insufficient_resources_do_not_mutate_encounter() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(1)).unwrap().stamina = 0;
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::UseAbility { actor: CombatantId(1), target: CombatantId(3), ability: AbilityId::HeavyStrike }, 0), Err(EncounterError::InsufficientStamina));
        assert_eq!(e, before);
    }
    #[test]
    fn partial_knockout_does_not_end_combat() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(2)).unwrap().hp = 0;
        e.combatant_mut(CombatantId(2)).unwrap().sync_condition();
        e.refresh_outcome();
        assert_eq!(e.outcome, EncounterOutcome::Ongoing);
        assert!(e.eligible_actors().contains(&CombatantId(1)));
    }
    #[test]
    fn actors_act_once_per_round_in_deterministic_order() {
        let mut e = encounter();
        let first = e.current_actor().unwrap();
        assert_eq!(first, CombatantId(1));
        e.apply_action(CombatAction::BasicAttack { actor: first, target: CombatantId(3) }, 99).unwrap();
        let after = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: first, target: CombatantId(3) }, 99), Err(EncounterError::TurnAlreadyUsed(first)));
        assert_eq!(e, after); assert_eq!(e.current_actor(), Some(CombatantId(2)));
    }
    #[test]
    fn victory_and_party_defeat_are_terminal() {
        let mut e = Encounter::new(vec![
            EncounterCombatant::new(CombatantId(1), CombatantSource::Player, "Hero", Allegiance::Allies, 10, 10, 10, 20, 0),
            EncounterCombatant::new(CombatantId(2), CombatantSource::Enemy, "Rat", Allegiance::Enemies, 1, 0, 0, 2, 0),
        ]).unwrap();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(2) }, 99).unwrap().outcome, EncounterOutcome::Victory);
        assert_eq!(e.apply_action(CombatAction::Defend { actor: CombatantId(1) }, 0), Err(EncounterError::EncounterFinished));
    }
    #[test]
    fn defeated_enemy_is_not_a_valid_target() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(3)).unwrap().hp = 0;
        e.combatant_mut(CombatantId(3)).unwrap().sync_condition();
        let before = e.clone();
        assert_eq!(e.apply_action(CombatAction::BasicAttack { actor: CombatantId(1), target: CombatantId(3) }, 0), Err(EncounterError::TargetIneligible(CombatantId(3))));
        assert_eq!(e, before);
    }
    #[test]
    fn retreat_is_terminal() {
        let mut e = encounter();
        assert_eq!(e.apply_action(CombatAction::Retreat { actor: CombatantId(1) }, 0).unwrap().outcome, EncounterOutcome::Retreated);
        assert_eq!(e.current_actor(), None);
    }
}
