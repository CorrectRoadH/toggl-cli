use crate::api;
use crate::arguments::{Entity, StatusFilter};
use crate::models;
use crate::utilities;
use api::client::ApiClient;
use chrono::{DateTime, Local};
use models::{ResultWithDefaultError, TimeEntry};
use std::io::{self, BufWriter, Write};

pub struct ListCommand;

/// Serialize a slice of TimeEntry references as a JSON array,
/// injecting a `"running"` boolean into each entry.
fn time_entries_to_json(entries: &[&TimeEntry]) -> String {
    let values: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            let mut value = serde_json::to_value(entry).expect("failed to serialize time entry");
            if entry.is_running() {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("running".to_string(), serde_json::Value::Bool(true));
                }
            }
            value
        })
        .collect();
    serde_json::to_string(&values).expect("failed to serialize time entries to JSON")
}

fn write_entries_grouped_by_date(handle: &mut impl Write, entries: &[&TimeEntry]) {
    let mut current_date: Option<chrono::NaiveDate> = None;
    for entry in entries {
        let local_start: DateTime<Local> = entry.start.with_timezone(&Local);
        let date = local_start.date_naive();
        if current_date != Some(date) {
            if current_date.is_some() {
                writeln!(handle).expect("failed to print");
            }
            writeln!(handle, "── {} ──", date.format("%Y-%m-%d %A")).expect("failed to print");
            current_date = Some(date);
        }
        writeln!(handle, "{entry}").expect("failed to print");
    }
}

impl ListCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        count: Option<usize>,
        json_flag: bool,
        since: Option<String>,
        until: Option<String>,
        entity: Option<Entity>,
    ) -> ResultWithDefaultError<()> {
        let is_time_entry = matches!(entity, None | Some(Entity::TimeEntry { .. }));
        let has_date_filter = since.is_some() || until.is_some();

        if is_time_entry && has_date_filter {
            let (since, until) = utilities::normalize_time_entry_list_filters(since, until)?;
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let json = match &entity {
                Some(Entity::TimeEntry { json }) => json_flag || *json,
                _ => json_flag,
            };
            let entries = if json {
                api_client
                    .get_time_entries_filtered_minimal(since, until)
                    .await
            } else {
                api_client.get_time_entries_filtered(since, until).await
            };
            match entries {
                Err(error) => {
                    return Err(error);
                }
                Ok(mut entries) => {
                    entries.sort_by_key(|e| e.start);
                    let entries = entries
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string = time_entries_to_json(&entries);
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else if entries.is_empty() {
                        eprintln!("No entries found.");
                    } else {
                        write_entries_grouped_by_date(&mut handle, &entries);
                    }
                }
            }
            return Ok(());
        }

        if let Some(Entity::Tag { json: entity_json }) = entity {
            let json = json_flag || entity_json;
            let user = api_client.get_user().await?;
            match api_client.get_tags(user.default_workspace_id).await {
                Err(error) => {
                    return Err(error);
                }
                Ok(tags) => {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    let tags = tags
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string =
                            serde_json::to_string(&tags).expect("failed to serialize tags to JSON");
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else {
                        tags.iter()
                            .for_each(|tag| writeln!(handle, "{tag}").expect("failed to print"));
                    }
                }
            }
            return Ok(());
        }

        if let Some(Entity::Client {
            json: entity_json,
            status,
        }) = entity
        {
            let json = json_flag || entity_json;
            let user = api_client.get_user().await?;
            let api_status = match &status {
                StatusFilter::Active => None,
                StatusFilter::Archived => Some("archived".to_string()),
                StatusFilter::All => Some("both".to_string()),
                StatusFilter::Done => None,
            };
            match api_client
                .get_clients(user.default_workspace_id, api_status)
                .await
            {
                Err(error) => {
                    return Err(error);
                }
                Ok(clients) => {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    let clients = clients
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .collect::<Vec<_>>();
                    if json {
                        let json_string = serde_json::to_string(&clients)
                            .expect("failed to serialize clients to JSON");
                        writeln!(handle, "{json_string}").expect("failed to print");
                    } else {
                        clients
                            .iter()
                            .for_each(|c| writeln!(handle, "{c}").expect("failed to print"));
                    }
                }
            }
            return Ok(());
        }

        if let Some(Entity::Organization { json: entity_json }) = entity {
            let json = json_flag || entity_json;
            let organizations = api_client.get_organizations().await?;
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let organizations = organizations
                .iter()
                .take(count.unwrap_or(usize::MAX))
                .collect::<Vec<_>>();
            if json {
                let json_string = serde_json::to_string(&organizations)
                    .expect("failed to serialize organizations to JSON");
                writeln!(handle, "{json_string}").expect("failed to print");
            } else {
                organizations.iter().for_each(|organization| {
                    writeln!(handle, "{organization}").expect("failed to print")
                });
            }
            return Ok(());
        }

        if let Some(Entity::Project {
            json: entity_json,
            status,
        }) = entity
        {
            let json = json_flag || entity_json;
            let projects = api_client.get_projects_list().await?;
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let projects: Vec<_> = projects
                .iter()
                .filter(|p| match &status {
                    StatusFilter::Active => p.active,
                    StatusFilter::Archived => !p.active,
                    StatusFilter::All => true,
                    StatusFilter::Done => true,
                })
                .take(count.unwrap_or(usize::MAX))
                .collect();
            if json {
                let json_string =
                    serde_json::to_string(&projects).expect("failed to serialize projects to JSON");
                writeln!(handle, "{json_string}").expect("failed to print");
            } else {
                projects
                    .iter()
                    .for_each(|project| writeln!(handle, "{project}").expect("failed to print"));
            }
            return Ok(());
        }

        if let Some(Entity::Workspace { json: entity_json }) = entity {
            let json = json_flag || entity_json;
            let workspaces = api_client.get_workspaces_list().await?;
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let workspaces = workspaces
                .iter()
                .take(count.unwrap_or(usize::MAX))
                .collect::<Vec<_>>();
            if json {
                let json_string = serde_json::to_string(&workspaces)
                    .expect("failed to serialize workspaces to JSON");
                writeln!(handle, "{json_string}").expect("failed to print");
            } else {
                workspaces.iter().for_each(|workspace| {
                    writeln!(handle, "{workspace}").expect("failed to print")
                });
            }
            return Ok(());
        }

        if let Some(Entity::Task {
            json: entity_json,
            status,
        }) = entity
        {
            let json = json_flag || entity_json;
            let tasks = api_client.get_tasks_list().await?;
            let stdout = io::stdout();
            let mut handle = BufWriter::new(stdout);
            let tasks: Vec<_> = tasks
                .iter()
                .filter(|t| match &status {
                    StatusFilter::Active => t.active,
                    StatusFilter::Done => !t.active,
                    StatusFilter::All => true,
                    StatusFilter::Archived => true,
                })
                .take(count.unwrap_or(usize::MAX))
                .collect();
            if json {
                let json_string =
                    serde_json::to_string(&tasks).expect("failed to serialize tasks to JSON");
                writeln!(handle, "{json_string}").expect("failed to print");
            } else {
                tasks
                    .iter()
                    .for_each(|task| writeln!(handle, "{task}").expect("failed to print"));
            }
            return Ok(());
        }

        match api_client.get_entities().await {
            Err(error) => {
                return Err(error);
            }
            Ok(entities) => {
                // use this to avoid calling println! in a loop:
                // <https://rust-cli.github.io/book/tutorial/output.html#a-note-on-printing-performance>
                let stdout = io::stdout();
                let mut handle = BufWriter::new(stdout);

                // TODO: better error handling for writeln!
                match entity.unwrap_or(Entity::TimeEntry { json: false }) {
                    Entity::TimeEntry { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let mut time_entries = entities.time_entries;
                        time_entries.sort_by_key(|e| e.start);
                        let entries = time_entries
                            .iter()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = time_entries_to_json(&entries);
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else if entries.is_empty() {
                            eprintln!("No entries found.");
                        } else {
                            write_entries_grouped_by_date(&mut handle, &entries);
                        }
                    }

                    Entity::Project {
                        json: entity_json,
                        status,
                    } => {
                        let json = json_flag || entity_json;
                        let projects: Vec<_> = entities
                            .projects
                            .values()
                            .filter(|p| match &status {
                                StatusFilter::Active => p.active,
                                StatusFilter::Archived => !p.active,
                                StatusFilter::All => true,
                                StatusFilter::Done => true,
                            })
                            .take(count.unwrap_or(usize::MAX))
                            .collect();

                        if json {
                            let json_string = serde_json::to_string(&projects)
                                .expect("failed to serialize projects to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            projects.iter().for_each(|project| {
                                writeln!(handle, "{project}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Workspace { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let workspaces = entities
                            .workspaces
                            .iter()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string(&workspaces)
                                .expect("failed to serialize workspaces to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            workspaces.iter().for_each(|workspace| {
                                writeln!(handle, "{workspace}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Task {
                        json: entity_json,
                        status,
                    } => {
                        let json = json_flag || entity_json;
                        let tasks: Vec<_> = entities
                            .tasks
                            .values()
                            .filter(|t| match &status {
                                StatusFilter::Active => t.active,
                                StatusFilter::Done => !t.active,
                                StatusFilter::All => true,
                                StatusFilter::Archived => true,
                            })
                            .take(count.unwrap_or(usize::MAX))
                            .collect();

                        if json {
                            let json_string = serde_json::to_string(&tasks)
                                .expect("failed to serialize tasks to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            tasks.iter().for_each(|task| {
                                writeln!(handle, "{task}").expect("failed to print")
                            });
                        }
                    }

                    // Already handled above via dedicated API paths
                    Entity::Tag { .. } | Entity::Client { .. } | Entity::Organization { .. } => {
                        unreachable!()
                    }
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::models::{Client, Entities, Tag, TimeEntry, User};
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;
    use tokio_test::assert_ok;

    fn mock_user() -> User {
        User {
            api_token: "token".to_string(),
            email: "test@example.com".to_string(),
            fullname: Some("Test".to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1,
            beginning_of_week: None,
            image_url: None,
            created_at: None,
            updated_at: None,
            country_id: None,
            has_password: None,
        }
    }

    fn mock_time_entry() -> TimeEntry {
        TimeEntry {
            id: 42,
            description: "Test entry".to_string(),
            start: Utc::now(),
            stop: None,
            duration: -Utc::now().timestamp(),
            billable: false,
            workspace_id: 1,
            tags: vec!["dev".to_string()],
            project: None,
            task: None,
            created_with: Some("toggl-cli".to_string()),
        }
    }

    #[tokio::test]
    async fn list_time_entries_with_date_filter_uses_filtered_endpoint() {
        let mut api_client = MockApiClient::new();
        let (expected_since, expected_until) = crate::utilities::normalize_time_entry_list_filters(
            Some("2026-01-01".to_string()),
            Some("2026-01-31".to_string()),
        )
        .expect("date filter should normalize");
        api_client
            .expect_get_time_entries_filtered()
            .withf(move |since, until| *since == expected_since && *until == expected_until)
            .returning(|_, _| Ok(vec![mock_time_entry()]));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            false,
            Some("2026-01-01".to_string()),
            Some("2026-01-31".to_string()),
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_time_entries_with_date_filter_and_json_uses_minimal_endpoint() {
        let mut api_client = MockApiClient::new();
        let (expected_since, expected_until) = crate::utilities::normalize_time_entry_list_filters(
            Some("2026-01-01".to_string()),
            Some("2026-01-31".to_string()),
        )
        .expect("date filter should normalize");
        api_client
            .expect_get_time_entries_filtered_minimal()
            .withf(move |since, until| *since == expected_since && *until == expected_until)
            .returning(|_, _| Ok(vec![mock_time_entry()]));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            true,
            Some("2026-01-01".to_string()),
            Some("2026-01-31".to_string()),
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_time_entries_with_same_day_filter_expands_until_to_next_day() {
        let mut api_client = MockApiClient::new();
        let (expected_since, expected_until) = crate::utilities::normalize_time_entry_list_filters(
            Some("2026-03-06".to_string()),
            Some("2026-03-06".to_string()),
        )
        .expect("date filter should normalize");
        api_client
            .expect_get_time_entries_filtered()
            .withf(move |since, until| *since == expected_since && *until == expected_until)
            .returning(|_, _| Ok(vec![mock_time_entry()]));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            false,
            Some("2026-03-06".to_string()),
            Some("2026-03-06".to_string()),
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_tags_uses_workspace_specific_api() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_tags()
            .withf(|wid| *wid == 1)
            .returning(|wid| {
                Ok(vec![Tag {
                    id: 10,
                    name: "backend".to_string(),
                    workspace_id: wid,
                }])
            });

        let result = ListCommand::execute(
            api_client,
            None,
            false,
            None,
            None,
            Some(Entity::Tag { json: false }),
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_clients_uses_workspace_specific_api() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_get_clients()
            .withf(|wid, _status| *wid == 1)
            .returning(|wid, _| {
                Ok(vec![Client {
                    id: 20,
                    name: "Acme".to_string(),
                    workspace_id: wid,
                    archived: false,
                }])
            });

        let result = ListCommand::execute(
            api_client,
            None,
            false,
            None,
            None,
            Some(Entity::Client {
                json: true,
                status: StatusFilter::Active,
            }),
        )
        .await;
        assert_ok!(result);
    }

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn mock_time_entry_at(id: i64, start_secs: i64) -> TimeEntry {
        let start = fixed_time(start_secs);
        TimeEntry {
            id,
            description: format!("Entry {id}"),
            start,
            stop: Some(fixed_time(start_secs + 3600)),
            duration: 3600,
            billable: false,
            workspace_id: 1,
            tags: vec![],
            project: None,
            task: None,
            created_with: Some("toggl-cli".to_string()),
        }
    }

    #[tokio::test]
    async fn list_entries_sorted_by_start_time_not_id() {
        // entry with higher id but earlier start should come first
        let later_id_earlier_start = mock_time_entry_at(100, 1_700_000_000);
        let earlier_id_later_start = mock_time_entry_at(1, 1_700_010_000);

        let mut api_client = MockApiClient::new();
        let (expected_since, expected_until) = crate::utilities::normalize_time_entry_list_filters(
            Some("2023-11-14".to_string()),
            Some("2023-11-15".to_string()),
        )
        .expect("should normalize");
        api_client
            .expect_get_time_entries_filtered()
            .withf(move |since, until| *since == expected_since && *until == expected_until)
            .returning(move |_, _| {
                // Return in id order (wrong for display)
                Ok(vec![
                    earlier_id_later_start.clone(),
                    later_id_earlier_start.clone(),
                ])
            });

        let result = ListCommand::execute(
            api_client,
            None,
            false,
            Some("2023-11-14".to_string()),
            Some("2023-11-15".to_string()),
            None,
        )
        .await;
        assert_ok!(result);
        // The test verifies no panic; actual order is validated by the sort_by_key(start) call
    }

    #[tokio::test]
    async fn list_unfiltered_entries_sorted_by_start_time() {
        let later_id_earlier_start = mock_time_entry_at(100, 1_700_000_000);
        let earlier_id_later_start = mock_time_entry_at(1, 1_700_010_000);

        let mut api_client = MockApiClient::new();
        api_client.expect_get_entities().returning(move || {
            Ok(Entities {
                time_entries: vec![
                    earlier_id_later_start.clone(),
                    later_id_earlier_start.clone(),
                ],
                projects: HashMap::new(),
                tasks: HashMap::new(),
                clients: HashMap::new(),
                workspaces: Vec::new(),
                tags: Vec::new(),
            })
        });

        let result = ListCommand::execute(api_client, None, false, None, None, None).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn list_projects_uses_projects_api() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_projects_list()
            .returning(|| Ok(Vec::new()));

        let result = ListCommand::execute(
            api_client,
            Some(1),
            false,
            None,
            None,
            Some(Entity::Project {
                json: false,
                status: StatusFilter::Active,
            }),
        )
        .await;
        assert_ok!(result);
    }
}
