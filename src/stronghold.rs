use crate::data::{ArmourTier, NpcType, WeaponTier, MAX_GOLD, MAX_PLAYER_LEVEL};
use crate::event::GameEvent;
use crate::party_battle::{run_party_battle, BattleConfig, BattleOutcome};
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GauntletModifier {
    None,
    Enraged,
    Fortified,
    ResourceDrain,
}
impl GauntletModifier {
    pub fn description(self) -> &'static str {
        match self {
            Self::None => "Standard Encounter",
            Self::Enraged => "Enraged: +25% enemy damage",
            Self::Fortified => "Fortified: +40% enemy health",
            Self::ResourceDrain => "Resource Drain: -5 Stamina/Mana per turn",
        }
    }
}
fn bounded(v: f64) -> u32 {
    if !v.is_finite() || v >= u32::MAX as f64 {
        u32::MAX
    } else if v <= 1.0 {
        1
    } else {
        v.round() as u32
    }
}
pub fn calculate_gauntlet_enemy_stats(w: u32, m: GauntletModifier) -> (u32, u32) {
    let w = f64::from(w);
    let mut h = 100.0 * 1.15_f64.powf(w);
    let mut d = 20.0 * 1.12_f64.powf(w);
    match m {
        GauntletModifier::Fortified => h *= 1.4,
        GauntletModifier::Enraged => d *= 1.25,
        _ => {}
    }
    (bounded(h), bounded(d))
}
pub fn calculate_gauntlet_rewards(w: u32) -> (u32, u32) {
    let r = bounded(100.0 * (1.0 + f64::from(w) * 0.25));
    (r, r)
}
pub fn enter_stronghold(tui: &mut Tui, s: &mut GameState) -> io::Result<()> {
    loop {
        match tui.select_menu(
            "The Stronghold",
            &[
                "Classic Story Mode · 10 guardians and Lacoyx",
                "Endless Gauntlet",
                "View Endless High Score",
                "Back",
            ],
            Some(s),
        )? {
            0 => {
                if stronghold_warning(tui, s)? {
                    stronghold_dungeon(tui, s)?
                }
            }
            1 => {
                tui.show_message("Endless Gauntlet","Survive increasingly difficult waves. Retire after a victory or continue; defeat ends the run. Inventory and fleeing are disabled.",Some(s))?;
                if tui.select_menu(
                    "Enter the gauntlet?",
                    &["Begin", "Return to safety"],
                    Some(s),
                )? == 0
                {
                    run_endless(tui, s)?
                }
            }
            2 => tui.show_message(
                "Endless Gauntlet Record",
                &format!(
                    "Best wave cleared: {}.\nReach wave 10 to unlock Obsidian Shadow.",
                    s.stronghold_high_score
                ),
                Some(s),
            )?,
            _ => return Ok(()),
        }
    }
}
fn award_xp(s: &mut GameState, x: u32) {
    s.player.xp = s.player.xp.saturating_add(x).min(1_000_000);
    s.player.xp_until_next = s.player.xp_until_next.saturating_sub(x);
    if s.player.xp_until_next == 0 && s.player.combat_level < MAX_PLAYER_LEVEL {
        s.player.level_up_combat();
        let n = s.player.combat_level;
        crate::quests::notify_event(s, &GameEvent::PlayerLeveledUp { new_level: n })
    }
}
fn run_endless(tui: &mut Tui, s: &mut GameState) -> io::Result<()> {
    let mut w = 1_u32;
    let mut rng = rand::thread_rng();
    loop {
        let m = if w < 3 {
            GauntletModifier::None
        } else {
            match rng.gen_range(0..3) {
                0 => GauntletModifier::Enraged,
                1 => GauntletModifier::Fortified,
                _ => GauntletModifier::ResourceDrain,
            }
        };
        let (h, d) = calculate_gauntlet_enemy_stats(w, m);
        let drain = if m == GauntletModifier::ResourceDrain {
            5
        } else {
            0
        };
        let name = format!("Gauntlet Fiend · Wave {w}");
        tui.show_message(
            &format!("Gauntlet Wave {w}"),
            &format!(
                "Modifier: {}\nHealth: {h}\nMaximum attack: {d}",
                m.description()
            ),
            Some(s),
        )?;
        match run_party_battle(
            tui,
            s,
            BattleConfig {
                enemy_name: &name,
                level: w.saturating_mul(10).max(1),
                enemy_hp: h,
                enemy_attack_power: d,
                allow_inventory: false,
                allow_flee: false,
                resource_drain_per_turn: drain,
            },
        )? {
            BattleOutcome::Victory => {
                s.stronghold_high_score = s.stronghold_high_score.max(w);
                let (g, x) = calculate_gauntlet_rewards(w);
                let before = s.player.gold;
                s.player.gold = s.player.gold.saturating_add(g).min(MAX_GOLD);
                let actual = s.player.gold.saturating_sub(before);
                award_xp(s, x);
                crate::quests::notify_event(
                    s,
                    &GameEvent::MonsterDefeated {
                        name: name.clone(),
                        is_stronghold: true,
                    },
                );
                tui.show_message("Wave cleared",&format!("Cleared wave {w}.\nReward: {actual} gold and {x} combat XP.\nRecord: wave {}.",s.stronghold_high_score),Some(s))?;
                if tui.select_menu("Continue?", &["Next wave", "Retire with rewards"], Some(s))?
                    == 0
                {
                    w = w.saturating_add(1)
                } else {
                    return Ok(());
                }
            }
            BattleOutcome::Defeated => {
                tui.show_message(
                    "Gauntlet over",
                    &format!(
                        "Defeated on wave {w}. Best: wave {}.",
                        s.stronghold_high_score
                    ),
                    Some(s),
                )?;
                return Ok(());
            }
            BattleOutcome::Fled => return Ok(()),
        }
    }
}

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

fn stronghold_guardian_from_roll(roll: u8) -> (NpcType, u32) {
    match roll {
        1 => (NpcType::StrongholdMage, 110),
        2 => (NpcType::StrongholdRanger, 111),
        _ => (NpcType::StrongholdWarrior, 108),
    }
}

/// Preserve the ten-victory Stronghold run while using the shared combat engine
/// for its warrior encounters and the Lacoyx boss.
pub fn stronghold_dungeon(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.stronghold_kill_count >= 10 {
        return boss_fight(tui, state);
    }

    let mut rng = rand::thread_rng();
    let (npc_type, level) = stronghold_guardian_from_roll(rng.gen_range(1..=3));
    let health = npc_type.base_health();

    match run_party_battle(
        tui,
        state,
        BattleConfig {
            enemy_name: npc_type.name(),
            level,
            enemy_hp: health,
            enemy_attack_power: npc_type.max_damage(),
            allow_inventory: false,
            allow_flee: false,
            resource_drain_per_turn: 0,
        },
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

    match run_party_battle(
        tui,
        state,
        BattleConfig {
            enemy_name: npc_type.name(),
            level,
            enemy_hp: health,
            enemy_attack_power: npc_type.max_damage(),
            allow_inventory: false,
            allow_flee: false,
            resource_drain_per_turn: 0,
        },
    )? {
        BattleOutcome::Victory => {
            crate::quests::notify_event(
                state,
                &GameEvent::MonsterDefeated {
                    name: npc_type.name().to_string(),
                    is_stronghold: true,
                },
            );
            state.lacoyx_defeated = true;
            state.player.inventory_weapons.push(WeaponTier::Omega);
            state.player.inventory_armour.push(ArmourTier::Runic);
            state.player.stronghold_kill_count = 0;

            let bonus_xp = npc_type.base_xp();
            state.player.xp = state.player.xp.saturating_add(bonus_xp).min(1_000_000);
            state.player.xp_until_next = state.player.xp_until_next.saturating_sub(bonus_xp);
            if state.player.xp_until_next == 0 && state.player.combat_level < MAX_PLAYER_LEVEL {
                state.player.level_up_combat();
                let new_level = state.player.combat_level;
                crate::quests::notify_event(state, &GameEvent::PlayerLeveledUp { new_level });
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

#[cfg(test)]
mod gauntlet_tests {
    use super::*;
    #[test]
    fn classic_stronghold_keeps_its_three_guardian_choices_and_levels() {
        assert_eq!(
            stronghold_guardian_from_roll(1),
            (NpcType::StrongholdMage, 110)
        );
        assert_eq!(
            stronghold_guardian_from_roll(2),
            (NpcType::StrongholdRanger, 111)
        );
        assert_eq!(
            stronghold_guardian_from_roll(3),
            (NpcType::StrongholdWarrior, 108)
        );
    }

    #[test]
    fn modifiers_change_only_their_stat() {
        let (h, d) = calculate_gauntlet_enemy_stats(3, GauntletModifier::None);
        let (hf, df) = calculate_gauntlet_enemy_stats(3, GauntletModifier::Fortified);
        let (he, de) = calculate_gauntlet_enemy_stats(3, GauntletModifier::Enraged);
        assert!(hf > h);
        assert_eq!(df, d);
        assert_eq!(he, h);
        assert!(de > d)
    }
    #[test]
    fn reward_multiplier_is_quarter_per_wave() {
        assert_eq!(calculate_gauntlet_rewards(1), (125, 125));
        assert_eq!(calculate_gauntlet_rewards(4), (200, 200))
    }
    #[test]
    fn scaling_saturates() {
        let (h, d) = calculate_gauntlet_enemy_stats(u32::MAX, GauntletModifier::Fortified);
        assert_eq!(h, u32::MAX);
        assert_eq!(d, u32::MAX)
    }
}
