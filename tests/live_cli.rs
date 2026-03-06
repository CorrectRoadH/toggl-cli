use serde::Deserialize;
use std::process::Command;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct ProjectRecord {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct TagRecord {
    name: String,
}

#[derive(Deserialize)]
struct ClientRecord {
    name: String,
}

#[derive(Default)]
struct CleanupState {
    project_name: Option<String>,
    tag_name: Option<String>,
    client_name: Option<String>,
}

impl Drop for CleanupState {
    fn drop(&mut self) {
        if let Some(project_name) = self.project_name.as_deref() {
            let _ = try_run_toggl(&["delete", "project", project_name]);
        }

        if let Some(tag_name) = self.tag_name.as_deref() {
            let _ = try_run_toggl(&["delete", "tag", tag_name]);
        }

        if let Some(client_name) = self.client_name.as_deref() {
            let _ = try_run_toggl(&["delete", "client", client_name]);
        }
    }
}

fn unique_name(prefix: &str) -> String {
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

fn list_projects() -> Vec<ProjectRecord> {
    serde_json::from_str(&run_toggl(&["list", "project", "--json"]))
        .expect("failed to parse project list JSON")
}

fn list_tags() -> Vec<TagRecord> {
    serde_json::from_str(&run_toggl(&["list", "tag", "--json"]))
        .expect("failed to parse tag list JSON")
}

fn list_clients() -> Vec<ClientRecord> {
    serde_json::from_str(&run_toggl(&["list", "client", "--json"]))
        .expect("failed to parse client list JSON")
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
fn live_cli_round_trip_covers_list_create_mutate_and_cleanup() {
    if !should_run_live_tests() {
        eprintln!("Skipping live CLI tests because TOGGL_API_TOKEN is not set.");
        return;
    }

    let project_name = unique_name("project");
    let renamed_project_name = format!("{project_name}-renamed");
    let tag_name = unique_name("tag");
    let renamed_tag_name = format!("{tag_name}-renamed");
    let client_name = unique_name("client");
    let renamed_client_name = format!("{client_name}-renamed");
    let mut cleanup = CleanupState::default();

    let projects_before = list_projects();
    assert!(
        !projects_before.iter().any(|item| item.name == project_name),
        "baseline already contains test project name {project_name}"
    );

    let tags_before = list_tags();
    assert!(
        !tags_before.iter().any(|item| item.name == tag_name),
        "baseline already contains test tag name {tag_name}"
    );

    let clients_before = list_clients();
    assert!(
        !clients_before.iter().any(|item| item.name == client_name),
        "baseline already contains test client name {client_name}"
    );

    run_toggl(&["create", "project", &project_name]);
    run_toggl(&["create", "tag", &tag_name]);
    run_toggl(&["create", "client", &client_name]);
    cleanup.project_name = Some(project_name.clone());
    cleanup.tag_name = Some(tag_name.clone());
    cleanup.client_name = Some(client_name.clone());

    let projects_after_create = list_projects();
    assert_eq!(projects_after_create.len(), projects_before.len() + 1);
    let created_project = projects_after_create
        .iter()
        .find(|item| item.name == project_name)
        .expect("created project missing from list");

    let tags_after_create = list_tags();
    assert_eq!(tags_after_create.len(), tags_before.len() + 1);
    assert!(
        tags_after_create.iter().any(|item| item.name == tag_name),
        "created tag missing from list"
    );

    let clients_after_create = list_clients();
    assert_eq!(clients_after_create.len(), clients_before.len() + 1);
    assert!(
        clients_after_create
            .iter()
            .any(|item| item.name == client_name),
        "created client missing from list"
    );

    run_toggl(&["rename", "project", &project_name, &renamed_project_name]);
    run_toggl(&["rename", "tag", &tag_name, &renamed_tag_name]);
    run_toggl(&["rename", "client", &client_name, &renamed_client_name]);
    cleanup.project_name = Some(renamed_project_name.clone());
    cleanup.tag_name = Some(renamed_tag_name.clone());
    cleanup.client_name = Some(renamed_client_name.clone());

    let projects_after_rename = wait_for("renamed project missing from list", || {
        let projects = list_projects();
        projects
            .iter()
            .any(|item| item.name == renamed_project_name)
            .then_some(projects)
    });
    let renamed_project = projects_after_rename
        .iter()
        .find(|item| item.name == renamed_project_name)
        .expect("renamed project missing from list");
    assert!(
        projects_after_rename
            .iter()
            .all(|item| item.name != project_name),
        "old project name still present after rename"
    );
    assert_eq!(renamed_project.id, created_project.id);

    let tags_after_rename = wait_for("renamed tag missing from list", || {
        let tags = list_tags();
        tags.iter()
            .any(|item| item.name == renamed_tag_name)
            .then_some(tags)
    });
    assert!(
        tags_after_rename
            .iter()
            .any(|item| item.name == renamed_tag_name),
        "renamed tag missing from list"
    );
    assert!(
        tags_after_rename.iter().all(|item| item.name != tag_name),
        "old tag name still present after rename"
    );

    let clients_after_rename = wait_for("renamed client missing from list", || {
        let clients = list_clients();
        clients
            .iter()
            .any(|item| item.name == renamed_client_name)
            .then_some(clients)
    });
    assert!(
        clients_after_rename
            .iter()
            .any(|item| item.name == renamed_client_name),
        "renamed client missing from list"
    );
    assert!(
        clients_after_rename
            .iter()
            .all(|item| item.name != client_name),
        "old client name still present after rename"
    );
    run_toggl(&["delete", "project", &renamed_project_name]);
    run_toggl(&["delete", "tag", &renamed_tag_name]);
    run_toggl(&["delete", "client", &renamed_client_name]);
    cleanup.project_name = None;
    cleanup.tag_name = None;
    cleanup.client_name = None;

    let projects_after_delete = wait_for("project cleanup did not restore baseline", || {
        let projects = list_projects();
        (projects.len() == projects_before.len()
            && projects
                .iter()
                .all(|item| item.name != project_name && item.name != renamed_project_name))
        .then_some(projects)
    });
    assert_eq!(projects_after_delete.len(), projects_before.len());
    assert!(
        projects_after_delete
            .iter()
            .all(|item| item.name != project_name && item.name != renamed_project_name),
        "project cleanup did not restore baseline"
    );

    let tags_after_delete = wait_for("tag cleanup did not restore baseline", || {
        let tags = list_tags();
        (tags.len() == tags_before.len()
            && tags
                .iter()
                .all(|item| item.name != tag_name && item.name != renamed_tag_name))
        .then_some(tags)
    });
    assert_eq!(tags_after_delete.len(), tags_before.len());
    assert!(
        tags_after_delete
            .iter()
            .all(|item| item.name != tag_name && item.name != renamed_tag_name),
        "tag cleanup did not restore baseline"
    );

    let clients_after_delete = wait_for("client cleanup did not restore baseline", || {
        let clients = list_clients();
        (clients.len() == clients_before.len()
            && clients
                .iter()
                .all(|item| item.name != client_name && item.name != renamed_client_name))
        .then_some(clients)
    });
    assert_eq!(clients_after_delete.len(), clients_before.len());
    assert!(
        clients_after_delete
            .iter()
            .all(|item| item.name != client_name && item.name != renamed_client_name),
        "client cleanup did not restore baseline"
    );
}
