mod data;
mod state;
mod save;
mod ui;
mod combat;
mod stronghold;
mod shops;
mod bank;
mod skills;
mod inventory;
mod quests;

use crate::data::ItemType;
use crate::state::GameState;
use crate::ui::Tui;
use std::io;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    loop {
        let choice = tui.select_menu(
            "Rusty Realm",
            &["Login", "Create Account", "Exit"],
            None,
        )?;
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
                tui.show_message("Migration cancelled", "Password cannot be empty. The save has not been changed.", None)?;
                return Ok(None);
            }
            let confirmation = tui.prompt_input("Confirm new password", true, None)?;
            if new_password != confirmation {
                tui.show_message("Migration cancelled", "Passwords do not match. The save has not been changed.", None)?;
                return Ok(None);
            }

            match crate::save::migrate_legacy_save(&username, &new_password) {
                Ok(state) => {
                    tui.show_message("Migration complete", "Legacy save migrated and password protection updated.", Some(&state))?;
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
        tui.show_message("Invalid username", "A username is required, and spaces are not allowed.", None)?;
        return Ok(None);
    }

    let password = tui.prompt_input("Choose password", true, None)?;
    if password.is_empty() {
        tui.show_message("Invalid password", "Password cannot be empty.", None)?;
        return Ok(None);
    }

    match crate::save::create_account(&username, &password) {
        Ok(state) => {
            tui.show_message("Account created", "Account created successfully!", Some(&state))?;
            Ok(Some(state))
        }
        Err(error) => {
            tui.show_message("Account creation failed", &error.to_string(), None)?;
            Ok(None)
        }
    }
}

fn main_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let options = [
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
        "Settings",
        "Log Out",
    ];

    loop {
        if let Err(error) = crate::save::save_game(state) {
            tui.show_message("Save warning", &format!("Failed to save the game: {}", error), Some(state))?;
        }

        let choice = tui.select_menu("Final Realm · Varrock", &options, Some(state))?;
        match choice {
            0 => combat::wander_gielinor(tui, state)?,
            1 => shops::weapon_shop(tui, state)?,
            2 => shops::armour_shop(tui, state)?,
            3 => shops::legends_store(tui, state)?,
            4 => general_store(tui, state)?,
            5 => shops::max_shop(tui, state)?,
            6 => shops::smuggler(tui, state)?,
            7 => {
                if stronghold::stronghold_warning(tui, state)? {
                    stronghold::stronghold_dungeon(tui, state)?;
                }
            }
            8 => skills::skill_plot(tui, state)?,
            9 => bank::bank_menu(tui, state)?,
            10 => quests::quest_hall(tui, state)?,
            11 => inventory::view_inventory(tui, state, false)?,
            12 => settings_menu(tui, state)?,
            _ => {
                crate::save::save_game(state)
                    .map_err(|error| io::Error::other(error.to_string()))?;
                return Ok(());
            }
        }
    }
}

fn general_store(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let items = [
        ItemType::GoblinMail, ItemType::HugeBones, ItemType::DragonHides, ItemType::RunicTablets,
        ItemType::Potions, ItemType::Food, ItemType::Seeds, ItemType::Fur,
        ItemType::Ore, ItemType::Logs, ItemType::Gemstones, ItemType::MagicOrbs,
        ItemType::RawFood, ItemType::Bait, ItemType::Ingots, ItemType::Bows,
    ];

    loop {
        let labels: Vec<String> = items.iter().map(|item| {
            format!("{} · owned {} · buy {}g / sell {}g", item.name(), state.player.get_item_count(*item), item.buy_price(), item.sell_price())
        }).collect();
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
            &format!("{} · owned {}", item.name(), state.player.get_item_count(item)),
            &["Buy", "Sell", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let amount_text = tui.prompt_input("Amount to buy", false, Some(state))?;
                let Ok(amount) = amount_text.parse::<u32>() else { continue };
                if amount == 0 { continue; }
                let Some(cost) = amount.checked_mul(item.buy_price()) else {
                    tui.show_message("Purchase failed", "That quantity is too large.", Some(state))?;
                    continue;
                };
                if cost > state.player.gold {
                    tui.show_message("Purchase failed", "You do not have enough money.", Some(state))?;
                    continue;
                }
                state.player.gold -= cost;
                state.player.add_item(item, amount);
                tui.show_message("Purchase complete", &format!("Bought {} {} for {} gold.", amount, item.name(), cost), Some(state))?;
            }
            1 => {
                let amount_text = tui.prompt_input("Amount to sell, or type all", false, Some(state))?;
                let owned = state.player.get_item_count(item);
                let amount = if amount_text.eq_ignore_ascii_case("all") {
                    owned
                } else {
                    match amount_text.parse::<u32>() { Ok(value) => value, Err(_) => continue }
                };
                if amount == 0 || amount > owned { 
                    tui.show_message("Sale failed", if amount == 0 { "Enter an amount greater than zero." } else { "You do not have that many items." }, Some(state))?;
                    continue;
                }
                let Some(value) = amount.checked_mul(item.sell_price()) else {
                    tui.show_message("Sale failed", "That quantity is too large.", Some(state))?;
                    continue;
                };
                if state.player.remove_item(item, amount) {
                    state.player.gold += value;
                    tui.show_message("Sale complete", &format!("Sold {} {} for {} gold.", amount, item.name(), value), Some(state))?;
                }
            }
            _ => return Ok(()),
        }
    }
}

fn settings_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice = tui.select_menu("Settings", &["Admin Commands (Debug)", "Back"], Some(state))?;
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
