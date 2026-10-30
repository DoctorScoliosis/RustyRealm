//! Persistent, data-driven campaign progression and the Lacoyx realm gate.
//!
//! Realm and objective IDs are save-facing contracts. Display names may change,
//! but these IDs must remain stable as later realm content is added.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::event::GameEvent;
use crate::state::GameState;
use crate::world::LocationId;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CampaignProgress {
    /// Entries are created only when a realm is discovered or an objective is
    /// evidenced. Missing entries are deliberately treated as undiscovered.
    pub realms: HashMap<String, RealmProgress>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RealmProgress {
    pub discovered: bool,
    /// Stable objective IDs completed by real gameplay events or trusted
    /// evidence recovered from an older save.
    pub completed_objectives: Vec<String>,
    /// Derived from discovery plus all mandatory objectives.
    pub completed: bool,
    /// Derived from completion plus every explicitly optional mastery objective.
    pub mastered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmObjectiveRequirement {
    QuestCompleted {
        quest_id: &'static str,
    },
    QuestStageCompleted {
        quest_id: &'static str,
        stage_id: &'static str,
    },
    EncounterCleared {
        encounter_id: &'static str,
    },
    LandmarkInteracted {
        target_id: &'static str,
    },
    RealmTaskCompleted {
        task_id: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealmObjectiveDefinition {
    pub id: &'static str,
    pub description: &'static str,
    /// Required objectives define realm completion. Optional objectives define
    /// mastery and can never block the Lacoyx gate.
    pub required: bool,
    pub requirement: RealmObjectiveRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealmDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub entry_location_id: &'static str,
    pub prerequisite_realm_ids: &'static [&'static str],
    pub required_for_lacoyx: bool,
    pub objectives: &'static [RealmObjectiveDefinition],
}

const OAKHAVEN_OBJECTIVES: &[RealmObjectiveDefinition] = &[
    RealmObjectiveDefinition {
        id: "ruins_stage",
        description: "Complete the Sunken Ruins stage of Missing Supplies",
        required: true,
        requirement: RealmObjectiveRequirement::QuestStageCompleted {
            quest_id: "oakhaven_missing_supplies",
            stage_id: "clear_the_ruins",
        },
    },
    RealmObjectiveDefinition {
        id: "main_quest",
        description: "Resolve Oakhaven's Missing Supplies quest",
        required: true,
        requirement: RealmObjectiveRequirement::QuestCompleted {
            quest_id: "oakhaven_missing_supplies",
        },
    },
    RealmObjectiveDefinition {
        id: "signature_encounter",
        description: "Clear the Sunken Ruins guardians",
        required: true,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "ruins_guardians",
        },
    },
    RealmObjectiveDefinition {
        id: "watchtower_survey",
        description: "Survey the Crumbling Watchtower",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "watchtower_scout_report",
        },
    },
    RealmObjectiveDefinition {
        id: "mill_cache",
        description: "Inspect the Abandoned Mill",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "abandoned_mill_cache",
        },
    },
    RealmObjectiveDefinition {
        id: "woods_patrol",
        description: "Clear the Whispering Woods patrol",
        required: false,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "woods_patrol",
        },
    },
    RealmObjectiveDefinition {
        id: "cave_expedition",
        description: "Clear the Goblin Caves expedition",
        required: false,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "cave_expedition",
        },
    },
];

const THORNVEIL_OBJECTIVES: &[RealmObjectiveDefinition] = &[
    RealmObjectiveDefinition {
        id: "main_quest",
        description: "Complete Thornveil's main quest",
        required: true,
        requirement: RealmObjectiveRequirement::QuestCompleted {
            quest_id: "thornveil_main_quest",
        },
    },
    RealmObjectiveDefinition {
        id: "signature_encounter",
        description: "Defeat Thornveil's signature encounter",
        required: true,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "thornveil_signature_encounter",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_landmark",
        description: "Investigate Thornveil's optional landmark",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "thornveil_observation_post",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_mastery_trial",
        description: "Complete Thornveil's optional survival trial",
        required: false,
        requirement: RealmObjectiveRequirement::RealmTaskCompleted {
            task_id: "thornveil_survival_trial",
        },
    },
];

const IRONVEIN_OBJECTIVES: &[RealmObjectiveDefinition] = &[
    RealmObjectiveDefinition {
        id: "main_quest",
        description: "Complete Ironvein's main quest",
        required: true,
        requirement: RealmObjectiveRequirement::QuestCompleted {
            quest_id: "ironvein_main_quest",
        },
    },
    RealmObjectiveDefinition {
        id: "signature_encounter",
        description: "Defeat Ironvein's signature encounter",
        required: true,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "ironvein_signature_encounter",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_landmark",
        description: "Survey Ironvein's forge landmark",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "ironvein_forge_survey",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_mastery_trial",
        description: "Complete Ironvein's optional masterwork trial",
        required: false,
        requirement: RealmObjectiveRequirement::RealmTaskCompleted {
            task_id: "ironvein_masterwork_trial",
        },
    },
];

const DROWNED_REACH_OBJECTIVES: &[RealmObjectiveDefinition] = &[
    RealmObjectiveDefinition {
        id: "main_quest",
        description: "Complete Drowned Reach's main quest",
        required: true,
        requirement: RealmObjectiveRequirement::QuestCompleted {
            quest_id: "drowned_reach_main_quest",
        },
    },
    RealmObjectiveDefinition {
        id: "signature_encounter",
        description: "Defeat Drowned Reach's signature encounter",
        required: true,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "drowned_reach_signature_encounter",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_landmark",
        description: "Investigate Drowned Reach's archive landmark",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "drowned_reach_archive_survey",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_mastery_trial",
        description: "Complete Drowned Reach's optional evidence challenge",
        required: false,
        requirement: RealmObjectiveRequirement::RealmTaskCompleted {
            task_id: "drowned_reach_evidence_mastery",
        },
    },
];

const ASHEN_MARCH_OBJECTIVES: &[RealmObjectiveDefinition] = &[
    RealmObjectiveDefinition {
        id: "main_quest",
        description: "Complete Ashen March's main quest",
        required: true,
        requirement: RealmObjectiveRequirement::QuestCompleted {
            quest_id: "ashen_march_main_quest",
        },
    },
    RealmObjectiveDefinition {
        id: "signature_encounter",
        description: "Defeat Ashen March's signature encounter",
        required: true,
        requirement: RealmObjectiveRequirement::EncounterCleared {
            encounter_id: "ashen_march_signature_encounter",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_landmark",
        description: "Survey Ashen March's command landmark",
        required: false,
        requirement: RealmObjectiveRequirement::LandmarkInteracted {
            target_id: "ashen_march_command_survey",
        },
    },
    RealmObjectiveDefinition {
        id: "optional_mastery_trial",
        description: "Complete Ashen March's optional elite trial",
        required: false,
        requirement: RealmObjectiveRequirement::RealmTaskCompleted {
            task_id: "ashen_march_elite_trial",
        },
    },
];

/// Campaign registry. The four later entry-location, quest, encounter, landmark
/// and task IDs are reserved contracts for Sections 28 and 30–39. Their content
/// is intentionally not fabricated here; until those sections ship, they remain
/// undiscovered and therefore keep the Stronghold sealed.
pub const REALMS: &[RealmDefinition] = &[
    RealmDefinition {
        id: "oakhaven",
        name: "Oakhaven",
        entry_location_id: "village_start",
        prerequisite_realm_ids: &[],
        required_for_lacoyx: true,
        objectives: OAKHAVEN_OBJECTIVES,
    },
    RealmDefinition {
        id: "thornveil_wilds",
        name: "Thornveil Wilds",
        entry_location_id: "thornveil_entry",
        prerequisite_realm_ids: &["oakhaven"],
        required_for_lacoyx: true,
        objectives: THORNVEIL_OBJECTIVES,
    },
    RealmDefinition {
        id: "ironvein_marches",
        name: "Ironvein Marches",
        entry_location_id: "ironvein_entry",
        prerequisite_realm_ids: &["thornveil_wilds"],
        required_for_lacoyx: true,
        objectives: IRONVEIN_OBJECTIVES,
    },
    RealmDefinition {
        id: "drowned_reach",
        name: "Drowned Reach",
        entry_location_id: "drowned_reach_entry",
        prerequisite_realm_ids: &["ironvein_marches"],
        required_for_lacoyx: true,
        objectives: DROWNED_REACH_OBJECTIVES,
    },
    RealmDefinition {
        id: "ashen_march",
        name: "Ashen March",
        entry_location_id: "ashen_march_entry",
        prerequisite_realm_ids: &["drowned_reach"],
        required_for_lacoyx: true,
        objectives: ASHEN_MARCH_OBJECTIVES,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrongholdGateStatus {
    pub unlocked: bool,
    pub message: String,
}

fn progress_for<'a>(state: &'a GameState, realm_id: &str) -> Option<&'a RealmProgress> {
    state.campaign_progress.realms.get(realm_id)
}

fn progress_for_mut<'a>(state: &'a mut GameState, realm_id: &str) -> &'a mut RealmProgress {
    state
        .campaign_progress
        .realms
        .entry(realm_id.to_string())
        .or_default()
}

fn objective_completed(progress: &RealmProgress, objective_id: &str) -> bool {
    progress
        .completed_objectives
        .iter()
        .any(|completed| completed == objective_id)
}

fn prerequisites_completed(state: &GameState, realm: &RealmDefinition) -> bool {
    realm.prerequisite_realm_ids.iter().all(|required_id| {
        REALMS.iter().any(|known| known.id == *required_id)
            && realm_is_completed(state, required_id)
    })
}

fn realm_is_completed(state: &GameState, realm_id: &str) -> bool {
    let Some(realm) = REALMS.iter().find(|realm| realm.id == realm_id) else {
        return false;
    };
    let Some(progress) = progress_for(state, realm_id) else {
        return false;
    };
    let required: Vec<_> = realm
        .objectives
        .iter()
        .filter(|objective| objective.required)
        .collect();
    progress.discovered
        && !required.is_empty()
        && required
            .iter()
            .all(|objective| objective_completed(progress, objective.id))
}

fn recompute_realm_progress(state: &mut GameState, realm: &RealmDefinition) {
    let Some(progress) = state.campaign_progress.realms.get_mut(realm.id) else {
        return;
    };
    let required: Vec<_> = realm
        .objectives
        .iter()
        .filter(|objective| objective.required)
        .collect();
    let mastery: Vec<_> = realm
        .objectives
        .iter()
        .filter(|objective| !objective.required)
        .collect();

    progress.completed = progress.discovered
        && !required.is_empty()
        && required
            .iter()
            .all(|objective| objective_completed(progress, objective.id));
    progress.mastered = progress.completed
        && !mastery.is_empty()
        && mastery
            .iter()
            .all(|objective| objective_completed(progress, objective.id));
}

fn insert_objective_once(state: &mut GameState, realm_id: &str, objective_id: &str) {
    let progress = progress_for_mut(state, realm_id);
    if !progress
        .completed_objectives
        .iter()
        .any(|completed| completed == objective_id)
    {
        progress.completed_objectives.push(objective_id.to_string());
    }
}

fn event_matches_requirement(
    realm_id: &str,
    requirement: RealmObjectiveRequirement,
    event: &GameEvent,
) -> bool {
    match (requirement, event) {
        (
            RealmObjectiveRequirement::QuestCompleted { quest_id },
            GameEvent::PrerequisiteQuestCompleted {
                quest_id: completed_id,
            },
        ) => quest_id == completed_id,
        (
            RealmObjectiveRequirement::QuestStageCompleted { quest_id, stage_id },
            GameEvent::QuestStageCompleted {
                quest_id: completed_quest,
                stage_id: completed_stage,
            },
        ) => quest_id == completed_quest && stage_id == completed_stage,
        (
            RealmObjectiveRequirement::EncounterCleared { encounter_id },
            GameEvent::EncounterCleared {
                encounter_id: cleared_id,
            },
        ) => encounter_id == cleared_id,
        (
            RealmObjectiveRequirement::LandmarkInteracted { target_id },
            GameEvent::TargetInteracted {
                target_id: interacted_id,
            },
        ) => target_id == interacted_id,
        (
            RealmObjectiveRequirement::RealmTaskCompleted { task_id },
            GameEvent::RealmTaskCompleted {
                realm_id: completed_realm,
                task_id: completed_task,
            },
        ) => realm_id == completed_realm && task_id == completed_task,
        _ => false,
    }
}

/// Record a successful gameplay event against matching objectives. Repeated
/// events are harmless: each stable objective ID is stored at most once.
pub fn record_event(state: &mut GameState, event: &GameEvent) {
    if let GameEvent::LocationReached { location_id } = event {
        for realm in REALMS {
            if realm.entry_location_id == location_id.as_str()
                && prerequisites_completed(state, realm)
            {
                progress_for_mut(state, realm.id).discovered = true;
            }
        }
    }

    for realm in REALMS {
        let matched_objectives: Vec<_> = realm
            .objectives
            .iter()
            .filter(|objective| event_matches_requirement(realm.id, objective.requirement, event))
            .map(|objective| objective.id)
            .collect();
        if !matched_objectives.is_empty() {
            // A successful realm-specific quest, encounter, landmark or task
            // event is itself evidence that the player discovered that realm.
            progress_for_mut(state, realm.id).discovered = true;
            for objective_id in matched_objectives {
                insert_objective_once(state, realm.id, objective_id);
            }
        }
    }

    reconcile_from_saved_evidence(state);
}

/// Rebuild progression from evidence already persisted by Sections 19 and
/// earlier quest systems. Missing campaign data never implies completion.
pub fn reconcile_from_saved_evidence(state: &mut GameState) {
    for realm in REALMS {
        let prerequisites_met = prerequisites_completed(state, realm);
        if prerequisites_met && state.current_location.as_str() == realm.entry_location_id {
            progress_for_mut(state, realm.id).discovered = true;
        }

        for objective in realm.objectives {
            if objective_has_saved_evidence(state, objective.requirement) {
                // Older saves have no campaign-discovery flag. A persisted
                // quest, encounter or landmark record proves the realm was found,
                // even when the player has since travelled away from its entry.
                progress_for_mut(state, realm.id).discovered = true;
                insert_objective_once(state, realm.id, objective.id);
            }
        }

        // REALMS is topologically ordered, so a newly reconstructed completion
        // can unlock discovery of the following realm during this same pass.
        recompute_realm_progress(state, realm);
    }
}

fn objective_has_saved_evidence(state: &GameState, requirement: RealmObjectiveRequirement) -> bool {
    match requirement {
        RealmObjectiveRequirement::QuestCompleted { quest_id } => {
            state
                .player
                .quests_completed
                .iter()
                .any(|completed| completed == quest_id)
                || state
                    .active_quests
                    .iter()
                    .any(|quest| quest.id == quest_id && (quest.is_completed || quest.is_claimed))
        }
        RealmObjectiveRequirement::QuestStageCompleted { quest_id, stage_id } => {
            state
                .player
                .quests_completed
                .iter()
                .any(|completed| completed == quest_id)
                || state.active_quests.iter().any(|quest| {
                    quest.id == quest_id
                        && (quest.is_completed
                            || quest
                                .stages
                                .iter()
                                .any(|stage| stage.id == stage_id && stage.is_completed))
                })
        }
        RealmObjectiveRequirement::EncounterCleared { encounter_id } => state
            .region_progress
            .cleared_encounters
            .iter()
            .any(|cleared| cleared == encounter_id),
        RealmObjectiveRequirement::LandmarkInteracted { target_id } => state
            .region_progress
            .landmark_interactions
            .iter()
            .any(|interacted| interacted == target_id),
        RealmObjectiveRequirement::RealmTaskCompleted { .. } => false,
    }
}

/// Future realm content can report a completed task through the same event path
/// used by quests, encounters and landmarks.
pub fn notify_realm_task_completed(state: &mut GameState, realm_id: &str, task_id: &str) {
    crate::quests::notify_event(
        state,
        &GameEvent::RealmTaskCompleted {
            realm_id: realm_id.to_string(),
            task_id: task_id.to_string(),
        },
    );
}

fn evaluate_gate_with_registry(
    state: &GameState,
    registry: &[RealmDefinition],
) -> StrongholdGateStatus {
    let mandatory: Vec<_> = registry
        .iter()
        .filter(|realm| realm.required_for_lacoyx)
        .collect();
    if mandatory.is_empty() {
        return StrongholdGateStatus {
            unlocked: false,
            message: "The Lacoyx Gate remains sealed because no mandatory realm requirements are configured.".into(),
        };
    }

    let mut undiscovered_required = false;
    let mut outstanding = Vec::new();

    for realm in mandatory {
        let Some(progress) = progress_for(state, realm.id) else {
            undiscovered_required = true;
            continue;
        };
        if !progress.discovered {
            undiscovered_required = true;
            continue;
        }

        let required: Vec<_> = realm
            .objectives
            .iter()
            .filter(|objective| objective.required)
            .collect();
        if required.is_empty() {
            outstanding.push(format!(
                "{}: mandatory completion objectives are not configured.",
                realm.name
            ));
            continue;
        }

        for objective in required {
            if !objective_completed(progress, objective.id) {
                outstanding.push(format!("{}: {}", realm.name, objective.description));
            }
        }
    }

    let unlocked = !undiscovered_required && outstanding.is_empty();
    if unlocked {
        return StrongholdGateStatus {
            unlocked: true,
            message:
                "Every required realm has been discovered and completed. The Lacoyx Gate is open."
                    .into(),
        };
    }

    let mut message = String::from("The Lacoyx Gate remains sealed.");
    if !outstanding.is_empty() {
        message.push_str("\n\nOutstanding requirements:\n");
        for requirement in outstanding {
            message.push_str("• ");
            message.push_str(&requirement);
            message.push('\n');
        }
    }
    if undiscovered_required {
        message.push_str(
            "\nFurther mandatory realms remain undiscovered. Their names and objectives will appear here when you find them.\n",
        );
    }
    message.push_str("\nOptional realm mastery never blocks entry.");
    StrongholdGateStatus { unlocked, message }
}

/// Side-effect-free gate evaluation. Call reconciliation at a load or gameplay
/// boundary before relying on this result for a legacy save.
pub fn stronghold_gate_status(state: &GameState) -> StrongholdGateStatus {
    evaluate_gate_with_registry(state, REALMS)
}

/// Player-facing Quest Hall summary. Undiscovered realm names and objectives are
/// intentionally omitted so campaign progress does not spoil unexplored places.
pub fn quest_hall_summary(state: &GameState) -> String {
    let discovered: Vec<_> = REALMS
        .iter()
        .filter(|realm| progress_for(state, realm.id).is_some_and(|progress| progress.discovered))
        .collect();

    let mut lines = vec!["Realm progression".to_string()];
    if discovered.is_empty() {
        lines.push("No realms discovered yet. Explore to reveal the campaign requirements.".into());
    } else {
        for realm in discovered {
            let progress = progress_for(state, realm.id).expect("filtered above");
            let status = if progress.mastered {
                "Mastered"
            } else if progress.completed {
                "Completed"
            } else {
                "In progress"
            };
            let required_total = realm
                .objectives
                .iter()
                .filter(|objective| objective.required)
                .count();
            let required_done = realm
                .objectives
                .iter()
                .filter(|objective| {
                    objective.required && objective_completed(progress, objective.id)
                })
                .count();
            let mastery_total = realm
                .objectives
                .iter()
                .filter(|objective| !objective.required)
                .count();
            let mastery_done = realm
                .objectives
                .iter()
                .filter(|objective| {
                    !objective.required && objective_completed(progress, objective.id)
                })
                .count();

            lines.push(format!(
                "\n{} — {}\n  Required: {required_done}/{required_total} · Optional mastery: {mastery_done}/{mastery_total}",
                realm.name, status
            ));
            for objective in realm.objectives {
                let mark = if objective_completed(progress, objective.id) {
                    "✓"
                } else {
                    "·"
                };
                let kind = if objective.required {
                    "Required"
                } else {
                    "Mastery"
                };
                lines.push(format!("  {mark} {kind}: {}", objective.description));
            }
        }
    }

    let status = stronghold_gate_status(state);
    lines.push(format!("\n{}", status.message));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::{RegionProgress, REGION_QUEST_ID};
    use crate::state::Player;

    fn fresh_state() -> GameState {
        let mut state = GameState::new(Player::new("realm-test".into(), "hash".into()));
        state.active_quests.clear();
        state
    }

    fn mark_realm_complete(state: &mut GameState, realm: &RealmDefinition) {
        let progress = progress_for_mut(state, realm.id);
        progress.discovered = true;
        for objective in realm
            .objectives
            .iter()
            .filter(|objective| objective.required)
        {
            if !progress
                .completed_objectives
                .iter()
                .any(|id| id == objective.id)
            {
                progress.completed_objectives.push(objective.id.to_string());
            }
        }
        recompute_realm_progress(state, realm);
    }

    #[test]
    fn a_fresh_save_is_gated_and_does_not_assume_realm_completion() {
        let mut state = fresh_state();
        reconcile_from_saved_evidence(&mut state);

        assert!(!stronghold_gate_status(&state).unlocked);
        let oakhaven = progress_for(&state, "oakhaven").unwrap();
        assert!(oakhaven.discovered);
        assert!(!oakhaven.completed);
        assert_eq!(state.campaign_progress.realms.len(), 1);
        assert!(!state
            .player
            .quests_completed
            .contains(&REGION_QUEST_ID.to_string()));
    }

    #[test]
    fn entry_discovers_oakhaven_but_does_not_complete_it() {
        let mut state = fresh_state();
        record_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("village_start"),
            },
        );

        let progress = progress_for(&state, "oakhaven").unwrap();
        assert!(progress.discovered);
        assert!(!progress.completed);
        assert!(!progress.mastered);
    }

    #[test]
    fn quest_stage_quest_and_signature_encounter_are_distinct_requirements() {
        let mut state = fresh_state();
        record_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("village_start"),
            },
        );
        record_event(
            &mut state,
            &GameEvent::QuestStageCompleted {
                quest_id: REGION_QUEST_ID.into(),
                stage_id: "clear_the_ruins".into(),
            },
        );
        record_event(
            &mut state,
            &GameEvent::PrerequisiteQuestCompleted {
                quest_id: REGION_QUEST_ID.into(),
            },
        );

        assert!(!realm_is_completed(&state, "oakhaven"));

        record_event(
            &mut state,
            &GameEvent::EncounterCleared {
                encounter_id: "ruins_guardians".into(),
            },
        );
        let progress = progress_for(&state, "oakhaven").unwrap();
        assert!(progress.completed);
        assert!(!progress.mastered);
    }

    #[test]
    fn duplicate_events_do_not_duplicate_objectives_or_issue_rewards() {
        let mut state = fresh_state();
        record_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("village_start"),
            },
        );
        let gold_before = state.player.gold;
        let clear = GameEvent::EncounterCleared {
            encounter_id: "ruins_guardians".into(),
        };
        record_event(&mut state, &clear);
        record_event(&mut state, &clear);

        let progress = progress_for(&state, "oakhaven").unwrap();
        assert_eq!(
            progress
                .completed_objectives
                .iter()
                .filter(|id| id.as_str() == "signature_encounter")
                .count(),
            1
        );
        assert_eq!(state.player.gold, gold_before);
    }

    #[test]
    fn all_mandatory_realms_are_required_but_optional_objectives_are_not() {
        let mut state = fresh_state();
        for realm in REALMS {
            mark_realm_complete(&mut state, realm);
        }

        assert!(stronghold_gate_status(&state).unlocked);
        let oakhaven = progress_for(&state, "oakhaven").unwrap();
        assert!(oakhaven.completed);
        assert!(!oakhaven.mastered);
    }

    #[test]
    fn optional_objectives_advance_mastery_without_changing_required_completion() {
        let mut state = fresh_state();
        let oakhaven = REALMS.iter().find(|realm| realm.id == "oakhaven").unwrap();
        mark_realm_complete(&mut state, oakhaven);
        assert!(realm_is_completed(&state, "oakhaven"));
        assert!(!progress_for(&state, "oakhaven").unwrap().mastered);

        for event in [
            GameEvent::TargetInteracted {
                target_id: "watchtower_scout_report".into(),
            },
            GameEvent::TargetInteracted {
                target_id: "abandoned_mill_cache".into(),
            },
            GameEvent::EncounterCleared {
                encounter_id: "woods_patrol".into(),
            },
            GameEvent::EncounterCleared {
                encounter_id: "cave_expedition".into(),
            },
        ] {
            record_event(&mut state, &event);
        }

        let progress = progress_for(&state, "oakhaven").unwrap();
        assert!(progress.completed);
        assert!(progress.mastered);
    }

    #[test]
    fn quest_hall_summary_does_not_reveal_undiscovered_realms() {
        let mut state = fresh_state();
        reconcile_from_saved_evidence(&mut state);
        let summary = quest_hall_summary(&state);

        assert!(summary.contains("Oakhaven"));
        assert!(!summary.contains("Thornveil"));
        assert!(!summary.contains("Ironvein"));
        assert!(!summary.contains("Drowned Reach"));
        assert!(!summary.contains("Ashen March"));
    }

    #[test]
    fn adding_a_required_realm_extends_the_gate_without_new_gate_logic() {
        static EXTRA_OBJECTIVES: &[RealmObjectiveDefinition] = &[RealmObjectiveDefinition {
            id: "main_quest",
            description: "Complete the added realm's quest",
            required: true,
            requirement: RealmObjectiveRequirement::QuestCompleted {
                quest_id: "added_realm_quest",
            },
        }];
        static EXTRA_REALM: RealmDefinition = RealmDefinition {
            id: "added_required_realm",
            name: "Added Realm",
            entry_location_id: "added_realm_entry",
            prerequisite_realm_ids: &[],
            required_for_lacoyx: true,
            objectives: EXTRA_OBJECTIVES,
        };

        let mut state = fresh_state();
        for realm in REALMS {
            mark_realm_complete(&mut state, realm);
        }
        let mut expanded_registry = REALMS.to_vec();
        expanded_registry.push(EXTRA_REALM);

        assert!(!evaluate_gate_with_registry(&state, &expanded_registry).unlocked);
        mark_realm_complete(&mut state, &EXTRA_REALM);
        assert!(evaluate_gate_with_registry(&state, &expanded_registry).unlocked);
    }

    #[test]
    fn optional_realms_do_not_block_the_gate() {
        static OPTIONAL_REALM: RealmDefinition = RealmDefinition {
            id: "optional_realm",
            name: "Optional Realm",
            entry_location_id: "optional_realm_entry",
            prerequisite_realm_ids: &[],
            required_for_lacoyx: false,
            objectives: &[],
        };

        let mut state = fresh_state();
        for realm in REALMS {
            mark_realm_complete(&mut state, realm);
        }
        let mut registry = REALMS.to_vec();
        registry.push(OPTIONAL_REALM);
        assert!(evaluate_gate_with_registry(&state, &registry).unlocked);
    }

    #[test]
    fn legacy_region_and_quest_evidence_reconstructs_only_oakhaven() {
        let mut state = fresh_state();
        state.current_location = LocationId::from("thornveil_entry");
        state.player.quests_completed.push(REGION_QUEST_ID.into());
        state.region_progress = RegionProgress {
            cleared_encounters: vec!["ruins_guardians".into()],
            landmark_interactions: vec!["standing_stones_survey".into()],
        };

        reconcile_from_saved_evidence(&mut state);

        let oakhaven = progress_for(&state, "oakhaven").unwrap();
        assert!(oakhaven.discovered);
        assert!(oakhaven.completed);
        assert!(!progress_for(&state, "thornveil_wilds").is_some_and(|progress| progress.completed));
    }

    #[test]
    fn old_save_without_campaign_progress_loads_safely_and_stays_gated() {
        let state = fresh_state();
        let mut value = serde_json::to_value(state).unwrap();
        value.as_object_mut().unwrap().remove("campaign_progress");

        let mut loaded: GameState = serde_json::from_value(value).unwrap();
        assert!(loaded.campaign_progress.realms.is_empty());
        reconcile_from_saved_evidence(&mut loaded);
        assert!(!stronghold_gate_status(&loaded).unlocked);
    }

    #[test]
    fn realm_task_events_are_scoped_to_their_realm_and_idempotent() {
        let mut state = fresh_state();
        record_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("thornveil_entry"),
            },
        );
        // Oakhaven has not been completed, so Thornveil is not yet discovered.
        assert!(
            !progress_for(&state, "thornveil_wilds").is_some_and(|progress| progress.discovered)
        );

        for realm in REALMS.iter().take(1) {
            mark_realm_complete(&mut state, realm);
        }
        record_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("thornveil_entry"),
            },
        );
        let task = GameEvent::RealmTaskCompleted {
            realm_id: "thornveil_wilds".into(),
            task_id: "thornveil_survival_trial".into(),
        };
        record_event(&mut state, &task);
        record_event(&mut state, &task);

        let progress = progress_for(&state, "thornveil_wilds").unwrap();
        assert_eq!(
            progress
                .completed_objectives
                .iter()
                .filter(|id| id.as_str() == "optional_mastery_trial")
                .count(),
            1
        );
    }
}
