use crate::event::WorldEvent;
use crate::quests::Quest;
use crate::data::{
    ArmourTier, ItemType, WeaponTier, MAX_BANK_GOLD, MAX_GOLD, MAX_PLAYER_LEVEL, MAX_SKILL_LEVEL,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    #[serde(skip)]
    pub password_hash: String, // Not saved in JSON, handled separately
    pub combat_level: u32,
    pub xp: u32,
    pub xp_until_next: u32,
    pub orig_xp_threshold: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub gold: u32,
    pub weapon: WeaponTier,
    pub armour: ArmourTier,
    pub inventory_weapons: Vec<WeaponTier>,
    pub inventory_armour: Vec<ArmourTier>,
    pub items: std::collections::HashMap<ItemType, u32>,
    pub skills: Skills,
    pub stronghold_kill_count: u32,
    #[serde(default)]
    pub lifetime_kills: u32,
    pub bank_accounts: [BankAccount; 5],
    pub quests_completed: Vec<String>,
    pub last_quest_refresh: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub woodcutting: SkillData,
    pub cooking: SkillData,
    pub fishing: SkillData,
    pub mining: SkillData,
    pub smithing: SkillData,
    pub thieving: SkillData,
}

impl Skills {
    pub fn new() -> Self {
        Self {
            woodcutting: SkillData::new(38),
            cooking: SkillData::new(41),
            fishing: SkillData::new(30),
            mining: SkillData::new(35),
            smithing: SkillData::new(33),
            thieving: SkillData::new(36),
        }
    }

    pub fn total_level(&self) -> u32 {
        self.woodcutting.level
            + self.cooking.level
            + self.fishing.level
            + self.mining.level
            + self.smithing.level
            + self.thieving.level
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub level: u32,
    pub current_xp: u32,
    pub xp_remaining: u32,
    pub xp_gain_per_action: u32,
    pub xp_threshold: u32,
}

impl SkillData {
    pub fn new(xp_gain: u32) -> Self {
        Self {
            level: 1,
            current_xp: 0,
            xp_remaining: 100,
            xp_gain_per_action: xp_gain,
            xp_threshold: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BankAccount {
    pub gold: u32,
    pub is_closed: bool,
}

impl BankAccount {
    pub fn new() -> Self {
        Self {
            gold: 0,
            is_closed: false,
        }
    }
}

impl Player {
    pub fn new(username: String, password_hash: String) -> Self {
        let mut items = std::collections::HashMap::new();
        items.insert(ItemType::Food, 0);
        items.insert(ItemType::RawFood, 0);
        items.insert(ItemType::Potions, 0);
        items.insert(ItemType::Bait, 0);
        items.insert(ItemType::Seeds, 0);
        items.insert(ItemType::Fur, 0);
        items.insert(ItemType::Ore, 0);
        items.insert(ItemType::Logs, 0);
        items.insert(ItemType::Gemstones, 0);
        items.insert(ItemType::MagicOrbs, 0);
        items.insert(ItemType::GoblinMail, 0);
        items.insert(ItemType::HugeBones, 0);
        items.insert(ItemType::DragonHides, 0);
        items.insert(ItemType::RunicTablets, 0);
        items.insert(ItemType::Ingots, 0);
        items.insert(ItemType::Bows, 0);

        Self {
            username,
            password_hash,
            combat_level: 1,
            xp: 0,
            xp_until_next: 500,
            orig_xp_threshold: 500,
            hp: 100,
            max_hp: 100,
            gold: 100,
            weapon: WeaponTier::BareHands,
            armour: ArmourTier::None,
            inventory_weapons: vec![],
            inventory_armour: vec![],
            items,
            skills: Skills::new(),
            stronghold_kill_count: 0,
            lifetime_kills: 0,
            bank_accounts: [BankAccount::new(); 5],
            quests_completed: vec![],
            last_quest_refresh: None,
        }
    }

    pub fn level_up_combat(&mut self) {
        if self.combat_level < MAX_PLAYER_LEVEL {
            self.combat_level += 1;
            self.orig_xp_threshold += 500;
            self.xp_until_next += self.orig_xp_threshold;
            self.max_hp += 25;
            self.hp = self.max_hp;
        }
    }

    /// Clamp persisted player values to valid game limits without discarding the
    /// remainder of the player's progress.
    pub fn cap_values(&mut self) {
        if self.combat_level == 0 {
            self.combat_level = 1;
        }
        if self.combat_level > MAX_PLAYER_LEVEL {
            self.combat_level = MAX_PLAYER_LEVEL;
        }

        if self.max_hp == 0 {
            self.max_hp = 100;
        }
        if self.max_hp > 1000 {
            self.max_hp = 1000;
        }
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }

        if self.gold > MAX_GOLD {
            self.gold = MAX_GOLD;
        }
        if self.xp > 1_000_000 {
            self.xp = 1_000_000;
        }

        for skill in [
            &mut self.skills.woodcutting,
            &mut self.skills.cooking,
            &mut self.skills.fishing,
            &mut self.skills.mining,
            &mut self.skills.smithing,
            &mut self.skills.thieving,
        ] {
            if skill.level == 0 {
                skill.level = 1;
            }
            if skill.level > MAX_SKILL_LEVEL {
                skill.level = MAX_SKILL_LEVEL;
            }
            if skill.level >= MAX_SKILL_LEVEL {
                skill.xp_remaining = 0;
            }
        }

        for account in &mut self.bank_accounts {
            if account.gold > MAX_BANK_GOLD {
                account.gold = MAX_BANK_GOLD;
            }
            // A full account should be closed; a non-full account can be reopened.
            account.is_closed = account.gold >= MAX_BANK_GOLD;
        }
    }

    pub fn add_item(&mut self, item: ItemType, amount: u32) {
        *self.items.entry(item).or_insert(0) += amount;
    }

    pub fn remove_item(&mut self, item: ItemType, amount: u32) -> bool {
        if let Some(count) = self.items.get_mut(&item) {
            if *count >= amount {
                *count -= amount;
                return true;
            }
        }
        false
    }

    pub fn get_item_count(&self, item: ItemType) -> u32 {
        *self.items.get(&item).unwrap_or(&0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    #[serde(default)]
    pub active_quests: Vec<Quest>,
    #[serde(default = "WorldEvent::none")]
    pub active_world_event: WorldEvent,
}

impl GameState {
    pub fn new(player: Player) -> Self {
        let mut state = Self {
            player,
            active_quests: Vec::new(),
            active_world_event: WorldEvent::none(),
        };
        crate::quests::check_and_refresh_quests(&mut state);
        state
    }
}
