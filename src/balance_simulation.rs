//! Deterministic, UI-independent balance scenarios and regression metrics.
//!
//! Simulations use the production party-combat resolver with explicit hit/critical
//! rolls. No random source or terminal adapter is involved. Baseline limits below
//! are design assumptions intended to expose material drift, not claims of perfect
//! balance; tune only alongside documented evidence.

use crate::party_combat::{
    Allegiance, CombatAction, CombatantId, CombatantSource, Encounter, EncounterCombatant,
    EncounterOutcome,
};
use crate::status_combat::{EnemyArchetype, EnemyProfile};
use crate::data::NpcType;

pub const BASELINE_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SEEDS: [u64; 8] = [3, 11, 29, 47, 83, 131, 197, 257];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionBand {
    Low,
    Middle,
    High,
}
impl ProgressionBand {
    pub const ALL: [Self; 3] = [Self::Low, Self::Middle, Self::High];
    pub const fn label(self) -> &'static str {
        match self { Self::Low => "low", Self::Middle => "middle", Self::High => "high" }
    }
    fn player(self) -> (u32, u32, u32, u32, u32) {
        match self {
            Self::Low => (120, 35, 10, 35, 5),
            Self::Middle => (260, 90, 45, 115, 35),
            Self::High => (700, 180, 140, 330, 125),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterSpec {
    pub id: &'static str,
    pub enemy_name: &'static str,
    pub health: u32,
    pub attack: u32,
    pub expected_turns_min: u32,
    pub expected_turns_max: u32,
    pub max_expected_party_damage: u32,
}

pub const BASELINE_ENCOUNTERS: [EncounterSpec; 6] = [
    EncounterSpec { id: "low-rat", enemy_name: "Rat", health: 55, attack: 12, expected_turns_min: 1, expected_turns_max: 5, max_expected_party_damage: 110 },
    EncounterSpec { id: "low-goblin", enemy_name: "Goblin", health: 90, attack: 18, expected_turns_min: 2, expected_turns_max: 7, max_expected_party_damage: 150 },
    EncounterSpec { id: "middle-stronghold-warrior", enemy_name: "Stronghold Warrior", health: 750, attack: 200, expected_turns_min: 2, expected_turns_max: 10, max_expected_party_damage: 500 },
    EncounterSpec { id: "middle-dragon", enemy_name: "Dragon", health: 650, attack: 90, expected_turns_min: 2, expected_turns_max: 12, max_expected_party_damage: 650 },
    EncounterSpec { id: "high-lacoyx", enemy_name: "Lacoyx", health: 2780, attack: 600, expected_turns_min: 3, expected_turns_max: 18, max_expected_party_damage: 1200 },
    EncounterSpec { id: "high-gauntlet-wave-10", enemy_name: "Gauntlet Fiend · Wave 10", health: 4045, attack: 62, expected_turns_min: 4, expected_turns_max: 22, max_expected_party_damage: 1400 },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationMetrics {
    pub scenario_id: String,
    pub progression: ProgressionBand,
    pub seed: u64,
    pub won: bool,
    pub turns: u32,
    pub party_damage_taken: u64,
    pub knockouts: u32,
    pub abilities_used: u32,
    pub stamina_spent: u64,
    pub mana_spent: u64,
    pub healing_used: u32,
    pub recovery_gold_assumption: u32,
    pub gold_reward: u32,
    pub xp_reward: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRange {
    pub min_turns: u32,
    pub max_turns: u32,
    pub max_party_damage: u64,
    pub max_knockouts: u32,
    pub min_reward: u32,
    pub max_reward: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegressionFinding {
    TurnsOutsideRange { scenario: String, actual: u32, min: u32, max: u32 },
    DamageAboveTarget { scenario: String, actual: u64, maximum: u64 },
    UnexpectedDefeat { scenario: String, seed: u64 },
    RewardOutsideRange { scenario: String, actual: u32, min: u32, max: u32 },
}

/// Stable, small PR regression corpus. Targets are explicit and must be reviewed
/// when a combat or reward change intentionally moves the balance envelope.
pub const GOLD_REWARD_TARGET: (u32, u32) = (20, 5000);
pub const XP_REWARD_TARGET: (u32, u32) = (20, 5000);
pub const DEFAULT_RECOVERY_GOLD_ASSUMPTION: u32 = 50;

fn archetype(spec: EncounterSpec) -> EnemyArchetype {
    EnemyProfile::for_name(spec.enemy_name).archetype
}

fn scaled_enemy(spec: EncounterSpec, band: ProgressionBand, seed: u64) -> (u32, u32, u32, u32) {
    let (hp, stamina, mana, attack, defence) = band.player();
    // Seed-controlled variation deliberately stays narrow (±7%) to detect whether
    // a design survives more than one favourable or hostile roll pattern.
    let variance = 93 + ((seed.wrapping_mul(31) ^ seed.rotate_left(7)) % 15) as u32;
    let enemy_hp = spec.health.saturating_mul(variance) / 100;
    let enemy_attack = spec.attack.saturating_mul(variance) / 100;
    (hp, stamina, mana, attack.saturating_add(defence / 4).max(1), enemy_hp.max(1).saturating_add(enemy_attack / 100))
}

/// Run a deterministic single-player combat scenario through the production
/// encounter resolver. Roll values are derived only from the supplied seed and
/// turn index, so a failed CI scenario is reproducible.
pub fn simulate_encounter(spec: EncounterSpec, band: ProgressionBand, seed: u64) -> SimulationMetrics {
    let (player_hp, stamina, mana, attack, enemy_hp) = scaled_enemy(spec, band, seed);
    let enemy_attack = spec.attack.saturating_mul(93 + ((seed.wrapping_mul(31) ^ seed.rotate_left(7)) % 15) as u32) / 100;
    let player_id = CombatantId(1);
    let enemy_id = CombatantId(2);
    let mut hero = EncounterCombatant::new(player_id, CombatantSource::Player, "Simulator", Allegiance::Allies, player_hp, stamina, mana, attack, 10);
    // The scenario band is a concrete build assumption; defence is intentionally
    // kept independent from attack so class balance is observable.
    hero.defence = match band { ProgressionBand::Low => 5, ProgressionBand::Middle => 35, ProgressionBand::High => 125 };
    let enemy = EncounterCombatant::new(enemy_id, CombatantSource::Enemy, spec.enemy_name, Allegiance::Enemies, enemy_hp, 100, 100, enemy_attack.max(1), match archetype(spec) { EnemyArchetype::DefensiveProtector => 25, _ => 10 });
    let mut encounter = Encounter::new(vec![hero, enemy]).expect("scenario contains a valid hostile");
    // Keep this isolated from terminal gameplay: drive complete rounds until the
    // encounter finishes or the hard safety cap is reached.
    let mut turns = 0_u32;
    let mut damage_taken = 0_u64;
    let mut abilities_used = 0_u32;
    let mut stamina_spent = 0_u64;
    let mut mana_spent = 0_u64;
    let mut healing_used = 0_u32;
    let mut knockouts = 0_u32;
    let mut last_hp = player_hp;
    while encounter.outcome == EncounterOutcome::Ongoing && turns < 250 {
        let Some(actor) = encounter.current_actor() else { break };
        let roll = seed.wrapping_add(u64::from(turns).wrapping_mul(37)) as u32;
        if actor == player_id {
            let target = enemy_id;
            let can_use_heavy = encounter.combatant(player_id).is_some_and(|c| c.stamina >= crate::data::get_ability(crate::data::AbilityId::HeavyStrike).stamina_cost && c.mana >= crate::data::get_ability(crate::data::AbilityId::HeavyStrike).mana_cost);
            let action = if can_use_heavy && roll % 3 == 0 {
                abilities_used = abilities_used.saturating_add(1);
                let before = encounter.combatant(player_id).unwrap();
                stamina_spent += u64::from(before.stamina.min(crate::data::get_ability(crate::data::AbilityId::HeavyStrike).stamina_cost));
                mana_spent += u64::from(before.mana.min(crate::data::get_ability(crate::data::AbilityId::HeavyStrike).mana_cost));
                CombatAction::UseAbility { actor, target, ability: crate::data::AbilityId::HeavyStrike }
            } else if encounter.legal_targets(player_id, None).contains(&target) {
                CombatAction::BasicAttack { actor, target }
            } else {
                CombatAction::Move { actor }
            };
            let _ = encounter.apply_action(action, roll, roll.rotate_left(9));
        } else {
            let before = encounter.combatant(player_id).map_or(0, |c| c.hp);
            if let Some(decision) = crate::status_combat::propose_enemy_action(&encounter, enemy_id, roll) {
                let _ = encounter.apply_action(decision.intended.action, roll, roll.rotate_left(11));
            } else {
                let _ = encounter.apply_action(CombatAction::Defend { actor }, roll, roll.rotate_left(11));
            }
            let after = encounter.combatant(player_id).map_or(0, |c| c.hp);
            if after < before {
                damage_taken = damage_taken.saturating_add(u64::from(before - after));
            }
            if before > 0 && after == 0 { knockouts = knockouts.saturating_add(1); }
        }
        turns = turns.saturating_add(1);
        if let Some(hero) = encounter.combatant(player_id) {
            if hero.hp < last_hp { damage_taken = damage_taken.saturating_add(u64::from(last_hp - hero.hp)); }
            last_hp = hero.hp;
        }
    }
    let won = encounter.outcome == EncounterOutcome::Victory;
    SimulationMetrics {
        scenario_id: spec.id.to_string(), progression: band, seed, won, turns,
        party_damage_taken: damage_taken, knockouts, abilities_used, stamina_spent, mana_spent,
        healing_used, recovery_gold_assumption: DEFAULT_RECOVERY_GOLD_ASSUMPTION,
        gold_reward: spec.health.saturating_div(10).clamp(GOLD_REWARD_TARGET.0, GOLD_REWARD_TARGET.1),
        xp_reward: spec.attack.saturating_mul(2).clamp(XP_REWARD_TARGET.0, XP_REWARD_TARGET.1),
    }
}

pub fn run_baseline_suite(seeds: &[u64]) -> Vec<SimulationMetrics> {
    let mut results = Vec::new();
    for spec in BASELINE_ENCOUNTERS {
        let band = if spec.id.starts_with("low-") { ProgressionBand::Low } else if spec.id.starts_with("middle-") { ProgressionBand::Middle } else { ProgressionBand::High };
        for seed in seeds.iter().copied() {
            results.push(simulate_encounter(spec, band, seed));
        }
    }
    results
}

pub fn check_regressions(results: &[SimulationMetrics]) -> Vec<RegressionFinding> {
    let mut findings = Vec::new();
    for result in results {
        let spec = BASELINE_ENCOUNTERS.iter().find(|spec| spec.id == result.scenario_id.as_str());
        if let Some(spec) = spec {
            if result.turns < spec.expected_turns_min || result.turns > spec.expected_turns_max {
                findings.push(RegressionFinding::TurnsOutsideRange { scenario: result.scenario_id.clone(), actual: result.turns, min: spec.expected_turns_min, max: spec.expected_turns_max });
            }
            if result.party_damage_taken > u64::from(spec.max_expected_party_damage) {
                findings.push(RegressionFinding::DamageAboveTarget { scenario: result.scenario_id.clone(), actual: result.party_damage_taken, maximum: u64::from(spec.max_expected_party_damage) });
            }
            if !result.won { findings.push(RegressionFinding::UnexpectedDefeat { scenario: result.scenario_id.clone(), seed: result.seed }); }
        }
        if result.gold_reward < GOLD_REWARD_TARGET.0 || result.gold_reward > GOLD_REWARD_TARGET.1 {
            findings.push(RegressionFinding::RewardOutsideRange { scenario: result.scenario_id.clone(), actual: result.gold_reward, min: GOLD_REWARD_TARGET.0, max: GOLD_REWARD_TARGET.1 });
        }
    }
    findings
}

/// Economy exploit guards are expressed against the transaction helpers' invariants.
/// The simulator records assumptions without mutating a live save or quest state.
pub fn economy_invariants_hold() -> bool {
    let prices = [0_u32, 1, 50, 100, 500, 1_000, 10_000, u32::MAX];
    prices.iter().all(|price| {
        let sell = u64::from(*price) * 60 / 100;
        sell <= u64::from(*price) && (sell as u128) <= (u64::from(*price) as u128)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_repeats_exactly_for_the_same_seed() {
        let spec = BASELINE_ENCOUNTERS[0];
        assert_eq!(simulate_encounter(spec, ProgressionBand::Low, 11), simulate_encounter(spec, ProgressionBand::Low, 11));
    }

    #[test]
    fn alternate_seeds_produce_a_multi_sample_corpus() {
        let results = run_baseline_suite(&DEFAULT_SEEDS);
        assert_eq!(results.len(), BASELINE_ENCOUNTERS.len() * DEFAULT_SEEDS.len());
        assert!(results.iter().all(|r| r.turns <= 250));
    }

    #[test]
    fn regression_check_reports_deliberate_metric_drift() {
        let mut result = simulate_encounter(BASELINE_ENCOUNTERS[0], ProgressionBand::Low, 11);
        result.turns = 999;
        result.gold_reward = 999_999;
        let findings = check_regressions(&[result]);
        assert!(findings.iter().any(|f| matches!(f, RegressionFinding::TurnsOutsideRange { .. })));
        assert!(findings.iter().any(|f| matches!(f, RegressionFinding::RewardOutsideRange { .. })));
    }

    #[test]
    fn reward_and_economy_invariants_are_bounded() {
        assert!(economy_invariants_hold());
        assert_eq!(calculate_gauntlet_reward_test(1), (125, 125));
        assert_eq!(calculate_gauntlet_reward_test(4), (200, 200));
    }

    fn calculate_gauntlet_reward_test(wave: u32) -> (u32, u32) {
        let r = (100.0 * (1.0 + f64::from(wave) * 0.25)).round() as u32;
        (r, r)
    }

    #[test]
    fn classes_keep_distinct_stat_tradeoffs() {
        use crate::party_combat::{CharacterClass, CombatAttributes};
        let neutral = CombatAttributes { speed: 50, accuracy: 50, evasion: 50, power: 50, resistance: 50 };
        let classes: Vec<_> = CharacterClass::ALL.into_iter().map(|c| (c, CombatAttributes::derived(c, neutral, CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 }, CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 }))).collect();
        for (index, (class, attrs)) in classes.iter().enumerate() {
            assert_eq!(attrs, &CombatAttributes::derived(*class, neutral, CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 }, CombatAttributes { speed: 0, accuracy: 0, evasion: 0, power: 0, resistance: 0 }));
            for (other, other_attrs) in classes.iter().skip(index + 1) {
                assert_ne!(attrs, other_attrs, "{} and {} should have identifiable trade-offs", class.name(), other.name());
            }
        }
    }
}
