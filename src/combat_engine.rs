use crate::data::{get_ability, get_npc_resistance, AbilityId, DamageType, LacoyxPhase};
use crate::state::Player;
use crate::status_combat::{
    EnemyArchetype, EnemyProfile, EnemyStatusPayload, StatusApplication, StatusEffect,
    StatusEffectKind, StatusTick, StatusResistance,
};
use rand::Rng;

/// Defensive response to an incoming attack. Consumable represents using one
/// potion/brew in the UI; the caller handles inventory deduction and healing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefensiveChoice {
    None,
    Block,
    Dodge,
    Consumable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphedAttack {
    pub name: String,
    pub description: String,
    pub damage: u32,
    pub damage_type: DamageType,
    /// Status payload is committed with the attack so its warning and resolution cannot diverge.
    pub status_payload: Option<EnemyStatusPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyTurnIntent {
    pub damage: u32,
    pub damage_type: DamageType,
    /// A new attack commitment, if the enemy is telegraphing instead of striking now.
    pub telegraph: Option<TelegraphedAttack>,
    /// Applied only if the immediate attack connects; telegraphed attacks carry it on the commitment.
    pub status_payload: Option<EnemyStatusPayload>,
    pub logs: Vec<String>,
}

/// Per-encounter state. The persistent player only stores HP and resource
/// balances; transient status effects and boss phases reset when a fight ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub stamina: u32,
    pub max_stamina: u32,
    pub mana: u32,
    pub max_mana: u32,
    pub attack_power: u32,
    pub critical_chance_pct: u32,
    pub status_effects: Vec<StatusEffect>,
    /// Percentage chance to resist status application; capped by the shared lifecycle at 90%.
    pub status_resistance_pct: u8,
    pub shield_wall_duration: u32,
    pub is_stunned: bool,
    pub telegraphed_attack: Option<TelegraphedAttack>,
    pub lacoyx_phase: Option<LacoyxPhase>,
}

impl Combatant {
    pub fn from_player(player: &Player) -> Self {
        Self {
            name: player.username.clone(),
            hp: player.hp,
            max_hp: player.max_hp,
            stamina: player.stamina,
            max_stamina: player.max_stamina,
            mana: player.mana,
            max_mana: player.max_mana,
            attack_power: player.weapon.max_damage().max(1),
            critical_chance_pct: 5,
            status_effects: Vec::new(),
            status_resistance_pct: 0,
            shield_wall_duration: 0,
            is_stunned: false,
            telegraphed_attack: None,
            lacoyx_phase: None,
        }
    }

    pub fn new_monster(name: &str, _level: u32, hp: u32) -> Self {
        let is_lacoyx = name == "Lacoyx";
        Self {
            name: name.to_string(),
            hp,
            max_hp: hp.max(1),
            stamina: 100,
            max_stamina: 100,
            mana: 50,
            max_mana: 50,
            attack_power: 30,
            critical_chance_pct: 0,
            status_effects: Vec::new(),
            shield_wall_duration: 0,
            is_stunned: false,
            telegraphed_attack: None,
            lacoyx_phase: if is_lacoyx {
                Some(LacoyxPhase::Phase1Commander)
            } else {
                None
            },
        }
    }

    /// Apply through the shared Section 13 lifecycle using this combatant's status resistance.
    pub fn apply_status_with_chance(
        &mut self,
        kind: StatusEffectKind,
        duration: u32,
        potency: u32,
        application_chance_pct: u8,
        roll: u32,
    ) -> StatusApplication {
        let effect = StatusEffect {
            kind,
            source: None,
            potency,
            remaining_activations: duration,
            application_chance_pct,
        };
        crate::status_combat::apply_status(
            &mut self.status_effects,
            effect,
            self.status_resistance_pct,
            roll,
        )
    }

    /// Existing player abilities retain guaranteed base application chance while
    /// still passing through the shared duration, refresh and resistance policy.
    pub fn apply_status(&mut self, kind: StatusEffectKind, duration: u32, potency: u32) {
        let _ = self.apply_status_with_chance(kind, duration, potency, 100, 0);
    }

    /// The legacy battle and party encounter now share the same effect clock.
    /// Effects tick at the start of this combatant's activation, then age.
    pub fn tick_status_effects(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        self.is_stunned = false;
        let ticks = crate::status_combat::tick_status_effects(&mut self.status_effects);
        for tick in ticks {
            match tick {
                StatusTick::Damage { kind, amount } => {
                    self.hp = self.hp.saturating_sub(amount);
                    logs.push(format!(
                        "{} suffers {} {:?} damage!",
                        self.name, amount, kind
                    ));
                }
                StatusTick::Stunned => {
                    self.is_stunned = true;
                    logs.push(format!("{} is Stunned and loses this turn!", self.name));
                }
                StatusTick::Expired(_) => {}
            }
        }

        if self.shield_wall_duration > 0 {
            self.shield_wall_duration -= 1;
        }
        logs
    }

    pub fn regen_resources(&mut self) {
        self.stamina = self.stamina.saturating_add(10).min(self.max_stamina);
        self.mana = self.mana.saturating_add(5).min(self.max_mana);
    }
}

fn lacoyx_telegraph_for_phase(phase: LacoyxPhase) -> (f64, TelegraphedAttack) {
    match phase {
        LacoyxPhase::Phase1Commander => (
            0.4,
            TelegraphedAttack {
                name: "Grand Cleave".into(),
                description: "Lacoyx winds up a massive sword slash!".into(),
                damage: 80,
                damage_type: DamageType::Physical,
                status_payload: None,
            },
        ),
        LacoyxPhase::Phase2InfernalWarlord => (
            0.5,
            TelegraphedAttack {
                name: "Infernal Nova".into(),
                description: "Lacoyx channels infernal flames!".into(),
                damage: 100,
                damage_type: DamageType::Fire,
                status_payload: None,
            },
        ),
        LacoyxPhase::Phase3DesperateOverlord => (
            0.6,
            TelegraphedAttack {
                name: "Apocalyptic Strike".into(),
                description: "Lacoyx gathers apocalyptic energy!".into(),
                damage: 140,
                damage_type: DamageType::Arcane,
                status_payload: None,
            },
        ),
    }
}

pub struct CombatEngine;

impl CombatEngine {
    pub fn is_critical_roll(chance: u32, roll: u32) -> bool {
        roll % 100 < chance.min(100)
    }
    pub fn critical_hit_damage(base: u32, critical: bool) -> u32 {
        if critical {
            base.saturating_mul(2)
        } else {
            base
        }
    }

    /// Compute post-resistance and post-defence damage with integer output.
    /// Successful dodges are the only normal path that returns zero damage.
    pub fn calculate_damage(
        base_damage: u32,
        damage_type: DamageType,
        target_name: &str,
        target_shielded: bool,
        defensive_choice: DefensiveChoice,
    ) -> (u32, Vec<String>) {
        let mut logs = Vec::new();

        if defensive_choice == DefensiveChoice::Dodge {
            if rand::thread_rng().gen_bool(0.8) {
                logs.push(format!("{target_name} dodged the attack completely!"));
                return (0, logs);
            }
            logs.push(format!("{target_name} tried to dodge but failed!"));
        }

        let resistance_pct = get_npc_resistance(target_name, damage_type);
        let resistance_multiplier = (100 - resistance_pct).max(0) as f64 / 100.0;
        let mut final_damage = base_damage as f64 * resistance_multiplier;

        if resistance_pct > 0 {
            logs.push(format!(
                "{target_name} resisted {damage_type:?} damage ({resistance_pct}%)!"
            ));
        } else if resistance_pct < 0 {
            logs.push(format!(
                "{target_name} is vulnerable to {damage_type:?} damage (+{}%)!",
                -resistance_pct
            ));
        }

        if target_shielded {
            final_damage *= 0.5;
            logs.push(format!("{target_name}'s Shield Wall absorbed 50% damage!"));
        }

        match defensive_choice {
            DefensiveChoice::Block => {
                final_damage *= 0.25;
                logs.push(format!("{target_name} blocked, mitigating 75% damage!"));
            }
            DefensiveChoice::Consumable => {
                final_damage *= 0.5;
                logs.push(format!(
                    "{target_name}'s defensive brew softened the blow by 50%!"
                ));
            }
            DefensiveChoice::None | DefensiveChoice::Dodge => {}
        }

        (final_damage.clamp(1.0, u32::MAX as f64) as u32, logs)
    }

    pub fn execute_player_ability(
        player: &mut Combatant,
        target: &mut Combatant,
        ability_id: AbilityId,
    ) -> Result<Vec<String>, &'static str> {
        let ability = get_ability(ability_id);
        if player.stamina < ability.stamina_cost {
            return Err("Not enough Stamina!");
        }
        if player.mana < ability.mana_cost {
            return Err("Not enough Mana!");
        }
        if target.hp == 0 {
            return Err("The target is already defeated.");
        }

        // Validate both resources before changing either one.
        player.stamina -= ability.stamina_cost;
        player.mana -= ability.mana_cost;

        let mut logs = vec![format!("{} uses {}!", player.name, ability.name)];
        match ability_id {
            AbilityId::HeavyStrike | AbilityId::LifeDrain | AbilityId::Cleave => {
                let scaled_damage = player
                    .attack_power
                    .saturating_mul(ability.damage_multiplier)
                    / 100;
                let critical = Self::is_critical_roll(
                    player.critical_chance_pct,
                    rand::thread_rng().gen_range(0..100),
                );
                let scaled_damage = Self::critical_hit_damage(scaled_damage, critical);
                if critical {
                    logs.push("Critical hit! Ability damage doubled.".into());
                }
                let (damage, damage_logs) = Self::calculate_damage(
                    scaled_damage,
                    ability.damage_type,
                    &target.name,
                    target.shield_wall_duration > 0,
                    DefensiveChoice::None,
                );
                logs.extend(damage_logs);
                target.hp = target.hp.saturating_sub(damage);
                match ability_id {
                    AbilityId::HeavyStrike => {
                        logs.push(format!(
                            "Heavy Strike dealt {damage} damage to {}!",
                            target.name
                        ));
                        target.apply_status(StatusEffectKind::Bleed, 2, 10);
                        logs.push(format!(
                            "{} is afflicted with Bleed for 10 damage per turn!",
                            target.name
                        ));
                    }
                    AbilityId::LifeDrain => {
                        let healed = damage / 2;
                        player.hp = player.hp.saturating_add(healed).min(player.max_hp);
                        logs.push(format!(
                            "Life Drain dealt {damage} Arcane damage and healed {healed} HP!"
                        ));
                    }
                    AbilityId::Cleave => {
                        target.apply_status(StatusEffectKind::Stun, 1, 0);
                        logs.push(format!(
                            "Cleave dealt {damage} damage and stunned {}!",
                            target.name
                        ));
                    }
                    AbilityId::ShieldWall => unreachable!(),
                }
            }
            AbilityId::ShieldWall => {
                player.shield_wall_duration = 2;
                logs.push(format!(
                    "{} raised a Shield Wall to reduce incoming damage for 2 turns!",
                    player.name
                ));
            }
        }

        Ok(logs)
    }

    pub fn update_lacoyx_boss_phase(boss: &mut Combatant) -> Vec<String> {
        let mut logs = Vec::new();
        if boss.name != "Lacoyx" || boss.max_hp == 0 {
            return logs;
        }

        let hp_pct = boss.hp as f64 / boss.max_hp as f64 * 100.0;
        match boss.lacoyx_phase {
            Some(LacoyxPhase::Phase1Commander) if hp_pct <= 33.0 => {
                boss.lacoyx_phase = Some(LacoyxPhase::Phase3DesperateOverlord);
                logs.push(
                    "Lacoyx enters Phase 3: Desperate Overlord! He unleashes enraged fury!".into(),
                );
            }
            Some(LacoyxPhase::Phase1Commander | LacoyxPhase::Phase2InfernalWarlord)
                if hp_pct <= 66.0 && hp_pct > 33.0 =>
            {
                boss.lacoyx_phase = Some(LacoyxPhase::Phase2InfernalWarlord);
                logs.push(
                    "Lacoyx enters Phase 2: Infernal Warlord! Infernal flames surround him!".into(),
                );
            }
            Some(LacoyxPhase::Phase2InfernalWarlord) if hp_pct <= 33.0 => {
                boss.lacoyx_phase = Some(LacoyxPhase::Phase3DesperateOverlord);
                logs.push(
                    "Lacoyx enters Phase 3: Desperate Overlord! He unleashes enraged fury!".into(),
                );
            }
            _ => {}
        }
        logs
    }

    /// Deterministic policy for ordinary enemies. Tests provide the rolls; gameplay
    /// supplies random rolls. The typed profile is also used by party-combat AI.
    pub fn propose_legacy_enemy_intent(
        enemy_name: &str,
        attack_power: u32,
        damage_roll: u32,
        telegraph_roll: u32,
    ) -> EnemyTurnIntent {
        let profile = EnemyProfile::for_name(enemy_name);
        let damage = damage_roll % attack_power.max(1) + 1;
        let telegraph = telegraph_roll % 100 < u32::from(profile.telegraph_chance_pct);
        if telegraph {
            let (name, posture) = match profile.archetype {
                EnemyArchetype::AggressiveMelee => ("Overwhelming Strike", "draws back for a forceful melee attack"),
                EnemyArchetype::RangedPressure => ("Ranged Volley", "takes aim for a ranged attack"),
                EnemyArchetype::DefensiveProtector => ("Guarded Counter", "sets up a guarded counterattack"),
                EnemyArchetype::StatusSpecialist => ("Status Assault", "channels an attack carrying a status effect"),
            };
            let status_description = profile.status_payload.map(|payload| {
                format!(
                    " It may apply {:?} for {} activation(s) at {}% base chance.",
                    payload.kind, payload.duration, payload.application_chance_pct
                )
            }).unwrap_or_default();
            let attack = TelegraphedAttack {
                name: name.into(),
                description: format!(
                    "{} {} ({} base physical damage).{}",
                    enemy_name, posture, damage, status_description
                ),
                damage,
                damage_type: DamageType::Physical,
                status_payload: profile.status_payload,
            };
            EnemyTurnIntent {
                damage: 0,
                damage_type: DamageType::Physical,
                telegraph: Some(attack.clone()),
                status_payload: None,
                logs: vec![format!("TELEGRAPH: {} — {}", attack.name, attack.description)],
            }
        } else {
            EnemyTurnIntent {
                damage,
                damage_type: DamageType::Physical,
                telegraph: None,
                status_payload: profile.status_payload,
                logs: Vec::new(),
            }
        }
    }

    /// Resolve an existing telegraph or propose a new intent. Lacoyx keeps his
    /// phase-specific attack identities and probabilities; ordinary hostiles use
    /// the shared Section 13 tactical profile and status payload.
    pub fn generate_enemy_turn(
        enemy: &mut Combatant,
        _player: &Combatant,
    ) -> EnemyTurnIntent {
        let mut rng = rand::thread_rng();

        if let Some(telegraphed) = enemy.telegraphed_attack.take() {
            return EnemyTurnIntent {
                damage: telegraphed.damage,
                damage_type: telegraphed.damage_type,
                telegraph: None,
                status_payload: telegraphed.status_payload,
                logs: vec![format!(
                    "{} strikes with telegraphed move {}!",
                    enemy.name, telegraphed.name
                )],
            };
        }

        if enemy.name == "Lacoyx" {
            let phase = enemy.lacoyx_phase.unwrap_or(LacoyxPhase::Phase1Commander);
            let (chance, attack) = match phase {
                LacoyxPhase::Phase1Commander => (
                    0.4,
                    TelegraphedAttack {
                        name: "Grand Cleave".into(),
                        description: "Lacoyx winds up a massive sword slash!".into(),
                        damage: 80,
                        damage_type: DamageType::Physical,
                        status_payload: None,
                    },
                ),
                LacoyxPhase::Phase2InfernalWarlord => (
                    0.5,
                    TelegraphedAttack {
                        name: "Infernal Nova".into(),
                        description: "Lacoyx channels infernal flames!".into(),
                        damage: 100,
                        damage_type: DamageType::Fire,
                        status_payload: None,
                    },
                ),
                LacoyxPhase::Phase3DesperateOverlord => (
                    0.6,
                    TelegraphedAttack {
                        name: "Apocalyptic Strike".into(),
                        description: "Lacoyx gathers apocalyptic energy!".into(),
                        damage: 140,
                        damage_type: DamageType::Arcane,
                        status_payload: None,
                    },
                ),
            };
            if rng.gen_bool(chance) {
                let logs = vec![format!(
                    "TELEGRAPH: {} — {}",
                    attack.name, attack.description
                )];
                enemy.telegraphed_attack = Some(attack.clone());
                return EnemyTurnIntent {
                    damage: 0,
                    damage_type: attack.damage_type,
                    telegraph: Some(attack.clone()),
                    status_payload: None,
                    logs,
                };
            }
            return EnemyTurnIntent {
                damage: rng.gen_range(1..=enemy.attack_power.max(1)),
                damage_type: DamageType::Physical,
                telegraph: None,
                status_payload: None,
                logs: Vec::new(),
            };
        }

        let intent = Self::propose_legacy_enemy_intent(
            &enemy.name,
            enemy.attack_power,
            rng.gen_range(0..u32::MAX),
            rng.gen_range(0..100),
        );
        if let Some(attack) = intent.telegraph.clone() {
            enemy.telegraphed_attack = Some(attack);
        }
        intent
    }

    /// Apply a profile payload only after an attack connects. A successful dodge
    /// is represented by zero mitigated damage in the legacy resolver.
    pub fn apply_enemy_status_payload(
        source_name: &str,
        target: &mut Combatant,
        payload: Option<EnemyStatusPayload>,
        roll: u32,
    ) -> Vec<String> {
        let Some(payload) = payload else {
            return Vec::new();
        };
        match target.apply_status_with_chance(
            payload.kind,
            payload.duration,
            payload.potency,
            payload.application_chance_pct,
            roll,
        ) {
            StatusApplication::Applied => vec![format!(
                "{}'s attack inflicted {:?} on {}.",
                source_name, payload.kind, target.name
            )],
            StatusApplication::Refreshed => vec![format!(
                "{}'s attack refreshed {:?} on {}.",
                source_name, payload.kind, target.name
            )],
            StatusApplication::Resisted => vec![format!(
                "{} resisted the {:?} effect.",
                target.name, payload.kind
            )],
            StatusApplication::FailedChance | StatusApplication::InvalidDuration => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AbilityId, LacoyxPhase};
    use crate::state::Player;

    fn player() -> Combatant {
        Combatant::from_player(&Player::new("TestHero".into(), String::new()))
    }

    #[test]
    fn critical_helpers_respect_chance_and_saturate() {
        assert!(CombatEngine::is_critical_roll(25, 24));
        assert!(!CombatEngine::is_critical_roll(25, 25));
        assert!(!CombatEngine::is_critical_roll(0, 0));
        assert_eq!(CombatEngine::critical_hit_damage(100, true), 200);
        assert_eq!(CombatEngine::critical_hit_damage(u32::MAX, true), u32::MAX);
    }
    #[test]
    fn abilities_reject_insufficient_resources_without_deducting_anything() {
        let mut player = player();
        let mut target = Combatant::new_monster("Goblin", 10, 100);
        player.stamina = 10;
        let stamina_before = player.stamina;
        let mana_before = player.mana;

        assert_eq!(
            CombatEngine::execute_player_ability(&mut player, &mut target, AbilityId::HeavyStrike),
            Err("Not enough Stamina!")
        );
        assert_eq!(player.stamina, stamina_before);
        assert_eq!(player.mana, mana_before);
    }

    #[test]
    fn status_effects_tick_and_reapplication_refreshes_without_stacking() {
        let mut target = Combatant::new_monster("Goblin", 10, 100);
        target.apply_status(StatusEffectKind::Bleed, 2, 10);
        target.apply_status(StatusEffectKind::Bleed, 4, 15);
        assert_eq!(target.status_effects.len(), 1);
        assert_eq!(target.status_effects[0].remaining_activations, 4);
        assert_eq!(target.status_effects[0].potency, 15);

        let logs = target.tick_status_effects();
        assert_eq!(target.hp, 85);
        assert_eq!(target.status_effects[0].remaining_activations, 3);
        assert!(!logs.is_empty());
    }

    #[test]
    fn elemental_resistances_and_weaknesses_adjust_damage() {
        let (dragon, logs) = CombatEngine::calculate_damage(
            100,
            DamageType::Fire,
            "Dragon",
            false,
            DefensiveChoice::None,
        );
        assert_eq!(dragon, 25);
        assert!(logs.iter().any(|line| line.contains("resisted")));

        let (ent, logs) = CombatEngine::calculate_damage(
            100,
            DamageType::Fire,
            "Ent",
            false,
            DefensiveChoice::None,
        );
        assert_eq!(ent, 150);
        assert!(logs.iter().any(|line| line.contains("vulnerable")));
    }

    #[test]
    fn lacoyx_phase_transitions_follow_hp_thresholds() {
        let mut lacoyx = Combatant::new_monster("Lacoyx", 741, 1000);
        assert_eq!(lacoyx.lacoyx_phase, Some(LacoyxPhase::Phase1Commander));

        lacoyx.hp = 600;
        let phase_two_logs = CombatEngine::update_lacoyx_boss_phase(&mut lacoyx);
        assert!(phase_two_logs.iter().any(|line| line.contains("Phase 2")));
        assert_eq!(
            lacoyx.lacoyx_phase,
            Some(LacoyxPhase::Phase2InfernalWarlord)
        );

        lacoyx.hp = 300;
        let phase_three_logs = CombatEngine::update_lacoyx_boss_phase(&mut lacoyx);
        assert!(phase_three_logs.iter().any(|line| line.contains("Phase 3")));
        assert_eq!(
            lacoyx.lacoyx_phase,
            Some(LacoyxPhase::Phase3DesperateOverlord)
        );
    }

    #[test]
    fn committed_telegraph_resolves_its_exact_attack_once_then_clears() {
        let mut enemy = Combatant::new_monster("Lacoyx", 741, 1000);
        let player = player();
        let attack = TelegraphedAttack {
            name: "Infernal Nova".into(),
            description: "Lacoyx channels infernal flames!".into(),
            damage: 100,
            damage_type: DamageType::Fire,
            status_payload: None,
        };
        enemy.telegraphed_attack = Some(attack.clone());

        let intent = CombatEngine::generate_enemy_turn(&mut enemy, &player);
        assert_eq!(intent.damage, attack.damage);
        assert_eq!(intent.damage_type, attack.damage_type);
        assert_eq!(intent.telegraph, None);
        assert_eq!(intent.status_payload, None);
        assert!(intent.logs.iter().any(|line| line.contains("Infernal Nova")));
        assert_eq!(enemy.telegraphed_attack, None);
    }

    #[test]
    fn shared_enemy_profiles_drive_legacy_status_intents_deterministically() {
        let mage = CombatEngine::propose_legacy_enemy_intent("Stronghold Mage", 40, 7, 0);
        assert_eq!(mage.damage, 0);
        let mage_telegraph = mage.telegraph.expect("telegraph roll 0 is below the Mage profile threshold");
        assert_eq!(mage_telegraph.damage, 8);
        assert_eq!(mage_telegraph.status_payload.unwrap().kind, StatusEffectKind::Slow);
        assert!(mage_telegraph.description.contains("Slow"));

        let ranger = CombatEngine::propose_legacy_enemy_intent("Stronghold Ranger", 40, 6, 99);
        assert_eq!(ranger.damage, 7);
        assert!(ranger.telegraph.is_none());
        assert_eq!(ranger.status_payload.unwrap().kind, StatusEffectKind::Mark);

        let warrior = CombatEngine::propose_legacy_enemy_intent("Stronghold Warrior", 40, 3, 99);
        assert_eq!(warrior.damage, 4);
        assert_eq!(
            warrior.status_payload.unwrap().kind,
            StatusEffectKind::WeakenedDefences
        );
    }

    #[test]
    fn legacy_enemy_status_payload_uses_shared_resistance_and_application_chance() {
        let mut target = Combatant::new_monster("TestHero", 1, 100);
        target.status_resistance_pct = 90;
        let payload = EnemyProfile::for_name("Stronghold Mage").status_payload.unwrap();

        let logs = CombatEngine::apply_enemy_status_payload(
            "Stronghold Mage",
            &mut target,
            Some(payload),
            4,
        );
        assert!(target.status_effects.is_empty());
        assert!(logs.iter().any(|line| line.contains("resisted")));
    }

    #[test]
    fn legacy_battle_status_tick_uses_shared_effect_rules_for_every_dot_kind() {
        for kind in [
            StatusEffectKind::Poison,
            StatusEffectKind::Bleed,
            StatusEffectKind::Burn,
        ] {
            let mut target = Combatant::new_monster("Goblin", 10, 100);
            target.apply_status(kind, 2, 7);
            let logs = target.tick_status_effects();
            assert_eq!(
                target.hp, 93,
                "{kind:?} should tick through shared lifecycle"
            );
            assert_eq!(target.status_effects[0].remaining_activations, 1);
            assert!(logs.iter().any(|line| line.contains("7")));
        }
    }

    #[test]
    fn block_and_consumable_reduce_damage() {
        assert_eq!(
            CombatEngine::calculate_damage(
                100,
                DamageType::Physical,
                "Hero",
                false,
                DefensiveChoice::Block,
            )
            .0,
            25
        );
        assert_eq!(
            CombatEngine::calculate_damage(
                100,
                DamageType::Physical,
                "Hero",
                false,
                DefensiveChoice::Consumable,
            )
            .0,
            50
        );
    }

    #[test]
    fn resources_regenerate_and_stay_within_their_caps() {
        let mut combatant = player();
        combatant.stamina = 50;
        combatant.mana = 20;
        combatant.regen_resources();
        assert_eq!(combatant.stamina, 60);
        assert_eq!(combatant.mana, 25);

        combatant.stamina = combatant.max_stamina;
        combatant.mana = combatant.max_mana;
        combatant.regen_resources();
        assert_eq!(combatant.stamina, combatant.max_stamina);
        assert_eq!(combatant.mana, combatant.max_mana);
    }

    #[test]
    fn abilities_apply_bleed_stun_and_life_drain() {
        let mut player = player();
        let mut target = Combatant::new_monster("Goblin", 10, 2000);

        let before = target.hp;
        let logs =
            CombatEngine::execute_player_ability(&mut player, &mut target, AbilityId::HeavyStrike)
                .unwrap();
        assert!(target.hp < before);
        assert_eq!(target.status_effects[0].kind, StatusEffectKind::Bleed);
        assert!(logs.iter().any(|line| line.contains("Bleed")));

        let logs =
            CombatEngine::execute_player_ability(&mut player, &mut target, AbilityId::Cleave)
                .unwrap();
        assert!(target
            .status_effects
            .iter()
            .any(|s| s.kind == StatusEffectKind::Stun));
        assert!(logs.iter().any(|line| line.contains("stunned")));
        let stun_logs = target.tick_status_effects();
        assert!(target.is_stunned);
        assert!(stun_logs
            .iter()
            .any(|line| line.contains("loses this turn")));

        player.hp = 10;
        let logs =
            CombatEngine::execute_player_ability(&mut player, &mut target, AbilityId::LifeDrain)
                .unwrap();
        assert!(player.hp > 10);
        assert!(logs.iter().any(|line| line.contains("Life Drain")));
    }
}
