use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use crate::data::{ArmourTier, DamageType, StatusEffectKind, WeaponTier};

/// Maximum number of companions that can be active in a party at once,
/// including the player character. The player always occupies one slot.
pub const DEFAULT_PARTY_SIZE_LIMIT: usize = 4;

/// Stable identifier for a companion definition. Each definition has a unique
/// ID that never changes across saves; the mutable `CompanionState` references
/// this ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionId {
    BranTheShield,
    LyraSwiftbow,
    MarenStarweave,
    GarrickIronfist,
    SeraNightwhisper,
    TorvalEmbershield,
}

impl CompanionId {
    pub const ALL: [CompanionId; 6] = [
        CompanionId::BranTheShield,
        CompanionId::LyraSwiftbow,
        CompanionId::MarenStarweave,
        CompanionId::GarrickIronfist,
        CompanionId::SeraNightwhisper,
        CompanionId::TorvalEmbershield,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            CompanionId::BranTheShield => "bran_the_shield",
            CompanionId::LyraSwiftbow => "lyra_swiftbow",
            CompanionId::MarenStarweave => "maren_starweave",
            CompanionId::GarrickIronfist => "garrick_ironfist",
            CompanionId::SeraNightwhisper => "sera_nightwhisper",
            CompanionId::TorvalEmbershield => "torval_embershield",
        }
    }
}

impl fmt::Display for CompanionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Combat role / class for a companion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionRole {
    FrontlineDefence,
    RangedDamage,
    MagicSupport,
}

impl CompanionRole {
    pub fn name(&self) -> &'static str {
        match self {
            CompanionRole::FrontlineDefence => "Frontline Defence",
            CompanionRole::RangedDamage => "Ranged Damage",
            CompanionRole::MagicSupport => "Magic & Support",
        }
    }
}

/// Condition of a companion, determining whether they can be selected for a party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompanionCondition {
    /// Ready to join a party.
    #[default]
    Active,
    /// Recruited but not currently in the active party.
    Available,
    /// Defeated in combat and needs recovery.
    KnockedOut,
    /// Currently recovering from injury.
    Recovering,
    /// Temporarily unavailable for story or other reasons.
    Unavailable,
}

impl CompanionCondition {
    pub fn name(&self) -> &'static str {
        match self {
            CompanionCondition::Active => "Active",
            CompanionCondition::Available => "Available",
            CompanionCondition::KnockedOut => "Knocked Out",
            CompanionCondition::Recovering => "Recovering",
            CompanionCondition::Unavailable => "Unavailable",
        }
    }

    /// Whether a companion in this condition can be added to an active party.
    pub fn can_join_party(&self) -> bool {
        matches!(
            self,
            CompanionCondition::Active | CompanionCondition::Available
        )
    }
}

/// Target selection for a companion ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionAbilityTarget {
    /// Targets a single enemy.
    SingleEnemy,
    /// Targets all enemies.
    AllEnemies,
    /// Targets the companion themselves.
    SelfTarget,
    /// Targets a single ally (including the player).
    SingleAlly,
    /// Targets all allies (including the player).
    AllAllies,
}

impl CompanionAbilityTarget {
    pub fn name(&self) -> &'static str {
        match self {
            CompanionAbilityTarget::SingleEnemy => "Single Enemy",
            CompanionAbilityTarget::AllEnemies => "All Enemies",
            CompanionAbilityTarget::SelfTarget => "Self",
            CompanionAbilityTarget::SingleAlly => "Single Ally",
            CompanionAbilityTarget::AllAllies => "All Allies",
        }
    }
}

/// An effect produced by a companion ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionAbilityEffect {
    /// Deals damage based on the ability's damage multiplier.
    Damage {
        multiplier: u32,
        damage_type: DamageType,
        /// Percentage of the target's defence ignored by this hit (0-100).
        defence_piercing_percent: u32,
    },
    /// Restores HP to the target.
    Heal { amount: u32 },
    /// Temporarily increases the target's attack power.
    BuffAttack { amount: u32, duration: u32 },
    /// Temporarily increases the target's defence.
    BuffDefence { amount: u32, duration: u32 },
    /// Temporarily decreases the target's defence.
    DebuffDefence { amount: u32, duration: u32 },
    /// Applies a status effect to the target.
    ApplyStatus {
        kind: StatusEffectKind,
        duration: u32,
        potency: u32,
    },
    /// Redirects an ally's incoming damage to the companion who used the ability.
    RedirectDamage { duration: u32 },
    /// Guarantees the next attack is a critical hit.
    GuaranteedCritical,
    /// Removes all negative status effects from the target.
    Cleanse,
}

/// Stable identifier for a companion-specific ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionAbilityId {
    // Frontline Defence abilities
    ShieldBash,
    GuardAlly,
    IronWill,
    // Ranged Damage abilities
    PiercingShot,
    VolleyArrows,
    AimedShot,
    // Magic & Support abilities
    HealingLight,
    ArcaneBlast,
    Empower,
    FlameWave,
    FrostBarrier,
    MassHeal,
}

impl CompanionAbilityId {
    pub const ALL: [CompanionAbilityId; 12] = [
        CompanionAbilityId::ShieldBash,
        CompanionAbilityId::GuardAlly,
        CompanionAbilityId::IronWill,
        CompanionAbilityId::PiercingShot,
        CompanionAbilityId::VolleyArrows,
        CompanionAbilityId::AimedShot,
        CompanionAbilityId::HealingLight,
        CompanionAbilityId::ArcaneBlast,
        CompanionAbilityId::Empower,
        CompanionAbilityId::FlameWave,
        CompanionAbilityId::FrostBarrier,
        CompanionAbilityId::MassHeal,
    ];
}

/// A companion ability definition. These are data-driven and describe what the
/// ability does without implementing the actual combat execution (which is
/// owned by Section 11). Multiple effects allow one ability to combine damage,
/// status changes, healing, and buffs without lying about its behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanionAbility {
    pub id: CompanionAbilityId,
    pub name: &'static str,
    pub description: &'static str,
    pub stamina_cost: u32,
    pub mana_cost: u32,
    pub cooldown_turns: u32,
    pub target: CompanionAbilityTarget,
    pub effects: Vec<CompanionAbilityEffect>,
}

/// Returns the full definition of a companion ability by ID.
pub fn get_companion_ability(id: CompanionAbilityId) -> CompanionAbility {
    match id {
        CompanionAbilityId::ShieldBash => CompanionAbility {
            id,
            name: "Shield Bash",
            description: "Bashes an enemy with a shield, dealing light damage and stunning them for 1 turn.",
            stamina_cost: 25,
            mana_cost: 0,
            cooldown_turns: 3,
            target: CompanionAbilityTarget::SingleEnemy,
            effects: vec![
                CompanionAbilityEffect::Damage {
                    multiplier: 75,
                    damage_type: DamageType::Physical,
                    defence_piercing_percent: 0,
                },
                CompanionAbilityEffect::ApplyStatus {
                    kind: StatusEffectKind::Stun,
                    duration: 1,
                    potency: 1,
                },
            ],
        },
        CompanionAbilityId::GuardAlly => CompanionAbility {
            id,
            name: "Guard Ally",
            description: "Protects one ally by redirecting damage they would take to the companion for 2 turns.",
            stamina_cost: 20,
            mana_cost: 0,
            cooldown_turns: 4,
            target: CompanionAbilityTarget::SingleAlly,
            effects: vec![CompanionAbilityEffect::RedirectDamage { duration: 2 }],
        },
        CompanionAbilityId::IronWill => CompanionAbility {
            id,
            name: "Iron Will",
            description: "Steels themselves, restoring 40 HP and increasing defence by 20 for 2 turns.",
            stamina_cost: 15,
            mana_cost: 0,
            cooldown_turns: 5,
            target: CompanionAbilityTarget::SelfTarget,
            effects: vec![
                CompanionAbilityEffect::Heal { amount: 40 },
                CompanionAbilityEffect::BuffDefence {
                    amount: 20,
                    duration: 2,
                },
            ],
        },
        CompanionAbilityId::PiercingShot => CompanionAbility {
            id,
            name: "Piercing Shot",
            description: "Fires a high-velocity arrow that deals heavy damage and ignores 25% of the target's defence.",
            stamina_cost: 20,
            mana_cost: 0,
            cooldown_turns: 2,
            target: CompanionAbilityTarget::SingleEnemy,
            effects: vec![CompanionAbilityEffect::Damage {
                multiplier: 180,
                damage_type: DamageType::Physical,
                defence_piercing_percent: 25,
            }],
        },
        CompanionAbilityId::VolleyArrows => CompanionAbility {
            id,
            name: "Volley of Arrows",
            description: "Fires a spread of arrows, dealing moderate damage to all enemies.",
            stamina_cost: 30,
            mana_cost: 0,
            cooldown_turns: 4,
            target: CompanionAbilityTarget::AllEnemies,
            effects: vec![CompanionAbilityEffect::Damage {
                multiplier: 100,
                damage_type: DamageType::Physical,
                defence_piercing_percent: 0,
            }],
        },
        CompanionAbilityId::AimedShot => CompanionAbility {
            id,
            name: "Aimed Shot",
            description: "Takes careful aim, guaranteeing the next attack is a critical hit.",
            stamina_cost: 15,
            mana_cost: 0,
            cooldown_turns: 3,
            target: CompanionAbilityTarget::SelfTarget,
            effects: vec![CompanionAbilityEffect::GuaranteedCritical],
        },
        CompanionAbilityId::HealingLight => CompanionAbility {
            id,
            name: "Healing Light",
            description: "Channels arcane energy to restore a moderate amount of HP to one ally.",
            stamina_cost: 0,
            mana_cost: 25,
            cooldown_turns: 2,
            target: CompanionAbilityTarget::SingleAlly,
            effects: vec![CompanionAbilityEffect::Heal { amount: 50 }],
        },
        CompanionAbilityId::ArcaneBlast => CompanionAbility {
            id,
            name: "Arcane Blast",
            description: "Unleashes a burst of arcane energy, dealing magic damage to one enemy.",
            stamina_cost: 0,
            mana_cost: 20,
            cooldown_turns: 1,
            target: CompanionAbilityTarget::SingleEnemy,
            effects: vec![CompanionAbilityEffect::Damage {
                multiplier: 150,
                damage_type: DamageType::Arcane,
                defence_piercing_percent: 0,
            }],
        },
        CompanionAbilityId::Empower => CompanionAbility {
            id,
            name: "Empower",
            description: "Infuses an ally with arcane energy, increasing their attack power for 3 turns.",
            stamina_cost: 0,
            mana_cost: 20,
            cooldown_turns: 4,
            target: CompanionAbilityTarget::SingleAlly,
            effects: vec![CompanionAbilityEffect::BuffAttack {
                amount: 25,
                duration: 3,
            }],
        },
        CompanionAbilityId::FlameWave => CompanionAbility {
            id,
            name: "Flame Wave",
            description: "Sends a wave of fire across the battlefield, dealing fire damage to all enemies.",
            stamina_cost: 0,
            mana_cost: 35,
            cooldown_turns: 4,
            target: CompanionAbilityTarget::AllEnemies,
            effects: vec![CompanionAbilityEffect::Damage {
                multiplier: 120,
                damage_type: DamageType::Fire,
                defence_piercing_percent: 0,
            }],
        },
        CompanionAbilityId::FrostBarrier => CompanionAbility {
            id,
            name: "Frost Barrier",
            description: "Creates a shield of frost around all allies, increasing their defence by 20 for 2 turns.",
            stamina_cost: 0,
            mana_cost: 30,
            cooldown_turns: 5,
            target: CompanionAbilityTarget::AllAllies,
            effects: vec![CompanionAbilityEffect::BuffDefence {
                amount: 20,
                duration: 2,
            }],
        },
        CompanionAbilityId::MassHeal => CompanionAbility {
            id,
            name: "Mass Heal",
            description: "Channels a powerful healing wave, restoring 35 HP to all allies.",
            stamina_cost: 0,
            mana_cost: 45,
            cooldown_turns: 6,
            target: CompanionAbilityTarget::AllAllies,
            effects: vec![CompanionAbilityEffect::Heal { amount: 35 }],
        },
    }
}

/// Static definition of a companion. These are read-only data that describe
/// the companion's identity, role, base stats, and recruitment requirements.
/// They are not persisted per-save; only the mutable `CompanionState` is saved.
#[derive(Debug, Clone)]
pub struct CompanionDefinition {
    pub id: CompanionId,
    pub name: &'static str,
    pub role: CompanionRole,
    pub description: &'static str,
    pub base_hp: u32,
    pub base_attack: u32,
    pub base_defence: u32,
    pub base_mana: u32,
    pub base_stamina: u32,
    pub default_weapon: WeaponTier,
    pub default_armour: ArmourTier,
    /// Minimum player combat level required to recruit this companion.
    pub recruitment_level: u32,
    /// Gold cost to recruit this companion (0 for free).
    pub recruitment_cost: u32,
    /// Abilities this companion can use in combat.
    pub abilities: Vec<CompanionAbilityId>,
}

/// Returns the full roster of companion definitions.
pub fn all_companion_definitions() -> Vec<CompanionDefinition> {
    vec![
        CompanionDefinition {
            id: CompanionId::BranTheShield,
            name: "Bran the Shield",
            role: CompanionRole::FrontlineDefence,
            description: "A stalwart defender who draws enemy attacks and absorbs heavy blows.",
            base_hp: 150,
            base_attack: 40,
            base_defence: 60,
            base_mana: 10,
            base_stamina: 80,
            default_weapon: WeaponTier::Iron,
            default_armour: ArmourTier::Steel,
            recruitment_level: 1,
            recruitment_cost: 0,
            abilities: vec![
                CompanionAbilityId::ShieldBash,
                CompanionAbilityId::GuardAlly,
                CompanionAbilityId::IronWill,
            ],
        },
        CompanionDefinition {
            id: CompanionId::LyraSwiftbow,
            name: "Lyra Swiftbow",
            role: CompanionRole::RangedDamage,
            description: "A keen-eyed ranger who deals consistent damage from a safe distance.",
            base_hp: 90,
            base_attack: 70,
            base_defence: 25,
            base_mana: 15,
            base_stamina: 70,
            default_weapon: WeaponTier::Steel,
            default_armour: ArmourTier::Chain,
            recruitment_level: 5,
            recruitment_cost: 500,
            abilities: vec![
                CompanionAbilityId::PiercingShot,
                CompanionAbilityId::VolleyArrows,
                CompanionAbilityId::AimedShot,
            ],
        },
        CompanionDefinition {
            id: CompanionId::MarenStarweave,
            name: "Maren Starweave",
            role: CompanionRole::MagicSupport,
            description: "A gifted mage who channels arcane energy to heal allies and harm foes.",
            base_hp: 80,
            base_attack: 55,
            base_defence: 20,
            base_mana: 80,
            base_stamina: 50,
            default_weapon: WeaponTier::Bronze,
            default_armour: ArmourTier::Cloth,
            recruitment_level: 8,
            recruitment_cost: 1000,
            abilities: vec![
                CompanionAbilityId::HealingLight,
                CompanionAbilityId::ArcaneBlast,
                CompanionAbilityId::Empower,
                CompanionAbilityId::MassHeal,
            ],
        },
        CompanionDefinition {
            id: CompanionId::GarrickIronfist,
            name: "Garrick Ironfist",
            role: CompanionRole::FrontlineDefence,
            description: "A battle-hardened warrior with devastating melee attacks.",
            base_hp: 130,
            base_attack: 65,
            base_defence: 45,
            base_mana: 5,
            base_stamina: 90,
            default_weapon: WeaponTier::Steel,
            default_armour: ArmourTier::Iron,
            recruitment_level: 10,
            recruitment_cost: 1500,
            abilities: vec![CompanionAbilityId::ShieldBash, CompanionAbilityId::IronWill],
        },
        CompanionDefinition {
            id: CompanionId::SeraNightwhisper,
            name: "Sera Nightwhisper",
            role: CompanionRole::RangedDamage,
            description: "A shadow-touched archer whose arrows find every weakness.",
            base_hp: 85,
            base_attack: 80,
            base_defence: 20,
            base_mana: 25,
            base_stamina: 65,
            default_weapon: WeaponTier::Crystal,
            default_armour: ArmourTier::Cloth,
            recruitment_level: 15,
            recruitment_cost: 3000,
            abilities: vec![
                CompanionAbilityId::PiercingShot,
                CompanionAbilityId::AimedShot,
                CompanionAbilityId::VolleyArrows,
            ],
        },
        CompanionDefinition {
            id: CompanionId::TorvalEmbershield,
            name: "Torval Embershield",
            role: CompanionRole::MagicSupport,
            description: "A fire-wielding sage who bolsters allies and scorches enemies.",
            base_hp: 95,
            base_attack: 50,
            base_defence: 30,
            base_mana: 70,
            base_stamina: 55,
            default_weapon: WeaponTier::Gold,
            default_armour: ArmourTier::Cloth,
            recruitment_level: 12,
            recruitment_cost: 2000,
            abilities: vec![
                CompanionAbilityId::FlameWave,
                CompanionAbilityId::ArcaneBlast,
                CompanionAbilityId::FrostBarrier,
                CompanionAbilityId::Empower,
            ],
        },
    ]
}

/// Looks up a companion definition by ID.
pub fn get_definition(id: CompanionId) -> CompanionDefinition {
    all_companion_definitions()
        .into_iter()
        .find(|def| def.id == id)
        .expect("all CompanionId variants must have a definition")
}

/// Mutable per-save state for a single companion. This is what gets persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionState {
    pub id: CompanionId,
    #[serde(default)]
    pub condition: CompanionCondition,
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_stamina: u32,
    pub max_stamina: u32,
    pub current_mana: u32,
    pub max_mana: u32,
    pub attack: u32,
    pub defence: u32,
    pub level: u32,
    pub xp: u32,
    pub equipped_weapon: WeaponTier,
    pub equipped_armour: ArmourTier,
}

impl CompanionState {
    /// Create a new companion state from a definition, with full health.
    pub fn new_from_definition(def: &CompanionDefinition) -> Self {
        Self {
            id: def.id,
            condition: CompanionCondition::Available,
            current_hp: def.base_hp,
            max_hp: def.base_hp,
            current_stamina: def.base_stamina,
            max_stamina: def.base_stamina,
            current_mana: def.base_mana,
            max_mana: def.base_mana,
            attack: def.base_attack,
            defence: def.base_defence,
            level: 1,
            xp: 0,
            equipped_weapon: def.default_weapon,
            equipped_armour: def.default_armour,
        }
    }

    /// Clamp persisted values to sane limits.
    pub fn cap_values(&mut self) {
        if self.max_hp == 0 {
            self.max_hp = 100;
        }
        if self.current_hp > self.max_hp {
            self.current_hp = self.max_hp;
        }
        if self.max_stamina == 0 {
            self.max_stamina = 50;
        }
        if self.current_stamina > self.max_stamina {
            self.current_stamina = self.max_stamina;
        }
        if self.max_mana == 0 {
            self.max_mana = 10;
        }
        if self.current_mana > self.max_mana {
            self.current_mana = self.max_mana;
        }
        if self.level == 0 {
            self.level = 1;
        }
    }

    /// Fully restore HP, stamina, and mana.
    pub fn fully_restore(&mut self) {
        self.current_hp = self.max_hp;
        self.current_stamina = self.max_stamina;
        self.current_mana = self.max_mana;
    }

    /// Returns the companion's display name from the static definition.
    pub fn display_name(&self) -> &'static str {
        get_definition(self.id).name
    }

    /// Returns the companion's role from the static definition.
    pub fn role(&self) -> CompanionRole {
        get_definition(self.id).role
    }

    /// Whether this companion can currently be selected for an active party.
    pub fn can_join_party(&self) -> bool {
        self.condition.can_join_party()
    }
}

/// Errors that can occur during party operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyError {
    PartyFull {
        limit: usize,
    },
    AlreadyInParty {
        companion: CompanionId,
    },
    NotRecruited {
        companion: CompanionId,
    },
    NotInParty {
        companion: CompanionId,
    },
    Unavailable {
        companion: CompanionId,
        condition: CompanionCondition,
    },
    RecruitmentLevelNotMet {
        companion: CompanionId,
        required: u32,
        current: u32,
    },
    RecruitmentCostNotMet {
        companion: CompanionId,
        required: u32,
        current: u32,
    },
    AlreadyRecruited {
        companion: CompanionId,
    },
    DuplicateCompanion {
        companion: CompanionId,
    },
    #[allow(dead_code)]
    PlayerNotInParty,
}

impl fmt::Display for PartyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartyError::PartyFull { limit } => {
                write!(
                    f,
                    "Party is full (maximum {} members including you).",
                    limit
                )
            }
            PartyError::AlreadyInParty { companion } => {
                write!(
                    f,
                    "{} is already in the active party.",
                    get_definition(*companion).name
                )
            }
            PartyError::NotRecruited { companion } => {
                write!(
                    f,
                    "{} has not been recruited yet.",
                    get_definition(*companion).name
                )
            }
            PartyError::NotInParty { companion } => {
                write!(
                    f,
                    "{} is not currently in the active party.",
                    get_definition(*companion).name
                )
            }
            PartyError::Unavailable {
                companion,
                condition,
            } => {
                write!(
                    f,
                    "{} is currently {} and cannot join the party.",
                    get_definition(*companion).name,
                    condition.name()
                )
            }
            PartyError::RecruitmentLevelNotMet {
                companion,
                required,
                current,
            } => {
                write!(
                    f,
                    "You need combat level {} to recruit {}, but your level is {}.",
                    required,
                    get_definition(*companion).name,
                    current
                )
            }
            PartyError::RecruitmentCostNotMet {
                companion,
                required,
                current,
            } => {
                write!(
                    f,
                    "Recruiting {} costs {} gold, but you only have {}.",
                    get_definition(*companion).name,
                    required,
                    current
                )
            }
            PartyError::AlreadyRecruited { companion } => {
                write!(
                    f,
                    "{} has already been recruited.",
                    get_definition(*companion).name
                )
            }
            PartyError::DuplicateCompanion { companion } => {
                write!(
                    f,
                    "{} cannot appear more than once in the party.",
                    get_definition(*companion).name
                )
            }
            PartyError::PlayerNotInParty => {
                write!(
                    f,
                    "The player character must always count as a party member."
                )
            }
        }
    }
}

impl std::error::Error for PartyError {}

/// The party tracks the active expedition composition. The player character
/// always occupies one slot implicitly; this struct stores only companion slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    /// IDs of companions currently in the active party.
    #[serde(default)]
    members: Vec<CompanionId>,
    /// Maximum total party size including the player character.
    #[serde(default = "default_party_size_limit")]
    size_limit: usize,
}

fn default_party_size_limit() -> usize {
    DEFAULT_PARTY_SIZE_LIMIT
}

impl Party {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
            size_limit: DEFAULT_PARTY_SIZE_LIMIT,
        }
    }

    pub fn with_size_limit(size_limit: usize) -> Self {
        Self {
            members: Vec::new(),
            size_limit: size_limit.max(1),
        }
    }

    /// Number of companion slots currently filled.
    pub fn companion_count(&self) -> usize {
        self.members.len()
    }

    /// Maximum number of companions that can be in the party (size_limit - 1 for the player).
    pub fn max_companions(&self) -> usize {
        self.size_limit.saturating_sub(1)
    }

    /// Whether there is room for another companion.
    pub fn has_room(&self) -> bool {
        self.members.len() < self.max_companions()
    }

    /// The configured party size limit (including the player).
    pub fn size_limit(&self) -> usize {
        self.size_limit
    }

    /// Set a new party size limit (minimum 1 for the player).
    #[allow(dead_code)]
    pub fn set_size_limit(&mut self, limit: usize) {
        self.size_limit = limit.max(1);
    }

    /// Returns the list of companion IDs currently in the party.
    pub fn members(&self) -> &[CompanionId] {
        &self.members
    }

    /// Check whether a companion is currently in the active party.
    pub fn contains(&self, id: CompanionId) -> bool {
        self.members.contains(&id)
    }

    /// Add a companion to the active party after validating all prerequisites.
    pub fn add_member(
        &mut self,
        id: CompanionId,
        companions: &[CompanionState],
    ) -> Result<(), PartyError> {
        if !self.has_room() {
            return Err(PartyError::PartyFull {
                limit: self.size_limit,
            });
        }

        if self.members.contains(&id) {
            return Err(PartyError::AlreadyInParty { companion: id });
        }

        // Check for duplicates (defensive, since contains already handles it).
        let mut seen = HashSet::new();
        for &member in &self.members {
            if !seen.insert(member) {
                return Err(PartyError::DuplicateCompanion { companion: member });
            }
        }

        let companion = companions
            .iter()
            .find(|c| c.id == id)
            .ok_or(PartyError::NotRecruited { companion: id })?;

        if !companion.can_join_party() {
            return Err(PartyError::Unavailable {
                companion: id,
                condition: companion.condition,
            });
        }

        self.members.push(id);
        Ok(())
    }

    /// Remove a companion from the active party.
    pub fn remove_member(&mut self, id: CompanionId) -> Result<(), PartyError> {
        let position = self
            .members
            .iter()
            .position(|&m| m == id)
            .ok_or(PartyError::NotInParty { companion: id })?;
        self.members.remove(position);
        Ok(())
    }

    /// Clear all companions from the party.
    pub fn clear(&mut self) {
        self.members.clear();
    }
}

impl Default for Party {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level party management operations that coordinate between the party,
/// companion roster, and player state.
pub struct PartyManager;

impl PartyManager {
    /// Recruit a companion, adding them to the roster. Validates level and cost.
    /// Returns the gold cost if successful.
    pub fn recruit(
        companions: &mut Vec<CompanionState>,
        id: CompanionId,
        player_combat_level: u32,
        player_gold: u32,
    ) -> Result<u32, PartyError> {
        // Check if already recruited
        if companions.iter().any(|c| c.id == id) {
            return Err(PartyError::AlreadyRecruited { companion: id });
        }

        let definition = get_definition(id);

        // Check level requirement
        if player_combat_level < definition.recruitment_level {
            return Err(PartyError::RecruitmentLevelNotMet {
                companion: id,
                required: definition.recruitment_level,
                current: player_combat_level,
            });
        }

        // Check cost
        if player_gold < definition.recruitment_cost {
            return Err(PartyError::RecruitmentCostNotMet {
                companion: id,
                required: definition.recruitment_cost,
                current: player_gold,
            });
        }

        let state = CompanionState::new_from_definition(&definition);
        companions.push(state);

        Ok(definition.recruitment_cost)
    }

    /// Dismiss a companion from the roster entirely. They are removed from the
    /// party if present and can be re-recruited later.
    pub fn dismiss(
        companions: &mut Vec<CompanionState>,
        party: &mut Party,
        id: CompanionId,
    ) -> Result<(), PartyError> {
        let position = companions
            .iter()
            .position(|c| c.id == id)
            .ok_or(PartyError::NotRecruited { companion: id })?;

        // Remove from party first (ignore error if not in party)
        let _ = party.remove_member(id);

        companions.remove(position);
        Ok(())
    }

    /// Add a recruited companion to the active party.
    pub fn add_to_party(
        party: &mut Party,
        companions: &[CompanionState],
        id: CompanionId,
    ) -> Result<(), PartyError> {
        party.add_member(id, companions)
    }

    /// Remove a companion from the active party.
    pub fn remove_from_party(party: &mut Party, id: CompanionId) -> Result<(), PartyError> {
        party.remove_member(id)
    }
}

/// Returns initial companion states for a new game. All roster companions
/// start unrecruited; the player must recruit them explicitly. This function
/// returns an empty vec, but tests can use it as a baseline.
#[allow(dead_code)]
pub fn initial_companion_roster() -> Vec<CompanionState> {
    Vec::new()
}

/// For testing: create a roster with all companions already recruited and available.
#[cfg(test)]
pub fn test_full_roster() -> Vec<CompanionState> {
    all_companion_definitions()
        .iter()
        .map(CompanionState::new_from_definition)
        .collect()
}

/// For testing: create a roster with specific companions.
#[cfg(test)]
#[allow(dead_code)]
pub fn test_roster_with(ids: &[CompanionId]) -> Vec<CompanionState> {
    ids.iter()
        .map(|&id| CompanionState::new_from_definition(&get_definition(id)))
        .collect()
}

// ---------------------------------------------------------------------------
// Roster action mapping (pure, testable helpers for the companion detail menu)
// ---------------------------------------------------------------------------

/// Typed actions that the companion detail menu can perform on a recruited
/// companion. Using an enum instead of numeric indexes prevents the bug where
/// conditional menu entries shift the positions of later actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterAction {
    Details,
    AddToParty,
    RemoveFromParty,
    Dismiss,
    Back,
}

impl RosterAction {
    pub fn label(&self) -> &'static str {
        match self {
            RosterAction::Details => "Details",
            RosterAction::AddToParty => "Add to Party",
            RosterAction::RemoveFromParty => "Remove from Party",
            RosterAction::Dismiss => "Dismiss",
            RosterAction::Back => "Back",
        }
    }
}

/// Builds the ordered list of actions available for a recruited companion.
/// The result is guaranteed to contain `Details` first and `Back` last, with
/// at most one of `AddToParty` or `RemoveFromParty` in between, and `Dismiss`
/// before `Back`.
pub fn build_roster_actions(in_party: bool, can_add_to_party: bool) -> Vec<RosterAction> {
    let mut actions = vec![RosterAction::Details];
    if in_party {
        actions.push(RosterAction::RemoveFromParty);
    } else if can_add_to_party {
        actions.push(RosterAction::AddToParty);
    }
    actions.push(RosterAction::Dismiss);
    actions.push(RosterAction::Back);
    actions
}

/// Resolves a user's menu selection index to a typed `RosterAction`.
/// Returns `None` if the index is out of range (treated as Back/no-op).
pub fn resolve_roster_action(actions: &[RosterAction], choice: usize) -> RosterAction {
    actions.get(choice).copied().unwrap_or(RosterAction::Back)
}

// ---------------------------------------------------------------------------
// Party management menu action mapping
// ---------------------------------------------------------------------------

/// Typed actions for the party management menu. `Member(usize)` identifies a
/// party member by index; `AddCompanion` opens the add-companion submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyMenuAction {
    /// Select a party member (index into the current party members list).
    Member(usize),
    /// Open the add-companion submenu.
    AddCompanion,
    /// Return to the previous menu.
    Back,
}

/// Builds the party management menu actions. When the party is empty, the
/// informational "No companions in party" text is NOT included as an
/// actionable entry.
pub fn build_party_menu_actions(member_count: usize) -> Vec<PartyMenuAction> {
    let mut actions = Vec::new();
    for i in 0..member_count {
        actions.push(PartyMenuAction::Member(i));
    }
    actions.push(PartyMenuAction::AddCompanion);
    actions.push(PartyMenuAction::Back);
    actions
}

/// Resolves a user's menu selection index to a typed `PartyMenuAction`.
pub fn resolve_party_menu_action(actions: &[PartyMenuAction], choice: usize) -> PartyMenuAction {
    actions
        .get(choice)
        .cloned()
        .unwrap_or(PartyMenuAction::Back)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_companions() -> Vec<CompanionState> {
        test_full_roster()
    }

    // --- Definition tests ---

    #[test]
    fn all_companion_ids_have_definitions() {
        let definitions = all_companion_definitions();
        for id in CompanionId::ALL {
            assert!(
                definitions.iter().any(|d| d.id == id),
                "Missing definition for {:?}",
                id
            );
        }
    }

    #[test]
    fn definitions_cover_all_roles() {
        let definitions = all_companion_definitions();
        let roles: HashSet<CompanionRole> = definitions.iter().map(|d| d.role).collect();
        assert!(roles.contains(&CompanionRole::FrontlineDefence));
        assert!(roles.contains(&CompanionRole::RangedDamage));
        assert!(roles.contains(&CompanionRole::MagicSupport));
    }

    // --- Party size and capacity tests ---

    #[test]
    fn new_party_has_default_size_limit() {
        let party = Party::new();
        assert_eq!(party.size_limit(), DEFAULT_PARTY_SIZE_LIMIT);
        assert_eq!(party.max_companions(), DEFAULT_PARTY_SIZE_LIMIT - 1);
        assert!(party.has_room());
        assert_eq!(party.companion_count(), 0);
    }

    #[test]
    fn party_full_rejects_new_members() {
        let companions = default_companions();
        let mut party = Party::with_size_limit(3); // 1 player + 2 companions

        assert!(party
            .add_member(CompanionId::BranTheShield, &companions)
            .is_ok());
        assert!(party
            .add_member(CompanionId::LyraSwiftbow, &companions)
            .is_ok());
        assert!(!party.has_room());

        let result = party.add_member(CompanionId::MarenStarweave, &companions);
        assert!(matches!(result, Err(PartyError::PartyFull { .. })));
    }

    #[test]
    fn party_size_limit_minimum_is_one() {
        let party = Party::with_size_limit(0);
        assert_eq!(party.size_limit(), 1);
        assert_eq!(party.max_companions(), 0);
        assert!(!party.has_room());
    }

    // --- Recruitment tests ---

    #[test]
    fn recruit_adds_companion_to_roster() {
        let mut companions = Vec::new();
        let cost = PartyManager::recruit(&mut companions, CompanionId::BranTheShield, 5, 1000)
            .expect("recruitment should succeed");
        assert_eq!(cost, 0); // Bran is free
        assert_eq!(companions.len(), 1);
        assert_eq!(companions[0].id, CompanionId::BranTheShield);
        assert_eq!(companions[0].condition, CompanionCondition::Available);
    }

    #[test]
    fn recruit_fails_if_already_recruited() {
        let mut companions = Vec::new();
        PartyManager::recruit(&mut companions, CompanionId::BranTheShield, 5, 1000).unwrap();
        let result = PartyManager::recruit(&mut companions, CompanionId::BranTheShield, 5, 1000);
        assert!(matches!(result, Err(PartyError::AlreadyRecruited { .. })));
    }

    #[test]
    fn recruit_fails_if_level_too_low() {
        let mut companions = Vec::new();
        let result =
            PartyManager::recruit(&mut companions, CompanionId::SeraNightwhisper, 5, 10000);
        assert!(matches!(
            result,
            Err(PartyError::RecruitmentLevelNotMet { .. })
        ));
    }

    #[test]
    fn recruit_fails_if_not_enough_gold() {
        let mut companions = Vec::new();
        let result = PartyManager::recruit(&mut companions, CompanionId::LyraSwiftbow, 10, 100);
        assert!(matches!(
            result,
            Err(PartyError::RecruitmentCostNotMet { .. })
        ));
    }

    #[test]
    fn recruit_costs_gold_for_paid_companions() {
        let mut companions = Vec::new();
        let cost = PartyManager::recruit(&mut companions, CompanionId::LyraSwiftbow, 10, 1000)
            .expect("recruitment should succeed");
        assert_eq!(cost, 500);
    }

    // --- Dismissal tests ---

    #[test]
    fn dismiss_removes_companion_from_roster_and_party() {
        let mut companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();

        assert!(
            PartyManager::dismiss(&mut companions, &mut party, CompanionId::BranTheShield).is_ok()
        );
        assert!(!party.contains(CompanionId::BranTheShield));
        assert!(companions
            .iter()
            .all(|c| c.id != CompanionId::BranTheShield));
    }

    #[test]
    fn dismiss_fails_if_not_recruited() {
        let mut companions = Vec::new();
        let mut party = Party::new();
        let result = PartyManager::dismiss(&mut companions, &mut party, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::NotRecruited { .. })));
    }

    // --- Add/remove from party tests ---

    #[test]
    fn add_to_party_succeeds_for_available_companion() {
        let companions = default_companions();
        let mut party = Party::new();
        assert!(
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield).is_ok()
        );
        assert!(party.contains(CompanionId::BranTheShield));
    }

    #[test]
    fn add_to_party_fails_for_unavailable_companion() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::KnockedOut;
        let mut party = Party::new();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_recovering_companion() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::Recovering;
        let mut party = Party::new();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_unavailable_condition() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::Unavailable;
        let mut party = Party::new();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_already_in_party() {
        let companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::AlreadyInParty { .. })));
    }

    #[test]
    fn add_to_party_fails_if_not_recruited() {
        let companions = Vec::new();
        let mut party = Party::new();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::NotRecruited { .. })));
    }

    #[test]
    fn remove_from_party_succeeds() {
        let companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        assert!(PartyManager::remove_from_party(&mut party, CompanionId::BranTheShield).is_ok());
        assert!(!party.contains(CompanionId::BranTheShield));
    }

    #[test]
    fn remove_from_party_fails_if_not_in_party() {
        let mut party = Party::new();
        let result = PartyManager::remove_from_party(&mut party, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::NotInParty { .. })));
    }

    // --- Duplicate prevention ---

    #[test]
    fn cannot_add_same_companion_twice() {
        let companions = default_companions();
        let mut party = Party::with_size_limit(10);
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        let result = party.add_member(CompanionId::BranTheShield, &companions);
        assert!(matches!(result, Err(PartyError::AlreadyInParty { .. })));
    }

    // --- Serialization roundtrip ---

    #[test]
    fn companion_state_serializes_and_deserializes() {
        let def = get_definition(CompanionId::LyraSwiftbow);
        let state = CompanionState::new_from_definition(&def);
        let json = serde_json::to_string(&state).unwrap();
        let loaded: CompanionState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, state.id);
        assert_eq!(loaded.condition, state.condition);
        assert_eq!(loaded.current_hp, state.max_hp);
        assert_eq!(loaded.equipped_weapon, def.default_weapon);
    }

    #[test]
    fn party_serializes_and_deserializes() {
        let companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        party
            .add_member(CompanionId::LyraSwiftbow, &companions)
            .unwrap();

        let json = serde_json::to_string(&party).unwrap();
        let loaded: Party = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.companion_count(), 2);
        assert!(loaded.contains(CompanionId::BranTheShield));
        assert!(loaded.contains(CompanionId::LyraSwiftbow));
        assert_eq!(loaded.size_limit(), DEFAULT_PARTY_SIZE_LIMIT);
    }

    #[test]
    fn old_party_json_deserializes_with_defaults() {
        // Simulate a save that has no party field at all - serde(default) handles it.
        let empty_json = "{}";
        let loaded: Party = serde_json::from_str(empty_json).unwrap();
        assert_eq!(loaded.companion_count(), 0);
        assert_eq!(loaded.size_limit(), DEFAULT_PARTY_SIZE_LIMIT);
    }

    #[test]
    fn companion_state_with_missing_fields_gets_defaults() {
        // Simulate a companion state saved without condition field.
        let json = r#"{
            "id": "bran_the_shield",
            "current_hp": 150,
            "max_hp": 150,
            "current_stamina": 80,
            "max_stamina": 80,
            "current_mana": 10,
            "max_mana": 10,
            "attack": 40,
            "defence": 60,
            "level": 1,
            "xp": 0,
            "equipped_weapon": "Iron",
            "equipped_armour": "Steel"
        }"#;
        let loaded: CompanionState = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.condition, CompanionCondition::Active);
    }

    // --- Condition tests ---

    #[test]
    fn condition_can_join_party_is_correct() {
        assert!(CompanionCondition::Active.can_join_party());
        assert!(CompanionCondition::Available.can_join_party());
        assert!(!CompanionCondition::KnockedOut.can_join_party());
        assert!(!CompanionCondition::Recovering.can_join_party());
        assert!(!CompanionCondition::Unavailable.can_join_party());
    }

    // --- Integration test: full recruitment flow ---

    #[test]
    fn full_recruitment_and_party_assembly_flow() {
        let mut companions = Vec::new();
        let mut party = Party::new();
        let mut player_gold: u32 = 5000;
        let player_level: u32 = 20;

        // Recruit Bran (free, level 1)
        let cost = PartyManager::recruit(
            &mut companions,
            CompanionId::BranTheShield,
            player_level,
            player_gold,
        )
        .unwrap();
        player_gold -= cost;
        assert_eq!(player_gold, 5000);

        // Recruit Lyra (500 gold, level 5)
        let cost = PartyManager::recruit(
            &mut companions,
            CompanionId::LyraSwiftbow,
            player_level,
            player_gold,
        )
        .unwrap();
        player_gold -= cost;
        assert_eq!(player_gold, 4500);

        // Recruit Maren (1000 gold, level 8)
        let cost = PartyManager::recruit(
            &mut companions,
            CompanionId::MarenStarweave,
            player_level,
            player_gold,
        )
        .unwrap();
        player_gold -= cost;
        assert_eq!(player_gold, 3500);

        // Add all three to party
        assert!(
            PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield).is_ok()
        );
        assert!(
            PartyManager::add_to_party(&mut party, &companions, CompanionId::LyraSwiftbow).is_ok()
        );
        assert!(
            PartyManager::add_to_party(&mut party, &companions, CompanionId::MarenStarweave)
                .is_ok()
        );

        assert_eq!(party.companion_count(), 3);
        assert_eq!(party.members().len(), 3);

        // Party is full (default limit 4 = 1 player + 3 companions)
        assert!(!party.has_room());

        // Cannot add another
        PartyManager::recruit(
            &mut companions,
            CompanionId::GarrickIronfist,
            player_level,
            player_gold,
        )
        .unwrap();
        let result =
            PartyManager::add_to_party(&mut party, &companions, CompanionId::GarrickIronfist);
        assert!(matches!(result, Err(PartyError::PartyFull { .. })));
    }

    #[test]
    fn dismissed_companion_frees_party_slot() {
        let mut companions = default_companions();
        let mut party = Party::with_size_limit(3); // 1 player + 2 companions

        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        party
            .add_member(CompanionId::LyraSwiftbow, &companions)
            .unwrap();
        assert!(!party.has_room());

        // Dismiss Bran (also removes from party)
        PartyManager::dismiss(&mut companions, &mut party, CompanionId::BranTheShield).unwrap();
        assert!(party.has_room());

        // Can now add another
        assert!(party
            .add_member(CompanionId::MarenStarweave, &companions)
            .is_ok());
    }

    #[test]
    fn party_clear_removes_all_members() {
        let companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        party
            .add_member(CompanionId::LyraSwiftbow, &companions)
            .unwrap();

        party.clear();
        assert_eq!(party.companion_count(), 0);
    }

    // --- Companion state tests ---

    #[test]
    fn companion_fully_restore_sets_all_to_max() {
        let def = get_definition(CompanionId::BranTheShield);
        let mut state = CompanionState::new_from_definition(&def);
        state.current_hp = 10;
        state.current_stamina = 5;
        state.current_mana = 0;

        state.fully_restore();

        assert_eq!(state.current_hp, state.max_hp);
        assert_eq!(state.current_stamina, state.max_stamina);
        assert_eq!(state.current_mana, state.max_mana);
    }

    #[test]
    fn companion_cap_values_clamps_bad_data() {
        let def = get_definition(CompanionId::LyraSwiftbow);
        let mut state = CompanionState::new_from_definition(&def);
        state.current_hp = 9999;
        state.max_hp = 0;
        state.level = 0;

        state.cap_values();

        assert_eq!(state.max_hp, 100);
        assert_eq!(state.current_hp, 100);
        assert_eq!(state.level, 1);
    }

    // --- Ability definition tests ---

    #[test]
    fn all_companion_ability_ids_have_definitions() {
        for id in CompanionAbilityId::ALL {
            let ability = get_companion_ability(id);
            assert_eq!(ability.id, id);
            assert!(!ability.name.is_empty());
            assert!(!ability.description.is_empty());
            assert!(
                !ability.effects.is_empty(),
                "Ability {} has no effects",
                ability.name
            );
        }
    }

    #[test]
    fn shield_bash_deals_damage_and_applies_stun() {
        let ability = get_companion_ability(CompanionAbilityId::ShieldBash);

        assert!(ability.effects.contains(&CompanionAbilityEffect::Damage {
            multiplier: 75,
            damage_type: DamageType::Physical,
            defence_piercing_percent: 0,
        }));
        assert!(ability
            .effects
            .contains(&CompanionAbilityEffect::ApplyStatus {
                kind: StatusEffectKind::Stun,
                duration: 1,
                potency: 1,
            }));
    }

    #[test]
    fn guard_ally_targets_an_ally_and_redirects_damage() {
        let ability = get_companion_ability(CompanionAbilityId::GuardAlly);

        assert_eq!(ability.target, CompanionAbilityTarget::SingleAlly);
        assert!(ability
            .effects
            .contains(&CompanionAbilityEffect::RedirectDamage { duration: 2 }));
    }

    #[test]
    fn iron_will_heals_and_increases_defence() {
        let ability = get_companion_ability(CompanionAbilityId::IronWill);

        assert_eq!(ability.target, CompanionAbilityTarget::SelfTarget);
        assert!(ability
            .effects
            .contains(&CompanionAbilityEffect::Heal { amount: 40 }));
        assert!(ability
            .effects
            .contains(&CompanionAbilityEffect::BuffDefence {
                amount: 20,
                duration: 2,
            }));
    }

    #[test]
    fn piercing_shot_ignores_some_target_defence() {
        let ability = get_companion_ability(CompanionAbilityId::PiercingShot);
        assert!(ability.effects.iter().any(|effect| matches!(
            effect,
            CompanionAbilityEffect::Damage {
                defence_piercing_percent: 1..=100,
                ..
            }
        )));
    }

    #[test]
    fn frost_barrier_increases_defence_for_all_allies() {
        let ability = get_companion_ability(CompanionAbilityId::FrostBarrier);

        assert_eq!(ability.target, CompanionAbilityTarget::AllAllies);
        assert!(ability
            .effects
            .contains(&CompanionAbilityEffect::BuffDefence {
                amount: 20,
                duration: 2,
            }));
    }

    #[test]
    fn every_companion_has_at_least_one_ability() {
        for def in all_companion_definitions() {
            assert!(
                !def.abilities.is_empty(),
                "Companion {} has no abilities defined",
                def.name
            );
        }
    }

    #[test]
    fn companion_abilities_reference_valid_ids() {
        for def in all_companion_definitions() {
            for &ability_id in &def.abilities {
                // This would panic if the ID is not handled in get_companion_ability.
                let ability = get_companion_ability(ability_id);
                assert_eq!(ability.id, ability_id);
            }
        }
    }

    #[test]
    fn frontline_companions_have_no_mana_cost_abilities() {
        for def in all_companion_definitions() {
            if def.role == CompanionRole::FrontlineDefence {
                for &ability_id in &def.abilities {
                    let ability = get_companion_ability(ability_id);
                    assert_eq!(
                        ability.mana_cost, 0,
                        "Frontline companion {} has ability {} with mana cost {}",
                        def.name, ability.name, ability.mana_cost
                    );
                }
            }
        }
    }

    #[test]
    fn magic_support_companions_have_mana_based_abilities() {
        let mut found_mana_ability = false;
        for def in all_companion_definitions() {
            if def.role == CompanionRole::MagicSupport {
                for &ability_id in &def.abilities {
                    let ability = get_companion_ability(ability_id);
                    if ability.mana_cost > 0 {
                        found_mana_ability = true;
                    }
                }
            }
        }
        assert!(
            found_mana_ability,
            "Magic/Support companions should have at least one mana-costing ability"
        );
    }

    #[test]
    fn ability_cooldowns_are_nonzero() {
        for id in CompanionAbilityId::ALL {
            let ability = get_companion_ability(id);
            assert!(
                ability.cooldown_turns > 0,
                "Ability {} has zero cooldown",
                ability.name
            );
        }
    }

    #[test]
    fn ability_targets_are_consistent_with_role() {
        for def in all_companion_definitions() {
            for &ability_id in &def.abilities {
                let ability = get_companion_ability(ability_id);
                match def.role {
                    CompanionRole::FrontlineDefence => {
                        // Frontline abilities can target enemies, themselves, or an ally to guard.
                        assert!(
                            matches!(
                                ability.target,
                                CompanionAbilityTarget::SingleEnemy
                                    | CompanionAbilityTarget::AllEnemies
                                    | CompanionAbilityTarget::SelfTarget
                                    | CompanionAbilityTarget::SingleAlly
                            ),
                            "Frontline ability {} has unexpected target {:?}",
                            ability.name,
                            ability.target
                        );
                    }
                    CompanionRole::RangedDamage => {
                        // Ranged abilities should target enemies or self.
                        assert!(
                            matches!(
                                ability.target,
                                CompanionAbilityTarget::SingleEnemy
                                    | CompanionAbilityTarget::AllEnemies
                                    | CompanionAbilityTarget::SelfTarget
                            ),
                            "Ranged ability {} has unexpected target {:?}",
                            ability.name,
                            ability.target
                        );
                    }
                    CompanionRole::MagicSupport => {
                        // Support abilities can target anything.
                        // Just verify the target is valid (covered by enum).
                    }
                }
            }
        }
    }

    // --- Roster action mapping tests ---

    #[test]
    fn roster_actions_for_companion_in_party() {
        let actions = build_roster_actions(true, false);
        assert_eq!(
            actions,
            vec![
                RosterAction::Details,
                RosterAction::RemoveFromParty,
                RosterAction::Dismiss,
                RosterAction::Back,
            ]
        );
    }

    #[test]
    fn roster_actions_for_available_companion_not_in_party() {
        let actions = build_roster_actions(false, true);
        assert_eq!(
            actions,
            vec![
                RosterAction::Details,
                RosterAction::AddToParty,
                RosterAction::Dismiss,
                RosterAction::Back,
            ]
        );
    }

    #[test]
    fn roster_actions_for_unavailable_companion() {
        let actions = build_roster_actions(false, false);
        assert_eq!(
            actions,
            vec![
                RosterAction::Details,
                RosterAction::Dismiss,
                RosterAction::Back,
            ]
        );
    }

    #[test]
    fn roster_actions_when_party_is_full() {
        // Companion is available but party has no room.
        let actions = build_roster_actions(false, false);
        assert!(!actions.contains(&RosterAction::AddToParty));
        assert!(!actions.contains(&RosterAction::RemoveFromParty));
    }

    #[test]
    fn resolve_roster_action_returns_correct_action() {
        let actions = build_roster_actions(true, false);
        assert_eq!(resolve_roster_action(&actions, 0), RosterAction::Details);
        assert_eq!(
            resolve_roster_action(&actions, 1),
            RosterAction::RemoveFromParty
        );
        assert_eq!(resolve_roster_action(&actions, 2), RosterAction::Dismiss);
        assert_eq!(resolve_roster_action(&actions, 3), RosterAction::Back);
    }

    #[test]
    fn resolve_roster_action_out_of_range_returns_back() {
        let actions = build_roster_actions(false, false);
        assert_eq!(resolve_roster_action(&actions, 99), RosterAction::Back);
    }

    #[test]
    fn back_action_is_always_last() {
        for in_party in [true, false] {
            for can_add in [true, false] {
                let actions = build_roster_actions(in_party, can_add);
                assert_eq!(*actions.last().unwrap(), RosterAction::Back);
            }
        }
    }

    #[test]
    fn dismiss_is_always_second_to_last() {
        for in_party in [true, false] {
            for can_add in [true, false] {
                let actions = build_roster_actions(in_party, can_add);
                assert_eq!(actions[actions.len() - 2], RosterAction::Dismiss);
            }
        }
    }

    // --- Party menu action mapping tests ---

    #[test]
    fn party_menu_actions_empty_party() {
        let actions = build_party_menu_actions(0);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], PartyMenuAction::AddCompanion);
        assert_eq!(actions[1], PartyMenuAction::Back);
    }

    #[test]
    fn party_menu_actions_with_members() {
        let actions = build_party_menu_actions(2);
        assert_eq!(actions.len(), 4);
        assert_eq!(actions[0], PartyMenuAction::Member(0));
        assert_eq!(actions[1], PartyMenuAction::Member(1));
        assert_eq!(actions[2], PartyMenuAction::AddCompanion);
        assert_eq!(actions[3], PartyMenuAction::Back);
    }

    #[test]
    fn resolve_party_menu_action_returns_correct_action() {
        let actions = build_party_menu_actions(2);
        assert_eq!(
            resolve_party_menu_action(&actions, 0),
            PartyMenuAction::Member(0)
        );
        assert_eq!(
            resolve_party_menu_action(&actions, 1),
            PartyMenuAction::Member(1)
        );
        assert_eq!(
            resolve_party_menu_action(&actions, 2),
            PartyMenuAction::AddCompanion
        );
        assert_eq!(
            resolve_party_menu_action(&actions, 3),
            PartyMenuAction::Back
        );
    }

    #[test]
    fn resolve_party_menu_action_out_of_range_returns_back() {
        let actions = build_party_menu_actions(0);
        assert_eq!(
            resolve_party_menu_action(&actions, 99),
            PartyMenuAction::Back
        );
    }

    #[test]
    fn party_menu_back_is_always_last() {
        for count in 0..5 {
            let actions = build_party_menu_actions(count);
            assert_eq!(*actions.last().unwrap(), PartyMenuAction::Back);
        }
    }

    #[test]
    fn party_menu_add_companion_is_always_second_to_last() {
        for count in 0..5 {
            let actions = build_party_menu_actions(count);
            assert_eq!(actions[actions.len() - 2], PartyMenuAction::AddCompanion);
        }
    }
}
