use crate::crafting::get_recipes_for_skill;
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

fn award_skill_action_xp(
    tui: &mut Tui,
    state: &mut GameState,
    skill: SkillKind,
) -> io::Result<()> {
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
                "Congratulations, your {} level is now {}{}.",
                skill.name(),
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
    Ok(())
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
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    (20, 5)
                } else {
                    (8, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, 25, modifier);
                if !check.succeeded {
                    format!(
                        "You fail to find a good cut ({}% success chance). No logs or XP gained; {} stamina and {} world turn(s) spent.",
                        check.success_chance, stamina_cost, turns
                    )
                } else {
                    let count = crate::mastery::gathering_yield(
                        state,
                        if level >= MAX_SKILL_LEVEL { 100 } else { 1 },
                    );
                    state.player.add_item(ItemType::Logs, count);
                    if level < MAX_SKILL_LEVEL {
                        award_skill_action_xp(tui, state, skill)?;
                        "You cut some logs.".to_string()
                    } else {
                        format!("Mastery: cut {count} logs. Stamina spent: {stamina_cost}; world turns spent: {turns}.")
                    }
                }
            }
            SkillKind::Cooking => {
                let level = skill.data(state).level;
                let raw_food = state.player.get_item_count(ItemType::RawFood);
                if raw_food == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough raw food to cook.",
                        Some(state),
                    )?;
                    continue;
                }
                let amount = if level >= MAX_SKILL_LEVEL {
                    batch_count(raw_food)
                } else {
                    1
                };
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    batch_budget(5, amount)
                } else {
                    (5, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, if level >= MAX_SKILL_LEVEL { 10 } else { 15 }, modifier);
                // Cooking always consumes the raw ingredients on an attempted
                // action; a failed batch is spoiled rather than refunded.
                let removed = state.player.remove_item(ItemType::RawFood, amount);
                debug_assert!(removed, "validated raw food must still exist");
                if !check.succeeded {
                    format!(
                        "The cooking attempt failed ({}% success chance). {amount} raw food spoiled; no food or XP gained. Stamina spent: {stamina_cost}; world turns spent: {turns}.",
                        check.success_chance
                    )
                } else {
                    state.player.add_item(ItemType::Food, amount);
                    if level < MAX_SKILL_LEVEL {
                        award_skill_action_xp(tui, state, skill)?;
                        "You cooked some food.".to_string()
                    } else {
                        format!("Mastery: cooked {amount} raw food. Stamina spent: {stamina_cost}; world turns spent: {turns}.")
                    }
                }
            }
            SkillKind::Fishing => {
                let level = skill.data(state).level;
                let bait = state.player.get_item_count(ItemType::Bait);
                if bait == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough bait to fish.",
                        Some(state),
                    )?;
                    continue;
                }
                let amount = if level >= MAX_SKILL_LEVEL {
                    batch_count(bait)
                } else {
                    1
                };
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    batch_budget(6, amount)
                } else {
                    (6, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, 25, modifier);
                let removed = state.player.remove_item(ItemType::Bait, amount);
                debug_assert!(removed, "validated bait must still exist");
                if !check.succeeded {
                    format!(
                        "You failed to catch anything ({}% success chance). {amount} bait was lost; no fish or XP gained. Stamina spent: {stamina_cost}; world turns spent: {turns}.",
                        check.success_chance
                    )
                } else {
                    state.player.add_item(ItemType::RawFood, amount);
                    if level < MAX_SKILL_LEVEL {
                        award_skill_action_xp(tui, state, skill)?;
                        "You caught a fish.".to_string()
                    } else {
                        format!("Mastery: caught {amount} fish. Stamina spent: {stamina_cost}; world turns spent: {turns}.")
                    }
                }
            }
            SkillKind::Mining => {
                let level = skill.data(state).level;
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    (20, 5)
                } else {
                    (10, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, 30, modifier);
                if !check.succeeded {
                    format!(
                        "You fail to extract usable ore ({}% success chance). No ore or XP gained; {} stamina and {} world turn(s) spent.",
                        check.success_chance, stamina_cost, turns
                    )
                } else {
                    let count = crate::mastery::gathering_yield(
                        state,
                        if level >= MAX_SKILL_LEVEL { 100 } else { 1 },
                    );
                    state.player.add_item(ItemType::Ore, count);
                    if level < MAX_SKILL_LEVEL {
                        award_skill_action_xp(tui, state, skill)?;
                        "You mined some ore.".to_string()
                    } else {
                        format!("Mastery: mined {count} ore. Stamina spent: {stamina_cost}; world turns spent: {turns}.")
                    }
                }
            }
            SkillKind::Smithing => {
                let level = skill.data(state).level;
                let ore = state.player.get_item_count(ItemType::Ore);
                if ore == 0 {
                    tui.show_message(
                        "Missing materials",
                        "You do not have enough ore to smith an ingot.",
                        Some(state),
                    )?;
                    continue;
                }
                let amount = batch_count(ore);
                if state
                    .player
                    .get_item_count(ItemType::ScrapMetal)
                    .checked_add(amount)
                    .is_none()
                {
                    tui.show_message(
                        "Inventory full",
                        "There is not enough capacity to store the Metal Scraps that a failed smelting action could produce.",
                        Some(state),
                    )?;
                    continue;
                }
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    batch_budget(8, amount)
                } else {
                    (8, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, 25, modifier);
                let removed = state.player.remove_item(ItemType::Ore, amount);
                debug_assert!(removed, "validated ore must still exist");
                if !check.succeeded {
                    state.player.add_item(ItemType::ScrapMetal, amount);
                    format!(
                        "Smelting failed ({}% success chance). {amount} ore became Metal Scraps. You can try recovering ingots from scraps at a lower success chance. Stamina spent: {stamina_cost}; world turns spent: {turns}.",
                        check.success_chance
                    )
                } else {
                    state.player.add_item(ItemType::Ingots, amount);
                    if level < MAX_SKILL_LEVEL {
                        award_skill_action_xp(tui, state, skill)?;
                        "You smithed an ingot.".to_string()
                    } else {
                        format!("Mastery: smelted {amount} ore into ingots. Stamina spent: {stamina_cost}; world turns spent: {turns}.")
                    }
                }
            }
            SkillKind::Thieving => {
                let level = skill.data(state).level;
                let (stamina_cost, turns) = if level >= MAX_SKILL_LEVEL {
                    (20, 5)
                } else {
                    (8, 1)
                };
                let Some(modifier) =
                    begin_skill_action(tui, state, stamina_cost, turns)?
                else {
                    continue;
                };
                let check = roll_skill_check(level, 25, modifier);
                if !check.succeeded {
                    format!(
                        "You get away empty-handed ({}% success chance). No gold, thread, or XP gained; {} stamina and {} world turn(s) spent.",
                        check.success_chance, stamina_cost, turns
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
                        award_skill_action_xp(tui, state, skill)?;
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
        assert_eq!(resolve_skill_check(50, 25, 0, 0).success_chance, 40);
        assert_eq!(resolve_skill_check(50, 30, 0, 0).success_chance, 35);
    }

    #[test]
    fn thieving_check_is_skill_sensitive_and_bounded() {
        let low = resolve_skill_check(1, 25, 0, 0);
        let high = resolve_skill_check(100, 25, 0, 0);
        assert!(high.success_chance > low.success_chance);
        assert!(low.success_chance >= crate::skill_checks::MIN_SUCCESS_CHANCE);
        assert!(high.success_chance <= crate::skill_checks::MAX_SUCCESS_CHANCE);
    }

    #[test]
    fn worst_roll_fails_without_awarding_success() {
        assert!(!resolve_skill_check(1, 25, 0, 99).succeeded);
        assert!(!resolve_skill_check(100, 25, 0, 99).succeeded);
        assert!(!resolve_skill_check(100, 25, 0, 99).succeeded);
    }

    #[test]
    fn best_roll_succeeds_for_high_skill_gathering() {
        assert!(resolve_skill_check(100, 25, 0, 0).succeeded);
        assert!(resolve_skill_check(100, 30, 0, 0).succeeded);
        assert!(resolve_skill_check(100, 25, 0, 0).succeeded);
    }

    #[test]
    fn fatigue_reduces_success_chance_after_stamina_is_spent() {
        let fresh = stamina_fatigue_modifier(90, 100);
        let tired = stamina_fatigue_modifier(40, 100);
        let exhausted = stamina_fatigue_modifier(10, 100);
        assert_eq!(fresh, 0);
        assert!(tired < fresh);
        assert!(exhausted < tired);
        assert!(resolve_skill_check(50, 25, tired, 0).success_chance
            < resolve_skill_check(50, 25, fresh, 0).success_chance);
    }
}
