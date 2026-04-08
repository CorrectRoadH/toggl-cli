use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;
use crate::utilities;
use chrono::{DateTime, Local, Utc};
use colored::Colorize;

pub struct EditCommand;

impl EditCommand {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        api_client: impl ApiClient,
        id: Option<i64>,
        description: Option<String>,
        billable: Option<bool>,
        project_name: Option<String>,
        task_name: Option<String>,
        tags: Option<Vec<String>>,
        start: Option<String>,
        end: Option<String>,
        json: bool,
    ) -> ResultWithDefaultError<()> {
        let needs_entities = project_name.is_some() || task_name.is_some() || id.is_none();
        let entities = if needs_entities {
            Some(api_client.get_entities().await?)
        } else {
            None
        };

        let time_entry = match id {
            Some(id) => match &entities {
                Some(entities) => entities.time_entries.iter().find(|te| te.id == id).cloned(),
                None => Some(api_client.get_time_entry(id).await?),
            },
            None => api_client.get_current_time_entry().await?,
        };

        match time_entry {
            None => println!("{}", "No matching time entry found".yellow()),
            Some(entry) => {
                let entry_start_date = entry.start.with_timezone(&Local).date_naive();
                let parsed_start = match start {
                    Some(value) => Some(utilities::parse_datetime_input_with_reference(
                        &value,
                        entry_start_date,
                    )?),
                    None => None,
                };
                let effective_start = parsed_start.unwrap_or(entry.start);
                let effective_start_date = effective_start.with_timezone(&Local).date_naive();
                let parsed_end = match end {
                    Some(value) if value.is_empty() => None,
                    Some(value) => Some(utilities::parse_datetime_input_with_reference(
                        &value,
                        effective_start_date,
                    )?),
                    None => entry.stop,
                };

                let project = match project_name.as_deref() {
                    Some("") => None,
                    Some(name) => entities
                        .as_ref()
                        .and_then(|entities| {
                            entities
                                .projects
                                .clone()
                                .into_values()
                                .find(|p| p.name == name)
                        })
                        .or(entry.project.clone()),
                    None => entry.project.clone(),
                };

                let task = match task_name.as_deref() {
                    Some("") => None,
                    Some(name) => entities
                        .as_ref()
                        .and_then(|entities| {
                            entities.tasks.values().find(|task| {
                                task.name == name
                                    && project
                                        .as_ref()
                                        .is_none_or(|project| task.project.id == project.id)
                            })
                        })
                        .cloned(),
                    None => {
                        let project_changed = project.as_ref().map(|project| project.id)
                            != entry.project.as_ref().map(|project| project.id);
                        if project_changed {
                            None
                        } else {
                            entry.task.clone()
                        }
                    }
                };

                let project = task.as_ref().map(|task| task.project.clone()).or(project);

                let tags = match tags {
                    Some(ref t) if t.len() == 1 && t[0].is_empty() => Vec::new(),
                    Some(t) => t,
                    None => entry.tags.clone(),
                };

                let start = effective_start;
                let (stop, duration) = compute_stop_and_duration(start, parsed_end)?;

                let updated = crate::models::TimeEntry {
                    description: description.unwrap_or(entry.description.clone()),
                    billable: billable.unwrap_or(entry.billable),
                    project,
                    task,
                    tags,
                    start,
                    stop,
                    duration,
                    ..entry
                };

                match api_client.update_time_entry(updated.clone()).await {
                    Err(error) => println!("{}\n{}", "Couldn't update time entry".red(), error),
                    Ok(_) => {
                        if json {
                            crate::commands::common::CommandUtils::print_time_entry_json(&updated);
                        } else {
                            println!("{}\n{}", "Time entry updated successfully".green(), updated);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn compute_stop_and_duration(
    start: DateTime<Utc>,
    stop: Option<DateTime<Utc>>,
) -> ResultWithDefaultError<(Option<DateTime<Utc>>, i64)> {
    match stop {
        Some(end) => {
            if end <= start {
                return Err(Box::new(ArgumentError::InvalidTimeRange(
                    "end must be later than start. For cross-day entries, use a full datetime (e.g. 2026-03-29 12:00)".to_string(),
                )));
            }
            Ok((Some(end), (end - start).num_seconds()))
        }
        None => Ok((None, -start.timestamp())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::models::{Entities, Project, Task, TimeEntry};
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

    fn fixed_time(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn mock_project(id: i64, name: &str) -> Project {
        Project {
            id,
            name: name.to_string(),
            workspace_id: 1,
            client: None,
            is_private: false,
            active: true,
            at: fixed_time(1_700_000_000),
            created_at: fixed_time(1_700_000_000),
            color: "#06aaf5".to_string(),
            billable: None,
        }
    }

    fn mock_task(id: i64, name: &str, project: Project) -> Task {
        Task {
            id,
            name: name.to_string(),
            workspace_id: 1,
            project,
            active: true,
        }
    }

    fn mock_entry() -> TimeEntry {
        let project = mock_project(10, "Platform");
        let task = mock_task(50, "Review", project.clone());
        TimeEntry {
            id: 42,
            description: "Initial entry".to_string(),
            start: fixed_time(1_700_000_000),
            stop: Some(fixed_time(1_700_003_600)),
            duration: 3600,
            billable: false,
            workspace_id: 1,
            tags: vec!["dev".to_string()],
            project: Some(project),
            task: Some(task),
            created_with: Some("toggl-cli".to_string()),
        }
    }

    fn mock_entities() -> Entities {
        let current_project = mock_project(10, "Platform");
        let current_task = mock_task(50, "Review", current_project.clone());
        let new_project = mock_project(20, "Ops");
        let new_task = mock_task(60, "Deploy", new_project.clone());

        let mut projects = HashMap::new();
        projects.insert(current_project.id, current_project.clone());
        projects.insert(new_project.id, new_project.clone());

        let mut tasks = HashMap::new();
        tasks.insert(current_task.id, current_task);
        tasks.insert(new_task.id, new_task);

        Entities {
            time_entries: vec![mock_entry()],
            projects,
            tasks,
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn edit_current_entry_clears_task_when_project_changes() {
        let mut api_client = MockApiClient::new();
        let entry = mock_entry();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_get_current_time_entry()
            .returning(move || Ok(Some(entry.clone())));
        api_client
            .expect_update_time_entry()
            .withf(|entry| {
                entry.project.as_ref().map(|project| project.id) == Some(20)
                    && entry.task.is_none()
                    && entry.description == "Initial entry"
            })
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            None,
            None,
            None,
            Some("Ops".to_string()),
            None,
            None,
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_uses_task_match_within_selected_project() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| {
                entry.project.as_ref().map(|project| project.id) == Some(20)
                    && entry.task.as_ref().map(|task| task.id) == Some(60)
                    && entry.description == "Updated entry"
            })
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            Some("Updated entry".to_string()),
            Some(true),
            Some("Ops".to_string()),
            Some("Deploy".to_string()),
            Some(vec!["ops".to_string()]),
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_clears_tags_with_empty_argument() {
        let mut api_client = MockApiClient::new();
        let entry = mock_entry();
        api_client
            .expect_get_time_entry()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(entry.clone()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| entry.tags.is_empty())
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            None,
            None,
            None,
            None,
            Some(vec!["".to_string()]),
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_without_project_or_task_resolution_fetches_single_entry() {
        let mut api_client = MockApiClient::new();
        let entry = mock_entry();
        api_client
            .expect_get_time_entry()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(entry.clone()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| entry.id == 42 && entry.description == "Renamed entry")
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            Some("Renamed entry".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_clears_project_with_empty_string() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| entry.project.is_none() && entry.task.is_none())
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            None,
            None,
            Some("".to_string()),
            None,
            None,
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_clears_end_to_reopen() {
        let mut api_client = MockApiClient::new();
        let entry = mock_entry();
        api_client
            .expect_get_time_entry()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(entry.clone()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| entry.stop.is_none() && entry.duration < 0)
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("".to_string()),
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_updates_billable_flag() {
        let mut api_client = MockApiClient::new();
        let entry = mock_entry();
        assert!(!entry.billable);
        api_client
            .expect_get_time_entry()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(entry.clone()));
        api_client
            .expect_update_time_entry()
            .withf(|entry| entry.billable)
            .returning(|entry| Ok(entry.id));

        let result = EditCommand::execute(
            api_client,
            Some(42),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn edit_entry_not_found_prints_message() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities()));
        api_client
            .expect_get_current_time_entry()
            .returning(|| Ok(None));

        // id=None triggers current entry lookup; returns None
        let result = EditCommand::execute(
            api_client,
            None,
            Some("test".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await;
        // Should succeed (prints message but doesn't error)
        assert_ok!(result);
    }

    #[test]
    fn compute_stop_and_duration_returns_running_duration_without_stop() {
        let start = fixed_time(1_700_000_000);

        let result = compute_stop_and_duration(start, None).unwrap();

        assert_eq!(result.0, None);
        assert_eq!(result.1, -1_700_000_000);
    }

    #[test]
    fn compute_stop_and_duration_rejects_non_increasing_range() {
        let start = fixed_time(1_700_000_000);
        let stop = Some(fixed_time(1_700_000_000));

        let result = compute_stop_and_duration(start, stop);

        assert_err!(result);
    }

    #[test]
    fn compute_stop_and_duration_error_suggests_full_datetime() {
        let start = fixed_time(1_700_000_000);
        let stop = Some(fixed_time(1_699_990_000)); // before start

        let err = compute_stop_and_duration(start, stop).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("full datetime"),
            "error should hint about full datetime for cross-day, got: {msg}"
        );
    }

    #[test]
    fn compute_stop_and_duration_valid_range() {
        let start = fixed_time(1_700_000_000);
        let stop = Some(fixed_time(1_700_003_600)); // 1 hour later

        let (end, duration) = compute_stop_and_duration(start, stop).unwrap();
        assert_eq!(end, Some(fixed_time(1_700_003_600)));
        assert_eq!(duration, 3600);
    }
}
