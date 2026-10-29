use crate::data::{ItemType, NpcType, MAX_GOLD};
use crate::event::GameEvent;
use crate::party_battle::{run_party_battle, BattleConfig, BattleOutcome};
use crate::state::GameState;
use crate::ui::Tui;
use rand::Rng;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WanderEncounterKind {
    Merchant,
    EliteMonster,
    TreasurePuzzle,
    StandardBattle,
}

fn encounter_kind_from_roll(roll: u8) -> WanderEncounterKind {
    match roll {
        0..=14 => WanderEncounterKind::Merchant,
        15..=29 => WanderEncounterKind::EliteMonster,
        30..=39 => WanderEncounterKind::TreasurePuzzle,
        _ => WanderEncounterKind::StandardBattle,
    }
}

fn scale_elite_stat(base: u32) -> u32 {
    base.saturating_add(base / 2).saturating_add(base % 2)
}

fn encounter_gold_reward(base_reward: u32, is_elite: bool) -> u32 {
    if is_elite {
        base_reward.saturating_mul(2)
    } else {
        base_reward
    }
}

/// Random encounters are drawn only from explicitly hostile fantasy creatures.
/// Generic civilians and ordinary animals stay out of the combat rotation.
const WANDERING_ENEMIES: [NpcType; 6] = [
    NpcType::Goblin,
    NpcType::Giant,
    NpcType::Dragon,
    NpcType::Rockbeast,
    NpcType::Ent,
    NpcType::Elemental,
];

pub fn wander_gielinor(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let encounter_kind = encounter_kind_from_roll(rng.gen_range(0..100));
    match encounter_kind {
        WanderEncounterKind::Merchant => return encounter_travelling_merchant(tui, state),
        WanderEncounterKind::TreasurePuzzle => return encounter_treasure_puzzle(tui, state),
        WanderEncounterKind::EliteMonster | WanderEncounterKind::StandardBattle => {}
    }

    let is_elite = encounter_kind == WanderEncounterKind::EliteMonster;

    // Only explicitly hostile fantasy creatures appear as random encounters.
    // Civilians and ordinary wildlife are not player combat targets.
    let npc_type = WANDERING_ENEMIES[rng.gen_range(0..WANDERING_ENEMIES.len())];

    let player_level = state.player.combat_level;
    let (base_health, base_level) = if player_level <= 5 {
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

    let health = if is_elite {
        scale_elite_stat(base_health)
    } else {
        base_health
    };
    let level = if is_elite {
        scale_elite_stat(base_level)
    } else {
        base_level
    };
    let attack_power = if is_elite {
        scale_elite_stat(npc_type.max_damage())
    } else {
        npc_type.max_damage()
    };
    let encounter_name = if is_elite {
        format!("Elite {}", npc_type.name())
    } else {
        npc_type.name().to_string()
    };

    match run_party_battle(
        tui,
        state,
        BattleConfig {
            enemy_name: &encounter_name,
            level,
            enemy_hp: health,
            enemy_attack_power: attack_power,
            allow_inventory: true,
            allow_flee: true,
            resource_drain_per_turn: 0,
        },
    )? {
        BattleOutcome::Victory => handle_victory(tui, state, npc_type, is_elite),
        BattleOutcome::Fled => tui.show_message("Fled", "You flee from the battle.", Some(state)),
        BattleOutcome::Defeated => Ok(()),
    }
}

fn encounter_travelling_merchant(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let item = ItemType::Potions;
    let retail_price = crate::economy::calculate_dynamic_buy_price(state, item);
    if retail_price == 0 {
        tui.show_message(
            "Travelling Merchant",
            "The merchant has no tradeable goods to offer today.",
            Some(state),
        )?;
        return Ok(());
    }
    let discounted_price = crate::mastery::vendor_price(state, retail_price, 25).clamp(1, MAX_GOLD);

    tui.show_message(
        "Travelling Merchant",
        "A travelling merchant offers one health potion at a 25% discount from the current Trading Post price.",
        Some(state),
    )?;
    let options = [
        format!("Buy 1 Health Potion · {discounted_price} gold"),
        "Decline".to_string(),
    ];
    let choice = tui.select_menu(
        "Travelling Merchant",
        &[options[0].as_str(), options[1].as_str()],
        Some(state),
    )?;
    if choice == 0 {
        match crate::economy::buy_item_transaction_at_price(state, item, 1, discounted_price) {
            Ok(cost) => tui.show_message(
                "Purchase complete",
                &format!("Bought one Health Potion for {cost} gold."),
                Some(state),
            )?,
            Err(error) => tui.show_message("Purchase failed", &error.to_string(), Some(state))?,
        }
    }
    Ok(())
}

fn encounter_treasure_puzzle(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    tui.show_message(
        "Ancient Chest",
        "You stumble upon a locked chest with a mechanical dial. Enter the correct combination to open it.",
        Some(state),
    )?;
    let choice = tui.select_menu(
        "Solve Puzzle",
        &["Dial 7-4-1", "Dial 1-2-3", "Leave"],
        Some(state),
    )?;
    match choice {
        0 => {
            let gold_before = state.player.gold;
            let gold_awarded = MAX_GOLD
                .saturating_sub(state.player.gold.min(MAX_GOLD))
                .min(300);
            state.player.gold = state.player.gold.saturating_add(gold_awarded).min(MAX_GOLD);
            let gemstone_awarded = if state
                .player
                .get_item_count(ItemType::Gemstones)
                .checked_add(1)
                .is_some()
            {
                state.player.add_item(ItemType::Gemstones, 1);
                1
            } else {
                0
            };
            let actual_gold_awarded = state.player.gold.saturating_sub(gold_before);
            tui.show_message(
                "Treasure found",
                &format!(
                    "The lock clicks open! You found {actual_gold_awarded} gold and {gemstone_awarded} Gemstone(s)."
                ),
                Some(state),
            )?;
        }
        1 => tui.show_message(
            "Locked",
            "The lock refuses to budge. The combination is incorrect.",
            Some(state),
        )?,
        _ => {}
    }
    Ok(())
}

fn handle_victory(
    tui: &mut Tui,
    state: &mut GameState,
    npc_type: NpcType,
    is_elite: bool,
) -> io::Result<()> {
    let base_gold_gain = rand::thread_rng().gen_range(1..=1000) + 1000;
    let requested_gold_gain = encounter_gold_reward(base_gold_gain, is_elite);
    let gold_before = state.player.gold;
    state.player.gold = state
        .player
        .gold
        .saturating_add(requested_gold_gain)
        .min(MAX_GOLD);
    let gold_gain = state.player.gold.saturating_sub(gold_before);

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
        &GameEvent::ItemCollected {
            item_type: item,
            count: amount,
        },
    );
    let encounter_name = if is_elite {
        format!("Elite {}", npc_type.name())
    } else {
        npc_type.name().to_string()
    };
    crate::quests::notify_event(
        state,
        &GameEvent::MonsterDefeated {
            name: encounter_name.clone(),
            is_stronghold: false,
        },
    );
    state.player.hp = state.player.max_hp;
    let elite_note = if is_elite {
        "\nElite victory: double gold awarded."
    } else {
        ""
    };
    tui.show_message(
        "Victory",
        &format!(
            "You defeated the {}.\nYou received {} {} and gained {} gold.{}\nYour health has been restored.",
            encounter_name, amount, item.name(), gold_gain, elite_note
        ),
        Some(state),
    )
}

#[cfg(test)]
mod encounter_tests {
    use super::*;

    #[test]
    fn wandering_pool_excludes_civilians_and_ordinary_wildlife() {
        assert_eq!(
            WANDERING_ENEMIES,
            [
                NpcType::Goblin,
                NpcType::Giant,
                NpcType::Dragon,
                NpcType::Rockbeast,
                NpcType::Ent,
                NpcType::Elemental,
            ]
        );
        assert!(!WANDERING_ENEMIES.iter().any(|npc| matches!(
            *npc,
            NpcType::Human | NpcType::Farmer | NpcType::Bear | NpcType::Rat | NpcType::Thief
        )));
    }

    #[test]
    fn wandering_encounter_rolls_match_the_documented_probabilities() {
        let mut merchant = 0;
        let mut elite = 0;
        let mut treasure = 0;
        let mut standard = 0;

        for roll in 0_u8..100 {
            match encounter_kind_from_roll(roll) {
                WanderEncounterKind::Merchant => merchant += 1,
                WanderEncounterKind::EliteMonster => elite += 1,
                WanderEncounterKind::TreasurePuzzle => treasure += 1,
                WanderEncounterKind::StandardBattle => standard += 1,
            }
        }

        assert_eq!((merchant, elite, treasure, standard), (15, 15, 10, 60));
    }

    #[test]
    fn elite_stats_are_scaled_to_one_and_a_half_with_safe_rounding() {
        assert_eq!(scale_elite_stat(100), 150);
        assert_eq!(scale_elite_stat(75), 113);
        assert_eq!(scale_elite_stat(u32::MAX), u32::MAX);
    }

    #[test]
    fn elite_victories_double_the_base_gold_reward() {
        assert_eq!(encounter_gold_reward(1250, false), 1250);
        assert_eq!(encounter_gold_reward(1250, true), 2500);
        assert_eq!(encounter_gold_reward(u32::MAX, true), u32::MAX);
    }
}
