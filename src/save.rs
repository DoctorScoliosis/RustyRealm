use std::fs;
use std::path::PathBuf;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::state::{GameState, Player};

pub fn get_save_dir() -> PathBuf {
    let proj_dirs = directories::ProjectDirs::from("com", "rustyrealm", "RustyRealm")
        .expect("Failed to get project directories");
    let save_dir = proj_dirs.data_dir().join("saves");
    fs::create_dir_all(&save_dir).expect("Failed to create save directory");
    save_dir
}

pub fn sanitize_username(username: &str) -> String {
    username.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>().to_lowercase()
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    password_hash
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(password_hash).expect("Failed to parse password hash");
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn save_game(state: &GameState) -> Result<(), String> {
    let save_dir = get_save_dir();
    let file_path = save_dir.join(format!("{}.json", state.player.username));
    
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(file_path, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn load_game(username: &str, password: &str) -> Result<GameState, String> {
    let safe_username = sanitize_username(username);
    let save_dir = get_save_dir();
    let file_path = save_dir.join(format!("{}.json", safe_username));
    
    if !file_path.exists() {
        return Err("Invalid username.".to_string());
    }
    
    let json = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let mut state: GameState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    
    if !verify_password(password, &state.player.password_hash) {
        return Err("Invalid password.".to_string());
    }
    
    // Re-cap values on load to prevent save file tampering
    state.player.cap_values();
    
    Ok(state)
}

pub fn create_account(username: &str, password: &str) -> Result<GameState, String> {
    let safe_username = sanitize_username(username);
    if safe_username.is_empty() {
        return Err("Username must contain alphanumeric characters.".to_string());
    }
    
    let save_dir = get_save_dir();
    let file_path = save_dir.join(format!("{}.json", safe_username));
    
    if file_path.exists() {
        return Err("Username already exists.".to_string());
    }
    
    let password_hash = hash_password(password);
    let player = Player::new(safe_username, password_hash);
    let state = GameState::new(player);
    
    save_game(&state)?;
    
    Ok(state)
}