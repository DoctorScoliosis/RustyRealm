use crate::data::{ArmourTier, ItemType, WeaponTier, MAX_GOLD};
use crate::state::GameState;
use crate::ui::Tui;
use std::io;

pub fn weapon_shop(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let weapons = [
        WeaponTier::Wooden,
        WeaponTier::Stone,
        WeaponTier::Bronze,
        WeaponTier::Iron,
        WeaponTier::Steel,
        WeaponTier::Gold,
        WeaponTier::Crystal,
        WeaponTier::Sacred,
    ];
    loop {
        let labels: Vec<String> = weapons
            .iter()
            .map(|tier| {
                let owned = state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&item| item == *tier)
                    .count();
                format!(
                    "{} · {} gold · level {} · owned {}",
                    tier.name(),
                    tier.buy_price(),
                    tier.level_required(),
                    owned
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Next page");
        options.push("Exit shop");
        let choice = tui.select_menu(
            &format!("Weapon Shop · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        match choice {
            0..=7 => try_buy_weapon(tui, state, weapons[choice])?,
            8 => weapon_shop_page_2(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn weapon_shop_page_2(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let weapons = [
        WeaponTier::Runic,
        WeaponTier::Infernal,
        WeaponTier::Omega,
        WeaponTier::Chaotic,
    ];
    loop {
        let labels: Vec<String> = weapons
            .iter()
            .map(|tier| {
                let owned = state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&item| item == *tier)
                    .count();
                format!(
                    "{} · {} gold · level {} · owned {}",
                    tier.name(),
                    tier.buy_price(),
                    tier.level_required(),
                    owned
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Previous page");
        options.push("Exit shop");
        let choice = tui.select_menu(
            &format!("Weapon Shop · Page 2 · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        match choice {
            0..=3 => try_buy_weapon(tui, state, weapons[choice])?,
            4 => return Ok(()),
            _ => return Ok(()),
        }
    }
}

fn try_buy_weapon(tui: &mut Tui, state: &mut GameState, tier: WeaponTier) -> io::Result<()> {
    if state.player.combat_level < tier.level_required() {
        tui.show_message(
            "Level requirement not met",
            &format!(
                "You need to be at least level {} to buy this item.",
                tier.level_required()
            ),
            Some(state),
        )?;
        return Ok(());
    }
    if state.player.gold < tier.buy_price() {
        tui.show_message(
            "Purchase failed",
            "You do not have enough money.",
            Some(state),
        )?;
        return Ok(());
    }
    state.player.gold -= tier.buy_price();
    state.player.inventory_weapons.push(tier);
    tui.show_message(
        "Weapon purchased",
        &format!(
            "{} purchased. Its maximum damage roll is {}.",
            tier.name(),
            tier.max_damage()
        ),
        Some(state),
    )
}

pub fn armour_shop(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let armours = [
        ArmourTier::Cloth,
        ArmourTier::Chain,
        ArmourTier::Bronze,
        ArmourTier::Iron,
        ArmourTier::Steel,
        ArmourTier::Gold,
        ArmourTier::Crystal,
        ArmourTier::Sacred,
    ];
    loop {
        let labels: Vec<String> = armours
            .iter()
            .map(|tier| {
                let owned = state
                    .player
                    .inventory_armour
                    .iter()
                    .filter(|&&item| item == *tier)
                    .count();
                format!(
                    "{} · {} gold · level {} · owned {}",
                    tier.name(),
                    tier.buy_price(),
                    tier.level_required(),
                    owned
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Next page");
        options.push("Exit shop");
        let choice = tui.select_menu(
            &format!("Armour Shop · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        match choice {
            0..=7 => try_buy_armour(tui, state, armours[choice])?,
            8 => armour_shop_page_2(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn armour_shop_page_2(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let armours = [
        ArmourTier::Runic,
        ArmourTier::Infernal,
        ArmourTier::Omega,
        ArmourTier::Chaotic,
    ];
    loop {
        let labels: Vec<String> = armours
            .iter()
            .map(|tier| {
                let owned = state
                    .player
                    .inventory_armour
                    .iter()
                    .filter(|&&item| item == *tier)
                    .count();
                format!(
                    "{} · {} gold · level {} · owned {}",
                    tier.name(),
                    tier.buy_price(),
                    tier.level_required(),
                    owned
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Previous page");
        options.push("Exit shop");
        let choice = tui.select_menu(
            &format!("Armour Shop · Page 2 · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        match choice {
            0..=3 => try_buy_armour(tui, state, armours[choice])?,
            4 => return Ok(()),
            _ => return Ok(()),
        }
    }
}

fn try_buy_armour(tui: &mut Tui, state: &mut GameState, tier: ArmourTier) -> io::Result<()> {
    if state.player.combat_level < tier.level_required() {
        tui.show_message(
            "Level requirement not met",
            &format!(
                "You need to be at least level {} to buy this item.",
                tier.level_required()
            ),
            Some(state),
        )?;
        return Ok(());
    }
    if state.player.gold < tier.buy_price() {
        tui.show_message(
            "Purchase failed",
            "You do not have enough money.",
            Some(state),
        )?;
        return Ok(());
    }
    state.player.gold -= tier.buy_price();
    state.player.inventory_armour.push(tier);
    tui.show_message(
        "Armour purchased",
        &format!(
            "{} purchased. Its maximum damage reduction roll is {}.",
            tier.name(),
            tier.max_dr()
        ),
        Some(state),
    )
}

pub fn legends_store(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let options = [
            format!(
                "Mythical Godsword · 50 Magic Orbs · owned {}",
                state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&tier| tier == WeaponTier::Mythical)
                    .count()
            ),
            format!(
                "Alydril Godsword · 100 Magic Orbs · owned {}",
                state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&tier| tier == WeaponTier::Alydril)
                    .count()
            ),
            format!(
                "Mythical Armour · 65 Magic Orbs · owned {}",
                state
                    .player
                    .inventory_armour
                    .iter()
                    .filter(|&&tier| tier == ArmourTier::Mythical)
                    .count()
            ),
            format!(
                "Alydril Armour · 120 Magic Orbs · owned {}",
                state
                    .player
                    .inventory_armour
                    .iter()
                    .filter(|&&tier| tier == ArmourTier::Alydril)
                    .count()
            ),
        ];
        let option_refs: Vec<&str> = options
            .iter()
            .map(String::as_str)
            .chain(std::iter::once("Exit shop"))
            .collect();
        let choice = tui.select_menu(
            &format!(
                "Legend's Store · {} gold · {} Magic Orbs",
                state.player.gold,
                state.player.get_item_count(ItemType::MagicOrbs)
            ),
            &option_refs,
            Some(state),
        )?;
        match choice {
            0 => try_buy_legend_weapon(tui, state, WeaponTier::Mythical, 50)?,
            1 => try_buy_legend_weapon(tui, state, WeaponTier::Alydril, 100)?,
            2 => try_buy_legend_armour(tui, state, ArmourTier::Mythical, 65)?,
            3 => try_buy_legend_armour(tui, state, ArmourTier::Alydril, 120)?,
            _ => return Ok(()),
        }
    }
}

fn try_buy_legend_weapon(
    tui: &mut Tui,
    state: &mut GameState,
    tier: WeaponTier,
    cost: u32,
) -> io::Result<()> {
    if state.player.get_item_count(ItemType::MagicOrbs) < cost {
        tui.show_message(
            "Purchase failed",
            "You do not have enough Magic Orbs.",
            Some(state),
        )?;
        return Ok(());
    }
    if state.player.inventory_weapons.contains(&tier) {
        tui.show_message("Already owned", "You already own this item.", Some(state))?;
        return Ok(());
    }
    state.player.remove_item(ItemType::MagicOrbs, cost);
    state.player.inventory_weapons.push(tier);
    tui.show_message(
        "Weapon purchased",
        &format!(
            "{} purchased. Its maximum damage roll is {}.",
            tier.name(),
            tier.max_damage()
        ),
        Some(state),
    )
}

fn try_buy_legend_armour(
    tui: &mut Tui,
    state: &mut GameState,
    tier: ArmourTier,
    cost: u32,
) -> io::Result<()> {
    if state.player.get_item_count(ItemType::MagicOrbs) < cost {
        tui.show_message(
            "Purchase failed",
            "You do not have enough Magic Orbs.",
            Some(state),
        )?;
        return Ok(());
    }
    if state.player.inventory_armour.contains(&tier) {
        tui.show_message("Already owned", "You already own this item.", Some(state))?;
        return Ok(());
    }
    state.player.remove_item(ItemType::MagicOrbs, cost);
    state.player.inventory_armour.push(tier);
    tui.show_message(
        "Armour purchased",
        &format!(
            "{} purchased. Its maximum damage reduction roll is {}.",
            tier.name(),
            tier.max_dr()
        ),
        Some(state),
    )
}

pub fn smuggler(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice = tui.select_menu(
            &format!("Smuggler · {} gold", state.player.gold),
            &["Who are you?", "Sell weapons or armour", "Walk away"],
            Some(state),
        )?;
        match choice {
            0 => tui.show_message(
                "Smuggler",
                "I am the Smuggler. I will buy your swords and armour off of you for a small amount.",
                Some(state),
            )?,
            1 => sell_to_smuggler(tui, state)?,
            _ => return Ok(()),
        }
    }
}

#[derive(Clone, Copy)]
enum SellChoice {
    Weapon(usize),
    Armour(usize),
}

fn sell_to_smuggler(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let mut labels = Vec::new();
        let mut choices = Vec::new();
        for (index, tier) in state.player.inventory_weapons.iter().enumerate() {
            labels.push(format!(
                "Weapon s{} · {} · sell for {} gold",
                index + 1,
                tier.name(),
                crate::economy::calculate_dynamic_equipment_sell_price(state, tier.sell_price())
            ));
            choices.push(SellChoice::Weapon(index));
        }
        for (index, tier) in state.player.inventory_armour.iter().enumerate() {
            labels.push(format!(
                "Armour a{} · {} · sell for {} gold",
                index + 1,
                tier.name(),
                crate::economy::calculate_dynamic_equipment_sell_price(state, tier.sell_price())
            ));
            choices.push(SellChoice::Armour(index));
        }
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!(
                "Smuggler · weapons {} · armour {} · equipped: {} / {}",
                state.player.inventory_weapons.len(),
                state.player.inventory_armour.len(),
                state.player.weapon.name(),
                state.player.armour.name(),
            ),
            &options,
            Some(state),
        )?;
        let Some(selected) = choices.get(choice).copied() else {
            return Ok(());
        };

        let (name, price) = match selected {
            SellChoice::Weapon(index) => {
                let tier = state.player.inventory_weapons[index];
                (
                    tier.name().to_string(),
                    crate::economy::calculate_dynamic_equipment_sell_price(state, tier.sell_price()),
                )
            }
            SellChoice::Armour(index) => {
                let tier = state.player.inventory_armour[index];
                (
                    tier.name().to_string(),
                    crate::economy::calculate_dynamic_equipment_sell_price(state, tier.sell_price()),
                )
            }
        };
        let confirmation = tui.select_menu(
            &format!("The {} can be sold for {} gold. Deal?", name, price),
            &["Deal", "No thanks"],
            Some(state),
        )?;
        if confirmation != 0 {
            continue;
        }

        if u64::from(state.player.gold) + u64::from(price) > u64::from(MAX_GOLD) {
            tui.show_message(
                "Sale unavailable",
                "Your gold balance is too close to its limit to accept this sale. Spend some gold first.",
                Some(state),
            )?;
            continue;
        }

        match selected {
            SellChoice::Weapon(index) => {
                state.player.inventory_weapons.remove(index);
            }
            SellChoice::Armour(index) => {
                state.player.inventory_armour.remove(index);
            }
        }
        state.player.gold += price;
        state.market_state.record_equipment_trade(-1);
        tui.show_message(
            "Sale complete",
            &format!("You sold the {} for {} gold.", name, price),
            Some(state),
        )?;
    }
}

pub fn max_shop(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    // The remaster lowers the original 1300 requirement to the achievable total level of 600.
    if state.player.skills.total_level() + state.player.combat_level < 600 {
        tui.show_message(
            "Max's Shop is closed",
            "This shop requires a combined combat and skill level of 600.",
            Some(state),
        )?;
        return Ok(());
    }

    loop {
        let choice = tui.select_menu(
            "Max's Shop",
            &["Buy weapons", "Armour (not available)", "Leave"],
            Some(state),
        )?;
        match choice {
            0 => max_shop_weapons(tui, state)?,
            1 => max_shop_armour(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn max_shop_weapons(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let options = [
            format!(
                "Elysian Blade · 3,000,000 gold · owned {}",
                state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&tier| tier == WeaponTier::Elysian)
                    .count()
            ),
            format!(
                "Arcane Blade · 400,000 gold · owned {}",
                state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&tier| tier == WeaponTier::Arcane)
                    .count()
            ),
            format!(
                "Spectral Blade · 5,000,000 gold · owned {}",
                state
                    .player
                    .inventory_weapons
                    .iter()
                    .filter(|&&tier| tier == WeaponTier::Spectral)
                    .count()
            ),
        ];
        let option_refs: Vec<&str> = options
            .iter()
            .map(String::as_str)
            .chain(std::iter::once("Back"))
            .collect();
        let choice = tui.select_menu(
            &format!("Max's Shop · {} gold", state.player.gold),
            &option_refs,
            Some(state),
        )?;
        match choice {
            0 => try_buy_max_weapon(tui, state, WeaponTier::Elysian, 3_000_000)?,
            1 => try_buy_max_weapon(tui, state, WeaponTier::Arcane, 400_000)?,
            2 => try_buy_max_weapon(tui, state, WeaponTier::Spectral, 5_000_000)?,
            _ => return Ok(()),
        }
    }
}

fn try_buy_max_weapon(
    tui: &mut Tui,
    state: &mut GameState,
    tier: WeaponTier,
    price: u32,
) -> io::Result<()> {
    if state.player.gold < price {
        tui.show_message(
            "Purchase failed",
            "You do not have enough money.",
            Some(state),
        )?;
        return Ok(());
    }
    if state.player.inventory_weapons.contains(&tier) {
        tui.show_message("Already owned", "You already own this item.", Some(state))?;
        return Ok(());
    }
    state.player.gold -= price;
    state.player.inventory_weapons.push(tier);
    tui.show_message(
        "Weapon purchased",
        &format!(
            "{} purchased. Its maximum damage roll is {}.",
            tier.name(),
            tier.max_damage()
        ),
        Some(state),
    )
}

fn max_shop_armour(tui: &mut Tui, state: &GameState) -> io::Result<()> {
    tui.show_message(
        "Armour unavailable",
        "Max currently only sells legendary weapons. Armour is not available.",
        Some(state),
    )
}
