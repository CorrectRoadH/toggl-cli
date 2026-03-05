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

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Box<dyn ItemPicker>,
        description: Option<String>,
        project_name: Option<String>,
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
                "--end requires --start".to_string(),
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
        let entities = api_client.get_entities().await?;

        let default_time_entry = config::locate::locate_config_path()
            .and_then(config::parser::get_config_from_file)
            .and_then(|track_config| track_config.get_default_entry(entities.clone()))
            .unwrap_or_else(|_| TimeEntry::default());

        let workspace_id = if default_time_entry.workspace_id != -1 {
            default_time_entry.workspace_id
        } else {
            workspace_id
        };

        let project = project_name
            .and_then(|name| {
                entities
                    .projects
                    .clone()
                    .into_values()
                    .find(|p| p.name == name)
            })
            .or(default_time_entry.project.clone());

        let tags = tags.unwrap_or(default_time_entry.tags.clone());

        let billable = billable
            || default_time_entry.billable
            || project.clone().and_then(|p| p.billable).unwrap_or(false);

        let description = description.unwrap_or(default_time_entry.description.clone());

        let mut time_entry_to_create = {
            let initial_entry = TimeEntry {
                description,
                project,
                tags,
                billable,
                workspace_id,
                ..TimeEntry::default()
            };
            if interactive {
                interactively_create_time_entry(initial_entry, entities.clone(), picker)
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
