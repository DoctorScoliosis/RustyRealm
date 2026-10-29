//! Terminal adapter for the shared party-aware combat domain.
//!
//! Encounter rules, action resolution, range, status effects, telegraphs, and
//! knockout outcomes are owned by `party_combat` and `status_combat`. This
//! module only collects player choices, drives companion/enemy policies, and
//! synchronises the resolved state back into the persistent game.

use crate::data::{get_ability, AbilityId, MAX_PLAYER_LEVEL};
use crate::event::GameEvent;
use crate::party::CompanionId;
use crate::party_combat::{
    Allegiance, CombatAction, CombatantId, CombatantSource, Encounter, EncounterCombatant,
    EncounterError, EncounterOutcome, RangeBand,
};
use crate::state::GameState;
use crate::status_combat::{
    propose_enemy_action, resolve_telegraph, EnemyDecision, TelegraphState, TelegraphedAction,
};
use crate::ui::Tui;
use crate::world::{build_initial_world, World};
use rand::Rng;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleOutcome {
    Victory,
    Fled,
    Defeated,
}

pub struct BattleConfig<'a> {
    pub enemy_name: &'a str,
    pub level: u32,
    pub enemy_hp: u32,
    pub enemy_attack_power: u32,
    pub allow_inventory: bool,
    pub allow_flee: bool,
    pub resource_drain_per_turn: u32,
}

fn build_encounter(
    state: &GameState,
    config: &BattleConfig<'_>,
) -> Result<Encounter, EncounterError> {
    let enemy = EncounterCombatant::new(
        CombatantId(10_000),
        CombatantSource::Enemy,
        config.enemy_name,
        Allegiance::Enemies,
        config.enemy_hp.max(1),
        100,
        100,
        config.enemy_attack_power.max(1),
        0,
    );
    Encounter::from_party(&state.player, &state.party, &state.companions, vec![enemy])
}

fn encounter_error(error: EncounterError) -> io::Error {
    io::Error::other(format!(
        "Party encounter could not resolve an action: {error:?}"
    ))
}

fn sync_player_state(state: &mut GameState, encounter: &Encounter) {
    let Some(hero) = encounter
        .combatants()
        .iter()
        .find(|combatant| combatant.source == CombatantSource::Player)
    else {
        return;
    };
    state.player.hp = hero.hp.min(state.player.max_hp);
    state.player.max_hp = hero.max_hp.max(hero.hp);
    state.player.stamina = hero.stamina.min(state.player.max_stamina);
    state.player.max_stamina = hero.max_stamina.max(hero.stamina);
    state.player.mana = hero.mana.min(state.player.max_mana);
    state.player.max_mana = hero.max_mana.max(hero.mana);
}

fn sync_persistent_state(
    state: &mut GameState,
    encounter: &Encounter,
    world: &World,
) -> Result<(), io::Error> {
    sync_player_state(state, encounter);
    // Recovery methods own a separate field but accept GameState as a whole.
    // Temporarily take the field to avoid aliasing it with the mutable state.
    let mut recovery = std::mem::take(&mut state.companion_recovery);
    let result = recovery.sync_party_encounter(state, encounter, world);
    state.companion_recovery = recovery;
    result
        .map(|_| ())
        .map_err(|error| io::Error::other(error.to_string()))
}

fn actor_source(encounter: &Encounter, actor: CombatantId) -> Option<CombatantSource> {
    encounter.combatant(actor).map(|combatant| combatant.source)
}

fn target_label(encounter: &Encounter, id: CombatantId) -> String {
    encounter
        .combatant(id)
        .map(|combatant| {
            format!(
                "{} · HP {}/{}",
                combatant.name, combatant.hp, combatant.max_hp
            )
        })
        .unwrap_or_else(|| format!("Combatant {}", id.0))
}

fn legal_targets(
    encounter: &Encounter,
    actor: CombatantId,
    ability: Option<AbilityId>,
) -> Vec<CombatantId> {
    encounter.legal_targets(actor, ability)
}

fn choose_target(
    tui: &mut Tui,
    state: &GameState,
    encounter: &Encounter,
    targets: &[CombatantId],
) -> io::Result<Option<CombatantId>> {
    match targets {
        [] => Ok(None),
        [only] => Ok(Some(*only)),
        _ => {
            let mut labels: Vec<String> = targets
                .iter()
                .map(|id| target_label(encounter, *id))
                .collect();
            labels.push("Back".to_string());
            let options: Vec<&str> = labels.iter().map(String::as_str).collect();
            let choice = tui.select_menu("Choose a target", &options, Some(state))?;
            Ok((choice < targets.len()).then_some(targets[choice]))
        }
    }
}

fn battle_status(encounter: &Encounter, round_drain: u32, last_log: &str) -> String {
    let hero = encounter
        .combatants()
        .iter()
        .find(|combatant| combatant.source == CombatantSource::Player);
    let hero_line = hero
        .map(|c| {
            format!(
                "You · HP {}/{} · Stamina {}/{} · Mana {}/{}",
                c.hp, c.max_hp, c.stamina, c.max_stamina, c.mana, c.max_mana
            )
        })
        .unwrap_or_else(|| "You · unavailable".to_string());
    let allies = encounter
        .combatants()
        .iter()
        .filter(|c| matches!(c.source, CombatantSource::Companion(_)))
        .map(|c| format!("{} {}/{}", c.name, c.hp, c.max_hp))
        .collect::<Vec<_>>();
    let hostiles = encounter
        .combatants()
        .iter()
        .filter(|c| c.allegiance == Allegiance::Enemies)
        .map(|c| format!("{} {}/{}", c.name, c.hp, c.max_hp))
        .collect::<Vec<_>>();
    let drain = if round_drain > 0 {
        format!(" · Resource drain {round_drain}/round")
    } else {
        String::new()
    };
    format!(
        "Round {}{drain}\n{hero_line}\nParty: {}\nHostiles: {}\n{}",
        encounter.round(),
        if allies.is_empty() {
            "none".to_string()
        } else {
            allies.join(" · ")
        },
        hostiles.join(" · "),
        last_log
    )
}

fn choose_player_action(
    tui: &mut Tui,
    state: &mut GameState,
    encounter: &mut Encounter,
    actor: CombatantId,
    config: &BattleConfig<'_>,
    last_log: &str,
) -> io::Result<Option<CombatAction>> {
    let mut labels = Vec::new();
    let mut actions = Vec::new();

    if !legal_targets(encounter, actor, None).is_empty() {
        labels.push("Basic attack".to_string());
        actions.push(Some(CombatAction::BasicAttack {
            actor,
            target: legal_targets(encounter, actor, None)[0],
        }));
    }
    for (id, label) in [
        (AbilityId::HeavyStrike, "Heavy Strike"),
        (AbilityId::LifeDrain, "Life Drain"),
        (AbilityId::Cleave, "Cleave"),
    ] {
        let ability = get_ability(id);
        let source = encounter.combatant(actor).expect("current actor exists");
        let targets = legal_targets(encounter, actor, Some(id));
        if !targets.is_empty()
            && source.stamina >= ability.stamina_cost
            && source.mana >= ability.mana_cost
        {
            labels.push(format!(
                "{label} · {} stamina / {} mana",
                ability.stamina_cost, ability.mana_cost
            ));
            actions.push(Some(CombatAction::UseAbility {
                actor,
                target: targets[0],
                ability: id,
            }));
        }
    }

    let source = encounter.combatant(actor).expect("current actor exists");
    let shield = get_ability(AbilityId::ShieldWall);
    if source.stamina >= shield.stamina_cost {
        labels.push(format!(
            "Shield Wall · {} stamina · guard the next two hits",
            shield.stamina_cost
        ));
        actions.push(Some(CombatAction::UseAbility {
            actor,
            target: actor,
            ability: AbilityId::ShieldWall,
        }));
    }
    labels.push("Move".to_string());
    actions.push(Some(CombatAction::Move { actor }));
    labels.push("Defend · reduce the next hit".to_string());
    actions.push(Some(CombatAction::Defend { actor }));
    if config.allow_inventory {
        labels.push("Inventory".to_string());
        actions.push(None);
    }
    if config.allow_flee {
        labels.push("Flee".to_string());
        actions.push(Some(CombatAction::Retreat { actor }));
    }
    labels.push("Back".to_string());
    actions.push(None);

    let status = battle_status(encounter, config.resource_drain_per_turn, last_log);
    let title = format!("Party combat · Level {} · {}", config.level, status);
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    loop {
        let choice = tui.select_menu(&title, &options, Some(state))?;
        let Some(action) = actions.get(choice).cloned().flatten() else {
            let inventory_index = labels.iter().position(|label| label == "Inventory");
            if inventory_index == Some(choice) && config.allow_inventory {
                sync_player_state(state, encounter);
                crate::inventory::view_inventory(tui, state, true)?;
                encounter
                    .sync_player_from(&state.player)
                    .map_err(encounter_error)?;
                continue;
            }
            return Ok(None);
        };
        match &action {
            CombatAction::BasicAttack { .. }
            | CombatAction::UseAbility {
                ability: AbilityId::HeavyStrike | AbilityId::LifeDrain | AbilityId::Cleave,
                ..
            } => {
                let ability = match action {
                    CombatAction::UseAbility { ability, .. } => Some(ability),
                    _ => None,
                };
                let targets = legal_targets(encounter, actor, ability);
                let Some(target) = choose_target(tui, state, encounter, &targets)? else {
                    continue;
                };
                return Ok(Some(match action {
                    CombatAction::BasicAttack { .. } => CombatAction::BasicAttack { actor, target },
                    CombatAction::UseAbility { ability, .. } => CombatAction::UseAbility {
                        actor,
                        target,
                        ability,
                    },
                    other => other,
                }));
            }
            _ => return Ok(Some(action)),
        }
    }
}

fn companion_action(encounter: &Encounter, actor: CombatantId) -> Option<CombatAction> {
    let source = encounter.combatant(actor)?;
    let ability = match source.source {
        CombatantSource::Companion(CompanionId::MarenStarweave) => Some(AbilityId::LifeDrain),
        CombatantSource::Companion(CompanionId::GarrickIronfist) => Some(AbilityId::Cleave),
        CombatantSource::Companion(CompanionId::BranTheShield) => Some(AbilityId::HeavyStrike),
        CombatantSource::Companion(CompanionId::LyraSwiftbow) => Some(AbilityId::HeavyStrike),
        _ => None,
    };
    if let Some(ability_id) = ability {
        let def = get_ability(ability_id);
        let targets = legal_targets(encounter, actor, Some(ability_id));
        if !targets.is_empty() && source.stamina >= def.stamina_cost && source.mana >= def.mana_cost
        {
            return Some(CombatAction::UseAbility {
                actor,
                target: targets[0],
                ability: ability_id,
            });
        }
    }
    legal_targets(encounter, actor, None)
        .first()
        .copied()
        .map(|target| CombatAction::BasicAttack { actor, target })
        .or(Some(CombatAction::Move { actor }))
}

fn roll() -> u32 {
    rand::thread_rng().gen_range(0..u32::MAX)
}

fn award_combat_xp(state: &mut GameState, damage: u32, tui: &mut Tui) -> io::Result<()> {
    let xp = (damage / 4).saturating_mul(3);
    if xp == 0 {
        return Ok(());
    }
    state.player.xp = state.player.xp.saturating_add(xp).min(1_000_000);
    state.player.xp_until_next = state.player.xp_until_next.saturating_sub(xp);
    if state.player.xp_until_next == 0 && state.player.combat_level < MAX_PLAYER_LEVEL {
        state.player.level_up_combat();
        let new_level = state.player.combat_level;
        crate::quests::notify_event(state, &GameEvent::PlayerLeveledUp { new_level });
        tui.show_message(
            "Combat level up",
            &format!(
                "You are now combat level {} and have {} health.",
                state.player.combat_level, state.player.hp
            ),
            Some(state),
        )?;
    }
    Ok(())
}

fn update_lacoyx_phase(
    encounter: &mut Encounter,
    enemy_id: CombatantId,
    base_attack: u32,
    phase: &mut u8,
    tui: &mut Tui,
    state: &GameState,
) -> io::Result<()> {
    let enemy = encounter.combatant(enemy_id).expect("boss exists");
    if enemy.hp == 0 {
        return Ok(());
    }
    let hp_pct = enemy.hp.saturating_mul(100) / enemy.max_hp.max(1);
    let next = if hp_pct <= 33 {
        3
    } else if hp_pct <= 66 {
        2
    } else {
        1
    };
    if next <= *phase {
        return Ok(());
    }
    *phase = next;
    let (label, description, multiplier) = match next {
        2 => (
            "Phase 2 · Infernal Warlord",
            "Lacoyx's strikes intensify and status pressure rises.",
            125,
        ),
        _ => (
            "Phase 3 · Desperate Overlord",
            "Lacoyx fights with overwhelming force.",
            150,
        ),
    };
    encounter
        .set_attack(enemy_id, base_attack.saturating_mul(multiplier) / 100)
        .map_err(encounter_error)?;
    tui.show_message(label, description, Some(state))?;
    Ok(())
}

/// Execute an encounter using the party-aware action resolver and existing enemy policy.
pub fn run_party_battle(
    tui: &mut Tui,
    state: &mut GameState,
    config: BattleConfig<'_>,
) -> io::Result<BattleOutcome> {
    let world = build_initial_world().map_err(|error| io::Error::other(error.to_string()))?;
    let mut encounter = build_encounter(state, &config).map_err(encounter_error)?;
    let enemy_id = encounter
        .combatants()
        .iter()
        .find(|c| c.allegiance == Allegiance::Enemies)
        .map(|c| c.id)
        .ok_or_else(|| io::Error::other("Party encounter was created without an enemy."))?;
    let base_attack = config.enemy_attack_power.max(1);
    let is_lacoyx = config.enemy_name == "Lacoyx";
    let mut boss_phase = 1_u8;
    let mut last_log = format!(
        "You encounter a level {} {} with {} health.",
        config.level, config.enemy_name, config.enemy_hp
    );
    let mut pending_telegraph: Option<TelegraphedAction> = None;
    let mut last_drain_round = 0_u64;
    let mut player_knockout_notice = false;

    while encounter.outcome == EncounterOutcome::Ongoing {
        let Some(actor) = encounter.current_actor() else {
            break;
        };

        if config.resource_drain_per_turn > 0 && encounter.round() != last_drain_round {
            last_drain_round = encounter.round();
            state.player.stamina = state
                .player
                .stamina
                .saturating_sub(config.resource_drain_per_turn);
            state.player.mana = state
                .player
                .mana
                .saturating_sub(config.resource_drain_per_turn);
            encounter
                .sync_player_from(&state.player)
                .map_err(encounter_error)?;
            last_log = format!(
                "Resource Drain removes {} Stamina and Mana.",
                config.resource_drain_per_turn
            );
        }

        let ticks = encounter.tick_current_actor_statuses();
        if !ticks.is_empty() {
            last_log = ticks
                .iter()
                .map(|(_, tick)| format!("{tick:?}"))
                .collect::<Vec<_>>()
                .join(" · ");
            if ticks
                .iter()
                .any(|(_, tick)| matches!(tick, crate::status_combat::StatusTick::Stunned))
                && pending_telegraph
                    .as_ref()
                    .is_some_and(|telegraph| telegraph.actor == actor)
            {
                let telegraph = pending_telegraph.take().expect("pending telegraph matched");
                last_log = format!("{} is interrupted by Stun.", telegraph.name);
            }
            sync_persistent_state(state, &encounter, &world)?;
            if encounter.current_actor() != Some(actor) {
                continue;
            }
        }

        if encounter.outcome != EncounterOutcome::Ongoing {
            break;
        }

        let source = actor_source(&encounter, actor).expect("current actor is in encounter");
        let action_result = match source {
            CombatantSource::Player => {
                let Some(action) =
                    choose_player_action(tui, state, &mut encounter, actor, &config, &last_log)?
                else {
                    continue;
                };
                encounter
                    .apply_action(action, roll(), roll())
                    .map_err(encounter_error)?
            }
            CombatantSource::Companion(_) => {
                let action = companion_action(&encounter, actor)
                    .ok_or_else(|| io::Error::other("Companion had no legal combat action."))?;
                encounter
                    .apply_action(action, roll(), roll())
                    .map_err(encounter_error)?
            }
            CombatantSource::Enemy => {
                if let Some(telegraph) = pending_telegraph
                    .as_ref()
                    .filter(|telegraph| telegraph.actor == actor)
                    .cloned()
                {
                    let target_is_player = encounter
                        .combatant(telegraph.target)
                        .is_some_and(|target| target.source == CombatantSource::Player);
                    tui.show_message(
                        "Incoming attack",
                        &format!("{}\n{}", telegraph.name, telegraph.description),
                        Some(state),
                    )?;
                    if target_is_player
                        && encounter
                            .combatant(telegraph.target)
                            .is_some_and(|target| target.stamina >= 10)
                    {
                        if tui.select_menu(
                            "Defensive reaction",
                            &[
                                "Brace · spend 10 stamina to reduce damage by 50%",
                                "Take the hit",
                            ],
                            Some(state),
                        )? == 0
                        {
                            encounter
                                .spend_stamina(telegraph.target, 10)
                                .map_err(encounter_error)?;
                            encounter
                                .guard_combatant(telegraph.target, 50, 1)
                                .map_err(encounter_error)?;
                        }
                    }
                    pending_telegraph = None;
                    match resolve_telegraph(&mut encounter, telegraph, roll(), roll())
                        .map_err(encounter_error)?
                    {
                        TelegraphState::Resolved(telegraph) => {
                            last_log = format!("{} resolves.", telegraph.name);
                        }
                        TelegraphState::Cancelled { name, reason } => {
                            last_log = format!("{name} is cancelled: {reason}.");
                        }
                    }
                    // The resolver consumed the enemy activation, so this loop
                    // must not attempt a second action for it.
                    sync_persistent_state(state, &encounter, &world)?;
                    if is_lacoyx {
                        update_lacoyx_phase(
                            &mut encounter,
                            enemy_id,
                            base_attack,
                            &mut boss_phase,
                            tui,
                            state,
                        )?;
                    }
                    continue;
                }

                let decision: Option<EnemyDecision> =
                    propose_enemy_action(&encounter, actor, roll());
                let Some(decision) = decision else {
                    let action = CombatAction::Defend { actor };
                    encounter
                        .apply_action(action, roll(), roll())
                        .map_err(encounter_error)?;
                    last_log = "The enemy loses its activation.".to_string();
                    sync_persistent_state(state, &encounter, &world)?;
                    continue;
                };
                if let Some(telegraph) = decision.intended.telegraph {
                    encounter
                        .apply_action(CombatAction::Telegraph { actor }, roll(), roll())
                        .map_err(encounter_error)?;
                    last_log = format!("{}: {}", telegraph.name, telegraph.description);
                    pending_telegraph = Some(telegraph);
                    sync_persistent_state(state, &encounter, &world)?;
                    continue;
                }
                encounter
                    .apply_action(decision.intended.action, roll(), roll())
                    .map_err(encounter_error)?
            }
        };

        if source == CombatantSource::Player && action_result.damage > 0 {
            sync_persistent_state(state, &encounter, &world)?;
            award_combat_xp(state, action_result.damage, tui)?;
            encounter
                .sync_player_from(&state.player)
                .map_err(encounter_error)?;
        }
        last_log = match (
            action_result.hit,
            action_result.damage,
            action_result.healing,
        ) {
            (Some(false), _, _) => format!(
                "{} misses.",
                encounter
                    .combatant(actor)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Actor")
            ),
            (_, damage, healing) if damage > 0 && healing > 0 => format!(
                "{} deals {damage} damage and recovers {healing} HP.",
                encounter
                    .combatant(actor)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Actor")
            ),
            (_, damage, _) if damage > 0 => format!(
                "{} deals {damage} damage.",
                encounter
                    .combatant(actor)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Actor")
            ),
            (_, _, healing) if healing > 0 => format!(
                "{} recovers {healing} HP.",
                encounter
                    .combatant(actor)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Actor")
            ),
            _ => format!(
                "{}: {}.",
                encounter
                    .combatant(actor)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Actor"),
                action_result.message
            ),
        };
        sync_persistent_state(state, &encounter, &world)?;

        if is_lacoyx {
            update_lacoyx_phase(
                &mut encounter,
                enemy_id,
                base_attack,
                &mut boss_phase,
                tui,
                state,
            )?;
        }
        if encounter.outcome == EncounterOutcome::PartyDefeated {
            let mut recovery = std::mem::take(&mut state.companion_recovery);
            let result = recovery.resolve_player_defeat(state, &world);
            state.companion_recovery = recovery;
            result.map_err(|error| io::Error::other(error.to_string()))?;
            tui.show_message(
                "Expedition defeated",
                "Your party has been incapacitated. You have been rescued at the nearest reachable settlement; knocked-out companions remain unavailable until recovered.",
                Some(state),
            )?;
            return Ok(BattleOutcome::Defeated);
        }
    }

    // If companions win while the player is down, they revive the player after combat.
    if state.player.hp == 0 && encounter.outcome == EncounterOutcome::Victory {
        state.player.hp = state.player.max_hp;
        player_knockout_notice = true;
    }
    if player_knockout_notice {
        tui.show_message(
            "Recovered after victory",
            "Your companions won the battle and brought you round afterwards.",
            Some(state),
        )?;
    }
    sync_persistent_state(state, &encounter, &world)?;
    match encounter.outcome {
        EncounterOutcome::Victory => Ok(BattleOutcome::Victory),
        EncounterOutcome::Retreated => Ok(BattleOutcome::Fled),
        EncounterOutcome::PartyDefeated | EncounterOutcome::Ongoing => Err(io::Error::other(
            "Party encounter ended without a terminal outcome.",
        )),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::party::{CompanionId, PartyManager};
    use crate::state::Player;

    fn test_state_with_party() -> GameState {
       let mut state = GameState::new(Player::new("party-test".into(), String::new()));
       state.player.combat_level = 99;
       state.player.gold = 10_000;
       let id = CompanionId::BranTheShield;
       let cost = PartyManager::recruit(&mut state.companions, id, 99, state.player.gold)
           .expect("companion should be recruitable");
       state.player.gold -= cost;
       PartyManager::add_to_party(&mut state.party, &state.companions, id)
           .expect("recruited companion should join");
       state
    }

    #[test]
    fn build_encounter_preserves_current_and_maximum_resources_for_party() {
       let mut state = test_state_with_party();
       state.player.hp = state.player.max_hp / 2;
       state.player.stamina = state.player.max_stamina / 2;
       state.player.mana = state.player.max_mana / 2;

       let config = BattleConfig {
           enemy_name: "Goblin",
           level: 5,
           enemy_hp: 120,
           enemy_attack_power: 18,
           allow_inventory: true,
           allow_flee: true,
           resource_drain_per_turn: 0,
       };
       let encounter = build_encounter(&state, &config).expect("valid party encounter");

       let hero = encounter
           .combatants()
           .iter()
           .find(|c| c.source == CombatantSource::Player)
           .expect("player combatant");
       assert_eq!(hero.hp, state.player.hp);
       assert_eq!(hero.max_hp, state.player.max_hp);
       assert_eq!(hero.stamina, state.player.stamina);
       assert_eq!(hero.max_stamina, state.player.max_stamina);
       assert_eq!(hero.mana, state.player.mana);
       assert_eq!(hero.max_mana, state.player.max_mana);

       let id = CompanionId::BranTheShield;
       let saved = state.companions.iter().find(|c| c.id == id).unwrap();
       let companion = encounter
           .combatants()
           .iter()
           .find(|c| c.source == CombatantSource::Companion(id))
           .expect("party companion combatant");
       assert_eq!(companion.hp, saved.current_hp);
       assert_eq!(companion.max_hp, saved.max_hp);
       assert_eq!(companion.stamina, saved.current_stamina);
       assert_eq!(companion.max_stamina, saved.max_stamina);
       assert_eq!(companion.mana, saved.current_mana);
       assert_eq!(companion.max_mana, saved.max_mana);

       let enemy = encounter
           .combatants()
           .iter()
           .find(|c| c.allegiance == Allegiance::Enemies)
           .expect("enemy combatant");
       assert_eq!(enemy.name, "Goblin");
       assert_eq!(enemy.max_hp, 120);
       assert_eq!(enemy.attack, 18);
    }
}
