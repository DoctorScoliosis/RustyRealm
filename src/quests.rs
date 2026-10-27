use crate::state::GameState;
use crate::ui::Tui;
use chrono::{Duration, Utc};
use rand::Rng;
use std::io;

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub is_completed: bool,
}

pub fn check_and_refresh_quests(state: &mut GameState) {
    let now = Utc::now();
    let should_refresh = match state.player.last_quest_refresh {
        Some(last) => (now - last) >= Duration::days(1),
        None => true,
    };

    if should_refresh {
        generate_daily_quests(state);
        state.player.last_quest_refresh = Some(now);
    }
}

fn generate_daily_quests(state: &mut GameState) {
    let mut rng = rand::thread_rng();
    let mut new_quests = Vec::new();
    let total_lvl = state.player.skills.total_level() + state.player.combat_level;

    let target_combat_lvl = (state.player.combat_level + rng.gen_range(1..=5)).min(37);
    new_quests.push(Quest {
        id: format!("combat_lvl_{}", target_combat_lvl),
        is_completed: state.player.combat_level >= target_combat_lvl,
    });

    let kill_targets = [10, 50, 100, 250, 500, 1000];
    let target_kills = kill_targets.iter()
        .find(|&&target| target > state.player.stronghold_kill_count)
        .copied()
        .unwrap_or(1000);
    new_quests.push(Quest {
        id: format!("kill_{}", target_kills),
        is_completed: state.player.stronghold_kill_count >= target_kills,
    });

    let skills = [
        ("Woodcutting", state.player.skills.woodcutting.level),
        ("Cooking", state.player.skills.cooking.level),
        ("Fishing", state.player.skills.fishing.level),
        ("Mining", state.player.skills.mining.level),
        ("Smithing", state.player.skills.smithing.level),
        ("Thieving", state.player.skills.thieving.level),
    ];
    let lowest_skill = skills.iter().min_by_key(|&&(_, level)| level).unwrap();
    let target_skill_lvl = (lowest_skill.1 + rng.gen_range(1..=10)).min(100);
    new_quests.push(Quest {
        id: format!("skill_{}_{}", lowest_skill.0, target_skill_lvl),
        is_completed: lowest_skill.1 >= target_skill_lvl,
    });

    let target_total = (total_lvl + rng.gen_range(10..=50)).min(637);
    new_quests.push(Quest {
        id: format!("total_lvl_{}", target_total),
        is_completed: total_lvl >= target_total,
    });

    let completed_quests: Vec<Quest> = state.player.quests_completed.iter().map(|id| Quest {
        id: id.clone(),
        is_completed: true,
    }).collect();

    let mut all_quests = new_quests;
    for quest in completed_quests {
        if !all_quests.iter().any(|existing| existing.id == quest.id) {
            all_quests.push(quest);
        }
    }
    state.player.quests_completed = all_quests
        .iter()
        .filter(|quest| quest.is_completed)
        .map(|quest| quest.id.clone())
        .collect();
}

pub fn quest_hall(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    check_and_refresh_quests(state);
    let total_lvl = state.player.skills.total_level() + state.player.combat_level;
    let skills = [
        ("Woodcutting", state.player.skills.woodcutting.level),
        ("Cooking", state.player.skills.cooking.level),
        ("Fishing", state.player.skills.fishing.level),
        ("Mining", state.player.skills.mining.level),
        ("Smithing", state.player.skills.smithing.level),
        ("Thieving", state.player.skills.thieving.level),
    ];

    let mut active_quests = Vec::new();
    let mut completed_quests = Vec::new();

    let target_combat = (state.player.combat_level + 5).min(37);
    if state.player.combat_level >= target_combat {
        completed_quests.push(format!("Reach combat level {}", target_combat));
    } else {
        active_quests.push(format!("Reach combat level {}", target_combat));
    }

    let kill_targets = [10, 50, 100, 250, 500, 1000];
    for target in kill_targets {
        if state.player.stronghold_kill_count >= target {
            completed_quests.push(format!("Kill {} monsters in the Stronghold", target));
        } else if active_quests.len() < 3 {
            active_quests.push(format!("Kill {} monsters in the Stronghold", target));
        }
    }

    let lowest_skill = skills.iter().min_by_key(|&&(_, level)| level).unwrap();
    let target_skill = (lowest_skill.1 + 10).min(100);
    if lowest_skill.1 >= target_skill {
        completed_quests.push(format!("Reach level {} in {}", target_skill, lowest_skill.0));
    } else {
        active_quests.push(format!("Reach level {} in {}", target_skill, lowest_skill.0));
    }

    let target_total = (total_lvl + 50).min(637);
    if total_lvl >= target_total {
        completed_quests.push(format!("Reach total level {}", target_total));
    } else {
        active_quests.push(format!("Reach total level {}", target_total));
    }

    let active_text = if active_quests.is_empty() {
        "  None! Check back tomorrow for new quests.".to_string()
    } else {
        active_quests.iter().map(|quest| format!("  • {}", quest)).collect::<Vec<_>>().join("\n")
    };
    let completed_text = if completed_quests.is_empty() {
        "  None yet. Keep playing!".to_string()
    } else {
        completed_quests.iter().map(|quest| format!("  ✓ {}", quest)).collect::<Vec<_>>().join("\n")
    };
    let content = format!(
        "Active quests\n{}\n\nCompleted quests\n{}\n\nQuests refresh daily based on your current progress.",
        active_text, completed_text
    );
    tui.show_text("Quest Hall", &content, Some(state))
}
