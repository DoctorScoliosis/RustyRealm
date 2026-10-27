use crate::state::GameState;
use crate::data::{ItemType, WeaponTier, ArmourTier};
use crate::ui::{clear_screen, wait_for_enter};

pub fn view_inventory(state: &mut GameState, in_combat: bool) {
    state.player.cap_values();
    
    loop {
        clear_screen();
        println!("FinalRealm - Inventory\n");
        println!("┌────────────────────────────────────────────────────────┐");
        println!("│ Health: {:<45} │", state.player.hp);
        println!("│ Level: {:<45} │", state.player.combat_level);
        println!("│ Gold: {:<45} │", state.player.gold);
        println!("│ EXP: {:<45} │", state.player.xp);
        
        let xp_until = if state.player.combat_level >= 37 {
            "None!".to_string()
        } else {
            state.player.xp_until_next.to_string()
        };
        println!("│ EXP until next level: {:<36} │", xp_until);
        println!("│ Total level: {:<43} │", state.player.skills.total_level() + state.player.combat_level);
        println!("│ Weapon: {:<44} │", state.player.weapon.name());
        println!("│ Armour: {:<44} │", state.player.armour.name());
        println!("├────────────────────────────────────────────────────────┤");
        println!("│ Items:                                                  │");
        println!("│ Bait: {:<4}        │ Gemstones: {:<4}                 │", 
                 state.player.get_item_count(ItemType::Bait), state.player.get_item_count(ItemType::Gemstones));
        println!("│ Seeds: {:<4}       │ Magic Orbs: {:<4}                │", 
                 state.player.get_item_count(ItemType::Seeds), state.player.get_item_count(ItemType::MagicOrbs));
        println!("│ Furs: {:<4}        │ Raw Food: {:<4}                  │", 
                 state.player.get_item_count(ItemType::Fur), state.player.get_item_count(ItemType::RawFood));
        println!("│ Ore: {:<4}         │ Huge Bones: {:<4}                │", 
                 state.player.get_item_count(ItemType::Ore), state.player.get_item_count(ItemType::HugeBones));
        println!("│ Logs: {:<4}        │ Dragon Hides: {:<4}              │", 
                 state.player.get_item_count(ItemType::Logs), state.player.get_item_count(ItemType::DragonHides));
        println!("│ Food: {:<4}        │ Runic Tablets: {:<4}             │", 
                 state.player.get_item_count(ItemType::Food), state.player.get_item_count(ItemType::RunicTablets));
        println!("│ Ingots: {:<4}      │ Goblin Mails: {:<4}              │", 
                 state.player.get_item_count(ItemType::Ingots), state.player.get_item_count(ItemType::GoblinMail));
        println!("│ Potions: {:<4}     │ Bows: {:<4}                      │", 
                 state.player.get_item_count(ItemType::Potions), state.player.get_item_count(ItemType::Bows));
        println!("├────────────────────────────────────────────────────────┤");
        println!("│ Weapons:             │ Armour:                          │");
        
        let weapon_tiers = [
            WeaponTier::Wooden, WeaponTier::Stone, WeaponTier::Bronze, WeaponTier::Iron,
            WeaponTier::Steel, WeaponTier::Gold, WeaponTier::Crystal, WeaponTier::Sacred,
            WeaponTier::Runic, WeaponTier::Infernal, WeaponTier::Omega, WeaponTier::Chaotic,
            WeaponTier::Mythical, WeaponTier::Alydril,
        ];
        
        let armour_tiers = [
            ArmourTier::Cloth, ArmourTier::Chain, ArmourTier::Bronze, ArmourTier::Iron,
            ArmourTier::Steel, ArmourTier::Gold, ArmourTier::Crystal, ArmourTier::Sacred,
            ArmourTier::Runic, ArmourTier::Infernal, ArmourTier::Omega, ArmourTier::Chaotic,
            ArmourTier::Mythical, ArmourTier::Alydril,
        ];
        
        for i in 0..14 {
            let w_count = state.player.inventory_weapons.iter().filter(|&&t| t == weapon_tiers[i]).count();
            let a_count = state.player.inventory_armour.iter().filter(|&&t| t == armour_tiers[i]).count();
            println!("│ {:<19}: {:<2} │ {:<29}: {:<2} │", 
                     weapon_tiers[i].name(), w_count, armour_tiers[i].name(), a_count);
        }
        println!("└────────────────────────────────────────────────────────┘\n");

        if in_combat {
            println!("What would you like to do?\n");
            println!("1) Return to battle");
            println!("2) Drink Potion");
            println!("3) Eat Food");
            println!("4) Change Weapon");
            println!("5) Change Armour\n");
        } else {
            println!("What would you like to do?\n");
            println!("1) Back");
            println!("2) Heal");
            println!("3) Change Weapon");
            println!("4) Change Armour");
            println!("5) View Levels\n");
        }

        let choice = crate::ui::get_input("Choice: ");
        if in_combat {
            match choice.as_str() {
                "1" => return,
                "2" => drink_potion(state),
                "3" => eat_food(state),
                "4" => change_weapon(state),
                "5" => change_armour(state),
                _ => continue,
            }
        } else {
            match choice.as_str() {
                "1" => return,
                "2" => {
                    state.player.hp = state.player.max_hp;
                    println!("\nYou have been healed back to full health.");
                    crate::ui::wait_for_enter();
                }
                "3" => change_weapon(state),
                "4" => change_armour(state),
                "5" => view_levels(state),
                _ => continue,
            }
        }
    }
}

fn drink_potion(state: &mut GameState) {
    if state.player.get_item_count(ItemType::Potions) == 0 {
        println!("\nYou don't have any potions.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.remove_item(ItemType::Potions, 1);
    let heal = rand::random::<u32>() % 100 + 1;
    state.player.hp = (state.player.hp + heal).min(state.player.max_hp);
    
    println!("\nYou drink a potion.");
    std::thread::sleep(std::time::Duration::from_millis(300));
    println!("You have been healed {} health.", heal);
    println!("You now have {} health.", state.player.hp);
    crate::ui::wait_for_enter();
}

fn eat_food(state: &mut GameState) {
    if state.player.get_item_count(ItemType::Food) == 0 {
        println!("\nYou don't have any food.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.remove_item(ItemType::Food, 1);
    let heal = rand::random::<u32>() % 50 + 1;
    state.player.hp = (state.player.hp + heal).min(state.player.max_hp);
    
    println!("\nYou eat some food.");
    std::thread::sleep(std::time::Duration::from_millis(300));
    println!("You have been healed {} health.", heal);
    println!("You now have {} health.", state.player.hp);
    crate::ui::wait_for_enter();
}

fn change_weapon(state: &mut GameState) {
    loop {
        clear_screen();
        println!("You currently have {} swords.", state.player.inventory_weapons.len());
        println!("You currently have a {} equipped.\n", state.player.weapon.name());
        println!("Select a weapon to equip, or type \"back\" to return.\n");

        if state.player.inventory_weapons.is_empty() {
            println!("You have no weapons in your inventory.");
            crate::ui::wait_for_enter();
            return;
        }

        for (i, tier) in state.player.inventory_weapons.iter().enumerate() {
            println!("{}) {}", i + 1, tier.name());
        }
        println!("\nType \"back\" to return.");

        let choice = crate::ui::get_input("Choice: ");
        if choice.to_lowercase() == "back" {
            return;
        }

        if let Ok(idx) = choice.parse::<usize>() {
            if idx > 0 && idx <= state.player.inventory_weapons.len() {
                let new_weapon = state.player.inventory_weapons[idx - 1];
                if state.player.weapon == new_weapon {
                    println!("\nYou already have that weapon equipped.");
                } else {
                    state.player.weapon = new_weapon;
                    println!("\nYou have equipped {}.", new_weapon.name());
                }
                crate::ui::wait_for_enter();
                return;
            }
        }
        println!("\nInvalid choice.");
        crate::ui::wait_for_enter();
    }
}

fn change_armour(state: &mut GameState) {
    loop {
        clear_screen();
        println!("You currently have {} pieces of Armour.", state.player.inventory_armour.len());
        println!("You currently have {} equipped.\n", state.player.armour.name());
        println!("Select an Armour set to wear, or type \"back\" to return.\n");

        if state.player.inventory_armour.is_empty() {
            println!("You have no armour in your inventory.");
            crate::ui::wait_for_enter();
            return;
        }

        for (i, tier) in state.player.inventory_armour.iter().enumerate() {
            println!("{}) {}", i + 1, tier.name());
        }
        println!("\nType \"back\" to return.");

        let choice = crate::ui::get_input("Choice: ");
        if choice.to_lowercase() == "back" {
            return;
        }

        if let Ok(idx) = choice.parse::<usize>() {
            if idx > 0 && idx <= state.player.inventory_armour.len() {
                let new_armour = state.player.inventory_armour[idx - 1];
                if state.player.armour == new_armour {
                    println!("\nYou already have that Armour equipped.");
                } else {
                    state.player.armour = new_armour;
                    println!("\nYou have equipped {}.", new_armour.name());
                }
                crate::ui::wait_for_enter();
                return;
            }
        }
        println!("\nInvalid choice.");
        crate::ui::wait_for_enter();
    }
}

fn view_levels(state: &GameState) {
    clear_screen();
    println!("Levels:\n");
    println!("Total level: {}", state.player.skills.total_level() + state.player.combat_level);
    println!("\nCombat level: {}", state.player.combat_level);
    println!("Woodcutting level: {}", state.player.skills.woodcutting.level);
    println!("Cooking level: {}", state.player.skills.cooking.level);
    println!("Fishing level: {}", state.player.skills.fishing.level);
    println!("Thieving level: {}", state.player.skills.thieving.level);
    println!("Mining level: {}", state.player.skills.mining.level);
    println!("Smithing level: {}", state.player.skills.smithing.level);
    println!("\nType \"back\" to go back.\n");

    let choice = crate::ui::get_input("Choice: ");
    if choice.to_lowercase() == "back" {
        return;
    }
}