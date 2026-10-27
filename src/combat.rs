use crate::combat_engine::{CombatEngine, Combatant, DefensiveChoice};
use crate::data::{AbilityId, DamageType, ItemType, NpcType, MAX_GOLD, MAX_PLAYER_LEVEL};
use crate::event::GameEvent;
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleOutcome {
    Victory,
    Fled,
    Defeated,
}

/// Configuration for a tactical encounter against an existing NPC.
pub struct BattleConfig<'a> {
    pub enemy_name: &'a str,
    pub level: u32,
    pub enemy_hp: u32,
    pub enemy_attack_power: u32,
    pub allow_inventory: bool,
    pub allow_flee: bool,
}

/// Run a tactical encounter against any existing NPC. Encounter rules live in
/// CombatEngine; this function handles the TUI, player resources and save state.
pub fn run_engine_battle(
    tui: &mut Tui,
    state: &mut GameState,
    config: BattleConfig<'_>,
) -> io::Result<BattleOutcome> {
    let BattleConfig {
        enemy_name,
        level,
        enemy_hp,
        enemy_attack_power,
        allow_inventory,
        allow_flee,
    } = config;
    let mut player = Combatant::from_player(&state.player);
    let mut enemy = Combatant::new_monster(enemy_name, level, enemy_hp);
    enemy.attack_power = enemy_attack_power.max(1);
    let mut battle_log = vec![format!(
        "You encounter a level {level} {enemy_name} with {enemy_hp} health."
    )];

    loop {
        let player_logs = player.tick_status_effects();
        battle_log.extend(player_logs);
        let player_stunned = player.is_stunned;
        player.is_stunned = false;

        if enemy.name == "Lacoyx" {
            battle_log.extend(CombatEngine::update_lacoyx_boss_phase(&mut enemy));
        }

        if player.hp == 0 {
            sync_player(state, &player);
            tui.show_message(
                "Defeat",
                "You were defeated and revived at full health.",
                Some(state),
            )?;
            state.player.hp = state.player.max_hp;
            return Ok(BattleOutcome::Defeated);
        }
        if enemy.hp == 0 {
            sync_player(state, &player);
            return Ok(BattleOutcome::Victory);
        }

        // A telegraph is resolved before the next player action, allowing a
        // deliberate defensive response while keeping one enemy turn per round.
        let mut enemy_turn_consumed = false;
        if let Some(attack) = enemy.telegraphed_attack.clone() {
            let enemy_logs = enemy.tick_status_effects();
            battle_log.extend(enemy_logs);
            if enemy.hp == 0 {
                sync_player(state, &player);
                return Ok(BattleOutcome::Victory);
            }

            sync_player(state, &player);
            tui.show_message(
                "DANGER · Incoming attack",
                &format!(
                    "{} is preparing {}!\n{}",
                    enemy.name, attack.name, attack.description
                ),
                Some(state),
            )?;

            let has_brew = state.player.get_item_count(ItemType::Potions) > 0;
            let defensive_options: Vec<&str> = if has_brew {
                vec![
                    "Block · mitigate 75%",
                    "Dodge · 80% chance",
                    "Use brew · heal and mitigate 50%",
                ]
            } else {
                vec!["Block · mitigate 75%", "Dodge · 80% chance"]
            };
            let defence_choice =
                tui.select_menu("Defensive response", &defensive_options, Some(state))?;
            let defence = match defence_choice {
                0 => DefensiveChoice::Block,
                1 => DefensiveChoice::Dodge,
                _ if has_brew => {
                    if state.player.remove_item(ItemType::Potions, 1) {
                        let healing = (player.max_hp / 4).max(1);
                        player.hp = player.hp.saturating_add(healing).min(player.max_hp);
                        battle_log.push(format!("You consume a brew and restore {healing} HP."));
                    }
                    DefensiveChoice::Consumable
                }
                _ => DefensiveChoice::Block,
            };

            let (raw_damage, damage_type, _, logs) =
                CombatEngine::generate_enemy_turn(&mut enemy, &player);
            battle_log.extend(logs);
            let (mitigated, logs) = CombatEngine::calculate_damage(
                raw_damage,
                damage_type,
                &player.name,
                player.shield_wall_duration > 0,
                defence,
            );
            battle_log.extend(logs);
            let armour_reduction = rand::thread_rng().gen_range(0..=state.player.armour.max_dr());
            let damage = mitigated.saturating_sub(armour_reduction);
            player.hp = player.hp.saturating_sub(damage);
            battle_log.push(format!(
                "{} deals {damage} damage to you after mitigation.",
                enemy.name
            ));
            enemy_turn_consumed = true;

            if player.hp == 0 {
                sync_player(state, &player);
                tui.show_message(
                    "Defeat",
                    &format!(
                        "{}'s {} defeated you. You were revived at full health.",
                        enemy.name, attack.name
                    ),
                    Some(state),
                )?;
                state.player.hp = state.player.max_hp;
                return Ok(BattleOutcome::Defeated);
            }
        }

        sync_player(state, &player);
        tui.render_screen(Some(state), |frame, area, _| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(area);

            let enemy_ratio = if enemy.max_hp > 0 {
                (enemy.hp as f64 / enemy.max_hp as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let phase_label = match enemy.lacoyx_phase {
                Some(phase) => format!("{} · ", phase.name()),
                None => String::new(),
            };
            frame.render_widget(
                Gauge::default()
                    .block(
                        Block::default()
                            .title(format!(" {phase_label}{} · Level {} ", enemy.name, level))
                            .borders(Borders::ALL),
                    )
                    .gauge_style(Style::default().fg(Color::Magenta))
                    .ratio(enemy_ratio)
                    .label(format!("{}/{}", enemy.hp, enemy.max_hp)),
                chunks[0],
            );

            let resource_line = format!(
                "HP {}/{}  ·  Stamina {}/{}  ·  Mana {}/{}",
                player.hp,
                player.max_hp,
                player.stamina,
                player.max_stamina,
                player.mana,
                player.max_mana
            );
            frame.render_widget(
                Paragraph::new(resource_line)
                    .block(Block::default().title(" You ").borders(Borders::ALL)),
                chunks[1],
            );

            let visible_logs: Vec<ListItem> = battle_log
                .iter()
                .rev()
                .take(chunks[2].height.saturating_sub(2) as usize)
                .map(|line| ListItem::new(line.as_str()))
                .collect();
            frame.render_widget(
                List::new(visible_logs)
                    .block(Block::default().title(" Battle log ").borders(Borders::ALL)),
                chunks[2],
            );
            frame.render_widget(
                Paragraph::new("Costs: Heavy Strike 20 stamina · Shield Wall 15 · Life Drain 25 mana · Cleave 25"),
                chunks[3],
            );
        })?;

        let mut fled = false;
        let player_acted = if player_stunned {
            battle_log.push("You are stunned and lose this turn!".into());
            true
        } else {
            // Inventory and failed ability attempts do not trigger another
            // status tick. Keep the player within this input loop until an
            // actual action or flee is selected.
            loop {
                let mut options = vec![
                    "Basic attack".to_string(),
                    format!("Heavy Strike · 20 stamina (have {})", player.stamina),
                    format!("Shield Wall · 15 stamina (have {})", player.stamina),
                    format!("Life Drain · 25 mana (have {})", player.mana),
                    format!("Cleave · 25 stamina (have {})", player.stamina),
                ];
                if allow_inventory {
                    options.push("Inventory".into());
                }
                if allow_flee {
                    options.push("Flee".into());
                }
                let refs: Vec<&str> = options.iter().map(String::as_str).collect();
                let choice = tui.select_menu(
                    &format!(
                        "Combat action · {} vs {}",
                        state.player.username, enemy.name
                    ),
                    &refs,
                    Some(state),
                )?;

                if choice == 0 {
                    let base = rand::thread_rng().gen_range(1..=player.attack_power.max(1));
                    let (damage, logs) = CombatEngine::calculate_damage(
                        base,
                        DamageType::Physical,
                        &enemy.name,
                        enemy.shield_wall_duration > 0,
                        DefensiveChoice::None,
                    );
                    battle_log.extend(logs);
                    let enemy_hp_before = enemy.hp;
                    enemy.hp = enemy.hp.saturating_sub(damage);
                    let actual_damage = enemy_hp_before.saturating_sub(enemy.hp);
                    battle_log.push(format!(
                        "You deal {actual_damage} damage to {}.",
                        enemy.name
                    ));
                    award_combat_xp(state, &mut player, actual_damage, tui)?;
                    break true;
                }

                let ability_id = match choice {
                    1 => Some(AbilityId::HeavyStrike),
                    2 => Some(AbilityId::ShieldWall),
                    3 => Some(AbilityId::LifeDrain),
                    4 => Some(AbilityId::Cleave),
                    _ => None,
                };
                if let Some(ability_id) = ability_id {
                    let before = enemy.hp;
                    match CombatEngine::execute_player_ability(&mut player, &mut enemy, ability_id)
                    {
                        Ok(logs) => {
                            battle_log.extend(logs);
                            let damage = before.saturating_sub(enemy.hp);
                            award_combat_xp(state, &mut player, damage, tui)?;
                            break true;
                        }
                        Err(error) => {
                            tui.show_message("Action unavailable", error, Some(state))?;
                            continue;
                        }
                    }
                }

                let inventory_index = 5;
                if allow_inventory && choice == inventory_index {
                    sync_player(state, &player);
                    crate::inventory::view_inventory(tui, state, true)?;
                    // Inventory may consume items or equip a different weapon.
                    player.hp = state.player.hp;
                    player.max_hp = state.player.max_hp;
                    player.stamina = state.player.stamina;
                    player.max_stamina = state.player.max_stamina;
                    player.mana = state.player.mana;
                    player.max_mana = state.player.max_mana;
                    player.attack_power = state.player.weapon.max_damage().max(1);
                    continue;
                }

                if allow_flee && choice == options.len() - 1 {
                    fled = true;
                    break false;
                }
            }
        };

        if fled {
            sync_player(state, &player);
            return Ok(BattleOutcome::Fled);
        }

        if enemy.hp == 0 {
            sync_player(state, &player);
            return Ok(BattleOutcome::Victory);
        }

        if player_acted && !enemy_turn_consumed {
            let enemy_logs = enemy.tick_status_effects();
            battle_log.extend(enemy_logs);

            if enemy.hp == 0 {
                sync_player(state, &player);
                return Ok(BattleOutcome::Victory);
            }

            if enemy.is_stunned {
                enemy.is_stunned = false;
                battle_log.push(format!("{} is stunned and misses its turn.", enemy.name));
            } else {
                let (raw_damage, damage_type, _, logs) =
                    CombatEngine::generate_enemy_turn(&mut enemy, &player);
                battle_log.extend(logs);

                if raw_damage > 0 {
                    let (mitigated, logs) = CombatEngine::calculate_damage(
                        raw_damage,
                        damage_type,
                        &player.name,
                        player.shield_wall_duration > 0,
                        DefensiveChoice::None,
                    );
                    battle_log.extend(logs);
                    let armour_reduction =
                        rand::thread_rng().gen_range(0..=state.player.armour.max_dr());
                    let damage = mitigated.saturating_sub(armour_reduction);
                    player.hp = player.hp.saturating_sub(damage);
                    battle_log.push(format!("{} deals {damage} damage to you.", enemy.name));
                }
            }

            if player.hp == 0 {
                sync_player(state, &player);
                tui.show_message(
                    "Defeat",
                    &format!(
                        "{} defeated you. You were revived at full health.",
                        enemy.name
                    ),
                    Some(state),
                )?;
                state.player.hp = state.player.max_hp;
                return Ok(BattleOutcome::Defeated);
            }
        }

        player.regen_resources();
        sync_player(state, &player);
    }
}

fn sync_player(state: &mut GameState, player: &Combatant) {
    state.player.hp = player.hp;
    state.player.max_hp = player.max_hp;
    state.player.stamina = player.stamina;
    state.player.max_stamina = player.max_stamina;
    state.player.mana = player.mana;
    state.player.max_mana = player.max_mana;
}

fn award_combat_xp(
    state: &mut GameState,
    player: &mut Combatant,
    damage: u32,
    tui: &mut Tui,
) -> io::Result<()> {
    let xp = (damage / 4).saturating_mul(3);
    if xp == 0 {
        return Ok(());
    }
    state.player.xp = state.player.xp.saturating_add(xp).min(1_000_000);
    state.player.xp_until_next = state.player.xp_until_next.saturating_sub(xp);

    if state.player.xp_until_next == 0 && state.player.combat_level < MAX_PLAYER_LEVEL {
        state.player.level_up_combat();
        let new_level = state.player.combat_level;
        crate::quests::notify_event(state, &GameEvent::PlayerLeveledUp { new_level });
        player.hp = state.player.hp;
        player.max_hp = state.player.max_hp;
        tui.show_message(
            "Combat level up",
            &format!(
                "You are now combat level {} and have {} health.",
                state.player.combat_level, state.player.hp
            ),
            Some(state),
        )?;
    }
    Ok(())
}

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

    let player_level = state.player.combat_level;
    let (health, level) = if player_level <= 5 {
        (rng.gen_range(75..=175), rng.gen_range(1..=4))
    } else if player_level <= 10 {
        (rng.gen_range(200..=300), rng.gen_range(5..=9))
    } else if player_level <= 15 {
        (rng.gen_range(350..=450), rng.gen_range(10..=14))
    } else if player_level <= 20 {
        (rng.gen_range(500..=600), rng.gen_range(15..=19))
    } else if player_level <= 25 {
        (rng.gen_range(625..=725), rng.gen_range(20..=26))
    } else if player_level <= 30 {
        (rng.gen_range(750..=850), rng.gen_range(25..=34))
    } else {
        (rng.gen_range(900..=1000), rng.gen_range(30..=44))
    };

    match run_engine_battle(
        tui,
        state,
        BattleConfig {
            enemy_name: npc_type.name(),
            level,
            enemy_hp: health,
            enemy_attack_power: npc_type.max_damage(),
            allow_inventory: true,
            allow_flee: true,
        },
    )? {
        BattleOutcome::Victory => handle_victory(tui, state, npc_type),
        BattleOutcome::Fled => tui.show_message("Fled", "You flee from the battle.", Some(state)),
        BattleOutcome::Defeated => Ok(()),
    }
}

fn handle_victory(tui: &mut Tui, state: &mut GameState, npc_type: NpcType) -> io::Result<()> {
    let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    state.player.gold = state.player.gold.saturating_add(gold_gain).min(MAX_GOLD);

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
    crate::quests::notify_event(
        state,
        &GameEvent::MonsterDefeated {
            name: npc_type.name().to_string(),
            is_stronghold: false,
        },
    );
    state.player.hp = state.player.max_hp;
    tui.show_message(
        "Victory",
        &format!(
            "You defeated the {}.\nYou received {} {} and gained {} gold.\nYour health has been restored.",
            npc_type.name(),
            amount,
            item.name(),
            gold_gain
        ),
        Some(state),
    )
}
