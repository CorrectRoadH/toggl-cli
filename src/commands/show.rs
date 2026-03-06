use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use std::io::{self, BufWriter, Write};

pub struct ShowCommand;

impl ShowCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: i64,
        json: bool,
    ) -> ResultWithDefaultError<()> {
        match api_client.get_time_entry(id).await {
            Err(error) => println!(
                "{}\n{}",
                format!("Couldn't fetch time entry with ID {id}").red(),
                error
            ),
            Ok(entry) => {
                let stdout = io::stdout();
                let mut handle = BufWriter::new(stdout);
                if json {
                    let json_string = serde_json::to_string_pretty(&entry)
                        .expect("failed to serialize time entry to JSON");
                    writeln!(handle, "{json_string}").expect("failed to print");
                } else {
                    writeln!(handle, "{}", "Time Entry Details".bold().underline())
                        .expect("failed to print");
                    writeln!(handle, "  {} {}", "ID:".bold(), entry.id).expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Description:".bold(),
                        entry.get_description()
                    )
                    .expect("failed to print");
                    writeln!(handle, "  {} {}", "Start:".bold(), entry.start)
                        .expect("failed to print");
                    match entry.stop {
                        Some(stop) => {
                            writeln!(handle, "  {} {}", "Stop:".bold(), stop)
                                .expect("failed to print");
                        }
                        None => {
                            writeln!(
                                handle,
                                "  {} {}",
                                "Status:".bold(),
                                "Running".green().bold()
                            )
                            .expect("failed to print");
                        }
                    }
                    writeln!(
                        handle,
                        "  {} {}",
                        "Duration:".bold(),
                        entry.get_duration_hmmss()
                    )
                    .expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Billable:".bold(),
                        if entry.billable { "Yes" } else { "No" }
                    )
                    .expect("failed to print");
                    writeln!(
                        handle,
                        "  {} {}",
                        "Workspace ID:".bold(),
                        entry.workspace_id
                    )
                    .expect("failed to print");
                    if !entry.tags.is_empty() {
                        writeln!(handle, "  {} {}", "Tags:".bold(), entry.tags.join(", "))
                            .expect("failed to print");
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::TimeEntry;
    use chrono::Utc;
    use tokio_test::assert_ok;

    fn mock_time_entry() -> TimeEntry {
        TimeEntry {
            id: 42,
            description: "Test entry".to_string(),
            start: Utc::now(),
            stop: Some(Utc::now()),
            duration: 3600,
            billable: true,
            workspace_id: 1,
            tags: vec!["dev".to_string(), "review".to_string()],
            project: None,
            task: None,
            created_with: None,
        }
    }

    #[tokio::test]
    async fn show_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let entry = mock_time_entry();
        api_client
            .expect_get_time_entry()
            .withf(|id| *id == 42)
            .returning(move |_| Ok(entry.clone()));

        let result = ShowCommand::execute(api_client, 42, false).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn show_returns_ok_with_json_flag() {
        let mut api_client = MockApiClient::new();
        let entry = mock_time_entry();
        api_client
            .expect_get_time_entry()
            .returning(move |_| Ok(entry.clone()));

        let result = ShowCommand::execute(api_client, 42, true).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn show_returns_ok_even_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_time_entry()
            .returning(|_| Err(Box::new(ApiError::Network)));

        let result = ShowCommand::execute(api_client, 999, false).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn show_handles_running_entry() {
        let mut api_client = MockApiClient::new();
        let mut entry = mock_time_entry();
        entry.stop = None;
        entry.duration = -Utc::now().timestamp();
        api_client
            .expect_get_time_entry()
            .returning(move |_| Ok(entry.clone()));

        let result = ShowCommand::execute(api_client, 42, false).await;
        assert_ok!(result);
    }
}
