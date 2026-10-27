use crate::combat::{run_engine_battle, BattleOutcome};
use crate::data::{ArmourTier, NpcType, WeaponTier, MAX_GOLD, MAX_PLAYER_LEVEL};
use crate::event::GameEvent;
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;

pub fn stronghold_warning(tui: &mut Tui, state: &GameState) -> io::Result<bool> {
    tui.show_message(
        "Warning · The Stronghold",
        "THE STRONGHOLD IS VERY DANGEROUS.\n\nYou will lose your progress if you are defeated. You cannot flee or access your inventory during battle, so make sure you are ready.",
        Some(state),
    )?;
    let choice = tui.select_menu(
        "Enter the Stronghold?",
        &["I am ready for battle!", "Return to safety"],
        Some(state),
    )?;
    Ok(choice == 0)
}

/// Preserve the ten-victory Stronghold run while using the shared combat engine
/// for its warrior encounters and the Lacoyx boss.
pub fn stronghold_dungeon(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.stronghold_kill_count >= 10 {
        return boss_fight(tui, state);
    }

    let mut rng = rand::thread_rng();
    let (npc_type, level) = match rng.gen_range(1..=3) {
        1 => (NpcType::StrongholdMage, 110),
        2 => (NpcType::StrongholdRanger, 111),
        _ => (NpcType::StrongholdWarrior, 108),
    };
    let health = npc_type.base_health();

    match run_engine_battle(
        tui,
        state,
        npc_type.name(),
        level,
        health,
        npc_type.max_damage(),
        false,
        false,
    )? {
        BattleOutcome::Victory => {
            let gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
            state.player.gold = state.player.gold.saturating_add(gold_gain).min(MAX_GOLD);
            state.player.stronghold_kill_count =
                state.player.stronghold_kill_count.saturating_add(1);
            crate::quests::notify_event(
                state,
                &GameEvent::MonsterDefeated {
                    name: npc_type.name().to_string(),
                    is_stronghold: true,
                },
            );

            let healing = state.player.stronghold_kill_count.is_multiple_of(2);
            if healing {
                state.player.hp = state.player.max_hp;
            }

            let mut result = format!(
                "You defeated the {} and gained {} gold.\nStronghold progress: {}/10.",
                npc_type.name(),
                gold_gain,
                state.player.stronghold_kill_count
            );
            if healing {
                result.push_str("\n\nA wave of healing energy restores your health.");
            }
            if state.player.stronghold_kill_count >= 10 {
                result.push_str("\n\nLacoyx awaits in the throne room.");
            }
            tui.show_message("Stronghold victory", &result, Some(state))
        }
        BattleOutcome::Defeated => {
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            tui.show_message(
                "Stronghold progress lost",
                "Your run has been reset. Your lifetime monster count is preserved.",
                Some(state),
            )
        }
        BattleOutcome::Fled => {
            // Flee is disabled for this encounter; keep a defensive fallback.
            Ok(())
        }
    }
}

fn boss_fight(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    state.player.hp = state.player.max_hp;
    let npc_type = NpcType::Lacoyx;
    let level = 741;
    let health = npc_type.base_health();

    tui.show_message(
        "Lacoyx's Throne Room",
        &format!(
            "You enter Lacoyx's throne room. The level {level} War Commander has {health} health.\n\nHe shifts tactics as his health falls. Watch for telegraphed attacks and choose how to defend.",
        ),
        Some(state),
    )?;

    match run_engine_battle(
        tui,
        state,
        npc_type.name(),
        level,
        health,
        npc_type.max_damage(),
        false,
        false,
    )? {
        BattleOutcome::Victory => {
            crate::quests::notify_event(
                state,
                &GameEvent::MonsterDefeated {
                    name: npc_type.name().to_string(),
                    is_stronghold: true,
                },
            );
            state.player.inventory_weapons.push(WeaponTier::Omega);
            state.player.inventory_armour.push(ArmourTier::Runic);
            state.player.stronghold_kill_count = 0;

            let bonus_xp = npc_type.base_xp();
            state.player.xp = state.player.xp.saturating_add(bonus_xp).min(1_000_000);
            state.player.xp_until_next = state.player.xp_until_next.saturating_sub(bonus_xp);
            if state.player.xp_until_next == 0
                && state.player.combat_level < MAX_PLAYER_LEVEL
            {
                state.player.level_up_combat();
                let new_level = state.player.combat_level;
                crate::quests::notify_event(
                    state,
                    &GameEvent::PlayerLeveledUp { new_level },
                );
            }
            state.player.hp = state.player.max_hp;

            tui.show_message(
                "Lacoyx defeated",
                "You won the battle against Lacoyx!\n\nRewards:\n• Omega Maul\n• Runic Armour\n• 5,000 Experience\n\nYour health has been restored and Stronghold progress has reset.",
                Some(state),
            )
        }
        BattleOutcome::Defeated => {
            state.player.stronghold_kill_count = 0;
            state.player.hp = state.player.max_hp;
            tui.show_message(
                "Lacoyx defeated you",
                "Your Stronghold progress has been reset. Your lifetime monster count is preserved.",
                Some(state),
            )
        }
        BattleOutcome::Fled => Ok(()),
    }
}
