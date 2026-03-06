use crate::api;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};

pub struct StopCommand;

pub enum StopCommandOrigin {
    CommandLine,
    StartCommand,
    ContinueCommand,
}

impl StopCommand {
    pub async fn execute(
        api_client: &impl ApiClient,
        origin: StopCommandOrigin,
    ) -> ResultWithDefaultError<Option<TimeEntry>> {
        match api_client.get_current_time_entry_minimal().await? {
            None => {
                match origin {
                    StopCommandOrigin::CommandLine => {
                        println!("{}", "No time entry is running at the moment".yellow())
                    }
                    StopCommandOrigin::StartCommand => (),
                    StopCommandOrigin::ContinueCommand => (),
                };

                Ok(None)
            }
            Some(running_time_entry) => {
                let stopped_time_entry = api_client
                    .stop_time_entry(running_time_entry.workspace_id, running_time_entry.id)
                    .await?;

                let message = match origin {
                    StopCommandOrigin::CommandLine => "Time entry stopped successfully".green(),
                    StopCommandOrigin::StartCommand => "Running time entry stopped".yellow(),
                    StopCommandOrigin::ContinueCommand => "Running time entry stopped".yellow(),
                };

                println!("{}\n{}", message, stopped_time_entry.clone());

                Ok(Some(stopped_time_entry))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::TimeEntry;
    use tokio_test::{assert_err, assert_ok};

    #[tokio::test]
    async fn stop_returns_ok_when_no_entry_is_running() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(|| Ok(None));

        let result = StopCommand::execute(&api_client, StopCommandOrigin::CommandLine).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn stop_uses_stop_endpoint_for_running_entry() {
        let mut api_client = MockApiClient::new();
        let current_entry = TimeEntry::default();
        let stopped_entry = current_entry.clone();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(move || Ok(Some(current_entry.clone())));
        api_client
            .expect_stop_time_entry()
            .withf(|workspace_id, time_entry_id| *workspace_id == -1 && *time_entry_id == -1)
            .returning(move |_, _| Ok(stopped_entry.clone()));

        let result = StopCommand::execute(&api_client, StopCommandOrigin::CommandLine).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn stop_returns_error_when_api_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry_minimal()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = StopCommand::execute(&api_client, StopCommandOrigin::CommandLine).await;
        assert_err!(result);
    }
}
