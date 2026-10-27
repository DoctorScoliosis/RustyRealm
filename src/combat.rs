use crate::data::{ItemType, NpcType};
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;

pub fn wander_gielinor(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
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
    let (base_health, level) = if player_lvl <= 5 {
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
    tui.show_message(
        "Wandering Gielinor",
        &format!(
            "You encounter a level {} {} with {} health.",
            level,
            npc_type.name(),
            health
        ),
        Some(state),
    )?;

    while health > 0 && state.player.hp > 0 {
        let choice = tui.select_menu(
            &format!("Battle · {} HP · {} HP", state.player.hp, health),
            &["Attack!", "Inventory", "Flee!"],
            Some(state),
        )?;
        match choice {
            0 => {
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
                    "Attack",
                    &format!(
                        "You deal {} damage to the {}.\nIt has {} health left.\nYou gain {} EXP.",
                        damage,
                        npc_type.name(),
                        health,
                        atk_xp
                    ),
                    Some(state),
                )?;

                if state.player.xp_until_next == 0 {
                    state.player.level_up_combat();
                    tui.show_message(
                        "Combat level up",
                        &format!(
                            "You are now combat level {} and have {} health.",
                            state.player.combat_level, state.player.hp
                        ),
                        Some(state),
                    )?;
                }

                if health == 0 {
                    handle_victory(tui, state, npc_type)?;
                    return Ok(());
                }
            }
            1 => {
                crate::inventory::view_inventory(tui, state, true)?;
                continue;
            }
            _ => {
                tui.show_message("Fled", "You flee from the battle.", Some(state))?;
                return Ok(());
            }
        }

        if health > 0 {
            let mut npc_dmg = rand::thread_rng().gen_range(1..=npc_type.max_damage());
            let dr = state.player.armour.max_dr();
            let dr_roll = rand::thread_rng().gen_range(0..=dr);
            npc_dmg = npc_dmg.saturating_sub(dr_roll);
            if npc_dmg == 0 {
                npc_dmg = 0;
            }

            state.player.hp = state.player.hp.saturating_sub(npc_dmg);
            if state.player.hp == 0 {
                tui.show_message(
                    "Defeated",
                    &format!(
                        "The {} dealt {} damage. You died and were revived at full health.",
                        npc_type.name(),
                        npc_dmg
                    ),
                    Some(state),
                )?;
                state.player.hp = state.player.max_hp;
                return Ok(());
            }

            tui.show_message(
                "Enemy attack",
                &format!(
                    "The {} deals {} damage to you.\nYou have {} health left.",
                    npc_type.name(),
                    npc_dmg,
                    state.player.hp
                ),
                Some(state),
            )?;
        }
    }
    Ok(())
}

fn handle_victory(tui: &mut Tui, state: &mut GameState, npc_type: NpcType) -> io::Result<()> {
    let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    state.player.gold = (state.player.gold + gold_gain).min(5_000_000);

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
    state.player.hp = state.player.max_hp;
    tui.show_message(
        "Victory",
        &format!(
            "You defeated the {}.\nYou received {} {} and gained {} gold.\nYour health has been restored.",
            npc_type.name(), amount, item.name(), gold_gain
        ),
        Some(state),
    )
}
