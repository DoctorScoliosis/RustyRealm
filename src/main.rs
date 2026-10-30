mod access;
mod achievements;
mod adventure;
mod balance_simulation;
mod bank;
mod calendar;
mod character;
mod checkpoint;
mod combat;
mod combat_engine;
mod companion_discovery;
mod companion_recovery;
mod crafting;
mod data;
mod economy;
mod event;
mod inventory;
mod mastery;
mod party;
mod party_battle;
mod party_combat;
mod quests;
mod realm_progression;
mod rest;
mod save;
mod shops;
mod skill_checks;
mod skills;
mod state;
mod status_combat;
mod stronghold;
mod ui;
// Some world queries are reserved for later gameplay slices; remove this
// allowance as the remaining APIs gain consumers.
#[allow(dead_code)]
mod world;

use crate::character::CharacterClass;
use crate::checkpoint::CheckpointSession;
use crate::data::ItemType;
use crate::state::GameState;
use crate::ui::Tui;
use crate::world::{build_initial_world, LocationId, World};
use std::io;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    loop {
        let accounts = match crate::save::list_local_accounts() {
            Ok(accounts) => accounts,
            Err(error) => {
                tui.show_message("Account storage unavailable", &error.to_string(), None)?;
                Vec::new()
            }
        };
        let mut options: Vec<String> = accounts.iter().cloned().collect();
        options.push("Create new account".into());
        options.push("Exit".into());
        let option_refs: Vec<&str> = options.iter().map(String::as_str).collect();
        let choice = tui.select_menu("Rusty Realm · Local Accounts", &option_refs, None)?;
        if choice < accounts.len() {
            let username = &accounts[choice];
            let password_required = match crate::save::account_requires_password(username) {
                Ok(required) => required,
                Err(crate::save::SaveError::PasswordMigrationRequired) => {
                    tui.show_message(
                        "Legacy account",
                        "This save predates password-hash persistence. The old password cannot be verified. You may migrate it by setting a new password, or leave it blank for local-only access. Cancelling leaves the save unchanged.",
                        None,
                    )?;
                    let consent = tui.prompt_input("Type YES to migrate this local save", false, None)?;
                    if consent != "YES" {
                        continue;
                    }
                    let new_password = tui.prompt_input("New password (leave blank for local-only access)", true, None)?;
                    if new_password.is_empty() {
                        tui.show_message(
                            "Passwordless local save",
                            "The migrated account will be accessible to anyone with access to this computer and its local save files.",
                            None,
                        )?;
                        let confirm_local = tui.prompt_input("Type LOCAL to continue without a password", false, None)?;
                        if confirm_local != "LOCAL" {
                            continue;
                        }
                    }
                    let confirmation = tui.prompt_input("Confirm password (leave blank if passwordless)", true, None)?;
                    if new_password != confirmation {
                        tui.show_message("Migration cancelled", "Passwords did not match. The save has not been changed.", None)?;
                        continue;
                    }
                    match crate::save::migrate_legacy_save(username, &new_password) {
                        Ok(mut state) => main_menu(&mut tui, &mut state)?,
                        Err(error) => tui.show_message("Migration failed", &error.to_string(), None)?,
                    }
                    continue;
                }
                Err(error) => {
                    tui.show_message("Account unavailable", &error.to_string(), None)?;
                    continue;
                }
            };
            let password = if password_required {
                tui.prompt_input("Password", true, None)?
            } else {
                String::new()
            };
            match crate::save::load_game(username, &password) {
                Ok(mut state) => main_menu(&mut tui, &mut state)?,
                Err(error) => tui.show_message("Login failed", &error.to_string(), None)?,
            }
        } else if choice == accounts.len() {
            if let Some(mut state) = create_account(&mut tui)? {
                main_menu(&mut tui, &mut state)?;
            }
        } else {
            break;
        }
    }
    // Tui's RAII guard restores raw mode, the cursor, and the previous screen.
    Ok(())
}


fn main_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let world = build_initial_world().map_err(|error| io::Error::other(error.to_string()))?;
    // Take the checkpoint baseline before startup reconciliation. Any migration
    // or repair performed below is therefore visible as unsaved progress.
    let mut checkpoint = CheckpointSession::new(state).map_err(io::Error::other)?;
    crate::adventure::ensure_region_quest(state);
    crate::realm_progression::reconcile_from_saved_evidence(state);
    crate::companion_discovery::reconcile_legacy_discoveries(state);
    // The location graph and recovery records are persisted separately; a
    // companion with zero HP is normalised here so stale party IDs cannot let
    // an older save field reintroduce them into combat.
    for companion in &mut state.companions {
        if companion.current_hp == 0
            && !matches!(
                companion.condition,
                crate::party::CompanionCondition::KnockedOut
                    | crate::party::CompanionCondition::Recovering
            )
        {
            companion.condition = crate::party::CompanionCondition::KnockedOut;
        }
    }
    let knocked_out: Vec<_> = state
        .companions
        .iter()
        .filter(|companion| {
            companion.current_hp == 0 && state.companion_recovery.record(companion.id).is_none()
        })
        .map(|companion| companion.id)
        .collect();
    let origin = state.current_location.clone();
    let mut new_records = Vec::new();
    for id in knocked_out {
        if let Some(companion) = state
            .companions
            .iter_mut()
            .find(|companion| companion.id == id)
        {
            if let Ok(true) = state
                .companion_recovery
                .record_knockout(companion, &world, &origin)
            {
                new_records.push(id);
            }
        }
    }
    for id in new_records {
        let _ = state.party.remove_member(id);
    }
    let options = [
        "Travel / Explore Region",
        "Interact with Current Location",
        "Expedition at Current Location",
        "Wander Gielinor",
        "Weapon Shop",
        "Armour Shop",
        "Legend's Store",
        "Trading Post (General Store)",
        "Max's Shop",
        "Smuggler",
        "The Stronghold",
        "Skill Plot",
        "RealmBank",
        "Quest Hall",
        "Your Inventory",
        "Mastery & Achievements",
        "Companions & Party",
        "Companion Recovery",
        "Rest / Recover",
        "Settings",
    ];

    loop {
        let newly_unlocked = crate::achievements::evaluate_achievements(state);
        if !newly_unlocked.is_empty() {
            let notice = newly_unlocked
                .iter()
                .map(|a| format!("{} — {}", a.title, a.description))
                .collect::<Vec<_>>()
                .join("\n\n");
            tui.show_message("Achievement unlocked", &notice, Some(state))?;
        }
        crate::quests::check_and_refresh_quests(state);
        let dirty = checkpoint.is_dirty(state).map_err(io::Error::other)?;

        let current_location_name = world
            .get_location(&state.current_location)
            .map(|location| location.name.clone())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Unknown current location '{}'.",
                    state.current_location
                ))
            })?;
        let dirty_marker = if dirty { " · UNSAVED" } else { "" };
        let title = format!(
            "Final Realm · {current_location_name} · {}{}",
            crate::calendar::format_calendar(state.world_turn),
            dirty_marker
        );
        let can_save_here = world
            .get_location(&state.current_location)
            .is_some_and(crate::checkpoint::is_checkpoint_location);
        let mut menu_options = options.to_vec();
        let save_index = if can_save_here {
            let index = menu_options.len();
            menu_options.push("Save Checkpoint");
            Some(index)
        } else {
            None
        };
        let logout_index = menu_options.len();
        menu_options.push("Log Out");
        let choice = tui.select_menu(&title, &menu_options, Some(state))?;

        if Some(choice) == save_index {
            let location = world
                .get_location(&state.current_location)
                .expect("the current location was validated above");
            match checkpoint.save_at(state, location) {
                Ok(()) => tui.show_message(
                    "Checkpoint saved",
                    "Your progress has been written successfully.",
                    Some(state),
                )?,
                Err(error) => tui.show_message(
                    "Checkpoint failed",
                    &format!("{error}\n\nYour progress remains unsaved."),
                    Some(state),
                )?,
            }
            continue;
        }

        if choice == logout_index {
            if handle_logout(tui, state, &world, &mut checkpoint)? {
                return Ok(());
            }
            continue;
        }

        match choice {
            0 => travel_menu(tui, state, &world)?,
            1 => interact_with_current_location(tui, state, &world)?,
            2 => region_expedition_menu(tui, state, &world)?,
            3 => combat::wander_gielinor(tui, state)?,
            4 => shops::weapon_shop(tui, state)?,
            5 => shops::armour_shop(tui, state)?,
            6 => shops::legends_store(tui, state)?,
            7 => general_store(tui, state)?,
            8 => shops::max_shop(tui, state)?,
            9 => shops::smuggler(tui, state)?,
            10 => stronghold::enter_stronghold(tui, state)?,
            11 => {
                skills::skill_plot(tui, state)?;
                continue;
            }
            12 => bank::bank_menu(tui, state)?,
            13 => quests::quest_hall(tui, state)?,
            14 => inventory::view_inventory(tui, state, false)?,
            15 => crate::achievements::progression_menu(tui, state)?,
            16 => companion_menu(tui, state)?,
            17 => companion_recovery_menu(tui, state, &world)?,
            18 => {
                rest_menu(tui, state, &world)?;
                continue;
            }
            19 => settings_menu(tui, state)?,
            _ => {}
        }
    }
}

fn nearest_checkpoint_step(state: &GameState, world: &World) -> Option<LocationId> {
    let current = &state.current_location;
    let target = world
        .get_reachable_destinations(current)
        .into_iter()
        .filter(|reachable| {
            world
                .get_location(&reachable.location_id)
                .is_some_and(|location| crate::checkpoint::is_checkpoint_location(location))
        })
        .min_by(|left, right| {
            left.total_travel_cost
                .cmp(&right.total_travel_cost)
                .then_with(|| left.location_id.cmp(&right.location_id))
        })?
        .location_id;

    world
        .get_direct_routes(current)
        .iter()
        .filter_map(|route| {
            let remaining = if route.destination_id == target {
                0
            } else {
                world
                    .get_reachable_destinations(&route.destination_id)
                    .into_iter()
                    .find(|reachable| reachable.location_id == target)
                    .map(|reachable| reachable.total_travel_cost)?
            };
            Some((
                u64::from(route.travel_cost).saturating_add(remaining),
                route.destination_id.clone(),
            ))
        })
        .min_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
        .map(|(_, destination)| destination)
}

fn travel_to_nearest_checkpoint(
    tui: &mut Tui,
    state: &mut GameState,
    world: &World,
) -> io::Result<()> {
    let mut elapsed_hours = 0_u64;
    let mut hops = 0_u32;
    loop {
        let Some(location) = world.get_location(&state.current_location) else {
            tui.show_message(
                "Checkpoint route unavailable",
                "Your current location is not present in the world graph.",
                Some(state),
            )?;
            return Ok(());
        };
        if crate::checkpoint::is_checkpoint_location(location) {
            tui.show_message(
                "Safe haven reached",
                &format!(
                    "You reached {} after {} hours of travel. Your progress is still unsaved; use Save Checkpoint before logging out.",
                    location.name, elapsed_hours
                ),
                Some(state),
            )?;
            return Ok(());
        }
        let Some(next_step) = nearest_checkpoint_step(state, world) else {
            tui.show_message(
                "No checkpoint route",
                "No tavern or guildhouse is reachable from your current location. Your progress has not been discarded.",
                Some(state),
            )?;
            return Ok(());
        };
        let (hours, recovered) = apply_travel(state, world, &next_step)
            .map_err(|error| io::Error::other(error.to_string()))?;
        elapsed_hours = elapsed_hours.saturating_add(u64::from(hours));
        hops = hops.saturating_add(1);
        crate::calendar::notify_recovered_companions(tui, state, &recovered)?;
        if hops >= 64 {
            tui.show_message(
                "Checkpoint route interrupted",
                "Travel stopped after 64 route steps to avoid following a malformed route indefinitely. Your progress remains in this session.",
                Some(state),
            )?;
            return Ok(());
        }
    }
}

fn handle_logout(
    tui: &mut Tui,
    state: &mut GameState,
    world: &World,
    checkpoint: &mut CheckpointSession,
) -> io::Result<bool> {
    if !checkpoint.is_dirty(state).map_err(io::Error::other)? {
        return Ok(true);
    }

    let current_location = world.get_location(&state.current_location);
    let can_save_here = current_location.is_some_and(crate::checkpoint::is_checkpoint_location);
    let can_travel_to_checkpoint =
        !can_save_here && nearest_checkpoint_step(state, world).is_some();

    let mut choices = vec!["Return to game"];
    let save_index = if can_save_here {
        let index = choices.len();
        choices.push("Save checkpoint and log out");
        Some(index)
    } else {
        None
    };
    let travel_index = if can_travel_to_checkpoint {
        let index = choices.len();
        choices.push("Travel to nearest save point");
        Some(index)
    } else {
        None
    };
    let discard_index = choices.len();
    choices.push("Discard unsaved progress and log out");

    let choice = tui.select_menu("Unsaved progress", &choices, Some(state))?;
    if choice == 0 {
        return Ok(false);
    }
    if Some(choice) == save_index {
        let location = current_location.expect("save eligibility requires a known location");
        match checkpoint.save_at(state, location) {
            Ok(()) => {
                tui.show_message(
                    "Checkpoint saved",
                    "Your progress has been written successfully. You are now logged out.",
                    Some(state),
                )?;
                return Ok(true);
            }
            Err(error) => {
                tui.show_message(
                    "Checkpoint failed",
                    &format!("{error}\n\nYou remain logged in and your progress is still unsaved."),
                    Some(state),
                )?;
                return Ok(false);
            }
        }
    }
    if Some(choice) == travel_index {
        travel_to_nearest_checkpoint(tui, state, world)?;
        return Ok(false);
    }
    if choice == discard_index {
        let confirmation = tui.prompt_input(
            "Type DISCARD to abandon all progress since the last checkpoint",
            false,
            Some(state),
        )?;
        if confirmation == "DISCARD" {
            tui.show_message(
                "Logged out without saving",
                "Unsaved gameplay changes were not written. The next login will load the last successful checkpoint. A crash or power loss has the same effect on unsaved changes.",
                Some(state),
            )?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Interact with the current location using a stable machine-facing ID.
fn interact_with_current_location(
    tui: &mut Tui,
    state: &mut GameState,
    world: &World,
) -> io::Result<()> {
    let Some(location) = world.get_location(&state.current_location) else {
        tui.show_message(
            "Explore",
            "The current location is not known to this world.",
            Some(state),
        )?;
        return Ok(());
    };
    let target_id = location.id.as_str().to_string();
    let location_name = location.name.clone();
    crate::quests::notify_event(
        state,
        &crate::event::GameEvent::TargetInteracted { target_id },
    );
    if location.id.as_str() == "village_start" {
        crate::adventure::notify_landmark_interaction(state, "village_supply_board");
        if state
            .active_quests
            .iter()
            .any(|q| q.id == crate::adventure::REGION_QUEST_ID && q.is_completed && !q.is_claimed)
        {
            match crate::adventure::claim_region_quest(state) {
                Ok(reward) => tui.show_message(
                    "Oakhaven supplies recovered",
                    &format!("The village receives its supplies. Quest reward: {reward:?}; bonus: {} Logs and {} XP.", crate::adventure::REGION_QUEST_REWARD_LOGS, crate::adventure::REGION_QUEST_REWARD_XP),
                    Some(state),
                )?,
                Err(error) => tui.show_message("Quest not ready", &error.to_string(), Some(state))?,
            }
        }
    } else if let Some(landmark) = crate::adventure::landmark_for_location(&location.id) {
        crate::adventure::notify_landmark_interaction(state, landmark.interaction_target);
    }
    let recovered = state.advance_world_time(crate::calendar::LOCATION_INTERACTION_HOURS);
    crate::calendar::notify_recovered_companions(tui, state, &recovered)?;
    tui.show_message(
        "Explore",
        &format!(
            "You inspect {location_name}. The interaction takes {} in-game hour(s).\n\n{}",
            crate::calendar::LOCATION_INTERACTION_HOURS,
            crate::calendar::format_calendar(state.world_turn)
        ),
        Some(state),
    )
}

/// Run the reusable encounter table attached to the current region location.
/// Each victory uses the party-aware combat engine and the shared domain events.
fn region_expedition_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    use crate::party_battle::{run_party_battle, BattleConfig, BattleOutcome};

    let location_id = state.current_location.clone();
    let Some(encounter) = crate::adventure::encounter_for_location(&location_id) else {
        let detail = world.get_location(&location_id)
            .map(|place| format!("{} has no hostile encounter on its table. You can still inspect landmarks, gather resources, trade, or travel onwards.", place.name))
            .unwrap_or_else(|| "This location is not part of the current region.".to_string());
        tui.show_message("No expedition encounter", &detail, Some(state))?;
        return Ok(());
    };
    let encounter = *encounter;
    let already_cleared = state
        .region_progress
        .cleared_encounters
        .iter()
        .any(|id| id == encounter.id);
    let roster = encounter
        .enemies
        .iter()
        .map(|enemy| {
            format!(
                "{} · level {} · {} gold · {}",
                enemy.name,
                state.player.combat_level.saturating_add(enemy.level_offset),
                enemy.gold_reward,
                enemy
                    .item_reward
                    .map(|(item, count)| format!("{} x{count}", item.name()))
                    .unwrap_or_else(|| "no item drop".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let danger = match encounter.danger {
        5.. => {
            "Extreme danger: prepare equipment, healing, and your active party before committing."
        }
        4 => "High danger: enemy health and damage are increased.",
        _ => "Moderate danger: check supplies and equipment before travelling deeper.",
    };
    tui.show_message(
        "Expedition briefing",
        &format!(
            "{}\nDanger: {}/5\n{}\n\nEncounter roster:\n{}\n\nClear reward: {} gold and {} XP.{}",
            encounter.title,
            encounter.danger,
            danger,
            roster,
            encounter.reward_gold,
            encounter.reward_xp,
            if already_cleared {
                "\nThe one-time clear reward was already claimed."
            } else {
                ""
            }
        ),
        Some(state),
    )?;

    for enemy in encounter.enemies {
        let level = state
            .player
            .combat_level
            .saturating_add(enemy.level_offset)
            .max(1);
        let enemy_hp = (80_u32.saturating_add(level.saturating_mul(22)))
            .saturating_mul(enemy.health_multiplier_pct)
            / 100;
        let attack = enemy
            .kind
            .max_damage()
            .saturating_mul(enemy.attack_multiplier_pct)
            / 100;
        match run_party_battle(
            tui,
            state,
            BattleConfig {
                enemy_name: enemy.name,
                level,
                enemy_hp: enemy_hp.max(1),
                enemy_attack_power: attack.max(1),
                allow_inventory: true,
                allow_flee: true,
                resource_drain_per_turn: 0,
            },
        )? {
            BattleOutcome::Victory => {
                tui.show_message(
                    "Expedition progress",
                    &format!(
                        "Defeated {}. Its loot and quest progress have been recorded.",
                        enemy.name
                    ),
                    Some(state),
                )?;
            }
            BattleOutcome::Fled => {
                tui.show_message(
                    "Expedition withdrawn",
                    "You retreat from the encounter. No clear reward is granted for an unfinished expedition.",
                    Some(state),
                )?;
                return Ok(());
            }
            BattleOutcome::Defeated => return Ok(()),
        }
    }
    if !already_cleared {
        match crate::adventure::claim_encounter_clear_reward(state, encounter.id) {
            Ok((gold, xp)) => tui.show_message(
                "Expedition complete",
                &format!("The area is secure. Clear reward: {gold} gold and {xp} XP."),
                Some(state),
            )?,
            Err(error) => {
                tui.show_message("Reward unavailable", &error.to_string(), Some(state))?
            }
        }
    } else {
        tui.show_message(
            "Expedition complete",
            "The area has been cleared again, but its one-time clear reward will not be paid twice.",
            Some(state),
        )?;
    }
    Ok(())
}

/// Rest trades resources and world time for stamina recovery.
/// Present safe-haven and field-camp rest choices. All state changes are
/// delegated to the domain rest module so costs, recovery and elapsed time are
/// validated and committed together.
fn rest_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    use crate::rest::{execute_rest, plan_rest, RestAction, RestContext, RestExecution, RestKind};

    let Some(location) = world.get_location(&state.current_location) else {
        tui.show_message(
            "Rest unavailable",
            "The current location is not known to this world. No resources or time were spent.",
            Some(state),
        )?;
        return Ok(());
    };
    let location_name = location.name.clone();
    let choices = [
        (RestAction::ShortRest, RestKind::Short),
        (RestAction::LongRest, RestKind::Long),
        (RestAction::FieldCamp, RestKind::FieldCamp),
    ];
    let mut labels = Vec::with_capacity(choices.len() + 1);
    for (_, kind) in choices {
        let label = match plan_rest(&state.player, location, kind, RestContext::MainMenu) {
            Ok(plan) => {
                let recovery = plan.recovery();
                let cost = if plan.gold_cost > 0 {
                    format!("{} gold", plan.gold_cost)
                } else if plan.food_cost > 0 {
                    format!("{} Food", plan.food_cost)
                } else {
                    "free".to_string()
                };
                format!(
                    "{} · {}h · {} · HP +{} · stamina +{} · mana +{}",
                    kind.label(),
                    plan.hours,
                    cost,
                    recovery.hp,
                    recovery.stamina,
                    recovery.mana
                )
            }
            Err(error) => format!("[Unavailable: {}] {}", error, kind.label()),
        };
        labels.push(label);
    }
    labels.push("Back".to_string());

    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    let choice = tui.select_menu(
        &format!(
            "Rest at {location_name} · HP {}/{} · Stamina {}/{} · Mana {}/{} · {}",
            state.player.hp,
            state.player.max_hp,
            state.player.stamina,
            state.player.max_stamina,
            state.player.mana,
            state.player.max_mana,
            crate::calendar::format_calendar(state.world_turn)
        ),
        &options,
        Some(state),
    )?;
    if choice >= choices.len() {
        // Choosing Back is a strict no-op: no cost, recovery or clock mutation.
        let _ = execute_rest(state, world, RestAction::Cancel, RestContext::MainMenu);
        return Ok(());
    }

    match execute_rest(state, world, choices[choice].0, RestContext::MainMenu) {
        Ok(RestExecution::Cancelled) => Ok(()),
        Ok(RestExecution::Completed(outcome)) => {
            let recovered = if outcome.recovered_companions.is_empty() {
                "None completed recovery during this rest.".to_string()
            } else {
                outcome
                    .recovered_companions
                    .iter()
                    .map(|id| crate::party::get_definition(*id).name)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let outstanding_recovery = if state.companion_recovery.records.is_empty() {
                "None.".to_string()
            } else {
                state
                    .companion_recovery
                    .records
                    .iter()
                    .map(|record| {
                        let name = crate::party::get_definition(record.companion_id).name;
                        match record.destination.as_ref().and_then(|id| world.get_location(id)) {
                            Some(location) => format!(
                                "{name}: {} elapsed hour(s) remaining at {}",
                                record.turns_remaining, location.name
                            ),
                            None => format!(
                                "{name}: awaiting a reachable healer; recovery timer paused at {} hour(s)",
                                record.turns_remaining
                            ),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let plan = outcome.plan;
            let cost = format!("{} gold and {} Food", plan.gold_cost, plan.food_cost);
            tui.show_message(
                "Rest complete",
                &format!(
                    "{} at {location_name}.
Cost: {cost}.
Time consumed: {} in-game hour(s).

HP: {}/{} → {}/{}
Stamina: {}/{} → {}/{}
Mana: {}/{} → {}/{}

Companions recovered: {recovered}

Outstanding companion recovery:
{outstanding_recovery}

Rest never replaces immediate treatment or erases a knockout record without a reachable healer.

{}",
                    plan.kind.label(),
                    plan.hours,
                    plan.before.hp,
                    state.player.max_hp,
                    plan.after.hp,
                    state.player.max_hp,
                    plan.before.stamina,
                    state.player.max_stamina,
                    plan.after.stamina,
                    state.player.max_stamina,
                    plan.before.mana,
                    state.player.max_mana,
                    plan.after.mana,
                    state.player.max_mana,
                    crate::calendar::format_calendar(state.world_turn)
                ),
                Some(state),
            )
        }
        Err(error) => tui.show_message(
            "Rest unavailable",
            &format!(
                "{error}

No resources or world time were spent."
            ),
            Some(state),
        ),
    }
}

/// Applies a directly connected journey after validating its route. The
/// route's existing cost value is its elapsed travel duration in hours.
fn apply_travel(
    state: &mut GameState,
    world: &World,
    destination_id: &LocationId,
) -> Result<(u32, Vec<crate::party::CompanionId>), String> {
    let current_id = state.current_location.clone();
    let route = world
        .get_direct_routes(&current_id)
        .iter()
        .find(|route| &route.destination_id == destination_id)
        .ok_or_else(|| {
            "That destination is not directly connected to your current location.".to_string()
        })?;
    if route.travel_cost == 0 {
        return Err("This route has an invalid travel cost.".into());
    }
    let destination = world
        .get_location(destination_id)
        .ok_or_else(|| "That destination is not known to the world.".to_string())?;
    let travel_cost = route.travel_cost;
    let gates = [route.access_gate.as_ref(), destination.access_gate.as_ref()];
    let access = crate::access::evaluate_access_gates(state, &gates);
    if !access.allowed {
        return Err(if destination.hidden_when_locked {
            "That destination is not currently available.".into()
        } else {
            access.summary()
        });
    }

    // Stage arrival, event progression and elapsed time on a clone. Entry
    // costs are committed only after the transition has succeeded.
    let mut next_state = state.clone();
    next_state.current_location = destination.id.clone();
    crate::quests::notify_event(
        &mut next_state,
        &crate::event::GameEvent::LocationTraversed {
            from_location_id: current_id,
            to_location_id: destination.id.clone(),
        },
    );
    crate::quests::notify_event(
        &mut next_state,
        &crate::event::GameEvent::LocationReached {
            location_id: destination.id.clone(),
        },
    );
    let recovered = next_state.advance_world_time(u64::from(travel_cost));
    crate::access::consume_entry_costs_atomically(&mut next_state, &gates)?;
    *state = next_state;
    Ok((travel_cost, recovered))
}

struct TravelDestination {
    id: LocationId,
    name: String,
    travel_hours: u32,
    danger_level: u32,
    access: crate::access::AccessResult,
}

fn travel_destinations(state: &GameState, world: &World) -> Vec<TravelDestination> {
    world
        .get_direct_routes(&state.current_location)
        .iter()
        .filter_map(|route| {
            let location = world.get_location(&route.destination_id)?;
            let gates = [route.access_gate.as_ref(), location.access_gate.as_ref()];
            let access = crate::access::evaluate_access_gates(state, &gates);
            if location.hidden_when_locked && !access.allowed {
                return None;
            }
            Some(TravelDestination {
                id: location.id.clone(),
                name: location.name.clone(),
                travel_hours: route.travel_cost,
                danger_level: route.danger_level,
                access,
            })
        })
        .collect()
}

fn travel_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    loop {
        let current_name = world
            .get_location(&state.current_location)
            .map(|location| location.name.clone())
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Unknown current location '{}'.",
                    state.current_location
                ))
            })?;
        let destinations = travel_destinations(state, world);
        if destinations.is_empty() {
            tui.show_message(
                "No routes available",
                "There are no currently visible direct travel destinations from this location.",
                Some(state),
            )?;
            return Ok(());
        }

        let labels: Vec<String> = destinations
            .iter()
            .map(|destination| {
                let locked = if destination.access.allowed {
                    String::new()
                } else {
                    format!(
                        " · LOCKED — {}",
                        destination.access.summary().replace('\n', " ")
                    )
                };
                format!(
                    "{} · travel time {}h · danger {}{}",
                    destination.name, destination.travel_hours, destination.danger_level, locked
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Travel from {current_name}"),
            &options,
            Some(state),
        )?;
        if choice >= destinations.len() {
            return Ok(());
        }

        let destination = &destinations[choice];
        match apply_travel(state, world, &destination.id) {
            Ok((travel_hours, recovered)) => {
                crate::calendar::notify_recovered_companions(tui, state, &recovered)?;
                tui.show_message(
                    "Travel complete",
                    &format!(
                        "You arrive at {}.\n\nTravel time: {travel_hours} hours\nRoute danger: {}\n\n{}",
                        destination.name,
                        destination.danger_level,
                        crate::calendar::format_calendar(state.world_turn)
                    ),
                    Some(state),
                )?;
                return Ok(());
            }
            Err(message) => tui.show_message("Travel unavailable", &message, Some(state))?,
        }
    }
}

fn general_store(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let items = [
        ItemType::GoblinMail,
        ItemType::HugeBones,
        ItemType::DragonHides,
        ItemType::RunicTablets,
        ItemType::Potions,
        ItemType::Food,
        ItemType::Seeds,
        ItemType::Fur,
        ItemType::Ore,
        ItemType::Logs,
        ItemType::Gemstones,
        ItemType::MagicOrbs,
        ItemType::RawFood,
        ItemType::Bait,
        ItemType::Ingots,
        ItemType::Bows,
    ];

    loop {
        let labels: Vec<String> = items
            .iter()
            .map(|item| {
                format!(
                    "{} · owned {} · buy {}g / sell {}g",
                    item.name(),
                    state.player.get_item_count(*item),
                    crate::mastery::vendor_price(
                        state,
                        crate::economy::calculate_dynamic_buy_price(state, *item),
                        0
                    ),
                    crate::economy::calculate_dynamic_sell_price(state, *item)
                )
            })
            .collect();
        let mut options: Vec<&str> = labels.iter().map(String::as_str).collect();
        options.push("Back");
        let choice = tui.select_menu(
            &format!("Trading Post · {} gold", state.player.gold),
            &options,
            Some(state),
        )?;
        if choice >= items.len() {
            return Ok(());
        }
        buy_sell_item(tui, state, items[choice])?;
    }
}

fn buy_sell_item(tui: &mut Tui, state: &mut GameState, item: ItemType) -> io::Result<()> {
    loop {
        let choice = tui.select_menu(
            &format!(
                "{} · owned {}",
                item.name(),
                state.player.get_item_count(item)
            ),
            &["Buy", "Sell", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let amount_text = tui.prompt_input("Amount to buy", false, Some(state))?;
                let amount = match amount_text.parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => {
                        tui.show_message(
                            "Purchase failed",
                            "Enter a whole number greater than zero.",
                            Some(state),
                        )?;
                        continue;
                    }
                };
                match crate::economy::buy_item_transaction(state, item, amount) {
                    Ok(cost) => tui.show_message(
                        "Purchase complete",
                        &format!("Bought {} {} for {} gold.", amount, item.name(), cost),
                        Some(state),
                    )?,
                    Err(error) => {
                        tui.show_message("Purchase failed", &error.to_string(), Some(state))?
                    }
                }
            }
            1 => {
                let amount_text =
                    tui.prompt_input("Amount to sell, or type all", false, Some(state))?;
                let owned = state.player.get_item_count(item);
                let amount = if amount_text.eq_ignore_ascii_case("all") {
                    owned
                } else {
                    match amount_text.parse::<u32>() {
                        Ok(value) => value,
                        Err(_) => {
                            tui.show_message(
                                "Sale failed",
                                "Enter a whole number or type all.",
                                Some(state),
                            )?;
                            continue;
                        }
                    }
                };
                match crate::economy::sell_item_transaction(state, item, amount) {
                    Ok(revenue) => tui.show_message(
                        "Sale complete",
                        &format!("Sold {} {} for {} gold.", amount, item.name(), revenue),
                        Some(state),
                    )?,
                    Err(error) => {
                        tui.show_message("Sale failed", &error.to_string(), Some(state))?
                    }
                }
            }
            _ => return Ok(()),
        }
    }
}

fn companion_recovery_menu(tui: &mut Tui, state: &mut GameState, world: &World) -> io::Result<()> {
    use crate::companion_recovery::{RecoveryError, TREATMENT_COST};

    loop {
        let mut labels = Vec::new();
        let mut ids = Vec::new();
        for record in &state.companion_recovery.records {
            if let Some(companion) = state.companions.iter().find(|c| {
                c.id == record.companion_id
                    && crate::companion_discovery::is_discovered(state, c.id)
            }) {
                let destination = record
                    .destination
                    .as_ref()
                    .and_then(|id| world.get_location(id))
                    .map(|location| location.name.as_str())
                    .unwrap_or("No reachable healer / treatment facility");
                labels.push(format!(
                    "{} · {} · {} elapsed hour(s) remaining · treatment {}g",
                    companion.display_name(),
                    destination,
                    record.turns_remaining,
                    TREATMENT_COST
                ));
                ids.push(record.companion_id);
            }
        }
        labels.push("Back".to_string());
        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let choice = tui.select_menu("Companion Recovery", &options, Some(state))?;
        if choice >= ids.len() {
            return Ok(());
        }
        let id = ids[choice];
        let record = state.companion_recovery.record(id).cloned();
        let destination = record.as_ref().and_then(|r| r.destination.clone());
        let Some(destination) = destination else {
            tui.show_message(
                "Treatment unavailable",
                "No reachable location with a healer service was found when the companion was knocked out. Their knockout record and unavailable state are preserved; neither resting nor camping can bypass the missing treatment route.",
                Some(state),
            )?;
            continue;
        };
        let destination_name = world
            .get_location(&destination)
            .map(|loc| loc.name.clone())
            .unwrap_or_else(|| destination.to_string());
        let is_here = state.current_location == destination;
        let mut detail = format!(
            "{} is being treated at {}.\nRecovery requirement: {} elapsed in-game hour(s).\nImmediate treatment: {} gold.\nCurrent gold: {}.",
            crate::party::get_definition(id).name,
            destination_name,
            record.as_ref().map(|r| r.turns_remaining).unwrap_or(0),
            TREATMENT_COST,
            state.player.gold
        );
        if !is_here {
            detail.push_str(
                "\n\nTravel to that treatment location before paying for immediate treatment.",
            );
        }
        tui.show_message("Recovery details", &detail, Some(state))?;
        let choices = if is_here {
            vec!["Pay for immediate treatment", "Back"]
        } else {
            vec!["Back"]
        };
        let selected = tui.select_menu("Recovery details", &choices, Some(state))?;
        if !is_here || selected != 0 {
            continue;
        }
        let treatment_location = state.current_location.clone();
        let treatment_result = {
            let recovery = &mut state.companion_recovery;
            let companion = state
                .companions
                .iter_mut()
                .find(|companion| companion.id == id);
            let Some(companion) = companion else {
                return Err(io::Error::other(
                    "Recovery record refers to a missing companion.",
                ));
            };
            let result = recovery.apply_treatment_parts(
                &mut state.player.gold,
                &mut state.party,
                companion,
                id,
                &treatment_location,
                world,
            );
            result
        };
        match treatment_result {
            Ok(cost) => tui.show_message(
                "Treatment complete",
                &format!("{} has recovered. {cost} gold paid; companion is available but has not been automatically added to the party.", crate::party::get_definition(id).name),
                Some(state),
            )?,
            Err(error) => {
                let title = if matches!(error, RecoveryError::InsufficientGold { .. }) {
                    "Treatment unaffordable"
                } else {
                    "Treatment unavailable"
                };
                tui.show_message(title, &error.to_string(), Some(state))?;
            }
        }
    }
}

fn companion_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let party_count = state.party.companion_count();
        let party_limit = state.party.max_companions();
        let roster_count = state.companions.len();
        let discovered_count = crate::companion_discovery::discovered_definitions(state).len();
        let title = format!(
            "Companions & Party · {} discovered · {} recruited · {}/{} party slots",
            discovered_count, roster_count, party_count, party_limit
        );
        let choice = tui.select_menu(
            &title,
            &["View Roster", "Manage Party", "Back"],
            Some(state),
        )?;
        match choice {
            0 => view_roster(tui, state)?,
            1 => manage_party(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn view_roster(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    use crate::party::{build_roster_actions, resolve_roster_action, PartyManager, RosterAction};

    let definitions = crate::companion_discovery::discovered_definitions(state);
    if definitions.is_empty() {
        tui.show_message(
            "Companion Roster",
            "No companion leads have been discovered yet. Explore landmarks and complete local objectives to meet potential companions.",
            Some(state),
        )?;
        return Ok(());
    }

    loop {
        let mut labels: Vec<String> = definitions
            .iter()
            .map(|def| {
                let recruited = state.companions.iter().find(|c| c.id == def.id);
                let status = match recruited {
                    Some(c) => format!("Recruited · {}", c.condition.name()),
                    None => {
                        let mut req = String::new();
                        if def.recruitment_level > 0 {
                            req.push_str(&format!("Lvl {}", def.recruitment_level));
                        }
                        if def.recruitment_cost > 0 {
                            if !req.is_empty() {
                                req.push_str(" · ");
                            }
                            req.push_str(&format!("{}g", def.recruitment_cost));
                        }
                        if req.is_empty() {
                            "Not recruited".to_string()
                        } else {
                            format!("Not recruited · {}", req)
                        }
                    }
                };
                format!("{} · {} · {}", def.name, def.role.name(), status)
            })
            .collect();
        labels.push("Back".to_string());

        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let choice = tui.select_menu("Companion Roster", &options, Some(state))?;
        if choice >= definitions.len() {
            return Ok(());
        }

        let def = &definitions[choice];
        let recruited = state.companions.iter().find(|c| c.id == def.id);

        if let Some(companion) = recruited {
            // Show companion details and options
            let in_party = state.party.contains(def.id);
            let can_add_to_party =
                !in_party && companion.can_join_party() && state.party.has_room();

            let mut detail = format!(
                "Name: {}\nRole: {}\nCondition: {}\nHP: {}/{}\nAttack: {}\nDefence: {}\nWeapon: {}\nArmour: {}",
                def.name,
                def.role.name(),
                companion.condition.name(),
                companion.current_hp,
                companion.max_hp,
                companion.attack,
                companion.defence,
                companion.equipped_weapon.name(),
                companion.equipped_armour.name(),
            );
            if in_party {
                detail.push_str("\n\nCurrently in active party.");
            }

            let actions = build_roster_actions(in_party, can_add_to_party);
            let action_labels: Vec<&str> = actions.iter().map(|a| a.label()).collect();

            let action_choice = tui.select_menu(def.name, &action_labels, Some(state))?;
            let resolved = resolve_roster_action(&actions, action_choice);

            match resolved {
                RosterAction::Details => {
                    tui.show_message(def.name, &detail, Some(state))?;
                }
                RosterAction::RemoveFromParty => {
                    match PartyManager::remove_from_party(&mut state.party, def.id) {
                        Ok(()) => tui.show_message(
                            "Party updated",
                            &format!("{} removed from the party.", def.name),
                            Some(state),
                        )?,
                        Err(e) => {
                            tui.show_message("Could not remove", &e.to_string(), Some(state))?
                        }
                    }
                }
                RosterAction::AddToParty => {
                    match PartyManager::add_to_party(&mut state.party, &state.companions, def.id) {
                        Ok(()) => tui.show_message(
                            "Party updated",
                            &format!("{} added to the party.", def.name),
                            Some(state),
                        )?,
                        Err(e) => tui.show_message("Could not add", &e.to_string(), Some(state))?,
                    }
                }
                RosterAction::Dismiss => {
                    match PartyManager::dismiss(&mut state.companions, &mut state.party, def.id) {
                        Ok(()) => tui.show_message(
                            "Companion dismissed",
                            &format!("{} has left your company.", def.name),
                            Some(state),
                        )?,
                        Err(e) => {
                            tui.show_message("Could not dismiss", &e.to_string(), Some(state))?
                        }
                    }
                }
                RosterAction::Back => {
                    // Return to roster selection without modifying any state.
                }
            }
        } else {
            // Not recruited - show recruitment option
            let detail = format!(
                "Name: {}\nRole: {}\n\n{}\n\nBase HP: {}\nBase Attack: {}\nBase Defence: {}\n\nRecruitment requires: Level {} · {} gold",
                def.name,
                def.role.name(),
                def.description,
                def.base_hp,
                def.base_attack,
                def.base_defence,
                def.recruitment_level,
                def.recruitment_cost,
            );
            let action_choice = tui.select_menu(
                &format!("{} · Recruit", def.name),
                &["View Details", "Recruit", "Back"],
                Some(state),
            )?;
            match action_choice {
                0 => {
                    tui.show_message(def.name, &detail, Some(state))?;
                }
                1 => {
                    match PartyManager::recruit(
                        &mut state.companions,
                        def.id,
                        state.player.combat_level,
                        state.player.gold,
                    ) {
                        Ok(cost) => {
                            state.player.gold -= cost;
                            tui.show_message(
                                "Companion recruited",
                                &format!(
                                    "{} has joined your company!\nGold spent: {}",
                                    def.name, cost
                                ),
                                Some(state),
                            )?;
                        }
                        Err(e) => {
                            tui.show_message("Recruitment failed", &e.to_string(), Some(state))?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn visible_party_members(state: &GameState) -> Vec<(crate::party::CompanionId, String, String)> {
    // Keep each visible label paired with the ID it represents. A save with
    // a party ID that has no matching recruited companion is filtered out,
    // without shifting a later selection onto another ID.
    state
        .party
        .members()
        .iter()
        .filter_map(|&id| {
            if !crate::companion_discovery::is_discovered(state, id) {
                return None;
            }
            state.companions.iter().find(|c| c.id == id).map(|c| {
                (
                    id,
                    format!(
                        "{} · {} · HP {}/{}",
                        c.display_name(),
                        c.role().name(),
                        c.current_hp,
                        c.max_hp
                    ),
                    c.display_name().to_string(),
                )
            })
        })
        .collect()
}

fn manage_party(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    use crate::party::{
        build_party_menu_actions, resolve_party_menu_action, PartyManager, PartyMenuAction,
    };

    loop {
        let party_members = visible_party_members(state);

        let member_count = party_members.len();
        let menu_actions = build_party_menu_actions(member_count);

        // Build display labels that match the menu_actions list exactly.
        // When the party is empty, show "No companions in party" as an
        // informational line in the title instead of a selectable entry.
        let mut labels: Vec<String> = party_members
            .iter()
            .map(|(_, label, _)| label.clone())
            .collect();
        labels.push(format!(
            "Add companion ({}/{})",
            state.party.companion_count(),
            state.party.max_companions()
        ));
        labels.push("Back".to_string());

        let options: Vec<&str> = labels.iter().map(String::as_str).collect();
        let title = if member_count == 0 {
            "Active Party · no companions · select Add companion or Back".to_string()
        } else {
            format!(
                "Active Party · {} companion{}",
                state.party.companion_count(),
                if state.party.companion_count() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        let choice = tui.select_menu(&title, &options, Some(state))?;

        let resolved = resolve_party_menu_action(&menu_actions, choice);

        match resolved {
            PartyMenuAction::Member(index) => {
                // Resolve through the same filtered list used to render the menu,
                // rather than indexing the raw saved party IDs.
                let Some((id, _, name)) = party_members.get(index) else {
                    tui.show_message(
                        "Party updated",
                        "That party member is no longer available. The menu has been refreshed.",
                        Some(state),
                    )?;
                    continue;
                };
                let id = *id;
                let name = name.clone();
                match PartyManager::remove_from_party(&mut state.party, id) {
                    Ok(()) => {
                        tui.show_message(
                            "Party updated",
                            &format!("{} removed from the party.", name),
                            Some(state),
                        )?;
                    }
                    Err(e) => {
                        tui.show_message("Could not remove", &e.to_string(), Some(state))?;
                    }
                }
            }
            PartyMenuAction::AddCompanion => {
                let available: Vec<_> = state
                    .companions
                    .iter()
                    .filter(|c| {
                        crate::companion_discovery::is_discovered(state, c.id)
                            && c.can_join_party()
                            && !state.party.contains(c.id)
                    })
                    .collect();
                if available.is_empty() {
                    tui.show_message(
                        "No companions available",
                        "All recruited companions are already in the party, unavailable, or you have no companions.",
                        Some(state),
                    )?;
                    continue;
                }
                let add_labels: Vec<String> = available
                    .iter()
                    .map(|c| {
                        format!(
                            "{} · {} · HP {}/{}",
                            c.display_name(),
                            c.role().name(),
                            c.current_hp,
                            c.max_hp
                        )
                    })
                    .collect();
                let mut add_options: Vec<&str> = add_labels.iter().map(String::as_str).collect();
                add_options.push("Back");
                let add_choice = tui.select_menu("Add to Party", &add_options, Some(state))?;
                if add_choice < available.len() {
                    let id = available[add_choice].id;
                    match PartyManager::add_to_party(&mut state.party, &state.companions, id) {
                        Ok(()) => {
                            tui.show_message(
                                "Party updated",
                                &format!(
                                    "{} added to the party.",
                                    available[add_choice].display_name()
                                ),
                                Some(state),
                            )?;
                        }
                        Err(e) => {
                            tui.show_message("Could not add", &e.to_string(), Some(state))?;
                        }
                    }
                }
            }
            PartyMenuAction::Back => {
                return Ok(());
            }
        }
    }
}

fn settings_menu(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice =
            tui.select_menu("Settings", &["Admin Commands (Debug)", "Back"], Some(state))?;
        match choice {
            0 => verify_admin(tui, state)?,
            _ => return Ok(()),
        }
    }
}

fn verify_admin(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    let code = tui.prompt_input("Administrator login code", false, Some(state))?;
    if code != "741" {
        tui.show_message("Access denied", "Invalid code.", Some(state))?;
        return Ok(());
    }

    let pin = tui.prompt_input("Administrator verification PIN", true, Some(state))?;
    if pin != "9763" {
        tui.show_message("Access denied", "Invalid PIN.", Some(state))?;
        return Ok(());
    }
    admin_commands(tui, state)
}

fn admin_commands(tui: &mut Tui, state: &mut GameState) -> io::Result<()> {
    loop {
        let choice = tui.select_menu(
            "Developer settings",
            &["Edit a variable", "List variables", "Back"],
            Some(state),
        )?;
        match choice {
            0 => {
                let command = tui.prompt_input("set variable=value", false, Some(state))?;
                if let Some((var, val)) = command.split_once('=') {
                    let var = var.trim();
                    let val = val.trim();
                    let updated = match var {
                        "combat_level" => val.parse::<u32>().map(|v| state.player.combat_level = v).is_ok(),
                        "xp" => val.parse::<u32>().map(|v| state.player.xp = v).is_ok(),
                        "gold" => val.parse::<u32>().map(|v| state.player.gold = v).is_ok(),
                        "hp" => val.parse::<u32>().map(|v| state.player.hp = v).is_ok(),
                        "max_hp" => val.parse::<u32>().map(|v| state.player.max_hp = v).is_ok(),
                        "stronghold_kill_count" => val.parse::<u32>().map(|v| state.player.stronghold_kill_count = v).is_ok(),
                        "woodcutting_lvl" => val.parse::<u32>().map(|v| state.player.skills.woodcutting.level = v).is_ok(),
                        "cooking_lvl" => val.parse::<u32>().map(|v| state.player.skills.cooking.level = v).is_ok(),
                        "fishing_lvl" => val.parse::<u32>().map(|v| state.player.skills.fishing.level = v).is_ok(),
                        "mining_lvl" => val.parse::<u32>().map(|v| state.player.skills.mining.level = v).is_ok(),
                        "smithing_lvl" => val.parse::<u32>().map(|v| state.player.skills.smithing.level = v).is_ok(),
                        "thieving_lvl" => val.parse::<u32>().map(|v| state.player.skills.thieving.level = v).is_ok(),
                        _ => false,
                    };
                    let detail = if updated {
                        format!("Set {} to {}.", var, val)
                    } else {
                        "Unknown variable or invalid value.".to_string()
                    };
                    tui.show_message(
                        if updated { "Variable updated" } else { "Variable not updated" },
                        &detail,
                        Some(state),
                    )?;
                } else {
                    tui.show_message("Invalid command", "Use the format set variable=value.", Some(state))?;
                }
            }
            1 => tui.show_text(
                "Editable variables",
                "combat_level\nxp\ngold\nhp\nmax_hp\nstronghold_kill_count\nwoodcutting_lvl\ncooking_lvl\nfishing_lvl\nmining_lvl\nsmithing_lvl\nthieving_lvl",
                Some(state),
            )?,
            _ => return Ok(()),
        }
    }
}

#[cfg(test)]
mod travel_tests {
    use super::*;
    use crate::state::Player;

    fn test_state() -> GameState {
        GameState::new(Player::new("traveller".into(), "test-hash".into()))
    }

    #[test]
    fn party_menu_selection_stays_aligned_when_a_saved_member_is_missing() {
        use crate::party::{
            build_party_menu_actions, resolve_party_menu_action, CompanionId, CompanionState,
            Party, PartyMenuAction,
        };

        let mut state = test_state();
        let lyra = crate::party::get_definition(CompanionId::LyraSwiftbow);
        state
            .companions
            .push(CompanionState::new_from_definition(&lyra));
        state.party = serde_json::from_str::<Party>(
            r#"{"members":["bran_the_shield","lyra_swiftbow"],"size_limit":4}"#,
        )
        .expect("the simulated inconsistent party should deserialize");

        let visible = visible_party_members(&state);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].0, CompanionId::LyraSwiftbow);
        assert!(visible[0].1.starts_with("Lyra Swiftbow"));

        let actions = build_party_menu_actions(visible.len());
        assert_eq!(
            resolve_party_menu_action(&actions, 0),
            PartyMenuAction::Member(0)
        );
        assert_eq!(
            visible[0].0,
            CompanionId::LyraSwiftbow,
            "the displayed first row must map back to Lyra, not the missing Bran ID"
        );
    }

    #[test]
    fn direct_travel_updates_location_after_route_validation() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let destination = LocationId::from("abandoned_mill");

        let (travel_hours, _) = apply_travel(&mut state, &world, &destination)
            .expect("direct route should be traversable");

        assert_eq!(travel_hours, 5);
        assert_eq!(state.world_turn, 5);
        assert_eq!(state.current_location, destination);
    }

    #[test]
    fn non_adjacent_destination_is_rejected_without_changing_location() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let starting_location = state.current_location.clone();
        let destination = LocationId::from("goblin_caves");

        assert!(world.is_reachable(&starting_location, &destination));
        assert!(!world.can_travel(&starting_location, &destination));
        assert!(apply_travel(&mut state, &world, &destination).is_err());
        assert_eq!(state.current_location, starting_location);
        assert_eq!(state.world_turn, 0);
    }

    #[test]
    fn unknown_destination_is_rejected_without_changing_location() {
        let world = build_initial_world().expect("initial world should be valid");
        let mut state = test_state();
        let starting_location = state.current_location.clone();

        assert!(apply_travel(&mut state, &world, &LocationId::from("unknown_location")).is_err());
        assert_eq!(state.current_location, starting_location);
        assert_eq!(state.world_turn, 0);
    }
}

#[cfg(test)]
mod access_travel_tests {
    use super::*;
    use crate::state::Player;

    fn test_state() -> GameState {
        GameState::new(Player::new("access-travel-test".into(), "test-hash".into()))
    }

    #[test]
    fn hidden_location_is_hidden_and_direct_travel_is_rejected_until_unlocked() {
        use crate::access::{AccessGate, AccessRequirement};
        use crate::world::{Location, LocationCategory};

        let mut world = build_initial_world().expect("initial world should be valid");
        let village = LocationId::from("village_start");
        let hidden = LocationId::from("hidden_archive");
        world
            .add_location(
                Location::new(
                    hidden.clone(),
                    "Hidden Archive",
                    "A concealed chamber.",
                    LocationCategory::Landmark,
                )
                .with_access_gate(
                    AccessGate {
                        all_of: vec![AccessRequirement::RealmDiscovered {
                            realm_id: "oakhaven".into(),
                        }],
                        ..AccessGate::default()
                    },
                    true,
                ),
            )
            .unwrap();
        world.add_route(&village, &hidden, 2, 0).unwrap();

        let mut state = test_state();
        assert!(world.is_reachable(&village, &hidden));
        assert!(!travel_destinations(&state, &world)
            .iter()
            .any(|destination| destination.id == hidden));
        assert!(apply_travel(&mut state, &world, &hidden).is_err());
        assert_eq!(state.current_location, village);
        assert_eq!(state.world_turn, 0);

        crate::quests::notify_event(
            &mut state,
            &crate::event::GameEvent::LocationReached {
                location_id: village.clone(),
            },
        );
        assert!(travel_destinations(&state, &world)
            .iter()
            .any(|destination| destination.id == hidden));
        apply_travel(&mut state, &world, &hidden).unwrap();
        assert_eq!(state.current_location, hidden);
        assert_eq!(state.world_turn, 2);
    }

    #[test]
    fn route_requirement_is_checked_by_the_domain_travel_operation() {
        use crate::access::{AccessGate, AccessRequirement, AttributeKind};
        use crate::world::{Location, LocationCategory};

        let mut world = World::new();
        let from = LocationId::from("from");
        let to = LocationId::from("to");
        world
            .add_location(Location::new(
                from.clone(),
                "From",
                "",
                LocationCategory::Settlement,
            ))
            .unwrap();
        world
            .add_location(Location::new(
                to.clone(),
                "To",
                "",
                LocationCategory::Wilderness,
            ))
            .unwrap();
        world
            .add_route_with_gate(
                &from,
                &to,
                3,
                1,
                AccessGate {
                    all_of: vec![AccessRequirement::AttributeAtLeast {
                        attribute: AttributeKind::Wisdom,
                        minimum: 15,
                    }],
                    ..AccessGate::default()
                },
            )
            .unwrap();

        let mut state = test_state();
        state.current_location = from.clone();
        assert!(world.can_travel(&from, &to));
        assert!(apply_travel(&mut state, &world, &to).is_err());
        assert_eq!(state.current_location, from);
        assert_eq!(state.world_turn, 0);
    }

    #[test]
    fn access_items_are_consumed_only_after_successful_travel() {
        use crate::access::{AccessGate, ItemCost};
        use crate::data::ItemType;
        let mut world = World::new();
        let from = LocationId::from("from");
        let to = LocationId::from("to");
        world
            .add_location(crate::world::Location::new(
                from.clone(),
                "From",
                "",
                crate::world::LocationCategory::Settlement,
            ))
            .unwrap();
        world
            .add_location(crate::world::Location::new(
                to.clone(),
                "To",
                "",
                crate::world::LocationCategory::Landmark,
            ))
            .unwrap();
        world
            .add_route_with_gate(
                &from,
                &to,
                1,
                0,
                AccessGate {
                    consume_on_success: vec![ItemCost {
                        item: ItemType::Food,
                        count: 1,
                    }],
                    ..AccessGate::default()
                },
            )
            .unwrap();

        let mut state = test_state();
        state.current_location = from.clone();
        state.player.add_item(ItemType::Food, 1);
        apply_travel(&mut state, &world, &to).unwrap();
        assert_eq!(state.player.get_item_count(ItemType::Food), 0);
        assert_eq!(state.current_location, to);
    }
}
