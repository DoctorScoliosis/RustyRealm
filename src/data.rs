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
