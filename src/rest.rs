//! Short/long rest and field-camping rules.
//!
//! Rest is a committed world action, not a menu tick. All costs and resource
//! gains are planned before mutation; successful rest advances only the
//! Section 21 world clock through GameState::advance_world_time.
//!
//! Rest never applies treatment. A companion recovery record with a healer
//! destination progresses only through elapsed hours; a record without one is
//! intentionally preserved until a valid treatment route exists.
//!
//! Player combat status effects live inside an encounter and are not persisted
//! as long-lived GameState conditions. Rest therefore does not clear combat
//! effects or alter persistent companion conditions directly.

use std::fmt;

use crate::data::ItemType;
use crate::party::CompanionId;
use crate::state::{GameState, Player};
use crate::world::{Location, LocationId, LocationService, World};

pub const SHORT_REST_HOURS: u64 = 1;
pub const LONG_REST_HOURS: u64 = 8;
pub const FIELD_CAMP_HOURS: u64 = 3;
pub const LONG_REST_GOLD_COST: u32 = 25;
pub const FIELD_CAMP_FOOD_COST: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestKind {
    Short,
    Long,
    FieldCamp,
}

impl RestKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Short => "Short rest",
            Self::Long => "Long rest",
            Self::FieldCamp => "Field camp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestAction {
    ShortRest,
    LongRest,
    FieldCamp,
    Cancel,
}

impl RestAction {
    const fn kind(self) -> Option<RestKind> {
        match self {
            Self::ShortRest => Some(RestKind::Short),
            Self::LongRest => Some(RestKind::Long),
            Self::FieldCamp => Some(RestKind::FieldCamp),
            Self::Cancel => None,
        }
    }
}

/// Callers must explicitly identify their execution context. The current UI
/// offers rest only from the main menu; this guard also protects future callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestContext {
    MainMenu,
    Combat,
    ScriptedSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceTotals {
    pub hp: u32,
    pub stamina: u32,
    pub mana: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRecovery {
    pub hp: u32,
    pub stamina: u32,
    pub mana: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestPlan {
    pub kind: RestKind,
    pub hours: u64,
    pub gold_cost: u32,
    pub food_cost: u32,
    pub before: ResourceTotals,
    pub after: ResourceTotals,
}

impl RestPlan {
    pub fn recovery(&self) -> ResourceRecovery {
        ResourceRecovery {
            hp: self.after.hp.saturating_sub(self.before.hp),
            stamina: self.after.stamina.saturating_sub(self.before.stamina),
            mana: self.after.mana.saturating_sub(self.before.mana),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestError {
    RestNotAllowed,
    MissingService(RestKind),
    NothingToRecover,
    InsufficientGold { required: u32, available: u32 },
    InsufficientFood { required: u32, available: u32 },
    UnknownLocation(LocationId),
    WorldClockExhausted,
}

impl fmt::Display for RestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RestNotAllowed => {
                f.write_str("Rest is unavailable during combat or a scripted sequence.")
            }
            Self::MissingService(RestKind::Short) => {
                f.write_str("A tavern, guildhouse or bed service is required for a short rest.")
            }
            Self::MissingService(RestKind::Long) => {
                f.write_str("A bed service is required for a long rest.")
            }
            Self::MissingService(RestKind::FieldCamp) => {
                f.write_str("This location is not designated for field camping.")
            }
            Self::NothingToRecover => f.write_str("HP, stamina and mana are already full."),
            Self::InsufficientGold {
                required,
                available,
            } => write!(
                f,
                "A long rest costs {required} gold, but only {available} is available."
            ),
            Self::InsufficientFood {
                required,
                available,
            } => write!(
                f,
                "Field camping requires {required} Food, but only {available} is available."
            ),
            Self::UnknownLocation(id) => write!(f, "Unknown current location '{id}'."),
            Self::WorldClockExhausted => {
                f.write_str("The world clock cannot advance any further; no rest was charged.")
            }
        }
    }
}

impl std::error::Error for RestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestOutcome {
    pub plan: RestPlan,
    pub recovered_companions: Vec<CompanionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestExecution {
    Cancelled,
    Completed(RestOutcome),
}

/// Calculate an exact recovery plan without changing the player or world.
/// A settlement category alone never grants access to a rest service.
pub fn plan_rest(
    player: &Player,
    location: &Location,
    kind: RestKind,
    context: RestContext,
) -> Result<RestPlan, RestError> {
    if context != RestContext::MainMenu {
        return Err(RestError::RestNotAllowed);
    }

    let has_required_service = match kind {
        RestKind::Short => [
            LocationService::Tavern,
            LocationService::Guildhouse,
            LocationService::Bed,
        ]
        .iter()
        .any(|service| location.has_service(*service)),
        RestKind::Long => location.has_service(LocationService::Bed),
        RestKind::FieldCamp => location.has_service(LocationService::FieldCamp),
    };
    if !has_required_service {
        return Err(RestError::MissingService(kind));
    }

    let before = ResourceTotals {
        hp: player.hp,
        stamina: player.stamina,
        mana: player.mana,
    };
    let after = match kind {
        RestKind::Short => ResourceTotals {
            hp: recover_by_percent(before.hp, player.max_hp, 25),
            stamina: recover_by_percent(before.stamina, player.max_stamina, 40),
            mana: recover_by_percent(before.mana, player.max_mana, 25),
        },
        RestKind::Long => ResourceTotals {
            hp: player.max_hp,
            stamina: player.max_stamina,
            mana: player.max_mana,
        },
        RestKind::FieldCamp => ResourceTotals {
            hp: recover_by_percent(before.hp, player.max_hp, 10),
            stamina: recover_by_percent(before.stamina, player.max_stamina, 25),
            mana: recover_by_percent(before.mana, player.max_mana, 10),
        },
    };
    if before == after {
        return Err(RestError::NothingToRecover);
    }

    let (hours, gold_cost, food_cost) = match kind {
        RestKind::Short => (SHORT_REST_HOURS, 0, 0),
        RestKind::Long => (LONG_REST_HOURS, LONG_REST_GOLD_COST, 0),
        RestKind::FieldCamp => (FIELD_CAMP_HOURS, 0, FIELD_CAMP_FOOD_COST),
    };
    if player.gold < gold_cost {
        return Err(RestError::InsufficientGold {
            required: gold_cost,
            available: player.gold,
        });
    }
    let food = player.get_item_count(ItemType::Food);
    if food < food_cost {
        return Err(RestError::InsufficientFood {
            required: food_cost,
            available: food,
        });
    }

    Ok(RestPlan {
        kind,
        hours,
        gold_cost,
        food_cost,
        before,
        after,
    })
}

fn recover_by_percent(current: u32, maximum: u32, percent: u32) -> u32 {
    let gain = (u64::from(maximum) * u64::from(percent) / 100) as u32;
    let gain = if current < maximum && maximum > 0 {
        gain.max(1)
    } else {
        0
    };
    current.saturating_add(gain).min(maximum)
}

/// Execute a rest transaction. Cancel is always a no-op, including in a
/// context where a rest itself would be forbidden.
pub fn execute_rest(
    state: &mut GameState,
    world: &World,
    action: RestAction,
    context: RestContext,
) -> Result<RestExecution, RestError> {
    let Some(kind) = action.kind() else {
        return Ok(RestExecution::Cancelled);
    };
    let location = world
        .get_location(&state.current_location)
        .ok_or_else(|| RestError::UnknownLocation(state.current_location.clone()))?;
    let plan = plan_rest(&state.player, location, kind, context)?;

    // Do not debit resources when the persisted clock cannot represent the
    // full selected rest duration.
    if state.world_turn.checked_add(plan.hours).is_none() {
        return Err(RestError::WorldClockExhausted);
    }

    // All preconditions have passed; from here the transaction cannot fail.
    state.player.gold -= plan.gold_cost;
    if plan.food_cost > 0 {
        let removed = state.player.remove_item(ItemType::Food, plan.food_cost);
        debug_assert!(removed, "food availability was checked before mutation");
    }
    state.player.hp = plan.after.hp;
    state.player.stamina = plan.after.stamina;
    state.player.mana = plan.after.mana;
    let recovered_companions = state.advance_world_time(plan.hours);

    Ok(RestExecution::Completed(RestOutcome {
        plan,
        recovered_companions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_recovery::CompanionRecoveryRecord;
    use crate::party::{CompanionCondition, CompanionId, CompanionState};
    use crate::state::Player;
    use crate::world::{build_initial_world, Location, LocationCategory};

    fn state() -> GameState {
        GameState::new(Player::new("rest-test".into(), "hash".into()))
    }

    fn injured_state() -> GameState {
        let mut state = state();
        state.player.hp = 40;
        state.player.stamina = 20;
        state.player.mana = 10;
        state
    }

    fn village(world: &World) -> &Location {
        world
            .get_location(&LocationId::from("village_start"))
            .expect("Oakhaven should exist")
    }

    #[test]
    fn short_rest_recovers_exact_bounded_amounts_at_an_explicit_safe_haven() {
        let world = build_initial_world().unwrap();
        let state = injured_state();
        let plan = plan_rest(
            &state.player,
            village(&world),
            RestKind::Short,
            RestContext::MainMenu,
        )
        .unwrap();

        assert_eq!(plan.hours, 1);
        assert_eq!(plan.gold_cost, 0);
        assert_eq!(plan.food_cost, 0);
        assert_eq!(plan.after, ResourceTotals { hp: 65, stamina: 60, mana: 22 });
        assert_eq!(
            plan.recovery(),
            ResourceRecovery { hp: 25, stamina: 40, mana: 12 }
        );
    }

    #[test]
    fn long_rest_requires_a_bed_and_restores_all_three_resource_pools() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        let plan = plan_rest(
            &state.player,
            village(&world),
            RestKind::Long,
            RestContext::MainMenu,
        )
        .unwrap();
        assert_eq!(plan.hours, 8);
        assert_eq!(plan.gold_cost, LONG_REST_GOLD_COST);
        assert_eq!(plan.after, ResourceTotals { hp: 100, stamina: 100, mana: 50 });

        let execution = execute_rest(
            &mut state,
            &world,
            RestAction::LongRest,
            RestContext::MainMenu,
        )
        .unwrap();
        let RestExecution::Completed(outcome) = execution else {
            panic!("long rest should complete");
        };
        assert_eq!(state.player.gold, 75);
        assert_eq!(state.world_turn, 8);
        assert_eq!(state.player.hp, 100);
        assert_eq!(state.player.stamina, 100);
        assert_eq!(state.player.mana, 50);
        assert_eq!(outcome.plan.after, plan.after);
    }

    #[test]
    fn a_settlement_category_does_not_grant_rest_services() {
        let location = Location::new(
            "bare_settlement",
            "Bare Settlement",
            "There is no inn or guildhouse here.",
            LocationCategory::Settlement,
        );
        let state = injured_state();
        assert!(matches!(
            plan_rest(&state.player, &location, RestKind::Short, RestContext::MainMenu),
            Err(RestError::MissingService(RestKind::Short))
        ));
        assert!(matches!(
            plan_rest(&state.player, &location, RestKind::Long, RestContext::MainMenu),
            Err(RestError::MissingService(RestKind::Long))
        ));
    }

    #[test]
    fn field_camp_is_explicit_costs_one_food_and_recovers_less_than_safe_rest() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        state.current_location = LocationId::from("whispering_woods");
        state.player.add_item(ItemType::Food, 1);

        let execution = execute_rest(
            &mut state,
            &world,
            RestAction::FieldCamp,
            RestContext::MainMenu,
        )
        .unwrap();
        let RestExecution::Completed(outcome) = execution else {
            panic!("field camp should complete");
        };
        assert_eq!(outcome.plan.hours, 3);
        assert_eq!(outcome.plan.food_cost, 1);
        assert_eq!(outcome.plan.gold_cost, 0);
        assert_eq!(outcome.plan.after, ResourceTotals { hp: 50, stamina: 45, mana: 15 });
        assert_eq!(state.player.get_item_count(ItemType::Food), 0);
        assert_eq!(state.world_turn, 3);
    }

    #[test]
    fn full_resources_and_unaffordable_rest_never_consume_time_or_costs() {
        let world = build_initial_world().unwrap();
        let mut full = state();
        assert!(matches!(
            execute_rest(&mut full, &world, RestAction::ShortRest, RestContext::MainMenu),
            Err(RestError::NothingToRecover)
        ));
        assert_eq!(full.world_turn, 0);

        let mut poor = injured_state();
        poor.player.gold = LONG_REST_GOLD_COST - 1;
        let gold_before = poor.player.gold;
        assert!(matches!(
            execute_rest(&mut poor, &world, RestAction::LongRest, RestContext::MainMenu),
            Err(RestError::InsufficientGold { .. })
        ));
        assert_eq!(poor.player.gold, gold_before);
        assert_eq!(poor.world_turn, 0);

        let mut hungry = injured_state();
        hungry.current_location = LocationId::from("whispering_woods");
        assert!(matches!(
            execute_rest(&mut hungry, &world, RestAction::FieldCamp, RestContext::MainMenu),
            Err(RestError::InsufficientFood { .. })
        ));
        assert_eq!(hungry.world_turn, 0);
    }

    #[test]
    fn cancel_is_a_strict_no_op_even_when_rest_would_be_forbidden() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        state.player.gold = 31;
        state.player.add_item(ItemType::Food, 2);
        let before = (
            state.player.hp,
            state.player.stamina,
            state.player.mana,
            state.player.gold,
            state.player.get_item_count(ItemType::Food),
            state.world_turn,
            state.market_state.clone(),
            state.companion_recovery.clone(),
        );
        assert_eq!(
            execute_rest(
                &mut state,
                &world,
                RestAction::Cancel,
                RestContext::Combat
            )
            .unwrap(),
            RestExecution::Cancelled
        );
        assert_eq!(
            (
                state.player.hp,
                state.player.stamina,
                state.player.mana,
                state.player.gold,
                state.player.get_item_count(ItemType::Food),
                state.world_turn,
                state.market_state.clone(),
                state.companion_recovery.clone(),
            ),
            before
        );
    }

    #[test]
    fn rest_is_rejected_in_combat_or_scripted_context_without_mutation() {
        let world = build_initial_world().unwrap();
        for context in [RestContext::Combat, RestContext::ScriptedSequence] {
            let mut state = injured_state();
            let before = (state.player.hp, state.player.gold, state.world_turn);
            assert_eq!(
                execute_rest(&mut state, &world, RestAction::LongRest, context),
                Err(RestError::RestNotAllowed)
            );
            assert_eq!((state.player.hp, state.player.gold, state.world_turn), before);
        }
    }

    #[test]
    fn rest_advances_only_treatment_backed_companion_recovery() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        let id = CompanionId::BranTheShield;
        let mut companion = CompanionState::new_from_definition(&crate::party::get_definition(id));
        companion.current_hp = 0;
        companion.condition = CompanionCondition::KnockedOut;
        state.companions.push(companion);
        state.companion_recovery.records.push(CompanionRecoveryRecord {
            companion_id: id,
            destination: Some(LocationId::from("village_start")),
            turns_remaining: 2,
        });

        let first = execute_rest(
            &mut state,
            &world,
            RestAction::ShortRest,
            RestContext::MainMenu,
        )
        .unwrap();
        let RestExecution::Completed(first) = first else {
            panic!("short rest should complete");
        };
        assert!(first.recovered_companions.is_empty());
        assert_eq!(state.companion_recovery.record(id).unwrap().turns_remaining, 1);

        let second = execute_rest(
            &mut state,
            &world,
            RestAction::ShortRest,
            RestContext::MainMenu,
        )
        .unwrap();
        let RestExecution::Completed(second) = second else {
            panic!("second short rest should complete");
        };
        assert_eq!(second.recovered_companions, vec![id]);
        assert!(state.companion_recovery.record(id).is_none());
        assert_eq!(state.companions[0].condition, CompanionCondition::Available);
    }

    #[test]
    fn resting_cannot_clear_or_complete_a_knockout_record_without_a_healer() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        let id = CompanionId::BranTheShield;
        let mut companion = CompanionState::new_from_definition(&crate::party::get_definition(id));
        companion.current_hp = 0;
        companion.condition = CompanionCondition::KnockedOut;
        state.companions.push(companion);
        state.companion_recovery.records.push(CompanionRecoveryRecord {
            companion_id: id,
            destination: None,
            turns_remaining: 3,
        });

        let outcome = execute_rest(
            &mut state,
            &world,
            RestAction::LongRest,
            RestContext::MainMenu,
        )
        .unwrap();
        assert!(matches!(outcome, RestExecution::Completed(_)));
        let record = state.companion_recovery.record(id).unwrap();
        assert_eq!(record.destination, None);
        assert_eq!(record.turns_remaining, 3);
        assert_eq!(state.companions[0].current_hp, 0);
        assert_eq!(state.companions[0].condition, CompanionCondition::KnockedOut);
    }

    #[test]
    fn exhausted_clock_rejects_rest_before_spending_resources() {
        let world = build_initial_world().unwrap();
        let mut state = injured_state();
        state.world_turn = u64::MAX;
        let gold_before = state.player.gold;
        assert_eq!(
            execute_rest(&mut state, &world, RestAction::LongRest, RestContext::MainMenu),
            Err(RestError::WorldClockExhausted)
        );
        assert_eq!(state.player.gold, gold_before);
        assert_eq!(state.world_turn, u64::MAX);
    }
}
