//! Persisted companion discovery, separate from recruitment and party membership.

use crate::event::GameEvent;
use crate::party::{all_companion_definitions, CompanionId};
use crate::state::GameState;

/// Whether a companion has been discovered in this save. Recruited records also
/// count as discovery so a pre-Section-25 save remains valid during migration.
pub fn is_discovered(state: &GameState, id: CompanionId) -> bool {
    state.discovered_companions.contains(&id)
        || state.companions.iter().any(|companion| companion.id == id)
}

/// Record one discovery. Repeated events are harmless and do not recruit the
/// companion or alter the active party.
pub fn discover(state: &mut GameState, id: CompanionId) -> bool {
    if is_discovered(state, id) {
        return false;
    }
    state.discovered_companions.push(id);
    true
}

/// Migrate only already-recruited companions from older saves. Do not reveal
/// every static definition or infer discovery from a party slot alone.
pub fn reconcile_legacy_discoveries(state: &mut GameState) {
    let mut known = Vec::new();
    for id in CompanionId::ALL {
        if is_discovered(state, id) {
            known.push(id);
        }
    }
    state.discovered_companions = known;
}

/// Definitions visible to the player. Discovered-but-unrecruited companions are
/// included as opportunities; all other definitions remain hidden.
pub fn discovered_definitions(state: &GameState) -> Vec<crate::party::CompanionDefinition> {
    all_companion_definitions()
        .into_iter()
        .filter(|definition| is_discovered(state, definition.id))
        .collect()
}

/// Authored discoveries use stable Section 19 quest, landmark and encounter IDs.
pub fn discovery_events_for(event: &GameEvent) -> Vec<GameEvent> {
    let companion = match event {
        GameEvent::QuestStageCompleted { quest_id, stage_id }
            if quest_id == "oakhaven_missing_supplies" && stage_id == "clear_the_ruins" =>
        {
            Some(CompanionId::BranTheShield)
        }
        GameEvent::TargetInteracted { target_id } if target_id == "watchtower_scout_report" => {
            Some(CompanionId::LyraSwiftbow)
        }
        GameEvent::TargetInteracted { target_id } if target_id == "standing_stones_survey" => {
            Some(CompanionId::MarenStarweave)
        }
        GameEvent::EncounterCleared { encounter_id } if encounter_id == "ruins_guardians" => {
            Some(CompanionId::GarrickIronfist)
        }
        GameEvent::EncounterCleared { encounter_id } if encounter_id == "woods_patrol" => {
            Some(CompanionId::SeraNightwhisper)
        }
        GameEvent::EncounterCleared { encounter_id } if encounter_id == "cave_expedition" => {
            Some(CompanionId::TorvalEmbershield)
        }
        _ => None,
    };
    companion
        .map(|companion_id| vec![GameEvent::CompanionDiscovered { companion_id }])
        .unwrap_or_default()
}

pub fn record_event(state: &mut GameState, event: &GameEvent) -> bool {
    match event {
        GameEvent::CompanionDiscovered { companion_id } => discover(state, *companion_id),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::{CompanionId, CompanionState};
    use crate::state::Player;

    fn state() -> GameState {
        GameState::new(Player::new("discovery-test".into(), "hash".into()))
    }

    #[test]
    fn fresh_save_hides_every_companion_definition() {
        let state = state();
        assert!(state.discovered_companions.is_empty());
        assert!(discovered_definitions(&state).is_empty());
        assert!(state.companions.is_empty());
    }

    #[test]
    fn discovery_is_idempotent_and_does_not_recruit() {
        let mut state = state();
        let event = GameEvent::CompanionDiscovered {
            companion_id: CompanionId::LyraSwiftbow,
        };
        assert!(record_event(&mut state, &event));
        assert!(!record_event(&mut state, &event));
        assert_eq!(state.discovered_companions, vec![CompanionId::LyraSwiftbow]);
        assert!(state.companions.is_empty());
        assert_eq!(discovered_definitions(&state).len(), 1);
    }

    #[test]
    fn authored_landmark_reveals_only_its_matching_companion() {
        let mut state = state();
        for event in discovery_events_for(&GameEvent::TargetInteracted {
            target_id: "watchtower_scout_report".into(),
        }) {
            record_event(&mut state, &event);
        }
        let visible = discovered_definitions(&state);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, CompanionId::LyraSwiftbow);
        assert!(!is_discovered(&state, CompanionId::MarenStarweave));
    }

    #[test]
    fn legacy_recruited_companions_migrate_without_revealing_other_definitions() {
        let mut state = state();
        state.companions.push(CompanionState::new_from_definition(
            &crate::party::get_definition(CompanionId::BranTheShield),
        ));
        reconcile_legacy_discoveries(&mut state);
        assert_eq!(
            state.discovered_companions,
            vec![CompanionId::BranTheShield]
        );
        assert_eq!(discovered_definitions(&state).len(), 1);
    }

    #[test]
    fn quest_and_encounter_triggers_use_stable_ids() {
        let quest_events = discovery_events_for(&GameEvent::QuestStageCompleted {
            quest_id: "oakhaven_missing_supplies".into(),
            stage_id: "clear_the_ruins".into(),
        });
        assert_eq!(
            quest_events,
            vec![GameEvent::CompanionDiscovered {
                companion_id: CompanionId::BranTheShield,
            }]
        );
        assert!(discovery_events_for(&GameEvent::EncounterCleared {
            encounter_id: "unknown_encounter".into(),
        })
        .is_empty());
    }
}
