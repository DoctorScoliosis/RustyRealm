use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::character::{CharacterClass, PrimaryAttributes};
use crate::state::{GameState, Player};

const CURRENT_SAVE_VERSION: u32 = 2;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    ChecksumMismatch { expected: String, computed: String },
    UnsupportedVersion(u32),
    AuthenticationFailed,
    InvalidPasswordHash,
    PasswordHashingFailed,
    PasswordMigrationRequired,
    Corrupted(String),
    ProjectDirectoriesUnavailable,
    InvalidUsername,
    AccountExists,
    EmptyPassword,
    UsernameMismatch,
    ConfirmationMismatch,
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(error) => write!(f, "Save-file I/O error: {error}"),
            SaveError::Serialization(error) => write!(f, "Save serialisation error: {error}"),
            SaveError::ChecksumMismatch { expected, computed } => write!(
                f,
                "Save checksum mismatch (stored {expected}, computed {computed}); the file may be corrupted."
            ),
            SaveError::UnsupportedVersion(version) => {
                write!(f, "Unsupported save format version: {version}")
            }
            SaveError::AuthenticationFailed => write!(f, "Invalid username or password."),
            SaveError::InvalidPasswordHash => {
                write!(f, "The saved password hash is missing or invalid.")
            }
            SaveError::PasswordHashingFailed => write!(f, "Could not securely hash the password."),
            SaveError::PasswordMigrationRequired => write!(
                f,
                "This legacy save has no verifiable password hash. Explicit password migration is required."
            ),
            SaveError::Corrupted(reason) => write!(f, "Save file is corrupted: {reason}"),
            SaveError::ProjectDirectoriesUnavailable => {
                write!(f, "Could not determine the platform-specific save directory.")
            }
            SaveError::InvalidUsername => {
                write!(f, "Username must contain at least one supported character.")
            }
            SaveError::AccountExists => write!(f, "An account with that username already exists."),
            SaveError::EmptyPassword => write!(f, "Password cannot be empty."),
            SaveError::UsernameMismatch => {
                write!(f, "The save username does not match the requested account.")
            }
            SaveError::ConfirmationMismatch => {
                write!(f, "The typed account name did not match; the account was not deleted.")
            }
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        SaveError::Io(error)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(error: serde_json::Error) -> Self {
        SaveError::Serialization(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthMetadata {
    password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveEnvelope {
    version: u32,
    checksum: String,
    authentication: Option<AuthMetadata>,
    data: GameState,
}

enum ParsedSave {
    Current(SaveEnvelope),
    Legacy(GameState),
}

pub fn get_save_dir() -> Result<PathBuf, SaveError> {
    let proj_dirs = directories::ProjectDirs::from("com", "rustyrealm", "RustyRealm")
        .ok_or(SaveError::ProjectDirectoriesUnavailable)?;
    let save_dir = proj_dirs.data_dir().join("saves");
    fs::create_dir_all(&save_dir)?;
    Ok(save_dir)
}

pub fn sanitize_username(username: &str) -> String {
    username
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

pub fn hash_password(password: &str) -> Result<String, SaveError> {
    if password.is_empty() {
        return Err(SaveError::EmptyPassword);
    }
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| SaveError::PasswordHashingFailed)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, SaveError> {
    if password_hash.trim().is_empty() {
        return Err(SaveError::InvalidPasswordHash);
    }

    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|_| SaveError::InvalidPasswordHash)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(_) => Err(SaveError::InvalidPasswordHash),
    }
}

fn compute_checksum_payload(payload: &serde_json::Value) -> Result<String, SaveError> {
    // serde_json::Value canonicalises object keys. When reading an older save,
    // checksum the raw data object before serde(default) adds newly introduced
    // fields, otherwise an unchanged version-2 save would appear corrupted.
    let payload = serde_json::to_vec(payload)?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_checksum(state: &GameState) -> Result<String, SaveError> {
    // Player::password_hash is #[serde(skip)] and is stored separately.
    let canonical_payload = serde_json::to_value(state)?;
    compute_checksum_payload(&canonical_payload)
}

/// Fingerprint the exact normalised gameplay payload that a successful
/// checkpoint write would persist. Authentication metadata is intentionally
/// excluded, as are transient values skipped by serde.
pub fn checkpoint_fingerprint(state: &GameState) -> Result<String, SaveError> {
    let mut safe_state = state.clone();
    safe_state.player.cap_values();
    for companion in &mut safe_state.companions {
        companion.cap_values();
    }
    compute_checksum(&safe_state)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("bak")
}

fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn temporary_path(path: &Path, purpose: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("save");
    parent_dir(path).join(format!(
        ".{file_name}.{purpose}.{}.{}.tmp",
        std::process::id(),
        now
    ))
}

fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), SaveError> {
    #[cfg(not(windows))]
    {
        fs::rename(temp_path, destination)?;
        Ok(())
    }

    #[cfg(windows)]
    {
        if !destination.exists() {
            fs::rename(temp_path, destination)?;
            return Ok(());
        }

        // std::fs::rename cannot replace an existing destination on Windows.
        // The active save has already been backed up when it was valid, so use
        // a recoverable swap and restore the destination if the second rename fails.
        let displaced = temporary_path(destination, "previous");
        if displaced.exists() {
            fs::remove_file(&displaced)?;
        }
        fs::rename(destination, &displaced)?;
        match fs::rename(temp_path, destination) {
            Ok(()) => {
                let _ = fs::remove_file(displaced);
                Ok(())
            }
            Err(replace_error) => match fs::rename(&displaced, destination) {
                Ok(()) => Err(SaveError::Io(replace_error)),
                Err(restore_error) => Err(SaveError::Corrupted(format!(
                    "replacement failed ({replace_error}) and restoring the previous file failed ({restore_error}); a recovery copy may remain at {}",
                    displaced.display()
                ))),
            },
        }
    }
}

fn write_file_atomically(path: &Path, contents: &[u8]) -> Result<(), SaveError> {
    fs::create_dir_all(parent_dir(path))?;
    let temp = temporary_path(path, "write");

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(contents)?;
        file.sync_all()?;
        replace_file(&temp, path)?;

        // Syncing the directory is supported on Unix-like systems. It is
        // best-effort because some platforms do not allow opening directories.
        if let Ok(directory) = File::open(parent_dir(path)) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn read_save(path: &Path) -> Result<ParsedSave, SaveError> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Inspect the envelope as JSON first, so integrity is checked against the
    // exact payload stored on disk. This keeps older version-2 saves valid when
    // newly added fields have serde defaults.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) {
        let is_envelope = value.get("version").is_some()
            && value.get("checksum").is_some()
            && value.get("data").is_some();
        if is_envelope {
            let envelope: SaveEnvelope = serde_json::from_value(value.clone())
                .map_err(|error| SaveError::Corrupted(format!("invalid save envelope: {error}")))?;
            if envelope.version != CURRENT_SAVE_VERSION {
                return Err(SaveError::UnsupportedVersion(envelope.version));
            }

            let computed = compute_checksum_payload(&value["data"])?;
            if envelope.checksum != computed {
                return Err(SaveError::ChecksumMismatch {
                    expected: envelope.checksum,
                    computed,
                });
            }
            return Ok(ParsedSave::Current(envelope));
        }
    }

    match serde_json::from_str::<GameState>(&contents) {
        Ok(mut legacy_state) => {
            legacy_state.player.cap_values();
            for companion in &mut legacy_state.companions {
                companion.cap_values();
            }
            Ok(ParsedSave::Legacy(legacy_state))
        }
        Err(error) => Err(SaveError::Corrupted(format!(
            "not a recognised save envelope or legacy game-state JSON ({error})"
        ))),
    }
}

fn save_to_path(state: &GameState, path: &Path) -> Result<(), SaveError> {
    let mut safe_state = state.clone();
    safe_state.player.cap_values();
    for companion in &mut safe_state.companions {
        companion.cap_values();
    }

    let password_hash = state.player.password_hash.trim();
    let checksum = compute_checksum(&safe_state)?;
    let envelope = SaveEnvelope {
        version: CURRENT_SAVE_VERSION,
        checksum,
        authentication: (!password_hash.is_empty()).then(|| AuthMetadata {
            password_hash: password_hash.to_owned(),
        }),
        data: safe_state,
    };
    let serialised = serde_json::to_vec_pretty(&envelope)?;

    // Only refresh the backup if the active file itself is valid. Do not use a
    // loader that falls back to .bak here: doing so could copy a corrupt active
    // file over the last known-good backup.
    if path.exists() && read_save(path).is_ok() {
        let current_contents = fs::read(path)?;
        write_file_atomically(&backup_path(path), &current_contents)?;
    }

    write_file_atomically(path, &serialised)
}

pub fn save_game(state: &GameState) -> Result<(), SaveError> {
    let path = get_save_dir()?.join(format!(
        "{}.json",
        sanitize_username(&state.player.username)
    ));
    save_to_path(state, &path)
}

fn account_requires_password_at(path: &Path) -> Result<bool, SaveError> {
    match read_save(path)? {
        ParsedSave::Current(envelope) => Ok(envelope.authentication.is_some()),
        ParsedSave::Legacy(_) => Err(SaveError::PasswordMigrationRequired),
    }
}

fn load_game_at(path: &Path, username: &str, password: &str) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }

    let (parsed, loaded_from_backup) = match read_save(path) {
        Ok(save) => (save, false),
        Err(error) if matches!(&error, SaveError::UnsupportedVersion(_)) => return Err(error),
        Err(primary_error) => {
            let backup = backup_path(path);
            if !backup.exists() {
                return Err(primary_error);
            }
            match read_save(&backup) {
                Ok(save) => (save, true),
                Err(_) => return Err(primary_error),
            }
        }
    };

    let mut state = match parsed {
        ParsedSave::Legacy(state) => {
            if sanitize_username(&state.player.username) != safe_username {
                return Err(SaveError::UsernameMismatch);
            }
            return Err(SaveError::PasswordMigrationRequired);
        }
        ParsedSave::Current(envelope) => {
            if sanitize_username(&envelope.data.player.username) != safe_username {
                return Err(SaveError::UsernameMismatch);
            }
            let mut state = envelope.data;
            if let Some(auth) = envelope.authentication {
                if !verify_password(password, &auth.password_hash)? {
                    return Err(SaveError::AuthenticationFailed);
                }
                state.player.password_hash = auth.password_hash;
            } else if !password.is_empty() {
                return Err(SaveError::AuthenticationFailed);
            }
            state
        }
    };

    state.player.cap_values();
    for companion in &mut state.companions {
        companion.cap_values();
    }

    // Restore only after a candidate backup has passed checksum and password
    // validation. The original corrupt file is never copied over the backup.
    if loaded_from_backup {
        let contents = fs::read(backup_path(path))?;
        write_file_atomically(path, &contents)?;
    }

    Ok(state)
}

fn read_save_with_backup(path: &Path) -> Result<(ParsedSave, bool), SaveError> {
    match read_save(path) {
        Ok(save) => Ok((save, false)),
        Err(error) if matches!(&error, SaveError::UnsupportedVersion(_)) => Err(error),
        Err(primary_error) => {
            let backup = backup_path(path);
            if !backup.exists() {
                return Err(primary_error);
            }
            match read_save(&backup) {
                Ok(save) => Ok((save, true)),
                Err(_) => Err(primary_error),
            }
        }
    }
}

/// Read a local account's displayable game data without authorising gameplay.
/// The preview is not a login: password verification still happens in load_game.
pub fn preview_local_account(username: &str) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    preview_account_at(&path, &safe_username)
}

fn preview_account_at(path: &Path, username: &str) -> Result<GameState, SaveError> {
    let (parsed, _) = read_save_with_backup(path)?;
    let mut state = match parsed {
        ParsedSave::Legacy(state) => state,
        ParsedSave::Current(envelope) => envelope.data,
    };
    if sanitize_username(&state.player.username) != sanitize_username(username) {
        return Err(SaveError::UsernameMismatch);
    }
    state.player.password_hash.clear();
    state.player.cap_values();
    for companion in &mut state.companions {
        companion.cap_values();
    }
    Ok(state)
}

/// Delete both the active local save and its recovery backup. Password-protected
/// accounts require their existing password; passwordless/legacy accounts
/// require the exact account name as a local confirmation.
pub fn delete_local_account(
    username: &str,
    password: &str,
    confirmation: &str,
) -> Result<(), SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    delete_account_at(&path, &safe_username, password, confirmation)
}

fn delete_account_at(
    path: &Path,
    username: &str,
    password: &str,
    confirmation: &str,
) -> Result<(), SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    if confirmation != safe_username {
        return Err(SaveError::ConfirmationMismatch);
    }
    let (parsed, _) = read_save_with_backup(path)?;
    let saved_username = match &parsed {
        ParsedSave::Legacy(state) => state.player.username.as_str(),
        ParsedSave::Current(envelope) => envelope.data.player.username.as_str(),
    };
    if sanitize_username(saved_username) != safe_username {
        return Err(SaveError::UsernameMismatch);
    }
    if let ParsedSave::Current(envelope) = parsed {
        match envelope.authentication {
            Some(auth) => {
                if !verify_password(password, &auth.password_hash)? {
                    return Err(SaveError::AuthenticationFailed);
                }
            }
            None if !password.is_empty() => return Err(SaveError::AuthenticationFailed),
            None => {}
        }
    } else if !password.is_empty() {
        // A legacy save has no verifiable password hash, so only the typed
        // account-name confirmation can be used before explicit migration.
        return Err(SaveError::PasswordMigrationRequired);
    }

    for candidate in [path.to_path_buf(), backup_path(path)] {
        match fs::remove_file(&candidate) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(SaveError::Io(error)),
        }
    }
    Ok(())
}

pub fn load_game(username: &str, password: &str) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    load_game_at(&path, &safe_username, password)
}

/// Return the names of valid local save files, newest modified first.
fn list_accounts_in(directory: &Path) -> Result<Vec<String>, SaveError> {
    let mut accounts = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Some(username) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        if sanitize_username(username) != username {
            continue;
        }
        if read_save(&path).is_ok() || read_save(&backup_path(&path)).is_ok() {
            let modified = entry.metadata()?.modified().unwrap_or(UNIX_EPOCH);
            accounts.push((username.to_owned(), modified));
        }
    }
    accounts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    Ok(accounts.into_iter().map(|(username, _)| username).collect())
}

pub fn list_local_accounts() -> Result<Vec<String>, SaveError> {
    list_accounts_in(&get_save_dir()?)
}
pub fn account_requires_password(username: &str) -> Result<bool, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    match read_save(&path) {
        Ok(_) => account_requires_password_at(&path),
        Err(primary_error) => {
            let backup = backup_path(&path);
            if !backup.exists() {
                return Err(primary_error);
            }
            account_requires_password_at(&backup)
        }
    }
}
fn migrate_legacy_at(
    path: &Path,
    username: &str,
    new_password: &str,
) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }

    let parsed = match read_save(path) {
        Ok(save) => save,
        Err(primary_error) => {
            let backup = backup_path(path);
            if !backup.exists() {
                return Err(primary_error);
            }
            match read_save(&backup) {
                Ok(save) => save,
                Err(_) => return Err(primary_error),
            }
        }
    };

    let mut state = match parsed {
        ParsedSave::Legacy(state) => state,
        ParsedSave::Current(envelope) if envelope.authentication.is_none() => envelope.data,
        ParsedSave::Current(_) => return Err(SaveError::AuthenticationFailed),
    };

    if sanitize_username(&state.player.username) != safe_username {
        return Err(SaveError::UsernameMismatch);
    }

    // Legacy JSON omitted password_hash via #[serde(skip)], so the old password
    // cannot be verified. The UI must explain local ownership implications and
    // obtain explicit consent before migrating it. An empty value opts out of
    // password protection on this computer.
    state.player.password_hash = if new_password.is_empty() {
        String::new()
    } else {
        hash_password(new_password)?
    };
    state.player.cap_values();
    for companion in &mut state.companions {
        companion.cap_values();
    }
    save_to_path(&state, path)?;
    Ok(state)
}

pub fn migrate_legacy_save(username: &str, new_password: &str) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    migrate_legacy_at(&path, &safe_username, new_password)
}

pub fn create_account(username: &str, password: &str) -> Result<GameState, SaveError> {
    create_account_with_class(username, password, CharacterClass::Warrior)
}

/// Create the initial account checkpoint with the selected class. The legacy
/// entry point preserves its Warrior default for existing callers.
pub fn create_account_with_class(
    username: &str,
    password: &str,
    class: CharacterClass,
) -> Result<GameState, SaveError> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err(SaveError::InvalidUsername);
    }
    let path = get_save_dir()?.join(format!("{safe_username}.json"));
    if path.exists() {
        return Err(SaveError::AccountExists);
    }

    let password_hash = hash_password(password)?;
    let player = Player::new_with_class(safe_username, password_hash, class);
    let state = GameState::new(player);
    save_to_path(&state, &path)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Player;

    fn test_dir(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rustyrealm-save-tests-{}-{}-{}",
            std::process::id(),
            now,
            label
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn local_account_listing_includes_valid_saves_and_ignores_backups_and_corrupt_files() {
        let directory = test_dir("account-list");
        let valid_path = directory.join("zeta.json");
        let backup_path = directory.join("zeta.bak");
        let corrupt_path = directory.join("corrupt.json");
        save_to_path(
            &GameState::new(Player::new("zeta".into(), String::new())),
            &valid_path,
        )
        .unwrap();
        fs::write(&backup_path, b"backup").unwrap();
        fs::write(&corrupt_path, b"not json").unwrap();

        let valid = list_accounts_in(&directory).unwrap();
        assert_eq!(valid, vec!["zeta".to_string()]);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn passwordless_local_accounts_can_be_saved_and_loaded() {
        let directory = test_dir("passwordless");
        let path = directory.join("localhero.json");
        let state = GameState::new(Player::new("localhero".into(), String::new()));
        save_to_path(&state, &path).unwrap();

        assert!(!account_requires_password_at(&path).unwrap());
        let loaded = load_game_at(&path, "localhero", "").unwrap();
        assert!(loaded.player.password_hash.is_empty());
        assert!(load_game_at(&path, "localhero", "unexpected").is_err());

        fs::remove_dir_all(directory).unwrap();
    }

    fn test_state(username: &str, password: &str) -> GameState {
        let hash = hash_password(password).unwrap();
        GameState::new(Player::new(username.to_owned(), hash))
    }

    #[test]
    fn new_game_starts_at_the_starting_village() {
        let state = test_state("newgame", "secret");
        assert_eq!(state.current_location.as_str(), "village_start");
    }

    #[test]
    fn save_load_roundtrip_preserves_state_and_authentication() {
        let dir = test_dir("roundtrip");
        let path = dir.join("player.json");
        let mut state = test_state("player", "secret");
        state.player.gold = 4321;
        state.player.class = CharacterClass::Rogue;
        state.player.attributes = PrimaryAttributes {
            strength: 8,
            dexterity: 14,
            constitution: 11,
            intelligence: 12,
            wisdom: 9,
            charisma: 16,
        };
        state.player.attribute_growth_accumulator = [10, -5, 7, -3, 2, -11];
        state.world_turn = 17;
        state.player.add_item(crate::data::ItemType::Logs, 7);
        state.current_location = crate::world::LocationId::from("whispering_woods");

        save_to_path(&state, &path).unwrap();
        let loaded = load_game_at(&path, "player", "secret").unwrap();
        assert_eq!(loaded.player.gold, 4321);
        assert_eq!(loaded.player.class, CharacterClass::Rogue);
        assert_eq!(loaded.player.attributes, state.player.attributes);
        assert_eq!(
            loaded.player.attribute_growth_accumulator,
            state.player.attribute_growth_accumulator
        );
        let mut original_growth = state.clone();
        let mut reloaded_growth = loaded.clone();
        original_growth.player.level_up_combat();
        reloaded_growth.player.level_up_combat();
        assert_eq!(
            reloaded_growth.player.attributes,
            original_growth.player.attributes
        );
        assert_eq!(
            reloaded_growth.player.attribute_growth_accumulator,
            original_growth.player.attribute_growth_accumulator
        );
        assert_eq!(loaded.world_turn, 17);
        assert_eq!(loaded.current_location.as_str(), "whispering_woods");
        assert_eq!(loaded.player.get_item_count(crate::data::ItemType::Logs), 7);
        assert!(!loaded.player.password_hash.is_empty());
        assert!(load_game_at(&path, "player", "wrong").is_err());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn combat_level_up_advances_primary_stats_once_and_deterministically() {
        let mut first = Player::new("growth".into(), "hash".into());
        let mut second = first.clone();

        first.level_up_combat();
        second.level_up_combat();

        assert_eq!(first.combat_level, 2);
        assert_eq!(first.attributes.total(), 61);
        assert_eq!(first.attributes, second.attributes);
        assert_eq!(
            first.attribute_growth_accumulator,
            second.attribute_growth_accumulator
        );
    }

    #[test]
    fn constitution_growth_increases_health_and_endurance_capacity() {
        let mut player = Player::new("constitution".into(), "hash".into());
        player.attributes.strength = PrimaryAttributes::MAX;

        player.level_up_combat();

        assert_eq!(player.attributes.constitution, 11);
        assert_eq!(player.max_hp, 130);
        assert_eq!(player.hp, 130);
        assert_eq!(player.max_stamina, 102);
        assert_eq!(player.stamina, 102);
    }

    #[test]
    fn dropping_unsaved_live_progress_keeps_the_last_checkpoint_for_restart() {
        let dir = test_dir("discard-checkpoint");
        let path = dir.join("player.json");
        let saved = test_state("discard", "secret");
        save_to_path(&saved, &path).unwrap();

        {
            let mut live_session = saved.clone();
            live_session.player.gold = live_session.player.gold.saturating_add(900);
            live_session.world_turn = live_session.world_turn.saturating_add(12);
            assert_ne!(
                checkpoint_fingerprint(&live_session).unwrap(),
                checkpoint_fingerprint(&saved).unwrap()
            );
            // Discard is represented by dropping this live session without a
            // save call, exactly as the logout flow does.
        }

        let restarted = load_game_at(&path, "discard", "secret").unwrap();
        assert_eq!(restarted.player.gold, saved.player.gold);
        assert_eq!(restarted.world_turn, saved.world_turn);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn invalid_password_hash_does_not_panic() {
        assert!(verify_password("anything", "").is_err());
        assert!(verify_password("anything", "not-a-password-hash").is_err());
    }

    #[test]
    fn legacy_save_requires_explicit_password_migration() {
        let dir = test_dir("legacy");
        let path = dir.join("legacy.json");
        let mut legacy_state =
            GameState::new(Player::new("legacy".into(), "discarded-hash".into()));
        legacy_state.player.xp = 12_345;
        legacy_state.player.gold = 4_321;
        legacy_state.player.add_item(crate::data::ItemType::Logs, 9);
        legacy_state
            .player
            .quests_completed
            .push("historic_quest".into());
        legacy_state
            .unlocked_achievements
            .push("first_steps".into());
        legacy_state.unlocked_themes.push("Amber".into());
        legacy_state.active_theme = "Amber".into();

        // Simulate a pre-Section-9.2 JSON save that has no current_location.
        let mut legacy_json = serde_json::to_value(&legacy_state).unwrap();
        legacy_json
            .as_object_mut()
            .unwrap()
            .remove("current_location");
        fs::write(&path, serde_json::to_vec_pretty(&legacy_json).unwrap()).unwrap();

        assert!(matches!(
            load_game_at(&path, "legacy", "old-password"),
            Err(SaveError::PasswordMigrationRequired)
        ));

        let migrated = migrate_legacy_at(&path, "legacy", "new-password").unwrap();
        assert_eq!(migrated.player.username, "legacy");
        assert_eq!(migrated.current_location.as_str(), "village_start");
        assert_eq!(migrated.player.xp, 12_345);
        assert_eq!(migrated.player.gold, 4_321);
        assert_eq!(
            migrated.player.get_item_count(crate::data::ItemType::Logs),
            9
        );
        assert_eq!(
            migrated.player.quests_completed,
            vec!["historic_quest".to_string()]
        );
        assert!(migrated
            .unlocked_achievements
            .contains(&"first_steps".to_string()));
        assert!(migrated.unlocked_themes.contains(&"Amber".to_string()));
        assert_eq!(migrated.active_theme, "Amber");
        assert!(load_game_at(&path, "legacy", "new-password").is_ok());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn checksum_detects_payload_changes() {
        let dir = test_dir("checksum");
        let path = dir.join("player.json");
        let state = test_state("checksum", "secret");
        save_to_path(&state, &path).unwrap();

        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["data"]["player"]["hp"] = serde_json::json!(17);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        assert!(matches!(
            load_game_at(&path, "checksum", "secret"),
            Err(SaveError::ChecksumMismatch { .. })
        ));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_active_save_restores_only_from_valid_backup() {
        let dir = test_dir("backup");
        let path = dir.join("player.json");
        let mut state = test_state("backup", "secret");
        state.player.gold = 100;
        save_to_path(&state, &path).unwrap();

        state.player.gold = 500;
        save_to_path(&state, &path).unwrap();
        assert!(backup_path(&path).exists());

        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["data"]["player"]["gold"] = serde_json::json!(9999);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        let loaded = load_game_at(&path, "backup", "secret").unwrap();
        assert_eq!(loaded.player.gold, 100);
        assert!(read_save(&path).is_ok());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn validation_clamps_bad_values_without_discarding_the_character() {
        let dir = test_dir("validation");
        let path = dir.join("player.json");
        let mut state = test_state("validated", "secret");
        state.player.max_hp = 1200;
        state.player.hp = u32::MAX;
        state.player.max_stamina = 0;
        state.player.stamina = u32::MAX;
        state.player.max_mana = 0;
        state.player.mana = u32::MAX;
        state.player.gold = u32::MAX;
        state.player.bank_accounts[0].gold = u32::MAX;
        state.player.skills.woodcutting.level = 250;

        save_to_path(&state, &path).unwrap();
        let loaded = load_game_at(&path, "validated", "secret").unwrap();

        assert_eq!(loaded.player.max_hp, 1000);
        assert_eq!(loaded.player.hp, 1000);
        assert_eq!(loaded.player.max_stamina, 100);
        assert_eq!(loaded.player.stamina, 100);
        assert_eq!(loaded.player.max_mana, 50);
        assert_eq!(loaded.player.mana, 50);
        assert_eq!(loaded.player.gold, crate::data::MAX_GOLD);
        assert_eq!(
            loaded.player.bank_accounts[0].gold,
            crate::data::MAX_BANK_GOLD
        );
        assert_eq!(
            loaded.player.skills.woodcutting.level,
            crate::data::MAX_SKILL_LEVEL
        );
        assert_eq!(loaded.player.skills.woodcutting.xp_remaining, 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_load_roundtrip_preserves_quest_progress_and_world_event() {
        let dir = test_dir("quest-persistence");
        let path = dir.join("player.json");
        let mut state = test_state("questpersist", "secret");
        crate::quests::notify_event(
            &mut state,
            &crate::event::GameEvent::MonsterDefeated {
                name: "Goblin".into(),
                is_stronghold: false,
            },
        );
        state.player.quests_completed.push("historic_quest".into());
        state.active_world_event =
            crate::event::WorldEvent::generate_deterministic(1, 1_800_000_000);

        save_to_path(&state, &path).unwrap();
        let loaded = load_game_at(&path, "questpersist", "secret").unwrap();

        assert_eq!(loaded.player.lifetime_kills, 1);
        assert_eq!(
            loaded.player.quests_completed,
            state.player.quests_completed
        );
        assert_eq!(loaded.active_quests, state.active_quests);
        assert_eq!(loaded.active_world_event, state.active_world_event);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn version_two_saves_without_new_fields_keep_valid_checksums() {
        let dir = test_dir("quest-fields-compat");
        let path = dir.join("player.json");
        let state = test_state("questcompat", "secret");
        save_to_path(&state, &path).unwrap();

        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("current_location");
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("active_quests");
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("active_world_event");
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("world_turn");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("lifetime_kills");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("class");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("attributes");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("attribute_growth_accumulator");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("stamina");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("max_stamina");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("mana");
        document["data"]["player"]
            .as_object_mut()
            .unwrap()
            .remove("max_mana");
        let checksum = compute_checksum_payload(&document["data"]).unwrap();
        document["checksum"] = serde_json::json!(checksum);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        let loaded = load_game_at(&path, "questcompat", "secret").unwrap();
        assert!(loaded.active_quests.is_empty());
        assert_eq!(loaded.player.lifetime_kills, 0);
        assert_eq!(loaded.player.class, CharacterClass::Warrior);
        assert_eq!(loaded.player.attributes, PrimaryAttributes::default());
        assert_eq!(loaded.player.attribute_growth_accumulator, [0; 6]);
        assert_eq!(loaded.player.stamina, 100);
        assert_eq!(loaded.player.max_stamina, 100);
        assert_eq!(loaded.world_turn, 0);
        assert_eq!(loaded.player.mana, 50);
        assert_eq!(loaded.player.max_mana, 50);
        assert_eq!(loaded.current_location.as_str(), "village_start");
        assert_eq!(
            loaded.active_world_event.event_type,
            crate::event::WorldEventType::None
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn unsupported_future_version_is_not_replaced_by_an_older_backup() {
        let dir = test_dir("future-version");
        let path = dir.join("player.json");
        let state = test_state("future", "secret");
        save_to_path(&state, &path).unwrap();

        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["version"] = serde_json::json!(CURRENT_SAVE_VERSION + 1);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        assert!(matches!(
            load_game_at(&path, "future", "secret"),
            Err(SaveError::UnsupportedVersion(_))
        ));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_corrupt_active_file_does_not_overwrite_the_last_good_backup() {
        let dir = test_dir("preserve-backup");
        let path = dir.join("player.json");
        let mut state = test_state("preserve", "secret");
        state.player.gold = 123;
        save_to_path(&state, &path).unwrap();

        state.player.gold = 456;
        save_to_path(&state, &path).unwrap();

        fs::write(&path, "truncated and invalid JSON").unwrap();

        // Saving a valid in-memory state can recover the active file, but the
        // invalid active bytes must never replace the known-good backup.
        state.player.gold = 789;
        save_to_path(&state, &path).unwrap();

        let backed_up = match read_save(&backup_path(&path)).unwrap() {
            ParsedSave::Current(envelope) => envelope.data,
            ParsedSave::Legacy(_) => panic!("expected a versioned backup"),
        };
        assert_eq!(backed_up.player.gold, 123);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_save_without_authentication_leaves_existing_file_untouched() {
        let dir = test_dir("failed-write");
        let path = dir.join("player.json");
        let valid = test_state("failed", "secret");
        save_to_path(&valid, &path).unwrap();
        let before = fs::read(&path).unwrap();

        let mut changed = valid.clone();
        changed.player.gold += 1;
        assert!(save_to_path(&changed, &path).is_ok());
        assert_ne!(fs::read(&path).unwrap(), before);

        let passwordless = GameState::new(Player::new("passwordless".into(), String::new()));
        let passwordless_path = dir.join("passwordless.json");
        assert!(save_to_path(&passwordless, &passwordless_path).is_ok());
        assert!(load_game_at(&passwordless_path, "passwordless", "").is_ok());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_load_roundtrip_preserves_companions_and_party() {
        let dir = test_dir("companion-roundtrip");
        let path = dir.join("player.json");
        let mut state = test_state("companion", "secret");

        // Recruit a companion and add to party
        let cost = crate::party::PartyManager::recruit(
            &mut state.companions,
            crate::party::CompanionId::BranTheShield,
            state.player.combat_level,
            state.player.gold,
        )
        .unwrap();
        state.player.gold -= cost;
        crate::party::PartyManager::add_to_party(
            &mut state.party,
            &state.companions,
            crate::party::CompanionId::BranTheShield,
        )
        .unwrap();

        save_to_path(&state, &path).unwrap();
        let loaded = load_game_at(&path, "companion", "secret").unwrap();

        assert_eq!(loaded.companions.len(), 1);
        assert_eq!(
            loaded.companions[0].id,
            crate::party::CompanionId::BranTheShield
        );
        assert!(loaded
            .party
            .contains(crate::party::CompanionId::BranTheShield));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn old_saves_without_companion_fields_load_with_defaults() {
        let dir = test_dir("companion-compat");
        let path = dir.join("player.json");
        let state = test_state("compat", "secret");
        save_to_path(&state, &path).unwrap();

        // Remove companion, party and region progress fields to simulate an older save.
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("companions");
        document["data"].as_object_mut().unwrap().remove("party");
        document["data"]
            .as_object_mut()
            .unwrap()
            .remove("region_progress");
        let checksum = compute_checksum_payload(&document["data"]).unwrap();
        document["checksum"] = serde_json::json!(checksum);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        let loaded = load_game_at(&path, "compat", "secret").unwrap();
        assert!(loaded.companions.is_empty());
        assert_eq!(loaded.party.companion_count(), 0);
        assert_eq!(
            loaded.party.size_limit(),
            crate::party::DEFAULT_PARTY_SIZE_LIMIT
        );
        assert!(loaded.region_progress.cleared_encounters.is_empty());
        assert!(loaded.region_progress.landmark_interactions.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_load_roundtrip_preserves_region_progress_and_quest_stage_state() {
        let dir = test_dir("region-progress");
        let path = dir.join("region.json");
        let mut state = test_state("region-progress", "secret");
        crate::adventure::ensure_region_quest(&mut state);
        state.current_location = crate::world::LocationId::from("sunken_ruins");
        state
            .region_progress
            .cleared_encounters
            .push("woods_patrol".into());
        state
            .region_progress
            .landmark_interactions
            .push("standing_stones_survey".into());
        crate::quests::notify_event(
            &mut state,
            &crate::event::GameEvent::LocationReached {
                location_id: crate::world::LocationId::from("standing_stones"),
            },
        );
        crate::quests::notify_event(
            &mut state,
            &crate::event::GameEvent::TargetInteracted {
                target_id: "standing_stones_survey".into(),
            },
        );

        save_to_path(&state, &path).unwrap();
        let loaded = load_game_at(&path, "region-progress", "secret").unwrap();
        assert_eq!(loaded.current_location.as_str(), "sunken_ruins");
        assert_eq!(
            loaded.region_progress.cleared_encounters,
            vec!["woods_patrol"]
        );
        assert_eq!(
            loaded.region_progress.landmark_interactions,
            vec!["standing_stones_survey"]
        );
        let quest = loaded
            .active_quests
            .iter()
            .find(|q| q.id == crate::adventure::REGION_QUEST_ID)
            .unwrap();
        assert_eq!(quest.stages[0].objectives[0].progress, 1);
        assert!(quest.stages[0].objectives[0].is_completed);
        assert!(!quest.stages[1].is_completed);

        let _ = fs::remove_dir_all(dir);
    }
}
