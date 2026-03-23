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
            .map_err(|keyring_err| match keyring_err {
                keyring::Error::NoEntry => Box::new(StorageError::Read),
                _ => Box::new(StorageError::Unknown) as Box<dyn std::error::Error + Send>,
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
        match self.keyring.delete_password() {
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

pub fn get_storage() -> Box<dyn CredentialsStorage> {
    if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
        return Box::new(EnvironmentStorage::new(api_token));
    }

    let keyring = Entry::new("togglcli", "default")
        .unwrap_or_else(|err| panic!("Couldn't create credentials_storage: {err}"));
    Box::new(KeyringStorage::new(keyring))
}
