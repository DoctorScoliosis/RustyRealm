//! Persistent companion knockout, rescue, recovery and expedition resolution.
//!
//! Recovery uses the world's existing shortest-path travel-cost queries. One recovery
//! turn is one committed main-menu action (the game already advances the market by
//! one turn after each main-menu selection); opening and closing nested menus does
//! not advance recovery.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::party::{CompanionCondition, CompanionId, CompanionState, Party};
use crate::state::GameState;
use crate::world::{LocationCategory, LocationId, World};

pub const RECOVERY_TURNS: u32 = 3;
pub const TREATMENT_COST: u32 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionRecoveryRecord {
    pub companion_id: CompanionId,
    pub destination: Option<LocationId>,
    /// Remaining committed main-menu turns. Menus and save/load do not advance it.
    pub turns_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecoveryState {
    #[serde(default)]
    pub records: Vec<CompanionRecoveryRecord>,
    #[serde(default)]
    pub expedition_defeat_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    CompanionNotRecruited(CompanionId),
    CompanionNotRecoverable(CompanionId),
    NoTreatmentSettlement,
    NotAtTreatmentSettlement,
    InsufficientGold { required: u32, available: u32 },
    InvalidSettlement(LocationId),
    DuplicateRecoveryRecord(CompanionId),
    PlayerAlreadyDefeated,
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompanionNotRecruited(id) => {
                write!(f, "Companion {id} has not been recruited.")
            }
            Self::CompanionNotRecoverable(id) => {
                write!(f, "Companion {id} is not awaiting recovery.")
            }
            Self::NoTreatmentSettlement => f.write_str(
                "No reachable settlement can provide treatment. The companion remains safe but unavailable until a route or settlement becomes available.",
            ),
            Self::NotAtTreatmentSettlement => f.write_str(
                "Travel to the companion's recorded treatment settlement before paying for immediate treatment.",
            ),
            Self::InsufficientGold { required, available } => {
                write!(f, "Treatment costs {required} gold, but only {available} is available.")
            }
            Self::InvalidSettlement(id) => {
                write!(f, "Location {id} is not a treatment-capable settlement.")
            }
            Self::DuplicateRecoveryRecord(id) => {
                write!(f, "Companion {id} already has a recovery record.")
            }
            Self::PlayerAlreadyDefeated => f.write_str("The player has no health remaining."),
        }
    }
}

impl std::error::Error for RecoveryError {}

impl RecoveryState {
    pub fn record(&self, id: CompanionId) -> Option<&CompanionRecoveryRecord> {
        self.records.iter().find(|record| record.companion_id == id)
    }

    pub fn record_knockout(
        &mut self,
        companion: &mut CompanionState,
        world: &World,
        origin: &LocationId,
    ) -> Result<bool, RecoveryError> {
        if companion.current_hp > 0 {
            return Ok(false);
        }
        if self.record(companion.id).is_some() {
            // Idempotent processing: retain the original destination and timer.
            companion.condition = CompanionCondition::KnockedOut;
            return Ok(false);
        }
        let destination = world.get_nearest_settlement(origin);
        companion.current_hp = 0;
        companion.condition = CompanionCondition::KnockedOut;
        self.records.push(CompanionRecoveryRecord {
            companion_id: companion.id,
            destination,
            turns_remaining: RECOVERY_TURNS,
        });
        self.records
            .sort_by(|left, right| left.companion_id.as_str().cmp(right.companion_id.as_str()));
        Ok(true)
    }

    pub fn record_knockout_by_id(
        &mut self,
        state: &mut GameState,
        id: CompanionId,
        world: &World,
    ) -> Result<bool, RecoveryError> {
        let origin = state.current_location.clone();
        let already_recorded = self.record(id).is_some();
        {
            let companion = state.companions.iter_mut().find(|c| c.id == id)
                .ok_or(RecoveryError::CompanionNotRecruited(id))?;
            if already_recorded {
                companion.current_hp = 0;
                companion.condition = CompanionCondition::KnockedOut;
            } else {
                self.record_knockout(companion, world, &origin)?;
            }
        }
        // A knocked-out companion is removed from the active roster. Repeating
        // this call is safe even if the first transition already removed them.
        let _ = state.party.remove_member(id);
        Ok(!already_recorded)
    }

    /// Transactional core used by the UI when borrowing GameState fields separately.
    pub fn apply_treatment_parts(
        &mut self,
        gold: &mut u32,
        party: &mut Party,
        companion: &mut CompanionState,
        id: CompanionId,
        at: &LocationId,
        world: &World,
    ) -> Result<u32, RecoveryError> {
        let record_index = self
            .records
            .iter()
            .position(|record| record.companion_id == id)
            .ok_or(RecoveryError::CompanionNotRecoverable(id))?;
        let destination = self.records[record_index]
            .destination
            .clone()
            .ok_or(RecoveryError::NoTreatmentSettlement)?;
        if destination != *at {
            return Err(RecoveryError::NotAtTreatmentSettlement);
        }
        let location = world
            .get_location(&destination)
            .ok_or_else(|| RecoveryError::InvalidSettlement(destination.clone()))?;
        if location.category != LocationCategory::Settlement {
            return Err(RecoveryError::InvalidSettlement(destination));
        }
        if companion.id != id {
            return Err(RecoveryError::CompanionNotRecoverable(id));
        }
        if !matches!(
            companion.condition,
            CompanionCondition::KnockedOut | CompanionCondition::Recovering
        ) {
            return Err(RecoveryError::CompanionNotRecoverable(id));
        }
        if *gold < TREATMENT_COST {
            return Err(RecoveryError::InsufficientGold {
                required: TREATMENT_COST,
                available: *gold,
            });
        }

        *gold -= TREATMENT_COST;
        companion.fully_restore();
        companion.condition = CompanionCondition::Available;
        self.records.remove(record_index);
        let _ = party.remove_member(id);
        Ok(TREATMENT_COST)
    }

    pub fn apply_treatment(
        &mut self,
        state: &mut GameState,
        id: CompanionId,
        at: &LocationId,
        world: &World,
    ) -> Result<u32, RecoveryError> {
        if at != &state.current_location {
            return Err(RecoveryError::NotAtTreatmentSettlement);
        }
        let destination = self
            .record(id)
            .and_then(|record| record.destination.as_ref())
            .ok_or(RecoveryError::NoTreatmentSettlement)?;
        if destination != at {
            return Err(RecoveryError::NotAtTreatmentSettlement);
        }
        let companion_index = state
            .companions
            .iter()
            .position(|c| c.id == id)
            .ok_or(RecoveryError::CompanionNotRecruited(id))?;
        self.apply_treatment_parts(
            &mut state.player.gold,
            &mut state.party,
            &mut state.companions[companion_index],
            id,
            at,
            world,
        )
    }

    /// Advance once for a committed main-menu action, never for nested menus or reloads.
    pub fn advance_committed_turn(&mut self, companions: &mut [CompanionState]) -> Vec<CompanionId> {
        let mut recovered = Vec::new();
        for record in &mut self.records {
            // Without a reachable treatment settlement, neither time nor payment
            // may silently complete rescue. A future world graph can be re-evaluated
            // explicitly without losing the original safe recovery record.
            if record.destination.is_some() && record.turns_remaining > 0 {
                record.turns_remaining -= 1;
            }
        }
        let completed: Vec<CompanionId> = self
            .records
            .iter()
            .filter(|record| record.destination.is_some() && record.turns_remaining == 0)
            .map(|record| record.companion_id)
            .collect();
        for id in &completed {
            if let Some(companion) = companions.iter_mut().find(|c| c.id == *id) {
                if matches!(
                    companion.condition,
                    CompanionCondition::KnockedOut | CompanionCondition::Recovering
                ) {
                    companion.fully_restore();
                    companion.condition = CompanionCondition::Available;
                    recovered.push(*id);
                }
            }
        }
        self.records.retain(|record| record.turns_remaining > 0);
        recovered
    }

    /// Synchronise a completed party-combat encounter back into persistent player
    /// state. Calling this repeatedly is safe: previously recorded knockouts keep
    /// their original destination and timer.
    pub fn sync_party_encounter(
        &mut self,
        state: &mut GameState,
        encounter: &crate::party_combat::Encounter,
        world: &World,
    ) -> Result<Vec<CompanionId>, RecoveryError> {
        let updates: Vec<_> = encounter
            .combatants()
            .iter()
            .filter_map(|combatant| match combatant.source {
                crate::party_combat::CombatantSource::Companion(id) => Some((
                    id,
                    combatant.hp,
                    combatant.stamina,
                    combatant.mana,
                    !combatant.can_act(),
                )),
                _ => None,
            })
            .collect();
        let mut newly_knocked_out = Vec::new();
        for (id, hp, stamina, mana, knocked_out) in updates {
            let already_recorded = self.record(id).is_some();
            if !state.companions.iter().any(|companion| companion.id == id) {
                return Err(RecoveryError::CompanionNotRecruited(id));
            }
            if knocked_out && !already_recorded {
                {
                    let companion = state
                        .companions
                        .iter_mut()
                        .find(|c| c.id == id)
                        .expect("validated companion");
                    companion.current_hp = 0;
                    companion.condition = CompanionCondition::KnockedOut;
                    companion.current_stamina = stamina.min(companion.max_stamina);
                    companion.current_mana = mana.min(companion.max_mana);
                }
                self.record_knockout_by_id(state, id, world)?;
                newly_knocked_out.push(id);
            } else {
                let companion = state
                    .companions
                    .iter_mut()
                    .find(|c| c.id == id)
                    .expect("validated companion");
                companion.current_hp = hp.min(companion.max_hp);
                companion.current_stamina = stamina.min(companion.max_stamina);
                companion.current_mana = mana.min(companion.max_mana);
                if knocked_out {
                    companion.current_hp = 0;
                    companion.condition = CompanionCondition::KnockedOut;
                }
            }
            if knocked_out {
                let _ = state.party.remove_member(id);
            }
        }
        Ok(newly_knocked_out)
    }

    pub fn resolve_player_defeat(
        &mut self,
        state: &mut GameState,
        world: &World,
    ) -> Result<(), RecoveryError> {
        if state.player.hp > 0 {
            return Err(RecoveryError::PlayerAlreadyDefeated);
        }

        let destination = world
            .get_nearest_settlement(&state.current_location)
            .ok_or(RecoveryError::NoTreatmentSettlement)?;

        // Work on a clone and commit only once all required destinations are valid.
        let mut next_state = state.clone();
        let mut next_recovery = self.clone();
        let origin = next_state.current_location.clone();
        let knocked_out: Vec<_> = next_state
            .party
            .members()
            .iter()
            .copied()
            .filter(|id| {
                next_state
                    .companions
                    .iter()
                    .any(|companion| companion.id == *id && companion.current_hp == 0)
            })
            .collect();
        for id in knocked_out {
            let companion = next_state
                .companions
                .iter_mut()
                .find(|companion| companion.id == id)
                .ok_or(RecoveryError::CompanionNotRecruited(id))?;
            next_recovery.record_knockout(companion, world, &origin)?;
        }
        next_state.player.hp = next_state.player.max_hp;
        next_state.current_location = destination;
        next_state.party.clear();
        next_recovery.expedition_defeat_count =
            next_recovery.expedition_defeat_count.saturating_add(1);

        *state = next_state;
        *self = next_recovery;
        Ok(())
    }
}

/// Find the nearest treatment-capable settlement; currently every graph settlement
/// supports the same base treatment service. World::get_nearest_settlement already
/// minimises summed route travel cost and breaks ties by lexical LocationId.
pub fn nearest_treatment_settlement(world: &World, origin: &LocationId) -> Option<LocationId> {
    world.get_nearest_settlement(origin)
}

/// Every recruited companion whose condition and HP allow selection must be in
/// a compatible state. Knockout is never permanent deletion/death.
pub fn validate_recovery_invariants(state: &GameState, recovery: &RecoveryState) -> bool {
    for record in &recovery.records {
        let Some(companion) = state
            .companions
            .iter()
            .find(|c| c.id == record.companion_id)
        else {
            return false;
        };
        if companion.current_hp != 0
            || !matches!(
                companion.condition,
                CompanionCondition::KnockedOut | CompanionCondition::Recovering
            )
            || state.party.contains(companion.id)
        {
            return false;
        }
    }
    state.companions.iter().all(|companion| {
        (companion.current_hp > 0 || !companion.condition.can_join_party())
            && (!matches!(
                companion.condition,
                CompanionCondition::KnockedOut | CompanionCondition::Recovering
            ) || recovery.record(companion.id).is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::{CompanionId, PartyManager};
    use crate::state::{GameState, Player};
    use crate::world::build_initial_world;

    fn state_with_companion() -> GameState {
    let mut state = GameState::new(Player::new("tester".into(), String::new()));
    let cost = PartyManager::recruit(&mut state.companions, CompanionId::BranTheShield, 99, 1000).unwrap();
    state.player.gold -= cost;
    PartyManager::add_to_party(&mut state.party, &state.companions, CompanionId::BranTheShield).unwrap();
    state
    }

    #[test]
    fn knockout_is_idempotent_and_uses_nearest_reachable_settlement() {
    let world = build_initial_world().unwrap();
    let mut state = state_with_companion();
    state.companions[0].current_hp = 0;
    state.current_location = LocationId::from("whispering_woods");
    let mut recovery = RecoveryState::default();
    let id = CompanionId::BranTheShield;
    assert!(recovery.record_knockout_by_id(&mut state, id, &world).unwrap());
    let original = recovery.record(id).unwrap().clone();
    assert!(!recovery.record_knockout_by_id(&mut state, id, &world).unwrap());
    assert_eq!(recovery.record(id), Some(&original));
    assert_eq!(original.destination, Some(LocationId::from("village_start")));
    assert_eq!(state.companions[0].condition, CompanionCondition::KnockedOut);
    assert!(!state.party.contains(id));
    // This operation records the state only; the caller's main combat adapter removes
    // the companion from the active party as soon as it processes the knockout.
    assert!(validate_recovery_invariants(&state, &recovery));
    }

    #[test]
    fn no_reachable_settlement_is_safe_and_explicit() {
    let mut world = build_initial_world().unwrap();
    world.add_location(crate::world::Location::new(
        "isolated", "Isolated", "No route to a settlement.", LocationCategory::Wilderness
    )).unwrap();
    let mut state = state_with_companion();
    let gold_before = state.player.gold;
    state.companions[0].current_hp = 0;
    state.current_location = LocationId::from("isolated");
    let mut recovery = RecoveryState::default();
    recovery.record_knockout_by_id(&mut state, CompanionId::BranTheShield, &world).unwrap();
    assert_eq!(recovery.record(CompanionId::BranTheShield).unwrap().destination, None);
    let location = state.current_location.clone();
    assert!(matches!(recovery.apply_treatment(&mut state, CompanionId::BranTheShield, &location, &world), Err(RecoveryError::NoTreatmentSettlement)));
    assert_eq!(state.player.gold, gold_before);
    }

    #[test]
    fn failed_treatment_never_charges_gold_or_partially_restores() {
    let world = build_initial_world().unwrap();
    let mut state = state_with_companion();
    let id = CompanionId::BranTheShield;
    state.companions[0].current_hp = 0;
    let mut recovery = RecoveryState::default();
    recovery.record_knockout_by_id(&mut state, id, &world).unwrap();
    state.player.gold = TREATMENT_COST - 1;
    let before = state.clone();
    let location = state.current_location.clone();
    assert!(matches!(recovery.apply_treatment(&mut state, id, &location, &world), Err(RecoveryError::InsufficientGold { .. })));
    assert_eq!(state.player.gold, before.player.gold);
    assert_eq!(state.companions[0].current_hp, 0);
    assert_eq!(recovery.record(id).unwrap().turns_remaining, RECOVERY_TURNS);
    }

    #[test]
    fn paid_treatment_restores_once_and_requires_explicit_reselection() {
    let world = build_initial_world().unwrap();
    let mut state = state_with_companion();
    let id = CompanionId::BranTheShield;
    state.companions[0].current_hp = 0;
    let mut recovery = RecoveryState::default();
    recovery.record_knockout_by_id(&mut state, id, &world).unwrap();
    state.current_location = LocationId::from("village_start");
    let location = state.current_location.clone();
    assert_eq!(recovery.apply_treatment(&mut state, id, &location, &world), Ok(TREATMENT_COST));
    assert_eq!(state.companions[0].condition, CompanionCondition::Available);
    assert_eq!(state.companions[0].current_hp, state.companions[0].max_hp);
    assert_eq!(state.player.gold, 1000 - TREATMENT_COST - crate::party::get_definition(CompanionId::BranTheShield).recruitment_cost);
    assert!(recovery.record(id).is_none());
    let location = state.current_location.clone();
    assert!(recovery.apply_treatment(&mut state, id, &location, &world).is_err());
    PartyManager::add_to_party(&mut state.party, &state.companions, id).unwrap();
    }

    #[test]
    fn committed_turns_advance_recovery_but_menus_and_reload_do_not() {
    let mut state = state_with_companion();
    state.companions[0].current_hp = 0;
    let mut recovery = RecoveryState::default();
    let world = build_initial_world().unwrap();
    recovery.record_knockout_by_id(&mut state, CompanionId::BranTheShield, &world).unwrap();
    assert!(recovery.advance_committed_turn(&mut state.companions).is_empty());
    assert_eq!(recovery.record(CompanionId::BranTheShield).unwrap().turns_remaining, 2);
    let mut loaded: RecoveryState = serde_json::from_str(&serde_json::to_string(&recovery).unwrap()).unwrap();
    assert_eq!(loaded, recovery);
    assert_eq!(loaded.advance_committed_turn(&mut state.companions), Vec::<CompanionId>::new());
    assert_eq!(loaded.advance_committed_turn(&mut state.companions), vec![CompanionId::BranTheShield]);
    assert_eq!(state.companions[0].condition, CompanionCondition::Available);
    assert!(loaded.record(CompanionId::BranTheShield).is_none());
    }

 
    #[test]
    fn party_combat_knockout_persists_once_and_keeps_other_allies_active() {
    use crate::party_combat::{
        Allegiance, CombatAction, CombatantId, CombatantSource, Encounter, EncounterCombatant,
        EncounterOutcome,
    };
    let world = build_initial_world().unwrap();
    let mut state = state_with_companion();
    let id = CompanionId::BranTheShield;
    state.companions[0].current_hp = 1;
    let companion = &state.companions[0];
    let encounter = Encounter::new(vec![
        EncounterCombatant::new(
            CombatantId(1), CombatantSource::Player, "tester", Allegiance::Allies,
            state.player.hp, 100, 50, 20, 5
        ),
        EncounterCombatant::new(
            CombatantId(2), CombatantSource::Companion(id), companion.display_name(),
            Allegiance::Allies, 1, companion.current_stamina, companion.current_mana,
            companion.attack, companion.defence
        ),
        EncounterCombatant::new(
            CombatantId(3), CombatantSource::Enemy, "Goblin", Allegiance::Enemies,
            500, 0, 0, 500, 0
        ),
    ]).unwrap();
    let mut encounter = encounter;
    encounter.set_position(CombatantId(3), crate::party_combat::RangeBand::Melee).unwrap();
    let result = encounter.apply_action(
        CombatAction::BasicAttack { actor: CombatantId(3), target: CombatantId(2) },
        0, 99
    ).unwrap();
    assert_eq!(result.target_source, Some(CombatantSource::Companion(id)));
    assert!(result.target_knocked_out);
    assert_eq!(result.outcome, EncounterOutcome::Ongoing);

    let mut recovery = RecoveryState::default();
    let first = recovery.sync_party_encounter(&mut state, &encounter, &world).unwrap();
    assert_eq!(first, vec![id]);
    let original = recovery.record(id).unwrap().clone();
    assert_eq!(state.companions[0].condition, CompanionCondition::KnockedOut);
    assert!(!state.party.contains(id));
    assert!(recovery.sync_party_encounter(&mut state, &encounter, &world).unwrap().is_empty());
    assert_eq!(recovery.record(id), Some(&original));
    assert!(validate_recovery_invariants(&state, &recovery));
    }

    #[test]
    fn companion_without_treatment_destination_does_not_recover_from_elapsed_turns() {
    let mut world = build_initial_world().unwrap();
    world.add_location(crate::world::Location::new(
        "isolated_recovery", "Isolated recovery", "Disconnected.", LocationCategory::Wilderness
    )).unwrap();
    let mut state = state_with_companion();
    state.companions[0].current_hp = 0;
    state.current_location = LocationId::from("isolated_recovery");
    state.companions[0].current_hp = 0;
    let mut recovery = RecoveryState::default();
    let id = CompanionId::BranTheShield;
    recovery.record_knockout_by_id(&mut state, id, &world).unwrap();
    let before = recovery.record(id).unwrap().clone();
    for _ in 0..(RECOVERY_TURNS + 2) {
        assert!(recovery.advance_committed_turn(&mut state.companions).is_empty());
    }
    assert_eq!(recovery.record(id), Some(&before));
    assert_eq!(state.companions[0].condition, CompanionCondition::KnockedOut);
    }

    #[test]
    fn defeat_resolution_is_atomic_and_recoverable() {
    let world = build_initial_world().unwrap();
    let mut state = state_with_companion();
    state.player.hp = 0;
    state.companions[0].current_hp = 0;
    let mut recovery = RecoveryState::default();
    recovery.resolve_player_defeat(&mut state, &world).unwrap();
    assert_eq!(state.player.hp, state.player.max_hp);
    assert_eq!(state.current_location, LocationId::from("village_start"));
    assert!(!state.party.contains(CompanionId::BranTheShield));
    assert_eq!(state.companions[0].condition, CompanionCondition::KnockedOut);
    assert_eq!(recovery.expedition_defeat_count, 1);
    assert!(recovery.record(CompanionId::BranTheShield).is_some());
    }
}
