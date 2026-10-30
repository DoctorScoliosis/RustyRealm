use crate::character::{CharacterClass, PrimaryAttributes};
use crate::data::{
    ArmourTier, ItemType, MasteryPerkId, WeaponTier, MAX_BANK_GOLD, MAX_GOLD, MAX_PLAYER_LEVEL,
    MAX_SKILL_LEVEL,
};
use crate::economy::MarketState;
use crate::event::WorldEvent;
use crate::party::{CompanionId, CompanionState, Party};
use crate::quests::Quest;
use crate::world::LocationId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

fn default_stamina() -> u32 {
    100
}

fn default_mana() -> u32 {
    50
}
fn default_theme() -> String {
    "Classic".to_string()
}
fn default_unlocked_themes() -> Vec<String> {
    vec!["Classic".to_string()]
}

fn default_current_location() -> LocationId {
    LocationId::from("village_start")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    #[serde(skip)]
    pub password_hash: String, // Not saved in JSON, handled separately
    /// Durable class selection. Legacy saves default deterministically to Warrior.
    #[serde(default)]
    pub class: CharacterClass,
    /// Persistent primary attributes on the Section 24 1..=20 scale.
    #[serde(default)]
    pub attributes: PrimaryAttributes,
    /// Smooth weighted-round-robin remainder for deterministic level-up growth.
    #[serde(default)]
    pub attribute_growth_accumulator: [i32; 6],
    pub combat_level: u32,
    pub xp: u32,
    pub xp_until_next: u32,
    pub orig_xp_threshold: u32,
    pub hp: u32,
    pub max_hp: u32,
    #[serde(default = "default_stamina")]
    pub stamina: u32,
    #[serde(default = "default_stamina")]
    pub max_stamina: u32,
    #[serde(default = "default_mana")]
    pub mana: u32,
    #[serde(default = "default_mana")]
    pub max_mana: u32,
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
        items.insert(ItemType::ScrapMetal, 0);
        items.insert(ItemType::Bows, 0);
        items.insert(ItemType::Thread, 0);
        items.insert(ItemType::GourmetFeast, 0);

        Self {
            username,
            password_hash,
            class: CharacterClass::Warrior,
            attributes: PrimaryAttributes::default(),
            attribute_growth_accumulator: [0; 6],
            combat_level: 1,
            xp: 0,
            xp_until_next: 500,
            orig_xp_threshold: 500,
            hp: 100,
            max_hp: 100,
            stamina: default_stamina(),
            max_stamina: default_stamina(),
            mana: default_mana(),
            max_mana: default_mana(),
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

    /// Create a player with a durable starting class. The legacy constructor
    /// remains available and defaults to Warrior.
    pub fn new_with_class(username: String, password_hash: String, class: CharacterClass) -> Self {
        let mut player = Self::new(username, password_hash);
        player.class = class;
        player
    }

    /// Effective weapon damage includes only the current class's bounded
    /// familiarity with the equipped weapon family. Equipment remains legal
    /// regardless of class.
    pub fn effective_weapon_damage(&self) -> u32 {
        let bonus = u32::from(
            self.class
                .weapon_proficiency_pct(self.weapon.profile().family),
        );
        self.weapon.max_damage().saturating_mul(100 + bonus) / 100
    }

    pub fn level_up_combat(&mut self) {
        if self.combat_level < MAX_PLAYER_LEVEL {
            self.combat_level += 1;
            self.orig_xp_threshold += 500;
            self.xp_until_next += self.orig_xp_threshold;
            self.max_hp += 25;
            self.hp = self.max_hp;
            let gained_attribute = self
                .attributes
                .add_weighted_point(self.class, &mut self.attribute_growth_accumulator);
            if gained_attribute == Some(2) {
                // Constitution strengthens both health capacity and endurance.
                self.max_hp = self.max_hp.saturating_add(5);
                self.max_stamina = self.max_stamina.saturating_add(2);
                self.hp = self.max_hp;
                self.stamina = self.max_stamina;
            }
        }
    }

    /// Clamp persisted player values to valid game limits without discarding the
    /// remainder of the player's progress.
    pub fn cap_values(&mut self) {
        self.attributes.cap_values();
        for remainder in &mut self.attribute_growth_accumulator {
            *remainder = (*remainder).clamp(-10_000, 10_000);
        }
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

        if self.max_stamina == 0 {
            self.max_stamina = default_stamina();
        }
        if self.stamina > self.max_stamina {
            self.stamina = self.max_stamina;
        }

        if self.max_mana == 0 {
            self.max_mana = default_mana();
        }
        if self.mana > self.max_mana {
            self.mana = self.max_mana;
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
    #[serde(default = "default_current_location")]
    pub current_location: LocationId,
    #[serde(default)]
    pub active_quests: Vec<Quest>,
    #[serde(default = "WorldEvent::none")]
    pub active_world_event: WorldEvent,
    /// Added with defaults for backward-compatible progression saves.
    #[serde(default)]
    pub market_state: MarketState,
    /// Persisted world clock. One committed world turn is exactly one in-game hour.
    /// The calendar is derived from this field so the two clocks cannot diverge.
    #[serde(default)]
    pub world_turn: u64,
    #[serde(default)]
    pub stronghold_high_score: u32,
    #[serde(default)]
    pub lacoyx_defeated: bool,
    #[serde(default)]
    pub unlocked_achievements: Vec<String>,
    #[serde(default)]
    pub unlocked_titles: Vec<String>,
    #[serde(default)]
    pub active_title: Option<String>,
    #[serde(default = "default_unlocked_themes")]
    pub unlocked_themes: Vec<String>,
    #[serde(default = "default_theme")]
    pub active_theme: String,
    #[serde(default)]
    pub mastery_perks: Vec<MasteryPerkId>,
    /// Stable companion IDs revealed through authored gameplay discovery.
    /// Recruited legacy records also count as discovered during migration.
    #[serde(default)]
    pub discovered_companions: Vec<CompanionId>,
    /// Recruited companions and their mutable state.
    #[serde(default)]
    pub companions: Vec<CompanionState>,
    /// Active party composition (player always occupies one implicit slot).
    #[serde(default)]
    pub party: Party,
    /// Persisted knockout destinations and recovery progression.
    #[serde(default)]
    pub companion_recovery: crate::companion_recovery::RecoveryState,
    /// Save-backed content progression for one-time regional rewards and surveys.
    #[serde(default)]
    pub region_progress: crate::adventure::RegionProgress,
    /// Persistent discovery, objective, completion and mastery state for the campaign.
    #[serde(default)]
    pub campaign_progress: crate::realm_progression::CampaignProgress,
}

impl GameState {
    /// Advance elapsed in-game hours. Zero hours are a strict no-op. The clock
    /// saturates at u64::MAX; dependent systems receive only the elapsed hours
    /// that could actually be represented by the persisted counter.
    pub fn advance_world_time(&mut self, hours: u64) -> Vec<crate::party::CompanionId> {
        if hours == 0 {
            return Vec::new();
        }
        let previous = self.world_turn;
        let next = previous.saturating_add(hours);
        let elapsed = next.saturating_sub(previous);
        if elapsed == 0 {
            return Vec::new();
        }

        self.world_turn = next;
        self.market_state.advance_turns(elapsed);
        self.companion_recovery
            .advance_committed_turns(&mut self.companions, elapsed)
    }

    /// Backward-compatible one-hour advancement for existing action systems.
    pub fn advance_world_turn(&mut self) -> Vec<crate::party::CompanionId> {
        self.advance_world_time(1)
    }

    pub fn new(player: Player) -> Self {
        let mut state = Self {
            player,
            current_location: default_current_location(),
            active_quests: Vec::new(),
            active_world_event: WorldEvent::none(),
            market_state: MarketState::new(),
            world_turn: 0,
            stronghold_high_score: 0,
            lacoyx_defeated: false,
            unlocked_achievements: Vec::new(),
            unlocked_titles: Vec::new(),
            active_title: None,
            unlocked_themes: default_unlocked_themes(),
            active_theme: default_theme(),
            mastery_perks: Vec::new(),
            discovered_companions: Vec::new(),
            companions: Vec::new(),
            party: Party::new(),
            companion_recovery: crate::companion_recovery::RecoveryState::default(),
            region_progress: crate::adventure::RegionProgress::default(),
            campaign_progress: crate::realm_progression::CampaignProgress::default(),
        };
        crate::quests::check_and_refresh_quests(&mut state);
        state
    }
}
