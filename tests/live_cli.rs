use serde::Deserialize;
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
    let output = try_run_toggl(args).expect("failed to execute toggl");

    assert!(
        output.status.success(),
        "command `toggl {}` failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
}

fn try_run_toggl(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_toggl"))
        .args(args)
        .output()
}

fn list_entries_on_test_day() -> Vec<TimeEntryRecord> {
    serde_json::from_str(&run_toggl(&[
        "list", "--json", "--since", TEST_DAY, "--until", TEST_DAY,
    ]))
    .expect("failed to parse time entry list JSON")
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

    let entries_before = list_entries_on_test_day();
    assert!(
        !entries_before
            .iter()
            .any(|entry| entry.description == description),
        "baseline already contains test description {description}"
    );

    run_toggl(&[
        "start",
        &description,
        "--start",
        TEST_START,
        "--end",
        TEST_END,
    ]);

    let created_entry = wait_for("created time entry missing from list", || {
        list_entries_on_test_day()
            .into_iter()
            .find(|entry| entry.description == description)
    });
    cleanup.time_entry_id = Some(created_entry.id);

    run_toggl(&[
        "edit",
        "time-entry",
        &created_entry.id.to_string(),
        "--description",
        &renamed_description,
    ]);

    let edited_entry = wait_for("edited time entry missing from list", || {
        list_entries_on_test_day()
            .into_iter()
            .find(|entry| entry.id == created_entry.id && entry.description == renamed_description)
    });
    assert_eq!(edited_entry.id, created_entry.id);

    run_toggl(&["delete", &created_entry.id.to_string()]);
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
