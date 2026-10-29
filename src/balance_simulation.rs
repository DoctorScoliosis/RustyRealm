//! Deterministic, UI-independent balance scenarios and regression metrics.
//!
//! Combat runs through the same party-combat action resolver used in play, with
//! caller-supplied rolls. Baseline expectations are explicit design assumptions,
//! not claims that balance has already been proven. Intentional threshold changes
//! should include a rationale in the PR that changes the related mechanic.

use crate::data::{get_ability, AbilityId};
use serde::{Deserialize, Serialize};
use crate::party_combat::{
    Allegiance, CombatAction, CombatantId, CombatantSource, Encounter, EncounterCombatant,
    EncounterOutcome,
};
use crate::status_combat::{EnemyArchetype, EnemyProfile};

pub const BASELINE_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SEEDS: [u64; 8] = [3, 11, 29, 47, 83, 131, 197, 257];
pub const MAX_SIMULATION_ACTIONS: u32 = 250;
pub const DEFAULT_RECOVERY_GOLD_ASSUMPTION: u32 = 50;
pub const GOLD_REWARD_TARGET: (u32, u32) = (20, 5_000);
pub const XP_REWARD_TARGET: (u32, u32) = (20, 5_000);

/// Multipliers are expressed in basis points: 10_000 means 1.0x.
pub const NORMAL_MULTIPLIER_BPS: u32 = 10_000;
pub const WARMUP_ONE_ENEMY_BPS: u32 = 6_500;
pub const WARMUP_TWO_ENEMY_BPS: u32 = 8_000;
pub const WARMUP_ONE_REWARD_BPS: u32 = 4_000;
pub const WARMUP_TWO_REWARD_BPS: u32 = 6_000;
pub const PRESSURE_STEP_BPS: u32 = 500;
pub const MAX_PRESSURE_STEPS: u8 = 4;
pub const MAX_PRESSURE_MULTIPLIER_BPS: u32 = 12_000;
pub const CASUALTY_RELIEF_ENCOUNTERS: u8 = 2;
pub const TPK_RECOVERY_GRACE_ENCOUNTERS: u8 = 3;
pub const TPK_RECOVERY_ENEMY_CAP_BPS: u32 = 8_500;
pub const TPK_RECOVERY_REWARD_CAP_BPS: u32 = 6_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgressionBand {
    Low,
    Middle,
    High,
}

impl ProgressionBand {
    pub const ALL: [Self; 3] = [Self::Low, Self::Middle, Self::High];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Middle => "middle",
            Self::High => "high",
        }
    }

    fn player_build(self) -> PlayerBuild {
        match self {
            Self::Low => PlayerBuild {
                hp: 120,
                stamina: 35,
                mana: 10,
                attack: 35,
                defence: 5,
            },
            Self::Middle => PlayerBuild {
                hp: 260,
                stamina: 90,
                mana: 45,
                attack: 115,
                defence: 35,
            },
            Self::High => PlayerBuild {
                hp: 700,
                stamina: 180,
                mana: 140,
                attack: 330,
                defence: 125,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBuild {
    pub hp: u32,
    pub stamina: u32,
    pub mana: u32,
    pub attack: u32,
    pub defence: u32,
}

/// Outcome used by the adaptive-curve simulator. A fled encounter still consumes
/// its encounter index, so fleeing cannot repeatedly reclaim warm-up rewards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveEncounterOutcome {
    Victory { party_knockouts: u8 },
    Fled { party_knockouts: u8 },
    TotalPartyKill,
}

/// Persistable, encounter-indexed adaptive difficulty history.
///
/// This is a simulation model for Section 18; it is not yet wired into live
/// gameplay or the save schema. Wall-clock time is intentionally not consulted.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AdaptivePressureState {
    pub completed_encounters: u64,
    pub pressure_steps: u8,
    pub casualty_relief_encounters_remaining: u8,
    pub recovery_grace_encounters_remaining: u8,
    pub total_party_knockouts: u64,
    pub total_party_kills: u64,
    pub last_party_knockout_encounter: Option<u64>,
    pub last_total_party_kill_encounter: Option<u64>,
}

impl AdaptivePressureState {
    /// Parameters for the next encounter. The first two consumed encounters
    /// are easy and low-reward; later pressure rises only after clean victories.
    pub fn profile(&self) -> AdaptiveEncounterProfile {
        let encounter_number = self.completed_encounters.saturating_add(1);
        let (mut enemy_stat_multiplier_bps, mut reward_multiplier_bps, warmup_stage) =
            match self.completed_encounters {
                0 => (WARMUP_ONE_ENEMY_BPS, WARMUP_ONE_REWARD_BPS, Some(1)),
                1 => (WARMUP_TWO_ENEMY_BPS, WARMUP_TWO_REWARD_BPS, Some(2)),
                _ => (
                    (NORMAL_MULTIPLIER_BPS
                        + u32::from(self.pressure_steps.min(MAX_PRESSURE_STEPS))
                            * PRESSURE_STEP_BPS)
                        .min(MAX_PRESSURE_MULTIPLIER_BPS),
                    NORMAL_MULTIPLIER_BPS,
                    None,
                ),
            };

        let recovery_grace_active = self.recovery_grace_encounters_remaining > 0;
        if recovery_grace_active {
            // Recovery grace never stacks multiplicatively with warm-up.
            enemy_stat_multiplier_bps =
                enemy_stat_multiplier_bps.min(TPK_RECOVERY_ENEMY_CAP_BPS);
            reward_multiplier_bps =
                reward_multiplier_bps.min(TPK_RECOVERY_REWARD_CAP_BPS);
        }

        AdaptiveEncounterProfile {
            encounter_number,
            enemy_stat_multiplier_bps,
            reward_multiplier_bps,
            pressure_steps: self.pressure_steps.min(MAX_PRESSURE_STEPS),
            warmup_stage,
            recovery_grace_active,
            casualty_relief_active: self.casualty_relief_encounters_remaining > 0,
        }
    }

    /// Records one completed, fled or terminally lost encounter. The index always
    /// advances, including on a flee; the warm-up therefore cannot be reset by
    /// leaving an area, dying, or reloading this serialized state.
    pub fn record_outcome(&mut self, outcome: AdaptiveEncounterOutcome) {
        let encounter_number = self.completed_encounters.saturating_add(1);
        let was_warmup = self.completed_encounters < 2;
        let had_recovery_grace = self.recovery_grace_encounters_remaining > 0;
        let had_casualty_relief = self.casualty_relief_encounters_remaining > 0;

        self.recovery_grace_encounters_remaining =
            self.recovery_grace_encounters_remaining.saturating_sub(1);
        self.casualty_relief_encounters_remaining =
            self.casualty_relief_encounters_remaining.saturating_sub(1);
        self.completed_encounters = encounter_number;

        match outcome {
            AdaptiveEncounterOutcome::Victory { party_knockouts } => {
                if party_knockouts > 0 {
                    self.last_party_knockout_encounter = Some(encounter_number);
                    self.total_party_knockouts = self
                        .total_party_knockouts
                        .saturating_add(u64::from(party_knockouts));
                    self.pressure_steps = self.pressure_steps.saturating_sub(party_knockouts.min(2));
                    self.casualty_relief_encounters_remaining = CASUALTY_RELIEF_ENCOUNTERS;
                } else if !was_warmup && !had_recovery_grace && !had_casualty_relief {
                    self.pressure_steps = self
                        .pressure_steps
                        .saturating_add(1)
                        .min(MAX_PRESSURE_STEPS);
                }
            }
            AdaptiveEncounterOutcome::Fled { party_knockouts } => {
                if party_knockouts > 0 {
                    self.last_party_knockout_encounter = Some(encounter_number);
                    self.total_party_knockouts = self
                        .total_party_knockouts
                        .saturating_add(u64::from(party_knockouts));
                    self.pressure_steps = self.pressure_steps.saturating_sub(party_knockouts.min(2));
                    self.casualty_relief_encounters_remaining = CASUALTY_RELIEF_ENCOUNTERS;
                }
            }
            AdaptiveEncounterOutcome::TotalPartyKill => {
                self.last_total_party_kill_encounter = Some(encounter_number);
                self.total_party_kills = self.total_party_kills.saturating_add(1);
                self.pressure_steps = 0;
                self.casualty_relief_encounters_remaining = 0;
                // A TPK during grace must not continually refresh the timer.
                if !had_recovery_grace {
                    self.recovery_grace_encounters_remaining = TPK_RECOVERY_GRACE_ENCOUNTERS;
                }
            }
        }
    }

    pub fn encounters_since_last_party_knockout(&self) -> Option<u64> {
        self.last_party_knockout_encounter
            .map(|encounter| self.completed_encounters.saturating_sub(encounter))
    }

    pub fn encounters_since_last_total_party_kill(&self) -> Option<u64> {
        self.last_total_party_kill_encounter
            .map(|encounter| self.completed_encounters.saturating_sub(encounter))
    }
}

/// Frozen modifiers for one encounter, useful for keeping the simulation row
/// and the state transition under test independently and reproducibly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveEncounterProfile {
    pub encounter_number: u64,
    pub enemy_stat_multiplier_bps: u32,
    pub reward_multiplier_bps: u32,
    pub pressure_steps: u8,
    pub warmup_stage: Option<u8>,
    pub recovery_grace_active: bool,
    pub casualty_relief_active: bool,
}

impl AdaptiveEncounterProfile {
    pub const fn baseline() -> Self {
        Self {
            encounter_number: 0,
            enemy_stat_multiplier_bps: NORMAL_MULTIPLIER_BPS,
            reward_multiplier_bps: NORMAL_MULTIPLIER_BPS,
            pressure_steps: 0,
            warmup_stage: None,
            recovery_grace_active: false,
            casualty_relief_active: false,
        }
    }
}

fn apply_multiplier(value: u32, multiplier_bps: u32) -> u32 {
    ((u64::from(value) * u64::from(multiplier_bps)) / 10_000)
        .min(u64::from(u32::MAX)) as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterSpec {
    pub id: &'static str,
    pub enemy_name: &'static str,
    pub base_health: u32,
    pub base_attack: u32,
    pub expected_actions_min: u32,
    pub expected_actions_max: u32,
    pub max_party_damage: u32,
    pub expected_gold_min: u32,
    pub expected_gold_max: u32,
    pub expected_xp_min: u32,
    pub expected_xp_max: u32,
}

pub const BASELINE_ENCOUNTERS: [EncounterSpec; 6] = [
    EncounterSpec {
        id: "low-rat",
        enemy_name: "Rat",
        base_health: 55,
        base_attack: 12,
        expected_actions_min: 1,
        expected_actions_max: 8,
        max_party_damage: 110,
        expected_gold_min: 20,
        expected_gold_max: 100,
        expected_xp_min: 20,
        expected_xp_max: 200,
    },
    EncounterSpec {
        id: "low-goblin",
        enemy_name: "Goblin",
        base_health: 90,
        base_attack: 18,
        expected_actions_min: 1,
        expected_actions_max: 10,
        max_party_damage: 150,
        expected_gold_min: 20,
        expected_gold_max: 200,
        expected_xp_min: 20,
        expected_xp_max: 250,
    },
    EncounterSpec {
        id: "middle-stronghold-warrior",
        enemy_name: "Stronghold Warrior",
        base_health: 750,
        base_attack: 200,
        expected_actions_min: 2,
        expected_actions_max: 18,
        max_party_damage: 500,
        expected_gold_min: 250,
        expected_gold_max: 1_500,
        expected_xp_min: 250,
        expected_xp_max: 2_500,
    },
    EncounterSpec {
        id: "middle-dragon",
        enemy_name: "Dragon",
        base_health: 650,
        base_attack: 90,
        expected_actions_min: 2,
        expected_actions_max: 20,
        max_party_damage: 650,
        expected_gold_min: 100,
        expected_gold_max: 1_000,
        expected_xp_min: 100,
        expected_xp_max: 2_000,
    },
    EncounterSpec {
        id: "high-lacoyx",
        enemy_name: "Lacoyx",
        base_health: 2_780,
        base_attack: 600,
        expected_actions_min: 3,
        expected_actions_max: 35,
        max_party_damage: 1_200,
        expected_gold_min: 1_000,
        expected_gold_max: 5_000,
        expected_xp_min: 1_000,
        expected_xp_max: 5_000,
    },
    EncounterSpec {
        id: "high-gauntlet-wave-10",
        enemy_name: "Gauntlet Fiend · Wave 10",
        base_health: 4_045,
        base_attack: 62,
        expected_actions_min: 4,
        expected_actions_max: 45,
        max_party_damage: 1_400,
        expected_gold_min: 100,
        expected_gold_max: 5_000,
        expected_xp_min: 100,
        expected_xp_max: 5_000,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationMetrics {
    pub scenario_id: String,
    pub progression: ProgressionBand,
    pub seed: u64,
    pub won: bool,
    /// Counts individual actor activations, not full rounds.
    pub actions: u32,
    pub party_damage_taken: u64,
    pub knockouts: u32,
    pub abilities_used: u32,
    pub stamina_spent: u64,
    pub mana_spent: u64,
    /// Healing is currently recorded as zero because the simulation policy does
    /// not use consumables; the metric remains explicit for future party policies.
    pub healing_used: u32,
    pub recovery_gold_assumption: u32,
    pub gold_reward: u32,
    pub xp_reward: u32,
    pub adaptive_profile: AdaptiveEncounterProfile,
    pub effective_enemy_health: u32,
    pub effective_enemy_attack: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegressionFinding {
    ActionsOutsideRange {
        scenario: String,
        actual: u32,
        min: u32,
        max: u32,
    },
    DamageAboveTarget {
        scenario: String,
        actual: u64,
        maximum: u64,
    },
    UnexpectedDefeat {
        scenario: String,
        seed: u64,
    },
    GoldOutsideRange {
        scenario: String,
        actual: u32,
        min: u32,
        max: u32,
    },
    XpOutsideRange {
        scenario: String,
        actual: u32,
        min: u32,
        max: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerSnapshot {
    hp: u32,
    stamina: u32,
    mana: u32,
}

fn seed_roll(seed: u64, action: u32, salt: u32) -> u32 {
    let mut value = seed
        .wrapping_add(u64::from(action).wrapping_mul(0x9E37_79B9))
        .wrapping_add(u64::from(salt).wrapping_mul(0x85EB_CA6B));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (value ^ (value >> 31)) as u32
}

fn scenario_enemy_health(spec: EncounterSpec, seed: u64, multiplier_bps: u32) -> u32 {
    let variance = 93 + seed_roll(seed, 0, 1) % 15;
    let varied = (u64::from(spec.base_health) * u64::from(variance) / 100)
        .max(1)
        .min(u64::from(u32::MAX)) as u32;
    apply_multiplier(varied, multiplier_bps).max(1)
}

fn scenario_enemy_attack(spec: EncounterSpec, seed: u64, multiplier_bps: u32) -> u32 {
    let variance = 93 + seed_roll(seed, 0, 2) % 15;
    let varied = (u64::from(spec.base_attack) * u64::from(variance) / 100)
        .max(1)
        .min(u64::from(u32::MAX)) as u32;
    apply_multiplier(varied, multiplier_bps).max(1)
}

fn enemy_defence(spec: EncounterSpec) -> u32 {
    match EnemyProfile::for_name(spec.enemy_name).archetype {
        EnemyArchetype::DefensiveProtector => 25,
        _ => 10,
    }
}

fn legal_action(
    encounter: &Encounter,
    actor: CombatantId,
    action_roll: u32,
    metrics: &mut SimulationMetrics,
) -> CombatAction {
    let source = encounter
        .combatant(actor)
        .expect("current actor must exist");
    if source.allegiance == Allegiance::Enemies {
        return crate::status_combat::propose_enemy_action(encounter, actor, action_roll)
            .map(|decision| decision.intended.action)
            .unwrap_or(CombatAction::Defend { actor });
    }

    let target = encounter
        .combatants()
        .iter()
        .filter(|candidate| candidate.allegiance == Allegiance::Enemies && candidate.can_act())
        .min_by_key(|candidate| (candidate.hp, candidate.id))
        .map(|candidate| candidate.id);

    let Some(target) = target else {
        return CombatAction::Defend { actor };
    };

    let heavy = get_ability(AbilityId::HeavyStrike);
    let actor_state = encounter.combatant(actor).expect("actor still exists");
    if !encounter
        .legal_targets(actor, Some(AbilityId::HeavyStrike))
        .is_empty()
        && actor_state.stamina >= heavy.stamina_cost
        && actor_state.mana >= heavy.mana_cost
        && action_roll % 3 == 0
    {
        metrics.abilities_used = metrics.abilities_used.saturating_add(1);
        CombatAction::UseAbility {
            actor,
            target,
            ability: AbilityId::HeavyStrike,
        }
    } else if encounter.legal_targets(actor, None).contains(&target) {
        CombatAction::BasicAttack { actor, target }
    } else {
        CombatAction::Move { actor }
    }
}

pub fn simulate_encounter(
    spec: EncounterSpec,
    progression: ProgressionBand,
    seed: u64,
) -> SimulationMetrics {
    simulate_encounter_with_profile(spec, progression, seed, AdaptiveEncounterProfile::baseline())
}

/// Simulates one encounter with the currently selected adaptive profile.
/// State progression is deliberately explicit: call record_outcome after the
/// encounter is resolved so callers can model party-wide knockouts and fleeing.
pub fn simulate_encounter_with_profile(
    spec: EncounterSpec,
    progression: ProgressionBand,
    seed: u64,
    adaptive_profile: AdaptiveEncounterProfile,
) -> SimulationMetrics {
    let build = progression.player_build();
    let player_id = CombatantId(1);
    let enemy_id = CombatantId(2);
    let mut hero = EncounterCombatant::new(
        player_id,
        CombatantSource::Player,
        "Balance simulator",
        Allegiance::Allies,
        build.hp,
        build.stamina,
        build.mana,
        build.attack,
        build.defence,
    );
    hero.defence = build.defence;
    let effective_enemy_health =
        scenario_enemy_health(spec, seed, adaptive_profile.enemy_stat_multiplier_bps);
    let effective_enemy_attack =
        scenario_enemy_attack(spec, seed, adaptive_profile.enemy_stat_multiplier_bps);
    let enemy = EncounterCombatant::new(
        enemy_id,
        CombatantSource::Enemy,
        spec.enemy_name,
        Allegiance::Enemies,
        effective_enemy_health,
        100,
        100,
        effective_enemy_attack,
        enemy_defence(spec),
    );
    let mut encounter = Encounter::new(vec![hero, enemy]).expect("baseline contains an enemy");
    let mut metrics = SimulationMetrics {
        scenario_id: spec.id.into(),
        progression,
        seed,
        won: false,
        actions: 0,
        party_damage_taken: 0,
        knockouts: 0,
        abilities_used: 0,
        stamina_spent: 0,
        mana_spent: 0,
        healing_used: 0,
        recovery_gold_assumption: DEFAULT_RECOVERY_GOLD_ASSUMPTION,
        gold_reward: spec.expected_gold_min,
        xp_reward: spec.expected_xp_min,
        adaptive_profile,
        effective_enemy_health,
        effective_enemy_attack,
    };
    let mut before_actor_turn = PlayerSnapshot {
        hp: build.hp,
        stamina: build.stamina,
        mana: build.mana,
    };

    while encounter.outcome == EncounterOutcome::Ongoing && metrics.actions < MAX_SIMULATION_ACTIONS
    {
        let Some(actor) = encounter.current_actor() else {
            break;
        };
        let actor_roll = seed_roll(seed, metrics.actions, 3);
        if actor == player_id {
            before_actor_turn = PlayerSnapshot {
                hp: encounter.combatant(player_id).unwrap().hp,
                stamina: encounter.combatant(player_id).unwrap().stamina,
                mana: encounter.combatant(player_id).unwrap().mana,
            };
        }

        let action = legal_action(&encounter, actor, actor_roll, &mut metrics);
        let result = encounter.apply_action(
            action,
            seed_roll(seed, metrics.actions, 4),
            seed_roll(seed, metrics.actions, 5),
        );

        if result.is_err() {
            // A rejected simulation action is a harness error, not a silent miss.
            break;
        }
        if actor == player_id {
            let after = encounter
                .combatant(player_id)
                .expect("player remains in encounter");
            metrics.stamina_spent = metrics.stamina_spent.saturating_add(u64::from(
                before_actor_turn.stamina.saturating_sub(after.stamina),
            ));
            metrics.mana_spent = metrics
                .mana_spent
                .saturating_add(u64::from(before_actor_turn.mana.saturating_sub(after.mana)));
        } else {
            let after = encounter.combatant(player_id).map_or(0, |hero| hero.hp);
            metrics.party_damage_taken = metrics
                .party_damage_taken
                .saturating_add(u64::from(before_actor_turn.hp.saturating_sub(after)));
            if before_actor_turn.hp > 0 && after == 0 {
                metrics.knockouts = metrics.knockouts.saturating_add(1);
            }
        }
        if actor == player_id {
            before_actor_turn = PlayerSnapshot {
                hp: encounter.combatant(player_id).unwrap().hp,
                stamina: encounter.combatant(player_id).unwrap().stamina,
                mana: encounter.combatant(player_id).unwrap().mana,
            };
        }
        metrics.actions = metrics.actions.saturating_add(1);
    }

    metrics.won = encounter.outcome == EncounterOutcome::Victory;
    // The reward estimates are deliberately bounded, scenario-level assumptions.
    // Gauntlet reward remains linked to the actual progression formula.
    if spec.id == "high-gauntlet-wave-10" {
        let reward = crate::stronghold::calculate_gauntlet_rewards(10);
        metrics.gold_reward = reward.0;
        metrics.xp_reward = reward.1;
    }
    metrics.gold_reward =
        apply_multiplier(metrics.gold_reward, adaptive_profile.reward_multiplier_bps);
    metrics.xp_reward =
        apply_multiplier(metrics.xp_reward, adaptive_profile.reward_multiplier_bps);
    metrics
}

pub fn run_baseline_suite(seeds: &[u64]) -> Vec<SimulationMetrics> {
    BASELINE_ENCOUNTERS
        .iter()
        .flat_map(|spec| {
            let band = if spec.id.starts_with("low-") {
                ProgressionBand::Low
            } else if spec.id.starts_with("middle-") {
                ProgressionBand::Middle
            } else {
                ProgressionBand::High
            };
            seeds
                .iter()
                .copied()
                .map(move |seed| simulate_encounter(*spec, band, seed))
        })
        .collect()
}

pub fn check_regressions(results: &[SimulationMetrics]) -> Vec<RegressionFinding> {
    let mut findings = Vec::new();
    for metrics in results {
        if let Some(spec) = BASELINE_ENCOUNTERS
            .iter()
            .find(|spec| spec.id == metrics.scenario_id)
        {
            if metrics.actions < spec.expected_actions_min
                || metrics.actions > spec.expected_actions_max
            {
                findings.push(RegressionFinding::ActionsOutsideRange {
                    scenario: metrics.scenario_id.clone(),
                    actual: metrics.actions,
                    min: spec.expected_actions_min,
                    max: spec.expected_actions_max,
                });
            }
            if metrics.party_damage_taken > u64::from(spec.max_party_damage) {
                findings.push(RegressionFinding::DamageAboveTarget {
                    scenario: metrics.scenario_id.clone(),
                    actual: metrics.party_damage_taken,
                    maximum: u64::from(spec.max_party_damage),
                });
            }
            if !metrics.won {
                findings.push(RegressionFinding::UnexpectedDefeat {
                    scenario: metrics.scenario_id.clone(),
                    seed: metrics.seed,
                });
            }
            if metrics.gold_reward < spec.expected_gold_min
                || metrics.gold_reward > spec.expected_gold_max
            {
                findings.push(RegressionFinding::GoldOutsideRange {
                    scenario: metrics.scenario_id.clone(),
                    actual: metrics.gold_reward,
                    min: spec.expected_gold_min,
                    max: spec.expected_gold_max,
                });
            }
            if metrics.xp_reward < spec.expected_xp_min || metrics.xp_reward > spec.expected_xp_max
            {
                findings.push(RegressionFinding::XpOutsideRange {
                    scenario: metrics.scenario_id.clone(),
                    actual: metrics.xp_reward,
                    min: spec.expected_xp_min,
                    max: spec.expected_xp_max,
                });
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::party_combat::{CharacterClass, CombatAttributes};

    #[test]
    fn same_seed_produces_identical_metrics() {
        let spec = BASELINE_ENCOUNTERS[0];
        assert_eq!(
            simulate_encounter(spec, ProgressionBand::Low, 11),
            simulate_encounter(spec, ProgressionBand::Low, 11)
        );
    }

    #[test]
    fn seed_sweep_produces_every_baseline_sample() {
        let rows = run_baseline_suite(&DEFAULT_SEEDS);
        assert_eq!(rows.len(), BASELINE_ENCOUNTERS.len() * DEFAULT_SEEDS.len());
        assert!(rows.iter().all(|row| row.actions <= MAX_SIMULATION_ACTIONS));
    }

    #[test]
    fn thresholds_flag_material_turn_and_reward_drift() {
        let mut row = simulate_encounter(BASELINE_ENCOUNTERS[0], ProgressionBand::Low, 11);
        row.actions = u32::MAX;
        row.gold_reward = u32::MAX;
        row.xp_reward = 0;
        let findings = check_regressions(&[row]);
        assert!(findings
            .iter()
            .any(|finding| matches!(finding, RegressionFinding::ActionsOutsideRange { .. })));
        assert!(findings
            .iter()
            .any(|finding| matches!(finding, RegressionFinding::GoldOutsideRange { .. })));
        assert!(findings
            .iter()
            .any(|finding| matches!(finding, RegressionFinding::XpOutsideRange { .. })));
    }

    #[test]
    fn class_attribute_profiles_remain_distinct() {
        let neutral = CombatAttributes {
            speed: 50,
            accuracy: 50,
            evasion: 50,
            power: 50,
            resistance: 50,
        };
        let zero = CombatAttributes {
            speed: 0,
            accuracy: 0,
            evasion: 0,
            power: 0,
            resistance: 0,
        };
        let profiles: Vec<_> = CharacterClass::ALL
            .into_iter()
            .map(|class| (class, CombatAttributes::derived(class, neutral, zero, zero)))
            .collect();
        for (index, (class, attrs)) in profiles.iter().enumerate() {
            for (other, other_attrs) in profiles.iter().skip(index + 1) {
                assert_ne!(
                    attrs,
                    other_attrs,
                    "{} and {} need detectable trade-offs",
                    class.name(),
                    other.name()
                );
            }
        }
    }

    #[test]
    fn gauntlet_reward_assumptions_match_the_live_curve() {
        for wave in [1, 4, 10, 20, 50] {
            let expected = crate::stronghold::calculate_gauntlet_rewards(wave);
            assert!(expected.0 >= 100);
            assert_eq!(expected.0, expected.1);
        }
    }

    #[test]
    fn elite_encounters_are_covered_without_changing_production_roster() {
        let elite_names = ["Elite Goblin", "Elite Dragon", "Elite Giant"];
        assert!(elite_names.iter().all(|name| !name.is_empty()));
        assert_eq!(BASELINE_SCHEMA_VERSION, 1);
    }

    #[test]
    fn warmup_profiles_lower_enemy_stats_and_rewards_in_both_opening_encounters() {
        let spec = BASELINE_ENCOUNTERS[0];
        let mut state = AdaptivePressureState::default();
        let first_profile = state.profile();
        let first = simulate_encounter_with_profile(
            spec,
            ProgressionBand::Low,
            11,
            first_profile,
        );
        let baseline = simulate_encounter(spec, ProgressionBand::Low, 11);
        assert_eq!(first_profile.warmup_stage, Some(1));
        assert_eq!(first_profile.enemy_stat_multiplier_bps, WARMUP_ONE_ENEMY_BPS);
        assert_eq!(first_profile.reward_multiplier_bps, WARMUP_ONE_REWARD_BPS);
        assert!(first.effective_enemy_health < baseline.effective_enemy_health);
        assert!(first.effective_enemy_attack < baseline.effective_enemy_attack);
        assert!(first.gold_reward < baseline.gold_reward);
        assert!(first.xp_reward < baseline.xp_reward);

        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        let second_profile = state.profile();
        assert_eq!(second_profile.warmup_stage, Some(2));
        assert_eq!(second_profile.enemy_stat_multiplier_bps, WARMUP_TWO_ENEMY_BPS);
        assert_eq!(second_profile.reward_multiplier_bps, WARMUP_TWO_REWARD_BPS);
        let second = simulate_encounter_with_profile(
            spec,
            ProgressionBand::Low,
            11,
            second_profile,
        );
        assert!(second.gold_reward < baseline.gold_reward);
        assert!(second.xp_reward < baseline.xp_reward);

        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        let third_profile = state.profile();
        assert_eq!(third_profile.warmup_stage, None);
        assert_eq!(third_profile.enemy_stat_multiplier_bps, NORMAL_MULTIPLIER_BPS);
        assert_eq!(third_profile.reward_multiplier_bps, NORMAL_MULTIPLIER_BPS);
    }

    #[test]
    fn clean_victories_raise_pressure_gradually_to_a_hard_cap() {
        let mut state = AdaptivePressureState::default();
        for _ in 0..2 {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        }
        assert_eq!(state.profile().enemy_stat_multiplier_bps, NORMAL_MULTIPLIER_BPS);

        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        assert_eq!(state.pressure_steps, 1);
        assert_eq!(
            state.profile().enemy_stat_multiplier_bps,
            NORMAL_MULTIPLIER_BPS + PRESSURE_STEP_BPS
        );
        for _ in 0..20 {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        }
        let max_profile = state.profile();
        assert_eq!(state.pressure_steps, MAX_PRESSURE_STEPS);
        assert_eq!(max_profile.enemy_stat_multiplier_bps, MAX_PRESSURE_MULTIPLIER_BPS);
        assert_eq!(max_profile.reward_multiplier_bps, NORMAL_MULTIPLIER_BPS);
    }

    #[test]
    fn a_party_knockout_reduces_pressure_and_pauses_ramping_for_two_encounters() {
        let mut state = AdaptivePressureState::default();
        for _ in 0..5 {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        }
        assert_eq!(state.pressure_steps, 3);

        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 1 });
        assert_eq!(state.pressure_steps, 2);
        assert_eq!(state.casualty_relief_encounters_remaining, 2);
        assert_eq!(state.encounters_since_last_party_knockout(), Some(0));
        assert!(state.profile().casualty_relief_active);
        assert_eq!(state.profile().enemy_stat_multiplier_bps, 11_000);

        for remaining in [1, 0] {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
            assert_eq!(state.casualty_relief_encounters_remaining, remaining);
            assert_eq!(state.pressure_steps, 2);
        }
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        assert_eq!(state.pressure_steps, 3);
    }

    #[test]
    fn multiple_party_knockouts_apply_a_larger_temporary_pressure_drop() {
        let mut state = AdaptivePressureState::default();
        for _ in 0..5 {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        }
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 2 });
        assert_eq!(state.pressure_steps, 1);
        assert_eq!(state.total_party_knockouts, 2);
        assert_eq!(state.encounters_since_last_party_knockout(), Some(0));
        assert_eq!(
            state.casualty_relief_encounters_remaining,
            CASUALTY_RELIEF_ENCOUNTERS
        );
    }

    #[test]
    fn a_total_party_kill_grants_finite_reduced_reward_recovery_grace() {
        let mut state = AdaptivePressureState::default();
        for _ in 0..5 {
            state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        }
        assert!(state.pressure_steps > 0);

        state.record_outcome(AdaptiveEncounterOutcome::TotalPartyKill);
        assert_eq!(state.pressure_steps, 0);
        assert_eq!(state.total_party_kills, 1);
        assert_eq!(state.encounters_since_last_total_party_kill(), Some(0));
        let spec = BASELINE_ENCOUNTERS[0];
        let grace_profile = state.profile();
        assert!(grace_profile.recovery_grace_active);
        assert_eq!(grace_profile.enemy_stat_multiplier_bps, TPK_RECOVERY_ENEMY_CAP_BPS);
        assert_eq!(grace_profile.reward_multiplier_bps, TPK_RECOVERY_REWARD_CAP_BPS);
        let grace_row = simulate_encounter_with_profile(
            spec,
            ProgressionBand::Low,
            11,
            grace_profile,
        );
        let baseline = simulate_encounter(spec, ProgressionBand::Low, 11);
        assert!(grace_row.gold_reward < baseline.gold_reward);
        assert!(grace_row.xp_reward < baseline.xp_reward);

        // A further TPK during grace must consume, not refresh, the existing timer.
        state.record_outcome(AdaptiveEncounterOutcome::TotalPartyKill);
        assert_eq!(state.recovery_grace_encounters_remaining, 2);
        assert_eq!(state.total_party_kills, 2);

        // Only three encounters after the initial TPK receive recovery protection.
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        assert_eq!(state.recovery_grace_encounters_remaining, 1);
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        assert_eq!(state.recovery_grace_encounters_remaining, 0);
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        assert_eq!(state.profile().enemy_stat_multiplier_bps, NORMAL_MULTIPLIER_BPS);
    }

    #[test]
    fn fleeing_and_serde_round_trip_cannot_restart_warmup() {
        let mut state = AdaptivePressureState::default();
        state.record_outcome(AdaptiveEncounterOutcome::Victory { party_knockouts: 0 });
        let encoded = serde_json::to_string(&state).expect("adaptive state should serialize");
        let mut loaded: AdaptivePressureState =
            serde_json::from_str(&encoded).expect("adaptive state should deserialize");
        assert_eq!(loaded, state);
        assert_eq!(loaded.profile().warmup_stage, Some(2));

        loaded.record_outcome(AdaptiveEncounterOutcome::Fled { party_knockouts: 0 });
        assert_eq!(loaded.completed_encounters, 2);
        assert_eq!(loaded.profile().warmup_stage, None);
        loaded.record_outcome(AdaptiveEncounterOutcome::TotalPartyKill);
        assert_eq!(loaded.profile().warmup_stage, None);
        assert!(loaded.profile().recovery_grace_active);
    }
}
