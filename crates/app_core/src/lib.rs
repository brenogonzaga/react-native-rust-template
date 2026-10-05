use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("User with ID '{0}' was not found")]
    UserNotFound(String),

    #[error("Factorial input '{0}' exceeds maximum supported limit of 20")]
    FactorialOverflow(u64),

    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        CoreError::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        CoreError::Storage(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AppVersion {
    pub version: &'static str,
    pub core_engine: &'static str,
}

/// Domain Service containing pure business logic (independent of mobile/FFI)
pub struct AppService {
    users: Mutex<HashMap<String, User>>,
    users_file: Mutex<Option<PathBuf>>,
}

impl Default for AppService {
    fn default() -> Self {
        Self::new()
    }
}

impl AppService {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(
            "1".to_string(),
            User {
                id: "1".to_string(),
                name: "Alice Core".to_string(),
                role: "Administrator".to_string(),
            },
        );
        Self {
            users: Mutex::new(users),
            users_file: Mutex::new(None),
        }
    }

    pub fn get_version(&self) -> AppVersion {
        AppVersion {
            version: env!("CARGO_PKG_VERSION"),
            core_engine: concat!("Pure Rust AppCore v", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn calculate_factorial(&self, n: u64) -> Result<u64, CoreError> {
        if n > 20 {
            tracing::warn!(n, "factorial input exceeds the supported limit");
            return Err(CoreError::FactorialOverflow(n));
        }
        let mut result = 1;
        for i in 1..=n {
            result *= i;
        }
        Ok(result)
    }

    pub fn get_user(&self, id: &str) -> Result<User, CoreError> {
        let guard = self.users.lock().unwrap_or_else(PoisonError::into_inner);
        guard
            .get(id)
            .cloned()
            .ok_or_else(|| CoreError::UserNotFound(id.to_string()))
    }

    /// Loads `users.json` from `dir` (creating `dir` if needed) and persists
    /// every later `save_user` there. Calling it again reloads from disk.
    pub fn open_storage(&self, dir: &Path) -> Result<(), CoreError> {
        fs::create_dir_all(dir)?;
        let file = dir.join("users.json");
        if file.exists() {
            let saved: HashMap<String, User> = serde_json::from_slice(&fs::read(&file)?)?;
            *self.users.lock().unwrap_or_else(PoisonError::into_inner) = saved;
        }
        tracing::info!(path = %file.display(), "storage opened");
        *self
            .users_file
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(file);
        Ok(())
    }

    pub fn save_user(&self, id: String, name: String, role: String) -> Result<User, CoreError> {
        let user = User {
            id: id.clone(),
            name,
            role,
        };
        let mut users = self.users.lock().unwrap_or_else(PoisonError::into_inner);
        users.insert(id, user.clone());
        if let Some(file) = &*self
            .users_file
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
        {
            // Write-then-rename: a crash mid-write leaves the previous file
            // intact instead of a truncated one.
            let tmp = file.with_extension("json.tmp");
            fs::write(&tmp, serde_json::to_vec(&*users)?)?;
            fs::rename(&tmp, file)?;
        }
        tracing::info!(id = %user.id, "user saved");
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factorial_computes_known_values() {
        let service = AppService::new();
        assert_eq!(service.calculate_factorial(0).unwrap(), 1);
        assert_eq!(service.calculate_factorial(5).unwrap(), 120);
        assert_eq!(
            service.calculate_factorial(20).unwrap(),
            2432902008176640000
        );
    }

    #[test]
    fn factorial_rejects_overflow() {
        let service = AppService::new();
        assert!(matches!(
            service.calculate_factorial(21),
            Err(CoreError::FactorialOverflow(21))
        ));
    }

    #[test]
    fn get_user_returns_seeded_user() {
        let service = AppService::new();
        let user = service.get_user("1").unwrap();
        assert_eq!(user.name, "Alice Core");
    }

    #[test]
    fn get_user_missing_id_errors() {
        let service = AppService::new();
        assert!(matches!(
            service.get_user("missing"),
            Err(CoreError::UserNotFound(id)) if id == "missing"
        ));
    }

    #[test]
    fn save_user_round_trips_through_get_user() {
        let service = AppService::new();
        let saved = service
            .save_user("2".into(), "Bob".into(), "Engineer".into())
            .unwrap();
        let fetched = service.get_user("2").unwrap();
        assert_eq!(saved.name, fetched.name);
        assert_eq!(fetched.role, "Engineer");
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("app_core-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn saved_users_survive_a_new_service_on_the_same_dir() {
        let dir = temp_dir("persist");
        let first = AppService::new();
        first.open_storage(&dir).unwrap();
        first
            .save_user("7".into(), "Dana".into(), "QA".into())
            .unwrap();

        let second = AppService::new();
        second.open_storage(&dir).unwrap();
        assert_eq!(second.get_user("7").unwrap().name, "Dana");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn corrupt_storage_is_reported_and_never_overwritten() {
        let dir = temp_dir("corrupt");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("users.json");
        fs::write(&file, "not json").unwrap();

        let service = AppService::new();
        assert!(matches!(
            service.open_storage(&dir),
            Err(CoreError::Storage(_))
        ));
        service
            .save_user("8".into(), "Eve".into(), "Ops".into())
            .unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "not json");
        fs::remove_dir_all(&dir).unwrap();
    }
}
