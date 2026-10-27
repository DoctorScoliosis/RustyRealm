use crate::state::GameState;
use crate::ui::{clear_screen, wait_for_enter};
use chrono::{Utc, Duration};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub description: String,
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
    
    // Combat level quests scaled to player level
    let target_combat_lvl = (state.player.combat_level + rng.gen_range(1..=5)).min(37);
    new_quests.push(Quest {
        id: format!("combat_lvl_{}", target_combat_lvl),
        description: format!("Reach combat level {}", target_combat_lvl),
        is_completed: state.player.combat_level >= target_combat_lvl,
    });

    // Kill count quests
    let kill_targets = [10, 50, 100, 250, 500, 1000];
    let target_kills = kill_targets.iter().find(|&&t| t > state.player.stronghold_kill_count).copied().unwrap_or(1000);
    new_quests.push(Quest {
        id: format!("kill_{}", target_kills),
        description: format!("Kill {} monsters in the Stronghold", target_kills),
        is_completed: state.player.stronghold_kill_count >= target_kills,
    });

    // Skill quests based on lowest skill
    let skills = [
        ("Woodcutting", state.player.skills.woodcutting.level),
        ("Cooking", state.player.skills.cooking.level),
        ("Fishing", state.player.skills.fishing.level),
        ("Mining", state.player.skills.mining.level),
        ("Smithing", state.player.skills.smithing.level),
        ("Thieving", state.player.skills.thieving.level),
    ];
    
    let lowest_skill = skills.iter().min_by_key(|&&(_, lvl)| lvl).unwrap();
    let target_skill_lvl = (lowest_skill.1 + rng.gen_range(1..=10)).min(100);
    new_quests.push(Quest {
        id: format!("skill_{}_{}", lowest_skill.0, target_skill_lvl),
        description: format!("Reach level {} in {}", target_skill_lvl, lowest_skill.0),
        is_completed: lowest_skill.1 >= target_skill_lvl,
    });

    // Total level quest
    let target_total = (total_lvl + rng.gen_range(10..=50)).min(637);
    new_quests.push(Quest {
        id: format!("total_lvl_{}", target_total),
        description: format!("Reach total level {}", target_total),
        is_completed: total_lvl >= target_total,
    });

    // Keep previously completed quests that are still relevant
    let mut completed_quests: Vec<Quest> = state.player.quests_completed.iter().map(|id| {
        Quest {
            id: id.clone(),
            description: "Completed quest".to_string(), // Simplified for persistence
            is_completed: true,
        }
    }).collect();

    // Merge and deduplicate
    let mut all_quests = new_quests;
    for cq in completed_quests {
        if !all_quests.iter().any(|q| q.id == cq.id) {
            all_quests.push(cq);
        }
    }

    // Update state (simplified: we just store completed IDs)
    state.player.quests_completed = all_quests.iter().filter(|q| q.is_completed).map(|q| q.id.clone()).collect();
    
    // For display purposes, we'll regenerate the active list
    state.player.quests_completed = all_quests.iter().filter(|q| q.is_completed).map(|q| q.id.clone()).collect();
}

pub fn quest_hall(state: &mut GameState) {
    check_and_refresh_quests(state);
    
    loop {
        clear_screen();
        println!("Welcome to the Quest Hall!\n");
        
        let total_lvl = state.player.skills.total_level() + state.player.combat_level;
        let skills = [
            ("Woodcutting", state.player.skills.woodcutting.level),
            ("Cooking", state.player.skills.cooking.level),
            ("Fishing", state.player.skills.fishing.level),
            ("Mining", state.player.skills.mining.level),
            ("Smithing", state.player.skills.smithing.level),
            ("Thieving", state.player.skills.thieving.level),
        ];
        
        // Generate current active quests for display
        let mut active_quests = Vec::new();
        let mut completed_quests = Vec::new();
        
        let target_combat = (state.player.combat_level + 5).min(37);
        if state.player.combat_level >= target_combat {
            completed_quests.push(format!("Reach combat level {}", target_combat));
        } else {
            active_quests.push(format!("Reach combat level {}", target_combat));
        }

        let kill_targets = [10, 50, 100, 250, 500, 1000];
        for &target in &kill_targets {
            if state.player.stronghold_kill_count >= target {
                completed_quests.push(format!("Kill {} monsters in the Stronghold", target));
            } else if active_quests.len() < 3 {
                active_quests.push(format!("Kill {} monsters in the Stronghold", target));
            }
        }

        let lowest_skill = skills.iter().min_by_key(|&&(_, lvl)| lvl).unwrap();
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

        println!("Active Quests:");
        if active_quests.is_empty() {
            println!("  - None! Check back tomorrow for new quests.");
        } else {
            for q in &active_quests {
                println!("  - {}", q);
            }
        }
        
        println!("\nCompleted Quests:");
        if completed_quests.is_empty() {
            println!("  - None yet. Keep playing!");
        } else {
            for q in &completed_quests {
                println!("  - [DONE] {}", q);
            }
        }

        println!("\nQuests refresh daily based on your current progress.");
        println!("\nType \"back\" to return.\n");

        let choice = crate::ui::get_input("Choice: ");
        if choice.to_lowercase() == "back" {
            return;
        }
    }
}