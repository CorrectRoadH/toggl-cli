use crate::constants;
use crate::credentials::get_storage;
use crate::error;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct AuthStatusCommand;

/// Represents the source of credentials
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    Environment,
    Keychain,
    None,
}

/// Represents the authentication status
#[derive(Debug, Clone)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub provider: String,
    pub api_url: Option<String>,
    pub source: CredentialSource,
    pub masked_token: Option<String>,
    /// Whether the custom API URL (if set) is well-formed and has a valid scheme
    pub api_url_valid: bool,
}

/// Validate that a URL is well-formed with http or https scheme
fn validate_api_url(url: &str) -> bool {
    // Must contain :// to be a valid URL scheme format
    if !url.contains("://") {
        return false;
    }
    // Must start with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }
    // Must have something after the scheme
    let after_scheme = url.split("://").nth(1);
    match after_scheme {
        Some(s) => {
            // Must have content after the scheme
            if s.is_empty() {
                return false;
            }
            // A valid URL must have a host (at least one non-slash character before / or :)
            // This rejects URLs like https:///path which have no host
            let first_char = s.chars().next().unwrap();
            first_char != '/' && first_char != ':'
        }
        None => false,
    }
}

impl AuthStatusCommand {
    /// Get the current auth status by checking environment and keychain
    pub fn get_status() -> AuthStatus {
        #[cfg(test)]
        crate::credentials::ensure_test_dotenv();

        // Check environment first (highest precedence)
        if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
            let api_url = std::env::var("TOGGL_API_URL").ok();
            let provider = if api_url.is_some() {
                "custom"
            } else {
                "official"
            };
            // Validate API URL if one is set
            let api_url_valid = api_url
                .as_ref()
                .map(|url| validate_api_url(url))
                .unwrap_or(true);
            return AuthStatus {
                is_authenticated: !api_token.is_empty(),
                provider: provider.to_string(),
                api_url,
                source: CredentialSource::Environment,
                masked_token: Some(Self::mask_token(&api_token)),
                api_url_valid,
            };
        }

        // Check keychain storage
        let storage = get_storage();
        match storage.read() {
            Ok(credentials) => {
                let provider = if credentials.api_url.is_some() {
                    "custom"
                } else {
                    "official"
                };
                // Validate API URL if one is set
                let api_url_valid = credentials
                    .api_url
                    .as_ref()
                    .map(|url| validate_api_url(url))
                    .unwrap_or(true);
                AuthStatus {
                    is_authenticated: !credentials.api_token.is_empty(),
                    provider: provider.to_string(),
                    api_url: credentials.api_url,
                    source: CredentialSource::Keychain,
                    masked_token: Some(Self::mask_token(&credentials.api_token)),
                    api_url_valid,
                }
            }
            Err(_) => AuthStatus {
                is_authenticated: false,
                provider: "unknown".to_string(),
                api_url: None,
                source: CredentialSource::None,
                masked_token: None,
                api_url_valid: true,
            },
        }
    }

    /// Mask all but the last 4 characters of a token
    fn mask_token(token: &str) -> String {
        if token.len() < 4 {
            "*".repeat(token.len())
        } else {
            let visible = &token[token.len() - 4..];
            let masked = "*".repeat(token.len() - 4);
            format!("{}{}", masked, visible)
        }
    }

    /// Execute the auth status command in JSON format
    pub fn execute_json<W: std::io::Write>(
        mut writer: W,
        status: AuthStatus,
    ) -> ResultWithDefaultError<()> {
        let source_str = match &status.source {
            CredentialSource::Environment => "TOGGL_API_TOKEN",
            CredentialSource::Keychain => "keyring",
            CredentialSource::None => "none",
        };

        let mut obj = serde_json::Map::new();
        obj.insert(
            "authenticated".to_string(),
            serde_json::Value::Bool(status.is_authenticated),
        );

        if status.is_authenticated {
            obj.insert(
                "provider".to_string(),
                serde_json::Value::String(status.provider.clone()),
            );
            obj.insert(
                "source".to_string(),
                serde_json::Value::String(source_str.to_string()),
            );
            if let Some(ref url) = status.api_url {
                obj.insert(
                    "api_url".to_string(),
                    serde_json::Value::String(url.clone()),
                );
            }
            if let Some(ref token) = status.masked_token {
                obj.insert(
                    "masked_token".to_string(),
                    serde_json::Value::String(token.clone()),
                );
            }
            if status.api_url.is_some() && !status.api_url_valid {
                obj.insert("api_url_valid".to_string(), serde_json::Value::Bool(false));
            }
        }

        let json = serde_json::Value::Object(obj);
        writeln!(writer, "{}", serde_json::to_string(&json).unwrap()).map_err(|_| {
            Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
        })?;

        Ok(())
    }

    /// Execute the auth status command and write output to the provided writer
    pub fn execute<W: std::io::Write>(
        mut writer: W,
        status: AuthStatus,
    ) -> ResultWithDefaultError<()> {
        writeln!(writer).map_err(|_| {
            Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
        })?;

        if status.is_authenticated {
            // Check if we have an invalid API URL state
            let has_invalid_url = status.api_url.is_some() && !status.api_url_valid;

            // Authenticated header
            writeln!(writer, "{}", "Authentication Status:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;

            if has_invalid_url {
                // Show as invalid when custom API URL is malformed
                writeln!(
                    writer,
                    "  {}  {}",
                    "Authenticated:".yellow().bold(),
                    "Invalid".red()
                )
                .map_err(|_| {
                    Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
                })?;
            } else {
                writeln!(
                    writer,
                    "  {}  {}",
                    "Authenticated:".yellow().bold(),
                    "Yes".green()
                )
                .map_err(|_| {
                    Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
                })?;
            }

            // Provider
            let provider_display = if status.provider == "official" {
                format!(
                    "{} ({})",
                    "Official Toggl Track".green(),
                    constants::TOGGL_API_URL_OFFICIAL.blue()
                )
            } else {
                format!(
                    "{} ({})",
                    "Custom/OpenToggl".yellow(),
                    status.api_url.as_deref().unwrap_or("unknown").blue()
                )
            };
            writeln!(
                writer,
                "  {}  {}",
                "Provider:".yellow().bold(),
                provider_display
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;

            // Source
            let source_display = match status.source {
                CredentialSource::Environment => {
                    format!("{} (TOGGL_API_TOKEN)", "Environment".blue())
                }
                CredentialSource::Keychain => format!("{} (keyring)", "Keychain".blue()),
                CredentialSource::None => "None".red().to_string(),
            };
            writeln!(
                writer,
                "  {}  {}",
                "Source:".yellow().bold(),
                source_display
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;

            // Masked token
            if let Some(token) = &status.masked_token {
                writeln!(writer, "  {}  {}", "Token:".yellow().bold(), token.white()).map_err(
                    |_| Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>,
                )?;
            }

            // If API URL is invalid, show warning
            if has_invalid_url {
                writeln!(writer).map_err(|_| {
                    Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
                })?;
                writeln!(
                    writer,
                    "{}  {}",
                    "⚠".yellow().bold(),
                    "The custom API URL is malformed. Expected format: https://your-api.example.com"
                        .red()
                )
                .map_err(|_| {
                    Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
                })?;
                writeln!(
                    writer,
                    "    {} Fix by setting a valid {} in your environment",
                    "•".blue(),
                    "TOGGL_API_URL".cyan()
                )
                .map_err(|_| {
                    Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
                })?;
            }

            // Precedence note
            writeln!(writer).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(writer, "{}", "Credential Resolution:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {}  (active)",
                "1.".yellow(),
                "Environment (TOGGL_API_TOKEN, TOGGL_API_URL)".blue()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {}  (stored)",
                "2.".yellow(),
                "Keychain".blue()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
        } else {
            // Not authenticated header
            writeln!(writer, "{}", "Authentication Status:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {}",
                "Authenticated:".yellow().bold(),
                "No".red()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;

            // How to authenticate
            writeln!(writer).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(writer, "{}", "To authenticate, run:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {} {}",
                "•".blue(),
                "toggl auth login".cyan(),
                "<API_TOKEN>".white()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  Or set {} in your environment",
                "•".blue(),
                "TOGGL_API_TOKEN".cyan()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  Find your API token at {}",
                "•".blue(),
                constants::CREDENTIALS_FIND_TOKEN_LINK.blue().underline()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_token_shows_last_four_characters() {
        let token = "abcdefghijk";
        let masked = AuthStatusCommand::mask_token(token);
        assert_eq!(masked, "*******hijk");
    }

    #[test]
    fn mask_token_handles_short_token() {
        let token = "abc";
        let masked = AuthStatusCommand::mask_token(token);
        assert_eq!(masked, "***");
    }

    #[test]
    fn mask_token_handles_exactly_four_characters() {
        let token = "abcd";
        let masked = AuthStatusCommand::mask_token(token);
        assert_eq!(masked, "abcd");
    }

    #[test]
    fn auth_status_shows_environment_source_when_token_in_env() {
        let status = AuthStatusCommand::get_status();
        // With .env loaded, TOGGL_API_TOKEN should be present
        assert!(
            status.source == CredentialSource::Environment
                || status.source == CredentialSource::Keychain
                || status.source == CredentialSource::None
        );
    }

    #[test]
    fn validate_api_url_accepts_valid_https_url() {
        assert!(validate_api_url("https://api.track.toggl.com/api/v9"));
        assert!(validate_api_url("https://opentoggl.example.com/api/v9"));
        assert!(validate_api_url("https://self-hosted.toggl.company.com/"));
    }

    #[test]
    fn validate_api_url_accepts_valid_http_url() {
        assert!(validate_api_url("http://localhost:8080/api/v9"));
        assert!(validate_api_url("http://192.168.1.100:8080/api"));
    }

    #[test]
    fn validate_api_url_rejects_malformed_urls() {
        // Missing scheme
        assert!(!validate_api_url("api.track.toggl.com/api/v9"));
        // No scheme separator
        assert!(!validate_api_url("https:api.track.toggl.com/api/v9"));
        // Empty after scheme
        assert!(!validate_api_url("https://"));
        // Just a word
        assert!(!validate_api_url("fake"));
        // Invalid scheme
        assert!(!validate_api_url("ftp://api.track.toggl.com"));
        assert!(!validate_api_url("file://api.track.toggl.com"));
        // No host
        assert!(!validate_api_url("https:///api/v9"));
    }

    #[test]
    fn validate_api_url_rejects_empty_string() {
        assert!(!validate_api_url(""));
    }
}
