use crate::data::{ItemType, MAX_GOLD, MAX_PLAYER_LEVEL, MAX_SKILL_LEVEL};
use crate::event::{GameEvent, WorldEvent};
use crate::state::GameState;
use crate::world::LocationId;
use crate::ui::Tui;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestError {
    NotFound,
    NotCompleted,
    AlreadyClaimed,
    AlreadyCompleted,
    InsufficientItems,
    InvalidOperation,
}

impl fmt::Display for QuestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Quest not found."),
            Self::NotCompleted => write!(f, "Quest objectives have not been met."),
            Self::AlreadyClaimed => write!(f, "The reward has already been claimed."),
            Self::AlreadyCompleted => write!(f, "This quest is already complete."),
            Self::InsufficientItems => write!(f, "You do not have enough items for that hand-in."),
            Self::InvalidOperation => write!(f, "That operation is not valid for this quest."),
        }
    }
}

impl std::error::Error for QuestError {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum QuestObjective {
    MonsterKills {
        required: u32,
    },
    PlayerLevel {
        required_level: u32,
    },
    SkillLevel {
        skill_name: String,
        required_level: u32,
    },
    ItemHandIn {
        item_type: ItemType,
        required_count: u32,
    },
    SpecificMonsterKills {
        monster_name: String,
        required: u32,
    },
    CollectItem {
        item_type: ItemType,
        required_count: u32,
    },
    GatherResource {
        item_type: ItemType,
        required_count: u32,
    },
    CraftItem {
        recipe_id: String,
        required_count: u32,
    },
    ReachLocation {
        location_id: LocationId,
    },
    TraverseLocation {
        from_location_id: Option<LocationId>,
        to_location_id: LocationId,
    },
    InteractWith {
        target_id: String,
        required_count: u32,
    },
    CompleteQuest {
        quest_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum QuestReward {
    Gold(u32),
    Xp(u32),
    Item { item_type: ItemType, count: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct QuestObjectiveState {
    pub objective: QuestObjective,
    #[serde(default)]
    pub progress: u32,
    #[serde(default)]
    pub is_completed: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub bonus_reward: Option<QuestReward>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct QuestStage {
    pub id: String,
    pub title: String,
    pub objectives: Vec<QuestObjectiveState>,
    #[serde(default)]
    pub is_completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective: QuestObjective,
    pub progress: u32,
    pub target: u32,
    pub is_completed: bool,
    pub is_claimed: bool,
    pub reward: QuestReward,
    pub is_daily: bool,
    /// Missing in legacy saves; an empty list preserves their original objective.
    #[serde(default)]
    pub stages: Vec<QuestStage>,
}

/// Create a stable set of daily objectives. IDs include the UTC day number so
/// historical completions never suppress the same type of quest on a later day.
pub fn default_daily_quests(state: &GameState, day_index: u64) -> Vec<Quest> {
    let mut rng = rand::thread_rng();
    let kill_target = 5 + (day_index % 3) as u32 * 5;
    let current_combat = state.player.combat_level;
    let combat_target = current_combat
        .saturating_add(1 + (day_index % 4) as u32)
        .min(MAX_PLAYER_LEVEL);

    let skills = [
        ("Woodcutting", state.player.skills.woodcutting.level),
        ("Cooking", state.player.skills.cooking.level),
        ("Fishing", state.player.skills.fishing.level),
        ("Mining", state.player.skills.mining.level),
        ("Smithing", state.player.skills.smithing.level),
        ("Thieving", state.player.skills.thieving.level),
    ];
    let (skill_name, current_skill_level) = skills
        .iter()
        .min_by_key(|(_, level)| *level)
        .copied()
        .unwrap_or(("Woodcutting", 1));
    let skill_target = current_skill_level
        .saturating_add(1 + rng.gen_range(0..=2))
        .min(MAX_SKILL_LEVEL);

    vec![
        Quest {
            id: format!("daily_monsters_{day_index}"),
            title: "Monster Hunter".into(),
            description: format!("Defeat {kill_target} monsters across Gielinor."),
            objective: QuestObjective::MonsterKills {
                required: kill_target,
            },
            progress: 0,
            target: kill_target,
            is_completed: false,
            is_claimed: false,
            reward: QuestReward::Gold(250),
            is_daily: true,
            stages: Vec::new(),
        },
        Quest {
            id: format!("daily_combat_level_{day_index}"),
            title: "Rising Adventurer".into(),
            description: format!("Reach combat level {combat_target}."),
            objective: QuestObjective::PlayerLevel {
                required_level: combat_target,
            },
            progress: current_combat.min(combat_target),
            target: combat_target,
            is_completed: current_combat >= combat_target,
            is_claimed: false,
            reward: QuestReward::Xp(750),
            is_daily: true,
            stages: Vec::new(),
        },
        Quest {
            id: format!("daily_skill_{day_index}"),
            title: format!("{skill_name} Practice"),
            description: format!("Reach level {skill_target} in {skill_name}."),
            objective: QuestObjective::SkillLevel {
                skill_name: skill_name.into(),
                required_level: skill_target,
            },
            progress: current_skill_level.min(skill_target),
            target: skill_target,
            is_completed: current_skill_level >= skill_target,
            is_claimed: false,
            reward: QuestReward::Gold(300),
            is_daily: true,
            stages: Vec::new(),
        },
        Quest {
            id: format!("daily_handin_ore_{day_index}"),
            title: "Supply the Smelter".into(),
            description: "Hand in 3 Ore at the Quest Hall.".into(),
            objective: QuestObjective::ItemHandIn {
                item_type: ItemType::Ore,
                required_count: 3,
            },
            progress: 0,
            target: 3,
            is_completed: false,
            is_claimed: false,
            reward: QuestReward::Item {
                item_type: ItemType::Ingots,
                count: 2,
            },
            is_daily: true,
            stages: Vec::new(),
        },
    ]
}

/// Refresh dailies when 24 hours have elapsed. Existing unclaimed quests and
/// the player's completion history are retained; claimed daily entries are
/// removed before the new set is added.
pub fn check_and_refresh_quests(state: &mut GameState) {
    refresh_quests_at(state, Utc::now());
}

fn refresh_quests_at(state: &mut GameState, now: DateTime<Utc>) {
    let should_refresh = match state.player.last_quest_refresh {
        Some(last) => now.signed_duration_since(last) >= Duration::days(1),
        None => true,
    };

    // A pre-Section-3 save has the old refresh timestamp but no persisted quest
    // collection. Initialise it once instead of leaving that save questless.
    if !should_refresh && !state.active_quests.is_empty() {
        return;
    }

    let timestamp = now.timestamp().max(0) as u64;
    let day_index = timestamp / 86_400;
    state
        .active_quests
        .retain(|quest| !quest.is_daily || !quest.is_claimed);

    for quest in default_daily_quests(state, day_index) {
        if !state
            .active_quests
            .iter()
            .any(|active| active.id == quest.id)
            && !state.player.quests_completed.contains(&quest.id)
        {
            state.active_quests.push(quest);
        }
    }

    state.player.last_quest_refresh = Some(now);
    state.active_world_event = WorldEvent::generate_deterministic(day_index, timestamp);
}

/// Progress legacy and composite quests from typed gameplay events. Each objective
/// state is visited once per event, and only the first unfinished stage receives it.
pub fn notify_event(state: &mut GameState, event: &GameEvent) {
    if matches!(event, GameEvent::MonsterDefeated { .. }) {
        state.player.lifetime_kills = state.player.lifetime_kills.saturating_add(1);
    }

    let mut pending = vec![event.clone()];
    while let Some(current_event) = pending.pop() {
        let mut newly_completed = Vec::new();
        let mut bonus_rewards = Vec::new();

        for quest in &mut state.active_quests {
            if quest.is_completed || quest.is_claimed {
                continue;
            }
            if quest.stages.is_empty() {
                let was_completed = quest.is_completed;
                progress_legacy_quest(quest, &current_event);
                if !was_completed && quest.is_completed {
                    newly_completed.push(quest.id.clone());
                }
                continue;
            }

            let Some(stage_index) = quest.stages.iter().position(|stage| !stage.is_completed)
            else {
                if !quest.is_completed {
                    quest.is_completed = true;
                    newly_completed.push(quest.id.clone());
                }
                continue;
            };
            let stage = &mut quest.stages[stage_index];

            for objective_state in &mut stage.objectives {
                if objective_state.is_completed {
                    continue;
                }
                if progress_objective(
                    &objective_state.objective,
                    &current_event,
                    &mut objective_state.progress,
                ) {
                    objective_state.is_completed = true;
                    if objective_state.optional {
                        if let Some(reward) = objective_state.bonus_reward.clone() {
                            bonus_rewards.push(reward);
                        }
                    }
                }
            }

            if stage
                .objectives
                .iter()
                .all(|objective| objective.optional || objective.is_completed)
            {
                stage.is_completed = true;
            }
            let mandatory_total = stage
                .objectives
                .iter()
                .filter(|objective| !objective.optional)
                .count();
            let mandatory_done = stage
                .objectives
                .iter()
                .filter(|objective| !objective.optional && objective.is_completed)
                .count();
            quest.progress = u32::try_from(mandatory_done).unwrap_or(u32::MAX);
            quest.target = u32::try_from(mandatory_total).unwrap_or(u32::MAX);
            let was_completed = quest.is_completed;
            quest.is_completed = quest.stages.iter().all(|candidate| candidate.is_completed);
            if !was_completed && quest.is_completed {
                newly_completed.push(quest.id.clone());
            }
        }

        for reward in bonus_rewards {
            grant_reward(state, &reward);
        }
        for quest_id in newly_completed {
            pending.push(GameEvent::PrerequisiteQuestCompleted { quest_id });
        }
    }
}

fn progress_legacy_quest(quest: &mut Quest, event: &GameEvent) {
    match (&quest.objective, event) {
        (QuestObjective::MonsterKills { required }, GameEvent::MonsterDefeated { .. }) => {
            quest.progress = quest.progress.saturating_add(1).min(*required);
            quest.is_completed = quest.progress >= *required;
        }
        (
            QuestObjective::SpecificMonsterKills {
                monster_name,
                required,
            },
            GameEvent::MonsterDefeated { name, .. },
        ) if canonical_monster_name(name).eq_ignore_ascii_case(monster_name) => {
            quest.progress = quest.progress.saturating_add(1).min(*required);
            quest.is_completed = quest.progress >= *required;
        }
        (
            QuestObjective::PlayerLevel { required_level },
            GameEvent::PlayerLeveledUp { new_level },
        ) => {
            quest.progress = (*new_level).min(*required_level);
            quest.is_completed = *new_level >= *required_level;
        }
        (
            QuestObjective::SkillLevel {
                skill_name,
                required_level,
            },
            GameEvent::SkillLeveledUp {
                skill_name: event_skill,
                new_level,
            },
        ) if skill_name.eq_ignore_ascii_case(event_skill) => {
            quest.progress = (*new_level).min(*required_level);
            quest.is_completed = *new_level >= *required_level;
        }
        (
            QuestObjective::ItemHandIn {
                item_type,
                required_count,
            },
            GameEvent::ItemHandedIn {
                item_type: event_item,
                count,
            },
        ) if item_type == event_item => {
            quest.progress = quest.progress.saturating_add(*count).min(*required_count);
            quest.is_completed = quest.progress >= *required_count;
        }
        _ => {}
    }
}

fn canonical_monster_name(name: &str) -> &str {
    name.strip_prefix("Elite ").unwrap_or(name).trim()
}

fn objective_target(objective: &QuestObjective) -> u32 {
    match objective {
        QuestObjective::MonsterKills { required }
        | QuestObjective::SpecificMonsterKills { required, .. } => *required,
        QuestObjective::PlayerLevel { required_level }
        | QuestObjective::SkillLevel { required_level, .. } => *required_level,
        QuestObjective::ItemHandIn { required_count, .. }
        | QuestObjective::CollectItem { required_count, .. }
        | QuestObjective::GatherResource { required_count, .. }
        | QuestObjective::CraftItem { required_count, .. }
        | QuestObjective::InteractWith { required_count, .. } => *required_count,
        QuestObjective::ReachLocation { .. }
        | QuestObjective::TraverseLocation { .. }
        | QuestObjective::CompleteQuest { .. } => 1,
    }
}

fn progress_objective(objective: &QuestObjective, event: &GameEvent, progress: &mut u32) -> bool {
    let target = objective_target(objective);
    let increment = match (objective, event) {
        (QuestObjective::MonsterKills { .. }, GameEvent::MonsterDefeated { .. }) => 1,
        (
            QuestObjective::SpecificMonsterKills { monster_name, .. },
            GameEvent::MonsterDefeated { name, .. },
        ) if canonical_monster_name(name).eq_ignore_ascii_case(monster_name) => target,
        (
            QuestObjective::PlayerLevel { required_level },
            GameEvent::PlayerLeveledUp { new_level },
        ) => {
            *progress = (*new_level).min(*required_level);
            return *new_level >= *required_level;
        }
        (
            QuestObjective::SkillLevel {
                skill_name,
                required_level,
            },
            GameEvent::SkillLeveledUp {
                skill_name: event_skill,
                new_level,
            },
        ) if skill_name.eq_ignore_ascii_case(event_skill) => {
            *progress = (*new_level).min(*required_level);
            return *new_level >= *required_level;
        }
        (
            QuestObjective::ItemHandIn { item_type, .. },
            GameEvent::ItemHandedIn {
                item_type: event_item,
                count,
            },
        ) if item_type == event_item => *count,
        (
            QuestObjective::CollectItem { item_type, .. },
            GameEvent::ItemCollected {
                item_type: event_item,
                count,
            },
        ) if item_type == event_item => *count,
        (
            QuestObjective::CollectItem { item_type, .. },
            GameEvent::ResourceGathered {
                item_type: event_item,
                count,
                ..
            },
        ) if item_type == event_item => *count,
        (
            QuestObjective::CollectItem { item_type, .. },
            GameEvent::ItemCrafted {
                item_type: Some(event_item),
                count,
                ..
            },
        ) if item_type == event_item => *count,
        (
            QuestObjective::GatherResource { item_type, .. },
            GameEvent::ResourceGathered {
                item_type: event_item,
                count,
                ..
            },
        ) if item_type == event_item => *count,
        (
            QuestObjective::CraftItem { recipe_id, .. },
            GameEvent::ItemCrafted {
                recipe_id: event_recipe,
                count,
                ..
            },
        ) if recipe_id == event_recipe => *count,
        (
            QuestObjective::ReachLocation { location_id },
            GameEvent::LocationReached {
                location_id: event_location,
            },
        ) if location_id == event_location => target,
        (
            QuestObjective::TraverseLocation {
                from_location_id,
                to_location_id,
            },
            GameEvent::LocationTraversed {
                from_location_id: event_from,
                to_location_id: event_to,
            },
        ) if to_location_id == event_to
            && from_location_id
                .as_ref()
                .map_or(true, |from| from == event_from) =>
        {
            target
        }
        (
            QuestObjective::InteractWith { target_id, .. },
            GameEvent::TargetInteracted {
                target_id: event_target,
            },
        ) if target_id == event_target => 1,
        (
            QuestObjective::CompleteQuest { quest_id },
            GameEvent::PrerequisiteQuestCompleted {
                quest_id: completed_id,
            },
        ) if quest_id == completed_id => target,
        _ => return false,
    };
    *progress = progress.saturating_add(increment).min(target);
    *progress >= target
}

pub fn make_composite_quest(
    id: impl Into<String>,
    title: impl Into<String>,
    description: impl Into<String>,
    stages: Vec<QuestStage>,
    reward: QuestReward,
    is_daily: bool,
) -> Quest {
    Quest {
        id: id.into(),
        title: title.into(),
        description: description.into(),
        objective: QuestObjective::MonsterKills { required: 0 },
        progress: 0,
        target: 0,
        is_completed: false,
        is_claimed: false,
        reward,
        is_daily,
        stages,
    }
}

/// Deduct items only after validating the active hand-in objective. Rewards
/// remain a separate, idempotent claim action.
pub fn hand_in_item_quest(state: &mut GameState, quest_id: &str) -> Result<(), QuestError> {
    let quest_index = state
        .active_quests
        .iter()
        .position(|quest| quest.id == quest_id)
        .ok_or(QuestError::NotFound)?;

    let (item_type, required_count, is_completed, is_claimed) = {
        let quest = &state.active_quests[quest_index];
        if quest.stages.is_empty() {
            match &quest.objective {
                QuestObjective::ItemHandIn {
                    item_type,
                    required_count,
                } => (*item_type, *required_count, quest.is_completed, quest.is_claimed),
                _ => return Err(QuestError::InvalidOperation),
            }
        } else {
            let stage = quest
                .stages
                .iter()
                .find(|stage| !stage.is_completed)
                .ok_or(QuestError::AlreadyCompleted)?;
            let objective = stage
                .objectives
                .iter()
                .find(|objective| {
                    !objective.is_completed
                        && matches!(&objective.objective, QuestObjective::ItemHandIn { .. })
                })
                .ok_or(QuestError::InvalidOperation)?;
            let (item_type, count) = match &objective.objective {
                QuestObjective::ItemHandIn {
                    item_type,
                    required_count,
                } => (*item_type, required_count.saturating_sub(objective.progress)),
                _ => unreachable!("filtered above"),
            };
            (item_type, count, quest.is_completed, quest.is_claimed)
        }
    };

    if is_claimed {
        return Err(QuestError::AlreadyClaimed);
    }
    if is_completed {
        return Err(QuestError::AlreadyCompleted);
    }
    if state.player.get_item_count(item_type) < required_count {
        return Err(QuestError::InsufficientItems);
    }
    if !state.player.remove_item(item_type, required_count) {
        return Err(QuestError::InsufficientItems);
    }

    notify_event(
        state,
        &GameEvent::ItemHandedIn {
            item_type,
            count: required_count,
        },
    );
    Ok(())
}

pub fn claim_quest_reward(
    state: &mut GameState,
    quest_id: &str,
) -> Result<QuestReward, QuestError> {
    let quest_index = state
        .active_quests
        .iter()
        .position(|quest| quest.id == quest_id)
        .ok_or(QuestError::NotFound)?;

    let (id, reward) = {
        let quest = &mut state.active_quests[quest_index];
        if !quest.is_completed {
            return Err(QuestError::NotCompleted);
        }
        if quest.is_claimed {
            return Err(QuestError::AlreadyClaimed);
        }
        quest.is_claimed = true;
        (quest.id.clone(), quest.reward.clone())
    };

    if !state.player.quests_completed.contains(&id) {
        state.player.quests_completed.push(id);
    }
    grant_reward(state, &reward);
    Ok(reward)
}

fn grant_reward(state: &mut GameState, reward: &QuestReward) {
    match reward {
        QuestReward::Gold(amount) => {
            state.player.gold = state.player.gold.saturating_add(*amount).min(MAX_GOLD);
        }
        QuestReward::Xp(amount) => {
            state.player.xp = state.player.xp.saturating_add(*amount).min(1_000_000);
            state.player.xp_until_next = state.player.xp_until_next.saturating_sub(*amount);
            if state.player.xp_until_next == 0 && state.player.combat_level < MAX_PLAYER_LEVEL {
                state.player.level_up_combat();
                let new_level = state.player.combat_level;
                notify_event(state, &GameEvent::PlayerLeveledUp { new_level });
            }
        }
        QuestReward::Item { item_type, count } => state.player.add_item(*item_type, *count),
    }
}

pub fn quest_hall(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        check_and_refresh_quests(state);
        let now = Utc::now().timestamp().max(0) as u64;
        let world_event_text = if state.active_world_event.is_active(now) {
            format!(
                "{}\n{}",
                state.active_world_event.name, state.active_world_event.description
            )
        } else {
            "No active world event.".into()
        };
        let quest_text = if state.active_quests.is_empty() {
            "No active quests. Return later for new daily objectives.".to_string()
        } else {
            state
                .active_quests
                .iter()
                .map(|quest| {
                    let status = if quest.is_claimed {
                        "Claimed"
                    } else if quest.is_completed {
                        "Ready to claim"
                    } else {
                        "In progress"
                    };
                    let stage_detail = if quest.stages.is_empty() {
                        String::new()
                    } else {
                        let current = quest.stages.iter().position(|stage| !stage.is_completed);
                        let stages = quest.stages.iter().enumerate().map(|(index, stage)| {
                            let stage_status = if stage.is_completed {
                                "complete"
                            } else if current == Some(index) {
                                "current"
                            } else {
                                "locked"
                            };
                            let goals = stage.objectives.iter().map(|objective| {
                                let mark = if objective.is_completed { "✓" } else { "·" };
                                let optional = if objective.optional { " (optional)" } else { "" };
                                format!(
                                    "    {mark} {}/{} {:?}{optional}",
                                    objective.progress,
                                    objective_target(&objective.objective),
                                    objective.objective
                                )
                            }).collect::<Vec<_>>().join("\n");
                            format!("  Stage {} — {} [{stage_status}]\n{}", index + 1, stage.title, goals)
                        }).collect::<Vec<_>>().join("\n");
                        format!("\n{stages}")
                    };
                    format!(
                        "{} — {}\n  Progress: {}/{} · {}{}\n",
                        quest.title, quest.description, quest.progress, quest.target, status, stage_detail
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let content = format!(
            "World event\n{}\n\nLifetime monsters defeated: {}\n\nQuests\n{}\n\nDaily quests refresh automatically every 24 hours.",
            world_event_text, state.player.lifetime_kills, quest_text
        );
        tui.show_text("Quest Hall", &content, Some(state))?;

        let choice = tui.select_menu(
            "Quest Hall Actions",
            &["Claim completed rewards", "Hand in quest items", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let claimable: Vec<String> = state
                    .active_quests
                    .iter()
                    .filter(|quest| quest.is_completed && !quest.is_claimed)
                    .map(|quest| quest.id.clone())
                    .collect();
                if claimable.is_empty() {
                    tui.show_message(
                        "Quest Hall",
                        "No completed rewards are available to claim.",
                        Some(state),
                    )?;
                } else {
                    for quest_id in claimable {
                        if let Ok(reward) = claim_quest_reward(state, &quest_id) {
                            tui.show_message(
                                "Reward claimed",
                                &format!("Claimed reward for {quest_id}: {reward:?}"),
                                Some(state),
                            )?;
                        }
                    }
                }
            }
            1 => {
                let hand_ins: Vec<(String, String)> = state
                    .active_quests
                    .iter()
                    .filter_map(|quest| {
                        if quest.is_claimed || quest.is_completed {
                            return None;
                        }
                        match &quest.objective {
                            QuestObjective::ItemHandIn {
                                item_type,
                                required_count,
                            } => Some((
                                quest.id.clone(),
                                format!(
                                    "{} — hand in {} {}",
                                    quest.title,
                                    required_count,
                                    item_type.name()
                                ),
                            )),
                            _ => None,
                        }
                    })
                    .collect();
                if hand_ins.is_empty() {
                    tui.show_message(
                        "Quest Hall",
                        "There are no available item hand-ins right now.",
                        Some(state),
                    )?;
                    continue;
                }

                let labels: Vec<String> = hand_ins.iter().map(|(_, label)| label.clone()).collect();
                let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
                options.push("Back");
                let selected = tui.select_menu("Choose a hand-in", &options, Some(state))?;
                if selected < hand_ins.len() {
                    match hand_in_item_quest(state, &hand_ins[selected].0) {
                        Ok(()) => tui.show_message(
                            "Hand-in complete",
                            "The required items were deducted and the quest is ready to claim.",
                            Some(state),
                        )?,
                        Err(error) => {
                            tui.show_message("Hand-in failed", &error.to_string(), Some(state))?
                        }
                    }
                }
            }
            _ => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Player;

    fn new_state() -> GameState {
        GameState::new(Player::new("Hero".into(), String::new()))
    }

    fn objective(objective: QuestObjective, optional: bool, bonus_reward: Option<QuestReward>) -> QuestObjectiveState {
        QuestObjectiveState { objective, progress: 0, is_completed: false, optional, bonus_reward }
    }

    #[test]
    fn monster_events_update_quest_progress_and_lifetime_kills_once_per_event() {
        let mut state = new_state();
        let quest_id = state
            .active_quests
            .iter()
            .find(|quest| matches!(&quest.objective, QuestObjective::MonsterKills { .. }))
            .unwrap()
            .id
            .clone();
        let target = state
            .active_quests
            .iter()
            .find(|quest| quest.id == quest_id)
            .unwrap()
            .target;

        for _ in 0..target {
            notify_event(
                &mut state,
                &GameEvent::MonsterDefeated {
                    name: "Goblin".into(),
                    is_stronghold: false,
                },
            );
        }

        let quest = state
            .active_quests
            .iter()
            .find(|quest| quest.id == quest_id)
            .unwrap();
        assert_eq!(state.player.lifetime_kills, target);
        assert_eq!(quest.progress, target);
        assert!(quest.is_completed);
    }

    #[test]
    fn reward_claiming_is_idempotent() {
        let mut state = new_state();
        let quest_id = state
            .active_quests
            .iter()
            .find(|quest| matches!(&quest.objective, QuestObjective::MonsterKills { .. }))
            .unwrap()
            .id
            .clone();
        let target = state
            .active_quests
            .iter()
            .find(|quest| quest.id == quest_id)
            .unwrap()
            .target;
        for _ in 0..target {
            notify_event(
                &mut state,
                &GameEvent::MonsterDefeated {
                    name: "Goblin".into(),
                    is_stronghold: false,
                },
            );
        }

        let gold_before = state.player.gold;
        assert_eq!(
            claim_quest_reward(&mut state, &quest_id),
            Ok(QuestReward::Gold(250))
        );
        assert_eq!(state.player.gold, gold_before + 250);
        assert_eq!(
            claim_quest_reward(&mut state, &quest_id),
            Err(QuestError::AlreadyClaimed)
        );
        assert_eq!(state.player.gold, gold_before + 250);
        assert!(state.player.quests_completed.contains(&quest_id));
    }

    #[test]
    fn level_events_complete_combat_and_skill_objectives() {
        let mut state = new_state();
        let combat = state
            .active_quests
            .iter()
            .find(|quest| matches!(&quest.objective, QuestObjective::PlayerLevel { .. }))
            .unwrap()
            .clone();
        if let QuestObjective::PlayerLevel { required_level } = combat.objective {
            notify_event(
                &mut state,
                &GameEvent::PlayerLeveledUp {
                    new_level: required_level,
                },
            );
            assert!(
                state
                    .active_quests
                    .iter()
                    .find(|quest| quest.id == combat.id)
                    .unwrap()
                    .is_completed
            );
        }

        let skill = state
            .active_quests
            .iter()
            .find(|quest| matches!(&quest.objective, QuestObjective::SkillLevel { .. }))
            .unwrap()
            .clone();
        if let QuestObjective::SkillLevel {
            skill_name,
            required_level,
        } = skill.objective
        {
            notify_event(
                &mut state,
                &GameEvent::SkillLeveledUp {
                    skill_name,
                    new_level: required_level,
                },
            );
            assert!(
                state
                    .active_quests
                    .iter()
                    .find(|quest| quest.id == skill.id)
                    .unwrap()
                    .is_completed
            );
        }
    }

    #[test]
    fn item_hand_in_does_not_consume_items_when_requirements_are_not_met() {
        let mut state = new_state();
        let quest_id = state
            .active_quests
            .iter()
            .find(|quest| matches!(&quest.objective, QuestObjective::ItemHandIn { .. }))
            .unwrap()
            .id
            .clone();

        assert_eq!(
            hand_in_item_quest(&mut state, &quest_id),
            Err(QuestError::InsufficientItems)
        );
        assert_eq!(state.player.get_item_count(ItemType::Ore), 0);

        state.player.add_item(ItemType::Ore, 5);
        assert_eq!(hand_in_item_quest(&mut state, &quest_id), Ok(()));
        assert_eq!(state.player.get_item_count(ItemType::Ore), 2);
        assert!(
            state
                .active_quests
                .iter()
                .find(|quest| quest.id == quest_id)
                .unwrap()
                .is_completed
        );
        assert_eq!(
            claim_quest_reward(&mut state, &quest_id),
            Ok(QuestReward::Item {
                item_type: ItemType::Ingots,
                count: 2,
            })
        );
        assert_eq!(state.player.get_item_count(ItemType::Ingots), 2);
        assert_eq!(
            hand_in_item_quest(&mut state, &quest_id),
            Err(QuestError::AlreadyClaimed)
        );
    }

    #[test]
    fn composite_stage_tracks_parallel_objectives_independently() {
        let mut state = new_state();
        state.active_quests.clear();
        state.active_quests.push(make_composite_quest(
            "composite_parallel",
            "Survey and Supply",
            "Do both goals.",
            vec![QuestStage {
                id: "first".into(),
                title: "Gather and defend".into(),
                is_completed: false,
                objectives: vec![
                    objective(
                        QuestObjective::SpecificMonsterKills {
                            monster_name: "Goblin".into(),
                            required: 2,
                        },
                        false,
                        None,
                    ),
                    objective(
                        QuestObjective::GatherResource {
                            item_type: ItemType::Logs,
                            required_count: 3,
                        },
                        false,
                        None,
                    ),
                ],
            }],
            QuestReward::Gold(50),
            false,
        ));
        notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Rat".into(),
                is_stronghold: false,
            },
        );
        notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Goblin".into(),
                is_stronghold: false,
            },
        );
        notify_event(
            &mut state,
            &GameEvent::ResourceGathered {
                skill_name: "Woodcutting".into(),
                item_type: ItemType::Logs,
                count: 2,
            },
        );
        assert_eq!(state.active_quests[0].stages[0].objectives[0].progress, 1);
        assert_eq!(state.active_quests[0].stages[0].objectives[1].progress, 2);
        assert!(!state.active_quests[0].is_completed);
        notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Goblin".into(),
                is_stronghold: false,
            },
        );
        notify_event(
            &mut state,
            &GameEvent::ResourceGathered {
                skill_name: "Woodcutting".into(),
                item_type: ItemType::Logs,
                count: 1,
            },
        );
        assert!(state.active_quests[0].is_completed);
    }

    #[test]
    fn sequential_stage_is_locked_until_previous_required_objectives_finish() {
        let mut state = new_state();
        state.active_quests.clear();
        state.active_quests.push(make_composite_quest(
            "sequence",
            "Two Steps",
            "Sequential stages.",
            vec![
                QuestStage {
                    id: "first".into(),
                    title: "Defeat".into(),
                    is_completed: false,
                    objectives: vec![objective(
                        QuestObjective::SpecificMonsterKills {
                            monster_name: "Goblin".into(),
                            required: 1,
                        },
                        false,
                        None,
                    )],
                },
                QuestStage {
                    id: "second".into(),
                    title: "Arrive".into(),
                    is_completed: false,
                    objectives: vec![objective(
                        QuestObjective::ReachLocation {
                            location_id: LocationId::from("abandoned_mill"),
                        },
                        false,
                        None,
                    )],
                },
            ],
            QuestReward::Gold(5),
            false,
        ));
        notify_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("abandoned_mill"),
            },
        );
        assert_eq!(state.active_quests[0].stages[1].objectives[0].progress, 0);
        notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Goblin".into(),
                is_stronghold: false,
            },
        );
        assert!(state.active_quests[0].stages[0].is_completed);
        assert!(!state.active_quests[0].stages[1].is_completed);
        notify_event(
            &mut state,
            &GameEvent::LocationReached {
                location_id: LocationId::from("abandoned_mill"),
            },
        );
        assert!(state.active_quests[0].is_completed);
    }

    #[test]
    fn optional_bonus_reward_does_not_block_main_completion_and_is_granted_once() {
        let mut state = new_state();
        state.active_quests.clear();
        state.active_quests.push(make_composite_quest(
            "optional",
            "Bonus",
            "Bonus doesn't block.",
            vec![QuestStage {
                id: "stage".into(),
                title: "Main objective".into(),
                is_completed: false,
                objectives: vec![
                    objective(
                        QuestObjective::SpecificMonsterKills {
                            monster_name: "Goblin".into(),
                            required: 1,
                        },
                        false,
                        None,
                    ),
                    objective(
                        QuestObjective::CollectItem {
                            item_type: ItemType::Thread,
                            required_count: 1,
                        },
                        true,
                        Some(QuestReward::Gold(7)),
                    ),
                ],
            }],
            QuestReward::Gold(1),
            false,
        ));
        let starting_gold = state.player.gold;
        notify_event(
            &mut state,
            &GameEvent::ItemCollected {
                item_type: ItemType::Thread,
                count: 1,
            },
        );
        assert_eq!(state.player.gold, starting_gold + 7);
        notify_event(
            &mut state,
            &GameEvent::MonsterDefeated {
                name: "Goblin".into(),
                is_stronghold: false,
            },
        );
        assert!(state.active_quests[0].is_completed);
        assert_eq!(state.player.gold, starting_gold + 7);
    }

    #[test]
    fn legacy_single_objective_quest_json_preserves_existing_progress() {
        let state = new_state();
        let mut legacy = state.active_quests[0].clone();
        legacy.progress = 4;
        legacy.target = 9;
        let mut value = serde_json::to_value(&legacy).unwrap();
        value.as_object_mut().unwrap().remove("stages");
        let restored: Quest = serde_json::from_value(value).unwrap();
        assert!(restored.stages.is_empty());
        assert_eq!(restored.progress, 4);
        assert_eq!(restored.target, 9);
        assert_eq!(restored.objective, legacy.objective);
    }

    #[test]
    fn daily_refresh_preserves_history_and_generates_new_daily_ids() {
        let mut state = new_state();
        state
            .player
            .quests_completed
            .push("historical_quest_1".into());
        state.active_quests.clear();
        state.player.last_quest_refresh = Some(DateTime::from_timestamp(1, 0).unwrap());

        refresh_quests_at(&mut state, DateTime::from_timestamp(86_402, 0).unwrap());

        assert!(state
            .player
            .quests_completed
            .contains(&"historical_quest_1".to_string()));
        assert!(state
            .active_quests
            .iter()
            .any(|quest| quest.id.ends_with("_1")));
        assert!(state.active_world_event.is_active(86_402));
    }
}
