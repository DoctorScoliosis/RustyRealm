use crate::state::GameState;
use crate::ui::{clear_screen, wait_for_enter};

pub fn bank_menu(state: &mut GameState) {
    loop {
        clear_screen();
        println!("Welcome to RealmBank! Here are your accounts which you can store up to 1,000,000,000 (1B) gold into each.\n");
        println!("You have {} gold.\n", state.player.gold);
        
        for i in 0..5 {
            let acc = &state.player.bank_accounts[i];
            let status = if acc.is_closed { "Closed" } else { "Open" };
            println!("Account {}: {}", i + 1, status);
            println!("   - Gold: {}\n", acc.gold);
        }
        
        println!("Type \"exit\" to exit.\n");
        
        let choice = crate::ui::get_input("Access account number (1-5) or \"exit\": ");
        if choice.to_lowercase() == "exit" {
            return;
        }
        
        if let Ok(acc_num) = choice.parse::<usize>() {
            if acc_num >= 1 && acc_num <= 5 {
                let idx = acc_num - 1;
                if state.player.bank_accounts[idx].is_closed {
                    println!("\nThis account is closed (full).");
                    crate::ui::wait_for_enter();
                    continue;
                }
                deposit_withdraw(state, idx);
            } else {
                println!("\nInvalid account number.");
                crate::ui::wait_for_enter();
            }
        } else {
            println!("\nInvalid input.");
            crate::ui::wait_for_enter();
        }
    }
}

fn deposit_withdraw(state: &mut GameState, acc_idx: usize) {
    loop {
        clear_screen();
        let acc = &state.player.bank_accounts[acc_idx];
        println!("Account {} ({} gold, {})\n", acc_idx + 1, acc.gold, if acc.is_closed { "Closed" } else { "Open" });
        println!("1) Deposit");
        println!("2) Withdraw");
        println!("3) Back\n");
        
        let choice = crate::ui::get_input("Choice: ");
        match choice.as_str() {
            "1" => {
                let amount_str = crate::ui::get_input("Amount to deposit (or \"all\"): ");
                if amount_str.to_lowercase() == "all" {
                    let to_deposit = state.player.gold;
                    if to_deposit == 0 {
                        println!("\nYou have no gold to deposit.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    
                    let new_bank_gold = acc.gold.saturating_add(to_deposit);
                    if new_bank_gold >= 1_000_000_000 {
                        let overflow = new_bank_gold.saturating_sub(1_000_000_000);
                        state.player.bank_accounts[acc_idx].gold = 1_000_000_000;
                        state.player.bank_accounts[acc_idx].is_closed = true;
                        state.player.gold = state.player.gold.saturating_add(overflow).saturating_sub(to_deposit);
                        println!("\nAccount is now full and closed. {} gold returned to you.", overflow);
                    } else {
                        state.player.bank_accounts[acc_idx].gold = new_bank_gold;
                        state.player.gold = state.player.gold.saturating_sub(to_deposit);
                        println!("\nDeposited {} gold.", to_deposit);
                    }
                    crate::ui::wait_for_enter();
                } else if let Ok(amount) = amount_str.parse::<u32>() {
                    if amount == 0 {
                        continue;
                    }
                    if amount > state.player.gold {
                        println!("\nYou don't have enough gold.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    
                    let new_bank_gold = acc.gold.saturating_add(amount);
                    if new_bank_gold >= 1_000_000_000 {
                        let overflow = new_bank_gold.saturating_sub(1_000_000_000);
                        state.player.bank_accounts[acc_idx].gold = 1_000_000_000;
                        state.player.bank_accounts[acc_idx].is_closed = true;
                        state.player.gold = state.player.gold.saturating_add(overflow).saturating_sub(amount);
                        println!("\nAccount is now full and closed. {} gold returned to you.", overflow);
                    } else {
                        state.player.bank_accounts[acc_idx].gold = new_bank_gold;
                        state.player.gold = state.player.gold.saturating_sub(amount);
                        println!("\nDeposited {} gold.", amount);
                    }
                    crate::ui::wait_for_enter();
                }
            }
            "2" => {
                let amount_str = crate::ui::get_input("Amount to withdraw (or \"all\"): ");
                if amount_str.to_lowercase() == "all" {
                    let to_withdraw = state.player.bank_accounts[acc_idx].gold;
                    if to_withdraw == 0 {
                        println!("\nAccount is empty.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    state.player.gold = state.player.gold.saturating_add(to_withdraw);
                    state.player.bank_accounts[acc_idx].gold = 0;
                    state.player.bank_accounts[acc_idx].is_closed = false;
                    println!("\nWithdrew {} gold.", to_withdraw);
                    crate::ui::wait_for_enter();
                } else if let Ok(amount) = amount_str.parse::<u32>() {
                    if amount == 0 {
                        continue;
                    }
                    if amount > state.player.bank_accounts[acc_idx].gold {
                        println!("\nAccount doesn't have enough gold.");
                        crate::ui::wait_for_enter();
                        continue;
                    }
                    state.player.gold = state.player.gold.saturating_add(amount);
                    state.player.bank_accounts[acc_idx].gold = state.player.bank_accounts[acc_idx].gold.saturating_sub(amount);
                    if state.player.bank_accounts[acc_idx].gold < 1_000_000_000 {
                        state.player.bank_accounts[acc_idx].is_closed = false;
                    }
                    println!("\nWithdrew {} gold.", amount);
                    crate::ui::wait_for_enter();
                }
            }
            "3" => return,
            _ => continue,
        }
    }
}