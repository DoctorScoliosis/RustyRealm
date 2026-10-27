// brevity omitted functions/structs/parts: none

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

use crate::state::GameState;
use crate::ui::{clear_screen, wait_for_enter, print_error, print_success};

fn main() {
    clear_screen();
    println!("========================================");
    println!("        Welcome to Rusty Realm        ");
    println!("  A cross-platform revival of Final   ");
    println!("           Realm by Brian             ");
    println!("========================================\n");

    loop {
        println!("What would you like to do?\n");
        println!("1) Login");
        println!("2) Create Account\n");
        println!("Current Version: v1.7_08 (Rusty Remaster)\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                if let Some(mut state) = login() {
                    main_menu(&mut state);
                }
            }
            "2" => {
                if let Some(mut state) = create_account() {
                    main_menu(&mut state);
                }
            }
            _ => continue,
        }
    }
}

fn login() -> Option<GameState> {
    clear_screen();
    let username = crate::ui::get_input("Username: ");
    if username.is_empty() {
        return None;
    }
    
    let password = crate::ui::get_hidden_input("Password: ");
    
    match crate::save::load_game(&username, &password) {
        Ok(state) => {
            print_success(&format!("Welcome back, {}!", state.player.username));
            Some(state)
        }
        Err(e) => {
            print_error(&e);
            None
        }
    }
}

fn create_account() -> Option<GameState> {
    clear_screen();
    let username = crate::ui::get_input("What would you like your Username to be? ");
    if username.is_empty() || username.contains(' ') {
        print_error("Sorry, you can't use spaces or leave it blank in your Username.");
        return None;
    }
    
    let password = crate::ui::get_hidden_input("What would you like your Password to be? ");
    if password.is_empty() {
        print_error("Password cannot be empty.");
        return None;
    }
    
    match crate::save::create_account(&username, &password) {
        Ok(state) => {
            print_success("Account created successfully!");
            Some(state)
        }
        Err(e) => {
            print_error(&e);
            None
        }
    }
}

fn main_menu(state: &mut GameState) {
    loop {
        crate::save::save_game(state).unwrap_or_else(|e| {
            eprintln!("Failed to save game: {}", e);
        });

        clear_screen();
        println!("========================================");
        println!("        FinalRealm - Varrock            ");
        println!("========================================\n");
        println!("What would you like to do?\n");
        println!(" 1) Wander Gielinor");
        println!(" 2) Weapon Shop");
        println!(" 3) Armour Shop");
        println!(" 4) Legend's Store");
        println!(" 5) Trading Post (General Store)");
        println!(" 6) Max's Shop");
        println!(" 7) Smuggler");
        println!(" 8) The Stronghold");
        println!(" 9) Skill Plot");
        println!("10) RealmBank");
        println!("11) Quest Hall");
        println!("97) Your Inventory");
        println!("98) Settings");
        println!("99) Log out\n");

        println!("Currently logged in as {}.", state.player.username);
        println!("Combat Level: {} | Total Level: {} | Gold: {}\n", 
                 state.player.combat_level, 
                 state.player.skills.total_level() + state.player.combat_level,
                 state.player.gold);

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => combat::wander_gielinor(state),
            "2" => shops::weapon_shop(state),
            "3" => shops::armour_shop(state),
            "4" => shops::legends_store(state),
            "5" => general_store(state),
            "6" => shops::max_shop(state),
            "7" => shops::smuggler(state),
            "8" => {
                if stronghold::stronghold_warning(state) {
                    stronghold::stronghold_dungeon(state);
                }
            }
            "9" => skills::skill_plot(state),
            "10" => bank::bank_menu(state),
            "11" => quests::quest_hall(state),
            "97" => inventory::view_inventory(state, false),
            "98" => settings_menu(state),
            "99" => return,
            _ => continue,
        }
    }
}

fn general_store(state: &mut GameState) {
    use crate::data::ItemType;
    loop {
        clear_screen();
        println!("FinalRealm - General Store\n");
        println!("Gold: {}\n", state.player.gold);
        println!("Your Items:\t\t\tPrices:");
        
        let items = [
            ItemType::GoblinMail, ItemType::HugeBones, ItemType::DragonHides, ItemType::RunicTablets,
            ItemType::Potions, ItemType::Food, ItemType::Seeds, ItemType::Fur,
            ItemType::Ore, ItemType::Logs, ItemType::Gemstones, ItemType::MagicOrbs,
            ItemType::RawFood, ItemType::Bait, ItemType::Ingots, ItemType::Bows,
        ];
        
        for (i, item) in items.iter().enumerate() {
            let count = state.player.get_item_count(*item);
            println!("{:2}) {:<18} {:<8}\t{}", i + 1, item.name(), count, item.buy_price());
        }
        
        println!("\nType \"exit\" to exit.\n");
        
        let choice = crate::ui::get_input("Enter the number of the item you would like to buy or sell: ");
        if choice.to_lowercase() == "exit" {
            return;
        }
        
        if let Ok(idx) = choice.parse::<usize>() {
            if idx >= 1 && idx <= items.len() {
                let item = items[idx - 1];
                buy_sell_item(state, item);
            }
        }
    }
}

fn buy_sell_item(state: &mut GameState, item: crate::data::ItemType) {
    loop {
        clear_screen();
        println!("Item: {}", item.name());
        println!("You have: {}", state.player.get_item_count(item));
        println!("Price: {} gold each\n", item.buy_price());
        println!("1) Buy");
        println!("2) Sell");
        println!("3) Back\n");
        
        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                let amount_str = crate::ui::get_input("Amount to purchase: ");
                if let Ok(amount) = amount_str.parse::<u32>() {
                    if amount == 0 { continue; }
                    let cost = amount * item.buy_price();
                    if cost > state.player.gold {
                        println!("\nYou do not have enough money.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    state.player.gold -= cost;
                    state.player.add_item(item, amount);
                    println!("\nBought {} of that item for {} gold.", amount, cost);
                    crate::ui::wait_for_enter();
                    return;
                }
            }
            "2" => {
                let amount_str = crate::ui::get_input("Amount to sell (or \"all\"): ");
                if amount_str.to_lowercase() == "all" {
                    let count = state.player.get_item_count(item);
                    if count == 0 {
                        println!("\nYou don't have any to sell.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    let price = count * item.sell_price();
                    state.player.gold += price;
                    state.player.remove_item(item, count);
                    println!("\nSold {} of that item for {} gold.", count, price);
                    crate::ui::wait_for_enter();
                    return;
                } else if let Ok(amount) = amount_str.parse::<u32>() {
                    if amount == 0 { continue; }
                    if amount > state.player.get_item_count(item) {
                        println!("\nYou don't have that many.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    let price = amount * item.sell_price();
                    state.player.gold += price;
                    state.player.remove_item(item, amount);
                    println!("\nSold {} of that item for {} gold.", amount, price);
                    crate::ui::wait_for_enter();
                    return;
                }
            }
            "3" => return,
            _ => continue,
        }
    }
}

fn settings_menu(state: &mut GameState) {
    loop {
        clear_screen();
        println!("What setting would you like to change?\n");
        println!("1) Admin Commands (Debug)");
        println!("2) Exit\n");
        
        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => verify_admin(state),
            "2" => return,
            _ => continue,
        }
    }
}

fn verify_admin(state: &mut GameState) {
    clear_screen();
    println!("Please verify that you are an Administrator.\n");
    
    let code = crate::ui::get_input("Please enter the Login Code: ");
    if code != "741" {
        println!("\nInvalid Code.");
        crate::ui::wait_for_enter();
        return;
    }
    
    let pin = crate::ui::get_input("Please enter the Verification PIN: ");
    if pin != "9763" {
        println!("\nInvalid PIN.");
        crate::ui::wait_for_enter();
        return;
    }
    
    admin_commands(state);
}

fn admin_commands(state: &mut GameState) {
    loop {
        clear_screen();
        println!("Type \"vars\" to get a list of Variables that can be edited, or type \"exit\" to exit.\n");
        println!("Format: set [variable]=[amount]\n");
        
        let cmd = crate::ui::get_input("Set: ");
        if cmd.to_lowercase() == "exit" {
            return;
        }
        if cmd.to_lowercase() == "vars" {
            println!("\nAvailable variables:");
            println!("  combat_level, xp, gold, hp, max_hp");
            println!("  stronghold_kill_count");
            println!("  woodcutting_lvl, cooking_lvl, fishing_lvl");
            println!("  mining_lvl, smithing_lvl, thieving_lvl\n");
            crate::ui::wait_for_enter();
            continue;
        }
        
        if let Some(eq_pos) = cmd.find('=') {
            let var = cmd[..eq_pos].trim();
            let val = cmd[eq_pos + 1..].trim();
            
            match var {
                "combat_level" => if let Ok(v) = val.parse::<u32>() { state.player.combat_level = v; },
                "xp" => if let Ok(v) = val.parse::<u32>() { state.player.xp = v; },
                "gold" => if let Ok(v) = val.parse::<u32>() { state.player.gold = v; },
                "hp" => if let Ok(v) = val.parse::<u32>() { state.player.hp = v; },
                "max_hp" => if let Ok(v) = val.parse::<u32>() { state.player.max_hp = v; },
                "stronghold_kill_count" => if let Ok(v) = val.parse::<u32>() { state.player.stronghold_kill_count = v; },
                "woodcutting_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.woodcutting.level = v; },
                "cooking_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.cooking.level = v; },
                "fishing_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.fishing.level = v; },
                "mining_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.mining.level = v; },
                "smithing_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.smithing.level = v; },
                "thieving_lvl" => if let Ok(v) = val.parse::<u32>() { state.player.skills.thieving.level = v; },
                _ => {
                    println!("\nUnknown variable.");
                    crate::ui::wait_for_enter();
                    continue;
                }
            }
            println!("\nSet {} to {}.", var, val);
            crate::ui::wait_for_enter();
        } else {
            println!("\nInvalid format. Use: set [variable]=[amount]");
            crate::ui::wait_for_enter();
        }
    }
}