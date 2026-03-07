use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use std::sync::OnceLock;
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

#[derive(Deserialize)]
struct WorkspaceRecord {
    id: i64,
    name: String,
}

#[derive(Default)]
struct CleanupState {
    time_entry_id: Option<i64>,
    extra_time_entry_ids: Vec<i64>,
    project_name: Option<String>,
    tag_name: Option<String>,
    client_name: Option<String>,
    task_project_name: Option<String>,
    task_name: Option<String>,
    workspace_original_name: Option<String>,
    workspace_temporary_name: Option<String>,
}

impl Drop for CleanupState {
    fn drop(&mut self) {
        if let Some(id) = self.time_entry_id {
            let _ = try_run_toggl(&["delete", &id.to_string()]);
        }
        for id in &self.extra_time_entry_ids {
            let _ = try_run_toggl(&["delete", &id.to_string()]);
        }
        if let (Some(project_name), Some(task_name)) =
            (self.task_project_name.as_deref(), self.task_name.as_deref())
        {
            let _ = try_run_toggl(&["delete", "task", "--project", project_name, task_name]);
        }
        if let Some(client_name) = self.client_name.as_deref() {
            let _ = try_run_toggl(&["delete", "client", client_name]);
        }
        if let Some(tag_name) = self.tag_name.as_deref() {
            let _ = try_run_toggl(&["delete", "tag", tag_name]);
        }
        if let Some(project_name) = self.project_name.as_deref() {
            let _ = try_run_toggl(&["delete", "project", project_name]);
        }
        if let (Some(original_name), Some(temporary_name)) = (
            self.workspace_original_name.as_deref(),
            self.workspace_temporary_name.as_deref(),
        ) {
            let _ = try_run_toggl(&["rename", "workspace", temporary_name, original_name]);
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

fn test_organization_id() -> Option<i64> {
    std::env::var("TOGGL_TEST_ORGANIZATION_ID")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
}

fn test_workspace_id() -> Option<i64> {
    std::env::var("TOGGL_TEST_WORKSPACE_ID")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
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

fn list_entries_on_day(day: &str) -> Result<Vec<TimeEntryRecord>, SkipReason> {
    let output = try_run_toggl_checked(&["list", "--json", "--since", day, "--until", day])?;
    Ok(serde_json::from_str(&output).expect("failed to parse time entry list JSON"))
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

fn find_item_by_name<'a>(items: &'a [Value], name: &str) -> Option<&'a Value> {
    items
        .iter()
        .find(|item| item["name"].as_str() == Some(name))
}

fn parse_workspaces(output: &str) -> Vec<WorkspaceRecord> {
    serde_json::from_str(output).expect("failed to parse workspace list JSON")
}

fn editable_preferences_payload(preferences_json: &Value) -> String {
    for key in ["date_format", "timeofday_format", "duration_format"] {
        if !preferences_json[key].is_null() {
            return serde_json::json!({ key: preferences_json[key].clone() }).to_string();
        }
    }

    panic!("could not find a stable editable preference field");
}

fn default_workspace_id_from_me(output: &str) -> i64 {
    output
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("Default Workspace ID: ")
                .and_then(|value| value.parse::<i64>().ok())
        })
        .expect("failed to find default workspace ID in `toggl me` output")
}

fn require_default_workspace_matches_test_workspace(me_output: &str) -> i64 {
    let default_workspace_id = default_workspace_id_from_me(me_output);
    if let Some(expected_workspace_id) = test_workspace_id() {
        assert_eq!(
            default_workspace_id, expected_workspace_id,
            "TOGGL_TEST_WORKSPACE_ID is set to {}, but `toggl me` reports default workspace {}. Live tests would otherwise operate outside the intended workspace.",
            expected_workspace_id, default_workspace_id
        );
    }
    default_workspace_id
}

static TEST_WORKSPACE_SCOPE_CHECK: OnceLock<()> = OnceLock::new();

fn ensure_test_workspace_scope() {
    if test_workspace_id().is_none() {
        return;
    }

    TEST_WORKSPACE_SCOPE_CHECK.get_or_init(|| {
        let Some(me_output) = run_checked_or_skip(&["me"]) else {
            return;
        };
        require_default_workspace_matches_test_workspace(&me_output);
    });
}

fn is_rate_limited(stderr: &str) -> bool {
    let stderr = stderr.to_ascii_lowercase();
    stderr.contains("hourly limit for api calls")
        || stderr.contains("quota will reset in")
        || stderr.contains("too many requests")
        || stderr.contains("rate limit")
}

fn is_workspace_creation_disabled(stderr: &str) -> bool {
    stderr
        .to_ascii_lowercase()
        .contains("multiple workspaces are not enabled in this organization")
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

fn wait_for_entry_on_day<F>(message: &str, day: &str, mut predicate: F) -> Option<TimeEntryRecord>
where
    F: FnMut(&TimeEntryRecord) -> bool,
{
    for _ in 0..10 {
        match list_entries_on_day(day) {
            Ok(entries) => {
                if let Some(entry) = entries.into_iter().find(|entry| predicate(entry)) {
                    return Some(entry);
                }
            }
            Err(SkipReason::RateLimited(message)) => {
                eprintln!(
                    "Skipping live CLI test because Toggl API rate limit was hit while polling `toggl list --json --since {} --until {}`.\nstderr:\n{}",
                    day, day, message
                );
                return None;
            }
        }
        sleep(std::time::Duration::from_millis(500));
    }

    panic!("{message}");
}

fn current_utc_day() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
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
    ensure_test_workspace_scope();

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

    let Some(created_entry) =
        wait_for_entry_on_day("created time entry missing from list", TEST_DAY, |entry| {
            entry.description == description
        })
    else {
        return;
    };
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

    let bulk_edited_description = format!("{description}-bulk");
    let bulk_edit_payload = format!(
        r#"[{{"op":"replace","path":"/description","value":"{}"}}]"#,
        bulk_edited_description
    );
    if try_run_toggl_checked(&[
        "bulk-edit-time-entries",
        &created_entry.id.to_string(),
        "--json",
        &bulk_edit_payload,
    ])
    .is_err()
    {
        return;
    }

    let Some(bulk_edited_entry) = wait_for_entry_on_day(
        "bulk-edited time entry missing from list",
        TEST_DAY,
        |entry| entry.id == created_entry.id && entry.description == bulk_edited_description,
    ) else {
        return;
    };
    assert_eq!(bulk_edited_entry.id, created_entry.id);

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

    let Some(edited_entry) =
        wait_for_entry_on_day("edited time entry missing from list", TEST_DAY, |entry| {
            entry.id == created_entry.id && entry.description == renamed_description
        })
    else {
        return;
    };
    assert_eq!(edited_entry.id, created_entry.id);

    let Some(filtered_list_output) =
        run_checked_or_skip(&["list", "--since", TEST_DAY, "--until", TEST_DAY])
    else {
        return;
    };
    assert!(
        filtered_list_output.contains(&renamed_description),
        "expected filtered non-JSON list to include the edited description, got:\n{}",
        filtered_list_output
    );

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

    ensure_test_workspace_scope();

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
fn live_cli_default_time_entry_listing_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    ensure_test_workspace_scope();

    let Some(items) = run_json_array_command(&["list", "--json", "--number", "5"]) else {
        return;
    };

    assert!(
        items.iter().all(Value::is_object),
        "expected every item from `toggl list --json --number 5` to be a JSON object"
    );
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

    let Some(organizations_output) = run_checked_or_skip(&["organization", "list", "--json"])
    else {
        return;
    };
    let organizations_json: Value = serde_json::from_str(&organizations_output)
        .expect("expected `toggl organization list --json` to return JSON");
    assert!(
        organizations_json.is_array(),
        "expected `toggl organization list --json` output to be a JSON array"
    );

    if let Some(organization_id) = test_organization_id() {
        let Some(organization_output) = run_checked_or_skip(&[
            "organization",
            "show",
            &organization_id.to_string(),
            "--json",
        ]) else {
            return;
        };
        let organization_json: Value = serde_json::from_str(&organization_output)
            .expect("expected `toggl organization show --json` to return JSON");
        assert!(
            organization_json.is_object(),
            "expected `toggl organization show --json` output to be a JSON object"
        );
    }
}

#[test]
fn live_cli_running_commands_succeed() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    ensure_test_workspace_scope();

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

#[test]
fn live_cli_start_and_stop_running_entry_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let description = unique_description("running");
    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();

    if run_checked_or_skip(&["start", &description]).is_none() {
        return;
    }

    let today = current_utc_day();
    let Some(created_entry) = wait_for_entry_on_day(
        "running time entry missing from current-day list",
        &today,
        |entry| entry.description == description,
    ) else {
        return;
    };
    cleanup.time_entry_id = Some(created_entry.id);

    let Some(running_output) = run_checked_or_skip(&["running"]) else {
        return;
    };
    assert!(
        running_output.contains(&description),
        "expected `toggl running` to show the created running entry, got:\n{}",
        running_output
    );

    let Some(stop_output) = run_checked_or_skip(&["stop"]) else {
        return;
    };
    assert!(
        stop_output.contains("Time entry stopped successfully"),
        "expected `toggl stop` to report success, got:\n{}",
        stop_output
    );
    cleanup.time_entry_id = None;

    let Some(running_after_stop_output) = run_checked_or_skip(&["running"]) else {
        return;
    };
    assert!(
        !running_after_stop_output.contains(&description),
        "expected running entry to be stopped, got:\n{}",
        running_after_stop_output
    );
}

#[test]
fn live_cli_continue_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let description = unique_description("continue");
    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();
    let end = chrono::Utc::now() - chrono::Duration::minutes(5);
    let start = end - chrono::Duration::minutes(5);
    let start_string = start.to_rfc3339();
    let end_string = end.to_rfc3339();

    if run_checked_or_skip(&[
        "start",
        &description,
        "--start",
        &start_string,
        "--end",
        &end_string,
    ])
    .is_none()
    {
        return;
    }

    let today = current_utc_day();
    let Some(stopped_entry) = wait_for_entry_on_day(
        "stopped source entry missing from current-day list",
        &today,
        |entry| entry.description == description,
    ) else {
        return;
    };
    cleanup.extra_time_entry_ids.push(stopped_entry.id);

    let Some(continue_output) = run_checked_or_skip(&["continue"]) else {
        return;
    };
    assert!(
        continue_output.contains("Time entry continued successfully"),
        "expected continue command to report success, got:\n{}",
        continue_output
    );

    let Some(running_output) = run_checked_or_skip(&["running"]) else {
        return;
    };
    assert!(
        running_output.contains(&description),
        "expected continued entry to be running, got:\n{}",
        running_output
    );

    let Some(running_entry) = wait_for_entry_on_day(
        "continued running entry missing from current-day list",
        &today,
        |entry| entry.description == description && entry.id != stopped_entry.id,
    ) else {
        return;
    };
    cleanup.time_entry_id = Some(running_entry.id);

    let Some(stop_output) = run_checked_or_skip(&["stop"]) else {
        return;
    };
    assert!(
        stop_output.contains("Time entry stopped successfully"),
        "expected stop after continue to report success, got:\n{}",
        stop_output
    );
}

#[test]
fn live_cli_workspace_resource_crud_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();
    let project_name = unique_description("project");
    let renamed_project_name = format!("{project_name}-renamed");
    let task_name = unique_description("task");
    let renamed_task_name = format!("{task_name}-renamed");
    let tag_name = unique_description("tag");
    let renamed_tag_name = format!("{tag_name}-renamed");
    let client_name = unique_description("client");
    let renamed_client_name = format!("{client_name}-renamed");

    let Some(projects_before) = run_json_array_command(&["list", "project", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&projects_before, &project_name).is_none());
    assert!(find_item_by_name(&projects_before, &renamed_project_name).is_none());

    if run_checked_or_skip(&["create", "project", &project_name]).is_none() {
        return;
    }
    cleanup.project_name = Some(project_name.clone());

    let Some(projects_after_create) = run_json_array_command(&["list", "project", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&projects_after_create, &project_name).is_some());

    if run_checked_or_skip(&["rename", "project", &project_name, &renamed_project_name]).is_none() {
        return;
    }
    cleanup.project_name = Some(renamed_project_name.clone());

    let Some(projects_after_rename) = run_json_array_command(&["list", "project", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&projects_after_rename, &project_name).is_none());
    assert!(find_item_by_name(&projects_after_rename, &renamed_project_name).is_some());

    if run_checked_or_skip(&[
        "create",
        "task",
        "--project",
        &renamed_project_name,
        &task_name,
    ])
    .is_none()
    {
        return;
    }
    cleanup.task_project_name = Some(renamed_project_name.clone());
    cleanup.task_name = Some(task_name.clone());

    let Some(tasks_after_create) = run_json_array_command(&["list", "task", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tasks_after_create, &task_name).is_some());

    if run_checked_or_skip(&[
        "edit",
        "task",
        "--project",
        &renamed_project_name,
        &task_name,
        "--new-name",
        &renamed_task_name,
        "--active",
        "false",
        "--estimated-seconds",
        "120",
    ])
    .is_none()
    {
        return;
    }
    cleanup.task_name = Some(renamed_task_name.clone());

    let Some(tasks_after_update) = run_json_array_command(&["list", "task", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tasks_after_update, &task_name).is_none());
    assert!(find_item_by_name(&tasks_after_update, &renamed_task_name).is_some());

    if run_checked_or_skip(&[
        "delete",
        "task",
        "--project",
        &renamed_project_name,
        &renamed_task_name,
    ])
    .is_none()
    {
        return;
    }
    cleanup.task_name = None;
    cleanup.task_project_name = None;

    let Some(tasks_after_delete) = run_json_array_command(&["list", "task", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tasks_after_delete, &renamed_task_name).is_none());

    if run_checked_or_skip(&["create", "tag", &tag_name]).is_none() {
        return;
    }
    cleanup.tag_name = Some(tag_name.clone());

    let Some(tags_after_create) = run_json_array_command(&["list", "tag", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tags_after_create, &tag_name).is_some());

    if run_checked_or_skip(&["rename", "tag", &tag_name, &renamed_tag_name]).is_none() {
        return;
    }
    cleanup.tag_name = Some(renamed_tag_name.clone());

    let Some(tags_after_rename) = run_json_array_command(&["list", "tag", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tags_after_rename, &tag_name).is_none());
    assert!(find_item_by_name(&tags_after_rename, &renamed_tag_name).is_some());

    if run_checked_or_skip(&["delete", "tag", &renamed_tag_name]).is_none() {
        return;
    }
    cleanup.tag_name = None;

    let Some(tags_after_delete) = run_json_array_command(&["list", "tag", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&tags_after_delete, &renamed_tag_name).is_none());

    if run_checked_or_skip(&["create", "client", &client_name]).is_none() {
        return;
    }
    cleanup.client_name = Some(client_name.clone());

    let Some(clients_after_create) = run_json_array_command(&["list", "client", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&clients_after_create, &client_name).is_some());

    if run_checked_or_skip(&["rename", "client", &client_name, &renamed_client_name]).is_none() {
        return;
    }
    cleanup.client_name = Some(renamed_client_name.clone());

    let Some(clients_after_rename) = run_json_array_command(&["list", "client", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&clients_after_rename, &client_name).is_none());
    assert!(find_item_by_name(&clients_after_rename, &renamed_client_name).is_some());

    if run_checked_or_skip(&["delete", "client", &renamed_client_name]).is_none() {
        return;
    }
    cleanup.client_name = None;

    let Some(clients_after_delete) = run_json_array_command(&["list", "client", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&clients_after_delete, &renamed_client_name).is_none());

    if run_checked_or_skip(&["delete", "project", &renamed_project_name]).is_none() {
        return;
    }
    cleanup.project_name = None;

    let Some(projects_after_delete) = run_json_array_command(&["list", "project", "--json"]) else {
        return;
    };
    assert!(find_item_by_name(&projects_after_delete, &renamed_project_name).is_none());
}

#[test]
fn live_cli_preferences_round_trip_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let Some(preferences_output) = run_checked_or_skip(&["preferences"]) else {
        return;
    };
    let preferences_json: Value =
        serde_json::from_str(&preferences_output).expect("failed to parse preferences JSON");
    assert!(preferences_json.is_object());
    let payload = editable_preferences_payload(&preferences_json);

    let Some(updated_output) = run_checked_or_skip(&["edit", "preferences", &payload]) else {
        return;
    };
    assert!(
        updated_output.contains("Preferences updated successfully"),
        "expected preferences update command to report success, got:\n{}",
        updated_output
    );
}

#[test]
fn live_cli_workspace_rename_round_trip_succeeds() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let mut cleanup = CleanupState::default();
    let Some(me_output) = run_checked_or_skip(&["me"]) else {
        return;
    };
    ensure_test_workspace_scope();
    let default_workspace_id = require_default_workspace_matches_test_workspace(&me_output);

    let Some(workspaces_output) = run_checked_or_skip(&["list", "workspace", "--json"]) else {
        return;
    };
    let workspaces = parse_workspaces(&workspaces_output);
    let workspace = workspaces
        .iter()
        .find(|workspace| workspace.id == default_workspace_id)
        .expect("default workspace missing from workspace list");

    let temporary_name = format!("{}-tmp-{}", workspace.name, unique_description("ws"));
    cleanup.workspace_original_name = Some(workspace.name.clone());
    cleanup.workspace_temporary_name = Some(temporary_name.clone());

    if run_checked_or_skip(&["rename", "workspace", &workspace.name, &temporary_name]).is_none() {
        return;
    }

    let Some(workspaces_after_rename_output) =
        run_checked_or_skip(&["list", "workspace", "--json"])
    else {
        return;
    };
    let workspaces_after_rename = parse_workspaces(&workspaces_after_rename_output);
    assert!(workspaces_after_rename
        .iter()
        .any(|workspace| workspace.id == default_workspace_id && workspace.name == temporary_name));

    if run_checked_or_skip(&[
        "rename",
        "workspace",
        &temporary_name,
        cleanup.workspace_original_name.as_deref().unwrap(),
    ])
    .is_none()
    {
        return;
    }
    cleanup.workspace_temporary_name = None;

    let Some(workspaces_after_restore_output) =
        run_checked_or_skip(&["list", "workspace", "--json"])
    else {
        return;
    };
    let workspaces_after_restore = parse_workspaces(&workspaces_after_restore_output);
    assert!(workspaces_after_restore.iter().any(|workspace| {
        workspace.id == default_workspace_id
            && workspace.name == cleanup.workspace_original_name.as_deref().unwrap()
    }));
}

#[test]
fn live_cli_create_workspace_succeeds_when_test_org_is_configured() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let Some(organization_id) = test_organization_id() else {
        eprintln!(
            "Skipping live workspace creation test because TOGGL_TEST_ORGANIZATION_ID is not set."
        );
        return;
    };
    ensure_test_workspace_scope();

    let workspace_name = unique_description("workspace");

    let Some(organization_output) = run_checked_or_skip(&[
        "organization",
        "show",
        &organization_id.to_string(),
        "--json",
    ]) else {
        return;
    };
    let organization_json: Value = serde_json::from_str(&organization_output)
        .expect("expected `toggl organization show --json` to return JSON");
    assert!(
        organization_json["id"].as_i64() == Some(organization_id),
        "expected organization lookup to return the configured organization id, got:\n{}",
        organization_output
    );

    let Some(workspaces_before) = run_checked_or_skip(&["list", "workspace", "--json"]) else {
        return;
    };
    let workspaces_before = parse_workspaces(&workspaces_before);
    assert!(workspaces_before
        .iter()
        .all(|workspace| workspace.name != workspace_name));

    let create_output = match try_run_toggl(&[
        "create",
        "workspace",
        &organization_id.to_string(),
        &workspace_name,
    ]) {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if is_rate_limited(&stderr) {
                eprintln!(
                    "Skipping live CLI test because Toggl API rate limit was hit while running `toggl create workspace {}`.\nstderr:\n{}",
                    organization_id, stderr
                );
                return;
            }
            if is_workspace_creation_disabled(&stderr) {
                eprintln!(
                    "Skipping live workspace creation test because the configured organization does not allow multiple workspaces.\nstderr:\n{}",
                    stderr
                );
                return;
            }
            panic!(
                "command `toggl create workspace {} {}` failed\nstdout:\n{}\nstderr:\n{}",
                organization_id,
                workspace_name,
                String::from_utf8_lossy(&output.stdout),
                stderr
            );
        }
        Err(error) => panic!("failed to execute toggl: {}", error),
    };
    assert!(
        create_output.contains("Workspace created successfully"),
        "expected workspace creation command to report success, got:\n{}",
        create_output
    );

    let workspaces_after_create =
        wait_for(
            "created workspace missing from workspace list",
            || match run_checked_or_skip(&["list", "workspace", "--json"]) {
                Some(output) => {
                    let workspaces = parse_workspaces(&output);
                    workspaces
                        .iter()
                        .any(|workspace| workspace.name == workspace_name)
                        .then_some(workspaces)
                }
                None => None,
            },
        );
    assert!(workspaces_after_create
        .iter()
        .any(|workspace| workspace.name == workspace_name));
}
