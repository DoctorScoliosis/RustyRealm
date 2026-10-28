//! Party-aware encounter rules, independent of terminal UI and event polling.
//!
//! Initiative uses descending speed, then stable combatant ID for deterministic
//! ties. Attribute points use a 0..=100 scale with a default of 50. Class bonuses
//! are additive; equipment/mastery inputs are applied once and then clamped.
//! Position uses encounter-owned range bands, not grid coordinates.

use crate::data::{get_ability, AbilityId};
use crate::party::{CompanionId, CompanionState, Party};
use crate::state::Player;
use std::collections::HashSet;

pub const MIN_STAT: i16 = 0;
pub const MAX_STAT: i16 = 100;
pub const BASE_STAT: i16 = 50;
pub const MIN_HIT_CHANCE: u8 = 5;
pub const MAX_HIT_CHANCE: u8 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Allegiance { Allies, Enemies }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatantSource { Player, Companion(CompanionId), Enemy }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatantCondition { Ready, KnockedOut, Removed }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterClass { Warrior, Ranger, Mage, Rogue }

impl CharacterClass {
    pub const ALL: [Self; 4] = [Self::Warrior, Self::Ranger, Self::Mage, Self::Rogue];
    pub const fn name(self) -> &'static str {
        match self { Self::Warrior => "Warrior", Self::Ranger => "Ranger", Self::Mage => "Mage", Self::Rogue => "Rogue" }
    }
    pub const fn definition(self) -> ClassDefinition {
        match self {
            Self::Warrior => ClassDefinition { speed: -5, accuracy: 0, evasion: -5, power: 5, resistance: 10, critical: 0 },
            Self::Ranger => ClassDefinition { speed: 5, accuracy: 10, evasion: 0, power: 0, resistance: 0, critical: 0 },
            Self::Mage => ClassDefinition { speed: 0, accuracy: 0, evasion: -2, power: 8, resistance: 5, critical: 0 },
            Self::Rogue => ClassDefinition { speed: 10, accuracy: 0, evasion: 10, power: -3, resistance: -5, critical: 10 },
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassDefinition {
    pub speed: i16, pub accuracy: i16, pub evasion: i16, pub power: i16,
    pub resistance: i16, pub critical: i16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatAttributes {
    pub speed: i16, pub accuracy: i16, pub evasion: i16, pub power: i16, pub resistance: i16,
}
impl Default for CombatAttributes {
    fn default() -> Self { Self { speed: 50, accuracy: 50, evasion: 50, power: 50, resistance: 50 } }
}
impl CombatAttributes {
    pub fn derived(class: CharacterClass, base: Self, equipment: Self, mastery: Self) -> Self {
        let c = class.definition();
        Self {
            speed: clamp_stat(base.speed.saturating_add(equipment.speed).saturating_add(mastery.speed).saturating_add(c.speed)),
            accuracy: clamp_stat(base.accuracy.saturating_add(equipment.accuracy).saturating_add(mastery.accuracy).saturating_add(c.accuracy)),
            evasion: clamp_stat(base.evasion.saturating_add(equipment.evasion).saturating_add(mastery.evasion).saturating_add(c.evasion)),
            power: clamp_stat(base.power.saturating_add(equipment.power).saturating_add(mastery.power).saturating_add(c.power)),
            resistance: clamp_stat(base.resistance.saturating_add(equipment.resistance).saturating_add(mastery.resistance).saturating_add(c.resistance)),
        }
    }
}
fn clamp_stat(v: i16) -> i16 { v.clamp(MIN_STAT, MAX_STAT) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RangeBand { Melee, Near, Far }
impl RangeBand {
    pub const fn name(self) -> &'static str {
        match self { Self::Melee => "melee", Self::Near => "near", Self::Far => "far" }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeProfile { pub min: RangeBand, pub max: RangeBand }
impl RangeProfile {
    pub const MELEE: Self = Self { min: RangeBand::Melee, max: RangeBand::Melee };
    pub const RANGED: Self = Self { min: RangeBand::Melee, max: RangeBand::Far };
    pub fn allows(self, range: RangeBand) -> bool {
        range >= self.min && range <= self.max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterCombatant {
    pub id: CombatantId, pub source: CombatantSource, pub name: String, pub allegiance: Allegiance,
    pub hp: u32, pub max_hp: u32, pub stamina: u32, pub max_stamina: u32, pub mana: u32, pub max_mana: u32,
    /// Existing weapon/attack magnitude and armour/defence magnitude are retained.
    pub attack: u32, pub defence: u32, pub condition: CombatantCondition,
    pub class: CharacterClass, pub base_attributes: CombatAttributes,
    pub equipment_attributes: CombatAttributes, pub mastery_attributes: CombatAttributes,
    pub attributes: CombatAttributes, pub attack_range: RangeProfile,
}
impl EncounterCombatant {
    pub fn new(id: CombatantId, source: CombatantSource, name: impl Into<String>, allegiance: Allegiance,
        hp: u32, stamina: u32, mana: u32, attack: u32, defence: u32) -> Self {
        let mut c = Self {
            id, source, name: name.into(), allegiance, hp, max_hp: hp, stamina, max_stamina: stamina,
            mana, max_mana: mana, attack, defence,
            condition: if hp == 0 { CombatantCondition::KnockedOut } else { CombatantCondition::Ready },
            class: default_class(source), base_attributes: CombatAttributes::default(),
            equipment_attributes: CombatAttributes::default(), mastery_attributes: CombatAttributes::default(),
            attributes: CombatAttributes::default(), attack_range: RangeProfile::MELEE,
        };
        c.recalculate_attributes();
        c
    }
    pub fn set_class(&mut self, class: CharacterClass) { self.class = class; self.recalculate_attributes(); }
    pub fn set_attribute_sources(&mut self, base: CombatAttributes, equipment: CombatAttributes, mastery: CombatAttributes) {
        self.base_attributes = base; self.equipment_attributes = equipment; self.mastery_attributes = mastery;
        self.recalculate_attributes();
    }
    pub fn recalculate_attributes(&mut self) {
        self.attributes = CombatAttributes::derived(self.class, self.base_attributes, self.equipment_attributes, self.mastery_attributes);
    }
    pub fn can_act(&self) -> bool { self.condition == CombatantCondition::Ready && self.hp > 0 }
    fn sync_condition(&mut self) {
        if self.hp == 0 { self.condition = CombatantCondition::KnockedOut; }
        else if self.condition == CombatantCondition::KnockedOut { self.condition = CombatantCondition::Ready; }
    }
}
fn default_class(source: CombatantSource) -> CharacterClass {
    match source {
        CombatantSource::Player | CombatantSource::Companion(CompanionId::BranTheShield)
        | CombatantSource::Companion(CompanionId::GarrickIronfist) => CharacterClass::Warrior,
        CombatantSource::Companion(CompanionId::LyraSwiftbow) => CharacterClass::Ranger,
        CombatantSource::Companion(CompanionId::MarenStarweave) => CharacterClass::Mage,
        CombatantSource::Companion(_) | CombatantSource::Enemy => CharacterClass::Warrior,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterOutcome { Ongoing, Victory, PartyDefeated, Retreated }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterPhase { Active, Finished(EncounterOutcome) }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncounterError {
    DuplicateId(CombatantId), MissingActor(CombatantId), MissingTarget(CombatantId),
    ActorIneligible(CombatantId), TargetIneligible(CombatantId), WrongAllegiance,
    OutOfRange { actor: CombatantId, target: CombatantId, actual: RangeBand, min: RangeBand, max: RangeBand },
    InsufficientStamina, InsufficientMana, TurnAlreadyUsed(CombatantId), EncounterFinished, NoHostiles,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatAction {
    BasicAttack { actor: CombatantId, target: CombatantId },
    UseAbility { actor: CombatantId, target: CombatantId, ability: AbilityId },
    Defend { actor: CombatantId }, Move { actor: CombatantId }, Retreat { actor: CombatantId },
}
impl CombatAction {
    fn actor(&self) -> CombatantId {
        match *self {
            Self::BasicAttack { actor, .. } | Self::UseAbility { actor, .. } | Self::Defend { actor }
            | Self::Move { actor } | Self::Retreat { actor } => actor,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub actor: CombatantId, pub target: Option<CombatantId>, pub damage: u32, pub healing: u32,
    pub outcome: EncounterOutcome, pub hit: Option<bool>, pub hit_chance: Option<u8>, pub message: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encounter {
    combatants: Vec<EncounterCombatant>, initiative: Vec<CombatantId>, turn_index: usize,
    round: u64, acted_this_round: HashSet<CombatantId>, positions: Vec<(CombatantId, i32)>,
    pub outcome: EncounterOutcome, pub phase: EncounterPhase,
}
impl Encounter {
    pub fn new(combatants: Vec<EncounterCombatant>) -> Result<Self, EncounterError> {
        let mut seen = HashSet::new();
        for c in &combatants { if !seen.insert(c.id) { return Err(EncounterError::DuplicateId(c.id)); } }
        if !combatants.iter().any(|c| c.allegiance == Allegiance::Enemies && c.can_act()) { return Err(EncounterError::NoHostiles); }
        let mut initiative: Vec<_> = combatants.iter().map(|c| c.id).collect();
        initiative.sort_by_key(|id| {
            let c = combatants.iter().find(|c| c.id == *id).expect("ID from combatant roster");
            (std::cmp::Reverse(c.attributes.speed), c.id)
        });
        let positions = combatants.iter().map(|c| (c.id, if c.allegiance == Allegiance::Allies { 0 } else { 2 })).collect();
        let mut encounter = Self { combatants, initiative, turn_index: 0, round: 1, acted_this_round: HashSet::new(), positions, outcome: EncounterOutcome::Ongoing, phase: EncounterPhase::Active };
        encounter.refresh_outcome();
        Ok(encounter)
    }
    pub fn from_party(player: &Player, party: &Party, companions: &[CompanionState], enemies: Vec<EncounterCombatant>) -> Result<Self, EncounterError> {
        let mut combatants = Vec::with_capacity(1 + party.companion_count() + enemies.len());
        let mut hero = EncounterCombatant::new(CombatantId(1), CombatantSource::Player, player.username.clone(), Allegiance::Allies,
            player.hp, player.stamina, player.mana, player.weapon.max_damage().max(1), player.armour.max_dr());
        hero.equipment_attributes.resistance = (player.armour.max_dr().min(50) / 5) as i16;
        hero.recalculate_attributes();
        combatants.push(hero);
        let mut next_id = 2;
        for id in party.members() {
            if let Some(c) = companions.iter().find(|c| c.id == *id) {
                if c.condition.can_join_party() && c.current_hp > 0 {
                    let class = match c.role() {
                        crate::party::CompanionRole::FrontlineDefence => CharacterClass::Warrior,
                        crate::party::CompanionRole::RangedDamage => CharacterClass::Ranger,
                        crate::party::CompanionRole::MagicSupport => CharacterClass::Mage,
                    };
                    let mut companion = EncounterCombatant::new(CombatantId(next_id), CombatantSource::Companion(c.id),
                        c.display_name(), Allegiance::Allies, c.current_hp, c.current_stamina, c.current_mana, c.attack, c.defence);
                    companion.set_class(class);
                    combatants.push(companion); next_id += 1;
                }
            }
        }
        for enemy in enemies {
            if enemy.allegiance != Allegiance::Enemies { return Err(EncounterError::WrongAllegiance); }
            combatants.push(EncounterCombatant { id: CombatantId(next_id), ..enemy }); next_id += 1;
        }
        Self::new(combatants)
    }
    pub fn combatants(&self) -> &[EncounterCombatant] { &self.combatants }
    pub fn round(&self) -> u64 { self.round }
    pub fn combatant(&self, id: CombatantId) -> Option<&EncounterCombatant> { self.combatants.iter().find(|c| c.id == id) }
    fn combatant_mut(&mut self, id: CombatantId) -> Option<&mut EncounterCombatant> {
        self.combatants.iter_mut().find(|c| c.id == id)
    }
    pub fn range_between(&self, actor: CombatantId, target: CombatantId) -> Option<RangeBand> {
        let a = self.positions.iter().find(|(id, _)| *id == actor)?.1;
        let t = self.positions.iter().find(|(id, _)| *id == target)?.1;
        Some(match a.abs_diff(t) {
            0 => RangeBand::Melee,
            1 => RangeBand::Near,
            _ => RangeBand::Far,
        })
    }

    /// Small encounter-local coordinates, not grid tiles. Distance projects to
    /// melee (0), near (1), and far (2+), so movement can change range bands.
    pub fn set_position(&mut self, id: CombatantId, position: RangeBand) -> Result<(), EncounterError> {
        if self.combatant(id).is_none() {
            return Err(EncounterError::MissingActor(id));
        }
        if let Some((_, coordinate)) = self.positions.iter_mut().find(|(who, _)| *who == id) {
            *coordinate = position as i32;
        }
        Ok(())
    }

    pub fn move_towards(&mut self, id: CombatantId, target_id: CombatantId) -> Result<RangeBand, EncounterError> {
        if self.combatant(id).is_none() {
            return Err(EncounterError::MissingActor(id));
        }
        if self.combatant(target_id).is_none() {
            return Err(EncounterError::MissingTarget(target_id));
        }
        let actor_index = self.positions.iter().position(|(who, _)| *who == id).expect("actor position exists");
        let target_coordinate = self.positions.iter().find(|(who, _)| *who == target_id).expect("target position exists").1;
        let actor_coordinate = self.positions[actor_index].1;
        if actor_coordinate < target_coordinate {
            self.positions[actor_index].1 += 1;
        } else if actor_coordinate > target_coordinate {
            self.positions[actor_index].1 -= 1;
        }
        self.range_between(id, target_id).ok_or(EncounterError::MissingTarget(target_id))
    }

    pub fn hit_chance(attacker: &EncounterCombatant, defender: &EncounterCombatant, range: RangeBand) -> u8 {
        let penalty = match range { RangeBand::Melee => 0_i16, RangeBand::Near => 5, RangeBand::Far => 15 };
        (70_i16 + (attacker.attributes.accuracy - BASE_STAT) / 2
            - (defender.attributes.evasion - BASE_STAT) / 2 - penalty)
            .clamp(MIN_HIT_CHANCE as i16, MAX_HIT_CHANCE as i16) as u8
    }
    pub fn legal_targets(&self, actor_id: CombatantId, ability: Option<AbilityId>) -> Vec<CombatantId> {
        let Some(actor) = self.combatant(actor_id) else { return Vec::new() };
        if !actor.can_act() { return Vec::new(); }
        self.combatants.iter().filter(|t| {
            if !t.can_act() || t.allegiance == actor.allegiance { return false; }
            if ability == Some(AbilityId::ShieldWall) { return false; }
            let range = self.range_between(actor_id, t.id).unwrap_or(RangeBand::Far);
            let max = if actor.class == CharacterClass::Ranger { RangeBand::Far } else { actor.attack_range.max };
            range >= actor.attack_range.min && range <= max
        }).map(|t| t.id).collect()
    }
    pub fn eligible_actors(&self) -> Vec<CombatantId> { self.combatants.iter().filter(|c| c.can_act()).map(|c| c.id).collect() }
    pub fn current_actor(&self) -> Option<CombatantId> {
        if self.outcome != EncounterOutcome::Ongoing || self.initiative.is_empty() { return None; }
        (0..self.initiative.len()).find_map(|offset| {
            let idx = (self.turn_index + offset) % self.initiative.len();
            let id = self.initiative[idx];
            (!self.acted_this_round.contains(&id) && self.combatant(id).is_some_and(EncounterCombatant::can_act)).then_some(id)
        })
    }
    fn consume_turn(&mut self, actor: CombatantId) {
        self.acted_this_round.insert(actor);
        if let Some(idx) = self.initiative.iter().position(|id| *id == actor) { self.turn_index = (idx + 1) % self.initiative.len().max(1); }
        let remaining = self.initiative.iter().any(|id| !self.acted_this_round.contains(id) && self.combatant(*id).is_some_and(EncounterCombatant::can_act));
        if !remaining { self.acted_this_round.clear(); self.round = self.round.saturating_add(1); self.turn_index = 0; }
    }
    pub fn apply_action(&mut self, action: CombatAction, hit_roll: u32, critical_roll: u32) -> Result<ActionResult, EncounterError> {
        if self.outcome != EncounterOutcome::Ongoing { return Err(EncounterError::EncounterFinished); }
        let actor_id = action.actor();
        let actor = self.combatant(actor_id).ok_or(EncounterError::MissingActor(actor_id))?;
        if !actor.can_act() { return Err(EncounterError::ActorIneligible(actor_id)); }
        if self.acted_this_round.contains(&actor_id) { return Err(EncounterError::TurnAlreadyUsed(actor_id)); }
        if matches!(&action, CombatAction::Retreat { .. }) {
            self.consume_turn(actor_id); self.outcome = EncounterOutcome::Retreated; self.phase = EncounterPhase::Finished(self.outcome);
            return Ok(ActionResult { actor: actor_id, target: None, damage: 0, healing: 0, outcome: self.outcome, hit: None, hit_chance: None, message: "retreated" });
        }
        if matches!(&action, CombatAction::Move { .. }) {
            let allegiance = actor.allegiance;
            let target = self.combatants.iter().filter(|c| c.can_act() && c.allegiance != allegiance)
                .min_by_key(|c| self.range_between(actor_id, c.id).unwrap_or(RangeBand::Far))
                .map(|c| c.id).ok_or(EncounterError::NoHostiles)?;
            self.move_towards(actor_id, target)?; self.consume_turn(actor_id);
            return Ok(ActionResult { actor: actor_id, target: None, damage: 0, healing: 0, outcome: self.outcome, hit: None, hit_chance: None, message: "moved one range band" });
        }
        if matches!(&action, CombatAction::Defend { .. }) {
            self.consume_turn(actor_id);
            return Ok(ActionResult { actor: actor_id, target: None, damage: 0, healing: 0, outcome: self.outcome, hit: None, hit_chance: None, message: "defended" });
        }
        let (target_id, ability_id) = match action {
            CombatAction::BasicAttack { target, .. } => (target, None),
            CombatAction::UseAbility { target, ability, .. } => (target, Some(ability)),
            _ => unreachable!("non-targeted actions returned above"),
        };
        let target = self.combatant(target_id).ok_or(EncounterError::MissingTarget(target_id))?;
        if !target.can_act() { return Err(EncounterError::TargetIneligible(target_id)); }
        if target.allegiance == self.combatant(actor_id).expect("actor validated").allegiance { return Err(EncounterError::WrongAllegiance); }
        let range = self.range_between(actor_id, target_id).expect("positions must exist");
        let source = self.combatant(actor_id).expect("actor validated");
        let ability = ability_id.map(get_ability);
        let max_range = if source.class == CharacterClass::Ranger { RangeBand::Far } else { source.attack_range.max };
        if (ability_id == Some(AbilityId::ShieldWall)) || range < source.attack_range.min || range > max_range {
            return Err(EncounterError::OutOfRange { actor: actor_id, target: target_id, actual: range, min: source.attack_range.min, max: max_range });
        }
        if let Some(def) = &ability {
            if source.stamina < def.stamina_cost { return Err(EncounterError::InsufficientStamina); }
            if source.mana < def.mana_cost { return Err(EncounterError::InsufficientMana); }
        }
        let source = self.combatant(actor_id).expect("actor validated");
        let target = self.combatant(target_id).expect("target validated");
        let chance = Self::hit_chance(source, target, range);
        let hit = hit_roll % 100 < u32::from(chance);
        let multiplier = ability.as_ref().map(|a| a.damage_multiplier).unwrap_or(100);
        let raw = source.attack.saturating_mul(multiplier) / 100;
        let crit_chance = (5_i16 + source.class.definition().critical).clamp(0, 25) as u32;
        let critical = hit && critical_roll % 100 < crit_chance;
        let scaled = if critical { raw.saturating_mul(2) } else { raw };
        let damage = if !hit || scaled == 0 { 0 } else { scaled.saturating_sub(target.defence).max(1).min(target.hp) };
        if let Some(def) = &ability {
            let source = self.combatants.iter_mut().find(|c| c.id == actor_id).expect("actor validated");
            source.stamina -= def.stamina_cost; source.mana -= def.mana_cost;
        }
        if damage > 0 {
            let target = self.combatants.iter_mut().find(|c| c.id == target_id).expect("target validated");
            target.hp -= damage; target.sync_condition();
        }
        self.consume_turn(actor_id); self.refresh_outcome();
        Ok(ActionResult { actor: actor_id, target: Some(target_id), damage, healing: 0, outcome: self.outcome,
            hit: Some(hit), hit_chance: Some(chance), message: if !hit { "miss" } else if self.outcome == EncounterOutcome::Victory { "victory" } else { "action resolved" } })
    }
    fn refresh_outcome(&mut self) {
        let enemies = self.combatants.iter().any(|c| c.allegiance == Allegiance::Enemies && c.can_act());
        let allies = self.combatants.iter().any(|c| c.allegiance == Allegiance::Allies && c.can_act());
        self.outcome = if !enemies { EncounterOutcome::Victory } else if !allies { EncounterOutcome::PartyDefeated } else { EncounterOutcome::Ongoing };
        self.phase = if self.outcome == EncounterOutcome::Ongoing { EncounterPhase::Active } else { EncounterPhase::Finished(self.outcome) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn encounter() -> Encounter {
        Encounter::new(vec![
            EncounterCombatant::new(
                CombatantId(1),
                CombatantSource::Player,
                "Hero",
                Allegiance::Allies,
                100,
                40,
                25,
                30,
                5,
            ),
            EncounterCombatant::new(
                CombatantId(2),
                CombatantSource::Companion(CompanionId::BranTheShield),
                "Bran",
                Allegiance::Allies,
                10,
                30,
                5,
                20,
                10,
            ),
            EncounterCombatant::new(
                CombatantId(3),
                CombatantSource::Enemy,
                "Goblin",
                Allegiance::Enemies,
                15,
                20,
                0,
                8,
                1,
            ),
            EncounterCombatant::new(
                CombatantId(4),
                CombatantSource::Enemy,
                "Goblin",
                Allegiance::Enemies,
                30,
                20,
                0,
                8,
                1,
            ),
        ])
        .unwrap()
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
        assert_eq!(
            e.apply_action(
                CombatAction::BasicAttack {
                    actor: CombatantId(1),
                    target: CombatantId(99),
                },
                0,
                0
            ),
            Err(EncounterError::MissingTarget(CombatantId(99)))
        );
        assert_eq!(e, before);
        assert_eq!(
            e.apply_action(
                CombatAction::BasicAttack {
                    actor: CombatantId(999),
                    target: CombatantId(3),
                },
                0,
                0
            ),
            Err(EncounterError::MissingActor(CombatantId(999)))
        );
        assert_eq!(e, before);
    }
    #[test]
    fn insufficient_resources_do_not_mutate_encounter() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(1)).unwrap().stamina = 0;
        e.set_position(CombatantId(3), RangeBand::Melee).unwrap();
        let before = e.clone();
        assert_eq!(
            e.apply_action(
                CombatAction::UseAbility {
                    actor: CombatantId(1),
                    target: CombatantId(3),
                    ability: AbilityId::HeavyStrike,
                },
                0,
                0
            ),
            Err(EncounterError::InsufficientStamina)
        );
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
        e.set_position(CombatantId(3), RangeBand::Melee).unwrap();
        e.apply_action(
            CombatAction::BasicAttack {
                actor: first,
                target: CombatantId(3),
            },
            0,
            99,
        )
        .unwrap();
        let after = e.clone();
        assert_eq!(
            e.apply_action(
                CombatAction::BasicAttack {
                    actor: first,
                    target: CombatantId(3),
                },
                0,
                99,
            ),
            Err(EncounterError::TurnAlreadyUsed(first))
        );
        assert_eq!(e, after);
        assert_eq!(e.combatant(CombatantId(2)).unwrap().attributes.speed, 50);
        assert_eq!(e.current_actor(), Some(CombatantId(2)));
    }
    #[test]
    fn victory_and_party_defeat_are_terminal() {
        let mut e = Encounter::new(vec![
            EncounterCombatant::new(
                CombatantId(1),
                CombatantSource::Player,
                "Hero",
                Allegiance::Allies,
                10,
                10,
                10,
                20,
                0,
            ),
            EncounterCombatant::new(
                CombatantId(2),
                CombatantSource::Enemy,
                "Rat",
                Allegiance::Enemies,
                1,
                0,
                0,
                2,
                0,
            ),
        ])
        .unwrap();
        e.set_position(CombatantId(2), RangeBand::Melee).unwrap();
        assert_eq!(
            e.apply_action(
                CombatAction::BasicAttack {
                    actor: CombatantId(1),
                    target: CombatantId(2),
                },
                0,
                0,
            )
            .unwrap()
            .outcome,
            EncounterOutcome::Victory
        );
        assert_eq!(
            e.apply_action(
                CombatAction::Defend {
                    actor: CombatantId(1),
                },
                0,
                0,
            ),
            Err(EncounterError::EncounterFinished)
        );
    }
    #[test]
    fn defeated_enemy_is_not_a_valid_target() {
        let mut e = encounter();
        e.combatant_mut(CombatantId(3)).unwrap().hp = 0;
        e.combatant_mut(CombatantId(3)).unwrap().sync_condition();
        let before = e.clone();
        assert_eq!(
            e.apply_action(
                CombatAction::BasicAttack {
                    actor: CombatantId(1),
                    target: CombatantId(3),
                },
                0,
                0
            ),
            Err(EncounterError::TargetIneligible(CombatantId(3)))
        );
        assert_eq!(e, before);
    }
    #[test]
    fn retreat_is_terminal() {
        let mut e = encounter();
        assert_eq!(
            e.apply_action(
                CombatAction::Retreat {
                    actor: CombatantId(1),
                },
                0,
                0,
            )
            .unwrap()
            .outcome,
            EncounterOutcome::Retreated
        );
        assert_eq!(e.current_actor(), None);
    }
    #[test]
    fn class_bonuses_are_distinct_and_clamped() {
        let base = CombatAttributes { speed: 40, accuracy: 40, evasion: 40, power: 40, resistance: 40 };
        let neutral = CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 };
        let warrior = CombatAttributes::derived(CharacterClass::Warrior, base, neutral, neutral);
        let ranger = CombatAttributes::derived(CharacterClass::Ranger, base, neutral, neutral);
        let mage = CombatAttributes::derived(CharacterClass::Mage, base, neutral, neutral);
        let rogue = CombatAttributes::derived(CharacterClass::Rogue, base, neutral, neutral);
        assert_eq!(ranger.accuracy, 50);
        assert_eq!(warrior.accuracy, 40);
        assert_eq!(mage.power, 48);
        assert_eq!(warrior.power, 45);
        assert!(rogue.speed > warrior.speed);
        assert!(rogue.evasion > warrior.evasion);
        assert!(rogue.resistance < warrior.resistance);
        let maxed = CombatAttributes::derived(
            CharacterClass::Mage,
            CombatAttributes { speed: i16::MAX, accuracy: i16::MAX, evasion: i16::MAX, power: i16::MAX, resistance: i16::MAX },
            neutral,
            neutral,
        );
        assert_eq!(maxed.speed, MAX_STAT);
        assert_eq!(maxed.accuracy, MAX_STAT);
    }

    #[test]
    fn initiative_orders_speed_descending_then_stable_id() {
        let mut roster = encounter().combatants.clone();
        for c in &mut roster {
            c.base_attributes = CombatAttributes { speed: 40, accuracy: 40, evasion: 40, power: 40, resistance: 40 };
            c.equipment_attributes = CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 };
            c.mastery_attributes = CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 };
        }
        roster[0].set_class(CharacterClass::Warrior);
        roster[1].set_class(CharacterClass::Rogue);
        roster[2].set_class(CharacterClass::Mage);
        roster[3].base_attributes.speed = 20;
        roster[3].recalculate_attributes();
        let mut e = Encounter::new(roster).unwrap();
        assert_eq!(e.current_actor(), Some(CombatantId(2)));
        let mut tied = e.combatants.clone();
        for c in &mut tied {
            c.set_class(CharacterClass::Warrior);
            c.base_attributes = CombatAttributes { speed: 40, accuracy: 40, evasion: 40, power: 40, resistance: 40 };
            c.equipment_attributes = CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 };
            c.mastery_attributes = CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 };
            c.recalculate_attributes();
        }
        e = Encounter::new(tied).unwrap();
        assert_eq!(e.current_actor(), Some(CombatantId(1)));
    }

    #[test]
    fn movement_changes_range_and_accuracy_is_bounded() {
        let mut e = encounter();
        assert_eq!(e.range_between(CombatantId(1), CombatantId(3)), Some(RangeBand::Far));
        assert_eq!(e.move_towards(CombatantId(1), CombatantId(3)).unwrap(), RangeBand::Near);
        assert_eq!(e.move_towards(CombatantId(1), CombatantId(3)).unwrap(), RangeBand::Melee);

        let attacker = e.combatant(CombatantId(1)).unwrap();
        let defender = e.combatant(CombatantId(3)).unwrap();
        assert!((MIN_HIT_CHANCE..=MAX_HIT_CHANCE).contains(&Encounter::hit_chance(attacker, defender, RangeBand::Far)));
    }

    #[test]
    fn out_of_range_actions_do_not_spend_resources_or_change_health() {
        let mut e = encounter();
        let before_stamina = e.combatant(CombatantId(1)).unwrap().stamina;
        let before_hp = e.combatant(CombatantId(3)).unwrap().hp;
        assert!(matches!(
            e.apply_action(CombatAction::UseAbility {
                actor: CombatantId(1), target: CombatantId(3), ability: AbilityId::HeavyStrike,
            }, 0, 0),
            Err(EncounterError::OutOfRange { .. })
        ));
        assert_eq!(e.combatant(CombatantId(1)).unwrap().stamina, before_stamina);
        assert_eq!(e.combatant(CombatantId(3)).unwrap().hp, before_hp);
    }

}
