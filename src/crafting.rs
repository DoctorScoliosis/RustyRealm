use crate::data::{ItemType, WeaponTier, MAX_SKILL_LEVEL};
use crate::event::GameEvent;
use crate::quests;
use crate::state::{GameState, SkillData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftingError {
    RecipeNotFound(String),
    UnknownSkill(String),
    InsufficientSkillLevel {
        skill: String,
        required: u32,
        current: u32,
    },
    InsufficientIngredients {
        item: ItemType,
        required: u32,
        available: u32,
    },
    OutputOverflow(ItemType),
    InvalidRecipeDefinition(String),
    InventoryCapacity,
}

impl fmt::Display for CraftingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecipeNotFound(id) => write!(f, "Recipe not found: {id}"),
            Self::UnknownSkill(skill) => write!(f, "Recipe references an unknown skill: {skill}"),
            Self::InsufficientSkillLevel {
                skill,
                required,
                current,
            } => write!(
                f,
                "Insufficient skill level for {skill}: required {required}, current {current}"
            ),
            Self::InsufficientIngredients {
                item,
                required,
                available,
            } => write!(
                f,
                "Insufficient ingredients: need {required} x {}, have {available}",
                item.name()
            ),
            Self::OutputOverflow(item) => {
                write!(
                    f,
                    "Crafting would exceed the inventory limit for {}.",
                    item.name()
                )
            }
            Self::InvalidRecipeDefinition(id) => {
                write!(f, "Recipe {id} has an invalid or overflowing quantity.")
            }
            Self::InventoryCapacity => {
                write!(
                    f,
                    "There is not enough memory available to store the crafted weapon."
                )
            }
        }
    }
}

impl std::error::Error for CraftingError {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RecipeOutput {
    Item { item_type: ItemType, count: u32 },
    Weapon { tier: WeaponTier, count: u32 },
}

impl RecipeOutput {
    pub fn description(&self) -> String {
        match self {
            Self::Item { item_type, count } => format!("{count} x {}", item_type.name()),
            Self::Weapon { tier, count } => format!("{count} x {}", tier.name()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub skill_name: String,
    pub required_level: u32,
    pub inputs: Vec<(ItemType, u32)>,
    pub outputs: Vec<RecipeOutput>,
    pub xp_reward: u32,
}

impl Recipe {
    pub fn input_description(&self) -> String {
        self.inputs
            .iter()
            .map(|(item, count)| format!("{count} x {}", item.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn output_description(&self) -> String {
        self.outputs
            .iter()
            .map(RecipeOutput::description)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The recipes use RustyRealm's typed inventory and weapon tiers. An iron sword
/// becomes an actual existing WeaponTier::Iron in the player's weapon inventory;
/// bows and food remain ordinary inventory items.
pub fn all_recipes() -> Vec<Recipe> {
    vec![
        Recipe {
            id: "smith_ingot".into(),
            name: "Smelt Iron Ingot".into(),
            skill_name: "Smithing".into(),
            required_level: 1,
            inputs: vec![(ItemType::Ore, 1)],
            outputs: vec![RecipeOutput::Item {
                item_type: ItemType::Ingots,
                count: 1,
            }],
            xp_reward: 33,
        },
        Recipe {
            id: "smith_iron_sword".into(),
            name: "Forge Iron Sword".into(),
            skill_name: "Smithing".into(),
            required_level: 5,
            inputs: vec![(ItemType::Ingots, 3)],
            outputs: vec![RecipeOutput::Weapon {
                tier: WeaponTier::Iron,
                count: 1,
            }],
            xp_reward: 75,
        },
        Recipe {
            id: "fletch_bow".into(),
            name: "Fletch Wooden Bow".into(),
            skill_name: "Woodcutting".into(),
            required_level: 5,
            inputs: vec![(ItemType::Logs, 2), (ItemType::Thread, 1)],
            outputs: vec![RecipeOutput::Item {
                item_type: ItemType::Bows,
                count: 1,
            }],
            xp_reward: 40,
        },
        Recipe {
            id: "cook_food".into(),
            name: "Cook Raw Fish".into(),
            skill_name: "Cooking".into(),
            required_level: 1,
            inputs: vec![(ItemType::RawFood, 1)],
            outputs: vec![RecipeOutput::Item {
                item_type: ItemType::Food,
                count: 1,
            }],
            xp_reward: 41,
        },
        Recipe {
            id: "cook_gourmet_feast".into(),
            name: "Prepare Gourmet Feast".into(),
            skill_name: "Cooking".into(),
            required_level: 10,
            inputs: vec![(ItemType::RawFood, 2), (ItemType::Seeds, 1)],
            outputs: vec![RecipeOutput::Item {
                item_type: ItemType::GourmetFeast,
                count: 1,
            }],
            xp_reward: 85,
        },
    ]
}

pub fn get_recipes_for_skill(skill_name: &str) -> Vec<Recipe> {
    all_recipes()
        .into_iter()
        .filter(|recipe| recipe.skill_name.eq_ignore_ascii_case(skill_name))
        .collect()
}

fn get_skill_data<'a>(state: &'a GameState, skill_name: &str) -> Option<&'a SkillData> {
    match skill_name.to_ascii_lowercase().as_str() {
        "woodcutting" => Some(&state.player.skills.woodcutting),
        "cooking" => Some(&state.player.skills.cooking),
        "fishing" => Some(&state.player.skills.fishing),
        "mining" => Some(&state.player.skills.mining),
        "smithing" => Some(&state.player.skills.smithing),
        "thieving" => Some(&state.player.skills.thieving),
        _ => None,
    }
}

fn get_skill_data_mut<'a>(state: &'a mut GameState, skill_name: &str) -> Option<&'a mut SkillData> {
    match skill_name.to_ascii_lowercase().as_str() {
        "woodcutting" => Some(&mut state.player.skills.woodcutting),
        "cooking" => Some(&mut state.player.skills.cooking),
        "fishing" => Some(&mut state.player.skills.fishing),
        "mining" => Some(&mut state.player.skills.mining),
        "smithing" => Some(&mut state.player.skills.smithing),
        "thieving" => Some(&mut state.player.skills.thieving),
        _ => None,
    }
}

/// Apply XP using RustyRealm's existing increasing skill thresholds. A recipe
/// grants at most one level per action, matching the existing gathering actions.
fn award_skill_xp(skill: &mut SkillData, amount: u32) -> Option<u32> {
    if amount == 0 || skill.level >= MAX_SKILL_LEVEL {
        return None;
    }

    skill.current_xp = skill.current_xp.saturating_add(amount);
    skill.xp_remaining = skill.xp_remaining.saturating_sub(amount);
    if skill.xp_remaining != 0 {
        return None;
    }

    skill.level = skill.level.saturating_add(1).min(MAX_SKILL_LEVEL);
    if skill.level >= MAX_SKILL_LEVEL {
        skill.xp_remaining = 0;
    } else {
        skill.xp_threshold = skill.xp_threshold.saturating_add(100);
        skill.xp_remaining = skill.xp_threshold;
    }
    Some(skill.level)
}

pub fn execute_recipe(
    state: &mut GameState,
    recipe_id: &str,
) -> Result<CraftingResult, CraftingError> {
    let recipe = all_recipes()
        .into_iter()
        .find(|recipe| recipe.id == recipe_id)
        .ok_or_else(|| CraftingError::RecipeNotFound(recipe_id.to_string()))?;

    let current_level = get_skill_data(state, &recipe.skill_name)
        .map(|skill| skill.level)
        .ok_or_else(|| CraftingError::UnknownSkill(recipe.skill_name.clone()))?;
    if current_level < recipe.required_level {
        return Err(CraftingError::InsufficientSkillLevel {
            skill: recipe.skill_name.clone(),
            required: recipe.required_level,
            current: current_level,
        });
    }

    let master_artisan = crate::mastery::has_perk(state, crate::data::MasteryPerkId::MasterArtisan);
    // Aggregate requirements first so duplicate inputs cannot bypass validation.
    let mut input_totals: HashMap<ItemType, u32> = HashMap::new();
    for (item, count) in &recipe.inputs {
        if *count == 0 {
            return Err(CraftingError::InvalidRecipeDefinition(recipe.id.clone()));
        }
        let total = input_totals.entry(*item).or_insert(0);
        *total = total
            .checked_add(*count)
            .ok_or_else(|| CraftingError::InvalidRecipeDefinition(recipe.id.clone()))?;
    }
    if master_artisan {
        for required in input_totals.values_mut() {
            *required = crate::mastery::discounted_ingredient_count(*required);
        }
    }
    for (item, required) in &input_totals {
        let available = state.player.get_item_count(*item);
        if available < *required {
            return Err(CraftingError::InsufficientIngredients {
                item: *item,
                required: *required,
                available,
            });
        }
    }

    // Validate every item output before consuming anything. This prevents a
    // saturated item count from silently swallowing a crafted product.
    let mut output_totals: HashMap<ItemType, u32> = HashMap::new();
    let mut weapon_output_count = 0usize;
    for output in &recipe.outputs {
        match output {
            RecipeOutput::Item { item_type, count } => {
                if *count == 0 {
                    return Err(CraftingError::InvalidRecipeDefinition(recipe.id.clone()));
                }
                let total = output_totals.entry(*item_type).or_insert(0);
                *total = total
                    .checked_add(*count)
                    .ok_or_else(|| CraftingError::InvalidRecipeDefinition(recipe.id.clone()))?;
            }
            RecipeOutput::Weapon { count, .. } => {
                if *count == 0 {
                    return Err(CraftingError::InvalidRecipeDefinition(recipe.id.clone()));
                }
                weapon_output_count =
                    weapon_output_count
                        .checked_add(usize::try_from(*count).map_err(|_| {
                            CraftingError::InvalidRecipeDefinition(recipe.id.clone())
                        })?)
                        .ok_or_else(|| CraftingError::InvalidRecipeDefinition(recipe.id.clone()))?;
            }
        }
    }
    for (item, count) in &output_totals {
        if state
            .player
            .get_item_count(*item)
            .checked_add(*count)
            .is_none()
        {
            return Err(CraftingError::OutputOverflow(*item));
        }
    }
    // Reserve the weapon slots up front so an allocation failure leaves the
    // ingredients, items, and skill XP unchanged.
    state
        .player
        .inventory_weapons
        .try_reserve(weapon_output_count)
        .map_err(|_| CraftingError::InventoryCapacity)?;

    // All possible validation failures have been handled; commit the recipe.
    for (item, count) in input_totals {
        let removed = state.player.remove_item(item, count);
        debug_assert!(removed, "validated recipe ingredients must still exist");
    }
    for output in &recipe.outputs {
        match output {
            RecipeOutput::Item { item_type, count } => {
                state.player.add_item(*item_type, *count);
            }
            RecipeOutput::Weapon { tier, count } => {
                for _ in 0..*count {
                    state.player.inventory_weapons.push(*tier);
                }
            }
        }
    }

    let xp_awarded = if current_level >= MAX_SKILL_LEVEL {
        0
    } else {
        recipe.xp_reward
    };
    let level_up = if xp_awarded == 0 {
        None
    } else {
        get_skill_data_mut(state, &recipe.skill_name)
            .and_then(|skill| award_skill_xp(skill, xp_awarded))
    };

    if let Some(new_level) = level_up {
        quests::notify_event(
            state,
            &GameEvent::SkillLeveledUp {
                skill_name: recipe.skill_name.clone(),
                new_level,
            },
        );
    }

    Ok(CraftingResult {
        recipe,
        xp_awarded,
        level_up,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingResult {
    pub recipe: Recipe,
    pub xp_awarded: u32,
    pub level_up: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Player;

    fn state() -> GameState {
        GameState::new(Player::new("CraftingHero".into(), String::new()))
    }

    #[test]
    fn smelting_consumes_ore_and_grants_ingot_and_skill_xp() {
        let mut state = state();
        state.player.add_item(ItemType::Ore, 3);

        let result = execute_recipe(&mut state, "smith_ingot").unwrap();

        assert_eq!(state.player.get_item_count(ItemType::Ore), 2);
        assert_eq!(state.player.get_item_count(ItemType::Ingots), 1);
        assert_eq!(result.xp_awarded, 33);
        assert_eq!(state.player.skills.smithing.current_xp, 33);
        assert_eq!(state.player.skills.smithing.xp_remaining, 67);
    }

    #[test]
    fn insufficient_ingredients_are_atomic() {
        let mut state = state();
        state.player.skills.smithing.level = 5;
        state.player.add_item(ItemType::Ingots, 1);
        let xp_before = state.player.skills.smithing.current_xp;
        let remaining_before = state.player.skills.smithing.xp_remaining;

        assert!(matches!(
            execute_recipe(&mut state, "smith_iron_sword"),
            Err(CraftingError::InsufficientIngredients { .. })
        ));
        assert_eq!(state.player.get_item_count(ItemType::Ingots), 1);
        assert!(state.player.inventory_weapons.is_empty());
        assert_eq!(state.player.skills.smithing.current_xp, xp_before);
        assert_eq!(state.player.skills.smithing.xp_remaining, remaining_before);
    }

    #[test]
    fn insufficient_skill_level_does_not_consume_ingredients_or_xp() {
        let mut state = state();
        state.player.add_item(ItemType::Ingots, 5);
        let xp_before = state.player.skills.smithing.current_xp;
        let remaining_before = state.player.skills.smithing.xp_remaining;

        assert!(matches!(
            execute_recipe(&mut state, "smith_iron_sword"),
            Err(CraftingError::InsufficientSkillLevel { .. })
        ));
        assert_eq!(state.player.get_item_count(ItemType::Ingots), 5);
        assert_eq!(state.player.skills.smithing.current_xp, xp_before);
        assert_eq!(state.player.skills.smithing.xp_remaining, remaining_before);
    }

    #[test]
    fn crafted_iron_sword_uses_the_existing_weapon_inventory() {
        let mut state = state();
        state.player.skills.smithing.level = 5;
        state.player.add_item(ItemType::Ingots, 3);

        let result = execute_recipe(&mut state, "smith_iron_sword").unwrap();

        assert_eq!(state.player.get_item_count(ItemType::Ingots), 0);
        assert_eq!(
            state
                .player
                .inventory_weapons
                .iter()
                .filter(|&&tier| tier == WeaponTier::Iron)
                .count(),
            1
        );
        assert_eq!(result.xp_awarded, 75);
    }

    #[test]
    fn output_overflow_does_not_consume_inputs() {
        let mut state = state();
        state.player.add_item(ItemType::Ore, 1);
        state.player.items.insert(ItemType::Ingots, u32::MAX);

        assert_eq!(
            execute_recipe(&mut state, "smith_ingot"),
            Err(CraftingError::OutputOverflow(ItemType::Ingots))
        );
        assert_eq!(state.player.get_item_count(ItemType::Ore), 1);
        assert_eq!(state.player.get_item_count(ItemType::Ingots), u32::MAX);
        assert_eq!(state.player.skills.smithing.current_xp, 0);
    }

    #[test]
    fn recipes_respect_the_existing_maximum_skill_level_and_xp_thresholds() {
        let mut state = state();
        state.player.skills.smithing.level = 99;
        state.player.skills.smithing.xp_remaining = 1;
        state.player.skills.smithing.xp_threshold = 9_900;
        state.player.add_item(ItemType::Ore, 3);

        execute_recipe(&mut state, "smith_ingot").unwrap();
        assert_eq!(state.player.skills.smithing.level, MAX_SKILL_LEVEL);
        assert_eq!(state.player.skills.smithing.xp_remaining, 0);
        let xp_at_cap = state.player.skills.smithing.current_xp;

        let result = execute_recipe(&mut state, "smith_ingot").unwrap();
        assert_eq!(result.xp_awarded, 0);
        assert_eq!(state.player.skills.smithing.current_xp, xp_at_cap);
        assert_eq!(state.player.skills.smithing.level, MAX_SKILL_LEVEL);
    }

    #[test]
    fn cooking_transforms_raw_food_using_the_recipe_engine() {
        let mut state = state();
        state.player.add_item(ItemType::RawFood, 1);

        let result = execute_recipe(&mut state, "cook_food").unwrap();

        assert_eq!(state.player.get_item_count(ItemType::RawFood), 0);
        assert_eq!(state.player.get_item_count(ItemType::Food), 1);
        assert_eq!(result.xp_awarded, 41);
    }

    #[test]
    fn fletching_consumes_logs_and_thread_and_produces_a_bow() {
        let mut state = state();
        state.player.skills.woodcutting.level = 5;
        state.player.add_item(ItemType::Logs, 2);
        state.player.add_item(ItemType::Thread, 1);

        let result = execute_recipe(&mut state, "fletch_bow").unwrap();

        assert_eq!(state.player.get_item_count(ItemType::Logs), 0);
        assert_eq!(state.player.get_item_count(ItemType::Thread), 0);
        assert_eq!(state.player.get_item_count(ItemType::Bows), 1);
        assert_eq!(result.xp_awarded, 40);
    }

    #[test]
    fn missing_second_ingredient_preserves_all_inputs_and_xp() {
        let mut state = state();
        state.player.skills.woodcutting.level = 5;
        state.player.add_item(ItemType::Logs, 2);
        let xp_before = state.player.skills.woodcutting.current_xp;

        assert!(matches!(
            execute_recipe(&mut state, "fletch_bow"),
            Err(CraftingError::InsufficientIngredients {
                item: ItemType::Thread,
                ..
            })
        ));
        assert_eq!(state.player.get_item_count(ItemType::Logs), 2);
        assert_eq!(state.player.get_item_count(ItemType::Thread), 0);
        assert_eq!(state.player.get_item_count(ItemType::Bows), 0);
        assert_eq!(state.player.skills.woodcutting.current_xp, xp_before);
    }

    #[test]
    fn gourmet_feast_consumes_raw_food_and_seeds() {
        let mut state = state();
        state.player.skills.cooking.level = 10;
        state.player.add_item(ItemType::RawFood, 2);
        state.player.add_item(ItemType::Seeds, 1);

        let result = execute_recipe(&mut state, "cook_gourmet_feast").unwrap();

        assert_eq!(state.player.get_item_count(ItemType::RawFood), 0);
        assert_eq!(state.player.get_item_count(ItemType::Seeds), 0);
        assert_eq!(state.player.get_item_count(ItemType::GourmetFeast), 1);
        assert_eq!(result.xp_awarded, 85);
    }

    #[test]
    fn recipe_xp_uses_the_existing_threshold_and_emits_a_level_up() {
        use crate::quests::{Quest, QuestObjective, QuestReward};

        let mut state = state();
        state.active_quests.push(Quest {
            id: "smithing_to_level_two".into(),
            title: "Practise Smithing".into(),
            description: "Reach Smithing level 2.".into(),
            objective: QuestObjective::SkillLevel {
                skill_name: "Smithing".into(),
                required_level: 2,
            },
            progress: 1,
            target: 2,
            is_completed: false,
            is_claimed: false,
            reward: QuestReward::Gold(1),
            is_daily: false,
        });
        state.player.skills.smithing.xp_remaining = 10;
        state.player.add_item(ItemType::Ore, 1);

        let result = execute_recipe(&mut state, "smith_ingot").unwrap();

        assert_eq!(result.level_up, Some(2));
        assert_eq!(state.player.skills.smithing.level, 2);
        assert_eq!(state.player.skills.smithing.xp_remaining, 200);
        let quest = state
            .active_quests
            .iter()
            .find(|quest| quest.id == "smithing_to_level_two")
            .unwrap();
        assert!(quest.is_completed);
        assert_eq!(quest.progress, 2);
    }

    #[test]
    fn recipes_are_discovered_case_insensitively_by_skill() {
        assert_eq!(get_recipes_for_skill("cOoKiNg").len(), 2);
        assert_eq!(get_recipes_for_skill("woodcutting").len(), 1);
    }
    #[test]
    fn master_artisan_reduces_iron_sword_ingredient_requirement() {
        let mut state = state();
        state.player.skills.smithing.level = 5;
        state.player.add_item(ItemType::Ingots, 2);
        state.mastery_perks.push(crate::data::MasteryPerkId::MasterArtisan);
        execute_recipe(&mut state, "smith_iron_sword").unwrap();
        assert_eq!(state.player.get_item_count(ItemType::Ingots), 0);
        assert!(state.player.inventory_weapons.contains(&WeaponTier::Iron));
    }

}
