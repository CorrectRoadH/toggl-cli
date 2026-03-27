use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};

// Indicator that this test run has dotenv loaded (for fake credential detection)
static DOTENV_LOADED: OnceLock<bool> = OnceLock::new();

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
            let _ = try_run_toggl(&["entry", "delete", &id.to_string()]);
        }
        for id in &self.extra_time_entry_ids {
            let _ = try_run_toggl(&["entry", "delete", &id.to_string()]);
        }
        if let (Some(project_name), Some(task_name)) =
            (self.task_project_name.as_deref(), self.task_name.as_deref())
        {
            let _ = try_run_toggl(&["task", "delete", "--project", project_name, task_name]);
        }
        if let Some(client_name) = self.client_name.as_deref() {
            let _ = try_run_toggl(&["client", "delete", client_name]);
        }
        if let Some(tag_name) = self.tag_name.as_deref() {
            let _ = try_run_toggl(&["tag", "delete", tag_name]);
        }
        if let Some(project_name) = self.project_name.as_deref() {
            let _ = try_run_toggl(&["project", "delete", project_name]);
        }
        if let (Some(original_name), Some(temporary_name)) = (
            self.workspace_original_name.as_deref(),
            self.workspace_temporary_name.as_deref(),
        ) {
            let _ = try_run_toggl(&["workspace", "rename", temporary_name, original_name]);
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

/// Returns true if we detect fake local-debug credentials that would cause
/// live tests to fail or hang when trying to reach an invalid endpoint.
fn is_using_fake_credentials() -> bool {
    if let Ok(token) = std::env::var("TOGGL_API_TOKEN") {
        let trimmed = token.trim();
        if trimmed.is_empty() || trimmed.starts_with("fake-") || trimmed.contains("local-debug") {
            return true;
        }
    }

    if let Ok(url) = std::env::var("TOGGL_API_URL") {
        let trimmed = url.trim().to_ascii_lowercase();
        if trimmed.is_empty()
            || trimmed.contains("invalid")
            || trimmed.contains("local-debug")
            || trimmed.contains("localhost")
            || trimmed.contains("127.0.0.1")
            || trimmed.contains("0.0.0.0")
        {
            return true;
        }
    }

    false
}

/// Attempt to load .env file if present. Returns true if dotenv was loaded.
fn try_load_dotenv() -> bool {
    if let Some(loaded) = DOTENV_LOADED.get() {
        return *loaded;
    }

    // Try to load .env from current directory or ancestor directories
    let loaded = dotenvy::dotenv().is_ok();
    DOTENV_LOADED.get_or_init(|| loaded);
    loaded
}

fn require_live_test_env() {
    // Try to load .env first
    try_load_dotenv();

    let token = std::env::var("TOGGL_API_TOKEN")
        .expect("TOGGL_API_TOKEN must be set when running live CLI tests");
    assert!(
        !token.trim().is_empty(),
        "TOGGL_API_TOKEN must not be empty when running live CLI tests"
    );

    if is_using_fake_credentials() {
        eprintln!(
            "Skipping live CLI test because fake local-debug credentials are detected; live tests require real OpenToggl credentials."
        );
    }
}

fn should_skip_live_tests() -> bool {
    try_load_dotenv();

    match std::env::var("TOGGL_API_TOKEN") {
        Ok(token) if !token.trim().is_empty() => is_using_fake_credentials(),
        _ => false,
    }
}

#[test]
fn direct_startup_prefers_repo_local_dotenv_over_parent_env_for_auth_status() {
    let temp_dir = std::env::temp_dir().join(format!(
        "toggl-cli-dotenv-regression-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX_EPOCH")
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).expect("failed to create temp workspace");

    let env_path = temp_dir.join(".env");
    let repo_local_token = "repo-local-regression-token-1234";
    let repo_local_url = "https://repo-local.example/api/v9";
    std::fs::write(
        &env_path,
        format!(
            "TOGGL_API_TOKEN={repo_local_token}\nTOGGL_API_URL={repo_local_url}\nTOGGL_DISABLE_HTTP_CACHE=1\n"
        ),
    )
    .expect("failed to write temp .env");

    let output = Command::new(env!("CARGO_BIN_EXE_toggl"))
        .current_dir(&temp_dir)
        .args(["auth", "status"])
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", std::env::var("HOME").unwrap_or_default())
        .env(
            "TMPDIR",
            std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string()),
        )
        .env("TOGGL_API_TOKEN", "parent-env-token-9999")
        .output()
        .expect("failed to execute toggl auth status");

    assert!(
        output.status.success(),
        "expected auth status to succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert!(
        stdout.contains(repo_local_url),
        "expected output to use repo-local .env api url instead of parent env or keychain fallback\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("***********************1234"),
        "expected masked token from repo-local .env value in auth status\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Environment (TOGGL_API_TOKEN, TOGGL_API_URL)  (active)"),
        "expected auth status to report environment credentials as the active source\nstdout:\n{stdout}"
    );

    std::fs::remove_file(env_path).ok();
    std::fs::remove_dir(temp_dir).ok();
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
    try_run_toggl_checked(args)
}

fn try_run_toggl_checked(args: &[&str]) -> String {
    let output = try_run_toggl(args).expect("failed to execute toggl");
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        return String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
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
        .env("TOGGL_DISABLE_HTTP_CACHE", "1")
        .output()
}

fn parse_json_output<T: DeserializeOwned>(args: &[&str]) -> T {
    let output = try_run_toggl_checked(args);
    let trimmed = output.trim();

    serde_json::from_str(trimmed)
        .or_else(|_| {
            let last_json_line = trimmed
                .lines()
                .rev()
                .find(|line| {
                    let candidate = line.trim_start();
                    candidate.starts_with('{') || candidate.starts_with('[')
                })
                .unwrap_or(trimmed);
            serde_json::from_str(last_json_line)
        })
        .unwrap_or_else(|error| {
            panic!(
                "failed to parse command JSON output for `toggl {}`: {}\noutput:\n{}",
                args.join(" "),
                error,
                output
            )
        })
}

fn list_entries_on_test_day() -> Vec<TimeEntryRecord> {
    let output = run_toggl(&[
        "entry", "list", "--json", "--since", TEST_DAY, "--until", TEST_DAY,
    ]);
    serde_json::from_str(&output).expect("failed to parse time entry list JSON")
}

fn list_entries_on_day(day: &str) -> Vec<TimeEntryRecord> {
    let output =
        try_run_toggl_checked(&["entry", "list", "--json", "--since", day, "--until", day]);
    serde_json::from_str(&output).expect("failed to parse time entry list JSON")
}

fn run_json_array_command(args: &[&str]) -> Vec<Value> {
    let parsed: Value = parse_json_output(args);
    parsed
        .as_array()
        .cloned()
        .expect("expected command JSON output to be an array")
}

fn run_checked(args: &[&str]) -> String {
    try_run_toggl_checked(args)
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
static LIVE_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_live_test_guard() -> MutexGuard<'static, ()> {
    LIVE_TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn ensure_test_workspace_scope() {
    if test_workspace_id().is_none() {
        return;
    }

    TEST_WORKSPACE_SCOPE_CHECK.get_or_init(|| {
        let me_output = run_checked(&["me"]);
        require_default_workspace_matches_test_workspace(&me_output);
    });
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
        let entries = list_entries_on_day(day);
        if let Some(entry) = entries.into_iter().find(|entry| predicate(entry)) {
            return Some(entry);
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
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let description = unique_description("entry");
    let renamed_description = format!("{description}-edited");
    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();

    let entries_before: Vec<TimeEntryRecord> = parse_json_output(&[
        "entry", "list", "--json", "--since", TEST_DAY, "--until", TEST_DAY,
    ]);
    assert!(
        !entries_before
            .iter()
            .any(|entry| entry.description == description),
        "baseline already contains test description {description}"
    );

    try_run_toggl_checked(&[
        "entry",
        "start",
        "-d",
        &description,
        "--start",
        TEST_START,
        "--end",
        TEST_END,
    ]);

    let Some(created_entry) =
        wait_for_entry_on_day("created time entry missing from list", TEST_DAY, |entry| {
            entry.description == description
        })
    else {
        return;
    };
    cleanup.time_entry_id = Some(created_entry.id);

    let shown_entry: TimeEntryRecord =
        parse_json_output(&["entry", "show", &created_entry.id.to_string(), "--json"]);
    assert_eq!(shown_entry.id, created_entry.id);
    assert_eq!(shown_entry.description, description);

    let bulk_edited_description = format!("{description}-bulk");
    let bulk_edit_payload = format!(
        r#"[{{"op":"replace","path":"/description","value":"{}"}}]"#,
        bulk_edited_description
    );
    try_run_toggl_checked(&[
        "entry",
        "bulk-edit",
        &created_entry.id.to_string(),
        "--json",
        &bulk_edit_payload,
    ]);

    let Some(bulk_edited_entry) = wait_for_entry_on_day(
        "bulk-edited time entry missing from list",
        TEST_DAY,
        |entry| entry.id == created_entry.id && entry.description == bulk_edited_description,
    ) else {
        return;
    };
    assert_eq!(bulk_edited_entry.id, created_entry.id);

    try_run_toggl_checked(&[
        "entry",
        "update",
        &created_entry.id.to_string(),
        "--description",
        &renamed_description,
    ]);

    let Some(edited_entry) =
        wait_for_entry_on_day("edited time entry missing from list", TEST_DAY, |entry| {
            entry.id == created_entry.id && entry.description == renamed_description
        })
    else {
        return;
    };
    assert_eq!(edited_entry.id, created_entry.id);

    let filtered_list_output =
        run_checked(&["entry", "list", "--since", TEST_DAY, "--until", TEST_DAY]);
    assert!(
        filtered_list_output.contains(&renamed_description),
        "expected filtered non-JSON list to include the edited description, got:\n{}",
        filtered_list_output
    );

    try_run_toggl_checked(&["entry", "delete", &created_entry.id.to_string()]);
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
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();

    let commands: [&[&str]; 5] = [
        &["project", "list", "--json"],
        &["client", "list", "--json"],
        &["task", "list", "--json"],
        &["workspace", "list", "--json"],
        &["tag", "list", "--json"],
    ];

    for args in commands {
        let items = run_json_array_command(args);

        assert!(
            items.iter().all(Value::is_object),
            "expected every item from `toggl {}` to be a JSON object",
            args.join(" ")
        );
    }
}

#[test]
fn live_cli_default_time_entry_listing_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();

    let items = run_json_array_command(&["entry", "list", "--json", "--number", "5"]);

    assert!(
        items.iter().all(Value::is_object),
        "expected every item from `toggl entry list --json --number 5` to be a JSON object"
    );
}

#[test]
fn live_cli_read_only_profile_commands_succeed() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let me_output = run_checked(&["me"]);
    assert!(
        me_output.contains("User Profile") && me_output.contains("Email:"),
        "expected `toggl me` output to contain basic profile fields, got:\n{}",
        me_output
    );

    let preferences_json: Value = parse_json_output(&["preferences", "read"]);
    assert!(
        preferences_json.is_object(),
        "expected `toggl preferences read` output to be a JSON object"
    );

    let organizations_json: Value = parse_json_output(&["org", "list", "--json"]);
    assert!(
        organizations_json.is_array(),
        "expected `toggl org list --json` output to be a JSON array"
    );

    if let Some(organization_id) = test_organization_id() {
        let organization_json: Value =
            parse_json_output(&["org", "show", &organization_id.to_string(), "--json"]);
        assert!(
            organization_json.is_object(),
            "expected `toggl org show --json` output to be a JSON object"
        );
    }
}

#[test]
fn live_cli_running_commands_succeed() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();

    for args in [
        &["entry", "running"][..],
        &["entry", "running"][..],
        &["entry", "list", "--number", "1"][..],
    ] {
        let output = run_checked(args);
        assert!(
            !output.trim().is_empty(),
            "expected `toggl {}` to produce some output",
            args.join(" ")
        );
    }
}

#[test]
fn live_cli_start_and_stop_running_entry_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let description = unique_description("running");
    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();

    run_checked(&["entry", "start", "-d", &description]);

    let today = current_utc_day();
    let Some(created_entry) = wait_for_entry_on_day(
        "running time entry missing from current-day list",
        &today,
        |entry| entry.description == description,
    ) else {
        return;
    };
    cleanup.time_entry_id = Some(created_entry.id);

    let running_output = run_checked(&["entry", "running"]);
    assert!(
        running_output.contains(&description),
        "expected `toggl entry running` to show the created running entry, got:\n{}",
        running_output
    );

    let stop_output = run_checked(&["entry", "stop"]);
    assert!(
        stop_output.contains("Time entry stopped successfully"),
        "expected `toggl entry stop` to report success, got:\n{}",
        stop_output
    );
    cleanup.time_entry_id = None;

    let running_after_stop_output = run_checked(&["entry", "running"]);
    assert!(
        !running_after_stop_output.contains(&description),
        "expected running entry to be stopped, got:\n{}",
        running_after_stop_output
    );
}

#[test]
fn live_cli_continue_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let description = unique_description("continue");
    let mut cleanup = CleanupState::default();
    ensure_test_workspace_scope();
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::minutes(5);
    let start_string = start.to_rfc3339();
    let end_string = end.to_rfc3339();

    run_checked(&[
        "entry",
        "start",
        "-d",
        &description,
        "--start",
        &start_string,
        "--end",
        &end_string,
    ]);

    let today = current_utc_day();
    let Some(stopped_entry) = wait_for_entry_on_day(
        "stopped source entry missing from current-day list",
        &today,
        |entry| entry.description == description,
    ) else {
        return;
    };
    cleanup.extra_time_entry_ids.push(stopped_entry.id);

    let continue_output = run_checked(&["entry", "continue"]);
    assert!(
        continue_output.contains("Time entry continued successfully"),
        "expected continue command to report success, got:\n{}",
        continue_output
    );

    let running_output = run_checked(&["entry", "running"]);
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

    let stop_output = run_checked(&["entry", "stop"]);
    assert!(
        stop_output.contains("Time entry stopped successfully"),
        "expected stop after continue to report success, got:\n{}",
        stop_output
    );
}

#[test]
fn live_cli_workspace_resource_crud_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
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

    let projects_before = run_json_array_command(&["project", "list", "--json"]);
    assert!(find_item_by_name(&projects_before, &project_name).is_none());
    assert!(find_item_by_name(&projects_before, &renamed_project_name).is_none());

    run_checked(&["project", "create", &project_name]);
    cleanup.project_name = Some(project_name.clone());

    let projects_after_create = run_json_array_command(&["project", "list", "--json"]);
    assert!(find_item_by_name(&projects_after_create, &project_name).is_some());

    run_checked(&["project", "rename", &project_name, &renamed_project_name]);
    cleanup.project_name = Some(renamed_project_name.clone());

    let projects_after_rename = run_json_array_command(&["project", "list", "--json"]);
    assert!(find_item_by_name(&projects_after_rename, &project_name).is_none());
    assert!(find_item_by_name(&projects_after_rename, &renamed_project_name).is_some());

    run_checked(&[
        "task",
        "create",
        "--project",
        &renamed_project_name,
        &task_name,
    ]);
    cleanup.task_project_name = Some(renamed_project_name.clone());
    cleanup.task_name = Some(task_name.clone());

    let tasks_after_create = run_json_array_command(&["task", "list", "--json"]);
    assert!(find_item_by_name(&tasks_after_create, &task_name).is_some());

    run_checked(&[
        "task",
        "update",
        "--project",
        &renamed_project_name,
        &task_name,
        "--new-name",
        &renamed_task_name,
        "--active",
        "false",
        "--estimated-seconds",
        "120",
    ]);
    cleanup.task_name = Some(renamed_task_name.clone());

    let tasks_after_update = run_json_array_command(&["task", "list", "--json"]);
    assert!(find_item_by_name(&tasks_after_update, &task_name).is_none());
    assert!(find_item_by_name(&tasks_after_update, &renamed_task_name).is_some());

    run_checked(&[
        "task",
        "delete",
        "--project",
        &renamed_project_name,
        &renamed_task_name,
    ]);
    cleanup.task_name = None;
    cleanup.task_project_name = None;

    let tasks_after_delete = run_json_array_command(&["task", "list", "--json"]);
    assert!(find_item_by_name(&tasks_after_delete, &renamed_task_name).is_none());

    run_checked(&["tag", "create", &tag_name]);
    cleanup.tag_name = Some(tag_name.clone());

    let tags_after_create = run_json_array_command(&["tag", "list", "--json"]);
    assert!(find_item_by_name(&tags_after_create, &tag_name).is_some());

    run_checked(&["tag", "rename", &tag_name, &renamed_tag_name]);
    cleanup.tag_name = Some(renamed_tag_name.clone());

    let tags_after_rename = run_json_array_command(&["tag", "list", "--json"]);
    assert!(find_item_by_name(&tags_after_rename, &tag_name).is_none());
    assert!(find_item_by_name(&tags_after_rename, &renamed_tag_name).is_some());

    run_checked(&["tag", "delete", &renamed_tag_name]);
    cleanup.tag_name = None;

    let tags_after_delete = run_json_array_command(&["tag", "list", "--json"]);
    assert!(find_item_by_name(&tags_after_delete, &renamed_tag_name).is_none());

    run_checked(&["client", "create", &client_name]);
    cleanup.client_name = Some(client_name.clone());

    let clients_after_create = run_json_array_command(&["client", "list", "--json"]);
    assert!(find_item_by_name(&clients_after_create, &client_name).is_some());

    run_checked(&["client", "rename", &client_name, &renamed_client_name]);
    cleanup.client_name = Some(renamed_client_name.clone());

    let clients_after_rename = run_json_array_command(&["client", "list", "--json"]);
    assert!(find_item_by_name(&clients_after_rename, &client_name).is_none());
    assert!(find_item_by_name(&clients_after_rename, &renamed_client_name).is_some());

    run_checked(&["client", "delete", &renamed_client_name]);
    cleanup.client_name = None;

    let clients_after_delete = run_json_array_command(&["client", "list", "--json"]);
    assert!(find_item_by_name(&clients_after_delete, &renamed_client_name).is_none());

    run_checked(&["project", "delete", &renamed_project_name]);
    cleanup.project_name = None;

    let projects_after_delete = run_json_array_command(&["project", "list", "--json"]);
    assert!(find_item_by_name(&projects_after_delete, &renamed_project_name).is_none());
}

#[test]
fn live_cli_preferences_round_trip_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let preferences_json: Value = parse_json_output(&["preferences", "read"]);
    assert!(preferences_json.is_object());
    let payload = editable_preferences_payload(&preferences_json);

    let updated_output = run_checked(&["preferences", "update", &payload]);
    assert!(
        updated_output.contains("Preferences updated successfully"),
        "expected preferences update command to report success, got:\n{}",
        updated_output
    );
}

#[test]
fn live_cli_workspace_rename_round_trip_succeeds() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    let mut cleanup = CleanupState::default();
    let me_output = run_checked(&["me"]);
    ensure_test_workspace_scope();
    let default_workspace_id = require_default_workspace_matches_test_workspace(&me_output);

    let workspaces_output = run_checked(&["workspace", "list", "--json"]);
    let workspaces = parse_workspaces(&workspaces_output);
    let workspace = workspaces
        .iter()
        .find(|workspace| workspace.id == default_workspace_id)
        .expect("default workspace missing from workspace list");

    let temporary_name = format!("{}-tmp-{}", workspace.name, unique_description("ws"));
    cleanup.workspace_original_name = Some(workspace.name.clone());
    cleanup.workspace_temporary_name = Some(temporary_name.clone());

    run_checked(&["workspace", "rename", &workspace.name, &temporary_name]);

    let workspaces_after_rename_output = run_checked(&["workspace", "list", "--json"]);
    let workspaces_after_rename = parse_workspaces(&workspaces_after_rename_output);
    assert!(workspaces_after_rename
        .iter()
        .any(|workspace| workspace.id == default_workspace_id && workspace.name == temporary_name));

    run_checked(&[
        "workspace",
        "rename",
        &temporary_name,
        cleanup.workspace_original_name.as_deref().unwrap(),
    ]);
    cleanup.workspace_temporary_name = None;

    let workspaces_after_restore_output = run_checked(&["workspace", "list", "--json"]);
    let workspaces_after_restore = parse_workspaces(&workspaces_after_restore_output);
    assert!(workspaces_after_restore.iter().any(|workspace| {
        workspace.id == default_workspace_id
            && workspace.name == cleanup.workspace_original_name.as_deref().unwrap()
    }));
}

#[test]
fn live_cli_create_workspace_succeeds_when_test_org_is_configured() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
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

    let organization_output = run_checked(&["org", "show", &organization_id.to_string(), "--json"]);
    let organization_json: Value =
        parse_json_output(&["org", "show", &organization_id.to_string(), "--json"]);
    assert!(
        organization_json["id"].as_i64() == Some(organization_id),
        "expected organization lookup to return the configured organization id, got:\n{}",
        organization_output
    );

    let workspaces_before = run_checked(&["workspace", "list", "--json"]);
    let workspaces_before = parse_workspaces(&workspaces_before);
    assert!(workspaces_before
        .iter()
        .all(|workspace| workspace.name != workspace_name));

    let create_output = match try_run_toggl(&[
        "workspace",
        "create",
        &organization_id.to_string(),
        &workspace_name,
    ]) {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if is_workspace_creation_disabled(&stderr) {
                eprintln!(
                    "Skipping live workspace creation test because the configured organization does not allow multiple workspaces.\nstderr:\n{}",
                    stderr
                );
                return;
            }
            panic!(
                "command `toggl workspace create {} {}` failed\nstdout:\n{}\nstderr:\n{}",
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

    let workspaces_after_create = wait_for("created workspace missing from workspace list", || {
        let output = run_checked(&["workspace", "list", "--json"]);
        let workspaces = parse_workspaces(&output);
        workspaces
            .iter()
            .any(|workspace| workspace.name == workspace_name)
            .then_some(workspaces)
    });
    assert!(workspaces_after_create
        .iter()
        .any(|workspace| workspace.name == workspace_name));
}

#[test]
fn live_cli_report_commands_succeed() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();

    // Create a time entry so reports have data
    let description = unique_description("report");
    let mut cleanup = CleanupState::default();
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::minutes(10);
    let start_string = start.to_rfc3339();
    let end_string = end.to_rfc3339();

    run_checked(&[
        "entry",
        "start",
        "-d",
        &description,
        "--start",
        &start_string,
        "--end",
        &end_string,
    ]);

    let today = current_utc_day();
    let Some(entry) =
        wait_for_entry_on_day("report test entry missing from today's list", &today, |e| {
            e.description == description
        })
    else {
        return;
    };
    cleanup.time_entry_id = Some(entry.id);

    // report summary with defaults (no args)
    let summary_output = run_checked(&["report", "summary"]);
    assert!(
        summary_output.contains("Summary Report:"),
        "expected summary report header, got:\n{}",
        summary_output
    );
    assert!(
        summary_output.contains("Total"),
        "expected Total line in summary report, got:\n{}",
        summary_output
    );

    // report summary --json
    let summary_json: Value = parse_json_output(&["report", "summary", "--json"]);
    assert!(
        summary_json.is_object(),
        "expected summary JSON to be an object, got:\n{summary_json}"
    );

    // report summary with natural language dates
    let summary_today = run_checked(&["report", "summary", "--since", "today", "--until", "today"]);
    assert!(
        summary_today.contains("Summary Report:"),
        "expected summary report with natural language dates, got:\n{}",
        summary_today
    );

    // report summary with this_week / last_week
    let summary_week = run_checked(&[
        "report",
        "summary",
        "--since",
        "this_week",
        "--until",
        "today",
    ]);
    assert!(
        summary_week.contains("Summary Report:"),
        "expected summary report with this_week, got:\n{}",
        summary_week
    );

    // report weekly
    let weekly_output = run_checked(&[
        "report",
        "weekly",
        "--since",
        "this_week",
        "--until",
        "today",
    ]);
    assert!(
        weekly_output.contains("Weekly Report:"),
        "expected weekly report header, got:\n{}",
        weekly_output
    );

    // report weekly --json
    let weekly_json: Value = parse_json_output(&[
        "report",
        "weekly",
        "--since",
        "this_week",
        "--until",
        "today",
        "--json",
    ]);
    assert!(
        weekly_json.is_object() || weekly_json.is_array(),
        "expected weekly JSON to be an object or array, got:\n{weekly_json}"
    );

    // report summary --since baddate should fail
    let bad_date_output = try_run_toggl(&[
        "report", "summary", "--since", "baddate", "--until", "today",
    ])
    .expect("failed to execute toggl");
    assert!(
        !bad_date_output.status.success(),
        "expected report with bad date to fail, but it succeeded"
    );
    let stderr = String::from_utf8_lossy(&bad_date_output.stderr);
    assert!(
        stderr.contains("Invalid date value"),
        "expected 'Invalid date value' error, got:\n{}",
        stderr
    );

    // report summary --since after --until should fail
    let inverted_output = try_run_toggl(&[
        "report",
        "summary",
        "--since",
        "2026-03-20",
        "--until",
        "2026-03-10",
    ])
    .expect("failed to execute toggl");
    assert!(
        !inverted_output.status.success(),
        "expected report with inverted date range to fail"
    );
}

#[test]
fn live_cli_natural_language_dates_work_for_entry_list() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();

    // entry list --since today should succeed
    let today_output =
        try_run_toggl(&["entry", "list", "--since", "today"]).expect("failed to execute toggl");
    assert!(
        today_output.status.success(),
        "expected entry list --since today to succeed\nstderr:\n{}",
        String::from_utf8_lossy(&today_output.stderr)
    );

    // entry list --since yesterday should succeed
    let yesterday_output =
        try_run_toggl(&["entry", "list", "--since", "yesterday"]).expect("failed to execute toggl");
    assert!(
        yesterday_output.status.success(),
        "expected entry list --since yesterday to succeed\nstderr:\n{}",
        String::from_utf8_lossy(&yesterday_output.stderr)
    );

    // entry list --since this_week should succeed
    let week_output =
        try_run_toggl(&["entry", "list", "--since", "this_week"]).expect("failed to execute toggl");
    assert!(
        week_output.status.success(),
        "expected entry list --since this_week to succeed\nstderr:\n{}",
        String::from_utf8_lossy(&week_output.stderr)
    );

    // entry list --since last_week should succeed
    let last_week_output =
        try_run_toggl(&["entry", "list", "--since", "last_week"]).expect("failed to execute toggl");
    assert!(
        last_week_output.status.success(),
        "expected entry list --since last_week to succeed\nstderr:\n{}",
        String::from_utf8_lossy(&last_week_output.stderr)
    );

    // entry list --since baddate should fail with helpful error
    let bad_output =
        try_run_toggl(&["entry", "list", "--since", "baddate"]).expect("failed to execute toggl");
    assert!(
        !bad_output.status.success(),
        "expected entry list --since baddate to fail"
    );
    let stderr = String::from_utf8_lossy(&bad_output.stderr);
    assert!(
        stderr.contains("Invalid date/time value"),
        "expected 'Invalid date/time value' error, got:\n{}",
        stderr
    );
}

#[test]
fn live_cli_mutation_json_flags_work() {
    let _guard = acquire_live_test_guard();
    require_live_test_env();
    if should_skip_live_tests() {
        return;
    }

    ensure_test_workspace_scope();
    let mut cleanup = CleanupState::default();
    let description = unique_description("json-mutation");

    // entry start --json should return real ID
    let start_json: Value = parse_json_output(&["entry", "start", "-d", &description, "--json"]);
    assert!(
        start_json["id"].is_number(),
        "expected start --json to return numeric id, got:\n{}",
        start_json
    );
    let entry_id = start_json["id"].as_i64().unwrap();
    assert!(
        entry_id > 0,
        "expected real positive entry ID, got {}",
        entry_id
    );
    assert_eq!(
        start_json["running"].as_bool(),
        Some(true),
        "expected running: true in start --json output"
    );
    cleanup.time_entry_id = Some(entry_id);

    // entry current --json should show the running entry
    let current_json: Value = parse_json_output(&["entry", "current", "--json"]);
    assert_eq!(
        current_json["running"].as_bool(),
        Some(true),
        "expected running: true in current --json output"
    );

    // entry stop --json should return the stopped entry
    let stop_json: Value = parse_json_output(&["entry", "stop", "--json"]);
    assert!(
        stop_json["id"].is_number(),
        "expected stop --json to return entry with id, got:\n{}",
        stop_json
    );
    assert_eq!(
        stop_json["running"].as_bool(),
        Some(false),
        "expected running: false in stop --json output"
    );

    // entry current --json when nothing running should return {"running": false}
    let idle_json: Value = parse_json_output(&["entry", "current", "--json"]);
    assert_eq!(
        idle_json["running"].as_bool(),
        Some(false),
        "expected running: false when no entry is running"
    );

    // entry stop --json when nothing running should return {"running": false}
    let stop_idle_json: Value = parse_json_output(&["entry", "stop", "--json"]);
    assert_eq!(
        stop_idle_json["running"].as_bool(),
        Some(false),
        "expected running: false from stop when nothing running"
    );
}
