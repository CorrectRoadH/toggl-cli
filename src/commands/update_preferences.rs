use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use serde_json::Value;

pub struct UpdatePreferencesCommand;

impl UpdatePreferencesCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        payload: String,
    ) -> ResultWithDefaultError<()> {
        let preferences: Value = serde_json::from_str(&payload)
            .map_err(|error| -> Box<dyn std::error::Error + Send> { Box::new(error) })?;
        let updated_preferences = api_client.update_preferences(preferences).await?;
        println!("{}", "Preferences updated successfully".green());
        println!(
            "{}",
            serde_json::to_string_pretty(&updated_preferences)
                .map_err(|error| -> Box<dyn std::error::Error + Send> { Box::new(error) })?
        );
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
    async fn update_preferences_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_update_preferences()
            .withf(|value| value["decimal_format"] == ".")
            .returning(Ok);

        let result =
            UpdatePreferencesCommand::execute(api_client, "{\"decimal_format\":\".\"}".to_string())
                .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn update_preferences_returns_error_on_invalid_json() {
        let api_client = MockApiClient::new();
        let result = UpdatePreferencesCommand::execute(api_client, "{not-json}".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn update_preferences_returns_error_on_api_failure() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_update_preferences()
            .returning(|_| Err(Box::new(ApiError::Network)));

        let result = UpdatePreferencesCommand::execute(
            api_client,
            json!({"time_format":"H:mm"}).to_string(),
        )
        .await;
        assert_err!(result);
    }
}
