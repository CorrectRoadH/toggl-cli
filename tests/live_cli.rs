use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_DAY: &str = "2026-03-05";
const TEST_START: &str = "2026-03-05T09:00:00Z";
const TEST_END: &str = "2026-03-05T09:05:00Z";

#[derive(Deserialize)]
struct TimeEntryRecord {
    id: i64,
    description: String,
}

#[derive(Default)]
struct CleanupState {
    time_entry_id: Option<i64>,
}

impl Drop for CleanupState {
    fn drop(&mut self) {
        if let Some(id) = self.time_entry_id {
            let _ = try_run_toggl(&["delete", &id.to_string()]);
        }
    }
}

fn unique_description(prefix: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX_EPOCH")
        .as_nanos();
    format!("toggl-cli-{prefix}-{nonce}")
}

fn should_run_live_tests() -> bool {
    matches!(std::env::var("TOGGL_API_TOKEN"), Ok(token) if !token.trim().is_empty())
}

fn run_toggl(args: &[&str]) -> String {
    match try_run_toggl_checked(args) {
        Ok(output) => output,
        Err(SkipReason::RateLimited(message)) => {
            eprintln!(
                "Skipping live CLI test because Toggl API rate limit was hit while running `toggl {}`.\nstderr:\n{}",
                args.join(" "),
                message
            );
            String::new()
        }
    }
}

enum SkipReason {
    RateLimited(String),
}

fn try_run_toggl_checked(args: &[&str]) -> Result<String, SkipReason> {
    let output = try_run_toggl(args).expect("failed to execute toggl");
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        return Ok(String::from_utf8(output.stdout).expect("stdout was not valid UTF-8"));
    }

    if is_rate_limited(&stderr) {
        return Err(SkipReason::RateLimited(stderr.into_owned()));
    }

    panic!(
        "command `toggl {}` failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
}

fn try_run_toggl(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_toggl"))
        .args(args)
        .output()
}

fn list_entries_on_test_day() -> Vec<TimeEntryRecord> {
    let output = run_toggl(&["list", "--json", "--since", TEST_DAY, "--until", TEST_DAY]);
    serde_json::from_str(&output).expect("failed to parse time entry list JSON")
}

fn run_json_array_command(args: &[&str]) -> Option<Vec<Value>> {
    match try_run_toggl_checked(args) {
        Ok(output) => {
            let parsed: Value =
                serde_json::from_str(&output).expect("failed to parse command JSON output");
            Some(
                parsed
                    .as_array()
                    .cloned()
                    .expect("expected command JSON output to be an array"),
            )
        }
        Err(SkipReason::RateLimited(message)) => {
            eprintln!(
                "Skipping live CLI test because Toggl API rate limit was hit while running `toggl {}`.\nstderr:\n{}",
                args.join(" "),
                message
            );
            None
        }
    }
}

fn run_checked_or_skip(args: &[&str]) -> Option<String> {
    match try_run_toggl_checked(args) {
        Ok(output) => Some(output),
        Err(SkipReason::RateLimited(message)) => {
            eprintln!(
                "Skipping live CLI test because Toggl API rate limit was hit while running `toggl {}`.\nstderr:\n{}",
                args.join(" "),
                message
            );
            None
        }
    }
}

fn is_rate_limited(stderr: &str) -> bool {
    let stderr = stderr.to_ascii_lowercase();
    stderr.contains("hourly limit for api calls")
        || stderr.contains("quota will reset in")
        || stderr.contains("too many requests")
        || stderr.contains("rate limit")
}

fn wait_for<T, F>(message: &str, mut fetch: F) -> T
where
    F: FnMut() -> Option<T>,
{
    for _ in 0..10 {
        if let Some(value) = fetch() {
            return value;
        }
        sleep(std::time::Duration::from_millis(500));
    }

    panic!("{message}");
}

#[test]
fn live_cli_round_trip_covers_time_entry_lifecycle() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let description = unique_description("entry");
    let renamed_description = format!("{description}-edited");
    let mut cleanup = CleanupState::default();

    let entries_before: Vec<TimeEntryRecord> = match try_run_toggl_checked(&[
        "list", "--json", "--since", TEST_DAY, "--until", TEST_DAY,
    ]) {
        Ok(output) => serde_json::from_str(&output).expect("failed to parse time entry list JSON"),
        Err(SkipReason::RateLimited(message)) => {
            eprintln!(
                "Skipping live CLI test because Toggl API rate limit was hit while loading the baseline list.\nstderr:\n{}",
                message
            );
            return;
        }
    };
    assert!(
        !entries_before
            .iter()
            .any(|entry| entry.description == description),
        "baseline already contains test description {description}"
    );

    if try_run_toggl_checked(&[
        "start",
        &description,
        "--start",
        TEST_START,
        "--end",
        TEST_END,
    ])
    .is_err()
    {
        return;
    }

    let created_entry = wait_for("created time entry missing from list", || {
        list_entries_on_test_day()
            .into_iter()
            .find(|entry| entry.description == description)
    });
    cleanup.time_entry_id = Some(created_entry.id);

    let Some(shown_entry_output) =
        run_checked_or_skip(&["show", &created_entry.id.to_string(), "--json"])
    else {
        return;
    };
    let shown_entry: TimeEntryRecord =
        serde_json::from_str(&shown_entry_output).expect("failed to parse show time entry JSON");
    assert_eq!(shown_entry.id, created_entry.id);
    assert_eq!(shown_entry.description, description);

    if try_run_toggl_checked(&[
        "edit",
        "time-entry",
        &created_entry.id.to_string(),
        "--description",
        &renamed_description,
    ])
    .is_err()
    {
        return;
    }

    let edited_entry = wait_for("edited time entry missing from list", || {
        list_entries_on_test_day()
            .into_iter()
            .find(|entry| entry.id == created_entry.id && entry.description == renamed_description)
    });
    assert_eq!(edited_entry.id, created_entry.id);

    if try_run_toggl_checked(&["delete", &created_entry.id.to_string()]).is_err() {
        return;
    }
    cleanup.time_entry_id = None;

    let entries_after_delete = wait_for("time entry cleanup did not restore baseline", || {
        let entries = list_entries_on_test_day();
        entries
            .iter()
            .all(|entry| {
                entry.id != created_entry.id
                    && entry.description != description
                    && entry.description != renamed_description
            })
            .then_some(entries)
    });
    assert!(
        entries_after_delete.iter().all(|entry| {
            entry.id != created_entry.id
                && entry.description != description
                && entry.description != renamed_description
        }),
        "time entry cleanup did not restore baseline"
    );
}

#[test]
fn live_cli_list_commands_cover_workspace_resources() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let commands: [&[&str]; 5] = [
        &["list", "project", "--json"],
        &["list", "client", "--json"],
        &["list", "task", "--json"],
        &["list", "workspace", "--json"],
        &["list", "tag", "--json"],
    ];

    for args in commands {
        let Some(items) = run_json_array_command(args) else {
            return;
        };

        assert!(
            items.iter().all(Value::is_object),
            "expected every item from `toggl {}` to be a JSON object",
            args.join(" ")
        );
    }
}

#[test]
fn live_cli_read_only_profile_commands_succeed() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let Some(me_output) = run_checked_or_skip(&["me"]) else {
        return;
    };
    assert!(
        me_output.contains("User Profile") && me_output.contains("Email:"),
        "expected `toggl me` output to contain basic profile fields, got:\n{}",
        me_output
    );

    let Some(preferences_output) = run_checked_or_skip(&["preferences"]) else {
        return;
    };
    let preferences_json: Value = serde_json::from_str(&preferences_output)
        .expect("expected `toggl preferences` to return JSON");
    assert!(
        preferences_json.is_object(),
        "expected `toggl preferences` output to be a JSON object"
    );
}

#[test]
fn live_cli_running_commands_succeed() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    for args in [&["running"][..], &[][..], &["current"][..]] {
        let Some(output) = run_checked_or_skip(args) else {
            return;
        };
        assert!(
            !output.trim().is_empty(),
            "expected `toggl {}` to produce some output",
            args.join(" ")
        );
    }
}
