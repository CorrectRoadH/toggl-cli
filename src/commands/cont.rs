use crate::api::client::ApiClient;
use crate::commands;
use crate::models;
use crate::picker;
use chrono::Utc;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::{ResultWithDefaultError, TimeEntry};
use picker::{ItemPicker, PickableItem};

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Option<Box<dyn ItemPicker>>,
    ) -> ResultWithDefaultError<()> {
        let running_time_entry =
            StopCommand::execute(&api_client, StopCommandOrigin::ContinueCommand).await?;

        let entities = api_client.get_entities().await?;
        if entities.time_entries.is_empty() {
            println!("{}", "No time entries in last 90 days".red());
            return Ok(());
        }

        let time_entry_to_continue = match picker {
            None => get_first_stopped_time_entry(entities.time_entries, running_time_entry),
            Some(time_entry_picker) => {
                let pickable_items = entities
                    .time_entries
                    .iter()
                    .map(|te| PickableItem::from_time_entry(te.clone()))
                    .collect();
                let picked_key = time_entry_picker.pick(pickable_items)?;
                let picked_time_entry = entities
                    .time_entries
                    .iter()
                    .find(|te| te.id == picked_key.id)
                    .unwrap();
                Some(picked_time_entry.clone())
            }
        };

        match time_entry_to_continue {
            None => println!("{}", "No time entry to continue".red()),
            Some(time_entry) => {
                let start_time = Utc::now();
                let time_entry_to_create = time_entry.as_running_time_entry(start_time);
                let continued_entry_id = api_client.create_time_entry(time_entry_to_create).await?;
                let entities = api_client.get_entities().await?;
                let continued_entry = entities
                    .time_entries
                    .iter()
                    .find(|te| te.id == continued_entry_id)
                    .unwrap();
                println!(
                    "{}\n{}",
                    "Time entry continued successfully".green(),
                    continued_entry
                )
            }
        }

        Ok(())
    }
}

fn get_first_stopped_time_entry(
    time_entries: Vec<TimeEntry>,
    running_time_entry: Option<TimeEntry>,
) -> Option<TimeEntry> {
    // Don't continue a running entry that was just stopped.
    let continue_entry_index = match running_time_entry {
        None => 0,
        Some(_) => 1,
    };
    time_entries.get(continue_entry_index).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::Entities;
    use std::collections::HashMap;
    use tokio_test::{assert_err, assert_ok};

    fn mock_time_entry(id: i64, description: &str) -> TimeEntry {
        TimeEntry {
            id,
            description: description.to_string(),
            duration: 60,
            stop: Some(Utc::now()),
            ..Default::default()
        }
    }

    fn mock_entities(time_entries: Vec<TimeEntry>) -> Entities {
        Entities {
            time_entries,
            projects: HashMap::new(),
            tasks: HashMap::new(),
            clients: HashMap::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn continue_returns_ok_when_no_time_entries_exist() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry()
            .returning(|| Ok(None));
        api_client
            .expect_get_entities()
            .returning(|| Ok(mock_entities(Vec::new())));

        let result = ContinueCommand::execute(api_client, None).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn continue_creates_new_running_entry_from_latest_stopped_entry() {
        let mut api_client = MockApiClient::new();
        let source_entry = mock_time_entry(7, "Review");
        let continued_entry = TimeEntry {
            id: 99,
            description: "Review".to_string(),
            ..Default::default()
        };
        let source_entry_for_create = source_entry.clone();
        let continued_entry_for_lookup = continued_entry.clone();

        api_client
            .expect_get_current_time_entry()
            .returning(|| Ok(None));
        api_client
            .expect_get_entities()
            .times(2)
            .returning(move || {
                Ok(mock_entities(vec![
                    source_entry.clone(),
                    continued_entry_for_lookup.clone(),
                ]))
            });
        api_client
            .expect_create_time_entry()
            .withf(move |entry| {
                entry.description == source_entry_for_create.description
                    && entry.id == source_entry_for_create.id
                    && entry.stop.is_none()
                    && entry.is_running()
            })
            .returning(|_| Ok(99));

        let result = ContinueCommand::execute(api_client, None).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn continue_returns_error_when_stop_step_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = ContinueCommand::execute(api_client, None).await;
        assert_err!(result);
    }
}
