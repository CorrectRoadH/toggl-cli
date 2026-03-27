use crate::api;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;
use std::io::{self, BufWriter, Write};

pub struct RunningTimeEntryCommand;

impl RunningTimeEntryCommand {
    pub async fn execute(api_client: impl ApiClient, json: bool) -> ResultWithDefaultError<()> {
        let current_entry = if json {
            api_client.get_current_time_entry().await?
        } else {
            api_client.get_current_time_entry_minimal().await?
        };
        match current_entry {
            None => {
                if json {
                    let stdout = io::stdout();
                    let mut handle = BufWriter::new(stdout);
                    writeln!(handle, "{{\"running\": false}}").expect("failed to print");
                } else {
                    println!("{}", "No time entry is running at the moment".yellow());
                }
            }
            Some(running_time_entry) => {
                if json {
                    crate::commands::common::CommandUtils::print_time_entry_json(
                        &running_time_entry,
                    );
                } else {
                    println!("{running_time_entry}");
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
    use tokio_test::{assert_err, assert_ok};

    #[tokio::test]
    async fn running_returns_ok_when_no_current_entry_exists() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(|| Ok(None));

        let result = RunningTimeEntryCommand::execute(api_client, false).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn running_returns_error_when_api_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = RunningTimeEntryCommand::execute(api_client, false).await;
        assert_err!(result);
    }
}
