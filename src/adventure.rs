//! Reusable region content and deterministic expedition-domain transitions.
//!
//! Static content is data-driven; route, encounter, quest, and expedition rules
//! stay independent from terminal presentation so save/reload regressions can
//! exercise the same state transitions as the UI.

use serde::{Deserialize, Serialize};

use crate::data::{ItemType, NpcType};
use crate::event::GameEvent;
use crate::quests::{
    make_composite_quest, Quest, QuestObjective, QuestObjectiveState, QuestReward, QuestStage,
};
use crate::state::GameState;
use crate::world::{LocationId, World};

pub const REGION_QUEST_ID: &str = "oakhaven_missing_supplies";
pub const REGION_QUEST_REWARD_GOLD: u32 = 450;
pub const REGION_QUEST_REWARD_XP: u32 = 150;
pub const REGION_QUEST_REWARD_LOGS: u32 = 2;

/// Save-backed idempotency keys for the region. IDs are stable content IDs,
/// not display names, so translations/renames cannot reset paid rewards.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RegionProgress {
    pub cleared_encounters: Vec<String>,
    pub landmark_interactions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterBand {
    SafeRoad,
    Woods,
    Ruins,
    Caves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyDefinition {
    pub name: &'static str,
    pub kind: NpcType,
    pub level_offset: u32,
    pub health_multiplier_pct: u32,
    pub attack_multiplier_pct: u32,
    pub gold_reward: u32,
    pub item_reward: Option<(ItemType, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterDefinition {
    pub id: &'static str,
    pub location_id: &'static str,
    pub title: &'static str,
    pub danger: u32,
    pub enemies: &'static [EnemyDefinition],
    pub reward_gold: u32,
    pub reward_xp: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandmarkDefinition {
    pub location_id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub interaction_target: &'static str,
}

const GOBLIN: EnemyDefinition = EnemyDefinition {
    name: "Goblin",
    kind: NpcType::Goblin,
    level_offset: 1,
    health_multiplier_pct: 80,
    attack_multiplier_pct: 55,
    gold_reward: 18,
    item_reward: Some((ItemType::GoblinMail, 1)),
};
const WOLF: EnemyDefinition = EnemyDefinition {
    name: "Wolf",
    kind: NpcType::Bear,
    level_offset: 2,
    health_multiplier_pct: 95,
    attack_multiplier_pct: 65,
    gold_reward: 24,
    item_reward: Some((ItemType::Fur, 1)),
};
const BANDIT: EnemyDefinition = EnemyDefinition {
    name: "Cave Bandit",
    kind: NpcType::Thief,
    level_offset: 4,
    health_multiplier_pct: 115,
    attack_multiplier_pct: 75,
    gold_reward: 35,
    item_reward: Some((ItemType::Ore, 1)),
};
const CAVE_GOBLIN: EnemyDefinition = EnemyDefinition {
    name: "Cave Goblin",
    kind: NpcType::Goblin,
    level_offset: 3,
    health_multiplier_pct: 105,
    attack_multiplier_pct: 70,
    gold_reward: 26,
    item_reward: Some((ItemType::GoblinMail, 1)),
};
const STONE_GUARDIAN: EnemyDefinition = EnemyDefinition {
    name: "Stone Guardian",
    kind: NpcType::Rockbeast,
    level_offset: 5,
    health_multiplier_pct: 140,
    attack_multiplier_pct: 85,
    gold_reward: 60,
    item_reward: Some((ItemType::Gemstones, 1)),
};

const WOODS_ENEMIES: &[EnemyDefinition] = &[WOLF, GOBLIN];
const RUINS_ENEMIES: &[EnemyDefinition] = &[STONE_GUARDIAN, BANDIT];
const CAVE_ENEMIES: &[EnemyDefinition] = &[CAVE_GOBLIN, BANDIT, STONE_GUARDIAN];

pub const ENCOUNTERS: &[EncounterDefinition] = &[
    EncounterDefinition {
        id: "woods_patrol",
        location_id: "whispering_woods",
        title: "The Whispering Woods",
        danger: 3,
        enemies: WOODS_ENEMIES,
        reward_gold: 35,
        reward_xp: 60,
    },
    EncounterDefinition {
        id: "ruins_guardians",
        location_id: "sunken_ruins",
        title: "Sunken Ruins",
        danger: 4,
        enemies: RUINS_ENEMIES,
        reward_gold: 75,
        reward_xp: 110,
    },
    EncounterDefinition {
        id: "cave_expedition",
        location_id: "goblin_caves",
        title: "Goblin Caves",
        danger: 5,
        enemies: CAVE_ENEMIES,
        reward_gold: 120,
        reward_xp: 180,
    },
];

pub const LANDMARKS: &[LandmarkDefinition] = &[
    LandmarkDefinition {
        location_id: "crumbling_watchtower",
        name: "Crumbling Watchtower",
        description: "A scout's vantage. You spot movement along the tree line.",
        interaction_target: "watchtower_scout_report",
    },
    LandmarkDefinition {
        location_id: "standing_stones",
        name: "Standing Stones",
        description: "Old route markers; travellers say the stones point towards the ruins.",
        interaction_target: "standing_stones_survey",
    },
    LandmarkDefinition {
        location_id: "abandoned_mill",
        name: "Abandoned Mill",
        description: "The miller's stores are abandoned, but usable timber remains nearby.",
        interaction_target: "abandoned_mill_cache",
    },
];

pub fn encounter_for_location(location_id: &LocationId) -> Option<&'static EncounterDefinition> {
    ENCOUNTERS
        .iter()
        .find(|encounter| encounter.location_id == location_id.as_str())
}

pub fn enemy_for_name(name: &str) -> Option<&'static EnemyDefinition> {
    ENCOUNTERS
        .iter()
        .flat_map(|encounter| encounter.enemies)
        .find(|enemy| enemy.name == name)
}

pub fn landmark_for_location(location_id: &LocationId) -> Option<&'static LandmarkDefinition> {
    LANDMARKS
        .iter()
        .find(|landmark| landmark.location_id == location_id.as_str())
}

/// The intro quest is installed once, never reinstalled after completion/claim,
/// and uses the existing composite-quest engine for staged travel, combat, and
/// gathering objectives.
pub fn ensure_region_quest(state: &mut GameState) -> bool {
    if state
        .active_quests
        .iter()
        .any(|quest| quest.id == REGION_QUEST_ID)
        || state
            .player
            .quests_completed
            .iter()
            .any(|id| id == REGION_QUEST_ID)
    {
        return false;
    }
    state.active_quests.push(region_quest_definition());
    true
}

pub fn region_quest_definition() -> Quest {
    make_composite_quest(
        REGION_QUEST_ID,
        "Oakhaven's Missing Supplies",
        "Survey the old road, clear threats near the ruins, and recover timber for the village.",
        vec![
            QuestStage {
                id: "find_the_route".into(),
                title: "Find the Old Road".into(),
                is_completed: false,
                objectives: vec![
                    objective(QuestObjective::ReachLocation {
                        location_id: LocationId::from("standing_stones"),
                    }),
                    objective(QuestObjective::InteractWith {
                        target_id: "standing_stones_survey".into(),
                        required_count: 1,
                    }),
                ],
            },
            QuestStage {
                id: "clear_the_ruins".into(),
                title: "Clear the Ruins".into(),
                is_completed: false,
                objectives: vec![
                    objective(QuestObjective::ReachLocation {
                        location_id: LocationId::from("sunken_ruins"),
                    }),
                    objective(QuestObjective::SpecificMonsterKills {
                        monster_name: "Stone Guardian".into(),
                        required: 1,
                    }),
                ],
            },
            QuestStage {
                id: "bring_supplies_home".into(),
                title: "Recover Village Supplies".into(),
                is_completed: false,
                objectives: vec![
                    objective(QuestObjective::GatherResource {
                        item_type: ItemType::Logs,
                        required_count: 3,
                    }),
                    objective(QuestObjective::ReachLocation {
                        location_id: LocationId::from("village_start"),
                    }),
                    objective(QuestObjective::InteractWith {
                        target_id: "village_supply_board".into(),
                        required_count: 1,
                    }),
                ],
            },
        ],
        QuestReward::Gold(REGION_QUEST_REWARD_GOLD),
        false,
    )
}

fn objective(objective: QuestObjective) -> QuestObjectiveState {
    QuestObjectiveState {
        objective,
        progress: 0,
        is_completed: false,
        optional: false,
        bonus_reward: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpeditionError {
    UnknownEncounter,
    WrongLocation,
    AlreadyClaimed,
    QuestNotReady,
    NotAtSettlement,
}

impl std::fmt::Display for ExpeditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownEncounter => f.write_str("No encounter is defined for this destination."),
            Self::WrongLocation => f.write_str("The party is not at this encounter location."),
            Self::AlreadyClaimed => f.write_str("This region reward has already been claimed."),
            Self::QuestNotReady => f.write_str("The region quest is not yet complete."),
            Self::NotAtSettlement => {
                f.write_str("Return to Oakhaven Village to claim the supplies.")
            }
        }
    }
}

impl std::error::Error for ExpeditionError {}

/// Shared, deterministic expedition-level reward entry point. Individual enemy
/// rewards use the same inventory and quest event systems; the encounter-clear
/// reward is gated by a persisted ID and location and cannot be claimed twice.
pub fn claim_encounter_clear_reward(
    state: &mut GameState,
    encounter_id: &str,
) -> Result<(u32, u32), ExpeditionError> {
    let encounter = ENCOUNTERS
        .iter()
        .find(|entry| entry.id == encounter_id)
        .ok_or(ExpeditionError::UnknownEncounter)?;
    if state.current_location.as_str() != encounter.location_id {
        return Err(ExpeditionError::WrongLocation);
    }

    if state
        .region_progress
        .cleared_encounters
        .iter()
        .any(|id| id == encounter_id)
    {
        return Err(ExpeditionError::AlreadyClaimed);
    }

    state.player.gold = state
        .player
        .gold
        .saturating_add(encounter.reward_gold)
        .min(crate::data::MAX_GOLD);
    state.player.xp = state
        .player
        .xp
        .saturating_add(encounter.reward_xp)
        .min(1_000_000);
    state
        .region_progress
        .cleared_encounters
        .push(encounter_id.to_string());
    Ok((encounter.reward_gold, encounter.reward_xp))
}

/// Claim the adventure quest only after every staged objective is complete and
/// from the village that commissioned it; both reward and completion marker are
/// committed by the existing quest claim path.
pub fn claim_region_quest(state: &mut GameState) -> Result<QuestReward, ExpeditionError> {
    if state.current_location.as_str() != "village_start" {
        return Err(ExpeditionError::NotAtSettlement);
    }
    if state
        .player
        .quests_completed
        .iter()
        .any(|id| id == REGION_QUEST_ID)
    {
        return Err(ExpeditionError::AlreadyClaimed);
    }
    let quest = state
        .active_quests
        .iter()
        .find(|quest| quest.id == REGION_QUEST_ID)
        .ok_or(ExpeditionError::QuestNotReady)?;
    if !quest.is_completed {
        return Err(ExpeditionError::QuestNotReady);
    }
    let reward = crate::quests::claim_quest_reward(state, REGION_QUEST_ID)
        .map_err(|_| ExpeditionError::QuestNotReady)?;
    state
        .player
        .add_item(ItemType::Logs, REGION_QUEST_REWARD_LOGS);
    state.player.xp = state
        .player
        .xp
        .saturating_add(REGION_QUEST_REWARD_XP)
        .min(1_000_000);
    Ok(reward)
}

pub fn notify_landmark_interaction(state: &mut GameState, target_id: &str) {
    if !state
        .region_progress
        .landmark_interactions
        .iter()
        .any(|known| known == target_id)
    {
        state
            .region_progress
            .landmark_interactions
            .push(target_id.to_string());
    }
    crate::quests::notify_event(
        state,
        &GameEvent::TargetInteracted {
            target_id: target_id.to_string(),
        },
    );
}

pub fn world_location_valid(world: &World, id: &LocationId) -> bool {
    world.get_location(id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::{CompanionId, PartyManager};
    use crate::state::Player;
    use crate::world::build_initial_world;

    fn state() -> GameState {
        let mut state = GameState::new(Player::new("region-test".into(), "hash".into()));
        state.active_quests.clear();
        state
    }

    #[test]
    fn region_has_multiple_encounters_and_landmarks_reuse_world_locations() {
        let world = build_initial_world().unwrap();
        assert!(world.validate().is_ok());
        assert!(world.can_travel(
            &LocationId::from("village_start"),
            &LocationId::from("whispering_woods")
        ));
        assert_eq!(ENCOUNTERS.len(), 3);
        assert!(ENCOUNTERS
            .iter()
            .any(|e| e.danger == 5 && e.location_id == "goblin_caves"));
        assert!(LANDMARKS.iter().all(|landmark| world
            .get_location(&LocationId::from(landmark.location_id))
            .is_some()));
        assert!(enemy_for_name("Stone Guardian").is_some());
        assert!(enemy_for_name("Cave Bandit").is_some());
    }

    #[test]
    fn quest_stages_travel_combat_and_gathering_and_claim_is_idempotent() {
        let mut state = state();
        ensure_region_quest(&mut state);
        let before = state.player.gold;
        crate::quests::notify_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("standing_stones"),
            },
        );
        notify_landmark_interaction(&mut state, "standing_stones_survey");
        crate::quests::notify_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("sunken_ruins"),
            },
        );
        crate::quests::notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Stone Guardian".into(),
                is_stronghold: false,
            },
        );
        crate::quests::notify_event(
            &mut state,
            &GameEvent::ResourceGathered {
                skill_name: "Woodcutting".into(),
                item_type: ItemType::Logs,
                count: 3,
            },
        );
        state.current_location = LocationId::from("village_start");
        crate::quests::notify_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("village_start"),
            },
        );
        notify_landmark_interaction(&mut state, "village_supply_board");
        assert!(state.active_quests.iter().find(|q| q.id == REGION_QUEST_ID).unwrap().is_completed);
        assert_eq!(
            claim_region_quest(&mut state),
            Ok(QuestReward::Gold(REGION_QUEST_REWARD_GOLD))
        );
        assert_eq!(state.player.gold, before + REGION_QUEST_REWARD_GOLD);
        let gold = state.player.gold;
        assert_eq!(
            claim_region_quest(&mut state),
            Err(ExpeditionError::AlreadyClaimed)
        );
        assert_eq!(state.player.gold, gold);
    }

    #[test]
    fn encounter_clear_reward_rejects_wrong_location_and_duplicates() {
        let mut state = state();
        state.current_location = LocationId::from("whispering_woods");
        assert_eq!(
            claim_encounter_clear_reward(&mut state, "cave_expedition"),
            Err(ExpeditionError::WrongLocation)
        );
        let before = state.player.gold;
        assert_eq!(
            claim_encounter_clear_reward(&mut state, "woods_patrol"),
            Ok((35, 60))
        );
        assert_eq!(state.player.gold, before + 35);
        let encoded = serde_json::to_string(&state).unwrap();
        let loaded: GameState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            loaded.region_progress.cleared_encounters,
            vec!["woods_patrol"]
        );
        assert_eq!(
            claim_encounter_clear_reward(&mut state, "woods_patrol"),
            Err(ExpeditionError::AlreadyClaimed)
        );
        assert_eq!(state.player.gold, before + 35);
    }

    #[test]
    fn knockout_recovery_is_persistent_and_not_recreated_on_reload() {
        let mut state = state();
        let world = build_initial_world().unwrap();
        let id = CompanionId::BranTheShield;
        let cost = PartyManager::recruit(
            &mut state.companions,
            id,
            state.player.combat_level,
            state.player.gold,
        )
        .unwrap();
        state.player.gold -= cost;
        PartyManager::add_to_party(&mut state.party, &state.companions, id).unwrap();
        state
            .companions
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap()
            .current_hp = 0;
        let mut recovery = std::mem::take(&mut state.companion_recovery);
        recovery
            .record_knockout_by_id(&mut state, id, &world)
            .unwrap();
        state.companion_recovery = recovery;
        assert_eq!(state.companion_recovery.records.len(), 1);
        let json = serde_json::to_string(&state).unwrap();
        let mut loaded: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.companion_recovery.records.len(), 1);
        loaded.advance_world_turn();
        loaded.advance_world_turn();
        loaded.advance_world_turn();
        assert!(loaded.companion_recovery.records.is_empty());
        assert!(
            loaded
                .companions
                .iter()
                .find(|c| c.id == id)
                .unwrap()
                .current_hp
                > 0
        );
    }
}
