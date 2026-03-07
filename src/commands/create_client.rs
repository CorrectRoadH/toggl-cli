use crate::api::client::ApiClient;
use crate::commands::common::CommandUtils;
use crate::models::ResultWithDefaultError;

pub struct CreateClientCommand;

impl CreateClientCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = CommandUtils::get_workspace_id(&api_client).await?;
        let client = api_client.create_client(workspace_id, name).await?;
        CommandUtils::print_creation_success("Client", &client);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error::ApiError;
    use crate::models::{Client, User};
    use tokio_test::{assert_err, assert_ok};

    fn mock_user() -> User {
        User {
            api_token: "token".to_string(),
            email: "test@example.com".to_string(),
            fullname: Some("Test".to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1,
            beginning_of_week: None,
            image_url: None,
            created_at: None,
            updated_at: None,
            country_id: None,
            has_password: None,
        }
    }

    #[tokio::test]
    async fn create_client_returns_ok_on_success() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_client()
            .withf(|wid, name| *wid == 1 && name == "Acme Corp")
            .returning(|wid, name| {
                Ok(Client {
                    id: 100,
                    name,
                    workspace_id: wid,
                })
            });

        let result = CreateClientCommand::execute(api_client, "Acme Corp".to_string()).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn create_client_returns_ok_on_api_failure() {
        let mut api_client = MockApiClient::new();
        let user = mock_user();
        api_client
            .expect_get_user()
            .returning(move || Ok(user.clone()));
        api_client
            .expect_create_client()
            .returning(|_, _| Err(Box::new(ApiError::Network)));

        let result = CreateClientCommand::execute(api_client, "Fail".to_string()).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn create_client_returns_error_when_user_fetch_fails() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_user()
            .returning(|| Err(Box::new(ApiError::Network)));

        let result = CreateClientCommand::execute(api_client, "Test".to_string()).await;
        assert_err!(result);
    }
}
