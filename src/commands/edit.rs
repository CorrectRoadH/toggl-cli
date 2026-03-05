use crate::api::client::ApiClient;
use crate::error::ArgumentError;
use crate::models::ResultWithDefaultError;
use crate::utilities;
use chrono::{DateTime, Utc};
use colored::Colorize;

pub struct EditCommand;

impl EditCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: Option<i64>,
        description: Option<String>,
        project_name: Option<String>,
        tags: Option<Vec<String>>,
        start: Option<String>,
        end: Option<String>,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;

        let time_entry = match id {
            Some(id) => entities.time_entries.into_iter().find(|te| te.id == id),
            None => entities.running_time_entry(),
        };

        match time_entry {
            None => println!("{}", "No matching time entry found".yellow()),
            Some(entry) => {
                let parsed_start = match start {
                    Some(value) => Some(utilities::parse_datetime_input(&value)?),
                    None => None,
                };
                let parsed_end = match end {
                    Some(value) if value.is_empty() => None,
                    Some(value) => Some(utilities::parse_datetime_input(&value)?),
                    None => entry.stop.clone(),
                };

                let project = match project_name.as_deref() {
                    Some("") => None,
                    Some(name) => entities
                        .projects
                        .into_values()
                        .find(|p| p.name == name)
                        .or(entry.project.clone()),
                    None => entry.project.clone(),
                };

                let tags = match tags {
                    Some(ref t) if t.len() == 1 && t[0].is_empty() => Vec::new(),
                    Some(t) => t,
                    None => entry.tags.clone(),
                };

                let start = parsed_start.unwrap_or(entry.start);
                let (stop, duration) = compute_stop_and_duration(start, parsed_end)?;

                let updated = crate::models::TimeEntry {
                    description: description.unwrap_or(entry.description.clone()),
                    project,
                    tags,
                    start,
                    stop,
                    duration,
                    ..entry
                };

                match api_client.update_time_entry(updated.clone()).await {
                    Err(error) => println!("{}\n{}", "Couldn't update time entry".red(), error),
                    Ok(_) => println!("{}\n{}", "Time entry updated successfully".green(), updated),
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
                    "end must be later than start".to_string(),
                )));
            }
            Ok((Some(end), (end - start).num_seconds()))
        }
        None => Ok((None, -start.timestamp())),
    }
}
