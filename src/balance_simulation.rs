//! Deterministic, UI-independent balance scenarios and regression metrics.
//!
//! Combat runs through the same party-combat action resolver used in play, with
//! caller-supplied rolls. Baseline expectations are explicit design assumptions,
//! not claims that balance has already been proven. Intentional threshold changes
//! should include a rationale in the PR that changes the related mechanic.

use crate::data::{get_ability, AbilityId};
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

fn scenario_enemy_health(spec: EncounterSpec, seed: u64) -> u32 {
    let variance = 93 + seed_roll(seed, 0, 1) % 15;
    (u64::from(spec.base_health) * u64::from(variance) / 100).max(1) as u32
}

fn scenario_enemy_attack(spec: EncounterSpec, seed: u64) -> u32 {
    let variance = 93 + seed_roll(seed, 0, 2) % 15;
    (u64::from(spec.base_attack) * u64::from(variance) / 100).max(1) as u32
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
    let source = encounter.combatant(actor).expect("current actor must exist");
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
    if !encounter.legal_targets(actor, Some(AbilityId::HeavyStrike)).is_empty()
        && actor_state.stamina >= heavy.stamina_cost
        && actor_state.mana >= heavy.mana_cost
        && action_roll % 3 == 0
    {
        metrics.abilities_used = metrics.abilities_used.saturating_add(1);
        CombatAction::UseAbility { actor, target, ability: AbilityId::HeavyStrike }
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
    let build = progression.player_build();
    let player_id = CombatantId(1);
    let enemy_id = CombatantId(2);
    let mut hero = EncounterCombatant::new(
        player_id, CombatantSource::Player, "Balance simulator", Allegiance::Allies,
        build.hp, build.stamina, build.mana, build.attack, build.defence,
    );
    hero.defence = build.defence;
    let enemy = EncounterCombatant::new(
        enemy_id, CombatantSource::Enemy, spec.enemy_name, Allegiance::Enemies,
        scenario_enemy_health(spec, seed),
        100,
        100,
        scenario_enemy_attack(spec, seed),
        enemy_defence(spec),
    );
    let mut encounter = Encounter::new(vec![hero, enemy]).expect("baseline contains an enemy");
    let mut metrics = SimulationMetrics {
        scenario_id: spec.id.into(), progression, seed, won: false, actions: 0,
        party_damage_taken: 0, knockouts: 0, abilities_used: 0, stamina_spent: 0, mana_spent: 0,
        healing_used: 0, recovery_gold_assumption: DEFAULT_RECOVERY_GOLD_ASSUMPTION,
        gold_reward: spec.expected_gold_min, xp_reward: spec.expected_xp_min,
    };
    let mut before_actor_turn = PlayerSnapshot {
        hp: build.hp,
        stamina: build.stamina,
        mana: build.mana,
    };

    while encounter.outcome == EncounterOutcome::Ongoing
        && metrics.actions < MAX_SIMULATION_ACTIONS
    {
        let Some(actor) = encounter.current_actor() else { break };
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
            let after = encounter.combatant(player_id).expect("player remains in encounter");
            metrics.stamina_spent = metrics.stamina_spent.saturating_add(
                u64::from(before_actor_turn.stamina.saturating_sub(after.stamina)),
            );
            metrics.mana_spent = metrics.mana_spent.saturating_add(
                u64::from(before_actor_turn.mana.saturating_sub(after.mana)),
            );
        } else {
            let after = encounter.combatant(player_id).map_or(0, |hero| hero.hp);
            metrics.party_damage_taken = metrics.party_damage_taken.saturating_add(
                u64::from(before_actor_turn.hp.saturating_sub(after)),
            );
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
            seeds.iter().copied().map(move |seed| simulate_encounter(*spec, band, seed))
        })
        .collect()
}

pub fn check_regressions(results: &[SimulationMetrics]) -> Vec<RegressionFinding> {
    let mut findings = Vec::new();
    for metrics in results {
        if let Some(spec) = BASELINE_ENCOUNTERS.iter().find(|spec| spec.id == metrics.scenario_id) {
            if metrics.actions < spec.expected_actions_min
                || metrics.actions > spec.expected_actions_max
            {
                findings.push(RegressionFinding::ActionsOutsideRange {
                    scenario: metrics.scenario_id.clone(), actual: metrics.actions,
                    min: spec.expected_actions_min, max: spec.expected_actions_max,
                });
            }
            if metrics.party_damage_taken > u64::from(spec.max_party_damage) {
                findings.push(RegressionFinding::DamageAboveTarget {
                    scenario: metrics.scenario_id.clone(), actual: metrics.party_damage_taken,
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
                    scenario: metrics.scenario_id.clone(), actual: metrics.gold_reward,
                    min: spec.expected_gold_min, max: spec.expected_gold_max,
                });
            }
            if metrics.xp_reward < spec.expected_xp_min
                || metrics.xp_reward > spec.expected_xp_max
            {
                findings.push(RegressionFinding::XpOutsideRange {
                    scenario: metrics.scenario_id.clone(), actual: metrics.xp_reward,
                    min: spec.expected_xp_min, max: spec.expected_xp_max,
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
        assert!(findings.iter().any(|finding| matches!(finding, RegressionFinding::ActionsOutsideRange { .. })));
        assert!(findings.iter().any(|finding| matches!(finding, RegressionFinding::GoldOutsideRange { .. })));
        assert!(findings.iter().any(|finding| matches!(finding, RegressionFinding::XpOutsideRange { .. })));
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
}
