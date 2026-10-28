use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use crate::data::{ArmourTier, WeaponTier};

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
        matches!(self, CompanionCondition::Active | CompanionCondition::Available)
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
                write!(f, "Party is full (maximum {} members including you).", limit)
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
                write!(f, "The player character must always count as a party member.")
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

        assert!(PartyManager::dismiss(&mut companions, &mut party, CompanionId::BranTheShield).is_ok());
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
        assert!(PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield).is_ok());
        assert!(party.contains(CompanionId::BranTheShield));
    }

    #[test]
    fn add_to_party_fails_for_unavailable_companion() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::KnockedOut;
        let mut party = Party::new();
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_recovering_companion() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::Recovering;
        let mut party = Party::new();
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_unavailable_condition() {
        let mut companions = default_companions();
        companions[0].condition = CompanionCondition::Unavailable;
        let mut party = Party::new();
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::Unavailable { .. })));
    }

    #[test]
    fn add_to_party_fails_for_already_in_party() {
        let companions = default_companions();
        let mut party = Party::new();
        party
            .add_member(CompanionId::BranTheShield, &companions)
            .unwrap();
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
        assert!(matches!(result, Err(PartyError::AlreadyInParty { .. })));
    }

    #[test]
    fn add_to_party_fails_if_not_recruited() {
        let companions = Vec::new();
        let mut party = Party::new();
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield);
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
        assert!(PartyManager::add_to_party(&mut party, &companions, CompanionId::BranTheShield).is_ok());
        assert!(PartyManager::add_to_party(&mut party, &companions, CompanionId::LyraSwiftbow).is_ok());
        assert!(PartyManager::add_to_party(&mut party, &companions, CompanionId::MarenStarweave).is_ok());

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
        let result = PartyManager::add_to_party(&mut party, &companions, CompanionId::GarrickIronfist);
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
}
