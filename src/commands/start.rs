use crate::api;
use crate::commands;
use crate::config;
use crate::error::ArgumentError;
use crate::models;
use crate::models::Entities;
use crate::picker::ItemPicker;
use crate::picker::PickableItem;
use crate::picker::PickableItemKind;
use crate::utilities;
use api::client::ApiClient;
use chrono::{DateTime, Utc};
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::{ResultWithDefaultError, TimeEntry};

pub struct StartCommand;

fn interactively_create_time_entry(
    time_entry: TimeEntry,
    entities: Entities,
    picker: Box<dyn ItemPicker>,
) -> TimeEntry {
    let yes_or_default_no = [
        "y".to_string(),
        "n".to_string(),
        "N".to_string(),
        "".to_string(),
    ];

    let (project, task) = match time_entry.project {
        Some(_) => (time_entry.project, None),
        None => {
            if entities.projects.is_empty() {
                (None, None)
            } else {
                let mut pickable_items: Vec<PickableItem> = entities
                    .projects
                    .clone()
                    .into_values()
                    .map(PickableItem::from_project)
                    .collect();

                pickable_items.extend(
                    entities
                        .tasks
                        .clone()
                        .into_values()
                        .map(PickableItem::from_task),
                );

                match picker.pick(pickable_items) {
                    Ok(picked_key) => match picked_key.kind {
                        PickableItemKind::TimeEntry => (None, None),
                        PickableItemKind::Project => {
                            (entities.projects.get(&picked_key.id).cloned(), None)
                        }
                        PickableItemKind::Task => {
                            let task = entities.tasks.get(&picked_key.id).cloned().unwrap();
                            (Some(task.clone().project), Some(task))
                        }
                    },

                    Err(_) => (None, None),
                }
            }
        }
    };

    // Only ask for billable if the user didn't provide a value AND if the selected project doesn't have a default billable setting.
    let billable = time_entry.billable
        || project.clone().and_then(|p| p.billable).unwrap_or(
            utilities::read_from_stdin_with_constraints(
                "Is your time entry billable? (y/N): ",
                &yes_or_default_no,
            ) == "y",
        );

    let task = task.or(time_entry.task.clone());

    TimeEntry {
        billable,
        project,
        task,
        ..time_entry
    }
}

fn resolve_task_from_name(
    entities: &Entities,
    task_name: &str,
    project_id: Option<i64>,
) -> Option<models::Task> {
    entities
        .tasks
        .values()
        .find(|task| task.name == task_name && project_id.is_none_or(|id| task.project.id == id))
        .cloned()
}

impl StartCommand {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Box<dyn ItemPicker>,
        description: Option<String>,
        project_name: Option<String>,
        task_name: Option<String>,
        tags: Option<Vec<String>>,
        billable: bool,
        interactive: bool,
        start: Option<String>,
        end: Option<String>,
    ) -> ResultWithDefaultError<()> {
        let parsed_start = match start {
            Some(value) => Some(utilities::parse_datetime_input(&value)?),
            None => None,
        };
        let parsed_end = match end {
            Some(value) => Some(utilities::parse_datetime_input(&value)?),
            None => None,
        };

        if parsed_end.is_some() && parsed_start.is_none() {
            return Err(Box::new(ArgumentError::InvalidTimeRange(
                "--end requires --start. Example: toggl entry start --start 09:00 --end 10:00"
                    .to_string(),
            )));
        }

        if let (Some(start), Some(end)) = (parsed_start, parsed_end) {
            if end <= start {
                return Err(Box::new(ArgumentError::InvalidTimeRange(
                    "end must be later than start".to_string(),
                )));
            }
        }

        if parsed_end.is_none() {
            StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;
        }

        let workspace_id = (api_client.get_user().await?).default_workspace_id;
        let track_config = config::locate::locate_config_path()
            .ok()
            .and_then(|path| config::parser::get_config_from_file(path).ok());
        let needs_entities =
            interactive || project_name.is_some() || task_name.is_some() || track_config.is_some();
        let entities = if needs_entities {
            Some(api_client.get_entities().await?)
        } else {
            None
        };

        let default_time_entry = match (&track_config, &entities) {
            (Some(track_config), Some(entities)) => {
                track_config.get_default_entry(entities.clone())?
            }
            _ => TimeEntry::default(),
        };

        let workspace_id = if default_time_entry.workspace_id != -1 {
            default_time_entry.workspace_id
        } else {
            workspace_id
        };

        let project = project_name
            .and_then(|name| {
                entities
                    .as_ref()
                    .into_iter()
                    .flat_map(|entities| entities.projects.clone().into_values())
                    .find(|p| p.name == name)
            })
            .or(default_time_entry.project.clone());

        let task = task_name
            .as_deref()
            .and_then(|name| {
                entities.as_ref().and_then(|entities| {
                    resolve_task_from_name(entities, name, project.as_ref().map(|p| p.id))
                })
            })
            .or(default_time_entry.task.clone());

        let project = task.as_ref().map(|task| task.project.clone()).or(project);

        let tags = tags.unwrap_or(default_time_entry.tags.clone());

        let billable = billable
            || default_time_entry.billable
            || project.clone().and_then(|p| p.billable).unwrap_or(false);

        let description = description.unwrap_or(default_time_entry.description.clone());

        let mut time_entry_to_create = {
            let initial_entry = TimeEntry {
                description,
                project,
                task,
                tags,
                billable,
                workspace_id,
                ..TimeEntry::default()
            };
            if interactive {
                interactively_create_time_entry(
                    initial_entry,
                    entities.expect("interactive mode requires entities"),
                    picker,
                )
            } else {
                initial_entry
            }
        };

        apply_custom_time_range(&mut time_entry_to_create, parsed_start, parsed_end)?;

        let started_entry_id = api_client
            .create_time_entry(time_entry_to_create.clone())
            .await;
        if started_entry_id.is_err() {
            println!("{}", "Failed to start time entry".red());
            return Err(started_entry_id.err().unwrap());
        }

        println!("{}\n{}", "Time entry started".green(), time_entry_to_create);

        Ok(())
    }
}

fn apply_custom_time_range(
    time_entry: &mut TimeEntry,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> ResultWithDefaultError<()> {
    if let Some(start) = start {
        time_entry.start = start;
        match end {
            Some(end) => {
                if end <= start {
                    return Err(Box::new(ArgumentError::InvalidTimeRange(
                        "end must be later than start".to_string(),
                    )));
                }
                time_entry.stop = Some(end);
                time_entry.duration = (end - start).num_seconds();
            }
            None => {
                time_entry.stop = None;
                time_entry.duration = -start.timestamp();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::models::{Project, Task};
    use crate::picker::{PickableItem, PickableItemKey};
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;
    use tokio_test::assert_err;

    struct StubPicker {
        key: PickableItemKey,
    }

    impl ItemPicker for StubPicker {
        fn pick(&self, _items: Vec<PickableItem>) -> ResultWithDefaultError<PickableItemKey> {
            Ok(self.key.clone())
        }
    }

    fn fixed_time(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn mock_project(id: i64, name: &str, billable: Option<bool>) -> Project {
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
            billable,
        }
    }

    fn mock_task(id: i64, name: &str, project: Project) -> Task {
        Task {
            id,
            name: name.to_string(),
            workspace_id: 1,
            project,
        }
    }

    fn mock_entities() -> Entities {
        let project = mock_project(10, "Platform", Some(true));
        let other_project = mock_project(20, "Ops", Some(false));
        let task = mock_task(50, "Review", project.clone());
        let other_task = mock_task(60, "Review", other_project.clone());

        let mut projects = HashMap::new();
        projects.insert(project.id, project);
        projects.insert(other_project.id, other_project);

        let mut tasks = HashMap::new();
        tasks.insert(task.id, task);
        tasks.insert(other_task.id, other_task);

        Entities {
            time_entries: Vec::new(),
            projects,
            tasks,
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[test]
    fn resolve_task_from_name_matches_within_project() {
        let entities = mock_entities();

        let task = resolve_task_from_name(&entities, "Review", Some(10)).unwrap();

        assert_eq!(task.id, 50);
        assert_eq!(task.project.id, 10);
    }

    #[test]
    fn interactively_create_time_entry_uses_selected_task_project_and_billable_default() {
        let entities = mock_entities();
        let picker = Box::new(StubPicker {
            key: PickableItemKey {
                id: 50,
                kind: PickableItemKind::Task,
            },
        });
        let initial_entry = TimeEntry {
            description: "Review PR".to_string(),
            workspace_id: 1,
            ..TimeEntry::default()
        };

        let entry = interactively_create_time_entry(initial_entry, entities, picker);

        assert_eq!(entry.project.as_ref().map(|project| project.id), Some(10));
        assert_eq!(entry.task.as_ref().map(|task| task.id), Some(50));
        assert!(entry.billable);
    }

    #[test]
    fn apply_custom_time_range_sets_running_entry_when_end_is_missing() {
        let start = fixed_time(1_700_000_000);
        let mut time_entry = TimeEntry::default();

        apply_custom_time_range(&mut time_entry, Some(start), None).unwrap();

        assert_eq!(time_entry.start, start);
        assert_eq!(time_entry.stop, None);
        assert_eq!(time_entry.duration, -1_700_000_000);
    }

    #[test]
    fn apply_custom_time_range_sets_finished_duration() {
        let start = fixed_time(1_700_000_000);
        let end = fixed_time(1_700_003_600);
        let mut time_entry = TimeEntry::default();

        apply_custom_time_range(&mut time_entry, Some(start), Some(end)).unwrap();

        assert_eq!(time_entry.start, start);
        assert_eq!(time_entry.stop, Some(end));
        assert_eq!(time_entry.duration, 3600);
    }

    #[tokio::test]
    async fn start_rejects_end_without_start() {
        let api_client = MockApiClient::new();
        let picker = Box::new(StubPicker {
            key: PickableItemKey {
                id: 10,
                kind: PickableItemKind::Project,
            },
        });

        let result = StartCommand::execute(
            api_client,
            picker,
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            Some("2026-01-01T10:00:00Z".to_string()),
        )
        .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn start_rejects_non_increasing_range() {
        let api_client = MockApiClient::new();
        let picker = Box::new(StubPicker {
            key: PickableItemKey {
                id: 10,
                kind: PickableItemKind::Project,
            },
        });

        let result = StartCommand::execute(
            api_client,
            picker,
            None,
            None,
            None,
            None,
            false,
            false,
            Some("2026-01-01T10:00:00Z".to_string()),
            Some("2026-01-01T09:00:00Z".to_string()),
        )
        .await;

        assert_err!(result);
    }
}
