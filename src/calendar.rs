//! Deterministic in-game calendar derived from the persisted world clock.
//!
//! One world turn is one in-game hour. Calendar state is never persisted
//! separately: old saves retain their exact world_turn and map to the same
//! date/time through the fixed epoch below.

use crate::party::CompanionId;
use crate::state::GameState;
use crate::ui::Tui;
use std::io;

pub const HOURS_PER_DAY: u64 = 24;
pub const DAYS_PER_MONTH: u64 = 30;
pub const MONTHS_PER_YEAR: u64 = 12;
pub const MONTHS_PER_SEASON: u64 = 3;
pub const SEASONS_PER_YEAR: u64 = 4;
pub const DAYS_PER_YEAR: u64 = DAYS_PER_MONTH * MONTHS_PER_YEAR;
pub const HOURS_PER_MONTH: u64 = HOURS_PER_DAY * DAYS_PER_MONTH;
pub const HOURS_PER_SEASON: u64 = HOURS_PER_MONTH * MONTHS_PER_SEASON;
pub const HOURS_PER_YEAR: u64 = HOURS_PER_DAY * DAYS_PER_YEAR;

/// Legacy world_turn == 0 maps to Year 1, Firstbloom 1, Spring, 06:00.
/// The six-hour offset makes the opening moment morning while retaining a
/// simple midnight-based calendar and a deterministic 360-day year.
pub const CALENDAR_EPOCH_HOUR: u8 = 6;

pub const LOCATION_INTERACTION_HOURS: u64 = 1;
pub const COMBAT_HOURS_PER_ROUND: u64 = 1;
pub const INN_REST_HOURS: u32 = 2;
pub const FOOD_CAMP_HOURS: u32 = 2;
pub const ROUGH_SLEEP_HOURS: u32 = 4;

/// Names are deliberately centralised and replaceable until campaign canon is
/// established. Three consecutive months belong to each season.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarConfig {
    pub month_names: [&'static str; MONTHS_PER_YEAR as usize],
    pub season_names: [&'static str; SEASONS_PER_YEAR as usize],
    pub epoch_hour: u8,
}

pub const DEFAULT_CALENDAR_CONFIG: CalendarConfig = CalendarConfig {
    month_names: [
        "Firstbloom",
        "Greenswell",
        "Brightreach",
        "Highsun",
        "Emberwane",
        "Goldleaf",
        "Duskhallow",
        "Frostfall",
        "Longnight",
        "Deepwinter",
        "Thawrise",
        "Newlight",
    ],
    season_names: ["Spring", "Summer", "Autumn", "Winter"],
    epoch_hour: CALENDAR_EPOCH_HOUR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    DeepNight,
    Dawn,
    Morning,
    Afternoon,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DeepNight => "Deep Night",
            Self::Dawn => "Dawn",
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Dusk => "Dusk",
            Self::Night => "Night",
        }
    }

    pub const fn from_hour(hour: u8) -> Self {
        match hour {
            0..=4 => Self::DeepNight,
            5..=7 => Self::Dawn,
            8..=11 => Self::Morning,
            12..=16 => Self::Afternoon,
            17..=19 => Self::Dusk,
            _ => Self::Night,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDateTime {
    pub year: u64,
    /// One-based month number (1–12).
    pub month: u8,
    pub month_name: &'static str,
    /// One-based day within the month (1–30).
    pub day: u8,
    /// Zero-based day within the year (0–359).
    pub day_of_year: u16,
    /// Zero-based hour within the day (0–23).
    pub hour: u8,
    /// Zero-based season index (0–3).
    pub season_index: u8,
    pub season_name: &'static str,
    pub time_of_day: TimeOfDay,
}

impl CalendarDateTime {
    pub fn format(self) -> String {
        format!(
            "Year {} · {} {} · {} · {} ({:02}:00)",
            self.year,
            self.month_name,
            self.day,
            self.season_name,
            self.time_of_day.label(),
            self.hour
        )
    }

    pub const fn is_hour(self, hour: u8) -> bool {
        self.hour == hour
    }

    pub const fn is_within_hours(self, start_hour: u8, end_hour: u8) -> bool {
        is_hour_in_window(self.hour, start_hour, end_hour)
    }
}

/// Convert the legacy persisted turn counter into calendar time. The u128
/// intermediate prevents the epoch offset overflowing at u64::MAX.
pub fn calendar_at(world_turn: u64) -> CalendarDateTime {
    calendar_at_with_config(world_turn, &DEFAULT_CALENDAR_CONFIG)
}

pub fn calendar_at_with_config(
    world_turn: u64,
    config: &CalendarConfig,
) -> CalendarDateTime {
    let epoch_hour = u128::from(config.epoch_hour.min(23));
    let absolute_hour = u128::from(world_turn) + epoch_hour;
    let hour_of_year = absolute_hour % u128::from(HOURS_PER_YEAR);
    let year = absolute_hour / u128::from(HOURS_PER_YEAR) + 1;
    let day_of_year = hour_of_year / u128::from(HOURS_PER_DAY);
    let month_index = day_of_year / u128::from(DAYS_PER_MONTH);
    let day_in_month = day_of_year % u128::from(DAYS_PER_MONTH);
    let hour = (hour_of_year % u128::from(HOURS_PER_DAY)) as u8;
    let season_index = month_index / u128::from(MONTHS_PER_SEASON);

    CalendarDateTime {
        year: u64::try_from(year).unwrap_or(u64::MAX),
        month: month_index as u8 + 1,
        month_name: config.month_names[month_index as usize],
        day: day_in_month as u8 + 1,
        day_of_year: day_of_year as u16,
        hour,
        season_index: season_index as u8,
        season_name: config.season_names[season_index as usize],
        time_of_day: TimeOfDay::from_hour(hour),
    }
}

pub fn format_calendar(world_turn: u64) -> String {
    calendar_at(world_turn).format()
}

/// Half-open hour window [start, end). Windows may cross midnight (for example
/// 22–06). Invalid endpoints and equal endpoints match no hours; callers that
/// need an all-day window should handle that case explicitly.
pub const fn is_hour_in_window(hour: u8, start_hour: u8, end_hour: u8) -> bool {
    if hour >= 24 || start_hour >= 24 || end_hour >= 24 || start_hour == end_hour {
        return false;
    }
    if start_hour < end_hour {
        hour >= start_hour && hour < end_hour
    } else {
        hour >= start_hour || hour < end_hour
    }
}

/// Show a recovery notice after an elapsed-time operation. Empty recovery
/// results are silent so callers can use this after every committed duration.
pub fn notify_recovered_companions(
    tui: &mut Tui,
    state: &GameState,
    recovered: &[CompanionId],
) -> io::Result<()> {
    if recovered.is_empty() {
        return Ok(());
    }
    let names = recovered
        .iter()
        .map(|id| crate::party::get_definition(*id).name)
        .collect::<Vec<_>>()
        .join(", ");
    tui.show_message(
        "Companion recovered",
        &format!(
            "{names} recovered as world time advanced. They are available for explicit party selection.\n\n{}",
            format_calendar(state.world_turn)
        ),
        Some(state),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_recovery::CompanionRecoveryRecord;
    use crate::data::ItemType;
    use crate::party::{CompanionCondition, CompanionId, CompanionState};
    use crate::state::Player;
    use crate::world::LocationId;

    fn state() -> GameState {
        GameState::new(Player::new("calendar-test".into(), "hash".into()))
    }

    #[test]
    fn legacy_epoch_is_a_documented_morning_in_the_first_spring_month() {
        let date = calendar_at(0);
        assert_eq!(date.year, 1);
        assert_eq!(date.month, 1);
        assert_eq!(date.month_name, "Firstbloom");
        assert_eq!(date.day, 1);
        assert_eq!(date.season_name, "Spring");
        assert_eq!(date.hour, 6);
        assert_eq!(date.time_of_day, TimeOfDay::Dawn);
    }

    #[test]
    fn midnight_and_month_boundaries_are_exact() {
        let last_moment = calendar_at(HOURS_PER_MONTH - u64::from(CALENDAR_EPOCH_HOUR) - 1);
        let next_month = calendar_at(HOURS_PER_MONTH - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((last_moment.month, last_moment.day, last_moment.hour), (1, 30, 23));
        assert_eq!((next_month.month, next_month.day, next_month.hour), (2, 1, 0));

        let midnight = calendar_at(HOURS_PER_DAY - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((midnight.day, midnight.hour), (2, 0));
    }

    #[test]
    fn season_and_year_boundaries_follow_three_months_and_twelve_months() {
        let summer = calendar_at(HOURS_PER_SEASON - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((summer.month, summer.day, summer.season_name), (4, 1, "Summer"));

        let autumn = calendar_at(2 * HOURS_PER_SEASON - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((autumn.month, autumn.day, autumn.season_name), (7, 1, "Autumn"));

        let winter = calendar_at(3 * HOURS_PER_SEASON - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((winter.month, winter.day, winter.season_name), (10, 1, "Winter"));

        let next_year = calendar_at(HOURS_PER_YEAR - u64::from(CALENDAR_EPOCH_HOUR));
        assert_eq!((next_year.year, next_year.month, next_year.day), (2, 1, 1));
        assert_eq!(next_year.season_name, "Spring");
    }

    #[test]
    fn time_windows_support_daytime_and_cross_midnight_ranges() {
        assert!(is_hour_in_window(9, 8, 17));
        assert!(!is_hour_in_window(17, 8, 17));
        assert!(is_hour_in_window(23, 22, 6));
        assert!(is_hour_in_window(4, 22, 6));
        assert!(!is_hour_in_window(12, 22, 6));
        assert!(!is_hour_in_window(4, 4, 4));
        assert!(!is_hour_in_window(24, 22, 6));
        assert!(calendar_at(0).is_within_hours(5, 8));
    }

    #[test]
    fn zero_elapsed_hours_change_neither_clock_market_nor_recovery() {
        let mut state = state();
        state.market_state.record_trade(ItemType::Ore, 5);
        let recovered = state.advance_world_time(0);

        assert!(recovered.is_empty());
        assert_eq!(state.world_turn, 0);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&5));
    }

    #[test]
    fn market_decay_is_applied_for_elapsed_hours_not_for_menu_reads() {
        let mut state = state();
        state.market_state.record_trade(ItemType::Ore, 5);

        state.advance_world_time(4);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&5));
        state.advance_world_time(1);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&4));
    }

    #[test]
    fn companion_recovery_uses_elapsed_hours_and_survives_save_roundtrip() {
        let mut state = state();
        let id = CompanionId::BranTheShield;
        let definition = crate::party::get_definition(id);
        let mut companion = CompanionState::new_from_definition(&definition);
        companion.current_hp = 0;
        companion.condition = CompanionCondition::KnockedOut;
        state.companions.push(companion);
        state.companion_recovery.records.push(CompanionRecoveryRecord {
            companion_id: id,
            destination: Some(LocationId::from("village_start")),
            turns_remaining: 3,
        });

        assert!(state.advance_world_time(2).is_empty());
        assert_eq!(state.companion_recovery.records[0].turns_remaining, 1);

        let encoded = serde_json::to_string(&state).unwrap();
        let mut loaded: GameState = serde_json::from_str(&encoded).unwrap();
        let recovered = loaded.advance_world_time(1);
        assert_eq!(recovered, vec![id]);
        assert!(loaded.companion_recovery.records.is_empty());
        let companion = loaded.companions.iter().find(|c| c.id == id).unwrap();
        assert_eq!(companion.condition, CompanionCondition::Available);
        assert_eq!(companion.current_hp, companion.max_hp);
    }

    #[test]
    fn legacy_save_without_a_clock_defaults_to_epoch_and_world_clock_roundtrips() {
        let mut original = state();
        let mut value = serde_json::to_value(&original).unwrap();
        value.as_object_mut().unwrap().remove("world_turn");

        let mut legacy: GameState = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.world_turn, 0);
        assert_eq!(calendar_at(legacy.world_turn), calendar_at(0));

        legacy.advance_world_time(1234);
        let value = serde_json::to_value(&legacy).unwrap();
        let loaded: GameState = serde_json::from_value(value).unwrap();
        assert_eq!(loaded.world_turn, 1234);
        assert_eq!(calendar_at(loaded.world_turn), calendar_at(1234));

        original.advance_world_time(0);
    }

    #[test]
    fn clock_overflow_saturates_and_only_commits_representable_elapsed_hours() {
        let mut state = state();
        state.world_turn = u64::MAX - 1;
        state.market_state.record_trade(ItemType::Ore, 5);

        state.advance_world_time(5);
        assert_eq!(state.world_turn, u64::MAX);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&5));

        state.advance_world_time(1);
        assert_eq!(state.world_turn, u64::MAX);
        assert_eq!(state.market_state.net_trades.get(&ItemType::Ore), Some(&5));
    }
}
