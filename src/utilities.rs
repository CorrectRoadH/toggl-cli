use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use colored::Colorize;
use directories::BaseDirs;

use crate::{constants, error::ArgumentError, models::ResultWithDefaultError};

pub fn remove_trailing_newline(value: String) -> String {
    value.trim_end().to_string()
}

pub fn read_from_stdin(text: &str) -> String {
    print_without_buffer(text);
    let mut result = String::new();
    io::stdin()
        .read_line(&mut result)
        .expect("Failed to read line");
    remove_trailing_newline(result)
}

pub fn simplify_config_path_for_display(dir: &Path) -> String {
    if !std::io::stdout().is_terminal() {
        return dir.display().to_string();
    }
    let base_dirs = BaseDirs::new().unwrap();
    let local_config_base_path = base_dirs.config_local_dir().to_str().unwrap();
    let mut display_config_path = dir.to_str().unwrap().to_string();
    display_config_path.replace_range(..local_config_base_path.len(), constants::SIMPLE_HOME_PATH);

    display_config_path
}

pub fn read_from_stdin_with_constraints(text: &str, valid_values: &[String]) -> String {
    loop {
        let result = read_from_stdin(text);
        if valid_values.contains(&result) {
            return result;
        } else {
            let error_message = format!(
                "Invalid value \"{}\". Valid values are: {}\n",
                result,
                valid_values.join(", ")
            )
            .red();
            print_without_buffer(&error_message);
        }
    }
}

pub fn open_path_in_editor<P>(path: P) -> ResultWithDefaultError<()>
where
    P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
{
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let mut child = std::process::Command::new(editor)
        .arg(path)
        .spawn()
        .expect("Failed to spawn editor");
    child.wait().expect("Failed to wait for editor");
    Ok(())
}

pub fn get_git_branch_for_dir(dir: &PathBuf) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8(output.stdout).ok()?;
    Some(branch.trim().to_string())
}

pub fn parse_datetime_input(input: &str) -> ResultWithDefaultError<DateTime<Utc>> {
    let value = input.trim();
    if value.is_empty() {
        return Err(Box::new(ArgumentError::InvalidDateTime(input.to_string())));
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }

    let local_formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ];

    for format in local_formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return match Local.from_local_datetime(&naive) {
                LocalResult::Single(parsed) => Ok(parsed.with_timezone(&Utc)),
                _ => Err(Box::new(ArgumentError::InvalidDateTime(input.to_string()))),
            };
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let naive = match date.and_hms_opt(0, 0, 0) {
            Some(naive) => naive,
            None => return Err(Box::new(ArgumentError::InvalidDateTime(input.to_string()))),
        };
        return match Local.from_local_datetime(&naive) {
            LocalResult::Single(parsed) => Ok(parsed.with_timezone(&Utc)),
            _ => Err(Box::new(ArgumentError::InvalidDateTime(input.to_string()))),
        };
    }

    Err(Box::new(ArgumentError::InvalidDateTime(input.to_string())))
}

pub fn normalize_time_entry_list_filters(
    since: Option<String>,
    until: Option<String>,
) -> ResultWithDefaultError<(Option<String>, Option<String>)> {
    let since = since
        .as_deref()
        .map(|value| normalize_time_entry_list_filter(value, false))
        .transpose()?;
    let until = until
        .as_deref()
        .map(|value| normalize_time_entry_list_filter(value, true))
        .transpose()?;

    // Validate that --since is not after --until
    if let (Some(since_val), Some(until_val)) = (&since, &until) {
        if since_val > until_val {
            return Err(Box::new(ArgumentError::InvalidTimeRange(
                "the --since date/time must not be after the --until date/time. \
                 Got reversed range: --since was later than --until. \
                 Hint: swap the values or use a valid date range."
                    .to_string(),
            )));
        }
    }

    Ok((since, until))
}

fn normalize_time_entry_list_filter(input: &str, is_until: bool) -> ResultWithDefaultError<String> {
    let value = input.trim();
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let boundary_date = if is_until {
            match date.succ_opt() {
                Some(next_date) => next_date,
                None => return Err(Box::new(ArgumentError::InvalidDateTime(input.to_string()))),
            }
        } else {
            date
        };
        let naive = match boundary_date.and_hms_opt(0, 0, 0) {
            Some(naive) => naive,
            None => return Err(Box::new(ArgumentError::InvalidDateTime(input.to_string()))),
        };
        // Convert local time to UTC for API, ensuring proper RFC3339 format
        return match Local.from_local_datetime(&naive) {
            LocalResult::Single(local_dt) => {
                let utc_dt = local_dt.with_timezone(&Utc);
                Ok(utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            }
            LocalResult::None => {
                // Handle DST gaps by using UTC directly
                let utc_dt = Utc.from_utc_datetime(&naive);
                Ok(utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            }
            LocalResult::Ambiguous(dt1, _dt2) => {
                // Handle DST overlaps by using the first option
                let utc_dt = dt1.with_timezone(&Utc);
                Ok(utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            }
        };
    }

    Ok(parse_datetime_input(value)?.to_rfc3339())
}

#[cfg(unix)]
pub fn get_shell_cmd(command: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c");
    cmd.arg(command);
    cmd
}

#[cfg(windows)]
pub fn get_shell_cmd(command: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/C");
    cmd.arg(command);
    cmd
}

fn print_without_buffer(text: &str) {
    print!("{text}");
    io::stdout().flush().unwrap();
}
