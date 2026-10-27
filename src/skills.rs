use crate::state::{GameState, SkillData};
use crate::data::{ItemType, MAX_SKILL_LEVEL};
use crate::ui::clear_screen;

fn award_xp(skill: &mut SkillData) -> bool {
    skill.current_xp += skill.xp_gain_per_action;
    skill.xp_remaining = skill.xp_remaining.saturating_sub(skill.xp_gain_per_action);
    if skill.xp_remaining != 0 { return false; }
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

pub fn skill_plot(state: &mut GameState) {
    loop {
        clear_screen();
        println!("Skills:\n");
        println!("1) Woodcutting (Level: {}, XP: {}/{})", 
                 state.player.skills.woodcutting.level, 
                 state.player.skills.woodcutting.current_xp,
                 state.player.skills.woodcutting.xp_threshold);
        println!("2) Cooking (Level: {}, XP: {}/{})", 
                 state.player.skills.cooking.level, 
                 state.player.skills.cooking.current_xp,
                 state.player.skills.cooking.xp_threshold);
        println!("3) Fishing (Level: {}, XP: {}/{})", 
                 state.player.skills.fishing.level, 
                 state.player.skills.fishing.current_xp,
                 state.player.skills.fishing.xp_threshold);
        println!("4) Mining (Level: {}, XP: {}/{})", 
                 state.player.skills.mining.level, 
                 state.player.skills.mining.current_xp,
                 state.player.skills.mining.xp_threshold);
        println!("5) Smithing (Level: {}, XP: {}/{})", 
                 state.player.skills.smithing.level, 
                 state.player.skills.smithing.current_xp,
                 state.player.skills.smithing.xp_threshold);
        println!("6) Thieving (Level: {}, XP: {}/{})", 
                 state.player.skills.thieving.level, 
                 state.player.skills.thieving.current_xp,
                 state.player.skills.thieving.xp_threshold);
        println!("\nType \"exit\" to leave.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => woodcutting_menu(state),
            "2" => cooking_menu(state),
            "3" => fishing_menu(state),
            "4" => mining_menu(state),
            "5" => smithing_menu(state),
            "6" => thieving_menu(state),
            "exit" => return,
            _ => continue,
        }
    }
}

fn woodcutting_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.woodcutting; (s.level, s.current_xp, s.xp_remaining) };
        println!("Woodcutting level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Logs: {}\n", state.player.get_item_count(ItemType::Logs));
        println!("1) Woodcut");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if level >= MAX_SKILL_LEVEL {
                    let logs_to_add = 100;
                    state.player.add_item(ItemType::Logs, logs_to_add);
                    println!("\nBecause you are a master at this skill, you instantly cut {} logs at a time.", logs_to_add);
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.add_item(ItemType::Logs, 1);
                if award_xp(&mut state.player.skills.woodcutting) {
                    let new_level = state.player.skills.woodcutting.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou cut some logs.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}

fn cooking_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.cooking; (s.level, s.current_xp, s.xp_remaining) };
        println!("Cooking level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Raw Food: {}", state.player.get_item_count(ItemType::RawFood));
        println!("Cooked Food: {}\n", state.player.get_item_count(ItemType::Food));
        println!("1) Cook");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if state.player.get_item_count(ItemType::RawFood) == 0 {
                    println!("\nYou do not have enough of the required materials to make this item.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                if level >= MAX_SKILL_LEVEL {
                    let raw_food = state.player.get_item_count(ItemType::RawFood);
                    state.player.remove_item(ItemType::RawFood, raw_food);
                    state.player.add_item(ItemType::Food, raw_food);
                    println!("\nBecause you are a master at this skill, you instantly cook all of your food.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.remove_item(ItemType::RawFood, 1);
                state.player.add_item(ItemType::Food, 1);
                if award_xp(&mut state.player.skills.cooking) {
                    let new_level = state.player.skills.cooking.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou cook some food.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}

fn fishing_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.fishing; (s.level, s.current_xp, s.xp_remaining) };
        println!("Fishing level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Bait: {}", state.player.get_item_count(ItemType::Bait));
        println!("Raw Food: {}\n", state.player.get_item_count(ItemType::RawFood));
        println!("1) Fish");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if state.player.get_item_count(ItemType::Bait) == 0 {
                    println!("\nYou do not have enough of the required materials to make this item.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                if level >= MAX_SKILL_LEVEL {
                    let bait = state.player.get_item_count(ItemType::Bait);
                    state.player.remove_item(ItemType::Bait, bait);
                    state.player.add_item(ItemType::RawFood, bait);
                    println!("\nBecause you are a master at this skill, you instantly use up all of your bait.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.remove_item(ItemType::Bait, 1);
                state.player.add_item(ItemType::RawFood, 1);
                if award_xp(&mut state.player.skills.fishing) {
                    let new_level = state.player.skills.fishing.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou catch a fish.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}

fn mining_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.mining; (s.level, s.current_xp, s.xp_remaining) };
        println!("Mining level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Ore: {}\n", state.player.get_item_count(ItemType::Ore));
        println!("1) Mine");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if level >= MAX_SKILL_LEVEL {
                    let ore_to_add = 100;
                    state.player.add_item(ItemType::Ore, ore_to_add);
                    println!("\nBecause you are a master at this skill, you instantly mine {} ore at a time.", ore_to_add);
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.add_item(ItemType::Ore, 1);
                if award_xp(&mut state.player.skills.mining) {
                    let new_level = state.player.skills.mining.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou mine some ore.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}

fn smithing_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.smithing; (s.level, s.current_xp, s.xp_remaining) };
        println!("Smithing level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Ore: {}", state.player.get_item_count(ItemType::Ore));
        println!("Ingots: {}\n", state.player.get_item_count(ItemType::Ingots));
        println!("1) Smith");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if state.player.get_item_count(ItemType::Ore) == 0 {
                    println!("\nYou do not have enough of the required materials to make this item.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                if level >= MAX_SKILL_LEVEL {
                    let ore = state.player.get_item_count(ItemType::Ore);
                    state.player.remove_item(ItemType::Ore, ore);
                    state.player.add_item(ItemType::Ingots, ore);
                    println!("\nBecause you are a master at this skill, you instantly smelt all of your ore.");
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.remove_item(ItemType::Ore, 1);
                state.player.add_item(ItemType::Ingots, 1);
                if award_xp(&mut state.player.skills.smithing) {
                    let new_level = state.player.skills.smithing.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou smith some ore into an ingot.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}

fn thieving_menu(state: &mut GameState) {
    loop {
        clear_screen();
        let (level, current_xp, xp_remaining) = { let s = &state.player.skills.thieving; (s.level, s.current_xp, s.xp_remaining) };
        println!("Thieving level: {}", level);
        println!("Current EXP: {}", current_xp);
        println!("Remainder: {}", xp_remaining);
        println!("Gold: {}\n", state.player.gold);
        println!("1) Pickpocket");
        println!("2) Back\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if level >= MAX_SKILL_LEVEL {
                    let gold_to_add = 10000;
                    state.player.gold = (state.player.gold + gold_to_add).min(5_000_000);
                    println!("\nBecause you are a master at this skill, you instantly steal {} coins at a time.", gold_to_add);
                    crate::ui::wait_for_enter();
                    continue;
                }
                
                state.player.gold = (state.player.gold + 10).min(5_000_000);
                if award_xp(&mut state.player.skills.thieving) {
                    let new_level = state.player.skills.thieving.level;
                    if new_level >= MAX_SKILL_LEVEL {
                        println!("\nCongratulations, you have reached the maximum level in this skill.");
                    } else {
                        println!("\nCongratulations, you have now reached level {}.", new_level);
                    }
                    crate::ui::wait_for_enter();
                } else {
                    println!("\nYou steal some coins.");
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
            "2" => return,
            _ => continue,
        }
    }
}
