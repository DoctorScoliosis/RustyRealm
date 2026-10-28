mod achievements;
mod bank;
mod combat;
mod combat_engine;
mod crafting;
mod data;
mod economy;
mod event;
mod inventory;
mod mastery;
mod party;
mod quests;
mod save;
mod shops;
mod skills;
mod state;
mod stronghold;
mod ui;
// Some world queries are reserved for later gameplay slices; remove this
// allowance as the remaining APIs gain consumers.
#[allow(dead_code)]
mod world;

use crate::data::ItemType;
use crate::state::GameState;
use crate::ui::Tui;
use crate::world::{build_initial_world, LocationId, World};
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
    let options = [
        "Travel / Explore Region",
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
        let title = format!("Final Realm · {current_location_name}");
        let choice = tui.select_menu(&title, &options, Some(state))?;
        match choice {
            0 => travel_menu(tui, state, &world)?,
            1 => combat::wander_gielinor(tui, state)?,
            2 => shops::weapon_shop(tui, state)?,
            3 => shops::armour_shop(tui, state)?,
            4 => shops::legends_store(tui, state)?,
            5 => general_store(tui, state)?,
            6 => shops::max_shop(tui, state)?,
            7 => shops::smuggler(tui, state)?,
            8 => stronghold::enter_stronghold(tui, state)?,
            9 => skills::skill_plot(tui, state)?,
            10 => bank::bank_menu(tui, state)?,
            11 => quests::quest_hall(tui, state)?,
            12 => inventory::view_inventory(tui, state, false)?,
            13 => crate::achievements::progression_menu(tui, state)?,
            14 => companion_menu(tui, state)?,
            15 => settings_menu(tui, state)?,
            _ => {
                crate::save::save_game(state)
                    .map_err(|error| io::Error::other(error.to_string()))?;
                return Ok(());
            }
        }
        state.market_state.advance_turn();
    }
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
    use crate::party::{all_companion_definitions, CompanionCondition, PartyManager};

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
                format!(
                    "{} · {} · {}",
                    def.name,
                    def.role.name(),
                    status
                )
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

            let mut actions = vec!["Details"];
            if in_party {
                actions.push("Remove from Party");
            } else if (companion.condition == CompanionCondition::Available || companion.condition == CompanionCondition::Active)
                && state.party.has_room()
            {
                actions.push("Add to Party");
            }
            actions.push("Dismiss");
            actions.push("Back");

            let action_choice = tui.select_menu(def.name, &actions, Some(state))?;
            match action_choice {
                0 => {
                    tui.show_message(def.name, &detail, Some(state))?;
                }
                1 => {
                    if in_party {
                        match PartyManager::remove_from_party(&mut state.party, def.id) {
                            Ok(()) => tui.show_message(
                                "Party updated",
                                &format!("{} removed from the party.", def.name),
                                Some(state),
                            )?,
                            Err(e) => tui.show_message(
                                "Could not remove",
                                &e.to_string(),
                                Some(state),
                            )?,
                        }
                    } else {
                        match PartyManager::add_to_party(&mut state.party, &state.companions, def.id) {
                            Ok(()) => tui.show_message(
                                "Party updated",
                                &format!("{} added to the party.", def.name),
                                Some(state),
                            )?,
                            Err(e) => tui.show_message(
                                "Could not add",
                                &e.to_string(),
                                Some(state),
                            )?,
                        }
                    }
                }
                2 => {
                    match PartyManager::dismiss(&mut state.companions, &mut state.party, def.id) {
                        Ok(()) => tui.show_message(
                            "Companion dismissed",
                            &format!("{} has left your company.", def.name),
                            Some(state),
                        )?,
                        Err(e) => tui.show_message(
                            "Could not dismiss",
                            &e.to_string(),
                            Some(state),
                        )?,
                    }
                }
                _ => {}
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

fn manage_party(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    use crate::party::PartyManager;

    loop {
        let party_members: Vec<String> = state
            .party
            .members()
            .iter()
            .filter_map(|&id| {
                state
                    .companions
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| {
                        format!(
                            "{} · {} · HP {}/{}",
                            c.display_name(),
                            c.role().name(),
                            c.current_hp,
                            c.max_hp
                        )
                    })
            })
            .collect();

        let mut labels: Vec<String> = party_members.clone();
        if labels.is_empty() {
            labels.push("No companions in party".to_string());
        }
        labels.push(format!(
            "Add companion ({}/{})",
            state.party.companion_count(),
            state.party.max_companions()
        ));
        labels.push("Back".to_string());

        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let title = format!(
            "Active Party · {} companion{}",
            state.party.companion_count(),
            if state.party.companion_count() == 1 { "" } else { "s" }
        );
        let choice = tui.select_menu(&title, &options, Some(state))?;

        let member_count = if party_members.is_empty() { 0 } else { party_members.len() };

        if choice == member_count {
            // Add companion
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
                            &format!("{} added to the party.", available[add_choice].display_name()),
                            Some(state),
                        )?;
                    }
                    Err(e) => {
                        tui.show_message("Could not add", &e.to_string(), Some(state))?;
                    }
                }
            }
        } else if choice == member_count + 1 {
            // Back
            return Ok(());
        } else if choice < member_count {
            // Remove companion
            let id = state.party.members()[choice];
            let name = state
                .companions
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.display_name())
                .unwrap_or("Unknown");
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
