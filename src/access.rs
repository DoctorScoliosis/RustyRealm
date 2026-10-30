//! Declarative, side-effect-free access checks for world entrances and quest stages.

use crate::data::ItemType;
use crate::party::CompanionId;
use crate::state::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeKind {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AttributeKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
    }

    fn value(self, state: &GameState) -> u8 {
        let attributes = state.player.attributes;
        match self {
            Self::Strength => attributes.strength,
            Self::Dexterity => attributes.dexterity,
            Self::Constitution => attributes.constitution,
            Self::Intelligence => attributes.intelligence,
            Self::Wisdom => attributes.wisdom,
            Self::Charisma => attributes.charisma,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillKind {
    Woodcutting,
    Cooking,
    Fishing,
    Mining,
    Smithing,
    Thieving,
}

impl SkillKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Woodcutting => "Woodcutting",
            Self::Cooking => "Cooking",
            Self::Fishing => "Fishing",
            Self::Mining => "Mining",
            Self::Smithing => "Smithing",
            Self::Thieving => "Thieving",
        }
    }

    fn level(self, state: &GameState) -> u32 {
        let skills = &state.player.skills;
        match self {
            Self::Woodcutting => skills.woodcutting.level,
            Self::Cooking => skills.cooking.level,
            Self::Fishing => skills.fishing.level,
            Self::Mining => skills.mining.level,
            Self::Smithing => skills.smithing.level,
            Self::Thieving => skills.thieving.level,
        }
    }
}

/// Stable, typed requirements. Unknown IDs and malformed thresholds fail closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessRequirement {
    RealmDiscovered { realm_id: String },
    RealmCompleted { realm_id: String },
    QuestCompleted { quest_id: String },
    QuestStageCompleted { quest_id: String, stage_id: String },
    LandmarkInteracted { target_id: String },
    ItemCount { item: ItemType, minimum: u32 },
    AttributeAtLeast { attribute: AttributeKind, minimum: u8 },
    SkillAtLeast { skill: SkillKind, minimum: u32 },
    CompanionDiscovered { companion_id: CompanionId },
    CompanionRecruited { companion_id: CompanionId },
    PartyIncludes { companion_id: CompanionId },
    TimeWindow { start_hour: u8, end_hour: u8 },
    Season { season_index: u8 },
}

/// Costs are checked before entry and deducted only after the entry succeeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemCost {
    pub item: ItemType,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccessGate {
    #[serde(default)]
    pub all_of: Vec<AccessRequirement>,
    #[serde(default)]
    pub any_of: Vec<AccessRequirement>,
    #[serde(default)]
    pub consume_on_success: Vec<ItemCost>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessResult {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

impl AccessResult {
    fn allowed() -> Self {
        Self { allowed: true, reasons: Vec::new() }
    }

    fn denied(reasons: Vec<String>) -> Self {
        Self { allowed: false, reasons }
    }

    pub fn summary(&self) -> String {
        if self.allowed {
            "All access requirements are met.".into()
        } else {
            self.reasons.join("\n")
        }
    }
}

fn known_quest(state: &GameState, quest_id: &str) -> bool {
    !quest_id.trim().is_empty()
        && (state.active_quests.iter().any(|quest| quest.id == quest_id)
            || state.player.quests_completed.iter().any(|id| id == quest_id))
}

fn known_landmark(target_id: &str) -> bool {
    target_id == "village_supply_board"
        || crate::adventure::LANDMARKS
            .iter()
            .any(|landmark| landmark.interaction_target == target_id)
}

fn requirement_failure(state: &GameState, requirement: &AccessRequirement) -> Option<String> {
    match requirement {
        AccessRequirement::RealmDiscovered { realm_id } => {
            if !crate::realm_progression::realm_exists(realm_id) {
                Some("This entrance references an unknown realm.".into())
            } else if !crate::realm_progression::realm_is_discovered(state, realm_id) {
                Some("A required realm has not yet been discovered.".into())
            } else { None }
        }
        AccessRequirement::RealmCompleted { realm_id } => {
            if !crate::realm_progression::realm_exists(realm_id) {
                Some("This entrance references an unknown realm.".into())
            } else if !crate::realm_progression::realm_is_completed(state, realm_id) {
                Some("A required realm has not yet been completed.".into())
            } else { None }
        }
        AccessRequirement::QuestCompleted { quest_id } => {
            if !known_quest(state, quest_id) {
                Some("This entrance references an unknown quest.".into())
            } else if state.player.quests_completed.iter().any(|id| id == quest_id)
                || state.active_quests.iter().any(|quest| quest.id == *quest_id && quest.is_completed)
            {
                None
            } else { Some("A required quest has not been completed.".into()) }
        }
        AccessRequirement::QuestStageCompleted { quest_id, stage_id } => {
            let Some(quest) = state.active_quests.iter().find(|quest| quest.id == *quest_id) else {
                return Some("This entrance references an unknown quest stage.".into());
            };
            let Some(stage) = quest.stages.iter().find(|stage| stage.id == *stage_id) else {
                return Some("This entrance references an unknown quest stage.".into());
            };
            if stage.is_completed { None } else { Some("A required quest stage has not been completed.".into()) }
        }
        AccessRequirement::LandmarkInteracted { target_id } => {
            if !known_landmark(target_id) {
                Some("This entrance references an unknown landmark.".into())
            } else if state.region_progress.landmark_interactions.iter().any(|id| id == target_id) {
                None
            } else { Some("A required landmark has not been inspected.".into()) }
        }
        AccessRequirement::ItemCount { item, minimum } => {
            if *minimum == 0 {
                Some("This entrance has an invalid item requirement.".into())
            } else {
                let current = state.player.get_item_count(*item);
                (current < *minimum).then(|| format!("You need {} more {}.", minimum - current, item.name()))
            }
        }
        AccessRequirement::AttributeAtLeast { attribute, minimum } => {
            if !(1..=20).contains(minimum) {
                Some("This entrance has an invalid attribute threshold.".into())
            } else {
                let current = attribute.value(state);
                (current < *minimum).then(|| format!("{} must be at least {} (currently {}).", attribute.name(), minimum, current))
            }
        }
        AccessRequirement::SkillAtLeast { skill, minimum } => {
            if *minimum == 0 || *minimum > crate::data::MAX_SKILL_LEVEL {
                Some("This entrance has an invalid skill threshold.".into())
            } else {
                let current = skill.level(state);
                (current < *minimum).then(|| format!("{} must be at least level {} (currently {}).", skill.name(), minimum, current))
            }
        }
        AccessRequirement::CompanionDiscovered { companion_id } => {
            if crate::companion_discovery::is_discovered(state, *companion_id) {
                None
            } else { Some("A required companion lead has not yet been discovered.".into()) }
        }
        AccessRequirement::CompanionRecruited { companion_id } => {
            if state.companions.iter().any(|companion| companion.id == *companion_id) {
                None
            } else if crate::companion_discovery::is_discovered(state, *companion_id) {
                Some("A required companion has not yet been recruited.".into())
            } else { Some("A required companion lead has not yet been discovered.".into()) }
        }
        AccessRequirement::PartyIncludes { companion_id } => {
            if state.companions.iter().any(|companion| companion.id == *companion_id)
                && crate::companion_discovery::is_discovered(state, *companion_id)
                && state.party.contains(*companion_id)
            {
                None
            } else if crate::companion_discovery::is_discovered(state, *companion_id) {
                Some("A required companion must be recruited and in the active party.".into())
            } else { Some("A required companion lead has not yet been discovered.".into()) }
        }
        AccessRequirement::TimeWindow { start_hour, end_hour } => {
            if *start_hour >= 24 || *end_hour >= 24 || start_hour == end_hour {
                Some("This entrance has an invalid time window.".into())
            } else {
                let hour = crate::calendar::calendar_at(state.world_turn).hour;
                (!crate::calendar::is_hour_in_window(hour, *start_hour, *end_hour))
                    .then(|| format!("This place is only accessible between {start_hour:02}:00 and {end_hour:02}:00."))
            }
        }
        AccessRequirement::Season { season_index } => {
            if *season_index > 3 {
                Some("This entrance references an invalid season.".into())
            } else {
                let current = crate::calendar::calendar_at(state.world_turn).season_index;
                (current != *season_index).then(|| "This place is not accessible in the current season.".into())
            }
        }
    }
}

fn aggregated_costs(gates: &[Option<&AccessGate>]) -> Result<Vec<(ItemType, u32)>, String> {
    let mut totals: Vec<(ItemType, u32)> = Vec::new();
    for cost in gates.iter().flatten().flat_map(|gate| gate.consume_on_success.iter()) {
        if cost.count == 0 {
            return Err("An entrance has an invalid item cost.".into());
        }
        if let Some((_, total)) = totals.iter_mut().find(|(item, _)| *item == cost.item) {
            *total = total.checked_add(cost.count)
                .ok_or_else(|| "An entrance requires an invalid item quantity.".to_string())?;
        } else {
            totals.push((cost.item, cost.count));
        }
    }
    Ok(totals)
}

/// Evaluate multiple independent gates without modifying game state.
pub fn evaluate_access_gates(state: &GameState, gates: &[Option<&AccessGate>]) -> AccessResult {
    let mut reasons = Vec::new();
    for gate in gates.iter().flatten() {
        for requirement in &gate.all_of {
            if let Some(reason) = requirement_failure(state, requirement) {
                reasons.push(reason);
            }
        }
        if !gate.any_of.is_empty()
            && !gate.any_of.iter().any(|requirement| requirement_failure(state, requirement).is_none())
        {
            reasons.push("At least one alternative access requirement must be met.".into());
        }
    }

    match aggregated_costs(gates) {
        Err(reason) => reasons.push(reason),
        Ok(costs) => {
            for (item, required) in costs {
                let available = state.player.get_item_count(item);
                if available < required {
                    reasons.push(format!("You need {} more {} to enter.", required - available, item.name()));
                }
            }
        }
    }

    if reasons.is_empty() { AccessResult::allowed() } else { AccessResult::denied(reasons) }
}

pub fn evaluate_access_gate(state: &GameState, gate: &AccessGate) -> AccessResult {
    evaluate_access_gates(state, &[Some(gate)])
}

/// Apply all entry costs together. The caller invokes this on a staged state
/// only after the domain entry operation succeeds.
pub fn consume_entry_costs_atomically(
    state: &mut GameState,
    gates: &[Option<&AccessGate>],
) -> Result<(), String> {
    let costs = aggregated_costs(gates)?;
    let mut items = state.player.items.clone();
    for (item, required) in costs {
        let available = items.get(&item).copied().unwrap_or(0);
        if available < required {
            return Err(format!("You need {} more {} to enter.", required - available, item.name()));
        }
        items.insert(item, available - required);
    }
    state.player.items = items;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::GameEvent;
    use crate::party::CompanionState;
    use crate::quests::{Quest, QuestObjective, QuestReward, QuestStage, QuestStageCompleted};
    use crate::state::Player;
    use crate::world::LocationId;

    fn state() -> GameState {
        GameState::new(Player::new("access-test".into(), "hash".into()))
    }

    fn quest_with_stage() -> Quest {
        Quest {
            id: "known_quest".into(),
            title: "Known quest".into(),
            description: "A test quest.".into(),
            objective: QuestObjective::MonsterKills { required: 1 },
            progress: 0,
            target: 1,
            is_completed: false,
            is_claimed: false,
            reward: QuestReward::Gold(0),
            is_daily: false,
            stages: vec![QuestStage {
                id: "known_stage".into(),
                title: "Known stage".into(),
                objectives: Vec::new(),
                is_completed: false,
                access_gate: None,
            }],
        }
    }

    #[test]
    fn unknown_references_fail_closed() {
        let state = state();
        for requirement in [
            AccessRequirement::RealmDiscovered { realm_id: "unknown".into() },
            AccessRequirement::RealmCompleted { realm_id: "unknown".into() },
            AccessRequirement::QuestCompleted { quest_id: "unknown".into() },
            AccessRequirement::QuestStageCompleted { quest_id: "known_quest".into(), stage_id: "unknown".into() },
            AccessRequirement::LandmarkInteracted { target_id: "unknown".into() },
        ] {
            let result = evaluate_access_gate(&state, &AccessGate {
                all_of: vec![requirement],
                ..AccessGate::default()
            });
            assert!(!result.allowed);
        }
    }

    #[test]
    fn quest_and_stage_requirements_use_persisted_progress() {
        let mut state = state();
        state.active_quests.push(quest_with_stage());
        let quest_gate = AccessGate {
            all_of: vec![AccessRequirement::QuestCompleted { quest_id: "known_quest".into() }],
            ..AccessGate::default()
        };
        assert!(!evaluate_access_gate(&state, &quest_gate).allowed);
        state.active_quests[0].is_completed = true;
        assert!(evaluate_access_gate(&state, &quest_gate).allowed);

        let stage_gate = AccessGate {
            all_of: vec![AccessRequirement::QuestStageCompleted {
                quest_id: "known_quest".into(),
                stage_id: "known_stage".into(),
            }],
            ..AccessGate::default()
        };
        assert!(!evaluate_access_gate(&state, &stage_gate).allowed);
        state.active_quests[0].stages[0].is_completed = true;
        assert!(evaluate_access_gate(&state, &stage_gate).allowed);
    }

    #[test]
    fn item_attribute_skill_landmark_and_season_requirements_work() {
        let mut state = state();
        state.player.add_item(ItemType::Food, 2);
        state.player.attributes.charisma = 14;
        state.player.skills.mining.level = 5;
        state.region_progress.landmark_interactions.push("standing_stones_survey".into());
        let season = crate::calendar::calendar_at(state.world_turn).season_index;
        let gate = AccessGate {
            all_of: vec![
                AccessRequirement::ItemCount { item: ItemType::Food, minimum: 2 },
                AccessRequirement::AttributeAtLeast { attribute: AttributeKind::Charisma, minimum: 14 },
                AccessRequirement::SkillAtLeast { skill: SkillKind::Mining, minimum: 5 },
                AccessRequirement::LandmarkInteracted { target_id: "standing_stones_survey".into() },
                AccessRequirement::Season { season_index: season },
                AccessRequirement::TimeWindow { start_hour: 5, end_hour: 8 },
            ],
            ..AccessGate::default()
        };
        assert!(evaluate_access_gate(&state, &gate).allowed);
    }

    #[test]
    fn companion_requirements_separate_discovery_recruitment_and_party_membership() {
        let mut state = state();
        let id = CompanionId::LyraSwiftbow;
        let discovered = AccessGate {
            all_of: vec![AccessRequirement::CompanionDiscovered { companion_id: id }],
            ..AccessGate::default()
        };
        let recruited = AccessGate {
            all_of: vec![AccessRequirement::CompanionRecruited { companion_id: id }],
            ..AccessGate::default()
        };
        let in_party = AccessGate {
            all_of: vec![AccessRequirement::PartyIncludes { companion_id: id }],
            ..AccessGate::default()
        };
        assert!(!evaluate_access_gate(&state, &discovered).allowed);
        state.discovered_companions.push(id);
        assert!(evaluate_access_gate(&state, &discovered).allowed);
        assert!(!evaluate_access_gate(&state, &recruited).allowed);
        state.companions.push(CompanionState::new_from_definition(&crate::party::get_definition(id)));
        assert!(evaluate_access_gate(&state, &recruited).allowed);
        assert!(!evaluate_access_gate(&state, &in_party).allowed);
        state.party.add_member(id, &state.companions).unwrap();
        assert!(evaluate_access_gate(&state, &in_party).allowed);
    }

    #[test]
    fn all_of_and_any_of_are_both_enforced_and_unknown_seasons_fail_closed() {
        let mut state = state();
        state.player.attributes.charisma = 14;
        let gate = AccessGate {
            all_of: vec![AccessRequirement::AttributeAtLeast {
                attribute: AttributeKind::Charisma, minimum: 12,
            }],
            any_of: vec![
                AccessRequirement::Season { season_index: 0 },
                AccessRequirement::SkillAtLeast { skill: SkillKind::Mining, minimum: 5 },
            ],
            consume_on_success: Vec::new(),
        };
        assert!(evaluate_access_gate(&state, &gate).allowed);
        state.world_turn = crate::calendar::HOURS_PER_YEAR;
        assert!(!evaluate_access_gate(&state, &gate).allowed);
        let invalid = AccessGate {
            all_of: vec![AccessRequirement::Season { season_index: 4 }],
            ..AccessGate::default()
        };
        assert!(!evaluate_access_gate(&state, &invalid).allowed);
    }

    #[test]
    fn entry_costs_are_side_effect_free_until_atomic_commit() {
        let mut state = state();
        state.player.add_item(ItemType::Food, 2);
        let first = AccessGate {
            consume_on_success: vec![ItemCost { item: ItemType::Food, count: 1 }],
            ..AccessGate::default()
        };
        let second = AccessGate {
            consume_on_success: vec![ItemCost { item: ItemType::Food, count: 1 }],
            ..AccessGate::default()
        };
        let gates = [Some(&first), Some(&second)];
        assert!(evaluate_access_gates(&state, &gates).allowed);
        assert_eq!(state.player.get_item_count(ItemType::Food), 2);
        consume_entry_costs_atomically(&mut state, &gates).unwrap();
        assert_eq!(state.player.get_item_count(ItemType::Food), 0);

        state.player.add_item(ItemType::Food, 1);
        assert!(consume_entry_costs_atomically(&mut state, &gates).is_err());
        assert_eq!(state.player.get_item_count(ItemType::Food), 1);
    }

    #[test]
    fn discovery_event_is_available_to_access_checks() {
        let mut state = state();
        crate::quests::notify_event(&mut state, &GameEvent::LocationReached {
            location_id: LocationId::from("village_start"),
        });
        // Realm discovery is recorded through the existing Section 20 event path.
        assert!(crate::realm_progression::realm_is_discovered(&state, "oakhaven"));
    }
}
