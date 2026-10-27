use crate::data::{ArmourTier, NpcType, WeaponTier};
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;

pub fn stronghold_warning(tui: &mut Tui, state: &GameState) -> io::Result<bool> {
    tui.show_message(
        "Warning · The Stronghold",
        "THE STRONGHOLD IS VERY DANGEROUS.\n\nYou will lose all your items if you are not properly prepared. You will not be able to flee or access your inventory during battle, so make sure you are ready.",
        Some(state),
    )?;
    let choice = tui.select_menu(
        "Enter the Stronghold?",
        &["I am ready for battle!", "Return to safety"],
        Some(state),
    )?;
    Ok(choice == 0)
}

pub fn stronghold_dungeon(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.stronghold_kill_count >= 10 {
        return boss_fight(tui, state);
    }

    let mut rng = rand::thread_rng();
    let npc_roll = rng.gen_range(1..=3);
    let (npc_type, level) = match npc_roll {
        1 => (NpcType::StrongholdMage, 110),
        2 => (NpcType::StrongholdRanger, 111),
        3 => (NpcType::StrongholdWarrior, 108),
        _ => (NpcType::StrongholdMage, 110),
    };
    let health = npc_type.base_health();
    let mut current_health = health;

    tui.show_message(
        "The Stronghold",
        &format!("You continue into the Stronghold and encounter a level {} {} with {} health.", level, npc_type.name(), health),
        Some(state),
    )?;

    while current_health > 0 && state.player.hp > 0 {
        let max_dmg = state.player.weapon.max_damage();
        let mut damage = rand::thread_rng().gen_range(1..=max_dmg);
        if damage > current_health {
            damage = current_health;
        }

        let atk_xp = (damage / 4) * 3;
        state.player.xp += atk_xp;
        state.player.xp_until_next = state.player.xp_until_next.saturating_sub(atk_xp);
        current_health = current_health.saturating_sub(damage);

        tui.show_message(
            "Stronghold · Player attack",
            &format!("You deal {} damage to the {}.\nIt has {} health left.\nYou gain {} EXP.\nStronghold victories: {}/10",
                damage, npc_type.name(), current_health, atk_xp, state.player.stronghold_kill_count),
            Some(state),
        )?;

        if state.player.xp_until_next == 0 {
            state.player.level_up_combat();
            tui.show_message(
                "Combat level up",
                &format!("You are now combat level {} and have {} health.", state.player.combat_level, state.player.hp),
                Some(state),
            )?;
        }

        if current_health == 0 {
            break;
        }

        let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
        let dr = state.player.armour.max_dr();
        let dr_roll = rand::thread_rng().gen_range(0..=dr);
        npc_dmg = npc_dmg.saturating_sub(dr_roll);
        if npc_dmg == 0 {
            npc_dmg = 0;
        }
        state.player.hp = state.player.hp.saturating_sub(npc_dmg);

        if state.player.hp == 0 {
            tui.show_message("Defeated", "You died. Your Stronghold progress has been reset.", Some(state))?;
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            return Ok(());
        }

        tui.show_message(
            "Stronghold · Enemy attack",
            &format!("The {} deals {} damage to you.\nYou have {} health left.", npc_type.name(), npc_dmg, state.player.hp),
            Some(state),
        )?;
    }

    let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    state.player.gold = (state.player.gold + gold_gain).min(5_000_000);
    state.player.stronghold_kill_count += 1;

    let healing = state.player.stronghold_kill_count.is_multiple_of(2);
    if healing {
        state.player.hp = state.player.max_hp;
    }

    let mut result = format!(
        "You defeated the {} and gained {} gold.\nStronghold progress: {}/10.",
        npc_type.name(), gold_gain, state.player.stronghold_kill_count
    );
    if healing {
        result.push_str("\n\nA wave of healing energy restores your health.");
    }
    if state.player.stronghold_kill_count >= 10 {
        result.push_str("\n\nLacoyx awaits in the throne room.");
    }
    tui.show_message("Stronghold victory", &result, Some(state))
}

fn boss_fight(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    state.player.hp = state.player.max_hp;
    let npc_type = NpcType::Lacoyx;
    let level = 741;
    let mut health = npc_type.base_health();
    let xp = npc_type.base_xp();

    tui.show_message(
        "Lacoyx's Throne Room",
        &format!("You enter Lacoyx's throne room. The level {} War Commander has {} health.", level, health),
        Some(state),
    )?;

    while health > 0 && state.player.hp > 0 {
        let max_dmg = state.player.weapon.max_damage();
        let mut damage = rand::thread_rng().gen_range(1..=max_dmg);
        if damage > health {
            damage = health;
        }

        let atk_xp = (damage / 4) * 3;
        state.player.xp += atk_xp;
        state.player.xp_until_next = state.player.xp_until_next.saturating_sub(atk_xp);
        health = health.saturating_sub(damage);

        tui.show_message(
            "Lacoyx · Player attack",
            &format!("You deal {} damage to Lacoyx.\nLacoyx has {} health left.\nYou gain {} EXP.", damage, health, atk_xp),
            Some(state),
        )?;

        if state.player.combat_level >= 37 {
            state.player.xp_until_next = 1;
        }
        if state.player.xp >= 1_000_000 {
            state.player.xp = 1_000_000;
        }

        if state.player.xp_until_next == 0 {
            state.player.level_up_combat();
            tui.show_message(
                "Combat level up",
                &format!("You are now combat level {}.", state.player.combat_level),
                Some(state),
            )?;
        }
        if health == 0 {
            break;
        }

        let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
        let dr = state.player.armour.max_dr();
        let dr_roll = rand::thread_rng().gen_range(0..=dr);
        npc_dmg = npc_dmg.saturating_sub(dr_roll);
        if npc_dmg == 0 {
            npc_dmg = 0;
        }
        state.player.hp = state.player.hp.saturating_sub(npc_dmg);

        if state.player.hp == 0 {
            tui.show_message("Defeated", "You died. Your Stronghold progress has been reset.", Some(state))?;
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            return Ok(());
        }

        tui.show_message(
            "Lacoyx attacks",
            &format!("Lacoyx deals {} damage to you.\nYou have {} health left.", npc_dmg, state.player.hp),
            Some(state),
        )?;
    }

    state.player.inventory_weapons.push(WeaponTier::Omega);
    state.player.inventory_armour.push(ArmourTier::Runic);
    state.player.stronghold_kill_count = 0;
    state.player.xp += xp;
    state.player.xp_until_next = state.player.xp_until_next.saturating_sub(xp);
    state.player.hp = state.player.max_hp;

    tui.show_message(
        "Lacoyx defeated",
        "You won the battle against Lacoyx!\n\nRewards:\n• Omega Maul\n• Runic Armour\n• 5,000 Experience\n\nYour health has been restored and Stronghold progress has reset.",
        Some(state),
    )
}
