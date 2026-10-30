use crate::data::{ItemType, MAX_GOLD};
use crate::event::WorldEventType;
use crate::state::GameState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

const PRICE_FLOOR_PERCENT: i32 = 50;
const PRICE_CAP_PERCENT: i32 = 250;
const PRICE_CHANGE_PER_NET_TRADE_PERCENT: i32 = 2;
const PLAYER_SELL_SPREAD_PERCENT: u64 = 60;
const TURNS_PER_VOLATILITY_DECAY: u8 = 5;
const MAX_RECORDED_TRADE_VOLUME: i32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EconomyError {
    InsufficientFunds {
        required: u64,
        available: u32,
    },
    InsufficientInventory {
        item_type: ItemType,
        required: u32,
        available: u32,
    },
    InvalidQuantity(u32),
    UntradeableItem(ItemType),
    InventoryOverflow(ItemType),
    GoldCapacityExceeded {
        balance: u32,
        revenue: u64,
        maximum: u32,
    },
}

impl fmt::Display for EconomyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientFunds {
                required,
                available,
            } => write!(
                f,
                "Insufficient funds: {required} gold required, but you have {available}."
            ),
            Self::InsufficientInventory {
                item_type,
                required,
                available,
            } => write!(
                f,
                "Insufficient inventory: {required} x {} required, but you have {available}.",
                item_type.name()
            ),
            Self::InvalidQuantity(quantity) => {
                write!(f, "Invalid transaction quantity: {quantity}.")
            }
            Self::UntradeableItem(item_type) => {
                write!(f, "{} is not traded at the Trading Post.", item_type.name())
            }
            Self::InventoryOverflow(item_type) => write!(
                f,
                "That purchase would exceed the inventory limit for {}.",
                item_type.name()
            ),
            Self::GoldCapacityExceeded {
                balance,
                revenue,
                maximum,
            } => write!(
                f,
                "This sale would exceed the gold limit. Current balance: {balance}; sale value: {revenue}; maximum: {maximum}."
            ),
        }
    }
}

impl std::error::Error for EconomyError {}

/// Persisted market pressure. Positive volume means players have bought more
/// than they have sold; negative volume means the reverse.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct MarketState {
    #[serde(default)]
    pub net_trades: HashMap<ItemType, i32>,
    #[serde(default)]
    pub global_net_trades: i32,
    /// Elapsed in-game hours since the last market-pressure decay step (0–4).
    #[serde(default)]
    turns_since_decay: u8,
}

impl MarketState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_trade(&mut self, item_type: ItemType, quantity_change: i32) {
        let entry = self.net_trades.entry(item_type).or_insert(0);
        *entry = entry
            .saturating_add(quantity_change)
            .clamp(-MAX_RECORDED_TRADE_VOLUME, MAX_RECORDED_TRADE_VOLUME);
        self.record_global_trade(quantity_change);
    }

    /// Equipment sold to the Smuggler affects the market's overall resale
    /// pressure without pretending that weapons and armour are stackable items.
    pub fn record_equipment_trade(&mut self, quantity_change: i32) {
        self.record_global_trade(quantity_change);
    }

    fn record_global_trade(&mut self, quantity_change: i32) {
        self.global_net_trades = self
            .global_net_trades
            .saturating_add(quantity_change)
            .clamp(-MAX_RECORDED_TRADE_VOLUME, MAX_RECORDED_TRADE_VOLUME);
    }

    /// Recover by one point of pressure. The calendar applies this once per
    /// five elapsed in-game hours, independent of how many menus were opened.
    pub fn decay_volatility(&mut self) {
        for volume in self.net_trades.values_mut() {
            if *volume > 0 {
                *volume -= 1;
            } else if *volume < 0 {
                *volume += 1;
            }
        }
        self.net_trades.retain(|_, volume| *volume != 0);

        if self.global_net_trades > 0 {
            self.global_net_trades -= 1;
        } else if self.global_net_trades < 0 {
            self.global_net_trades += 1;
        }
    }

    /// Advance market pressure for an arbitrary elapsed number of world hours.
    /// Batched decay is equivalent to calling advance_turn once per hour, while
    /// avoiding a potentially enormous loop after a long simulated rest.
    pub fn advance_turns(&mut self, turns: u64) {
        if turns == 0 {
            return;
        }

        let elapsed = u128::from(self.turns_since_decay.min(TURNS_PER_VOLATILITY_DECAY - 1))
            + u128::from(turns);
        let decay_steps = elapsed / u128::from(TURNS_PER_VOLATILITY_DECAY);
        self.turns_since_decay = (elapsed % u128::from(TURNS_PER_VOLATILITY_DECAY)) as u8;
        if decay_steps == 0 {
            return;
        }

        let steps = i32::try_from(decay_steps.min(i32::MAX as u128)).unwrap_or(i32::MAX);
        for volume in self.net_trades.values_mut() {
            if *volume > 0 {
                *volume = volume.saturating_sub(steps).max(0);
            } else if *volume < 0 {
                *volume = volume.saturating_add(steps).min(0);
            }
        }
        self.net_trades.retain(|_, volume| *volume != 0);

        if self.global_net_trades > 0 {
            self.global_net_trades = self.global_net_trades.saturating_sub(steps).max(0);
        } else if self.global_net_trades < 0 {
            self.global_net_trades = self.global_net_trades.saturating_add(steps).min(0);
        }
    }

    pub fn advance_turn(&mut self) {
        self.advance_turns(1);
    }
}

fn now_timestamp() -> u64 {
    Utc::now().timestamp().max(0) as u64
}

fn volume_modifier_percent(net_trades: i32) -> i32 {
    100_i32
        .saturating_add(net_trades.saturating_mul(PRICE_CHANGE_PER_NET_TRADE_PERCENT))
        .clamp(PRICE_FLOOR_PERCENT, PRICE_CAP_PERCENT)
}

fn active_market_multiplier(state: &GameState, item_type: ItemType, timestamp: u64) -> u32 {
    if !state.active_world_event.is_active(timestamp) {
        return 1;
    }

    match &state.active_world_event.event_type {
        WorldEventType::MarketBoom {
            item_type: event_item,
            price_multiplier,
        } if *event_item == item_type => (*price_multiplier).max(1),
        _ => 1,
    }
}

/// Calculate the retail price at an explicit timestamp. This makes world-event
/// pricing deterministic in tests and uses the same active-event rules in play.
pub fn calculate_dynamic_buy_price_at(
    state: &GameState,
    item_type: ItemType,
    timestamp: u64,
) -> u32 {
    let base_price = item_type.buy_price();
    if base_price == 0 {
        return 0;
    }

    let net_trades = state
        .market_state
        .net_trades
        .get(&item_type)
        .copied()
        .unwrap_or(0);
    let volume_percent = volume_modifier_percent(net_trades) as u128;
    let event_multiplier = active_market_multiplier(state, item_type, timestamp) as u128;

    let price = u128::from(base_price)
        .saturating_mul(volume_percent)
        .saturating_mul(event_multiplier)
        / 100;

    price.clamp(1, u128::from(MAX_GOLD)) as u32
}

pub fn calculate_dynamic_buy_price(state: &GameState, item_type: ItemType) -> u32 {
    calculate_dynamic_buy_price_at(state, item_type, now_timestamp())
}

pub fn calculate_dynamic_sell_price_at(
    state: &GameState,
    item_type: ItemType,
    timestamp: u64,
) -> u32 {
    let retail_price = calculate_dynamic_buy_price_at(state, item_type, timestamp);
    if retail_price == 0 {
        return 0;
    }

    (u64::from(retail_price) * PLAYER_SELL_SPREAD_PERCENT / 100)
        .max(1)
        .min(u64::from(MAX_GOLD)) as u32
}

pub fn calculate_dynamic_sell_price(state: &GameState, item_type: ItemType) -> u32 {
    calculate_dynamic_sell_price_at(state, item_type, now_timestamp())
}

/// The Smuggler's equipment prices follow the broad market pressure generated by
/// item trading, while preserving each weapon or armour tier's existing base
/// resale value.
pub fn calculate_dynamic_equipment_sell_price(
    state: &GameState,
    base_sell_price: u32,
    base_buy_price: u32,
) -> u32 {
    if base_sell_price == 0 {
        return 0;
    }
    let volume_percent = volume_modifier_percent(state.market_state.global_net_trades) as u128;
    let dynamic_price = u128::from(base_sell_price).saturating_mul(volume_percent) / 100;
    // Keep equipment bought with gold below a 60% buy/sell spread, even while
    // global demand is high; otherwise fixed-price equipment shops could be
    // exploited for arbitrage. Special-store items with no gold buy price retain
    // their existing resale value, adjusted by broad market pressure.
    let capped_price = if base_buy_price > 0 {
        dynamic_price.min(u128::from(base_buy_price) * PLAYER_SELL_SPREAD_PERCENT as u128 / 100)
    } else {
        dynamic_price
    };
    capped_price.clamp(1, u128::from(MAX_GOLD)) as u32
}

fn quantity_movement(quantity: u32, is_buy: bool) -> i32 {
    let capped = quantity.min(MAX_RECORDED_TRADE_VOLUME as u32) as i32;
    if is_buy {
        capped
    } else {
        -capped
    }
}

pub fn buy_item_transaction(
    state: &mut GameState,
    item_type: ItemType,
    quantity: u32,
) -> Result<u32, EconomyError> {
    let retail = calculate_dynamic_buy_price(state, item_type);
    let unit_price = crate::mastery::vendor_price(state, retail, 0);
    buy_item_transaction_at_price(state, item_type, quantity, unit_price)
}

/// Complete a purchase at a caller-supplied unit price, used for the travelling
/// merchant's discounted offer. Validation and the market update remain shared.
pub fn buy_item_transaction_at_price(
    state: &mut GameState,
    item_type: ItemType,
    quantity: u32,
    unit_price: u32,
) -> Result<u32, EconomyError> {
    if quantity == 0 {
        return Err(EconomyError::InvalidQuantity(quantity));
    }
    if item_type.buy_price() == 0 || unit_price == 0 {
        return Err(EconomyError::UntradeableItem(item_type));
    }

    let total_cost = u64::from(unit_price) * u64::from(quantity);
    if total_cost > u64::from(state.player.gold) {
        return Err(EconomyError::InsufficientFunds {
            required: total_cost,
            available: state.player.gold,
        });
    }

    let available = state.player.get_item_count(item_type);
    let new_quantity = available
        .checked_add(quantity)
        .ok_or(EconomyError::InventoryOverflow(item_type))?;

    // All validation happens before the first state mutation.
    state.player.gold -= total_cost as u32;
    state.player.items.insert(item_type, new_quantity);
    state
        .market_state
        .record_trade(item_type, quantity_movement(quantity, true));

    Ok(total_cost as u32)
}

pub fn sell_item_transaction(
    state: &mut GameState,
    item_type: ItemType,
    quantity: u32,
) -> Result<u32, EconomyError> {
    if quantity == 0 {
        return Err(EconomyError::InvalidQuantity(quantity));
    }
    if item_type.buy_price() == 0 {
        return Err(EconomyError::UntradeableItem(item_type));
    }

    let available = state.player.get_item_count(item_type);
    if available < quantity {
        return Err(EconomyError::InsufficientInventory {
            item_type,
            required: quantity,
            available,
        });
    }

    let unit_price = calculate_dynamic_sell_price(state, item_type);
    let total_revenue = u64::from(unit_price) * u64::from(quantity);
    let resulting_gold = u64::from(state.player.gold) + total_revenue;
    if resulting_gold > u64::from(MAX_GOLD) {
        return Err(EconomyError::GoldCapacityExceeded {
            balance: state.player.gold,
            revenue: total_revenue,
            maximum: MAX_GOLD,
        });
    }

    // The validated removal cannot fail unless the inventory is concurrently
    // mutated, which cannot occur in this single-threaded game loop.
    let removed = state.player.remove_item(item_type, quantity);
    debug_assert!(removed, "validated sale inventory must still exist");
    state.player.gold += total_revenue as u32;
    state
        .market_state
        .record_trade(item_type, quantity_movement(quantity, false));

    Ok(total_revenue as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ItemType;
    use crate::event::{WorldEvent, WorldEventType};
    use crate::state::{GameState, Player};

    fn state() -> GameState {
        let mut state = GameState::new(Player::new("MarketHero".into(), String::new()));
        // Keep transaction expectations deterministic regardless of the current
        // day's generated world event. Event pricing has dedicated tests below.
        state.active_world_event = WorldEvent::none();
        state
    }

    #[test]
    fn prices_start_at_base_and_remain_within_market_bounds() {
        let mut state = state();
        let base = ItemType::Ore.buy_price();

        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 0),
            base
        );

        state.market_state.record_trade(ItemType::Ore, 100);
        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 0),
            base * 5 / 2
        );

        state.market_state.record_trade(ItemType::Ore, -200);
        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 0),
            base / 2
        );
    }

    #[test]
    fn market_boom_changes_only_the_matching_items_retail_and_resale_prices() {
        let mut state = state();
        state.active_world_event = WorldEvent {
            id: "ore_market_test".into(),
            name: "Ore Shortage".into(),
            description: "Ore is in demand.".into(),
            event_type: WorldEventType::MarketBoom {
                item_type: ItemType::Ore,
                price_multiplier: 2,
            },
            duration_secs: 1000,
            start_timestamp: 100,
        };

        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 200),
            ItemType::Ore.buy_price() * 2
        );
        assert_eq!(
            calculate_dynamic_sell_price_at(&state, ItemType::Ore, 200),
            ItemType::Ore.buy_price() * 2 * 60 / 100
        );
        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Logs, 200),
            ItemType::Logs.buy_price()
        );
        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 1100),
            ItemType::Ore.buy_price()
        );
    }

    #[test]
    fn successful_purchase_updates_gold_inventory_and_market_pressure() {
        let mut state = state();
        state.player.gold = 5000;

        assert_eq!(buy_item_transaction(&mut state, ItemType::Ore, 1), Ok(500));
        assert_eq!(state.player.gold, 4500);
        assert_eq!(state.player.get_item_count(ItemType::Ore), 1);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&1));
        assert_eq!(
            calculate_dynamic_buy_price_at(&state, ItemType::Ore, 0),
            510
        );
    }

    #[test]
    fn failed_purchase_leaves_gold_inventory_and_market_unchanged() {
        let mut state = state();
        state.player.gold = 100;

        assert!(matches!(
            buy_item_transaction(&mut state, ItemType::Ore, 1),
            Err(EconomyError::InsufficientFunds { .. })
        ));
        assert_eq!(state.player.gold, 100);
        assert_eq!(state.player.get_item_count(ItemType::Ore), 0);
        assert!(state.market_state.net_trades.is_empty());
        assert_eq!(state.market_state.global_net_trades, 0);
    }

    #[test]
    fn inventory_overflow_rejects_purchase_without_charging_gold() {
        let mut state = state();
        state.player.gold = 5000;
        state.player.items.insert(ItemType::Ore, u32::MAX);

        assert_eq!(
            buy_item_transaction(&mut state, ItemType::Ore, 1),
            Err(EconomyError::InventoryOverflow(ItemType::Ore))
        );
        assert_eq!(state.player.gold, 5000);
        assert_eq!(state.player.get_item_count(ItemType::Ore), u32::MAX);
        assert!(state.market_state.net_trades.is_empty());
    }

    #[test]
    fn sale_pays_sixty_percent_and_records_negative_market_pressure() {
        let mut state = state();
        state.player.add_item(ItemType::Ore, 2);

        assert_eq!(sell_item_transaction(&mut state, ItemType::Ore, 1), Ok(300));
        assert_eq!(state.player.gold, 400);
        assert_eq!(state.player.get_item_count(ItemType::Ore), 1);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&-1));
    }

    #[test]
    fn sale_over_the_gold_cap_is_atomic() {
        let mut state = state();
        state.player.gold = MAX_GOLD - 100;
        state.player.add_item(ItemType::MagicOrbs, 1);

        assert!(matches!(
            sell_item_transaction(&mut state, ItemType::MagicOrbs, 1),
            Err(EconomyError::GoldCapacityExceeded { .. })
        ));
        assert_eq!(state.player.gold, MAX_GOLD - 100);
        assert_eq!(state.player.get_item_count(ItemType::MagicOrbs), 1);
        assert!(state.market_state.net_trades.is_empty());
    }

    #[test]
    fn zero_quantity_and_crafting_only_items_are_rejected() {
        let mut state = state();
        assert_eq!(
            buy_item_transaction(&mut state, ItemType::Ore, 0),
            Err(EconomyError::InvalidQuantity(0))
        );
        assert_eq!(
            buy_item_transaction(&mut state, ItemType::Thread, 1),
            Err(EconomyError::UntradeableItem(ItemType::Thread))
        );
        assert_eq!(
            sell_item_transaction(&mut state, ItemType::GourmetFeast, 1),
            Err(EconomyError::UntradeableItem(ItemType::GourmetFeast))
        );
        assert_eq!(state.player.gold, 100);
        assert!(state.market_state.net_trades.is_empty());
    }

    #[test]
    fn market_pressure_decays_only_after_five_turns() {
        let mut market = MarketState::new();
        market.record_trade(ItemType::Logs, 5);

        for _ in 0..4 {
            market.advance_turn();
        }
        assert_eq!(market.net_trades.get(&ItemType::Logs), Some(&5));

        market.advance_turn();
        assert_eq!(market.net_trades.get(&ItemType::Logs), Some(&4));
        assert_eq!(market.global_net_trades, 4);
    }

    #[test]
    fn smuggler_price_tracks_global_market_pressure() {
        let mut state = state();
        state
            .market_state
            .record_trade(ItemType::Ore, MAX_RECORDED_TRADE_VOLUME);

        // A high-pressure market cannot turn a fixed-price equipment purchase
        // into a profitable resale loop; the payout is capped at 60% of retail.
        assert_eq!(
            calculate_dynamic_equipment_sell_price(&state, 500, 1000),
            600
        );
    }

    #[test]
    fn selling_items_does_not_advance_quest_hall_hand_in_progress() {
        use crate::quests::{Quest, QuestObjective, QuestReward};

        let mut state = state();
        state.active_quests.push(Quest {
            id: "ore_hand_in_test".into(),
            title: "Supply the Smelter".into(),
            description: "Hand in three Ore at the Quest Hall.".into(),
            objective: QuestObjective::ItemHandIn {
                item_type: ItemType::Ore,
                required_count: 3,
            },
            progress: 0,
            target: 3,
            is_completed: false,
            is_claimed: false,
            reward: QuestReward::Gold(1),
            is_daily: false,
            stages: Vec::new(),
        });
        state.player.add_item(ItemType::Ore, 1);

        sell_item_transaction(&mut state, ItemType::Ore, 1).unwrap();

        let quest = state
            .active_quests
            .iter()
            .find(|quest| quest.id == "ore_hand_in_test")
            .unwrap();
        assert_eq!(quest.progress, 0);
        assert!(!quest.is_completed);
    }

    #[test]
    fn old_game_state_without_market_data_uses_a_default_market() {
        let state = state();
        let mut value = serde_json::to_value(&state).unwrap();
        value.as_object_mut().unwrap().remove("market_state");

        let loaded: GameState = serde_json::from_value(value).unwrap();
        assert!(loaded.market_state.net_trades.is_empty());
        assert_eq!(loaded.market_state.global_net_trades, 0);
    }
}
