use rand::Rng;
use crate::state::GameState;
use crate::data::{NpcType, ItemType};
use crate::ui::{clear_screen, wait_for_enter};

pub fn wander_gielinor(state: &mut GameState) {
    let mut rng = rand::thread_rng();
    let npc_roll = rng.gen_range(1..=13);
    
    let npc_type = match npc_roll {
        1 => NpcType::Goblin,
        2 => NpcType::Giant,
        3 => NpcType::Dragon,
        4 => NpcType::Mage,
        5 => NpcType::Sorcerer,
        6 => NpcType::Human,
        7 => NpcType::Farmer,
        8 => NpcType::Bear,
        9 => NpcType::Rockbeast,
        10 => NpcType::Ent,
        11 => NpcType::Thief,
        12 => NpcType::Elemental,
        13 => NpcType::Rat,
        _ => NpcType::Goblin,
    };

    let player_lvl = state.player.combat_level;
    let (base_health, base_level) = if player_lvl <= 5 {
        (rng.gen_range(75..=175), rng.gen_range(1..=4))
    } else if player_lvl <= 10 {
        (rng.gen_range(200..=300), rng.gen_range(5..=9))
    } else if player_lvl <= 15 {
        (rng.gen_range(350..=450), rng.gen_range(10..=14))
    } else if player_lvl <= 20 {
        (rng.gen_range(500..=600), rng.gen_range(15..=19))
    } else if player_lvl <= 25 {
        (rng.gen_range(625..=725), rng.gen_range(20..=26))
    } else if player_lvl <= 30 {
        (rng.gen_range(750..=850), rng.gen_range(25..=34))
    } else {
        (rng.gen_range(900..=1000), rng.gen_range(30..=44))
    };

    let mut health = base_health;
    let level = base_level;

    clear_screen();
    println!("\nYou are wandering Gielinor...\n");
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("You encounter a level {} {}.\n", level, npc_type.name());
    std::thread::sleep(std::time::Duration::from_millis(500));

    while health > 0 && state.player.hp > 0 {
        clear_screen();
        println!("┌────────────────────────────────────────────────────────────────┐");
        println!("│ **** Your Health: {:<4} ***** {}'s Health: {:<4} **** │", 
                 state.player.hp, npc_type.name(), health);
        println!("└────────────────────────────────────────────────────────────────┘\n");
        println!("What would you like to do?\n");
        println!("1) Attack!");
        println!("2) Inventory");
        println!("3) Flee!\n");

        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                let max_dmg = state.player.weapon.max_damage();
                let mut damage = rand::thread_rng().gen_range(1..=max_dmg);
                if damage > health { damage = health; }

                println!("\nYou attack!");
                std::thread::sleep(std::time::Duration::from_millis(300));

                let atk_xp = (damage / 4) * 3;
                state.player.xp += atk_xp;
                state.player.xp_until_next = state.player.xp_until_next.saturating_sub(atk_xp);
                health = health.saturating_sub(damage);

                println!("You deal {} damage to the {}!", damage, npc_type.name());
                println!("It now has {} health left.", health);
                println!("You gain {} EXP.", atk_xp);
                crate::ui::wait_for_enter();

                if state.player.xp_until_next == 0 {
                    state.player.level_up_combat();
                    println!("\nYou have leveled up! You are now level {}.", state.player.combat_level);
                    println!("You now have {} health.", state.player.hp);
                    crate::ui::wait_for_enter();
                }

                if health == 0 {
                    handle_victory(state, npc_type);
                    return;
                }
            }
            "2" => {
                crate::inventory::view_inventory(state, true);
                continue;
            }
            "3" => {
                println!("\nYou flee from the battle.");
                crate::ui::wait_for_enter();
                return;
            }
            _ => continue,
        }

        // NPC turn
        if health > 0 {
            let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
            let dr = state.player.armour.max_dr();
            let dr_roll = rand::thread_rng().gen_range(0..=dr);
            npc_dmg = npc_dmg.saturating_sub(dr_roll);
            if npc_dmg == 0 { npc_dmg = 0; } // Ensure no negative damage

            println!("\nThe {} attacks.", npc_type.name());
            std::thread::sleep(std::time::Duration::from_millis(300));
            println!("The {} deals {} damage to you.", npc_type.name(), npc_dmg);
            
            state.player.hp = state.player.hp.saturating_sub(npc_dmg);
            
            if state.player.hp == 0 {
                println!("\nYou died.");
                crate::ui::wait_for_enter();
                state.player.hp = state.player.max_hp;
                return;
            }
            
            println!("You now have {} health left.", state.player.hp);
            crate::ui::wait_for_enter();
        }
    }
}

fn handle_victory(state: &mut GameState, npc_type: NpcType) {
    let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    state.player.gold = (state.player.gold + gold_gain).min(5_000_000);
    
    clear_screen();
    println!("\nYou won the battle against the {}.", npc_type.name());
    
    let (item, amount) = match npc_type {
        NpcType::Goblin => (ItemType::GoblinMail, 1),
        NpcType::Giant => (ItemType::HugeBones, 3),
        NpcType::Dragon => (ItemType::DragonHides, 3),
        NpcType::Mage => (ItemType::RunicTablets, 2),
        NpcType::Sorcerer => (ItemType::Potions, 2),
        NpcType::Human => (ItemType::Food, 2),
        NpcType::Farmer => (ItemType::Seeds, 4),
        NpcType::Bear | NpcType::Rat => (ItemType::Fur, 5),
        NpcType::Rockbeast => (ItemType::Ore, 6),
        NpcType::Ent => (ItemType::Logs, 10),
        NpcType::Thief => (ItemType::Gemstones, 5),
        NpcType::Elemental => (ItemType::MagicOrbs, 5),
        _ => (ItemType::Food, 1),
    };
    
    state.player.add_item(item, amount);
    println!("You received {} {}.", amount, item.name());
    println!("You gained {} gold.", gold_gain);
    
    state.player.hp = state.player.max_hp;
    crate::ui::wait_for_enter();
}