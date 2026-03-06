use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;

pub struct PreferencesCommand;

impl PreferencesCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let preferences = api_client.get_preferences().await?;
        let json = serde_json::to_string_pretty(&preferences)
            .map_err(|error| -> Box<dyn std::error::Error + Send> { Box::new(error) })?;
        println!("{json}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use serde_json::json;
    use tokio_test::{assert_err, assert_ok};

    #[tokio::test]
    async fn preferences_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_preferences()
            .returning(|| Ok(json!({ "decimal_format": "." })));

        let result = PreferencesCommand::execute(api_client).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn preferences_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_preferences()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = PreferencesCommand::execute(api_client).await;
        assert_err!(result);
    }
}
