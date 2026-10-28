use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponTier {
    BareHands,
    Wooden,
    Stone,
    Bronze,
    Iron,
    Steel,
    Gold,
    Crystal,
    Sacred,
    Runic,
    Infernal,
    Omega,
    Chaotic,
    Mythical,
    Alydril,
    Elysian,
    Arcane,
    Spectral,
}

impl WeaponTier {
    pub fn max_damage(&self) -> u32 {
        match self {
            WeaponTier::BareHands => 50,
            WeaponTier::Wooden => 100,
            WeaponTier::Stone => 125,
            WeaponTier::Bronze => 150,
            WeaponTier::Iron => 175,
            WeaponTier::Steel => 200,
            WeaponTier::Gold => 225,
            WeaponTier::Crystal => 250,
            WeaponTier::Sacred => 275,
            WeaponTier::Runic => 300,
            WeaponTier::Infernal => 300,
            WeaponTier::Omega => 300,
            WeaponTier::Chaotic => 300,
            WeaponTier::Mythical => 450,
            WeaponTier::Alydril => 600,
            WeaponTier::Elysian => 750,
            WeaponTier::Arcane => 850,
            WeaponTier::Spectral => 1000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            WeaponTier::BareHands => "Your bare hands",
            WeaponTier::Wooden => "Wooden Sword",
            WeaponTier::Stone => "Stone Sword",
            WeaponTier::Bronze => "Bronze Sword",
            WeaponTier::Iron => "Iron Sword",
            WeaponTier::Steel => "Steel Sword",
            WeaponTier::Gold => "Gold Sword",
            WeaponTier::Crystal => "Crystal Sword",
            WeaponTier::Sacred => "Sacred Blade",
            WeaponTier::Runic => "Runic Sword",
            WeaponTier::Infernal => "Infernal Rapier",
            WeaponTier::Omega => "Omega Maul",
            WeaponTier::Chaotic => "Chaotic Spear",
            WeaponTier::Mythical => "Mythical Godsword",
            WeaponTier::Alydril => "Alydril Godsword",
            WeaponTier::Elysian => "Elysian Blade",
            WeaponTier::Arcane => "Arcane Blade",
            WeaponTier::Spectral => "Spectral Blade",
        }
    }

    pub fn buy_price(&self) -> u32 {
        match self {
            WeaponTier::BareHands => 0,
            WeaponTier::Wooden => 100,
            WeaponTier::Stone => 250,
            WeaponTier::Bronze => 500,
            WeaponTier::Iron => 1000,
            WeaponTier::Steel => 2500,
            WeaponTier::Gold => 5000,
            WeaponTier::Crystal => 10000,
            WeaponTier::Sacred => 25000,
            WeaponTier::Runic => 50000,
            WeaponTier::Infernal => 50000,
            WeaponTier::Omega => 50000,
            WeaponTier::Chaotic => 50000,
            WeaponTier::Mythical => 0, // Legend's store only
            WeaponTier::Alydril => 0,  // Legend's store only
            WeaponTier::Elysian => 3000000,
            WeaponTier::Arcane => 400000,
            WeaponTier::Spectral => 5000000,
        }
    }

    pub fn sell_price(&self) -> u32 {
        match self {
            WeaponTier::BareHands => 0,
            WeaponTier::Wooden => 50,
            WeaponTier::Stone => 125,
            WeaponTier::Bronze => 250,
            WeaponTier::Iron => 500,
            WeaponTier::Steel => 1250,
            WeaponTier::Gold => 2500,
            WeaponTier::Crystal => 5000,
            WeaponTier::Sacred => 12500,
            WeaponTier::Runic => 25000,
            WeaponTier::Infernal => 25000,
            WeaponTier::Omega => 25000,
            WeaponTier::Chaotic => 25000,
            WeaponTier::Mythical => 750000,
            WeaponTier::Alydril => 1500000,
            WeaponTier::Elysian => 1500000,
            WeaponTier::Arcane => 200000,
            WeaponTier::Spectral => 2500000,
        }
    }

    pub fn rarity(&self) -> &'static str {
        match self {
            WeaponTier::BareHands | WeaponTier::Wooden => "Common",
            WeaponTier::Stone | WeaponTier::Bronze => "Uncommon",
            WeaponTier::Iron | WeaponTier::Steel => "Rare",
            WeaponTier::Gold | WeaponTier::Crystal => "Epic",
            _ => "Legendary",
        }
    }

    pub fn level_required(&self) -> u32 {
        match self {
            WeaponTier::BareHands => 1,
            WeaponTier::Wooden => 1,
            WeaponTier::Stone => 1,
            WeaponTier::Bronze => 5,
            WeaponTier::Iron => 5,
            WeaponTier::Steel => 10,
            WeaponTier::Gold => 10,
            WeaponTier::Crystal => 15,
            WeaponTier::Sacred => 15,
            WeaponTier::Runic => 20,
            WeaponTier::Infernal => 20,
            WeaponTier::Omega => 20,
            WeaponTier::Chaotic => 20,
            WeaponTier::Mythical => 30,
            WeaponTier::Alydril => 35,
            WeaponTier::Elysian => 37,
            WeaponTier::Arcane => 37,
            WeaponTier::Spectral => 37,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmourTier {
    None,
    Cloth,
    Chain,
    Bronze,
    Iron,
    Steel,
    Gold,
    Crystal,
    Sacred,
    Runic,
    Infernal,
    Omega,
    Chaotic,
    Mythical,
    Alydril,
}

impl ArmourTier {
    pub fn max_dr(&self) -> u32 {
        match self {
            ArmourTier::None => 0,
            ArmourTier::Cloth => 10,
            ArmourTier::Chain => 20,
            ArmourTier::Bronze => 40,
            ArmourTier::Iron => 65,
            ArmourTier::Steel => 100,
            ArmourTier::Gold => 120,
            ArmourTier::Crystal => 145,
            ArmourTier::Sacred => 175,
            ArmourTier::Runic => 200,
            ArmourTier::Infernal => 215,
            ArmourTier::Omega => 230,
            ArmourTier::Chaotic => 250,
            ArmourTier::Mythical => 275,
            ArmourTier::Alydril => 300,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ArmourTier::None => "No Armour",
            ArmourTier::Cloth => "Cloth Armour",
            ArmourTier::Chain => "Chain Armour",
            ArmourTier::Bronze => "Bronze Armour",
            ArmourTier::Iron => "Iron Armour",
            ArmourTier::Steel => "Steel Armour",
            ArmourTier::Gold => "Gold Armour",
            ArmourTier::Crystal => "Crystal Armour",
            ArmourTier::Sacred => "Sacred Armour",
            ArmourTier::Runic => "Runic Armour",
            ArmourTier::Infernal => "Infernal Armour",
            ArmourTier::Omega => "Omega Armour",
            ArmourTier::Chaotic => "Chaotic Armour",
            ArmourTier::Mythical => "Mythical Armour",
            ArmourTier::Alydril => "Alydril Armour",
        }
    }

    pub fn buy_price(&self) -> u32 {
        match self {
            ArmourTier::None => 0,
            ArmourTier::Cloth => 150,
            ArmourTier::Chain => 300,
            ArmourTier::Bronze => 750,
            ArmourTier::Iron => 1500,
            ArmourTier::Steel => 3000,
            ArmourTier::Gold => 5000,
            ArmourTier::Crystal => 10000,
            ArmourTier::Sacred => 15000,
            ArmourTier::Runic => 25000,
            ArmourTier::Infernal => 40000,
            ArmourTier::Omega => 75000,
            ArmourTier::Chaotic => 100000,
            ArmourTier::Mythical => 0, // Legend's store only
            ArmourTier::Alydril => 0,  // Legend's store only
        }
    }

    pub fn sell_price(&self) -> u32 {
        match self {
            ArmourTier::None => 0,
            ArmourTier::Cloth => 75,
            ArmourTier::Chain => 150,
            ArmourTier::Bronze => 400,
            ArmourTier::Iron => 750,
            ArmourTier::Steel => 1500,
            ArmourTier::Gold => 2500,
            ArmourTier::Crystal => 5000,
            ArmourTier::Sacred => 7500,
            ArmourTier::Runic => 12500,
            ArmourTier::Infernal => 20000,
            ArmourTier::Omega => 40000,
            ArmourTier::Chaotic => 50000,
            ArmourTier::Mythical => 1000000,
            ArmourTier::Alydril => 1750000,
        }
    }

    pub fn rarity(&self) -> &'static str {
        match self {
            ArmourTier::None | ArmourTier::Cloth => "Common",
            ArmourTier::Chain | ArmourTier::Bronze => "Uncommon",
            ArmourTier::Iron | ArmourTier::Steel => "Rare",
            ArmourTier::Gold | ArmourTier::Crystal => "Epic",
            _ => "Legendary",
        }
    }

    pub fn level_required(&self) -> u32 {
        match self {
            ArmourTier::None => 1,
            ArmourTier::Cloth => 1,
            ArmourTier::Chain => 1,
            ArmourTier::Bronze => 5,
            ArmourTier::Iron => 5,
            ArmourTier::Steel => 10,
            ArmourTier::Gold => 10,
            ArmourTier::Crystal => 15,
            ArmourTier::Sacred => 15,
            ArmourTier::Runic => 20,
            ArmourTier::Infernal => 20,
            ArmourTier::Omega => 20,
            ArmourTier::Chaotic => 20,
            ArmourTier::Mythical => 30,
            ArmourTier::Alydril => 35,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcType {
    Goblin,
    Giant,
    Dragon,
    Mage,
    Sorcerer,
    Human,
    Farmer,
    Bear,
    Rockbeast,
    Ent,
    Thief,
    Elemental,
    Rat,
    Lacoyx,
    StrongholdMage,
    StrongholdRanger,
    StrongholdWarrior,
}

impl NpcType {
    pub fn name(&self) -> &'static str {
        match self {
            NpcType::Goblin => "Goblin",
            NpcType::Giant => "Giant",
            NpcType::Dragon => "Dragon",
            NpcType::Mage => "Mage",
            NpcType::Sorcerer => "Sorcerer",
            NpcType::Human => "Human",
            NpcType::Farmer => "Farmer",
            NpcType::Bear => "Bear",
            NpcType::Rockbeast => "Rockbeast",
            NpcType::Ent => "Ent",
            NpcType::Thief => "Thief",
            NpcType::Elemental => "Elemental",
            NpcType::Rat => "Rat",
            NpcType::Lacoyx => "Lacoyx",
            NpcType::StrongholdMage => "Stronghold Mage",
            NpcType::StrongholdRanger => "Stronghold Ranger",
            NpcType::StrongholdWarrior => "Stronghold Warrior",
        }
    }

    pub fn max_damage(&self) -> u32 {
        match self {
            NpcType::Rat => 53,
            NpcType::Goblin => 30,
            NpcType::Human => 50,
            NpcType::Giant => 60,
            NpcType::Farmer => 85,
            NpcType::Dragon => 90,
            NpcType::Mage => 120,
            NpcType::Sorcerer => 150,
            NpcType::Bear => 183,
            NpcType::Rockbeast => 240,
            NpcType::Ent => 300,
            NpcType::Thief => 330,
            NpcType::Elemental => 360,
            NpcType::StrongholdMage => 200,
            NpcType::StrongholdRanger => 200,
            NpcType::StrongholdWarrior => 200,
            NpcType::Lacoyx => 600,
        }
    }

    pub fn base_health(&self) -> u32 {
        match self {
            NpcType::StrongholdMage => 750,
            NpcType::StrongholdRanger => 750,
            NpcType::StrongholdWarrior => 750,
            NpcType::Lacoyx => 2780,
            _ => 0, // Wandering NPC health is scaled dynamically.
        }
    }

    pub fn base_xp(&self) -> u32 {
        match self {
            NpcType::StrongholdMage => 1000,
            NpcType::StrongholdRanger => 1000,
            NpcType::StrongholdWarrior => 1000,
            NpcType::Lacoyx => 5000,
            _ => 0, // Wandering NPC experience is calculated per action.
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    GoblinMail,
    HugeBones,
    DragonHides,
    RunicTablets,
    Potions,
    Food,
    Seeds,
    Fur,
    Ore,
    Logs,
    Gemstones,
    MagicOrbs,
    RawFood,
    Bait,
    Ingots,
    Bows,
    Thread,
    GourmetFeast,
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        match self {
            ItemType::GoblinMail => "Goblin Mails",
            ItemType::HugeBones => "Huge Bones",
            ItemType::DragonHides => "Dragon Hides",
            ItemType::RunicTablets => "Runic Tablets",
            ItemType::Potions => "Potions",
            ItemType::Food => "Food",
            ItemType::Seeds => "Seeds",
            ItemType::Fur => "Furs",
            ItemType::Ore => "Ore",
            ItemType::Logs => "Logs",
            ItemType::Gemstones => "Gemstones",
            ItemType::MagicOrbs => "Magic Orbs",
            ItemType::RawFood => "Raw Food",
            ItemType::Bait => "Bait",
            ItemType::Ingots => "Ingots",
            ItemType::Bows => "Bows",
            ItemType::Thread => "Thread",
            ItemType::GourmetFeast => "Gourmet Feast",
        }
    }

    pub fn buy_price(&self) -> u32 {
        match self {
            ItemType::GoblinMail => 300,
            ItemType::HugeBones => 550,
            ItemType::DragonHides => 750,
            ItemType::RunicTablets => 250,
            ItemType::Potions => 200,
            ItemType::Food => 100,
            ItemType::Seeds => 150,
            ItemType::Fur => 200,
            ItemType::Ore => 500,
            ItemType::Logs => 275,
            ItemType::Gemstones => 1000,
            ItemType::MagicOrbs => 15000,
            ItemType::RawFood => 100,
            ItemType::Bait => 2,
            ItemType::Ingots => 473,
            ItemType::Bows => 713,
            // Crafting-only products are not sold by the Trading Post.
            ItemType::Thread | ItemType::GourmetFeast => 0,
        }
    }

    pub fn sell_price(&self) -> u32 {
        self.buy_price() // Same as buy price in original
    }
}

pub const MAX_PLAYER_LEVEL: u32 = 37;
pub const MAX_SKILL_LEVEL: u32 = 100;
pub const MAX_GOLD: u32 = 5_000_000;
pub const MAX_BANK_GOLD: u32 = 1_000_000_000;

/// Damage channels used by the combat engine and NPC affinity table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Fire,
    Poison,
    Arcane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Bleed,
    Poison,
    Burn,
    Stun,
    Slow,
    Mark,
    WeakenedDefences,
}

/// Legacy combat-engine status storage. The Section 13 shared lifecycle lives
/// in status_combat::StatusEffect; this data shape remains for save/API
/// compatibility while legacy battle flows are migrated incrementally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub duration: u32,
    pub potency: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityId {
    HeavyStrike,
    ShieldWall,
    LifeDrain,
    Cleave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    pub id: AbilityId,
    pub name: String,
    pub description: String,
    pub stamina_cost: u32,
    pub mana_cost: u32,
    /// Percentage of the combatant's attack power (100 = 1.0x).
    pub damage_multiplier: u32,
    pub damage_type: DamageType,
}

pub fn get_ability(id: AbilityId) -> Ability {
    match id {
        AbilityId::HeavyStrike => Ability {
            id,
            name: "Heavy Strike".into(),
            description: "Deals 2.0x physical damage and inflicts Bleed.".into(),
            stamina_cost: 20,
            mana_cost: 0,
            damage_multiplier: 200,
            damage_type: DamageType::Physical,
        },
        AbilityId::ShieldWall => Ability {
            id,
            name: "Shield Wall".into(),
            description: "Reduces incoming damage by 50% for 2 turns.".into(),
            stamina_cost: 15,
            mana_cost: 0,
            damage_multiplier: 0,
            damage_type: DamageType::Physical,
        },
        AbilityId::LifeDrain => Ability {
            id,
            name: "Life Drain".into(),
            description: "Deals 1.2x Arcane damage and heals 50% of damage dealt.".into(),
            stamina_cost: 0,
            mana_cost: 25,
            damage_multiplier: 120,
            damage_type: DamageType::Arcane,
        },
        AbilityId::Cleave => Ability {
            id,
            name: "Cleave".into(),
            description: "Deals 1.5x physical damage and stuns the target for one turn.".into(),
            stamina_cost: 25,
            mana_cost: 0,
            damage_multiplier: 150,
            damage_type: DamageType::Physical,
        },
    }
}

/// Positive values resist damage; negative values increase damage taken.
pub fn get_npc_resistance(npc_name: &str, damage_type: DamageType) -> i32 {
    // Elite encounters reuse their base monster's elemental affinities.
    let base_name = npc_name.strip_prefix("Elite ").unwrap_or(npc_name);
    match (base_name, damage_type) {
        ("Dragon", DamageType::Fire) => 75,
        ("Dragon", DamageType::Physical) => 25,
        ("Ent", DamageType::Fire) => -50,
        ("Ent", DamageType::Physical) => 50,
        ("Rockbeast", DamageType::Physical) => 50,
        ("Rockbeast", DamageType::Arcane) => -50,
        ("Mage" | "Sorcerer", DamageType::Arcane) => 50,
        ("Mage" | "Sorcerer", DamageType::Physical) => -25,
        _ => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LacoyxPhase {
    Phase1Commander,
    Phase2InfernalWarlord,
    Phase3DesperateOverlord,
}

impl LacoyxPhase {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Phase1Commander => "Phase 1: War Commander",
            Self::Phase2InfernalWarlord => "Phase 2: Infernal Warlord",
            Self::Phase3DesperateOverlord => "Phase 3: Desperate Overlord",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MasteryPerkId {
    LumberjacksGreed,
    MasterArtisan,
    LethalPrecision,
    SilverTongue,
}
impl MasteryPerkId {
    pub const ALL: [Self; 4] = [
        Self::LumberjacksGreed,
        Self::MasterArtisan,
        Self::LethalPrecision,
        Self::SilverTongue,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::LumberjacksGreed => "Lumberjack's Greed",
            Self::MasterArtisan => "Master Artisan",
            Self::LethalPrecision => "Lethal Precision",
            Self::SilverTongue => "Silver Tongue",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::LumberjacksGreed => "+25% expected Woodcutting and Mining yield.",
            Self::MasterArtisan => "-15% recipe ingredients, minimum one each.",
            Self::LethalPrecision => "+10 percentage points critical chance (maximum 25%).",
            Self::SilverTongue => {
                "15% discount at gold-priced vendors; total discount caps at 30%."
            }
        }
    }
}
pub fn get_ascii_portrait(npc_name: &str) -> Vec<&'static str> {
    if npc_name.contains("Lacoyx") {
        vec![
            r"       /\\_____/\\",
            r"      /  o   o  \\",
            r"     ( ==  ^  == )",
            r"      ) INFERNAL (",
            r"     (___________)",
        ]
    } else if npc_name.contains("Dragon") {
        vec![
            r"         /\\",
            r"    ____/  \\___",
            r"   (  o      o )",
            r"    \\___/\\___/",
            r"        /  \\",
        ]
    } else {
        vec![
            r"       .-^^^-.",
            r"      /  o o  \\",
            r"     |    ^    |",
            r"      \\  '-'  /",
            r"       '-----'",
        ]
    }
}

#[cfg(test)]
mod npc_resistance_tests {
    use super::*;

    #[test]
    fn elite_monsters_keep_their_base_elemental_affinities() {
        for damage_type in [
            DamageType::Physical,
            DamageType::Fire,
            DamageType::Poison,
            DamageType::Arcane,
        ] {
            assert_eq!(
                get_npc_resistance("Elite Dragon", damage_type),
                get_npc_resistance("Dragon", damage_type)
            );
            assert_eq!(
                get_npc_resistance("Elite Ent", damage_type),
                get_npc_resistance("Ent", damage_type)
            );
        }
    }
}
