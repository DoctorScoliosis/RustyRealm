use crate::data::ItemType;
use serde::{Deserialize, Serialize};

/// A domain event emitted after a gameplay action succeeds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    MonsterDefeated { name: String, is_stronghold: bool },
    PlayerLeveledUp { new_level: u32 },
    SkillLeveledUp { skill_name: String, new_level: u32 },
    ItemHandedIn { item_type: ItemType, count: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum WorldEventType {
    MonsterSurge {
        monster_type: String,
    },
    MarketBoom {
        item_type: ItemType,
        price_multiplier: u32,
    },
    DoubleSkillXp {
        skill_name: String,
    },
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorldEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_type: WorldEventType,
    pub duration_secs: u64,
    pub start_timestamp: u64,
}

impl WorldEvent {
    pub fn none() -> Self {
        Self {
            id: "none".into(),
            name: "Peaceful Days".into(),
            description: "Gielinor is quiet today.".into(),
            event_type: WorldEventType::None,
            duration_secs: 86_400,
            start_timestamp: 0,
        }
    }

    pub fn is_active(&self, current_timestamp: u64) -> bool {
        self.event_type != WorldEventType::None
            && current_timestamp >= self.start_timestamp
            && current_timestamp < self.start_timestamp.saturating_add(self.duration_secs)
    }

    /// Generate the same world event for a given daily seed.
    ///
    /// Gameplay effects for market prices and skill XP are intentionally left
    /// for the economy and skilling sections; this section persists and displays
    /// the event state only.
    pub fn generate_deterministic(seed: u64, timestamp: u64) -> Self {
        match seed % 3 {
            0 => Self {
                id: format!("surge_{seed}"),
                name: "Goblin Surge".into(),
                description: "Goblins are appearing more frequently across Gielinor!".into(),
                event_type: WorldEventType::MonsterSurge {
                    monster_type: "Goblin".into(),
                },
                duration_secs: 86_400,
                start_timestamp: timestamp,
            },
            1 => Self {
                id: format!("market_{seed}"),
                name: "Ore Shortage".into(),
                description: "Trading Post vendors are paying high rates for Ore!".into(),
                event_type: WorldEventType::MarketBoom {
                    item_type: ItemType::Ore,
                    price_multiplier: 2,
                },
                duration_secs: 86_400,
                start_timestamp: timestamp,
            },
            _ => Self {
                id: format!("xp_{seed}"),
                name: "Woodcutting Focus".into(),
                description: "Woodcutting yields extra experience today!".into(),
                event_type: WorldEventType::DoubleSkillXp {
                    skill_name: "Woodcutting".into(),
                },
                duration_secs: 86_400,
                start_timestamp: timestamp,
            },
        }
    }
}

impl Default for WorldEvent {
    fn default() -> Self {
        Self::none()
    }
}
