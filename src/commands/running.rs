use crate::api;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct RunningTimeEntryCommand;

impl RunningTimeEntryCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        match api_client.get_current_time_entry().await? {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => println!("{running_time_entry}"),
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
    use tokio_test::{assert_err, assert_ok};

    #[tokio::test]
    async fn running_returns_ok_when_current_entry_exists() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry()
            .returning(|| Ok(Some(TimeEntry::default())));

        let result = RunningTimeEntryCommand::execute(api_client).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn running_returns_ok_when_no_current_entry_exists() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry()
            .returning(|| Ok(None));

        let result = RunningTimeEntryCommand::execute(api_client).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn running_returns_error_when_api_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_current_time_entry()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = RunningTimeEntryCommand::execute(api_client).await;
        assert_err!(result);
    }
}
