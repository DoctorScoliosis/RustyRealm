mod achievements;
mod adventure;
mod balance_simulation;
mod bank;
mod combat;
mod combat_engine;
mod companion_recovery;
mod crafting;
mod data;
mod economy;
mod event;
mod inventory;
mod mastery;
mod party;
mod party_battle;
mod party_combat;
mod quests;
mod save;
mod shops;
mod skill_checks;
mod skills;
mod state;
mod status_combat;
mod stronghold;
mod ui;
// Some world queries are reserved for later gameplay slices; remove this
// allowance as the remaining APIs gain consumers.
#[allow(dead_code)]
mod world;

use crate::data::ItemType;
use crate::state::GameState;
use crate::ui::Tui;
use crate::world::{build_initial_world, LocationCategory, LocationId, World};
use std::io;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    loop {
        let choice = tui.select_menu("Rusty Realm", &["Login", "Create Account", "Exit"], None)?;
        match choice {
            0 => {
                if let Some(mut state) = login(&mut tui)? {
                    main_menu(&mut tui, &mut state)?;
                }
            }
            1 => {
                if let Some(mut state) = create_account(&mut tui)? {
                    main_menu(&mut tui, &mut state)?;
                }
            }
            _ => break,
        }
    }
    // Tui's RAII guard restores raw mode, the cursor, and the previous screen.
    Ok(())
}

fn login(tui: &mut Tui) -> io::Result<Option<GameState>> {
    let username = tui.prompt_input("Username", false, None)?;
    if username.is_empty() {
        return Ok(None);
    }

    let password = tui.prompt_input("Password", true, None)?;
    if password.is_empty() {
        tui.show_message("Login cancelled", "A password is required.", None)?;
        return Ok(None);
    }

    match crate::save::load_game(&username, &password) {
        Ok(state) => {
            tui.show_message(
                "Welcome back",
                &format!("Welcome back, {}!", state.player.username),
                Some(&state),
            )?;
            Ok(Some(state))
        }
        Err(crate::save::SaveError::PasswordMigrationRequired) => {
            tui.show_message(
                "Legacy save migration",
                "This save predates password-hash persistence. The old password cannot be verified because no hash was stored.\n\nYour entered password has NOT been accepted as proof of ownership. Because this is a local save, you may explicitly choose a new password to migrate it. Cancelling leaves the save unchanged.",
                None,
            )?;
            let choice = tui.prompt_input(
                "Type YES to set a new password and migrate this save",
                false,
                None,
            )?;
            if choice != "YES" {
                return Ok(None);
            }

            let new_password = tui.prompt_input("New password", true, None)?;
            if new_password.is_empty() {
                tui.show_message(
                    "Migration cancelled",
                    "Password cannot be empty. The save has not been changed.",
                    None,
                )?;
                return Ok(None);
            }
            let confirmation = tui.prompt_input("Confirm new password", true, None)?;
            if new_password != confirmation {
                tui.show_message(
                    "Migration cancelled",
                    "Passwords do not match. The save has not been changed.",
                    None,
                )?;
                return Ok(None);
            }

            match crate::save::migrate_legacy_save(&username, &new_password) {
                Ok(state) => {
                    tui.show_message(
                        "Migration complete",
                        "Legacy save migrated and password protection updated.",
                        Some(&state),
                    )?;
                    Ok(Some(state))
                }
                Err(error) => {
                    tui.show_message("Migration failed", &error.to_string(), None)?;
                    Ok(None)
                }
            }
        }
        Err(error) => {
            tui.show_message("Login failed", &error.to_string(), None)?;
            Ok(None)
        }
    }
}

fn create_account(tui: &mut Tui) -> io::Result<Option<GameState>> {
    let username = tui.prompt_input("Choose username (no spaces)", false, None)?;
    if username.is_empty() || username.contains(' ') {
        tui.show_message(
            "Invalid username",
            "A username is required, and spaces are not allowed.",
            None,
        )?;
        return Ok(None);
    }

    let password = tui.prompt_input("Choose password", true, None)?;
    if password.is_empty() {
        tui.show_message("Invalid password", "Password cannot be empty.", None)?;
        return Ok(None);
    }

    match crate::save::create_account(&username, &password) {
        Ok(state) => {
            tui.show_message(
                "Account created",
                "Account created successfully!",
                Some(&state),
            )?;
            Ok(Some(state))
        }
        Err(error) => {
            tui.show_message("Account creation failed", &error.to_string(), None)?;
            Ok(None)
        }
    }
}

fn main_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let world = build_initial_world().map_err(|error| io::Error::other(error.to_string()))?;
    crate::adventure::ensure_region_quest(state);
    // The location graph and recovery records are persisted separately; a
    // companion with zero HP is normalised here so stale party IDs cannot let
    // an older save field reintroduce them into combat.
    for companion in &mut state.companions {
        if companion.current_hp == 0
            && !matches!(
                companion.condition,
                crate::party::CompanionCondition::KnockedOut
                    | crate::party::CompanionCondition::Recovering
            )
        {
            companion.condition = crate::party::CompanionCondition::KnockedOut;
        }
    }
    let knocked_out: Vec<_> = state
        .companions
        .iter()
        .filter(|companion| {
            companion.current_hp == 0 && state.companion_recovery.record(companion.id).is_none()
        })
        .map(|companion| companion.id)
        .collect();
    let origin = state.current_location.clone();
    let mut new_records = Vec::new();
    for id in knocked_out {
        if let Some(companion) = state
            .companions
            .iter_mut()
            .find(|companion| companion.id == id)
        {
            if let Ok(true) = state
                .companion_recovery
                .record_knockout(companion, &world, &origin)
            {
                new_records.push(id);
            }
        }
    }
    for id in new_records {
        let _ = state.party.remove_member(id);
    }
    let options = [
        "Travel / Explore Region",
        "Interact with Current Location",
        "Expedition at Current Location",
        "Wander Gielinor",
        "Weapon Shop",
        "Armour Shop",
        "Legend's Store",
        "Trading Post (General Store)",
        "Max's Shop",
        "Smuggler",
        "The Stronghold",
        "Skill Plot",
        "RealmBank",
        "Quest Hall",
        "Your Inventory",
        "Mastery & Achievements",
        "Companions & Party",
        "Companion Recovery",
        "Rest / Recover",
        "Settings",
        "Log Out",
    ];

    loop {
        let newly_unlocked = crate::achievements::evaluate_achievements(state);
        if !newly_unlocked.is_empty() {
            let notice = newly_unlocked
                .iter()
                .map(|a| format!("{} — {}", a.title, a.description))
                .collect::<Vec<_>>()
                .join("\n\n");
            tui.show_message("Achievement unlocked", &notice, Some(state))?;
        }
        crate::quests::check_and_refresh_quests(state);
        if let Err(error) = crate::save::save_game(state) {
            tui.show_message(
                "Save warning",
                &format!("Failed to save the game: {}", error),
                Some(state),
            )?;
        }

        let current_location_name = world
            .get_location(&state.current_location)
            .map(|location| location.name.clone())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Unknown current location '{}'.",
                    state.current_location
                ))
            })?;
        let title = format!(
            "Final Realm · {current_location_name} · Turn {}",
            state.world_turn
        );
        let choice = tui.select_menu(&title, &options, Some(state))?;
        match choice {
            0 => travel_menu(tui, state, &world)?,
            1 => interact_with_current_location(tui, state, &world)?,
            2 => region_expedition_menu(tui, state, &world)?,
            3 => combat::wander_gielinor(tui, state)?,
            4 => shops::weapon_shop(tui, state)?,
            5 => shops::armour_shop(tui, state)?,
            6 => shops::legends_store(tui, state)?,
            7 => general_store(tui, state)?,
            8 => shops::max_shop(tui, state)?,
            9 => shops::smuggler(tui, state)?,
            10 => stronghold::enter_stronghold(tui, state)?,
            11 => {
                skills::skill_plot(tui, state)?;
                continue;
            }
            12 => bank::bank_menu(tui, state)?,
            13 => quests::quest_hall(tui, state)?,
            14 => inventory::view_inventory(tui, state, false)?,
            15 => crate::achievements::progression_menu(tui, state)?,
            16 => companion_menu(tui, state)?,
            17 => companion_recovery_menu(tui, state, &world)?,
            18 => {
                rest_menu(tui, state, &world)?;
                continue;
            }
            19 => settings_menu(tui, state)?,
            _ => {
                crate::save::save_game(state)
                    .map_err(|error| io::Error::other(error.to_string()))?;
                return Ok(());
            }
        }
        let recovered = state.advance_world_turn();
        if !recovered.is_empty() {
            let names = recovered
                .iter()
                .map(|id| crate::party::get_definition(*id).name)
                .collect::<Vec<_>>()
                .join(", ");
            tui.show_message(
                "Companion recovered",
                &format!("{names} recovered after completing the required main-menu turns. They are available for explicit party selection."),
                Some(state),
            )?;
        }
    }
}

/// Interact with the current location using a stable machine-facing ID.
fn interact_with_current_location(
    tui: &mut Tui,
    state: &mut GameState,
    world: &World,
) -> io::Result<()> {
    let Some(location) = world.get_location(&state.current_location) else {
        tui.show_message(
            "Explore",
            "The current location is not known to this world.",
            Some(state),
        )?;
        return Ok(());
    };
    let target_id = location.id.as_str().to_string();
    let location_name = location.name.clone();
    crate::quests::notify_event(
        state,
        &crate::event::GameEvent::TargetInteracted { target_id },
    );
    if location.id.as_str() == "village_start" {
        crate::adventure::notify_landmark_interaction(state, "village_supply_board");
        if state
            .active_quests
            .iter()
            .any(|q| q.id == crate::adventure::REGION_QUEST_ID && q.is_completed && !q.is_claimed)
        {
            match crate::adventure::claim_region_quest(state) {
                Ok(reward) => tui.show_message(
                    "Oakhaven supplies recovered",
                    &format!("The village receives its supplies. Quest reward: {reward:?}; bonus: {} Logs and {} XP.", crate::adventure::REGION_QUEST_REWARD_LOGS, crate::adventure::REGION_QUEST_REWARD_XP),
                    Some(state),
                )?,
                Err(error) => tui.show_message("Quest not ready", &error.to_string(), Some(state))?,
            }
        }
    } else if let Some(landmark) = crate::adventure::landmark_for_location(&location.id) {
        crate::adventure::notify_landmark_interaction(state, landmark.interaction_target);
    }
    tui.show_message(
        "Explore",
        &format!("You inspect {location_name}."),
        Some(state),
    )
}

/// Run the reusable encounter table attached to the current region location.
/// Each victory uses the party-aware combat engine and the shared domain events.
fn region_expedition_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    use crate::party_battle::{run_party_battle, BattleConfig, BattleOutcome};

    let location_id = state.current_location.clone();
    let Some(encounter) = crate::adventure::encounter_for_location(&location_id) else {
        let detail = world.get_location(&location_id)
            .map(|place| format!("{} has no hostile encounter on its table. You can still inspect landmarks, gather resources, trade, or travel onwards.", place.name))
            .unwrap_or_else(|| "This location is not part of the current region.".to_string());
        tui.show_message("No expedition encounter", &detail, Some(state))?;
        return Ok(());
    };
    let encounter = *encounter;
    let already_cleared = state
        .region_progress
        .cleared_encounters
        .iter()
        .any(|id| id == encounter.id);
    let roster = encounter
        .enemies
        .iter()
        .map(|enemy| {
            format!(
                "{} · level {} · {} gold · {}",
                enemy.name,
                state.player.combat_level.saturating_add(enemy.level_offset),
                enemy.gold_reward,
                enemy
                    .item_reward
                    .map(|(item, count)| format!("{} x{count}", item.name()))
                    .unwrap_or_else(|| "no item drop".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let danger = match encounter.danger {
        5.. => {
            "Extreme danger: prepare equipment, healing, and your active party before committing."
        }
        4 => "High danger: enemy health and damage are increased.",
        _ => "Moderate danger: check supplies and equipment before travelling deeper.",
    };
    tui.show_message(
        "Expedition briefing",
        &format!(
            "{}\nDanger: {}/5\n{}\n\nEncounter roster:\n{}\n\nClear reward: {} gold and {} XP.{}",
            encounter.title,
            encounter.danger,
            danger,
            roster,
            encounter.reward_gold,
            encounter.reward_xp,
            if already_cleared {
                "\nThe one-time clear reward was already claimed."
            } else {
                ""
            }
        ),
        Some(state),
    )?;

    for enemy in encounter.enemies {
        let level = state
            .player
            .combat_level
            .saturating_add(enemy.level_offset)
            .max(1);
        let enemy_hp = (80_u32.saturating_add(level.saturating_mul(22)))
            .saturating_mul(enemy.health_multiplier_pct) / 100;
        let attack = enemy.kind.max_damage().saturating_mul(enemy.attack_multiplier_pct) / 100;
        match run_party_battle(
            tui,
            state,
            BattleConfig {
                enemy_name: enemy.name,
                level,
                enemy_hp: enemy_hp.max(1),
                enemy_attack_power: attack.max(1),
                allow_inventory: true,
                allow_flee: true,
                resource_drain_per_turn: 0,
            },
        )? {
            BattleOutcome::Victory => {
                tui.show_message(
                    "Expedition progress",
                    &format!(
                        "Defeated {}. Its loot and quest progress have been recorded.",
                        enemy.name
                    ),
                    Some(state),
                )?;
            }
            BattleOutcome::Fled => {
                tui.show_message(
                    "Expedition withdrawn",
                    "You retreat from the encounter. No clear reward is granted for an unfinished expedition.",
                    Some(state),
                )?;
                return Ok(());
            }
            BattleOutcome::Defeated => return Ok(()),
        }
    }
    if !already_cleared {
        match crate::adventure::claim_encounter_clear_reward(state, encounter.id) {
            Ok((gold, xp)) => tui.show_message(
                "Expedition complete",
                &format!("The area is secure. Clear reward: {gold} gold and {xp} XP."),
                Some(state),
            )?,
            Err(error) => {
                tui.show_message("Reward unavailable", &error.to_string(), Some(state))?
            }
        }
    } else {
        tui.show_message(
            "Expedition complete",
            "The area has been cleared again, but its one-time clear reward will not be paid twice.",
            Some(state),
        )?;
    }
    Ok(())
}

/// Rest trades resources and world time for stamina recovery.
fn rest_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    let location = world.get_location(&state.current_location);
    let in_settlement = location
        .map(|place| place.category == LocationCategory::Settlement)
        .unwrap_or(false);
    let location_name = location
        .map(|place| place.name.as_str())
        .unwrap_or("Unknown location");
    let food = state.player.get_item_count(ItemType::Food);
    let needs_recovery =
        state.player.stamina < state.player.max_stamina || state.player.hp < state.player.max_hp;

    let mut options = Vec::<String>::new();
    if in_settlement {
        options.push(if state.player.gold >= 25 && needs_recovery {
            "Inn rest — 25 gold; restore stamina and up to 25 HP (2 turns)".to_string()
        } else if state.player.gold < 25 {
            format!(
                "[Disabled: requires 25 gold (have {}).] Inn rest — restore stamina and HP",
                state.player.gold
            )
        } else {
            "[Disabled: already fully recovered.] Inn rest — restore stamina and HP".to_string()
        });
    }
    options.push(if food > 0 && needs_recovery {
        "Camp with food — consume 1 Food; +50 stamina and up to 10 HP (2 turns)".to_string()
    } else if food == 0 {
        "[Disabled: requires 1 Food.] Camp with food — +50 stamina and up to 10 HP".to_string()
    } else {
        "[Disabled: already fully recovered.] Camp with food — recover stamina and HP".to_string()
    });
    options.push(if state.player.stamina < state.player.max_stamina {
        "Sleep rough — +20 stamina; no HP recovery (4 turns)".to_string()
    } else {
        "[Disabled: stamina is already full.] Sleep rough — +20 stamina (4 turns)".to_string()
    });
    options.push("Back".to_string());

    let refs: Vec<&str> = options.iter().map(String::as_str).collect();
    let choice = tui.select_menu(
        &format!(
            "Rest at {location_name} · Stamina {}/{} · Turn {}",
            state.player.stamina, state.player.max_stamina, state.world_turn
        ),
        &refs,
        Some(state),
    )?;
    if choice >= options.len() - 1 {
        return Ok(());
    }

    let inn_offset = usize::from(in_settlement);
    let (stamina_gain, hp_gain, turns, gold_cost, food_cost, full_restore) =
        if in_settlement && choice == 0 {
            (0, 25, 2, 25, 0, true)
        } else if choice == inn_offset {
            (50, 10, 2, 0, 1, false)
        } else {
            (20, 0, 4, 0, 0, false)
        };

    if state.player.gold < gold_cost || state.player.get_item_count(ItemType::Food) < food_cost {
        tui.show_message(
            "Rest unavailable",
            "Not enough gold or food for that rest option.",
            Some(state),
        )?;
        return Ok(());
    }

    state.player.gold -= gold_cost;
    if food_cost > 0 {
        let removed = state.player.remove_item(ItemType::Food, food_cost);
        debug_assert!(removed, "validated rest food must exist");
    }
    if full_restore {
        state.player.stamina = state.player.max_stamina;
    } else {
        state.player.stamina = state
            .player
            .stamina
            .saturating_add(stamina_gain)
            .min(state.player.max_stamina);
    }
    state.player.hp = state
        .player
        .hp
        .saturating_add(hp_gain)
        .min(state.player.max_hp);

    let mut recovered = Vec::new();
    for _ in 0..turns {
        recovered.extend(state.advance_world_turn());
    }
    let names = recovered
        .iter()
        .map(|id| crate::party::get_definition(*id).name)
        .collect::<Vec<_>>();
    let note = if names.is_empty() {
        String::new()
    } else {
        format!("\nCompanion recovery completed: {}.", names.join(", "))
    };
    tui.show_message(
        "Rest complete",
        &format!(
            "Rested at {location_name}. Stamina: {}/{} · HP: {}/{}. World time advanced by {turns} turns.{note}",
            state.player.stamina, state.player.max_stamina, state.player.hp, state.player.max_hp
        ),
        Some(state),
    )?;
    Ok(())
}

/// Applies a single directly connected journey after validating the route,
/// destination, and positive travel cost. An error leaves the saved location
/// unchanged.
fn apply_travel(
    state: &mut GameState,
    world: &World,
    destination_id: &LocationId,
) -> Result<u32, &'static str> {
    let current_id = state.current_location.clone();
    let route = world
        .get_direct_routes(&current_id)
        .iter()
        .find(|route| &route.destination_id == destination_id)
        .ok_or("That destination is not directly connected to your current location.")?;

    if route.travel_cost == 0 {
        return Err("This route has an invalid travel cost.");
    }
    let destination = world
        .get_location(destination_id)
        .ok_or("That destination is not known to the world.")?;
    let travel_cost = route.travel_cost;

    // Mutate only after the direct route and its cost have been validated.
    state.current_location = destination.id.clone();
    crate::quests::notify_event(
        state,
        &crate::event::GameEvent::LocationTraversed {
            from_location_id: current_id.clone(),
            to_location_id: destination.id.clone(),
        },
    );
    crate::quests::notify_event(
        state,
        &crate::event::GameEvent::LocationReached {
            location_id: destination.id.clone(),
        },
    );
    Ok(travel_cost)
}

fn travel_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    loop {
        let current_id = state.current_location.clone();
        let current_name = world
            .get_location(&current_id)
            .map(|location| location.name.clone())
            .ok_or_else(|| io::Error::other(format!("Unknown current location '{current_id}'.")))?;

        let destinations: Vec<(LocationId, String, u32, u32)> = world
            .get_direct_routes(&current_id)
            .iter()
            .filter_map(|route| {
                world.get_location(&route.destination_id).map(|location| {
                    (
                        location.id.clone(),
                        location.name.clone(),
                        route.travel_cost,
                        route.danger_level,
                    )
                })
            })
            .collect();

        if destinations.is_empty() {
            tui.show_message(
                "No routes available",
                "There are no direct travel connections from this location.",
                Some(state),
            )?;
            return Ok(());
        }

        let labels: Vec<String> = destinations
            .iter()
            .map(|(_, name, cost, danger)| format!("{name} · travel cost {cost} · danger {danger}"))
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");

        let title = format!("Travel from {current_name}");
        let choice = tui.select_menu(&title, &options, Some(state))?;
        if choice >= destinations.len() {
            return Ok(());
        }

        let (destination_id, destination_name, _listed_cost, danger_level) = &destinations[choice];
        match apply_travel(state, world, destination_id) {
            Ok(travel_cost) => {
                tui.show_message(
                    "Travel complete",
                    &format!(
                        "You arrive at {destination_name}.\n\nTravel cost: {travel_cost}\nRoute danger: {danger_level}"
                    ),
                    Some(state),
                )?;
                return Ok(());
            }
            Err(message) => {
                tui.show_message("Travel unavailable", message, Some(state))?;
            }
        }
    }
}

fn general_store(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let items = [
        ItemType::GoblinMail,
        ItemType::HugeBones,
        ItemType::DragonHides,
        ItemType::RunicTablets,
        ItemType::Potions,
        ItemType::Food,
        ItemType::Seeds,
        ItemType::Fur,
        ItemType::Ore,
        ItemType::Logs,
        ItemType::Gemstones,
        ItemType::MagicOrbs,
        ItemType::RawFood,
        ItemType::Bait,
        ItemType::Ingots,
        ItemType::Bows,
    ];

    loop {
        let labels: Vec<String> = items
            .iter()
            .map(|item| {
                format!(
                    "{} · owned {} · buy {}g / sell {}g",
                    item.name(),
                    state.player.get_item_count(*item),
                    crate::mastery::vendor_price(
                        state,
                        crate::economy::calculate_dynamic_buy_price(state, *item),
                        0
                    ),
                    crate::economy::calculate_dynamic_sell_price(state, *item)
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Trading Post · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        if choice >= items.len() {
            return Ok(());
        }
        buy_sell_item(tui, state, items[choice])?;
    }
}

fn buy_sell_item(tui: &mut Tui, state: &mut GameState, item: ItemType) -> io::Result<()> {
    loop {
        let choice = tui.select_menu(
            &format!(
                "{} · owned {}",
                item.name(),
                state.player.get_item_count(item)
            ),
            &["Buy", "Sell", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let amount_text = tui.prompt_input("Amount to buy", false, Some(state))?;
                let amount = match amount_text.parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => {
                        tui.show_message(
                            "Purchase failed",
                            "Enter a whole number greater than zero.",
                            Some(state),
                        )?;
                        continue;
                    }
                };
                match crate::economy::buy_item_transaction(state, item, amount) {
                    Ok(cost) => tui.show_message(
                        "Purchase complete",
                        &format!("Bought {} {} for {} gold.", amount, item.name(), cost),
                        Some(state),
                    )?,
                    Err(error) => {
                        tui.show_message("Purchase failed", &error.to_string(), Some(state))?
                    }
                }
            }
            1 => {
                let amount_text =
                    tui.prompt_input("Amount to sell, or type all", false, Some(state))?;
                let owned = state.player.get_item_count(item);
                let amount = if amount_text.eq_ignore_ascii_case("all") {
                    owned
                } else {
                    match amount_text.parse::<u32>() {
                        Ok(value) => value,
                        Err(_) => {
                            tui.show_message(
                                "Sale failed",
                                "Enter a whole number or type all.",
                                Some(state),
                            )?;
                            continue;
                        }
                    }
                };
                match crate::economy::sell_item_transaction(state, item, amount) {
                    Ok(revenue) => tui.show_message(
                        "Sale complete",
                        &format!("Sold {} {} for {} gold.", amount, item.name(), revenue),
                        Some(state),
                    )?,
                    Err(error) => {
                        tui.show_message("Sale failed", &error.to_string(), Some(state))?
                    }
                }
            }
            _ => return Ok(()),
        }
    }
}

fn companion_recovery_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    use crate::companion_recovery::{RecoveryError, TREATMENT_COST};

    loop {
        let mut labels = Vec::new();
        let mut ids = Vec::new();
        for record in &state.companion_recovery.records {
            if let Some(companion) = state
                .companions
                .iter()
                .find(|c| c.id == record.companion_id)
            {
                let destination = record
                    .destination
                    .as_ref()
                    .and_then(|id| world.get_location(id))
                    .map(|location| location.name.as_str())
                    .unwrap_or("No reachable treatment settlement");
                labels.push(format!(
                    "{} · {} · {} committed turn(s) remaining · treatment {}g",
                    companion.display_name(),
                    destination,
                    record.turns_remaining,
                    TREATMENT_COST
                ));
                ids.push(record.companion_id);
            }
        }
        labels.push("Back".to_string());
        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let choice = tui.select_menu("Companion Recovery", &options, Some(state))?;
        if choice >= ids.len() {
            return Ok(());
        }
        let id = ids[choice];
        let record = state.companion_recovery.record(id).cloned();
        let destination = record.as_ref().and_then(|r| r.destination.clone());
        let Some(destination) = destination else {
            tui.show_message(
                "Treatment unavailable",
                "No reachable settlement was found when the companion was knocked out. Their state is preserved; no teleport or payment has been applied.",
                Some(state),
            )?;
            continue;
        };
        let destination_name = world
            .get_location(&destination)
            .map(|loc| loc.name.clone())
            .unwrap_or_else(|| destination.to_string());
        let is_here = state.current_location == destination;
        let mut detail = format!(
            "{} is being treated at {}.\nRecovery requirement: {} committed main-menu turn(s).\nImmediate treatment: {} gold.\nCurrent gold: {}.",
            crate::party::get_definition(id).name,
            destination_name,
            record.as_ref().map(|r| r.turns_remaining).unwrap_or(0),
            TREATMENT_COST,
            state.player.gold
        );
        if !is_here {
            detail.push_str("\n\nTravel to that settlement before paying for immediate treatment.");
        }
        tui.show_message("Recovery details", &detail, Some(state))?;
        let choices = if is_here {
            vec!["Pay for immediate treatment", "Back"]
        } else {
            vec!["Back"]
        };
        let selected = tui.select_menu("Recovery details", &choices, Some(state))?;
        if !is_here || selected != 0 {
            continue;
        }
        let treatment_location = state.current_location.clone();
        let treatment_result = {
            let recovery = &mut state.companion_recovery;
            let companion = state
                .companions
                .iter_mut()
                .find(|companion| companion.id == id);
            let Some(companion) = companion else {
                return Err(io::Error::other(
                    "Recovery record refers to a missing companion.",
                ));
            };
            let result = recovery.apply_treatment_parts(
                &mut state.player.gold,
                &mut state.party,
                companion,
                id,
                &treatment_location,
                world,
            );
            result
        };
        match treatment_result {
            Ok(cost) => tui.show_message(
                "Treatment complete",
                &format!("{} has recovered. {cost} gold paid; companion is available but has not been automatically added to the party.", crate::party::get_definition(id).name),
                Some(state),
            )?,
            Err(error) => {
                let title = if matches!(error, RecoveryError::InsufficientGold { .. }) {
                    "Treatment unaffordable"
                } else {
                    "Treatment unavailable"
                };
                tui.show_message(title, &error.to_string(), Some(state))?;
            }
        }
    }
}

fn companion_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let party_count = state.party.companion_count();
        let party_limit = state.party.max_companions();
        let roster_count = state.companions.len();
        let title = format!(
            "Companions & Party · {} recruited · {}/{} party slots",
            roster_count, party_count, party_limit
        );
        let choice = tui.select_menu(
            &title,
            &["View Roster", "Manage Party", "Back"],
            Some(state),
        )?;
        match choice {
            0 => view_roster(tui, state)?,
            1 => manage_party(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn view_roster(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    use crate::party::{
        all_companion_definitions, build_roster_actions, resolve_roster_action, PartyManager,
        RosterAction,
    };

    let definitions = all_companion_definitions();

    loop {
        let mut labels: Vec<String> = definitions
            .iter()
            .map(|def| {
                let recruited = state.companions.iter().find(|c| c.id == def.id);
                let status = match recruited {
                    Some(c) => format!("Recruited · {}", c.condition.name()),
                    None => {
                        let mut req = String::new();
                        if def.recruitment_level > 0 {
                            req.push_str(&format!("Lvl {}", def.recruitment_level));
                        }
                        if def.recruitment_cost > 0 {
                            if !req.is_empty() {
                                req.push_str(" · ");
                            }
                            req.push_str(&format!("{}g", def.recruitment_cost));
                        }
                        if req.is_empty() {
                            "Not recruited".to_string()
                        } else {
                            format!("Not recruited · {}", req)
                        }
                    }
                };
                format!("{} · {} · {}", def.name, def.role.name(), status)
            })
            .collect();
        labels.push("Back".to_string());

        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let choice = tui.select_menu("Companion Roster", &options, Some(state))?;
        if choice >= definitions.len() {
            return Ok(());
        }

        let def = &definitions[choice];
        let recruited = state.companions.iter().find(|c| c.id == def.id);

        if let Some(companion) = recruited {
            // Show companion details and options
            let in_party = state.party.contains(def.id);
            let can_add_to_party =
                !in_party && companion.can_join_party() && state.party.has_room();

            let mut detail = format!(
                "Name: {}\nRole: {}\nCondition: {}\nHP: {}/{}\nAttack: {}\nDefence: {}\nWeapon: {}\nArmour: {}",
                def.name,
                def.role.name(),
                companion.condition.name(),
                companion.current_hp,
                companion.max_hp,
                companion.attack,
                companion.defence,
                companion.equipped_weapon.name(),
                companion.equipped_armour.name(),
            );
            if in_party {
                detail.push_str("\n\nCurrently in active party.");
            }

            let actions = build_roster_actions(in_party, can_add_to_party);
            let action_labels: Vec<&str> = actions.iter().map(|a| a.label()).collect();

            let action_choice = tui.select_menu(def.name, &action_labels, Some(state))?;
            let resolved = resolve_roster_action(&actions, action_choice);

            match resolved {
                RosterAction::Details => {
                    tui.show_message(def.name, &detail, Some(state))?;
                }
                RosterAction::RemoveFromParty => {
                    match PartyManager::remove_from_party(&mut state.party, def.id) {
                        Ok(()) => tui.show_message(
                            "Party updated",
                            &format!("{} removed from the party.", def.name),
                            Some(state),
                        )?,
                        Err(e) => {
                            tui.show_message("Could not remove", &e.to_string(), Some(state))?
                        }
                    }
                }
                RosterAction::AddToParty => {
                    match PartyManager::add_to_party(&mut state.party, &state.companions, def.id) {
                        Ok(()) => tui.show_message(
                            "Party updated",
                            &format!("{} added to the party.", def.name),
                            Some(state),
                        )?,
                        Err(e) => tui.show_message("Could not add", &e.to_string(), Some(state))?,
                    }
                }
                RosterAction::Dismiss => {
                    match PartyManager::dismiss(&mut state.companions, &mut state.party, def.id) {
                        Ok(()) => tui.show_message(
                            "Companion dismissed",
                            &format!("{} has left your company.", def.name),
                            Some(state),
                        )?,
                        Err(e) => {
                            tui.show_message("Could not dismiss", &e.to_string(), Some(state))?
                        }
                    }
                }
                RosterAction::Back => {
                    // Return to roster selection without modifying any state.
                }
            }
        } else {
            // Not recruited - show recruitment option
            let detail = format!(
                "Name: {}\nRole: {}\n\n{}\n\nBase HP: {}\nBase Attack: {}\nBase Defence: {}\n\nRecruitment requires: Level {} · {} gold",
                def.name,
                def.role.name(),
                def.description,
                def.base_hp,
                def.base_attack,
                def.base_defence,
                def.recruitment_level,
                def.recruitment_cost,
            );
            let action_choice = tui.select_menu(
                &format!("{} · Recruit", def.name),
                &["View Details", "Recruit", "Back"],
                Some(state),
            )?;
            match action_choice {
                0 => {
                    tui.show_message(def.name, &detail, Some(state))?;
                }
                1 => {
                    match PartyManager::recruit(
                        &mut state.companions,
                        def.id,
                        state.player.combat_level,
                        state.player.gold,
                    ) {
                        Ok(cost) => {
                            state.player.gold -= cost;
                            tui.show_message(
                                "Companion recruited",
                                &format!(
                                    "{} has joined your company!\nGold spent: {}",
                                    def.name, cost
                                ),
                                Some(state),
                            )?;
                        }
                        Err(e) => {
                            tui.show_message("Recruitment failed", &e.to_string(), Some(state))?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn visible_party_members(state: &GameState) -> Vec<(crate::party::CompanionId, String, String)> {
    // Keep each visible label paired with the ID it represents. A save with
    // a party ID that has no matching recruited companion is filtered out,
    // without shifting a later selection onto another ID.
    state
        .party
        .members()
        .iter()
        .filter_map(|&id| {
            state.companions.iter().find(|c| c.id == id).map(|c| {
                (
                    id,
                    format!(
                        "{} · {} · HP {}/{}",
                        c.display_name(),
                        c.role().name(),
                        c.current_hp,
                        c.max_hp
                    ),
                    c.display_name().to_string(),
                )
            })
        })
        .collect()
}

fn manage_party(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    use crate::party::{
        build_party_menu_actions, resolve_party_menu_action, PartyManager, PartyMenuAction,
    };

    loop {
        let party_members = visible_party_members(state);

        let member_count = party_members.len();
        let menu_actions = build_party_menu_actions(member_count);

        // Build display labels that match the menu_actions list exactly.
        // When the party is empty, show "No companions in party" as an
        // informational line in the title instead of a selectable entry.
        let mut labels: Vec<String> = party_members
            .iter()
            .map(|(_, label, _)| label.clone())
            .collect();
        labels.push(format!(
            "Add companion ({}/{})",
            state.party.companion_count(),
            state.party.max_companions()
        ));
        labels.push("Back".to_string());

        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let title = if member_count == 0 {
            "Active Party · no companions · select Add companion or Back".to_string()
        } else {
            format!(
                "Active Party · {} companion{}",
                state.party.companion_count(),
                if state.party.companion_count() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        let choice = tui.select_menu(&title, &options, Some(state))?;

        let resolved = resolve_party_menu_action(&menu_actions, choice);

        match resolved {
            PartyMenuAction::Member(index) => {
                // Resolve through the same filtered list used to render the menu,
                // rather than indexing the raw saved party IDs.
                let Some((id, _, name)) = party_members.get(index) else {
                    tui.show_message(
                        "Party updated",
                        "That party member is no longer available. The menu has been refreshed.",
                        Some(state),
                    )?;
                    continue;
                };
                let id = *id;
                let name = name.clone();
                match PartyManager::remove_from_party(&mut state.party, id) {
                    Ok(()) => {
                        tui.show_message(
                            "Party updated",
                            &format!("{} removed from the party.", name),
                            Some(state),
                        )?;
                    }
                    Err(e) => {
                        tui.show_message("Could not remove", &e.to_string(), Some(state))?;
                    }
                }
            }
            PartyMenuAction::AddCompanion => {
                let available: Vec<_> = state
                    .companions
                    .iter()
                    .filter(|c| c.can_join_party() && !state.party.contains(c.id))
                    .collect();
                if available.is_empty() {
                    tui.show_message(
                        "No companions available",
                        "All recruited companions are already in the party, unavailable, or you have no companions.",
                        Some(state),
                    )?;
                    continue;
                }
                let add_labels: Vec<String> = available
                    .iter()
                    .map(|c| {
                        format!(
                            "{} · {} · HP {}/{}",
                            c.display_name(),
                            c.role().name(),
                            c.current_hp,
                            c.max_hp
                        )
                    })
                    .collect();
                let mut add_options: Vec<&str> = add_labels.iter().map(String::as_str).collect();
                add_options.push("Back");
                let add_choice = tui.select_menu("Add to Party", &add_options, Some(state))?;
                if add_choice < available.len() {
                    let id = available[add_choice].id;
                    match PartyManager::add_to_party(&mut state.party, &state.companions, id) {
                        Ok(()) => {
                            tui.show_message(
                                "Party updated",
                                &format!(
                                    "{} added to the party.",
                                    available[add_choice].display_name()
                                ),
                                Some(state),
                            )?;
                        }
                        Err(e) => {
                            tui.show_message("Could not add", &e.to_string(), Some(state))?;
                        }
                    }
                }
            }
            PartyMenuAction::Back => {
                return Ok(());
            }
        }
    }
}

fn settings_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice =
            tui.select_menu("Settings", &["Admin Commands (Debug)", "Back"], Some(state))?;
        match choice {
            0 => verify_admin(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn verify_admin(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let code = tui.prompt_input("Administrator login code", false, Some(state))?;
    if code != "741" {
        tui.show_message("Access denied", "Invalid code.", Some(state))?;
        return Ok(());
    }

    let pin = tui.prompt_input("Administrator verification PIN", true, Some(state))?;
    if pin != "9763" {
        tui.show_message("Access denied", "Invalid PIN.", Some(state))?;
        return Ok(());
    }
    admin_commands(tui, state)
}

fn admin_commands(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice = tui.select_menu(
            "Developer settings",
            &["Edit a variable", "List variables", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let command = tui.prompt_input("set variable=value", false, Some(state))?;
                if let Some((var, val)) = command.split_once('=') {
                    let var = var.trim();
                    let val = val.trim();
                    let updated = match var {
                        "combat_level" => val.parse::<u32>().map(|v| state.player.combat_level = v).is_ok(),
                        "xp" => val.parse::<u32>().map(|v| state.player.xp = v).is_ok(),
                        "gold" => val.parse::<u32>().map(|v| state.player.gold = v).is_ok(),
                        "hp" => val.parse::<u32>().map(|v| state.player.hp = v).is_ok(),
                        "max_hp" => val.parse::<u32>().map(|v| state.player.max_hp = v).is_ok(),
                        "stronghold_kill_count" => val.parse::<u32>().map(|v| state.player.stronghold_kill_count = v).is_ok(),
                        "woodcutting_lvl" => val.parse::<u32>().map(|v| state.player.skills.woodcutting.level = v).is_ok(),
                        "cooking_lvl" => val.parse::<u32>().map(|v| state.player.skills.cooking.level = v).is_ok(),
                        "fishing_lvl" => val.parse::<u32>().map(|v| state.player.skills.fishing.level = v).is_ok(),
                        "mining_lvl" => val.parse::<u32>().map(|v| state.player.skills.mining.level = v).is_ok(),
                        "smithing_lvl" => val.parse::<u32>().map(|v| state.player.skills.smithing.level = v).is_ok(),
                        "thieving_lvl" => val.parse::<u32>().map(|v| state.player.skills.thieving.level = v).is_ok(),
                        _ => false,
                    };
                    let detail = if updated {
                        format!("Set {} to {}.", var, val)
                    } else {
                        "Unknown variable or invalid value.".to_string()
                    };
                    tui.show_message(
                        if updated { "Variable updated" } else { "Variable not updated" },
                        &detail,
                        Some(state),
                    )?;
                } else {
                    tui.show_message("Invalid command", "Use the format set variable=value.", Some(state))?;
                }
            }
            1 => tui.show_text(
                "Editable variables",
                "combat_level\nxp\ngold\nhp\nmax_hp\nstronghold_kill_count\nwoodcutting_lvl\ncooking_lvl\nfishing_lvl\nmining_lvl\nsmithing_lvl\nthieving_lvl",
                Some(state),
            )?,
            _ => return Ok(()),
        }
    }
}

#[cfg(test)]
mod travel_tests {
    use super::*;
    use crate::state::Player;

    fn test_state() -> GameState {
        GameState::new(Player::new("traveller".into(), "test-hash".into()))
    }

    #[test]
    fn party_menu_selection_stays_aligned_when_a_saved_member_is_missing() {
        use crate::party::{
            build_party_menu_actions, resolve_party_menu_action, CompanionId, CompanionState,
            Party, PartyMenuAction,
        };

        let mut state = test_state();
        let lyra = crate::party::get_definition(CompanionId::LyraSwiftbow);
        state
            .companions
            .push(CompanionState::new_from_definition(&lyra));
        state.party = serde_json::from_str::<Party>(
            r#"{"members":["bran_the_shield","lyra_swiftbow"],"size_limit":4}"#,
        )
        .expect("the simulated inconsistent party should deserialize");

        let visible = visible_party_members(&state);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].0, CompanionId::LyraSwiftbow);
        assert!(visible[0].1.starts_with("Lyra Swiftbow"));

        let actions = build_party_menu_actions(visible.len());
        assert_eq!(
            resolve_party_menu_action(&actions, 0),
            PartyMenuAction::Member(0)
        );
        assert_eq!(
            visible[0].0,
            CompanionId::LyraSwiftbow,
            "the displayed first row must map back to Lyra, not the missing Bran ID"
        );
    }

    #[test]
    fn direct_travel_updates_location_after_route_validation() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let destination = LocationId::from("abandoned_mill");

        let cost = apply_travel(&mut state, &world, &destination)
            .expect("direct route should be traversable");

        assert_eq!(cost, 5);
        assert_eq!(state.current_location, destination);
    }

    #[test]
    fn non_adjacent_destination_is_rejected_without_changing_location() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let starting_location = state.current_location.clone();
        let destination = LocationId::from("goblin_caves");

        assert!(world.is_reachable(&starting_location, &destination));
        assert!(!world.can_travel(&starting_location, &destination));
        assert!(apply_travel(&mut state, &world, &destination).is_err());
        assert_eq!(state.current_location, starting_location);
    }

    #[test]
    fn unknown_destination_is_rejected_without_changing_location() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let starting_location = state.current_location.clone();

        assert!(apply_travel(&mut state, &world, &LocationId::from("unknown_location")).is_err());
        assert_eq!(state.current_location, starting_location);
    }
}
