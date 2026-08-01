use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("User with ID '{0}' was not found")]
    UserNotFound(String),

    #[error("Factorial input '{0}' exceeds maximum supported limit of 20")]
    FactorialOverflow(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub version: &'static str,
    pub core_engine: &'static str,
}

/// Domain Service containing pure business logic (independent of mobile/FFI)
pub struct AppService {
    users: Mutex<HashMap<String, User>>,
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
        }
    }

    pub fn get_version(&self) -> AppVersion {
        AppVersion {
            version: "1.0.0",
            core_engine: "Pure Rust AppCore v1.0",
        }
    }

    pub fn calculate_factorial(&self, n: u64) -> Result<u64, CoreError> {
        if n > 20 {
            return Err(CoreError::FactorialOverflow(n));
        }
        let mut result = 1;
        for i in 1..=n {
            result *= i;
        }
        Ok(result)
    }

    pub fn get_user(&self, id: &str) -> Result<User, CoreError> {
        let guard = self.users.lock().unwrap();
        guard
            .get(id)
            .cloned()
            .ok_or_else(|| CoreError::UserNotFound(id.to_string()))
    }

    pub fn save_user(&self, id: String, name: String, role: String) -> User {
        let user = User { id: id.clone(), name, role };
        let mut guard = self.users.lock().unwrap();
        guard.insert(id, user.clone());
        user
    }
}
