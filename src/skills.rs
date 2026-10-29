use crate::crafting::{execute_recipe, get_recipes_for_skill};
use crate::data::{ItemType, MAX_GOLD, MAX_SKILL_LEVEL};
use crate::event::GameEvent;
use crate::skill_checks::SkillCheck;
use crate::state::{GameState, SkillData};
use crate::ui::Tui;
use rand::Rng;
use std::io;

/// A depleted stamina bar makes skill checks less reliable, even when the
/// player has the skill level for the job. The penalty is calculated after
/// paying the action's stamina cost, so long sessions progressively get riskier.
fn stamina_fatigue_modifier(stamina: u32, max_stamina: u32) -> i32 {
    let maximum = max_stamina.max(1);
    let percent = u64::from(stamina) * 100 / u64::from(maximum);
    match percent {
        75..=u64::MAX => 0,
        50..=74 => -5,
        25..=49 => -15,
        _ => -25,
    }
}

fn resolve_skill_check(
    skill_level: u32,
    difficulty: i32,
    modifier: i32,
    roll: u8,
) -> crate::skill_checks::SkillCheckResult {
    SkillCheck {
        skill_level,
        difficulty,
        modifier,
    }
    .resolve(roll)
    .expect("generated skill-check roll is within 0..=99")
}

fn roll_skill_check(
    skill_level: u32,
    difficulty: i32,
    modifier: i32,
) -> crate::skill_checks::SkillCheckResult {
    resolve_skill_check(
        skill_level,
        difficulty,
        modifier,
        rand::thread_rng().gen_range(0..100),
    )
}

/// Commit stamina and world time for an attempted action. Prerequisites must be
/// validated before this is called; a failed roll still pays this budget.
fn begin_skill_action(
    tui: &mut Tui,
    state: &mut GameState,
    stamina_cost: u32,
    turns: u32,
) -> io::Result<Option<i32>> {
    if state.player.stamina < stamina_cost {
        tui.show_message(
            "Too exhausted to continue",
            &format!(
                "This action needs {stamina_cost} stamina, but you have {}. Return to the world and rest before attempting it again.",
                state.player.stamina
            ),
            Some(state),
        )?;
        return Ok(None);
    }

    state.player.stamina -= stamina_cost;
    let modifier = stamina_fatigue_modifier(state.player.stamina, state.player.max_stamina);
    advance_skill_time(tui, state, turns)?;
    Ok(Some(modifier))
}

fn advance_skill_time(tui: &mut Tui, state: &mut GameState, turns: u32) -> io::Result<()> {
    let mut recovered = Vec::new();
    for _ in 0..turns {
        recovered.extend(state.advance_world_turn());
    }
    recovered.sort();
    recovered.dedup();
    if !recovered.is_empty() {
        let names = recovered
            .iter()
            .map(|id| crate::party::get_definition(*id).name)
            .collect::<Vec<_>>()
            .join(", ");
        tui.show_message(
            "Companion recovered",
            &format!("{names} recovered as world time advanced. They are available for explicit party selection."),
            Some(state),
        )?;
    }
    Ok(())
}

fn batch_count(available: u32) -> u32 {
    // Mastery can process a useful batch, but never an unbounded inventory in
    // one action. Each batch still has a stamina and world-time cost.
    available.min(50)
}

fn batch_budget(base_stamina: u32, amount: u32) -> (u32, u32) {
    let turns = amount.saturating_add(4) / 5;
    (base_stamina.saturating_add(turns), turns.max(1))
}

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

    /// Use the recipe engine for the existing cook/smelt transformations while
    /// retaining the original all-at-once mastery behaviour at level 100.
    fn primary_recipe(self) -> Option<&'static str> {
        match self {
            Self::Cooking => Some("cook_food"),
            Self::Smithing => Some("smith_ingot"),
            _ => None,
        }
    }
}

pub fn skill_plot(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let skills = [
        SkillKind::Woodcutting,
        SkillKind::Cooking,
        SkillKind::Fishing,
        SkillKind::Mining,
        SkillKind::Smithing,
        SkillKind::Thieving,
    ];
    loop {
        let labels: Vec<String> = skills
            .iter()
            .map(|skill| {
                let data = skill.data(state);
                format!(
                    "{} · level {} · {} XP / threshold {}",
                    skill.name(),
                    data.level,
                    data.current_xp,
                    data.xp_threshold
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!(
                "Skill Plot · total level {} · Stamina {}/{} · Turn {}",
                state.player.skills.total_level(),
                state.player.stamina,
                state.player.max_stamina,
                state.world_turn
            ),
            &options,
            Some(state),
        )?;
        if choice >= skills.len() {
            return Ok(());
        }
        skill_menu(tui, state, skills[choice])?;
    }
}

/// Attempt a recipe with a skill roll. Valid recipes spend stamina and one or
/// more world turns whether they succeed or fail. Failure consumes ingredients;
/// Smithing failures produce salvage scraps, while scrap recovery can consume
/// a scrap without returning one.
fn run_recipe(tui: &mut Tui, state: &mut GameState, recipe_id: &str) -> io::Result<String> {
    let (stamina_cost, turns) = match recipe_id {
        "cook_food" => (5, 1),
        "cook_gourmet_feast" => (12, 2),
        "fletch_bow" => (9, 2),
        "smith_ingot" => (8, 1),
        "smith_iron_sword" => (15, 3),
        "smith_scrap_ingot" => (7, 1),
        _ => (8, 1),
    };
    let modifier = stamina_fatigue_modifier(
        state.player.stamina.saturating_sub(stamina_cost),
        state.player.max_stamina,
    );
    let roll = rand::thread_rng().gen_range(0..100);
    match crate::crafting::attempt_recipe(state, recipe_id, roll, stamina_cost, modifier) {
        Ok(result) => {
            advance_skill_time(tui, state, turns)?;
            if result.succeeded {
                let mut message = format!(
                    "{} succeeded ({}% chance).\n\nInputs: {}\nOutputs: {}\nSkill XP gained: {}.\nStamina spent: {} · World turns spent: {}",
                    result.recipe.name,
                    result.success_chance,
                    result.recipe.input_description(),
                    result.recipe.output_description(),
                    result.xp_awarded,
                    stamina_cost,
                    turns
                );
                if let Some(new_level) = result.level_up {
                    message.push_str(&format!(
                        "\n\nYour {} level is now {}.",
                        result.recipe.skill_name, new_level
                    ));
                }
                tui.show_message("Crafting complete", &message, Some(state))?;
                Ok(format!("Crafted {}.", result.recipe.name))
            } else {
                let outcome = if result.scraps_awarded > 0 {
                    format!(
                        "You recovered {} Metal Scraps. Try 'Recover Ingot from Metal Scraps' for a lower success chance.",
                        result.scraps_awarded
                    )
                } else {
                    "The ingredients were consumed and nothing usable was produced.".to_string()
                };
                let message = format!(
                    "{} failed ({}% chance).\n\n{}\nStamina spent: {} · World turns spent: {}",
                    result.recipe.name, result.success_chance, outcome, stamina_cost, turns
                );
                tui.show_message("Crafting failed", &message, Some(state))?;
                Ok(format!("{} failed.", result.recipe.name))
            }
        }
        Err(error) => {
            let message = error.to_string();
            tui.show_message("Crafting unavailable", &message, Some(state))?;
            Ok(message)
        }
    }
}

fn skill_menu(tui: &mut Tui, state: &mut GameState, skill: SkillKind) -> io::Result<()> {
    let mut status = String::new();
    loop {
        let data = skill.data(state);
        let detail = format!(
            "{} · level {} · XP {} · remaining {} · Stamina {}/{} · Turn {}",
            skill.name(),
            data.level,
            data.current_xp,
            data.xp_remaining,
            state.player.stamina,
            state.player.max_stamina,
            state.world_turn
        );
        let action_label = format!(
            "{}{}",
            skill.action(),
            if status.is_empty() {
                String::new()
            } else {
                format!(" — {}", status)
            }
        );
        let recipes: Vec<_> = get_recipes_for_skill(skill.name())
            .into_iter()
            .filter(|recipe| skill.primary_recipe() != Some(recipe.id.as_str()))
            .collect();
        let mut options = vec![action_label];
        options.extend(
            recipes.iter().map(|recipe| {
                format!("Craft: {} (Req Lvl {})", recipe.name, recipe.required_level)
            }),
        );
        options.push("Back".into());
        let option_refs: Vec<&str> = options.iter().map(String::as_str).collect();
        let choice = tui.select_menu(&detail, &option_refs, Some(state))?;

        if choice >= options.len().saturating_sub(1) {
            return Ok(());
        }
        if choice > 0 {
            let Some(recipe) = recipes.get(choice - 1) else {
                return Ok(());
            };
            status = run_recipe(tui, state, &recipe.id)?;
            continue;
        }

        if skill.data(state).level < MAX_SKILL_LEVEL {
            if let Some(recipe_id) = skill.primary_recipe() {
                status = run_recipe(tui, state, recipe_id)?;
                continue;
            }
        }

        status = match skill {
            SkillKind::Woodcutting => {
                let level = skill.data(state).level;
                let difficulty = 25;
                let check =
                    gathering_check(level, difficulty, rand::thread_rng().gen_range(0..100));
                if !check.succeeded {
                    format!(
                        "You fail to find a good cut and gain no logs or XP ({}% chance).",
                        check.success_chance
                    )
                } else if level >= MAX_SKILL_LEVEL {
                    let count = crate::mastery::gathering_yield(state, 100);
                    state.player.add_item(ItemType::Logs, count);
                    format!("Mastery: instantly cut {count} logs.")
                } else {
                    let count = crate::mastery::gathering_yield(state, 1);
                    state.player.add_item(ItemType::Logs, count);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        crate::quests::notify_event(
                            state,
                            &GameEvent::SkillLeveledUp {
                                skill_name: skill.name().to_string(),
                                new_level,
                            },
                        );
                        tui.show_message(
                            "Skill level up",
                            &format!(
                                "Congratulations, your Woodcutting level is now {}{}.",
                                new_level,
                                if new_level >= MAX_SKILL_LEVEL {
                                    " (maximum level)"
                                } else {
                                    ""
                                }
                            ),
                            Some(state),
                        )?;
                    }
                    "You cut some logs.".to_string()
                }
            }
            SkillKind::Cooking => {
                let raw_food = state.player.get_item_count(ItemType::RawFood);
                if raw_food == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough raw food to cook.",
                        Some(state),
                    )?;
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
                        crate::quests::notify_event(
                            state,
                            &GameEvent::SkillLeveledUp {
                                skill_name: skill.name().to_string(),
                                new_level,
                            },
                        );
                        tui.show_message(
                            "Skill level up",
                            &format!(
                                "Congratulations, your Cooking level is now {}{}.",
                                new_level,
                                if new_level >= MAX_SKILL_LEVEL {
                                    " (maximum level)"
                                } else {
                                    ""
                                }
                            ),
                            Some(state),
                        )?;
                    }
                    "You cooked some food.".to_string()
                }
            }
            SkillKind::Fishing => {
                let bait = state.player.get_item_count(ItemType::Bait);
                if bait == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough bait to fish.",
                        Some(state),
                    )?;
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
                        crate::quests::notify_event(
                            state,
                            &GameEvent::SkillLeveledUp {
                                skill_name: skill.name().to_string(),
                                new_level,
                            },
                        );
                        tui.show_message(
                            "Skill level up",
                            &format!(
                                "Congratulations, your Fishing level is now {}{}.",
                                new_level,
                                if new_level >= MAX_SKILL_LEVEL {
                                    " (maximum level)"
                                } else {
                                    ""
                                }
                            ),
                            Some(state),
                        )?;
                    }
                    "You caught a fish.".to_string()
                }
            }
            SkillKind::Mining => {
                let level = skill.data(state).level;
                let difficulty = 30;
                let check =
                    gathering_check(level, difficulty, rand::thread_rng().gen_range(0..100));
                if !check.succeeded {
                    format!(
                        "You fail to extract usable ore and gain no ore or XP ({}% chance).",
                        check.success_chance
                    )
                } else if level >= MAX_SKILL_LEVEL {
                    let count = crate::mastery::gathering_yield(state, 100);
                    state.player.add_item(ItemType::Ore, count);
                    format!("Mastery: instantly mined {count} ore.")
                } else {
                    let count = crate::mastery::gathering_yield(state, 1);
                    state.player.add_item(ItemType::Ore, count);
                    if award_xp(skill.data_mut(state)) {
                        let new_level = skill.data(state).level;
                        crate::quests::notify_event(
                            state,
                            &GameEvent::SkillLeveledUp {
                                skill_name: skill.name().to_string(),
                                new_level,
                            },
                        );
                        tui.show_message(
                            "Skill level up",
                            &format!(
                                "Congratulations, your Mining level is now {}{}.",
                                new_level,
                                if new_level >= MAX_SKILL_LEVEL {
                                    " (maximum level)"
                                } else {
                                    ""
                                }
                            ),
                            Some(state),
                        )?;
                    }
                    "You mined some ore.".to_string()
                }
            }
            SkillKind::Smithing => {
                let ore = state.player.get_item_count(ItemType::Ore);
                if ore == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough ore to smith an ingot.",
                        Some(state),
                    )?;
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
                        crate::quests::notify_event(
                            state,
                            &GameEvent::SkillLeveledUp {
                                skill_name: skill.name().to_string(),
                                new_level,
                            },
                        );
                        tui.show_message(
                            "Skill level up",
                            &format!(
                                "Congratulations, your Smithing level is now {}{}.",
                                new_level,
                                if new_level >= MAX_SKILL_LEVEL {
                                    " (maximum level)"
                                } else {
                                    ""
                                }
                            ),
                            Some(state),
                        )?;
                    }
                    "You smithed an ingot.".to_string()
                }
            }
            SkillKind::Thieving => {
                let level = skill.data(state).level;
                let check = thieving_check(level, rand::thread_rng().gen_range(0..100));
                if !check.succeeded {
                    // Failure is a wasted attempt only: no gold, thread, or XP.
                    format!(
                        "You get away empty-handed ({}% success chance).",
                        check.success_chance
                    )
                } else {
                    let found_thread = rand::thread_rng().gen_bool(0.4);
                    if found_thread {
                        state.player.add_item(ItemType::Thread, 1);
                    }

                    if level >= MAX_SKILL_LEVEL {
                        state.player.gold = state.player.gold.saturating_add(10_000).min(MAX_GOLD);
                        if found_thread {
                            "Mastery: stole 10,000 coins and found a thread.".to_string()
                        } else {
                            "Mastery: stole 10,000 coins.".to_string()
                        }
                    } else {
                        state.player.gold = state.player.gold.saturating_add(10).min(MAX_GOLD);
                        if award_xp(skill.data_mut(state)) {
                            let new_level = skill.data(state).level;
                            crate::quests::notify_event(
                                state,
                                &GameEvent::SkillLeveledUp {
                                    skill_name: skill.name().to_string(),
                                    new_level,
                                },
                            );
                            tui.show_message(
                                "Skill level up",
                                &format!(
                                    "Congratulations, your Thieving level is now {}{}.",
                                    new_level,
                                    if new_level >= MAX_SKILL_LEVEL {
                                        " (maximum level)"
                                    } else {
                                        ""
                                    }
                                ),
                                Some(state),
                            )?;
                        }
                        if found_thread {
                            "You stole some coins and found a thread.".to_string()
                        } else {
                            "You stole some coins.".to_string()
                        }
                    }
                }
            }
        };
    }
}

#[cfg(test)]
mod skill_check_integration_tests {
    use super::*;

    #[test]
    fn gathering_checks_use_distinct_documented_difficulties() {
        assert_eq!(gathering_check(50, 25, 0).success_chance, 30);
        assert_eq!(gathering_check(50, 30, 0).success_chance, 25);
    }

    #[test]
    fn thieving_check_is_skill_sensitive_and_bounded() {
        let low = thieving_check(1, 0);
        let high = thieving_check(100, 0);
        assert!(high.success_chance > low.success_chance);
        assert!(low.success_chance >= crate::skill_checks::MIN_SUCCESS_CHANCE);
        assert!(high.success_chance <= crate::skill_checks::MAX_SUCCESS_CHANCE);
    }

    #[test]
    fn worst_roll_fails_without_awarding_success() {
        assert!(!gathering_check(1, 25, 99).succeeded);
        assert!(!gathering_check(100, 25, 99).succeeded);
        assert!(!thieving_check(100, 99).succeeded);
    }

    #[test]
    fn best_roll_succeeds_for_high_skill_gathering() {
        assert!(gathering_check(100, 25, 0).succeeded);
        assert!(gathering_check(100, 30, 0).succeeded);
        assert!(thieving_check(100, 0).succeeded);
    }
}
