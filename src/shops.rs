use crate::state::GameState;
use crate::data::{WeaponTier, ArmourTier, ItemType};
use crate::ui::{clear_screen, wait_for_enter};

pub fn weapon_shop(state: &mut GameState) {
    loop {
        clear_screen();
        println!("FinalRealm Weapon Shop\n");
        println!("You currently have {} Gold.\n", state.player.gold);

        let weapons = [
            (WeaponTier::Wooden, 1),
            (WeaponTier::Stone, 1),
            (WeaponTier::Bronze, 5),
            (WeaponTier::Iron, 5),
            (WeaponTier::Steel, 10),
            (WeaponTier::Gold, 10), // Fixed: was incorrectly incrementing steel slot
            (WeaponTier::Crystal, 15),
            (WeaponTier::Sacred, 15),
        ];

        for (i, (tier, req_lvl)) in weapons.iter().enumerate() {
            let owned = state.player.inventory_weapons.iter().filter(|&&t| t == *tier).count();
            println!("{}) {:<16} Costs {:<6} gold. Owned: {}", 
                     i + 1, tier.name(), tier.buy_price(), owned);
        }
        println!("9) Next Page");
        println!("Type \"exit\" to exit the store.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "9" => {
                weapon_shop_page_2(state);
                continue;
            }
            "1" => try_buy_weapon(state, WeaponTier::Wooden),
            "2" => try_buy_weapon(state, WeaponTier::Stone),
            "3" => try_buy_weapon(state, WeaponTier::Bronze),
            "4" => try_buy_weapon(state, WeaponTier::Iron),
            "5" => try_buy_weapon(state, WeaponTier::Steel),
            "6" => try_buy_weapon(state, WeaponTier::Gold),
            "7" => try_buy_weapon(state, WeaponTier::Crystal),
            "8" => try_buy_weapon(state, WeaponTier::Sacred),
            _ => continue,
        }
    }
}

fn weapon_shop_page_2(state: &mut GameState) {
    loop {
        clear_screen();
        println!("FinalRealm Weapon Shop - Page 2\n");
        println!("You currently have {} Gold.\n", state.player.gold);

        let weapons = [
            WeaponTier::Runic,
            WeaponTier::Infernal,
            WeaponTier::Omega,
            WeaponTier::Chaotic,
        ];

        for (i, tier) in weapons.iter().enumerate() {
            let owned = state.player.inventory_weapons.iter().filter(|&&t| t == *tier).count();
            println!("{}) {:<16} Costs {:<6} gold. Owned: {}", 
                     i + 1, tier.name(), tier.buy_price(), owned);
        }
        println!("5) Previous Page");
        println!("Type \"exit\" to exit the store.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "5" => return,
            "1" => try_buy_weapon(state, WeaponTier::Runic),
            "2" => try_buy_weapon(state, WeaponTier::Infernal),
            "3" => try_buy_weapon(state, WeaponTier::Omega),
            "4" => try_buy_weapon(state, WeaponTier::Chaotic),
            _ => continue,
        }
    }
}

fn try_buy_weapon(state: &mut GameState, tier: WeaponTier) {
    if state.player.combat_level < tier.level_required() {
        println!("\nYou are not a high enough level to buy this item. You need to be at least level {}.", tier.level_required());
        crate::ui::wait_for_enter();
        return;
    }
    if state.player.gold < tier.buy_price() {
        println!("\nYou don't have enough money.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.gold -= tier.buy_price();
    state.player.inventory_weapons.push(tier);
    println!("\n{} purchased. You have a damage bonus of up to {} with this item.", tier.name(), tier.max_damage());
    crate::ui::wait_for_enter();
}

pub fn armour_shop(state: &mut GameState) {
    loop {
        clear_screen();
        println!("FinalRealm - Armour Shop\n");
        println!("You currently have {} Gold.\n", state.player.gold);

        let armours = [
            (ArmourTier::Cloth, 1),
            (ArmourTier::Chain, 1),
            (ArmourTier::Bronze, 5),
            (ArmourTier::Iron, 5),
            (ArmourTier::Steel, 10),
            (ArmourTier::Gold, 10), // Fixed: was incorrectly incrementing steel slot
            (ArmourTier::Crystal, 15),
            (ArmourTier::Sacred, 15),
        ];

        for (i, (tier, req_lvl)) in armours.iter().enumerate() {
            let owned = state.player.inventory_armour.iter().filter(|&&t| t == *tier).count();
            println!("{}) {:<16} Costs {:<6} gold. Owned: {}", 
                     i + 1, tier.name(), tier.buy_price(), owned);
        }
        println!("9) Next Page");
        println!("Type \"exit\" to exit the store.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "9" => {
                armour_shop_page_2(state);
                continue;
            }
            "1" => try_buy_armour(state, ArmourTier::Cloth),
            "2" => try_buy_armour(state, ArmourTier::Chain),
            "3" => try_buy_armour(state, ArmourTier::Bronze),
            "4" => try_buy_armour(state, ArmourTier::Iron),
            "5" => try_buy_armour(state, ArmourTier::Steel),
            "6" => try_buy_armour(state, ArmourTier::Gold),
            "7" => try_buy_armour(state, ArmourTier::Crystal),
            "8" => try_buy_armour(state, ArmourTier::Sacred),
            _ => continue,
        }
    }
}

fn armour_shop_page_2(state: &mut GameState) {
    loop {
        clear_screen();
        println!("FinalRealm Armour Shop - Page 2\n");
        println!("You currently have {} Gold.\n", state.player.gold);

        let armours = [
            ArmourTier::Runic,
            ArmourTier::Infernal,
            ArmourTier::Omega,
            ArmourTier::Chaotic,
        ];

        for (i, tier) in armours.iter().enumerate() {
            let owned = state.player.inventory_armour.iter().filter(|&&t| t == *tier).count();
            println!("{}) {:<16} Costs {:<6} gold. Owned: {}", 
                     i + 1, tier.name(), tier.buy_price(), owned);
        }
        println!("5) Previous Page");
        println!("Type \"exit\" to exit the store.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "5" => return,
            "1" => try_buy_armour(state, ArmourTier::Runic),
            "2" => try_buy_armour(state, ArmourTier::Infernal),
            "3" => try_buy_armour(state, ArmourTier::Omega),
            "4" => try_buy_armour(state, ArmourTier::Chaotic),
            _ => continue,
        }
    }
}

fn try_buy_armour(state: &mut GameState, tier: ArmourTier) {
    if state.player.combat_level < tier.level_required() {
        println!("\nYou are not a high enough level to buy this item. You need to be at least level {}.", tier.level_required());
        crate::ui::wait_for_enter();
        return;
    }
    if state.player.gold < tier.buy_price() {
        println!("\nYou don't have enough money.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.gold -= tier.buy_price();
    state.player.inventory_armour.push(tier);
    println!("\n{} purchased. You have a damage reduction bonus of up to {} with this item.", tier.name(), tier.max_dr());
    crate::ui::wait_for_enter();
}

pub fn legends_store(state: &mut GameState) {
    loop {
        clear_screen();
        println!("FinalRealm Legend's Store\n");
        println!("You currently have {} gold.\n", state.player.gold);
        println!("You currently have {} Magic Orbs.\n", state.player.get_item_count(ItemType::MagicOrbs));

        println!("1) Mythical Godsword:  Costs 50 Magic Orbs.  Owned: {}", 
                 state.player.inventory_weapons.iter().filter(|&&t| t == WeaponTier::Mythical).count());
        println!("2) Alydril Godsword:   Costs 100 Magic Orbs. Owned: {}", 
                 state.player.inventory_weapons.iter().filter(|&&t| t == WeaponTier::Alydril).count());
        println!("3) Mythical Armour:    Costs 65 Magic Orbs.  Owned: {}", 
                 state.player.inventory_armour.iter().filter(|&&t| t == ArmourTier::Mythical).count());
        println!("4) Alydril Armour:     Costs 120 Magic Orbs. Owned: {}", 
                 state.player.inventory_armour.iter().filter(|&&t| t == ArmourTier::Alydril).count());
        println!("Type \"exit\" to exit the store.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "1" => try_buy_legend_weapon(state, WeaponTier::Mythical, 50),
            "2" => try_buy_legend_weapon(state, WeaponTier::Alydril, 100),
            "3" => try_buy_legend_armour(state, ArmourTier::Mythical, 65),
            "4" => try_buy_legend_armour(state, ArmourTier::Alydril, 120),
            _ => continue,
        }
    }
}

fn try_buy_legend_weapon(state: &mut GameState, tier: WeaponTier, cost: u32) {
    if state.player.get_item_count(ItemType::MagicOrbs) < cost {
        println!("\nYou don't have enough Magic Orbs.");
        crate::ui::wait_for_enter();
        return;
    }
    if state.player.inventory_weapons.iter().any(|&t| t == tier) {
        println!("\nYou already own this item.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.remove_item(ItemType::MagicOrbs, cost);
    state.player.inventory_weapons.push(tier);
    println!("\n{} purchased. You have a damage bonus of up to {} with this item.", tier.name(), tier.max_damage());
    crate::ui::wait_for_enter();
}

fn try_buy_legend_armour(state: &mut GameState, tier: ArmourTier, cost: u32) {
    if state.player.get_item_count(ItemType::MagicOrbs) < cost {
        println!("\nYou don't have enough Magic Orbs.");
        crate::ui::wait_for_enter();
        return;
    }
    if state.player.inventory_armour.iter().any(|&t| t == tier) {
        println!("\nYou already own this item.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.remove_item(ItemType::MagicOrbs, cost);
    state.player.inventory_armour.push(tier);
    println!("\n{} purchased. You have a damage reduction bonus of up to {} with this item.", tier.name(), tier.max_dr());
    crate::ui::wait_for_enter();
}

pub fn smuggler(state: &mut GameState) {
    loop {
        clear_screen();
        println!("Smuggler:\n");
        println!("Hello {}, what can I do for you?\n", state.player.username);
        println!("1) Who are you?");
        println!("2) What do you have for trade?");
        println!("3) Walk away\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                clear_screen();
                println!("{}:\n\nWho are you?\n", state.player.username);
                std::thread::sleep(std::time::Duration::from_millis(500));
                println!("Smuggler:\n\nI am the Smuggler. I will buy your swords and armour off of you for a small amount.\n");
                crate::ui::wait_for_enter();
            }
            "2" => {
                sell_to_smuggler(state);
            }
            "3" => return,
            _ => continue,
        }
    }
}

fn sell_to_smuggler(state: &mut GameState) {
    loop {
        clear_screen();
        println!("You currently have {} swords.", state.player.inventory_weapons.len());
        println!("You currently have a {} equipped.", state.player.weapon.name());
        println!("You currently have {} pieces of Armour.", state.player.inventory_armour.len());
        println!("You currently have {} equipped.\n", state.player.armour.name());
        println!("Select an item to sell, or type \"back\" to go back.\n");

        for (i, tier) in state.player.inventory_weapons.iter().enumerate() {
            println!("s{}) {}", i + 1, tier.name());
        }
        for (i, tier) in state.player.inventory_armour.iter().enumerate() {
            println!("a{}) {}", i + 1, tier.name());
        }
        println!("\nType \"back\" to return.");

        let choice = crate::ui::get_input("Choice: ");
        if choice == "back" {
            return;
        }

        if let Some(stripped) = choice.strip_prefix('s') {
            if let Ok(idx) = stripped.parse::<usize>() {
                if idx > 0 && idx <= state.player.inventory_weapons.len() {
                    let tier = state.player.inventory_weapons[idx - 1];
                    let price = tier.sell_price();
                    println!("\nAh, the {}. That's a nice piece of metal. Tell ya what.", tier.name());
                    println!("I'll buy it off of you for {} gold. Deal?\n", price);
                    println!("1) Deal.\n2) No thanks.\n");
                    let deal = crate::ui::get_input("Choice: ");
                    if deal == "1" {
                        state.player.inventory_weapons.remove(idx - 1);
                        state.player.gold += price;
                        println!("\nYou sell the {} for {} gold.", tier.name(), price);
                        crate::ui::wait_for_enter();
                        continue;
                    }
                }
            }
        } else if let Some(stripped) = choice.strip_prefix('a') {
            if let Ok(idx) = stripped.parse::<usize>() {
                if idx > 0 && idx <= state.player.inventory_armour.len() {
                    let tier = state.player.inventory_armour[idx - 1];
                    let price = tier.sell_price();
                    println!("\nAh, the {}. That's a nice piece of metal. Tell ya what.", tier.name());
                    println!("I'll buy it off of you for {} gold. Deal?\n", price);
                    println!("1) Deal.\n2) No thanks.\n");
                    let deal = crate::ui::get_input("Choice: ");
                    if deal == "1" {
                        state.player.inventory_armour.remove(idx - 1);
                        state.player.gold += price;
                        println!("\nYou sell the {} for {} gold.", tier.name(), price);
                        crate::ui::wait_for_enter();
                        continue;
                    }
                }
            }
        }
        
        println!("\nYou don't have that item.");
        crate::ui::wait_for_enter();
    }
}

pub fn max_shop(state: &mut GameState) {
    // Max's Shop requires total level 1300 in original, but max achievable is 37 + 6*100 = 637.
    // We'll lower the requirement to 600 to make it actually reachable as a remaster feature.
    if state.player.skills.total_level() + state.player.combat_level < 600 {
        clear_screen();
        println!("\nMax's shop is closed at the moment. (Requires combined level 600)\n");
        crate::ui::wait_for_enter();
        return;
    }

    loop {
        clear_screen();
        println!("FinalRealm - Max's Shop\n");
        println!("What would you like to do?\n");
        println!("1) Buy Weapons");
        println!("2) Buy Armour");
        println!("3) Leave\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => max_shop_weapons(state),
            "2" => max_shop_armour(state),
            "3" => return,
            _ => continue,
        }
    }
}

fn max_shop_weapons(state: &mut GameState) {
    loop {
        clear_screen();
        println!("Gold: {}\n", state.player.gold);
        println!("Weapons:\n");
        println!("1) Elysian Blade       3,000,000 gold");
        println!("2) Arcane Blade          400,000 gold");
        println!("3) Spectral Blade      5,000,000 gold\n");
        println!("Type \"exit\" to exit.\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "exit" => return,
            "1" => try_buy_max_weapon(state, WeaponTier::Elysian, 3_000_000),
            "2" => try_buy_max_weapon(state, WeaponTier::Arcane, 400_000),
            "3" => try_buy_max_weapon(state, WeaponTier::Spectral, 5_000_000),
            _ => continue,
        }
    }
}

fn try_buy_max_weapon(state: &mut GameState, tier: WeaponTier, price: u32) {
    if state.player.gold < price {
        println!("\nYou don't have enough money.");
        crate::ui::wait_for_enter();
        return;
    }
    if state.player.inventory_weapons.iter().any(|&t| t == tier) {
        println!("\nYou already own this item.");
        crate::ui::wait_for_enter();
        return;
    }
    
    state.player.gold -= price;
    state.player.inventory_weapons.push(tier);
    println!("\n{} purchased. You have a damage bonus of up to {} with this item.", tier.name(), tier.max_damage());
    crate::ui::wait_for_enter();
}

fn max_shop_armour(state: &mut GameState) {
    clear_screen();
    println!("Max currently only sells legendary weapons. Armour is not available.\n");
    crate::ui::wait_for_enter();
}