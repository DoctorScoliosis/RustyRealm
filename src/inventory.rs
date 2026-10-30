use crate::data::{ArmourTier, ItemType, WeaponTier};
use crate::state::GameState;
use crate::ui::Tui;
use std::io;

pub fn view_inventory(tui: &mut Tui, state: &mut GameState, in_combat: bool) -> io::Result<()> {
    state.player.cap_values();

    loop {
        let xp_until = if state.player.combat_level >= crate::data::MAX_PLAYER_LEVEL {
            "None!".to_string()
        } else {
            state.player.xp_until_next.to_string()
        };
        let mut lines = vec![
            format!("Health: {}/{}", state.player.hp, state.player.max_hp),
            format!("Combat level: {}", state.player.combat_level),
            format!("Class: {}", state.player.class.name()),
            format!(
                "Attributes: STR {} · DEX {} · CON {} · INT {} · WIS {} · CHA {}",
                state.player.attributes.strength,
                state.player.attributes.dexterity,
                state.player.attributes.constitution,
                state.player.attributes.intelligence,
                state.player.attributes.wisdom,
                state.player.attributes.charisma
            ),
            format!(
                "Weapon proficiency: {} (+{}% damage)",
                state.player.weapon.family().name(),
                state.player.class.weapon_proficiency_pct(state.player.weapon.family())
            ),
            format!("Gold: {}", state.player.gold),
            format!("EXP: {}", state.player.xp),
            format!("EXP until next level: {}", xp_until),
            format!(
                "Total level: {}",
                state.player.skills.total_level() + state.player.combat_level
            ),
            format!(
                "Weapon: [{}] {}",
                state.player.weapon.rarity(),
                state.player.weapon.name()
            ),
            format!(
                "Armour: [{}] {}",
                state.player.armour.rarity(),
                state.player.armour.name()
            ),
            String::new(),
            "Items".to_string(),
        ];

        let item_types = [
            ItemType::Bait,
            ItemType::Gemstones,
            ItemType::Seeds,
            ItemType::MagicOrbs,
            ItemType::Fur,
            ItemType::RawFood,
            ItemType::Ore,
            ItemType::HugeBones,
            ItemType::Logs,
            ItemType::DragonHides,
            ItemType::Food,
            ItemType::RunicTablets,
            ItemType::Ingots,
            ItemType::GoblinMail,
            ItemType::Potions,
            ItemType::Bows,
            ItemType::Thread,
            ItemType::GourmetFeast,
        ];
        for item in item_types {
            lines.push(format!(
                "{:<18} {}",
                item.name(),
                state.player.get_item_count(item)
            ));
        }

        lines.push(String::new());
        lines.push("Weapons".to_string());
        let weapon_tiers = [
            WeaponTier::Wooden,
            WeaponTier::Stone,
            WeaponTier::Bronze,
            WeaponTier::Iron,
            WeaponTier::Steel,
            WeaponTier::Gold,
            WeaponTier::Crystal,
            WeaponTier::Sacred,
            WeaponTier::Runic,
            WeaponTier::Infernal,
            WeaponTier::Omega,
            WeaponTier::Chaotic,
            WeaponTier::Mythical,
            WeaponTier::Alydril,
        ];
        for tier in weapon_tiers {
            let count = state
                .player
                .inventory_weapons
                .iter()
                .filter(|&&owned| owned == tier)
                .count();
            lines.push(format!("[{}] {:<24} {}", tier.rarity(), tier.name(), count));
        }

        lines.push(String::new());
        lines.push("Armour".to_string());
        let armour_tiers = [
            ArmourTier::Cloth,
            ArmourTier::Chain,
            ArmourTier::Bronze,
            ArmourTier::Iron,
            ArmourTier::Steel,
            ArmourTier::Gold,
            ArmourTier::Crystal,
            ArmourTier::Sacred,
            ArmourTier::Runic,
            ArmourTier::Infernal,
            ArmourTier::Omega,
            ArmourTier::Chaotic,
            ArmourTier::Mythical,
            ArmourTier::Alydril,
        ];
        for tier in armour_tiers {
            let count = state
                .player
                .inventory_armour
                .iter()
                .filter(|&&owned| owned == tier)
                .count();
            lines.push(format!("[{}] {:<24} {}", tier.rarity(), tier.name(), count));
        }
        tui.show_text("Final Realm · Inventory", &lines.join("\n"), Some(state))?;

        let options = if in_combat {
            [
                "Return to battle",
                "Drink potion",
                "Eat food",
                "Change weapon",
                "Change armour",
            ]
        } else {
            [
                "Back",
                "Heal",
                "Change weapon",
                "Change armour",
                "View levels",
            ]
        };
        let choice = tui.select_menu("Inventory actions", &options, Some(state))?;
        if in_combat {
            match choice {
                0 => return Ok(()),
                1 => drink_potion(tui, state)?,
                2 => eat_food(tui, state)?,
                3 => change_weapon(tui, state)?,
                4 => change_armour(tui, state)?,
                _ => {}
            }
        } else {
            match choice {
                0 => return Ok(()),
                1 => {
                    state.player.hp = state.player.max_hp;
                    tui.show_message(
                        "Healed",
                        "You have been healed back to full health.",
                        Some(state),
                    )?;
                }
                2 => change_weapon(tui, state)?,
                3 => change_armour(tui, state)?,
                4 => view_levels(tui, state)?,
                _ => {}
            }
        }
    }
}

fn drink_potion(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.get_item_count(ItemType::Potions) == 0 {
        tui.show_message("No potions", "You do not have any potions.", Some(state))?;
        return Ok(());
    }

    state.player.remove_item(ItemType::Potions, 1);
    let heal = rand::random::<u32>() % 100 + 1;
    state.player.hp = (state.player.hp + heal).min(state.player.max_hp);
    tui.show_message(
        "Potion used",
        &format!(
            "You drink a potion and heal {} health.\nYou now have {}/{} health.",
            heal, state.player.hp, state.player.max_hp
        ),
        Some(state),
    )
}

fn eat_food(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.get_item_count(ItemType::Food) == 0 {
        tui.show_message("No food", "You do not have any food.", Some(state))?;
        return Ok(());
    }

    state.player.remove_item(ItemType::Food, 1);
    let heal = rand::random::<u32>() % 50 + 1;
    state.player.hp = (state.player.hp + heal).min(state.player.max_hp);
    tui.show_message(
        "Food eaten",
        &format!(
            "You eat some food and heal {} health.\nYou now have {}/{} health.",
            heal, state.player.hp, state.player.max_hp
        ),
        Some(state),
    )
}

fn change_weapon(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.inventory_weapons.is_empty() {
        tui.show_message(
            "No weapons",
            "You have no weapons in your inventory.",
            Some(state),
        )?;
        return Ok(());
    }

    loop {
        let labels: Vec<String> = state
            .player
            .inventory_weapons
            .iter()
            .enumerate()
            .map(|(index, tier)| format!("{} · [{}] {}", index + 1, tier.rarity(), tier.name()))
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Weapons · equipped: {}", state.player.weapon.name()),
            &options,
            Some(state),
        )?;
        if choice >= state.player.inventory_weapons.len() {
            return Ok(());
        }
        let new_weapon = state.player.inventory_weapons[choice];
        if state.player.weapon == new_weapon {
            tui.show_message(
                "Already equipped",
                "You already have that weapon equipped.",
                Some(state),
            )?;
            continue;
        }

        state.player.weapon = new_weapon;
        tui.show_message(
            "Weapon equipped",
            &format!("You have equipped {}.", new_weapon.name()),
            Some(state),
        )?;
        return Ok(());
    }
}

fn change_armour(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    if state.player.inventory_armour.is_empty() {
        tui.show_message(
            "No armour",
            "You have no armour in your inventory.",
            Some(state),
        )?;
        return Ok(());
    }

    loop {
        let labels: Vec<String> = state
            .player
            .inventory_armour
            .iter()
            .enumerate()
            .map(|(index, tier)| format!("{} · [{}] {}", index + 1, tier.rarity(), tier.name()))
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Armour · equipped: {}", state.player.armour.name()),
            &options,
            Some(state),
        )?;
        if choice >= state.player.inventory_armour.len() {
            return Ok(());
        }
        let new_armour = state.player.inventory_armour[choice];
        if state.player.armour == new_armour {
            tui.show_message(
                "Already equipped",
                "You already have that armour equipped.",
                Some(state),
            )?;
            continue;
        }

        state.player.armour = new_armour;
        tui.show_message(
            "Armour equipped",
            &format!("You have equipped {}.", new_armour.name()),
            Some(state),
        )?;
        return Ok(());
    }
}

fn view_levels(tui: &mut Tui, state: &GameState) -> io::Result<()> {
    let details = format!(
        "Total level: {}\nCombat level: {}\nWoodcutting level: {}\nCooking level: {}\nFishing level: {}\nThieving level: {}\nMining level: {}\nSmithing level: {}",
        state.player.skills.total_level() + state.player.combat_level,
        state.player.combat_level,
        state.player.skills.woodcutting.level,
        state.player.skills.cooking.level,
        state.player.skills.fishing.level,
        state.player.skills.thieving.level,
        state.player.skills.mining.level,
        state.player.skills.smithing.level,
    );
    tui.show_text("Levels", &details, Some(state))
}
