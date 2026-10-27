use crate::data::{ItemType, MAX_GOLD, MAX_SKILL_LEVEL};
use crate::state::{GameState, SkillData};
use crate::ui::Tui;
use std::io;

fn award_xp(skill: &mut SkillData) -> bool {
    skill.current_xp += skill.xp_gain_per_action;
    skill.xp_remaining = skill.xp_remaining.saturating_sub(skill.xp_gain_per_action);
    if skill.xp_remaining != 0 {
        return false;
    }
    skill.level += 1;
    if skill.level >= MAX_SKILL_LEVEL {
        skill.level = MAX_SKILL_LEVEL;
        skill.xp_remaining = 0;
    } else {
        skill.xp_threshold += 100;
        skill.xp_remaining = skill.xp_threshold;
    }
    true
}

#[derive(Clone, Copy)]
enum SkillKind {
    Woodcutting,
    Cooking,
    Fishing,
    Mining,
    Smithing,
    Thieving,
}

impl SkillKind {
    fn name(self) -> &'static str {
        match self {
            Self::Woodcutting => "Woodcutting",
            Self::Cooking => "Cooking",
            Self::Fishing => "Fishing",
            Self::Mining => "Mining",
            Self::Smithing => "Smithing",
            Self::Thieving => "Thieving",
        }
    }

    fn data(self, state: &GameState) -> &SkillData {
        match self {
            Self::Woodcutting => &state.player.skills.woodcutting,
            Self::Cooking => &state.player.skills.cooking,
            Self::Fishing => &state.player.skills.fishing,
            Self::Mining => &state.player.skills.mining,
            Self::Smithing => &state.player.skills.smithing,
            Self::Thieving => &state.player.skills.thieving,
        }
    }

    fn data_mut(self, state: &mut GameState) -> &mut SkillData {
        match self {
            Self::Woodcutting => &mut state.player.skills.woodcutting,
            Self::Cooking => &mut state.player.skills.cooking,
            Self::Fishing => &mut state.player.skills.fishing,
            Self::Mining => &mut state.player.skills.mining,
            Self::Smithing => &mut state.player.skills.smithing,
            Self::Thieving => &mut state.player.skills.thieving,
        }
    }

    fn action(self) -> &'static str {
        match self {
            Self::Woodcutting => "Woodcut",
            Self::Cooking => "Cook",
            Self::Fishing => "Fish",
            Self::Mining => "Mine",
            Self::Smithing => "Smith",
            Self::Thieving => "Pickpocket",
        }
    }
}

pub fn skill_plot(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let skills = [
        SkillKind::Woodcutting, SkillKind::Cooking, SkillKind::Fishing,
        SkillKind::Mining, SkillKind::Smithing, SkillKind::Thieving,
    ];
    loop {
        let labels: Vec<String> = skills.iter().map(|skill| {
            let data = skill.data(state);
            format!("{} · level {} · {} XP / threshold {}", skill.name(), data.level, data.current_xp, data.xp_threshold)
        }).collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Skill Plot · total skill level {}", state.player.skills.total_level()),
            &options,
            Some(state),
        )?;
        if choice >= skills.len() {
            return Ok(());
        }
        skill_menu(tui, state, skills[choice])?;
    }
}

fn skill_menu(tui: &mut Tui, state: &mut GameState, skill: SkillKind) -> io::Result<()> {
    let mut status = String::new();
    loop {
        let data = skill.data(state);
        let detail = format!(
            "{} · level {} · XP {} · remaining {}",
            skill.name(), data.level, data.current_xp, data.xp_remaining
        );
        let action_label = format!("{}{}", skill.action(), if status.is_empty() { String::new() } else { format!(" — {}", status) });
        let action_options = [action_label.as_str(), "Back"];
        let choice = tui.select_menu(&detail, &action_options, Some(state))?;
        if choice != 0 {
            return Ok(());
        }

        status = match skill {
            SkillKind::Woodcutting => {
                let level = skill.data(state).level;
                if level >= MAX_SKILL_LEVEL {
                    state.player.add_item(ItemType::Logs, 100);
                    "Mastery: instantly cut 100 logs.".to_string()
                } else {
                    state.player.add_item(ItemType::Logs, 1);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message(
                            "Skill level up",
                            &format!("Congratulations, your Woodcutting level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }),
                            Some(state),
                        )?;
                    }
                    "You cut some logs.".to_string()
                }
            }
            SkillKind::Cooking => {
                let raw_food = state.player.get_item_count(ItemType::RawFood);
                if raw_food == 0 {
                    tui.show_message("Missing materials", "You do not have enough raw food to cook.", Some(state))?;
                    continue;
                }
                if skill.data(state).level >= MAX_SKILL_LEVEL {
                    state.player.remove_item(ItemType::RawFood, raw_food);
                    state.player.add_item(ItemType::Food, raw_food);
                    "Mastery: cooked all your raw food.".to_string()
                } else {
                    state.player.remove_item(ItemType::RawFood, 1);
                    state.player.add_item(ItemType::Food, 1);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message("Skill level up", &format!("Congratulations, your Cooking level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }), Some(state))?;
                    }
                    "You cooked some food.".to_string()
                }
            }
            SkillKind::Fishing => {
                let bait = state.player.get_item_count(ItemType::Bait);
                if bait == 0 {
                    tui.show_message("Missing materials", "You do not have enough bait to fish.", Some(state))?;
                    continue;
                }
                if skill.data(state).level >= MAX_SKILL_LEVEL {
                    state.player.remove_item(ItemType::Bait, bait);
                    state.player.add_item(ItemType::RawFood, bait);
                    "Mastery: used all your bait at once.".to_string()
                } else {
                    state.player.remove_item(ItemType::Bait, 1);
                    state.player.add_item(ItemType::RawFood, 1);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message("Skill level up", &format!("Congratulations, your Fishing level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }), Some(state))?;
                    }
                    "You caught a fish.".to_string()
                }
            }
            SkillKind::Mining => {
                if skill.data(state).level >= MAX_SKILL_LEVEL {
                    state.player.add_item(ItemType::Ore, 100);
                    "Mastery: instantly mined 100 ore.".to_string()
                } else {
                    state.player.add_item(ItemType::Ore, 1);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message("Skill level up", &format!("Congratulations, your Mining level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }), Some(state))?;
                    }
                    "You mined some ore.".to_string()
                }
            }
            SkillKind::Smithing => {
                let ore = state.player.get_item_count(ItemType::Ore);
                if ore == 0 {
                    tui.show_message("Missing materials", "You do not have enough ore to smith an ingot.", Some(state))?;
                    continue;
                }
                if skill.data(state).level >= MAX_SKILL_LEVEL {
                    state.player.remove_item(ItemType::Ore, ore);
                    state.player.add_item(ItemType::Ingots, ore);
                    "Mastery: smelted all your ore.".to_string()
                } else {
                    state.player.remove_item(ItemType::Ore, 1);
                    state.player.add_item(ItemType::Ingots, 1);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message("Skill level up", &format!("Congratulations, your Smithing level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }), Some(state))?;
                    }
                    "You smithed an ingot.".to_string()
                }
            }
            SkillKind::Thieving => {
                if skill.data(state).level >= MAX_SKILL_LEVEL {
                    state.player.gold = state.player.gold.saturating_add(10_000).min(MAX_GOLD);
                    "Mastery: stole 10,000 coins.".to_string()
                } else {
                    state.player.gold = state.player.gold.saturating_add(10).min(MAX_GOLD);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        tui.show_message("Skill level up", &format!("Congratulations, your Thieving level is now {}{}.", new_level, if new_level >= MAX_SKILL_LEVEL { " (maximum level)" } else { "" }), Some(state))?;
                    }
                    "You stole some coins.".to_string()
                }
            }
        };
    }
}
