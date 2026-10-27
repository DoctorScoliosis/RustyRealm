use crate::state::GameState;
use crate::ui::Tui;
use std::io;

const MAX_ACCOUNT_GOLD: u32 = 1_000_000_000;

pub fn bank_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let summary = (0..state.player.bank_accounts.len())
            .map(|i| {
                let account = state.player.bank_accounts[i];
                format!(
                    "Account {} · {} · {} gold",
                    i + 1,
                    if account.is_closed { "Closed" } else { "Open" },
                    account.gold
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!(
            "RealmBank accounts can hold up to 1,000,000,000 gold each.\nYour wallet: {} gold\n\n{}",
            state.player.gold, summary
        );
        tui.show_text("RealmBank", &content, Some(state))?;

        let mut options = vec![
            "Account 1", "Account 2", "Account 3", "Account 4", "Account 5", "Exit bank",
        ];
        let choice = tui.select_menu("Select an account", &options, Some(state))?;
        if choice >= 5 {
            return Ok(());
        }
        if state.player.bank_accounts[choice].is_closed {
            tui.show_message("Account unavailable", "This account is full and closed.", Some(state))?;
            continue;
        }
        deposit_withdraw(tui, state, choice)?;
    }
}

fn deposit_withdraw(tui: &mut Tui, state: &mut GameState, acc_idx: usize) -> io::Result<()> {
    loop {
        let acc = state.player.bank_accounts[acc_idx];
        let choice = tui.select_menu(
            &format!("Account {} · {} gold", acc_idx + 1, acc.gold),
            &["Deposit gold", "Withdraw gold", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let amount_text = tui.prompt_input("Amount to deposit, or all", false, Some(state))?;
                let to_deposit = if amount_text.eq_ignore_ascii_case("all") {
                    state.player.gold
                } else {
                    match amount_text.parse::<u32>() {
                        Ok(amount) => amount,
                        Err(_) => {
                            tui.show_message("Invalid amount", "Enter a positive number or all.", Some(state))?;
                            continue;
                        }
                    }
                };
                if to_deposit == 0 {
                    tui.show_message("Nothing to deposit", "You have no gold to deposit.", Some(state))?;
                    continue;
                }
                if to_deposit > state.player.gold {
                    tui.show_message("Deposit failed", "You do not have enough gold.", Some(state))?;
                    continue;
                }

                let current_gold = state.player.bank_accounts[acc_idx].gold;
                let new_bank_gold = current_gold.saturating_add(to_deposit);
                if new_bank_gold >= MAX_ACCOUNT_GOLD {
                    let overflow = new_bank_gold.saturating_sub(MAX_ACCOUNT_GOLD);
                    state.player.bank_accounts[acc_idx].gold = MAX_ACCOUNT_GOLD;
                    state.player.bank_accounts[acc_idx].is_closed = true;
                    state.player.gold = state.player.gold.saturating_add(overflow).saturating_sub(to_deposit);
                    tui.show_message(
                        "Account is full",
                        &format!("The account is now full and closed. {} gold was returned to you.", overflow),
                        Some(state),
                    )?;
                } else {
                    state.player.bank_accounts[acc_idx].gold = new_bank_gold;
                    state.player.gold = state.player.gold.saturating_sub(to_deposit);
                    tui.show_message("Deposit complete", &format!("Deposited {} gold.", to_deposit), Some(state))?;
                }
            }
            1 => {
                let amount_text = tui.prompt_input("Amount to withdraw, or all", false, Some(state))?;
                let available = state.player.bank_accounts[acc_idx].gold;
                let to_withdraw = if amount_text.eq_ignore_ascii_case("all") {
                    available
                } else {
                    match amount_text.parse::<u32>() {
                        Ok(amount) => amount,
                        Err(_) => {
                            tui.show_message("Invalid amount", "Enter a positive number or all.", Some(state))?;
                            continue;
                        }
                    }
                };
                if to_withdraw == 0 {
                    tui.show_message("Nothing to withdraw", "The account is empty or the amount is zero.", Some(state))?;
                    continue;
                }
                if to_withdraw > available {
                    tui.show_message("Withdrawal failed", "The account does not have enough gold.", Some(state))?;
                    continue;
                }

                state.player.gold = state.player.gold.saturating_add(to_withdraw);
                state.player.bank_accounts[acc_idx].gold =
                    state.player.bank_accounts[acc_idx].gold.saturating_sub(to_withdraw);
                if state.player.bank_accounts[acc_idx].gold < MAX_ACCOUNT_GOLD {
                    state.player.bank_accounts[acc_idx].is_closed = false;
                }
                tui.show_message("Withdrawal complete", &format!("Withdrew {} gold.", to_withdraw), Some(state))?;
            }
            _ => return Ok(()),
        }
    }
}
