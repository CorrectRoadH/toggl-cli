use crate::constants;
use colored::Colorize;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    #[allow(dead_code)]
    Network,
    NetworkWithMessage(String),
    /// HTTP client/server error (4xx/5xx) that is NOT a connection problem.
    HttpErrorWithMessage(String),
    RateLimitedWithMessage(String),
    OfficialApiUsageLimitWithMessage(String),
    Deserialization,
    DeserializationWithMessage(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            ApiError::Network => format!("{}", constants::NETWORK_ERROR_MESSAGE.red()),
            ApiError::NetworkWithMessage(message) => format!(
                "{}\n{}: {}",
                constants::NETWORK_ERROR_MESSAGE.red(),
                "Details".yellow().bold(),
                message
            ),
            ApiError::HttpErrorWithMessage(message) => {
                format!("{}: {}", "Request failed".red(), message)
            }
            ApiError::RateLimitedWithMessage(message) => format!(
                "{}\n{}: {}",
                constants::RATE_LIMITED_ERROR_MESSAGE.red(),
                "Details".yellow().bold(),
                message
            ),
            ApiError::OfficialApiUsageLimitWithMessage(message) => format!(
                "{}\n{}: {}\n{} {}",
                constants::OFFICIAL_API_USAGE_LIMIT_ERROR_MESSAGE.red(),
                "Details".yellow().bold(),
                message,
                constants::OPENTOGGL_ALTERNATIVE_MESSAGE.blue().bold(),
                constants::OPENTOGGL_LINK.blue().bold().underline()
            ),
            ApiError::Deserialization => format!(
                "{}\n{} {}",
                constants::DESERIALIZATION_ERROR_MESSAGE.red(),
                constants::OUTDATED_APP_ERROR_MESSAGE.blue().bold(),
                constants::ISSUE_LINK.blue().bold().underline()
            ),
            ApiError::DeserializationWithMessage(message) => format!(
                "{}\n{}: {}\n{} {}",
                constants::DESERIALIZATION_ERROR_MESSAGE.red(),
                "Details".yellow().bold(),
                message,
                constants::OUTDATED_APP_ERROR_MESSAGE.blue().bold(),
                constants::ISSUE_LINK.blue().bold().underline()
            ),
        };
        write!(f, "{summary}")
    }
}

impl Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_usage_limit_error_mentions_opentoggl() {
        let rendered = ApiError::OfficialApiUsageLimitWithMessage(
            "HTTP 402 You have hit your hourly limit for API calls.".to_string(),
        )
        .to_string();

        assert!(rendered.contains(constants::OFFICIAL_API_USAGE_LIMIT_ERROR_MESSAGE));
        assert!(rendered.contains(constants::OPENTOGGL_ALTERNATIVE_MESSAGE));
        assert!(rendered.contains(constants::OPENTOGGL_LINK));
    }
}

#[derive(Debug)]
pub enum StorageError {
    Read,
    Write,
    Delete,
    Unknown,
    EnvironmentOverride,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            StorageError::Read => {
                format!(
                    "{}\n{} {}",
                    constants::CREDENTIALS_EMPTY_ERROR.red(),
                    constants::CREDENTIALS_FIND_TOKEN_MESSAGE.blue().bold(),
                    constants::CREDENTIALS_FIND_TOKEN_LINK
                        .blue()
                        .bold()
                        .underline()
                )
            }
            StorageError::Write | StorageError::Delete | StorageError::Unknown => {
                let message = match self {
                    StorageError::Write => constants::CREDENTIALS_WRITE_ERROR.red(),
                    StorageError::Delete => constants::CREDENTIALS_DELETE_ERROR.red(),
                    _ => constants::CREDENTIALS_ACCESS_ERROR.red(),
                };
                format!(
                    "{}\n{} {}",
                    message,
                    constants::OUTDATED_APP_ERROR_MESSAGE.blue().bold(),
                    constants::ISSUE_LINK.blue().bold().underline()
                )
            }
            StorageError::EnvironmentOverride => {
                format!("{}", constants::CREDENTIALS_OVERRIDE_ERROR.red())
            }
        };

        writeln!(f, "{summary}")
    }
}

impl Error for StorageError {}

#[derive(Debug)]
pub enum PickerError {
    Cancelled,
    FzfNotInstalled,
    Generic,
}

impl Display for PickerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}",
            match self {
                PickerError::Cancelled => constants::OPERATION_CANCELLED,
                PickerError::FzfNotInstalled => constants::FZF_NOT_INSTALLED_ERROR,
                PickerError::Generic => constants::GENERIC_ERROR,
            }
            .red(),
        );
        write!(f, "{summary}")
    }
}

impl Error for PickerError {}

#[derive(Debug)]
pub enum ConfigError {
    Parse,
    FileNotFound,
    UnrecognizedMarco(String),
    ShellResolution(String, String),
    WorkspaceNotFound(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            ConfigError::Parse => {
                format!(
                    "{}\nTo edit the configuration file in your editor run {}",
                    constants::CONFIG_PARSE_ERROR.red().bold(),
                    "toggl config --edit".blue().bold(),
                )
            }
            ConfigError::FileNotFound => {
                format!(
                    "{}\nRun {} to create one",
                    constants::CONFIG_FILE_NOT_FOUND_ERROR.red().bold(),
                    "toggl config init".blue().bold(),
                )
            }
            ConfigError::UnrecognizedMarco(marco) => {
                format!(
                    "{}: {}",
                    constants::CONFIG_UNRECOGNIZED_MACRO_ERROR.red().bold(),
                    marco.red().bold(),
                )
            }
            ConfigError::ShellResolution(command, output_or_error) => {
                format!(
                    "{}: {}\n{}: {}",
                    constants::CONFIG_SHELL_MACRO_RESOLUTION_ERROR.red(),
                    output_or_error.red().bold(),
                    "Command".yellow(),
                    command.yellow().bold(),
                )
            }
            ConfigError::WorkspaceNotFound(workspace) => {
                format!(
                    "{}: {}\n{}\n{}",
                    constants::CONFIG_INVALID_WORKSPACE_ERROR.red().bold(),
                    workspace.red().bold(),
                    "Check your configuration file".yellow().bold(),
                    "toggl config --edit".yellow().bold(),
                )
            }
        };
        writeln!(f, "{summary}")
    }
}

impl Error for ConfigError {}

#[derive(Debug)]
pub enum ArgumentError {
    DirectoryNotFound(PathBuf),
    NotADirectory(PathBuf),
    InvalidDateTime(String),
    InvalidReportDate(String),
    InvalidTimeRange(String),
    MissingUpdateFields(String),
    MultipleWorkspaces(String),
    MissingArgument(String),
    ResourceNotFound(String),
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            ArgumentError::DirectoryNotFound(path) => {
                format!(
                    "{}: {}",
                    constants::DIRECTORY_NOT_FOUND_ERROR.red(),
                    path.display()
                )
            }
            ArgumentError::NotADirectory(path) => {
                format!(
                    "{}: {}",
                    constants::NOT_A_DIRECTORY_ERROR.red(),
                    path.display()
                )
            }
            ArgumentError::InvalidDateTime(value) => {
                format!(
                    "{}: {}\nAccepted formats: today, yesterday, now, this_week, last_week, RFC3339 (e.g. 2026-03-05T09:00:00+08:00), YYYY-MM-DD HH:MM[:SS], YYYY-MM-DDTHH:MM[:SS], YYYY-MM-DD",
                    "Invalid date/time value".red(),
                    value
                )
            }
            ArgumentError::InvalidReportDate(value) => {
                format!(
                    "{}: {}\nAccepted formats: YYYY-MM-DD (e.g. 2026-03-27), today, yesterday, now, this_week, last_week",
                    "Invalid date value".red(),
                    value
                )
            }
            ArgumentError::InvalidTimeRange(message) => {
                format!("{}: {}", "Invalid time range".red(), message)
            }
            ArgumentError::MissingUpdateFields(message) => {
                format!("{}: {}", "Missing update fields".red(), message)
            }
            ArgumentError::MultipleWorkspaces(message) => {
                format!("{}: {}", "Multiple workspaces".red(), message)
            }
            ArgumentError::MissingArgument(message) => {
                format!("{}: {}", "Missing argument".red(), message)
            }
            ArgumentError::ResourceNotFound(message) => {
                format!("{}: {}", "Resource not found".red(), message)
            }
        };
        writeln!(f, "{summary}")
    }
}

impl Error for ArgumentError {}
