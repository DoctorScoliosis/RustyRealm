//! Safe-location checkpoint rules and per-login dirty-state tracking.

use std::fmt;

use crate::save::{self, SaveError};
use crate::state::GameState;
use crate::world::{Location, LocationService};

/// Checkpoints are available only at an explicitly tagged tavern or guildhouse.
pub fn is_checkpoint_location(location: &Location) -> bool {
    location.has_service(LocationService::Tavern)
        || location.has_service(LocationService::Guildhouse)
}

#[derive(Debug)]
pub enum CheckpointError {
    IneligibleLocation,
    LocationMismatch,
    Save(SaveError),
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IneligibleLocation => {
                f.write_str("A gameplay checkpoint can only be saved at a tavern or guildhouse.")
            }
            Self::LocationMismatch => {
                f.write_str("The checkpoint location must be the player's current location.")
            }
            Self::Save(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CheckpointError {}

/// Tracks the last successful checkpoint for one login session. Comparing the
/// canonical persisted payload catches changes in every menu/subsystem without
/// requiring each mutation site to remember a dirty flag.
#[derive(Debug, Clone)]
pub struct CheckpointSession {
    last_saved_fingerprint: String,
}

impl CheckpointSession {
    pub fn new(state: &GameState) -> Result<Self, SaveError> {
        Ok(Self {
            last_saved_fingerprint: save::checkpoint_fingerprint(state)?,
        })
    }

    pub fn is_dirty(&self, state: &GameState) -> Result<bool, SaveError> {
        Ok(save::checkpoint_fingerprint(state)? != self.last_saved_fingerprint)
    }

    pub fn save_at(
        &mut self,
        state: &GameState,
        location: &Location,
    ) -> Result<(), CheckpointError> {
        if !is_checkpoint_location(location) {
            return Err(CheckpointError::IneligibleLocation);
        }
        if location.id != state.current_location {
            return Err(CheckpointError::LocationMismatch);
        }
        self.commit_with(state, save::save_game)
    }

    fn commit_with(
        &mut self,
        state: &GameState,
        write: impl FnOnce(&GameState) -> Result<(), SaveError>,
    ) -> Result<(), CheckpointError> {
        // Update the baseline only after the atomic save succeeds.
        let fingerprint = save::checkpoint_fingerprint(state).map_err(CheckpointError::Save)?;
        write(state).map_err(CheckpointError::Save)?;
        self.last_saved_fingerprint = fingerprint;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ItemType;
    use crate::state::Player;
    use crate::world::LocationCategory;

    fn state() -> GameState {
        GameState::new(Player::new("checkpoint-test".into(), "test-hash".into()))
    }

    fn location(services: &[LocationService]) -> Location {
        Location::new(
            "test_location",
            "Test Location",
            "Test",
            LocationCategory::Settlement,
        )
        .with_services(services.iter().copied())
    }

    #[test]
    fn only_explicit_taverns_and_guildhouses_are_checkpoint_locations() {
        assert!(is_checkpoint_location(&location(&[
            LocationService::Tavern
        ])));
        assert!(is_checkpoint_location(&location(&[
            LocationService::Guildhouse
        ])));
        assert!(!is_checkpoint_location(&location(&[LocationService::Bed])));
        assert!(!is_checkpoint_location(&location(&[
            LocationService::FieldCamp
        ])));
        assert!(!is_checkpoint_location(&location(&[])));
    }

    #[test]
    fn session_detects_any_persisted_state_change() {
        let mut state = state();
        let session = CheckpointSession::new(&state).unwrap();
        assert!(!session.is_dirty(&state).unwrap());
        state.player.gold += 1;
        assert!(session.is_dirty(&state).unwrap());
        state.player.add_item(ItemType::Logs, 1);
        assert!(session.is_dirty(&state).unwrap());
    }

    #[test]
    fn checkpoint_write_rejects_a_safe_location_other_than_the_current_location() {
        let state = state();
        let mut session = CheckpointSession::new(&state).unwrap();
        let tavern = Location::new(
            "another_tavern",
            "Another Tavern",
            "Tavern",
            LocationCategory::Settlement,
        )
        .with_services([LocationService::Tavern]);
        assert!(matches!(
            session.save_at(&state, &tavern),
            Err(CheckpointError::LocationMismatch)
        ));
        assert!(!session.is_dirty(&state).unwrap());
    }

    #[test]
    fn dirty_state_clears_only_after_a_successful_checkpoint_write() {
        let mut state = state();
        let mut session = CheckpointSession::new(&state).unwrap();
        state.player.gold += 5;
        assert!(session.is_dirty(&state).unwrap());
        session
            .commit_with(&state, |_| Ok::<(), SaveError>(()))
            .unwrap();
        assert!(!session.is_dirty(&state).unwrap());
    }

    #[test]
    fn ineligible_location_and_failed_write_do_not_clear_dirty_state() {
        let mut state = state();
        let mut session = CheckpointSession::new(&state).unwrap();
        state.player.gold += 5;
        let field_camp = location(&[LocationService::FieldCamp]);
        assert!(matches!(
            session.save_at(&state, &field_camp),
            Err(CheckpointError::IneligibleLocation)
        ));
        assert!(session.is_dirty(&state).unwrap());

        let tavern = location(&[LocationService::Tavern]);
        state.current_location = tavern.id.clone();
        // Passwordless local accounts are still valid save targets.
        session.save_at(&state, &tavern).unwrap();
        assert!(!session.is_dirty(&state).unwrap());
    }
}
