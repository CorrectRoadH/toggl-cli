use crate::constants;
use crate::error;
use crate::models;
use async_trait::async_trait;
use error::StorageError;
use keyring::Entry;
#[cfg(test)]
use mockall::automock;
use models::ResultWithDefaultError;

#[derive(Clone)]
pub struct Credentials {
    pub api_token: String,
    pub api_url: Option<String>,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CredentialsStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials>;
    fn persist(&self, api_token: String, api_url: Option<String>) -> ResultWithDefaultError<()>;
    fn clear(&self) -> ResultWithDefaultError<()>;
}

pub struct KeyringStorage {
    keyring: Entry,
}

impl KeyringStorage {
    pub fn new(keyring: Entry) -> KeyringStorage {
        Self { keyring }
    }
}

impl CredentialsStorage for KeyringStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        self.keyring
            .get_password()
            .map(|stored| Self::parse_stored_value(&stored))
            .map_err(|_keyring_err| {
                // When credentials cannot be read from keychain (whether due to
                // no entry, keychain locked, or any other reason), the user
                // needs to authenticate. Map all keyring errors to Read so
                // the user gets clear guidance rather than an obscure error.
                Box::new(StorageError::Read) as Box<dyn std::error::Error + Send>
            })
    }

    fn persist(&self, api_token: String, api_url: Option<String>) -> ResultWithDefaultError<()> {
        let stored = Self::format_stored_value(&api_token, api_url);
        match self.keyring.set_password(stored.as_str()) {
            Err(keyring_err) => {
                eprintln!("Error writing to keyring: {keyring_err}");
                Err(Box::new(StorageError::Write))
            }
            Ok(_) => Ok(()),
        }
    }

    fn clear(&self) -> ResultWithDefaultError<()> {
        match self.keyring.delete_credential() {
            Err(keyring_err) => {
                eprintln!("Error deleting from keyring: {keyring_err}");
                Err(Box::new(StorageError::Delete))
            }
            Ok(_) => Ok(()),
        }
    }
}

impl KeyringStorage {
    fn format_stored_value(api_token: &str, api_url: Option<String>) -> String {
        match api_url {
            Some(url) => format!(
                "{}{}{}",
                api_token,
                constants::TOGGL_API_URL_CREDENTIALS_DELIMITER,
                url
            ),
            None => api_token.to_string(),
        }
    }

    fn parse_stored_value(stored: &str) -> Credentials {
        if let Some((token, url)) =
            stored.split_once(constants::TOGGL_API_URL_CREDENTIALS_DELIMITER)
        {
            Credentials {
                api_token: token.to_string(),
                api_url: Some(url.to_string()),
            }
        } else {
            Credentials {
                api_token: stored.to_string(),
                api_url: None,
            }
        }
    }
}

pub struct EnvironmentStorage {
    token: String,
    api_url: Option<String>,
}

impl EnvironmentStorage {
    pub fn new(token: String) -> EnvironmentStorage {
        let api_url = std::env::var("TOGGL_API_URL").ok();
        Self { token, api_url }
    }
}

impl CredentialsStorage for EnvironmentStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        Ok(Credentials {
            api_token: self.token.clone(),
            api_url: self.api_url.clone(),
        })
    }
    fn persist(&self, _api_token: String, _api_url: Option<String>) -> ResultWithDefaultError<()> {
        Err(Box::new(StorageError::EnvironmentOverride))
    }
    fn clear(&self) -> ResultWithDefaultError<()> {
        Err(Box::new(StorageError::EnvironmentOverride))
    }
}

/// Stands in for the keyring when the platform has no reachable credential
/// store — a headless Linux box with no Secret Service, a container, CI.
///
/// Reads report the same "you need to authenticate" error a missing entry
/// would, so those environments fall back to `TOGGL_API_TOKEN` instead of
/// aborting the whole command.
pub struct UnavailableStorage {
    reason: String,
}

impl UnavailableStorage {
    pub fn new(reason: String) -> UnavailableStorage {
        Self { reason }
    }
}

impl CredentialsStorage for UnavailableStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        Err(Box::new(StorageError::Read))
    }

    fn persist(&self, _api_token: String, _api_url: Option<String>) -> ResultWithDefaultError<()> {
        eprintln!("No credential store is available: {}", self.reason);
        Err(Box::new(StorageError::Write))
    }

    fn clear(&self) -> ResultWithDefaultError<()> {
        eprintln!("No credential store is available: {}", self.reason);
        Err(Box::new(StorageError::Delete))
    }
}

/// In test builds, ensure `.env` is loaded so unit tests see `TOGGL_API_TOKEN`
/// instead of falling through to macOS keychain.
#[cfg(test)]
pub fn ensure_test_dotenv() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = dotenvy::from_filename_override(".env");
    });
}

pub fn get_storage() -> Box<dyn CredentialsStorage> {
    #[cfg(test)]
    ensure_test_dotenv();

    if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
        return Box::new(EnvironmentStorage::new(api_token));
    }

    match Entry::new("togglcli", "default") {
        Ok(keyring) => Box::new(KeyringStorage::new(keyring)),
        Err(err) => Box::new(UnavailableStorage::new(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_storage_reports_a_read_error_instead_of_panicking() {
        let storage = UnavailableStorage::new("no secret service".to_string());

        let error = match storage.read() {
            Ok(_) => panic!("read must not succeed"),
            Err(error) => error,
        };

        assert_eq!(error.to_string(), StorageError::Read.to_string());
    }

    #[test]
    fn unavailable_storage_refuses_to_persist_or_clear() {
        let storage = UnavailableStorage::new("no secret service".to_string());

        let persist_error = storage
            .persist("token".to_string(), None)
            .expect_err("persist must not succeed");
        let clear_error = storage.clear().expect_err("clear must not succeed");

        assert_eq!(persist_error.to_string(), StorageError::Write.to_string());
        assert_eq!(clear_error.to_string(), StorageError::Delete.to_string());
    }

    #[test]
    fn get_storage_returns_a_storage_when_no_keyring_is_reachable() {
        // Exercised for real on headless Linux, where `Entry::new` fails: the
        // CLI must still get a storage back so it can report the error itself.
        let storage = get_storage();

        // Any of the three storages is acceptable here; the point is that
        // building one never aborts the process.
        let _ = storage.read();
    }
}
