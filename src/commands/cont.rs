use crate::api::client::ApiClient;
use crate::commands;
use crate::models;
use crate::picker;
use chrono::Utc;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::{ResultWithDefaultError, TimeEntry};
use picker::{ItemPicker, PickableItem};
use std::io::{self, BufWriter, Write};

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Option<Box<dyn ItemPicker>>,
        id: Option<i64>,
        json: bool,
    ) -> ResultWithDefaultError<()> {
        let running_time_entry =
            StopCommand::execute(&api_client, StopCommandOrigin::ContinueCommand, false).await?;

        let time_entry_to_continue = if let Some(entry_id) = id {
            Some(api_client.get_time_entry(entry_id).await?)
        } else {
            let entities = api_client.get_entities().await?;
            if entities.time_entries.is_empty() {
                println!("{}", "No time entries in last 90 days".red());
                return Ok(());
            }

            match picker {
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
            }
        };

        match time_entry_to_continue {
            None => {
                if json {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    writeln!(handle, "null").expect("failed to print");
                } else {
                    println!("{}", "No time entry to continue".red());
                }
            }
            Some(time_entry) => {
                let start_time = Utc::now();
                let time_entry_to_create = time_entry.as_running_time_entry(start_time);
                let continued_entry_id = api_client.create_time_entry(time_entry_to_create).await?;
                let continued_entry = api_client.get_time_entry(continued_entry_id).await?;
                if json {
                    commands::common::CommandUtils::print_time_entry_json(&continued_entry);
                } else {
                    println!(
                        "{}\n{}",
                        "Time entry continued successfully".green(),
                        continued_entry
                    );
                }
            }
        }

        Ok(())
    }
}

fn get_first_stopped_time_entry(
    time_entries: Vec<TimeEntry>,
    running_time_entry: Option<TimeEntry>,
) -> Option<TimeEntry> {
    let just_stopped_id = running_time_entry.map(|entry| entry.id);
    let mut stopped_entries = time_entries
        .into_iter()
        .filter(|entry| entry.stop.is_some())
        .filter(|entry| Some(entry.id) != just_stopped_id)
        .collect::<Vec<_>>();

    stopped_entries.sort_by(|a, b| {
        let a_key = a.stop.unwrap_or(a.start);
        let b_key = b.stop.unwrap_or(b.start);
        b_key.cmp(&a_key)
    });

    stopped_entries.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use tokio_test::assert_err;

    fn mock_time_entry(id: i64, description: &str) -> TimeEntry {
        TimeEntry {
            id,
            description: description.to_string(),
            duration: 60,
            stop: Some(Utc::now()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn continue_returns_error_when_stop_step_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = ContinueCommand::execute(api_client, None, None, false).await;
        assert_err!(result);
    }

    #[test]
    fn get_first_stopped_time_entry_returns_none_for_empty_history() {
        assert!(get_first_stopped_time_entry(Vec::new(), None).is_none());
    }

    #[test]
    fn get_first_stopped_time_entry_uses_latest_stopped_entry_when_nothing_was_running() {
        let now = Utc::now();
        let latest = TimeEntry {
            id: 1,
            description: "Latest".to_string(),
            duration: 60,
            stop: Some(now),
            ..Default::default()
        };
        let older = TimeEntry {
            id: 2,
            description: "Older".to_string(),
            duration: 60,
            stop: Some(now - chrono::Duration::seconds(10)),
            ..Default::default()
        };

        let result = get_first_stopped_time_entry(vec![latest.clone(), older], None);

        assert_eq!(result.unwrap().id, latest.id);
    }

    #[test]
    fn get_first_stopped_time_entry_skips_recently_stopped_running_entry() {
        let just_stopped = mock_time_entry(1, "Just stopped");
        let previous = mock_time_entry(2, "Previous");

        let result = get_first_stopped_time_entry(
            vec![just_stopped, previous.clone()],
            Some(TimeEntry::default()),
        );

        assert_eq!(result.unwrap().id, previous.id);
    }
}
