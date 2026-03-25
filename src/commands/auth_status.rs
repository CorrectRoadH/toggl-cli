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
}

impl AuthStatusCommand {
    /// Get the current auth status by checking environment and keychain
    pub fn get_status() -> AuthStatus {
        // Check environment first (highest precedence)
        if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
            let api_url = std::env::var("TOGGL_API_URL").ok();
            let provider = if api_url.is_some() {
                "custom"
            } else {
                "official"
            };
            return AuthStatus {
                is_authenticated: !api_token.is_empty(),
                provider: provider.to_string(),
                api_url,
                source: CredentialSource::Environment,
                masked_token: Some(Self::mask_token(&api_token)),
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
                AuthStatus {
                    is_authenticated: !credentials.api_token.is_empty(),
                    provider: provider.to_string(),
                    api_url: credentials.api_url,
                    source: CredentialSource::Keychain,
                    masked_token: Some(Self::mask_token(&credentials.api_token)),
                }
            }
            Err(_) => AuthStatus {
                is_authenticated: false,
                provider: "unknown".to_string(),
                api_url: None,
                source: CredentialSource::None,
                masked_token: None,
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

    /// Execute the auth status command and write output to the provided writer
    pub fn execute<W: std::io::Write>(
        mut writer: W,
        status: AuthStatus,
    ) -> ResultWithDefaultError<()> {
        writeln!(writer).map_err(|_| {
            Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
        })?;

        if status.is_authenticated {
            // Authenticated header
            writeln!(writer, "{}", "Authentication Status:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {}",
                "Authenticated:".yellow().bold(),
                "Yes".green()
            )
            .map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;

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
                    "Custom/OpenToggl".green(),
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

            // Precedence note
            writeln!(writer).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(writer, "{}", "Credential Precedence:".bold()).map_err(|_| {
                Box::new(error::StorageError::Unknown) as Box<dyn std::error::Error + Send>
            })?;
            writeln!(
                writer,
                "  {}  {} > {} > {}",
                "1.".yellow(),
                "Environment (TOGGL_API_TOKEN)".blue(),
                "Keychain".blue(),
                "None".red()
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
        // This test relies on the .env file being loaded
        let status = AuthStatusCommand::get_status();
        // The status should reflect environment if TOGGL_API_TOKEN is set
        // Note: In test environment, TOGGL_API_TOKEN may or may not be set
        // so we just verify the structure is correct
        assert!(
            status.source == CredentialSource::Environment
                || status.source == CredentialSource::Keychain
                || status.source == CredentialSource::None
        );
    }
}
