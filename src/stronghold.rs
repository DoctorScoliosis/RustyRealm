use rand::Rng;
use crate::state::GameState;
use crate::data::{NpcType, WeaponTier, ArmourTier};
use crate::ui::{clear_screen, wait_for_enter};

pub fn stronghold_warning(state: &mut GameState) -> bool {
    clear_screen();
    println!("\nWARNING:\n");
    println!("THE STRONGHOLD IS VERY DANGEROUS. YOU WILL LOSE ALL YOUR ITEMS IF YOU ARE NOT PROPERLY PREPARED.");
    println!("YOU WILL NOT BE ABLE TO FLEE OR ACCESS YOUR INVENTORY DURING BATTLE, SO BE PREPARED WITH GOOD DEFENCES.\n");
    println!("ARE YOU SURE YOU WISH TO CONTINUE?\n");
    println!("1) Yeah, after all, I am an adventurer.");
    println!("2) Maybe I should come back later...\n");

    let choice = crate::ui::get_input("Choice: ");
    if choice == "1" {
        true
    } else {
        false
    }
}

pub fn stronghold_dungeon(state: &mut GameState) {
    if state.player.stronghold_kill_count >= 10 {
        boss_fight(state);
        return;
    }

    let mut rng = rand::thread_rng();
    let npc_roll = rng.gen_range(1..=3);
    
    let (npc_type, level, health, xp) = match npc_roll {
        1 => (NpcType::StrongholdMage, 110, 750, 1000),
        2 => (NpcType::StrongholdRanger, 111, 750, 1000),
        3 => (NpcType::StrongholdWarrior, 108, 750, 1000),
        _ => (NpcType::StrongholdMage, 110, 750, 1000),
    };

    let mut current_health = health;

    clear_screen();
    println!("\nYou continue into the stronghold...\n");
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("You encounter a level {} {}.\n", level, npc_type.name());
    std::thread::sleep(std::time::Duration::from_millis(500));

    while current_health > 0 && state.player.hp > 0 {
        clear_screen();
        println!("┌────────────────────────────────────────────────────────────────┐");
        println!("│ **** Your Health: {:<4} ***** {}'s Health: {:<4} ***** Kill Count: {} **** │", 
                 state.player.hp, npc_type.name(), current_health, state.player.stronghold_kill_count);
        println!("└────────────────────────────────────────────────────────────────┘\n");

        // Player turn
        let max_dmg = state.player.weapon.max_damage();
        let mut damage = rand::thread_rng().gen_range(1..=max_dmg);
        if damage > current_health { damage = current_health; }

        println!("You attack.");
        std::thread::sleep(std::time::Duration::from_millis(300));

        let atk_xp = (damage / 4) * 3;
        state.player.xp += atk_xp;
        state.player.xp_until_next = state.player.xp_until_next.saturating_sub(atk_xp);
        current_health = current_health.saturating_sub(damage);

        println!("You deal {} damage to the {}.", damage, npc_type.name());
        println!("It now has {} health left.", current_health);
        println!("You gain {} EXP.", atk_xp);
        std::thread::sleep(std::time::Duration::from_millis(300));

        if state.player.xp_until_next == 0 {
            state.player.level_up_combat();
            println!("\nYou have leveled up! You are now level {}.", state.player.combat_level);
            println!("You now have {} health.", state.player.hp);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        if current_health == 0 {
            break;
        }

        // NPC turn
        let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
        let dr = state.player.armour.max_dr();
        let dr_roll = rand::thread_rng().gen_range(0..=dr);
        npc_dmg = npc_dmg.saturating_sub(dr_roll);
        if npc_dmg == 0 { npc_dmg = 0; }

        println!("\nThe {} attacks.", npc_type.name());
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("The {} deals {} damage to you.", npc_type.name(), npc_dmg);
        
        state.player.hp = state.player.hp.saturating_sub(npc_dmg);
        
        if state.player.hp == 0 {
            println!("\nYou died. Your stronghold progress has been reset.");
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            crate::ui::wait_for_enter();
            return;
        }
        
        println!("You now have {} health left.", state.player.hp);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Victory
    let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    state.player.gold = (state.player.gold + gold_gain).min(5_000_000);
    
    clear_screen();
    println!("\nYou won the battle against the {}.", npc_type.name());
    println!("You gain {} gold.", gold_gain);
    
    state.player.stronghold_kill_count += 1;
    
    // Heal at milestones
    if state.player.stronghold_kill_count % 2 == 0 {
        state.player.hp = state.player.max_hp;
        println!("You feel a wave of healing energy restore your health.");
    }
    
    crate::ui::wait_for_enter();
}

fn boss_fight(state: &mut GameState) {
    state.player.hp = state.player.max_hp;
    let npc_type = NpcType::Lacoyx;
    let level = 741;
    let mut health = 2780;
    let xp = 5000;

    clear_screen();
    println!("\nYou enter Lacoyx's throne room...\n");
    std::thread::sleep(std::time::Duration::from_millis(500));

    while health > 0 && state.player.hp > 0 {
        clear_screen();
        println!("┌────────────────────────────────────────────────────────────────┐");
        println!("│ **** Your Health: {:<4} ***** {}'s Health: {:<4} **** │", 
                 state.player.hp, npc_type.name(), health);
        println!("└────────────────────────────────────────────────────────────────┘\n");

        println!("You encounter {}, a level {} War Commander.\n", npc_type.name(), level);
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Player turn
        let max_dmg = state.player.weapon.max_damage();
        let mut damage = rand::thread_rng().gen_range(1..=max_dmg);
        if damage > health { damage = health; }

        println!("You attack.");
        std::thread::sleep(std::time::Duration::from_millis(300));

        let atk_xp = (damage / 4) * 3;
        state.player.xp += atk_xp;
        state.player.xp_until_next = state.player.xp_until_next.saturating_sub(atk_xp);
        health = health.saturating_sub(damage);

        println!("You deal {} damage to {}.", damage, npc_type.name());
        println!("It now has {} health left.", health);
        println!("You gain {} EXP.", atk_xp);
        std::thread::sleep(std::time::Duration::from_millis(300));

        if state.player.combat_level >= 37 {
            state.player.xp_until_next = 1;
        }
        if state.player.xp >= 1_000_000 {
            state.player.xp = 1_000_000;
        }

        if state.player.xp_until_next == 0 {
            state.player.level_up_combat();
            println!("\nYou have leveled up! You are now level {}.", state.player.combat_level);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        if health == 0 {
            break;
        }

        // NPC turn
        let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
        let dr = state.player.armour.max_dr();
        let dr_roll = rand::thread_rng().gen_range(0..=dr);
        npc_dmg = npc_dmg.saturating_sub(dr_roll);
        if npc_dmg == 0 { npc_dmg = 0; }

        println!("\n{} attacks.", npc_type.name());
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("{} deals {} damage to you.", npc_type.name(), npc_dmg);
        
        state.player.hp = state.player.hp.saturating_sub(npc_dmg);
        
        if state.player.hp == 0 {
            println!("\nYou died. Your stronghold progress has been reset.");
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            crate::ui::wait_for_enter();
            return;
        }
        
        println!("You now have {} health left.", state.player.hp);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Boss Victory
    clear_screen();
    println!("\nYou won the battle against {}.", npc_type.name());
    println!("You received an Omega Maul.");
    println!("You received Runic Armour.");
    println!("You have gained {} Experience.", xp);
    
    state.player.inventory_weapons.push(WeaponTier::Omega);
    state.player.inventory_armour.push(ArmourTier::Runic);
    state.player.stronghold_kill_count = 0;
    state.player.xp += xp;
    state.player.xp_until_next = state.player.xp_until_next.saturating_sub(xp);
    
    state.player.hp = state.player.max_hp;
    crate::ui::wait_for_enter();
}
